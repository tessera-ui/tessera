//! Material Design snackbars for transient feedback messages.
//!
//! ## Usage
//!
//! Show brief status updates with optional actions at the bottom of a screen.

use std::{collections::VecDeque, time::Duration};

use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    Callback, Color, Dp, FocusRestorationTarget, FocusScopeNode, Modifier, RenderSlotWith, State,
    accesskit::{Live, Role},
    focus, launch_once,
    layout::{Stack, layout},
    modifier::FocusModifierExt as _,
    provide_context, remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::Alignment,
    animation::{Delay, run_frames},
    md3::{
        button::button,
        icon_button::{IconButtonVariant, icon_button},
        material_icons::filled,
        surface::surface,
        text::text,
        theme::MaterialTheme,
    },
    modifier::with_keyboard_input,
    res::I18N,
    shape_def::Shape,
};

const SHORT_SNACKBAR_DURATION: Duration = Duration::from_millis(4_000);
const LONG_SNACKBAR_DURATION: Duration = Duration::from_millis(10_000);

/// Possible results of a snackbar being shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnackbarResult {
    /// The snackbar was dismissed (timeout or user dismissal).
    Dismissed,
    /// The snackbar action button was invoked.
    ActionPerformed,
}

/// Duration options for snackbars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnackbarDuration {
    /// Show for a short period.
    #[default]
    Short,
    /// Show for a long period.
    Long,
    /// Show until dismissed explicitly.
    Indefinite,
}

impl SnackbarDuration {
    fn timeout(self) -> Option<Duration> {
        match self {
            SnackbarDuration::Short => Some(SHORT_SNACKBAR_DURATION),
            SnackbarDuration::Long => Some(LONG_SNACKBAR_DURATION),
            SnackbarDuration::Indefinite => None,
        }
    }

    fn default_for_action(has_action: bool) -> Self {
        if has_action {
            SnackbarDuration::Indefinite
        } else {
            SnackbarDuration::Short
        }
    }
}

/// Request data for showing a snackbar with default behavior.
#[derive(Clone, Debug)]
pub struct SnackbarRequest {
    /// Primary message shown in the snackbar.
    pub message: String,
    /// Optional label for the action button.
    pub action_label: Option<String>,
    /// Whether a dismiss action should be shown.
    pub with_dismiss_action: bool,
    /// Optional duration override for the snackbar.
    pub duration: Option<SnackbarDuration>,
}

impl SnackbarRequest {
    /// Creates a new request with the required message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action_label: None,
            with_dismiss_action: false,
            duration: None,
        }
    }
}

impl From<String> for SnackbarRequest {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for SnackbarRequest {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

#[derive(Clone, PartialEq, Debug)]
struct ResolvedSnackbar {
    message: String,
    action_label: Option<String>,
    with_dismiss_action: bool,
    duration: SnackbarDuration,
}

impl From<SnackbarRequest> for ResolvedSnackbar {
    fn from(request: SnackbarRequest) -> Self {
        let action_label = request.action_label.filter(|label| !label.is_empty());
        let has_action = action_label.is_some();
        let duration = request
            .duration
            .unwrap_or_else(|| SnackbarDuration::default_for_action(has_action));
        Self {
            message: request.message,
            action_label,
            with_dismiss_action: request.with_dismiss_action,
            duration,
        }
    }
}

/// Data describing the current snackbar shown by [`snackbar_host`].
#[derive(Clone, PartialEq)]
pub struct SnackbarData {
    message: String,
    action_label: Option<String>,
    with_dismiss_action: bool,
    duration: SnackbarDuration,
    host_state: State<SnackbarHostController>,
    id: u64,
}

impl SnackbarData {
    fn new(record: SnackbarRecord, host_state: State<SnackbarHostController>) -> Self {
        let SnackbarRecord { id, resolved } = record;
        let ResolvedSnackbar {
            message,
            action_label,
            with_dismiss_action,
            duration,
        } = resolved;
        Self {
            message,
            action_label,
            with_dismiss_action,
            duration,
            host_state,
            id,
        }
    }

