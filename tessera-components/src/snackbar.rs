//! Material Design snackbars for transient feedback messages.
//!
//! ## Usage
//!
//! Show brief status updates with optional actions at the bottom of a screen.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, Modifier, State, tessera, use_context, with_frame_nanos,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    icon::IconArgs,
    icon_button::{IconButtonArgs, IconButtonVariant, icon_button},
    image_vector::TintMode,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding},
    row::{RowArgs, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
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
#[derive(Clone, PartialEq, Debug, Setters)]
pub struct SnackbarRequest {
    /// Primary message shown in the snackbar.
    #[setters(into)]
    pub message: String,
    /// Optional label for the action button.
    #[setters(strip_option, into)]
    pub action_label: Option<String>,
    /// Whether a dismiss action should be shown.
    pub with_dismiss_action: bool,
    /// Optional duration override for the snackbar.
    #[setters(strip_option)]
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
    current_started_at: Option<Instant>,
    next_id: u64,
    last_result: Option<SnackbarResult>,
}

impl SnackbarHostState {
    /// Creates a new empty snackbar host state.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            current_started_at: None,
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

    fn poll(&mut self, now: Instant) -> Option<SnackbarRecord> {
        if self.current.is_none() {
            self.advance_queue();
        }

        if self.current.is_some() && self.current_started_at.is_none() {
            self.current_started_at = Some(now);
        }

        let mut should_dismiss = false;
        if let Some(current) = &self.current
            && let Some(timeout) = current.resolved.duration.timeout()
            && let Some(started_at) = self.current_started_at
            && now.duration_since(started_at) >= timeout
        {
            should_dismiss = true;
        }

        if should_dismiss && let Some(current) = &self.current {
            self.resolve_current(current.id, SnackbarResult::Dismissed);
        }

        self.current.clone()
    }

    fn has_pending_timeout(&self, now: Instant) -> bool {
        let Some(current) = &self.current else {
            return false;
        };
        let Some(timeout) = current.resolved.duration.timeout() else {
            return false;
        };

        self.current_started_at
            .map(|started_at| now.duration_since(started_at) < timeout)
            .unwrap_or(true)
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
        self.current_started_at = None;
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

/// Arguments for the [`snackbar`] component.
#[derive(PartialEq, Clone, Setters)]
pub struct SnackbarArgs {
    /// Modifier chain applied to the snackbar container.
    pub modifier: Modifier,
    /// Message shown in the snackbar.
    #[setters(into)]
    pub message: String,
    /// Optional label for the action button.
    #[setters(strip_option, into)]
    pub action_label: Option<String>,
    /// Whether to show a dismiss action icon.
    pub with_dismiss_action: bool,
    /// Whether the action should be placed on a new line.
    pub action_on_new_line: bool,
    /// Shape of the snackbar container.
    pub shape: crate::shape_def::Shape,
    /// Container color for the snackbar background.
    pub container_color: Color,
    /// Content color for the message text.
    pub content_color: Color,
    /// Content color for the action label.
    pub action_color: Color,
    /// Content color for the dismiss action.
    pub dismiss_action_color: Color,
    /// Padding applied to the content inside the snackbar.
    pub content_padding: Padding,
    /// Optional action callback.
    #[setters(skip)]
    pub on_action: Option<Callback>,
    /// Optional dismiss callback.
    #[setters(skip)]
    pub on_dismiss: Option<Callback>,
}

impl SnackbarArgs {
    /// Creates snackbar arguments with the required message.
    pub fn new(message: impl Into<String>) -> Self {
        Self::default().message(message)
    }

    /// Sets the action callback.
    pub fn on_action<F>(mut self, on_action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_action = Some(Callback::new(on_action));
        self
    }

    /// Sets the action callback using a shared closure.
    pub fn on_action_shared(mut self, on_action: impl Into<Callback>) -> Self {
        self.on_action = Some(on_action.into());
        self
    }

    /// Sets the dismiss callback.
    pub fn on_dismiss<F>(mut self, on_dismiss: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_dismiss = Some(Callback::new(on_dismiss));
        self
    }

    /// Sets the dismiss callback using a shared closure.
    pub fn on_dismiss_shared(mut self, on_dismiss: impl Into<Callback>) -> Self {
        self.on_dismiss = Some(on_dismiss.into());
        self
    }
}

impl Default for SnackbarArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_width(),
            message: String::new(),
            action_label: None,
            with_dismiss_action: false,
            action_on_new_line: false,
            shape: SnackbarDefaults::shape(),
            container_color: SnackbarDefaults::container_color(),
            content_color: SnackbarDefaults::content_color(),
            action_color: SnackbarDefaults::action_color(),
            dismiss_action_color: SnackbarDefaults::dismiss_action_color(),
            content_padding: SnackbarDefaults::CONTENT_PADDING,
            on_action: None,
            on_dismiss: None,
        }
    }
}

