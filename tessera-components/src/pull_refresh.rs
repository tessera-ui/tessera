//! Pull-to-refresh container and indicator for scrollable content.
//!
//! ## Usage
//!
//! Trigger data reloads when users pull down at the top of a scrollable view.
use std::time::{Duration, Instant};

use derive_setters::Setters;
use tessera_ui::{
    Callback, Color, CursorEventContent, Dp, Modifier, PressKeyEventType, Px, RenderSlot, State,
    receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    modifier::ModifierExt,
    pos_misc::is_position_inside_bounds,
    progress::{CircularProgressIndicatorArgs, circular_progress_indicator},
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
    theme::{MaterialTheme, content_color_for},
};

/// Material defaults for pull-to-refresh.
pub struct PullRefreshDefaults;

impl PullRefreshDefaults {
    /// Pull distance required to trigger a refresh.
    pub const REFRESH_THRESHOLD: Dp = Dp(80.0);
    /// Offset where the indicator rests while refreshing.
    pub const REFRESHING_OFFSET: Dp = Dp(56.0);
    /// Indicator container size.
    pub const INDICATOR_SIZE: Dp = Dp(40.0);
    /// Indicator stroke width.
    pub const INDICATOR_STROKE_WIDTH: Dp = Dp(2.5);
    /// Indicator elevation.
    pub const INDICATOR_ELEVATION: Dp = Dp(6.0);
}

const DRAG_MULTIPLIER: f32 = 0.5;
const INDICATOR_CONTENT_SCALE: f32 = 0.6;
const INDICATOR_SMOOTHING: f32 = 0.2;
const SCROLL_IDLE_RELEASE_TIMEOUT: Duration = Duration::from_millis(500);
const INDICATOR_FADE_START_PROGRESS: f32 = 0.05;
const INDICATOR_FADE_END_PROGRESS: f32 = 0.25;

/// Tracks pull-to-refresh state and indicator position.
pub struct PullRefreshController {
    refreshing: bool,
    position: f32,
    target_position: f32,
    distance_pulled: f32,
    threshold: f32,
    refreshing_offset: f32,
    last_frame_time: Option<Instant>,
    is_pressed: bool,
    last_scroll_at: Option<Instant>,
}

impl Default for PullRefreshController {
    fn default() -> Self {
        Self::new()
    }
}

impl PullRefreshController {
    /// Creates a new `PullRefreshController` with default thresholds.
    pub fn new() -> Self {
        Self {
            refreshing: false,
            position: 0.0,
            target_position: 0.0,
            distance_pulled: 0.0,
            threshold: PullRefreshDefaults::REFRESH_THRESHOLD.to_pixels_f32(),
            refreshing_offset: PullRefreshDefaults::REFRESHING_OFFSET.to_pixels_f32(),
            last_frame_time: None,
            is_pressed: false,
            last_scroll_at: None,
        }
    }

    /// Returns the pull progress as a ratio of the refresh threshold.
    pub fn progress(&self) -> f32 {
        if self.threshold <= 0.0 {
            return 0.0;
        }
        self.adjusted_distance_pulled() / self.threshold
    }

    /// Returns whether a refresh is currently in progress.
    pub fn refreshing(&self) -> bool {
        self.refreshing
    }

    /// Returns the current indicator position in pixels.
    pub fn position(&self) -> Px {
        Px::saturating_from_f32(self.position)
    }

    fn is_pulling(&self) -> bool {
        self.distance_pulled > 0.0
    }

    fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    fn mark_scroll(&mut self, now: Instant) {
        self.last_scroll_at = Some(now);
    }

    fn should_release(&self, now: Instant, idle_timeout: Duration) -> bool {
        self.last_scroll_at
            .map(|last| now.duration_since(last) >= idle_timeout)
            .unwrap_or(true)
    }

    fn set_refreshing(&mut self, refreshing: bool) {
        if self.refreshing == refreshing {
            return;
        }
        self.refreshing = refreshing;
        self.distance_pulled = 0.0;
        self.last_scroll_at = None;
        let target = if refreshing {
            self.refreshing_offset
        } else {
            0.0
        };
        self.set_target_position(target);
    }