    /// Returns the message displayed in the snackbar.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the action label displayed in the snackbar, if any.
    pub fn action_label(&self) -> Option<&str> {
        self.action_label.as_deref()
    }

    /// Returns whether a dismiss action is shown.
    pub fn with_dismiss_action(&self) -> bool {
        self.with_dismiss_action
    }

    /// Returns the resolved duration for the snackbar.
    pub fn duration(&self) -> SnackbarDuration {
        self.duration
    }

    /// Report that the snackbar action was performed.
    pub fn perform_action(&self) {
        self.host_state.with_mut(|state| {
            state.resolve_current(self.id, SnackbarResult::ActionPerformed);
        });
    }

    /// Dismiss the snackbar.
    pub fn dismiss(&self) {
        self.host_state.with_mut(|state| {
            state.resolve_current(self.id, SnackbarResult::Dismissed);
        });
    }
}

#[derive(Clone, PartialEq)]
struct SnackbarRecord {
    id: u64,
    resolved: ResolvedSnackbar,
}

/// State container for managing snackbar queues.
pub struct SnackbarHostController {
    queue: VecDeque<SnackbarRecord>,
    current: Option<SnackbarRecord>,
    timeout: Option<Delay>,
    next_id: u64,
    last_result: Option<SnackbarResult>,
}

impl SnackbarHostController {
    /// Creates a new empty snackbar host state.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            timeout: None,
            next_id: 1,
            last_result: None,
        }
    }

    /// Enqueue a snackbar with default behavior.
    ///
    /// Returns the unique snackbar id.
    pub fn show_snackbar(&mut self, request: impl Into<SnackbarRequest>) -> u64 {
        let resolved: ResolvedSnackbar = request.into().into();
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.queue.push_back(SnackbarRecord { id, resolved });
        if self.current.is_none() {
            self.advance_queue();
        }
        id
    }

    /// Returns whether a snackbar is currently visible.
    pub fn is_showing(&self) -> bool {
        self.current.is_some()
    }

    /// Dismisses the current snackbar, if any.
    pub fn dismiss_current(&mut self) {
        if let Some(current) = &self.current {
            self.resolve_current(current.id, SnackbarResult::Dismissed);
        }
    }

    /// Returns the last snackbar result and clears it.
    pub fn take_last_result(&mut self) -> Option<SnackbarResult> {
        self.last_result.take()
    }

    /// Advances the current snackbar's timeout to `now` and returns whether
    /// a timeout is still pending.
    fn advance(&mut self, now: Instant) -> bool {
        if let Some(timeout) = &mut self.timeout
            && timeout.elapsed(now)
            && let Some(id) = self.current.as_ref().map(|current| current.id)
        {
            self.resolve_current(id, SnackbarResult::Dismissed);
        }
        self.timeout.is_some()
    }
    fn resolve_current(&mut self, id: u64, result: SnackbarResult) -> bool {
        if self.current.as_ref().map(|record| record.id) != Some(id) {
            return false;
        }
        self.last_result = Some(result);
        self.advance_queue();
        true
    }

    fn advance_queue(&mut self) {
        self.current = self.queue.pop_front();
        self.timeout = self
            .current
            .as_ref()
            .and_then(|current| current.resolved.duration.timeout())
            .map(|duration| Delay::new(None, duration));
    }
}

impl Default for SnackbarHostController {
    fn default() -> Self {
        Self::new()
    }
}

/// Default values used by snackbars.
pub struct SnackbarDefaults;