impl From<SnackbarData> for SnackbarArgs {
    fn from(data: SnackbarData) -> Self {
        let SnackbarData {
            message,
            action_label,
            with_dismiss_action,
            host_state,
            id,
            ..
        } = data;
        let mut args = SnackbarArgs::new(message).with_dismiss_action(with_dismiss_action);

        if let Some(label) = action_label.as_deref() {
            args = args.action_label(label);
        }

        if action_label.is_some() {
            let on_action = Callback::new(move || {
                host_state.with_mut(|state| {
                    state.resolve_current(id, SnackbarResult::ActionPerformed);
                });
            });
            args = args.on_action_shared(on_action);
        }

        if with_dismiss_action {
            let on_dismiss = Callback::new(move || {
                host_state.with_mut(|state| {
                    state.resolve_current(id, SnackbarResult::Dismissed);
                });
            });
            args = args.on_dismiss_shared(on_dismiss);
        }

        args
    }
}

/// Arguments for the [`snackbar_host`] component.
#[derive(PartialEq, Clone, Setters)]
pub struct SnackbarHostArgs {
    /// Modifier chain applied to the snackbar host container.
    pub modifier: Modifier,
    /// State that provides snackbar queue data.
    pub state: State<SnackbarHostState>,
    /// Optional custom snackbar slot for rendering.
    #[setters(skip)]
    pub snackbar: Option<CallbackWith<SnackbarData>>,
}

impl SnackbarHostArgs {
    /// Creates host arguments with the required state.
    pub fn new(state: State<SnackbarHostState>) -> Self {
        Self {
            modifier: Modifier::new(),
            state,
            snackbar: None,
        }
    }

    /// Sets the custom snackbar slot.
    pub fn snackbar<F>(mut self, snackbar: F) -> Self
    where
        F: Fn(SnackbarData) + Send + Sync + 'static,
    {
        self.snackbar = Some(CallbackWith::new(snackbar));
        self
    }

    /// Sets the custom snackbar slot using a shared closure.
    pub fn snackbar_shared(mut self, snackbar: impl Into<CallbackWith<SnackbarData>>) -> Self {
        self.snackbar = Some(snackbar.into());
        self
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
/// - `args` — configures message text, colors, and actions; see
///   [`SnackbarArgs`]. Can also be created from [`SnackbarData`].
///
/// ## Examples
///
/// ```
/// use tessera_components::snackbar::{SnackbarArgs, snackbar};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::tessera;
///
/// #[tessera]
/// fn demo() {
///     let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///         || MaterialTheme::default(),
///         || {
///             let args = SnackbarArgs::new("Saved")
///                 .action_label("Undo")
///                 .with_dismiss_action(true)
///                 .on_action(|| {})
///                 .on_dismiss(|| {});
///             assert_eq!(args.message, "Saved");
///             snackbar(&args);
///         },
///     );
///     material_theme(&args);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn snackbar(args: &SnackbarArgs) {
    let args: SnackbarArgs = args.clone();
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let typography = theme.typography;
    let SnackbarArgs {
        modifier,
        message,
        action_label,
        with_dismiss_action,
        action_on_new_line,
        shape,
        container_color,
        content_color,
        action_color,
        dismiss_action_color,
        content_padding,
        on_action,
        on_dismiss,
    } = args;
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

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
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
            .elevation(SnackbarDefaults::CONTAINER_ELEVATION),
        move || {
            let message = message.clone();
            let action_label = action_label.clone();
            let on_action = on_action.clone();
            let on_dismiss = on_dismiss.clone();
            if action_on_new_line {
                render_snackbar_column(SnackbarLayoutArgs {
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
                render_snackbar_row(SnackbarLayoutArgs {
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
        },
    ));
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
/// - `args` — configures the host state and optional custom snackbar slot; see
///   [`SnackbarHostArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::snackbar::{SnackbarHostArgs, SnackbarHostState, snackbar_host};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///         || MaterialTheme::default(),
///         || {
///             let host_state = remember(SnackbarHostState::new);
///             host_state.with_mut(|state| {
///                 state.show_snackbar("Saved");
///             });
///             snackbar_host(&SnackbarHostArgs::new(host_state));
///             assert!(host_state.with(|state| state.is_showing()));
///         },
///     );
///     material_theme(&args);
/// }
///
/// demo();
/// ```
#[tessera]
pub fn snackbar_host(args: &SnackbarHostArgs) {
    let args: SnackbarHostArgs = args.clone();
    let state = args.state;
    let snackbar_slot = args.snackbar;
    let now = Instant::now();
    let record = state.with_mut(|host| host.poll(now));
    if state.with(|host| host.has_pending_timeout(now)) {
        let state_for_frame = state;
        with_frame_nanos(move |_| {
            state_for_frame.with_mut(|host| {
                let _ = host.poll(Instant::now());
            });
        });
    }
    let Some(record) = record else {
        return;
    };
    let data = SnackbarData::new(record, state);

    args.modifier.run(move || {
        let snackbar_slot = snackbar_slot.clone();
        let data = data.clone();
        if let Some(snackbar_slot) = snackbar_slot {
            snackbar_slot.call(data.clone());
        } else {
            Modifier::new()
                .padding(SnackbarDefaults::HOST_PADDING)
                .run(move || {
                    snackbar(&SnackbarArgs::from(data.clone()));
                });
        }
    });
}

fn render_snackbar_row(args: SnackbarLayoutArgs) {
    let SnackbarLayoutArgs {
        message,
        message_style,
        message_color,
        action_label,
        action_color,
        dismiss_action_color,
        on_action,
        on_dismiss,
        padding,
    } = args;
    row(
        RowArgs::default()
            .modifier(Modifier::new().fill_max_width().padding(padding))
            .cross_axis_alignment(CrossAxisAlignment::Center),
        |scope| {
            let message_text = message.clone();
            scope.child_weighted(
                move || {
                    boxed(
                        BoxedArgs::default().alignment(Alignment::CenterStart),
                        |boxed_scope| {
                            let message_text = message_text.clone();
                            boxed_scope.child(move || {
                                render_message(message_text.clone(), message_style, message_color);
                            });
                        },
                    );
                },
                1.0,
            );

            if let Some(label) = action_label.clone() {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().width(SnackbarDefaults::ACTION_SPACING),
                    ))
                });
                scope.child(move || {
                    render_action_button(label.clone(), action_color, on_action.clone());
                });
            }

            if on_dismiss.is_some() {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().width(SnackbarDefaults::ACTION_SPACING),
                    ))
                });
                scope.child(move || {
                    render_dismiss_button(dismiss_action_color, on_dismiss.clone());
                });
            }
        },
    );
}