    fn set_threshold(&mut self, threshold: f32) {
        if threshold.is_finite() && threshold > 0.0 {
            self.threshold = threshold;
        }
    }

    fn set_refreshing_offset(&mut self, offset: f32) {
        if offset.is_finite() && offset >= 0.0 {
            self.refreshing_offset = offset;
            if self.refreshing {
                self.set_target_position(offset);
            }
        }
    }

    fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    fn on_pull(&mut self, pull_delta: f32) -> f32 {
        if self.refreshing {
            return 0.0;
        }

        let new_offset = (self.distance_pulled + pull_delta).max(0.0);
        let consumed = new_offset - self.distance_pulled;
        self.distance_pulled = new_offset;
        let position = self.calculate_indicator_position();
        self.position = position;
        self.target_position = position;
        consumed
    }

    fn on_release(&mut self) -> bool {
        if self.refreshing {
            return false;
        }

        let should_refresh = self.adjusted_distance_pulled() > self.threshold;
        self.distance_pulled = 0.0;
        self.set_target_position(0.0);
        should_refresh
    }

    fn update_position(&mut self, smoothing: f32) -> bool {
        let current_time = Instant::now();
        let delta_time = if let Some(last_time) = self.last_frame_time {
            current_time.duration_since(last_time).as_secs_f32()
        } else {
            0.016
        };
        self.last_frame_time = Some(current_time);

        let diff = self.target_position - self.position;
        if diff.abs() < 0.5 {
            if (self.position - self.target_position).abs() > f32::EPSILON {
                self.position = self.target_position;
                return true;
            }
            return false;
        }

        let mut movement_factor = (1.0 - smoothing).mul_add(delta_time * 60.0, 0.0);
        if movement_factor > 1.0 {
            movement_factor = 1.0;
        }

        let old_position = self.position;
        self.position += diff * movement_factor;
        (self.position - old_position).abs() > f32::EPSILON
    }

    fn adjusted_distance_pulled(&self) -> f32 {
        self.distance_pulled * DRAG_MULTIPLIER
    }

    fn calculate_indicator_position(&self) -> f32 {
        let adjusted = self.adjusted_distance_pulled();
        if adjusted <= self.threshold {
            return adjusted;
        }

        let overshoot_percent = (self.progress().abs() - 1.0).max(0.0);
        let linear_tension = overshoot_percent.clamp(0.0, 2.0);
        let tension_percent = linear_tension - (linear_tension * linear_tension / 4.0);
        self.threshold + self.threshold * tension_percent
    }

    fn set_target_position(&mut self, target: f32) {
        self.target_position = target.max(0.0);
    }

    fn has_pending_animation_frame(&self) -> bool {
        (self.target_position - self.position).abs() > f32::EPSILON
    }
}

/// Arguments for the `pull_refresh` component.
#[derive(PartialEq, Clone, Setters)]
pub struct PullRefreshArgs {
    /// Modifier chain applied to the pull-refresh container.
    pub modifier: Modifier,
    /// Callback invoked when a refresh is triggered.
    #[setters(skip)]
    pub on_refresh: Callback,
    /// Whether a refresh is currently in progress.
    pub refreshing: bool,
    /// Whether pull-to-refresh interactions are enabled.
    pub enabled: bool,
    /// Pull distance required to trigger a refresh.
    pub refresh_threshold: Dp,
    /// Offset where the indicator rests while refreshing.
    pub refreshing_offset: Dp,
    /// Indicator container size.
    pub indicator_size: Dp,
    /// Indicator background color.
    pub indicator_background_color: Color,
    /// Indicator content color.
    pub indicator_content_color: Color,
    /// Indicator track color.
    pub indicator_track_color: Color,
    /// Indicator stroke width.
    pub indicator_stroke_width: Dp,
    /// Indicator elevation.
    pub indicator_elevation: Dp,
    /// Optional external refresh controller.
    ///
    /// When this is `None`, `pull_refresh` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<PullRefreshController>>,
    /// Optional child content rendered inside the pull-refresh container.
    #[setters(skip)]
    pub child: Option<RenderSlot>,
}

