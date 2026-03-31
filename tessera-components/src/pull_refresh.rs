//! Pull-to-refresh container and indicator for scrollable content.
//!
//! ## Usage
//!
//! Trigger data reloads when users pull down at the top of a scrollable view.
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, Modifier, Px, RenderSlot, State, current_frame_nanos,
    layout::layout_primitive, provide_context, receive_frame_nanos, remember, tessera, use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::ModifierExt,
    nested_scroll::{
        NestedScrollConnection, PostScrollInput, PreFlingInput, PreScrollInput, ScrollDelta,
        ScrollVelocity,
    },
    progress::circular_progress_indicator,
    shape_def::Shape,
    surface::surface,
    theme::{MaterialTheme, content_color_for},
};

/// Material defaults for pull-to-refresh.
pub struct PullRefreshDefaults;

#[allow(missing_docs)]
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
    last_frame_nanos: Option<u64>,
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
            last_frame_nanos: None,
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

    fn set_refreshing(&mut self, refreshing: bool) {
        if self.refreshing == refreshing {
            return;
        }
        self.refreshing = refreshing;
        self.distance_pulled = 0.0;
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

    fn update_position(&mut self, frame_nanos: u64, smoothing: f32) -> bool {
        let delta_time = if let Some(last_frame_nanos) = self.last_frame_nanos {
            frame_nanos.saturating_sub(last_frame_nanos) as f32 / 1_000_000_000.0
        } else {
            0.016
        };
        self.last_frame_nanos = Some(frame_nanos);

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

#[allow(missing_docs)]
impl PullRefreshBuilder {
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.props.modifier = Some(modifier);
        self
    }

    pub fn refresh_threshold(mut self, refresh_threshold: Dp) -> Self {
        self.props.refresh_threshold = Some(refresh_threshold);
        self
    }

    pub fn refreshing_offset(mut self, refreshing_offset: Dp) -> Self {
        self.props.refreshing_offset = Some(refreshing_offset);
        self
    }

    pub fn indicator_size(mut self, indicator_size: Dp) -> Self {
        self.props.indicator_size = Some(indicator_size);
        self
    }

    pub fn indicator_background_color(mut self, indicator_background_color: Color) -> Self {
        self.props.indicator_background_color = Some(indicator_background_color);
        self
    }

    pub fn indicator_content_color(mut self, indicator_content_color: Color) -> Self {
        self.props.indicator_content_color = Some(indicator_content_color);
        self
    }

    pub fn indicator_track_color(mut self, indicator_track_color: Color) -> Self {
        self.props.indicator_track_color = Some(indicator_track_color);
        self
    }

    pub fn indicator_stroke_width(mut self, indicator_stroke_width: Dp) -> Self {
        self.props.indicator_stroke_width = Some(indicator_stroke_width);
        self
    }

    pub fn indicator_elevation(mut self, indicator_elevation: Dp) -> Self {
        self.props.indicator_elevation = Some(indicator_elevation);
        self
    }

    pub fn controller(mut self, controller: State<PullRefreshController>) -> Self {
        self.props.controller = Some(controller);
        self
    }
}

#[allow(missing_docs)]
impl PullRefreshIndicatorBuilder {
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.props.modifier = Some(modifier);
        self
    }

    pub fn size(mut self, size: Dp) -> Self {
        self.props.size = Some(size);
        self
    }

    pub fn background_color(mut self, background_color: Color) -> Self {
        self.props.background_color = Some(background_color);
        self
    }

    pub fn content_color(mut self, content_color: Color) -> Self {
        self.props.content_color = Some(content_color);
        self
    }

    pub fn track_color(mut self, track_color: Color) -> Self {
        self.props.track_color = Some(track_color);
        self
    }

    pub fn stroke_width(mut self, stroke_width: Dp) -> Self {
        self.props.stroke_width = Some(stroke_width);
        self
    }

    pub fn elevation(mut self, elevation: Dp) -> Self {
        self.props.elevation = Some(elevation);
        self
    }

    pub fn controller(mut self, controller: State<PullRefreshController>) -> Self {
        self.props.controller = Some(controller);
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
/// - `modifier` — optional modifier chain applied to the indicator.
/// - `size` — optional indicator container size.
/// - `background_color` — optional indicator background color.
/// - `content_color` — optional indicator content color.
/// - `track_color` — optional indicator track color.
/// - `stroke_width` — optional indicator stroke width.
/// - `elevation` — optional indicator elevation.
/// - `controller` — optional controller that drives progress and refreshing.
///
/// ## Examples
///
/// ```
/// use tessera_components::pull_refresh::{PullRefreshController, pull_refresh_indicator};
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let controller = remember(PullRefreshController::new);
///             pull_refresh_indicator().controller(controller);
///             assert_eq!(controller.with(|s| s.progress()), 0.0);
///         });
/// }
/// ```
#[tessera]
pub fn pull_refresh_indicator(
    #[prop(skip_setter)] modifier: Option<Modifier>,
    #[prop(skip_setter)] size: Option<Dp>,
    #[prop(skip_setter)] background_color: Option<Color>,
    #[prop(skip_setter)] content_color: Option<Color>,
    #[prop(skip_setter)] track_color: Option<Color>,
    #[prop(skip_setter)] stroke_width: Option<Dp>,
    #[prop(skip_setter)] elevation: Option<Dp>,
    #[prop(skip_setter)] controller: Option<State<PullRefreshController>>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let background_color = background_color.unwrap_or(scheme.surface);
    let content_color = content_color.unwrap_or_else(|| {
        content_color_for(background_color, &scheme).unwrap_or(scheme.on_surface)
    });
    let controller = controller.unwrap_or_else(|| remember(PullRefreshController::new));
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

    let indicator_size = size.unwrap_or(PullRefreshDefaults::INDICATOR_SIZE);
    let indicator_modifier = modifier
        .unwrap_or_default()
        .size(indicator_size, indicator_size)
        .alpha(indicator_alpha);
    let content_size = Dp(indicator_size.0 * INDICATOR_CONTENT_SCALE as f64);

    surface()
        .modifier(indicator_modifier)
        .shape(Shape::Ellipse)
        .style(background_color.into())
        .elevation(elevation.unwrap_or(PullRefreshDefaults::INDICATOR_ELEVATION))
        .content_alignment(Alignment::Center)
        .with_child(move || {
            if !refreshing {
                circular_progress_indicator()
                    .diameter(content_size)
                    .stroke_width(
                        stroke_width.unwrap_or(PullRefreshDefaults::INDICATOR_STROKE_WIDTH),
                    )
                    .color(content_color)
                    .track_color(track_color.unwrap_or(Color::TRANSPARENT))
                    .progress(progress);
            } else {
                circular_progress_indicator()
                    .diameter(content_size)
                    .stroke_width(
                        stroke_width.unwrap_or(PullRefreshDefaults::INDICATOR_STROKE_WIDTH),
                    )
                    .color(content_color)
                    .track_color(track_color.unwrap_or(Color::TRANSPARENT));
            }
        });
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
/// - `modifier` — optional modifier chain applied to the pull-refresh
///   container.
/// - `on_refresh` — optional callback invoked when a refresh is triggered.
/// - `refreshing` — whether a refresh is currently in progress.
/// - `enabled` — whether pull-to-refresh interactions are enabled.
/// - `refresh_threshold` — optional pull distance required to trigger a
///   refresh.
/// - `refreshing_offset` — optional resting offset while refreshing.
/// - `indicator_size` — optional indicator container size.
/// - `indicator_background_color` — optional indicator background color.
/// - `indicator_content_color` — optional indicator content color.
/// - `indicator_track_color` — optional indicator track color.
/// - `indicator_stroke_width` — optional indicator stroke width.
/// - `indicator_elevation` — optional indicator elevation.
/// - `controller` — optional external refresh controller.
/// - `child` — optional content rendered inside the pull-refresh container.
///
/// ## Examples
///
/// ```
/// use tessera_components::theme::{MaterialTheme, material_theme};
/// use tessera_components::{
///     column::column,
///     pull_refresh::{PullRefreshController, pull_refresh},
///     scrollable::scrollable,
///     text::text,
/// };
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn demo() {
///     material_theme()
///         .theme(|| MaterialTheme::default())
///         .child(|| {
///             let refresh_controller = remember(PullRefreshController::new);
///             pull_refresh()
///                 .on_refresh(|| {})
///                 .refreshing(false)
///                 .controller(refresh_controller)
///                 .child(|| {
///                     scrollable().child(|| {
///                         column().children(|| {
///                             text().content("Pull down to refresh");
///                         });
///                     });
///                 });
///             assert!(!refresh_controller.with(|s| s.refreshing()));
///         });
/// }
///
/// demo();
/// ```
#[tessera]
pub fn pull_refresh(
    #[prop(skip_setter)] modifier: Option<Modifier>,
    on_refresh: Option<Callback>,
    refreshing: bool,
    enabled: bool,
    #[prop(skip_setter)] refresh_threshold: Option<Dp>,
    #[prop(skip_setter)] refreshing_offset: Option<Dp>,
    #[prop(skip_setter)] indicator_size: Option<Dp>,
    #[prop(skip_setter)] indicator_background_color: Option<Color>,
    #[prop(skip_setter)] indicator_content_color: Option<Color>,
    #[prop(skip_setter)] indicator_track_color: Option<Color>,
    #[prop(skip_setter)] indicator_stroke_width: Option<Dp>,
    #[prop(skip_setter)] indicator_elevation: Option<Dp>,
    #[prop(skip_setter)] controller: Option<State<PullRefreshController>>,
    child: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let indicator_background_color = indicator_background_color.unwrap_or(scheme.surface);
    let indicator_content_color = indicator_content_color.unwrap_or_else(|| {
        content_color_for(indicator_background_color, &scheme).unwrap_or(scheme.on_surface)
    });
    let controller = controller.unwrap_or_else(|| remember(PullRefreshController::new));
    let child = child.unwrap_or_else(RenderSlot::empty);
    let modifier = modifier
        .unwrap_or_else(|| Modifier::new().fill_max_size())
        .clip_to_bounds();

    controller.with_mut(|state| {
        state.set_threshold(
            refresh_threshold
                .unwrap_or(PullRefreshDefaults::REFRESH_THRESHOLD)
                .to_pixels_f32(),
        );
        state.set_refreshing_offset(
            refreshing_offset
                .unwrap_or(PullRefreshDefaults::REFRESHING_OFFSET)
                .to_pixels_f32(),
        );
        state.set_refreshing(refreshing);
    });
    let frame_nanos = current_frame_nanos();
    controller.with_mut(|s| {
        s.update_position(frame_nanos, INDICATOR_SMOOTHING);
    });
    if controller.with(|s| s.has_pending_animation_frame()) {
        receive_frame_nanos(move |frame_nanos| {
            let has_pending_animation_frame = controller.with_mut(|s| {
                s.update_position(frame_nanos, INDICATOR_SMOOTHING);
                s.has_pending_animation_frame()
            });
            if has_pending_animation_frame {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }

    let parent_nested_scroll = use_context::<NestedScrollConnection>().map(|context| context.get());
    let on_refresh = on_refresh.unwrap_or_default();
    let nested_scroll_connection = NestedScrollConnection::new()
        .with_pre_scroll_handler(CallbackWith::new({
            move |input: PreScrollInput| {
                if !enabled || input.source != tessera_ui::ScrollEventSource::Touch {
                    return ScrollDelta::ZERO;
                }

                let consumed_y = if input.available.y < 0.0 && controller.with(|s| s.is_pulling()) {
                    controller.with_mut(|s| s.on_pull(input.available.y))
                } else {
                    0.0
                };
                ScrollDelta::new(0.0, consumed_y)
            }
        }))
        .with_post_scroll_handler(CallbackWith::new({
            move |input: PostScrollInput| {
                if !enabled || input.source != tessera_ui::ScrollEventSource::Touch {
                    return ScrollDelta::ZERO;
                }
                let _ = input.consumed_by_child;

                let consumed_y = if input.available.y > 0.0 {
                    controller.with_mut(|s| s.on_pull(input.available.y))
                } else {
                    0.0
                };
                ScrollDelta::new(0.0, consumed_y)
            }
        }))
        .with_pre_fling_handler(CallbackWith::new({
            move |input: PreFlingInput| {
                if !enabled || !controller.with(|s| s.is_pulling()) {
                    return ScrollVelocity::ZERO;
                }

                let should_refresh = controller.with_mut(|s| s.on_release());
                if should_refresh {
                    on_refresh.call();
                }

                ScrollVelocity::new(0.0, input.available.y.max(0.0))
            }
        }))
        .with_parent(parent_nested_scroll);

    layout_primitive().modifier(modifier).child(move || {
        let child = child;
        let nested_scroll_connection = nested_scroll_connection.clone();
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::TopCenter)
            .children(move || {
                let nested_scroll_connection = nested_scroll_connection.clone();
                provide_context(
                    || nested_scroll_connection.clone(),
                    move || {
                        child.render();
                    },
                );
                layout_primitive()
                    .modifier(Modifier::new().align(Alignment::TopCenter))
                    .child(move || {
                        let offset = indicator_offset_dp(
                            controller,
                            indicator_size.unwrap_or(PullRefreshDefaults::INDICATOR_SIZE),
                        );
                        pull_refresh_indicator_with_offset(PullRefreshIndicatorOffsetArgs {
                            indicator_size: indicator_size
                                .unwrap_or(PullRefreshDefaults::INDICATOR_SIZE),
                            indicator_background_color,
                            indicator_content_color,
                            indicator_track_color: indicator_track_color
                                .unwrap_or(Color::TRANSPARENT),
                            indicator_stroke_width: indicator_stroke_width
                                .unwrap_or(PullRefreshDefaults::INDICATOR_STROKE_WIDTH),
                            indicator_elevation: indicator_elevation
                                .unwrap_or(PullRefreshDefaults::INDICATOR_ELEVATION),
                            refresh_controller: controller,
                            offset,
                        });
                    });
            });
    });
}

struct PullRefreshIndicatorOffsetArgs {
    indicator_size: Dp,
    indicator_background_color: Color,
    indicator_content_color: Color,
    indicator_track_color: Color,
    indicator_stroke_width: Dp,
    indicator_elevation: Dp,
    refresh_controller: State<PullRefreshController>,
    offset: Dp,
}

fn pull_refresh_indicator_with_offset(args: PullRefreshIndicatorOffsetArgs) {
    layout_primitive()
        .modifier(Modifier::new().offset(Dp(0.0), args.offset))
        .child(move || {
            pull_refresh_indicator()
                .size(args.indicator_size)
                .background_color(args.indicator_background_color)
                .content_color(args.indicator_content_color)
                .track_color(args.indicator_track_color)
                .stroke_width(args.indicator_stroke_width)
                .elevation(args.indicator_elevation)
                .controller(args.refresh_controller);
        });
}

fn indicator_offset_dp(controller: State<PullRefreshController>, indicator_size: Dp) -> Dp {
    let indicator_offset_px =
        controller.with(|s| s.position().to_f32()) - indicator_size.to_pixels_f32();
    Dp::from_pixels_f32(indicator_offset_px)
}