impl SnackbarDefaults {
    /// Maximum width for snackbar containers.
    pub const MAX_WIDTH: Dp = Dp(600.0);
    /// Minimum height for single-line snackbars.
    pub const MIN_HEIGHT_ONE_LINE: Dp = Dp(48.0);
    /// Minimum height for multi-line snackbars.
    pub const MIN_HEIGHT_TWO_LINE: Dp = Dp(68.0);
    /// Default container elevation.
    pub const CONTAINER_ELEVATION: Dp = Dp(6.0);
    /// Default padding around snackbar message text.
    pub const CONTENT_PADDING: Padding = Padding::symmetric(Dp(16.0), Dp(14.0));
    /// Default padding applied when rendering snackbars from host data.
    pub const HOST_PADDING: Padding = Padding::all(Dp(12.0));
    /// Horizontal spacing between text and actions.
    pub const ACTION_SPACING: Dp = Dp(8.0);
    /// Bottom spacing below actions placed on a separate line.
    pub const ACTION_VERTICAL_SPACING: Dp = Dp(4.0);

    /// Default snackbar shape.
    pub fn shape() -> crate::shape_def::Shape {
        crate::shape_def::Shape::RoundedRectangle {
            top_left: crate::shape_def::RoundedCorner::manual(Dp(4.0), 3.0),
            top_right: crate::shape_def::RoundedCorner::manual(Dp(4.0), 3.0),
            bottom_right: crate::shape_def::RoundedCorner::manual(Dp(4.0), 3.0),
            bottom_left: crate::shape_def::RoundedCorner::manual(Dp(4.0), 3.0),
        }
    }

    /// Default container color derived from the current theme.
    pub fn container_color() -> Color {
        Color::new(0.1, 0.1, 0.1, 1.0)
    }

    /// Default content color derived from the current theme.
    pub fn content_color() -> Color {
        Color::new(0.95, 0.95, 0.95, 1.0)
    }

    /// Default action label color derived from the current theme.
    pub fn action_color() -> Color {
        Color::new(0.7, 0.8, 1.0, 1.0)
    }

    /// Default dismiss action color derived from the current theme.
    pub fn dismiss_action_color() -> Color {
        Color::new(0.95, 0.95, 0.95, 1.0)
    }

    /// Returns the minimum height for explicit message lines and action
    /// placement.
    ///
    /// Measured wrapping, larger typography and taller actions may require
    /// more.
    pub fn min_height(message: &str, action_on_new_line: bool) -> Dp {
        let message_height = if message.contains('\n') {
            Self::MIN_HEIGHT_TWO_LINE
        } else {
            Self::MIN_HEIGHT_ONE_LINE
        };
        if action_on_new_line {
            Dp(message_height.0 + 48.0 + Self::ACTION_VERTICAL_SPACING.0)
        } else {
            message_height
        }
    }
}