impl PullRefreshArgs {
    /// Creates arguments with the required refresh callback.
    pub fn new(on_refresh: impl Fn() + Send + Sync + 'static) -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let background = scheme.surface;
        Self {
            modifier: Modifier::new().fill_max_size(),
            on_refresh: Callback::new(on_refresh),
            refreshing: false,
            enabled: true,
            refresh_threshold: PullRefreshDefaults::REFRESH_THRESHOLD,
            refreshing_offset: PullRefreshDefaults::REFRESHING_OFFSET,
            indicator_size: PullRefreshDefaults::INDICATOR_SIZE,
            indicator_background_color: background,
            indicator_content_color: content_color_for(background, &scheme)
                .unwrap_or(scheme.on_surface),
            indicator_track_color: Color::TRANSPARENT,
            indicator_stroke_width: PullRefreshDefaults::INDICATOR_STROKE_WIDTH,
            indicator_elevation: PullRefreshDefaults::INDICATOR_ELEVATION,
            controller: None,
            child: None,
        }
    }

    /// Set the refresh handler using a shared callback.
    pub fn on_refresh_shared(mut self, on_refresh: impl Into<Callback>) -> Self {
        self.on_refresh = on_refresh.into();
        self
    }

    /// Sets an external pull-refresh controller.
    pub fn controller(mut self, controller: State<PullRefreshController>) -> Self {
        self.controller = Some(controller);
        self
    }

    /// Sets the child content.
    pub fn child<F>(mut self, child: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.child = Some(RenderSlot::new(child));
        self
    }

    /// Sets the child content using a shared render slot.
    pub fn child_shared(mut self, child: impl Into<RenderSlot>) -> Self {
        self.child = Some(child.into());
        self
    }
}

/// Arguments for the pull-refresh indicator.
#[derive(PartialEq, Clone, Setters)]
pub struct PullRefreshIndicatorArgs {
    /// Modifier chain applied to the indicator.
    pub modifier: Modifier,
    /// Indicator container size.
    pub size: Dp,
    /// Indicator background color.
    pub background_color: Color,
    /// Indicator content color.
    pub content_color: Color,
    /// Indicator track color.
    pub track_color: Color,
    /// Indicator stroke width.
    pub stroke_width: Dp,
    /// Indicator elevation.
    pub elevation: Dp,
    /// Optional external refresh controller.
    ///
    /// When this is `None`, `pull_refresh_indicator` creates and owns an
    /// internal controller.
    #[setters(skip)]
    pub controller: Option<State<PullRefreshController>>,
}

impl Default for PullRefreshIndicatorArgs {
    fn default() -> Self {
        let scheme = use_context::<MaterialTheme>()
            .expect("MaterialTheme must be provided")
            .get()
            .color_scheme;
        let background = scheme.surface;
        Self {
            modifier: Modifier::new(),
            size: PullRefreshDefaults::INDICATOR_SIZE,
            background_color: background,
            content_color: content_color_for(background, &scheme).unwrap_or(scheme.on_surface),
            track_color: Color::TRANSPARENT,
            stroke_width: PullRefreshDefaults::INDICATOR_STROKE_WIDTH,
            elevation: PullRefreshDefaults::INDICATOR_ELEVATION,
            controller: None,
        }
    }
}

impl PullRefreshIndicatorArgs {
    /// Sets an external pull-refresh controller.
    pub fn controller(mut self, controller: State<PullRefreshController>) -> Self {
        self.controller = Some(controller);
        self
    }
}

