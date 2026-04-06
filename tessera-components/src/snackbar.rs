//! Material Design snackbars for transient feedback messages.
//!
//! ## Usage
//!
//! Show brief status updates with optional actions at the bottom of a screen.

use std::{collections::VecDeque, time::Duration};

use tessera_ui::{
    Callback, CallbackWith, Color, Dp, Modifier, State, current_frame_nanos, layout::layout,
    receive_frame_nanos, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    icon_button::{IconButtonVariant, icon_button},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, provide_text_style},
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

/// Data describing the current snackbar shown by a [`SnackbarHost`].
#[derive(Clone, PartialEq)]
pub struct SnackbarData {
    message: String,
    action_label: Option<String>,
    with_dismiss_action: bool,
    duration: SnackbarDuration,
    host_state: State<SnackbarHostState>,
    id: u64,
}

impl SnackbarData {
    fn new(record: SnackbarRecord, host_state: State<SnackbarHostState>) -> Self {
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
pub struct SnackbarHostState {
    queue: VecDeque<SnackbarRecord>,
    current: Option<SnackbarRecord>,
    current_started_frame_nanos: Option<u64>,
    next_id: u64,
    last_result: Option<SnackbarResult>,
}

impl SnackbarHostState {
    /// Creates a new empty snackbar host state.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            current_started_frame_nanos: None,
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

    fn poll(&mut self, frame_nanos: u64) -> Option<SnackbarRecord> {
        if self.current.is_none() {
            self.advance_queue();
        }

        if self.current.is_some() && self.current_started_frame_nanos.is_none() {
            self.current_started_frame_nanos = Some(frame_nanos);
        }

        let mut should_dismiss = false;
        if let Some(current) = &self.current
            && let Some(timeout) = current.resolved.duration.timeout()
            && let Some(started_frame_nanos) = self.current_started_frame_nanos
        {
            let elapsed_nanos = frame_nanos.saturating_sub(started_frame_nanos);
            let timeout_nanos = timeout.as_nanos().min(u64::MAX as u128) as u64;
            if elapsed_nanos >= timeout_nanos {
                should_dismiss = true;
            }
        }

        if should_dismiss && let Some(current) = &self.current {
            self.resolve_current(current.id, SnackbarResult::Dismissed);
        }

        self.current.clone()
    }

    fn has_pending_timeout(&self, frame_nanos: u64) -> bool {
        let Some(current) = &self.current else {
            return false;
        };
        let Some(timeout) = current.resolved.duration.timeout() else {
            return false;
        };

        self.current_started_frame_nanos
            .map(|started_frame_nanos| {
                let elapsed_nanos = frame_nanos.saturating_sub(started_frame_nanos);
                let timeout_nanos = timeout.as_nanos().min(u64::MAX as u128) as u64;
                elapsed_nanos < timeout_nanos
            })
            .unwrap_or(true)
    }

    fn should_dismiss_current_timeout(&self, frame_nanos: u64) -> bool {
        let Some(current) = &self.current else {
            return false;
        };
        let Some(timeout) = current.resolved.duration.timeout() else {
            return false;
        };
        let Some(started_frame_nanos) = self.current_started_frame_nanos else {
            return false;
        };

        let elapsed_nanos = frame_nanos.saturating_sub(started_frame_nanos);
        let timeout_nanos = timeout.as_nanos().min(u64::MAX as u128) as u64;
        elapsed_nanos >= timeout_nanos && self.current.as_ref().map(|r| r.id) == Some(current.id)
    }

    fn should_poll(&self, frame_nanos: u64) -> bool {
        if self.current.is_none() {
            return !self.queue.is_empty();
        }

        if self.current_started_frame_nanos.is_none() {
            return true;
        }

        self.should_dismiss_current_timeout(frame_nanos)
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
        self.current_started_frame_nanos = None;
    }
}

impl Default for SnackbarHostState {
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
    /// Default padding applied to snackbar content.
    pub const CONTENT_PADDING: Padding = Padding::symmetric(Dp(16.0), Dp(6.0));
    /// Default padding applied when rendering snackbars from host data.
    pub const HOST_PADDING: Padding = Padding::all(Dp(12.0));
    /// Horizontal spacing between text and actions.
    pub const ACTION_SPACING: Dp = Dp(8.0);
    /// Vertical spacing between text and actions when stacked.
    pub const ACTION_VERTICAL_SPACING: Dp = Dp(2.0);

    /// Default snackbar shape.
    pub fn shape() -> crate::shape_def::Shape {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .shapes
            .extra_small
    }

    /// Default container color derived from the current theme.
    pub fn container_color() -> Color {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .inverse_surface
    }

    /// Default content color derived from the current theme.
    pub fn content_color() -> Color {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .inverse_on_surface
    }

    /// Default action label color derived from the current theme.
    pub fn action_color() -> Color {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .inverse_primary
    }

    /// Default dismiss action color derived from the current theme.
    pub fn dismiss_action_color() -> Color {
        use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme
            .inverse_on_surface
    }

    /// Computes the minimum container height based on content.
    pub fn min_height(message: &str, action_on_new_line: bool) -> Dp {
        if action_on_new_line || message.contains('\n') {
            Self::MIN_HEIGHT_TWO_LINE
        } else {
            Self::MIN_HEIGHT_ONE_LINE
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
/// - `content_padding` — optional content padding override.
/// - `on_action` — optional action callback.
/// - `on_dismiss` — optional dismiss callback.
///
/// ## Examples
///
/// ```
/// use tessera_components::snackbar::snackbar;
/// use tessera_components::theme::{MaterialTheme, material_theme};
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
/// demo();
/// ```
#[tessera]
pub fn snackbar(
    modifier: Modifier,
    #[prop(into)] message: String,
    #[prop(into)] action_label: Option<String>,
    with_dismiss_action: bool,
    action_on_new_line: bool,
    shape: Option<Shape>,
    container_color: Option<Color>,
    content_color: Option<Color>,
    action_color: Option<Color>,
    dismiss_action_color: Option<Color>,
    content_padding: Option<Padding>,
    on_action: Option<Callback>,
    on_dismiss: Option<Callback>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let shape = shape.unwrap_or_else(SnackbarDefaults::shape);
    let container_color = container_color.unwrap_or_else(SnackbarDefaults::container_color);
    let content_color = content_color.unwrap_or_else(SnackbarDefaults::content_color);
    let action_color = action_color.unwrap_or_else(SnackbarDefaults::action_color);
    let dismiss_action_color =
        dismiss_action_color.unwrap_or_else(SnackbarDefaults::dismiss_action_color);
    let content_padding = content_padding.unwrap_or(SnackbarDefaults::CONTENT_PADDING);
    let action_label = action_label.filter(|label| !label.is_empty());
    let has_action = action_label.is_some();
    let action_on_new_line = action_on_new_line && has_action;
    let min_height = SnackbarDefaults::min_height(&message, action_on_new_line);
    let modifier = modifier.size_in(
        None,
        Some(SnackbarDefaults::MAX_WIDTH),
        Some(min_height),
        None,
    );
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
    let row_padding = if show_dismiss_action && !action_on_new_line {
        Padding::new(
            content_padding.left,
            content_padding.top,
            Dp(0.0),
            content_padding.bottom,
        )
    } else {
        content_padding
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
            if action_on_new_line {
                render_snackbar_column(SnackbarRenderArgs {
                    message,
                    message_style: typography.body_medium,
                    message_color: content_color,
                    action_label,
                    action_color,
                    dismiss_action_color,
                    on_action,
                    on_dismiss,
                    padding: content_padding,
                });
            } else {
                render_snackbar_row(SnackbarRenderArgs {
                    message,
                    message_style: typography.body_medium,
                    message_color: content_color,
                    action_label,
                    action_color,
                    dismiss_action_color,
                    on_action,
                    on_dismiss,
                    padding: row_padding,
                });
            }
        });
}

/// # snackbar_host
///
/// Display queued snackbars driven by a [`SnackbarHostState`].
///
/// ## Usage
///
/// Use with [`scaffold`](crate::scaffold::scaffold) to show transient messages
/// above app content.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the host container.
/// - `state` — state that provides snackbar queue data.
/// - `snackbar` — optional custom snackbar slot for rendering.
///
/// ## Examples
///
/// ```
/// use tessera_components::snackbar::{SnackbarHostState, snackbar_host};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let host_state = remember(SnackbarHostState::new);
///             host_state.with_mut(|state| {
///                 state.show_snackbar("Saved");
///             });
///             snackbar_host().state(host_state);
///             assert!(host_state.with(|state| state.is_showing()));
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn snackbar_host(
    modifier: Modifier,
    state: Option<State<SnackbarHostState>>,
    snackbar: Option<CallbackWith<SnackbarData>>,
) {
    let state = state.expect("snackbar_host requires state to be set");
    let snackbar_slot = snackbar;
    let frame_nanos = current_frame_nanos();
    let should_poll = state.with(|host| host.should_poll(frame_nanos));
    let record = if should_poll {
        state.with_mut(|host| host.poll(frame_nanos))
    } else {
        state.with(|host| host.current.clone())
    };
    if state.with(|host| host.has_pending_timeout(frame_nanos)) {
        receive_frame_nanos(move |frame_nanos| {
            let should_dismiss =
                state.with(|host| host.should_dismiss_current_timeout(frame_nanos));
            if should_dismiss {
                state.with_mut(|host| {
                    let _ = host.poll(frame_nanos);
                });
            }
            let has_pending_timeout = state.with(|host| host.has_pending_timeout(frame_nanos));
            if has_pending_timeout {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let Some(record) = record else {
        return;
    };
    let data = SnackbarData::new(record, state);

    layout().modifier(modifier).child(move || {
        let data = data.clone();
        if let Some(snackbar_slot) = snackbar_slot {
            snackbar_slot.call(data.clone());
        } else {
            layout()
                .modifier(Modifier::new().padding(SnackbarDefaults::HOST_PADDING))
                .child(move || {
                    snackbar_from_data(data.clone());
                });
        }
    });
}

#[derive(Clone)]
struct SnackbarRenderArgs {
    message: String,
    message_style: crate::theme::TextStyle,
    message_color: Color,
    action_label: Option<String>,
    action_color: Color,
    dismiss_action_color: Color,
    on_action: Option<Callback>,
    on_dismiss: Option<Callback>,
    padding: Padding,
}

fn render_snackbar_row(args: SnackbarRenderArgs) {
    row()
        .modifier(Modifier::new().fill_max_width().padding(args.padding))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            let message_text = args.message.clone();
            boxed()
                .alignment(Alignment::CenterStart)
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    let message_text = message_text.clone();
                    render_message(message_text.clone(), args.message_style, args.message_color);
                });

            if let Some(label) = args.action_label.clone() {
                spacer().modifier(Modifier::new().width(SnackbarDefaults::ACTION_SPACING));
                render_action_button(label.clone(), args.action_color, args.on_action);
            }

            if args.on_dismiss.is_some() {
                spacer().modifier(Modifier::new().width(SnackbarDefaults::ACTION_SPACING));
                render_dismiss_button(args.dismiss_action_color, args.on_dismiss);
            }
        });
}

fn render_snackbar_column(args: SnackbarRenderArgs) {
    column()
        .modifier(Modifier::new().fill_max_width().padding(args.padding))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .children(move || {
            let message_text = args.message.clone();
            render_message(message_text.clone(), args.message_style, args.message_color);

            if args.action_label.is_some() || args.on_dismiss.is_some() {
                spacer()
                    .modifier(Modifier::new().height(SnackbarDefaults::ACTION_VERTICAL_SPACING));
                let action_label = args.action_label.clone();
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .main_axis_alignment(MainAxisAlignment::End)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if let Some(label) = action_label.clone() {
                            render_action_button(label.clone(), args.action_color, args.on_action);
                        }

                        if args.on_dismiss.is_some() {
                            spacer()
                                .modifier(Modifier::new().width(SnackbarDefaults::ACTION_SPACING));
                            render_dismiss_button(args.dismiss_action_color, args.on_dismiss);
                        }
                    });
            }
        });
}

fn render_message(message: String, style: crate::theme::TextStyle, color: Color) {
    provide_text_style(style, move || {
        text().content(message.clone()).color(color);
    });
}

fn render_action_button(label: String, action_color: Color, on_action: Option<Callback>) {
    let on_action = on_action.unwrap_or_default();
    button()
        .text()
        .on_click(move || {
            on_action.call();
        })
        .content_color(action_color)
        .ripple_color(action_color)
        .child(move || {
            text().content(label.clone()).color(action_color);
        });
}

fn render_dismiss_button(dismiss_color: Color, on_dismiss: Option<Callback>) {
    icon_button()
        .icon(filled::CLOSE_SVG)
        .variant(IconButtonVariant::Standard)
        .content_color(dismiss_color)
        .on_click_shared(on_dismiss.unwrap_or_default());
}

fn snackbar_from_data(data: SnackbarData) {
    let SnackbarData {
        message,
        action_label,
        with_dismiss_action,
        host_state,
        id,
        ..
    } = data;

    let mut builder = snackbar()
        .message(message)
        .with_dismiss_action(with_dismiss_action);

    if let Some(action_label) = action_label.clone() {
        builder = builder.action_label(action_label);
    }

    if action_label.is_some() {
        builder = builder.on_action_shared(Callback::new(move || {
            host_state.with_mut(|state| {
                state.resolve_current(id, SnackbarResult::ActionPerformed);
            });
        }));
    }

    if with_dismiss_action {
        builder = builder.on_dismiss_shared(Callback::new(move || {
            host_state.with_mut(|state| {
                state.resolve_current(id, SnackbarResult::Dismissed);
            });
        }));
    }

    drop(builder);
}