/// # snackbar
///
/// Show a brief message with optional action and dismiss controls.
///
/// ## Usage
///
/// Display transient feedback or status updates that do not interrupt the
/// workflow.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the snackbar container.
/// - `message` — message shown in the snackbar.
/// - `action_label` — optional label for the action button.
/// - `with_dismiss_action` — whether to show a dismiss action icon.
/// - `action_on_new_line` — whether the action should be placed on a new line.
/// - `shape` — optional shape override.
/// - `container_color` — optional container color override.
/// - `content_color` — optional message color override.
/// - `action_color` — optional action color override.
/// - `dismiss_action_color` — optional dismiss action color override.
/// - `content_padding` — optional message insets; vertical padding excludes
///   action controls. Explicit horizontal insets also bound the action row.
/// - `dismiss_action_label` — accessible close-button name; defaults to
///   Dismiss.
/// - `on_action` — optional action callback.
/// - `on_dismiss` — optional dismiss callback, also invoked when Escape is
///   pressed while one of the snackbar controls has focus.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::snackbar::snackbar;
/// use tessera_components::md3::theme::{MaterialTheme, material_theme};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             snackbar()
///                 .message("Saved")
///                 .action_label("Undo")
///                 .with_dismiss_action(true)
///                 .on_action(|| {})
///                 .on_dismiss(|| {});
///         });
/// }
///
/// # tessera_ui::testing::UiTest::new().content(demo).frame();
/// ```
#[tessera]
pub fn snackbar(
    modifier: Option<Modifier>,
    #[prop(into)] message: Option<String>,
    #[prop(into)] action_label: Option<String>,
    with_dismiss_action: Option<bool>,
    action_on_new_line: Option<bool>,
    shape: Option<Shape>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    action_color: Option<Color>,
    dismiss_action_color: Option<Color>,
    #[prop(into)] dismiss_action_label: Option<String>,
    content_padding: Option<Padding>,
    on_action: Option<Callback>,
    on_dismiss: Option<Callback>,
) {
    let modifier = modifier.unwrap_or_default();
    let message = message.unwrap_or_default();
    let with_dismiss_action = with_dismiss_action.unwrap_or(false);
    let action_on_new_line = action_on_new_line.unwrap_or(false);
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let scheme = theme.color_scheme;
    let shape = shape.unwrap_or(theme.shapes.extra_small);
    let container_color = container_color.unwrap_or(scheme.inverse_surface);
    let content_color = content_color.unwrap_or(scheme.inverse_on_surface);
    let action_color = action_color.unwrap_or(scheme.inverse_primary);
    let dismiss_action_color = dismiss_action_color.unwrap_or(scheme.inverse_on_surface);
    let dismiss_action_label =
        dismiss_action_label.unwrap_or_else(|| I18N.text("md3-dismiss").into());
    let action_label = action_label.filter(|label| !label.is_empty());
    let has_action = action_label.is_some();
    let action_on_new_line = action_on_new_line && has_action;
    let modifier = modifier.size_in(None, Some(SnackbarDefaults::MAX_WIDTH), None, None);
    let on_action = if has_action {
        Some(on_action.unwrap_or_default())
    } else {
        None
    };
    let show_dismiss_action = with_dismiss_action || on_dismiss.is_some();
    let on_dismiss = if show_dismiss_action {
        Some(on_dismiss.unwrap_or_default())
    } else {
        None
    };
    let modifier = if let Some(on_dismiss) = on_dismiss {
        with_keyboard_input(modifier, move |input: tessera_ui::KeyboardInput<'_>| {
            input
                .keyboard_events
                .retain(|event| !dismiss_key(event.physical_key, event.state, on_dismiss));
        })
    } else {
        modifier
    };

    surface()
        .modifier(modifier)
        .style(container_color.into())
        .shape(shape)
        .content_alignment(if action_on_new_line {
            Alignment::TopStart
        } else {
            Alignment::CenterStart
        })
        .block_input(true)
        .content_color(content_color)
        .elevation(SnackbarDefaults::CONTAINER_ELEVATION)
        .child(move || {
            let message = message.clone();
            let action_label = action_label.clone();
            let dismiss_action_label = dismiss_action_label.clone();
            tessera_ui::layout::layout(snackbar_layout::SnackbarLayout {
                stacked: action_on_new_line,
                action: has_action,
                dismiss: show_dismiss_action,
                padding: content_padding,
            })
            .child(move || {
                provide_context(typography.body_medium, {
                    let message = message.clone();
                    move || {
                        text(message.clone())
                            .modifier(Modifier::new().semantics(SemanticsArgs {
                                role: Some(Role::Status),
                                live: Some(Live::Polite),
                                ..Default::default()
                            }))
                            .color(content_color);
                    }
                });
                if let Some(label) = action_label.clone() {
                    button()
                        .text()
                        .accessibility_label(label.clone())
                        .on_click_shared(on_action.unwrap_or_default())
                        .content_color(action_color)
                        .ripple_color(action_color)
                        .child(move || {
                            text(label.clone()).color(action_color);
                        });
                }
                if let Some(on_dismiss) = on_dismiss {
                    icon_button()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label(dismiss_action_label.clone())
                        .variant(IconButtonVariant::Standard)
                        .content_color(dismiss_action_color)
                        .on_click_shared(on_dismiss);
                }
            });
        });
}

fn dismiss_key(
    key: tessera_ui::winit::keyboard::PhysicalKey,
    state: tessera_ui::winit::event::ElementState,
    on_dismiss: Callback,
) -> bool {
    if state == tessera_ui::winit::event::ElementState::Pressed
        && key
            == tessera_ui::winit::keyboard::PhysicalKey::Code(
                tessera_ui::winit::keyboard::KeyCode::Escape,
            )
    {
        on_dismiss.call();
        true
    } else {
        false
    }
}