/// # pull_refresh_indicator
///
/// Draws the default pull-to-refresh indicator for refreshable lists and feeds.
///
/// ## Usage
///
/// Use inside a pull-to-refresh container to visualize pull progress or
/// refreshing state.
///
/// ## Parameters
///
/// - `args` — configures indicator size and colors; see
///   [`PullRefreshIndicatorArgs`].
/// - `controller` — the [`PullRefreshController`] that drives progress and
///   refreshing.
///
/// ## Examples
///
/// ```
/// use tessera_components::pull_refresh::{
///     PullRefreshController, PullRefreshIndicatorArgs, pull_refresh_indicator,
/// };
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///         || MaterialTheme::default(),
///         || {
///             let controller = remember(PullRefreshController::new);
///             pull_refresh_indicator(&PullRefreshIndicatorArgs::default().controller(controller));
///             assert_eq!(controller.with(|s| s.progress()), 0.0);
///         },
///     );
///     material_theme(&args);
/// }
/// ```
#[tessera]
pub fn pull_refresh_indicator(args: &PullRefreshIndicatorArgs) {
    let args: PullRefreshIndicatorArgs = args.clone();
    let controller = args
        .controller
        .unwrap_or_else(|| remember(PullRefreshController::new));
    let refreshing = controller.with(|s| s.refreshing());
    let progress = controller.with(|s| s.progress()).clamp(0.0, 1.0);
    let indicator_alpha = if refreshing {
        1.0
    } else if progress <= INDICATOR_FADE_START_PROGRESS {
        0.0
    } else if progress >= INDICATOR_FADE_END_PROGRESS {
        1.0
    } else {
        (progress - INDICATOR_FADE_START_PROGRESS)
            / (INDICATOR_FADE_END_PROGRESS - INDICATOR_FADE_START_PROGRESS)
    };

    if indicator_alpha <= 0.0 {
        return;
    }

    let indicator_size = args.size;
    let indicator_modifier = args
        .modifier
        .size(indicator_size, indicator_size)
        .alpha(indicator_alpha);
    let content_size = Dp(indicator_size.0 * INDICATOR_CONTENT_SCALE as f64);

    surface(&crate::surface::SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(indicator_modifier)
            .shape(Shape::Ellipse)
            .style(args.background_color.into())
            .elevation(args.elevation)
            .content_alignment(Alignment::Center),
        move || {
            let mut indicator_args = CircularProgressIndicatorArgs::default()
                .diameter(content_size)
                .stroke_width(args.stroke_width)
                .color(args.content_color)
                .track_color(args.track_color);

            if !refreshing {
                indicator_args = indicator_args.progress(progress);
            }

            circular_progress_indicator(&indicator_args);
        },
    ));
}