fn render_snackbar_column(args: SnackbarLayoutArgs) {
    let SnackbarLayoutArgs {
        message,
        message_style,
        message_color,
        action_label,
        action_color,
        dismiss_action_color,
        on_action,
        on_dismiss,
        padding,
    } = args;
    column(
        ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width().padding(padding))
            .cross_axis_alignment(CrossAxisAlignment::Start),
        |scope| {
            let message_text = message.clone();
            scope.child(move || {
                render_message(message_text.clone(), message_style, message_color);
            });

            if action_label.is_some() || on_dismiss.is_some() {
                scope.child(|| {
                    spacer(&crate::spacer::SpacerArgs::new(
                        Modifier::new().height(SnackbarDefaults::ACTION_VERTICAL_SPACING),
                    ))
                });
                let action_label_for_row = action_label.clone();
                let on_action_for_row = on_action.clone();
                let on_dismiss_for_row = on_dismiss.clone();
                scope.child(move || {
                    let action_label_for_row = action_label_for_row.clone();
                    let on_action_for_row = on_action_for_row.clone();
                    let on_dismiss_for_row = on_dismiss_for_row.clone();
                    row(
                        RowArgs::default()
                            .modifier(Modifier::new().fill_max_width())
                            .main_axis_alignment(MainAxisAlignment::End)
                            .cross_axis_alignment(CrossAxisAlignment::Center),
                        move |row_scope| {
                            if let Some(label) = action_label_for_row.clone() {
                                let on_action = on_action_for_row.clone();
                                row_scope.child(move || {
                                    render_action_button(
                                        label.clone(),
                                        action_color,
                                        on_action.clone(),
                                    );
                                });
                            }

                            if on_dismiss_for_row.is_some() {
                                row_scope.child(|| {
                                    spacer(&crate::spacer::SpacerArgs::new(
                                        Modifier::new().width(SnackbarDefaults::ACTION_SPACING),
                                    ));
                                });
                                let on_dismiss = on_dismiss_for_row.clone();
                                row_scope.child(move || {
                                    render_dismiss_button(dismiss_action_color, on_dismiss.clone());
                                });
                            }
                        },
                    );
                });
            }
        },
    );
}

fn render_message(message: String, style: crate::theme::TextStyle, color: Color) {
    provide_text_style(style, move || {
        text(&crate::text::TextArgs::from(
            &TextArgs::default().text(&message).color(color),
        ));
    });
}

struct SnackbarLayoutArgs {
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

fn render_action_button(label: String, action_color: Color, on_action: Option<Callback>) {
    let on_action = on_action.unwrap_or_default();
    button(&crate::button::ButtonArgs::with_child(
        ButtonArgs::text(move || {
            on_action.call();
        })
        .content_color(action_color)
        .ripple_color(action_color),
        move || {
            text(&crate::text::TextArgs::from(
                &TextArgs::default().text(label.clone()).color(action_color),
            ));
        },
    ));
}

fn render_dismiss_button(dismiss_color: Color, on_dismiss: Option<Callback>) {
    let on_dismiss = on_dismiss.unwrap_or_default();
    let icon_args = IconArgs::from(filled::close_icon()).tint_mode(TintMode::Solid);
    icon_button(
        &IconButtonArgs::new(icon_args)
            .variant(IconButtonVariant::Standard)
            .content_color(dismiss_color)
            .on_click_shared(on_dismiss),
    );
}