/// # snackbar_host
///
/// Display queued snackbars driven by a [`SnackbarHostController`].
///
/// ## Usage
///
/// Use with [`scaffold`](crate::md3::scaffold::scaffold) to show transient messages
/// above app content.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the host container.
/// - `controller` — state that provides snackbar queue data.
/// - `snackbar` — optional custom snackbar slot for rendering.
///
/// Appearance preserves the current focus. When the final notification closes
/// while its controls own focus, the host restores the previously focused
/// target after layout if that target remains available. Focus moved outside
/// the host is preserved.
///
/// ## Examples
///
/// ```
/// use tessera_components::md3::snackbar::{SnackbarHostController, snackbar_host};
/// use tessera_components::md3::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let host_state = remember(SnackbarHostController::new);
///             host_state.with_mut(|state| {
///                 state.show_snackbar("Saved");
///             });
///             snackbar_host().controller(host_state);
///             assert!(host_state.with(|state| state.is_showing()));
///         });
/// }
///
/// # tessera_ui::testing::UiTest::new().content(demo).frame();
/// ```
#[tessera]
pub fn snackbar_host(
    modifier: Option<Modifier>,
    controller: Option<State<SnackbarHostController>>,
    snackbar: Option<RenderSlotWith<SnackbarData>>,
) {
    let modifier = modifier.unwrap_or_default();
    let state = match controller {
        Some(value) => value,
        None => remember(SnackbarHostController::default),
    };
    let snackbar_slot = snackbar;
    let record = state.with(|host| host.current.clone());
    if state.with(|host| host.timeout.is_some()) {
        launch_once(run_frames(state, SnackbarHostController::advance));
    }
    let scope = remember(FocusScopeNode::new).get();
    let showing = remember(|| false);
    let origin = remember(|| None::<FocusRestorationTarget>);
    let pending_restore = remember(|| None::<FocusRestorationTarget>);
    let visible = record.is_some();
    if visible != showing.get() {
        if visible {
            origin.set(focus::save_focus());
        } else {
            if scope.has_focus() {
                pending_restore.set(origin.get());
                focus::defer_keyboard_until_layout();
            }
            origin.set(None);
        }
        showing.set(visible);
    }
    let mut modifier = modifier.focus_scope(Some(scope));
    if pending_restore.get().is_some() {
        modifier = modifier.on_globally_positioned(move |_| {
            if let Some(target) = pending_restore.with_mut(Option::take) {
                target.restore();
            }
        });
    }
    let data = record.map(|record| SnackbarData::new(record, state));
    layout(Stack).modifier(modifier).child(move || {
        let Some(data) = data.clone() else {
            return;
        };
        if let Some(snackbar_slot) = snackbar_slot {
            snackbar_slot.render(data.clone());
        } else {
            layout(Stack)
                .modifier(Modifier::new().padding(SnackbarDefaults::HOST_PADDING))
                .child(move || {
                    snackbar_from_data().data(data.clone());
                });
        }
    });
}

#[tessera]
fn snackbar_from_data(data: Option<SnackbarData>) {
    let Some(data) = data else {
        return;
    };
    let current = remember(|| data.clone());
    if current.get() != data {
        current.set(data.clone());
    }
    let on_action = if data.action_label.is_some() {
        Some(Callback::new(move || current.get().perform_action()))
    } else {
        None
    };
    let on_dismiss = if data.with_dismiss_action {
        Some(Callback::new(move || current.get().dismiss()))
    } else {
        None
    };
    snackbar()
        .message(data.message)
        .with_dismiss_action(data.with_dismiss_action)
        .action_label_optional(data.action_label)
        .on_action_optional(on_action)
        .on_dismiss_optional(on_dismiss);
}

#[cfg(test)]
mod accessibility_tests;

#[cfg(test)]
mod host_tests;

mod snackbar_layout;

#[cfg(test)]
mod geometry_tests;