/// # pull_refresh
///
/// Wraps a scrollable child and adds pull-to-refresh interaction for lists and
/// feeds.
///
/// ## Usage
///
/// Use for feeds or lists that need to reload data when pulled from the top.
///
/// ## Parameters
///
/// - `args` — configures indicator visuals and refresh behavior; see
///   [`PullRefreshArgs`].
/// - `child` — closure that renders the scrollable content (e.g., `scrollable`
///   or `lazy_column`).
///
/// ## Examples
///
/// ```
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_components::{
///     column::{ColumnArgs, column},
///     pull_refresh::{PullRefreshArgs, PullRefreshController, pull_refresh},
///     scrollable::{ScrollableArgs, scrollable},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     let args = tessera_components::theme::MaterialThemeProviderArgs::new(
///         || MaterialTheme::default(),
///         || {
///             let refresh_controller = remember(PullRefreshController::new);
///             let pull_args = PullRefreshArgs::new(|| {})
///                 .refreshing(false)
///                 .controller(refresh_controller)
///                 .child(|| {
///                     scrollable(&ScrollableArgs::default().child(|| {
///                         column(ColumnArgs::default(), |scope| {
///                             scope.child(|| {
///                                 text(&TextArgs::default().text("Pull down to refresh"));
///                             });
///                         });
///                     }));
///                 });
///             pull_refresh(&pull_args);
///             assert!(!refresh_controller.with(|s| s.refreshing()));
///         },
///     );
///     material_theme(&args);
/// }
/// ```
#[tessera]
pub fn pull_refresh(args: &PullRefreshArgs) {
    let args: PullRefreshArgs = args.clone();
    let controller = args
        .controller
        .unwrap_or_else(|| remember(PullRefreshController::new));
    let child = args.child.clone().unwrap_or_else(|| RenderSlot::new(|| {}));
    let modifier = args.modifier.clone().clip_to_bounds();

    controller.with_mut(|state| {
        state.set_threshold(args.refresh_threshold.to_pixels_f32());
        state.set_refreshing_offset(args.refreshing_offset.to_pixels_f32());
        state.set_refreshing(args.refreshing);
        if !args.enabled {
            state.set_pressed(false);
        }
    });
    controller.with_mut(|s| {
        s.update_position(INDICATOR_SMOOTHING);
    });
    if controller.with(|s| s.has_pending_animation_frame()) {
        let controller_for_frame = controller;
        receive_frame_nanos(move |_| {
            let has_pending_animation_frame = controller_for_frame.with_mut(|s| {
                s.update_position(INDICATOR_SMOOTHING);
                s.has_pending_animation_frame()
            });
            if has_pending_animation_frame {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let on_refresh = args.on_refresh.clone();
    let enabled = args.enabled;

    input_handler(move |input| {
        let size = input.computed_data;
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in_component = cursor_pos_option
            .map(|pos| is_position_inside_bounds(size, pos))
            .unwrap_or(false);

        if enabled && !is_cursor_in_component {
            controller.with_mut(|s| s.set_pressed(false));
        }

        let mut saw_scroll = false;
        let mut saw_release = false;
        let mut did_pull = false;
        let now = Instant::now();

        if is_cursor_in_component && enabled {
            for event in input.cursor_events.iter() {
                match event.content {
                    CursorEventContent::Pressed(PressKeyEventType::Left) => {
                        controller.with_mut(|s| s.set_pressed(true));
                    }
                    CursorEventContent::Released(PressKeyEventType::Left) => {
                        controller.with_mut(|s| s.set_pressed(false));
                        saw_release = true;
                    }
                    _ => {}
                }
            }
        }

        if is_cursor_in_component {
            for event in input
                .cursor_events
                .iter()
                .filter_map(|event| match &event.content {
                    CursorEventContent::Scroll(event) => Some(event),
                    _ => None,
                })
            {
                saw_scroll = true;
                let mut delta_y = event.delta_y;
                if enabled && delta_y < 0.0 && controller.with(|s| s.is_pulling()) {
                    let consumed = controller.with_mut(|s| s.on_pull(delta_y));
                    if consumed.abs() > f32::EPSILON {
                        did_pull = true;
                    }
                    delta_y -= consumed;
                }

                if enabled && delta_y > 0.0 {
                    let consumed = controller.with_mut(|s| s.on_pull(delta_y));
                    if consumed.abs() > f32::EPSILON {
                        did_pull = true;
                    }
                }
            }

            if enabled && saw_scroll {
                controller.with_mut(|s| s.mark_scroll(now));
            }

            if enabled
                && controller.with(|s| s.is_pulling())
                && (saw_release
                    || (!saw_scroll
                        && !controller.with(|s| s.is_pressed())
                        && controller.with(|s| s.should_release(now, SCROLL_IDLE_RELEASE_TIMEOUT))))
            {
                let should_refresh = controller.with_mut(|s| s.on_release());
                if should_refresh {
                    on_refresh.call();
                }
                did_pull = true;
            }

            if enabled && (did_pull || saw_release) {
                input.cursor_events.clear();
            }
        }
    });

    let render_args = args.clone();
    modifier.run(move || {
        let child = child.clone();
        let indicator_args = render_args.clone();
        boxed(
            BoxedArgs::default()
                .modifier(Modifier::new().fill_max_size())
                .alignment(Alignment::TopCenter),
            |scope| {
                let child = child.clone();
                scope.child(move || {
                    child.render();
                });
                scope.child_with_alignment(Alignment::TopCenter, move || {
                    let offset = indicator_offset_dp(controller, indicator_args.indicator_size);
                    pull_refresh_indicator_with_offset(indicator_args.clone(), controller, offset);
                });
            },
        );
    });
}

fn pull_refresh_indicator_with_offset(
    args: PullRefreshArgs,
    refresh_controller: State<PullRefreshController>,
    offset: Dp,
) {
    let indicator_size = args.indicator_size;
    let indicator_args = PullRefreshIndicatorArgs::default()
        .size(indicator_size)
        .background_color(args.indicator_background_color)
        .content_color(args.indicator_content_color)
        .track_color(args.indicator_track_color)
        .stroke_width(args.indicator_stroke_width)
        .elevation(args.indicator_elevation)
        .controller(refresh_controller);

    Modifier::new().offset(Dp(0.0), offset).run(move || {
        pull_refresh_indicator(&indicator_args);
    });
}

fn indicator_offset_dp(controller: State<PullRefreshController>, indicator_size: Dp) -> Dp {
    let indicator_offset_px =
        controller.with(|s| s.position().to_f32()) - indicator_size.to_pixels_f32();
    Dp::from_pixels_f32(indicator_offset_px)
}
