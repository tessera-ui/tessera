//! A container that allows its content to be scrolled.
//!
//! ## Usage
//!
//! Use to display content that might overflow the available space.
pub(crate) mod scrollbar;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use derive_setters::Setters;
use tessera_ui::{
    Color, ComputedData, Constraint, CursorEvent, CursorEventContent, DimensionValue, Dp,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, ScrollEventSource, State,
    current_frame_nanos,
    layout::{LayoutInput, LayoutOutput, LayoutSpec, RenderInput},
    receive_frame_nanos, remember, tessera,
};

use crate::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    modifier::ModifierExt,
    pos_misc::is_position_inside_bounds,
    scrollable::scrollbar::{ScrollBarArgs, ScrollBarState, scrollbar_h, scrollbar_v},
};

/// Arguments for the `scrollable` container.
#[derive(PartialEq, Setters, Clone)]
pub struct ScrollableArgs {
    /// Modifier chain applied to the scrollable subtree.
    pub modifier: Modifier,
    /// Is vertical scrollable?
    /// Defaults to `true` since most scrollable areas are vertical.
    pub vertical: bool,
    /// Is horizontal scrollable?
    /// Defaults to `false` since most scrollable areas are not horizontal.
    pub horizontal: bool,
    /// Scroll smoothing factor (0.0 = instant, 1.0 = very smooth).
    /// Defaults to 0.05 for very responsive but still smooth scrolling.
    pub scroll_smoothing: f32,
    /// The behavior of the scrollbar visibility.
    pub scrollbar_behavior: ScrollBarBehavior,
    /// The color of the scrollbar track.
    pub scrollbar_track_color: Color,
    /// The color of the scrollbar thumb.
    pub scrollbar_thumb_color: Color,
    /// The color of the scrollbar thumb when hovered.
    pub scrollbar_thumb_hover_color: Color,
    /// The layout of the scrollbar relative to the content.
    pub scrollbar_layout: ScrollBarLayout,
    /// Optional external controller for scroll position and animation.
    ///
    /// When this is `None`, `scrollable` creates and owns an internal
    /// controller.
    #[setters(skip)]
    pub controller: Option<State<ScrollableController>>,
    /// Optional child content rendered inside the scroll container.
    #[setters(skip)]
    pub child: Option<RenderSlot>,
}

const SCROLL_INERTIA_DECAY_CONSTANT: f32 = 5.0;
const SCROLL_INERTIA_MIN_VELOCITY: f32 = 10.0;
const SCROLL_INERTIA_START_THRESHOLD: f32 = 50.0;
const SCROLL_INERTIA_MAX_VELOCITY: f32 = 6000.0;
const SCROLL_VELOCITY_SAMPLE_WINDOW: Duration = Duration::from_millis(90);
const SCROLL_VELOCITY_IDLE_CUTOFF: Duration = Duration::from_millis(65);

#[derive(Clone, PartialEq)]
struct ScrollVelocityTracker {
    samples: VecDeque<(Instant, f32, f32)>,
    last_sample_time: Instant,
}

#[derive(Clone, PartialEq)]
struct ActiveInertia {
    velocity_x: f32,
    velocity_y: f32,
    last_tick_time: Instant,
}

fn clamp_inertia_velocity(vx: f32, vy: f32) -> (f32, f32) {
    if !vx.is_finite() || !vy.is_finite() {
        return (0.0, 0.0);
    }

    let magnitude_sq = vx * vx + vy * vy;
    if !magnitude_sq.is_finite() {
        return (0.0, 0.0);
    }

    let magnitude = magnitude_sq.sqrt();
    if magnitude > SCROLL_INERTIA_MAX_VELOCITY && SCROLL_INERTIA_MAX_VELOCITY > 0.0 {
        let scale = SCROLL_INERTIA_MAX_VELOCITY / magnitude;
        return (vx * scale, vy * scale);
    }

    (vx, vy)
}

/// Defines the behavior of the scrollbar visibility.
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollBarBehavior {
    /// The scrollbar is always visible.
    AlwaysVisible,
    /// The scrollbar is only visible when scrolling.
    AutoHide,
    /// No scrollbar at all.
    Hidden,
}

/// Defines the layout of the scrollbar relative to the scrollable content.
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollBarLayout {
    /// The scrollbar is placed alongside the content (takes up space in the
    /// layout).
    Alongside,
    /// The scrollbar is overlaid on top of the content (doesn't take up space).
    Overlay,
}

impl Default for ScrollableArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new().fill_max_size(),
            vertical: true,
            horizontal: false,
            scroll_smoothing: 0.05,
            scrollbar_behavior: ScrollBarBehavior::AlwaysVisible,
            scrollbar_track_color: Color::new(0.0, 0.0, 0.0, 0.1),
            scrollbar_thumb_color: Color::new(0.0, 0.0, 0.0, 0.3),
            scrollbar_thumb_hover_color: Color::new(0.0, 0.0, 0.0, 0.5),
            scrollbar_layout: ScrollBarLayout::Alongside,
            controller: None,
            child: None,
        }
    }
}

impl ScrollableArgs {
    /// Sets an external scroll controller.
    pub fn controller(mut self, controller: State<ScrollableController>) -> Self {
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

impl From<&ScrollableArgs> for ScrollableArgs {
    fn from(value: &ScrollableArgs) -> Self {
        value.clone()
    }
}

/// Holds the state for a `scrollable` component, managing scroll position and
/// interaction.
///
/// It tracks the current and target scroll positions, the size of the
/// scrollable content, and focus state.
///
/// The scroll position is smoothly interpolated over time to create a fluid
/// scrolling effect.
#[derive(Clone, PartialEq)]
pub struct ScrollableController {
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// The child component size
    child_size: ComputedData,
    /// The visible area size
    visible_size: ComputedData,
    /// Optional override for the child size used to clamp scroll extents.
    override_child_size: Option<ComputedData>,
    /// Last frame time for delta time calculation
    last_frame_nanos: Option<u64>,
    /// The state for vertical scrollbar
    scrollbar_state_v: ScrollBarState,
    /// The state for horizontal scrollbar
    scrollbar_state_h: ScrollBarState,
    /// Velocity tracking for touch-driven inertia.
    velocity_tracker: Option<ScrollVelocityTracker>,
    /// Active inertia state after a touch release.
    active_inertia: Option<ActiveInertia>,
}

impl Default for ScrollableController {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollableController {
    /// Creates a new `ScrollableController` with default values.
    pub fn new() -> Self {
        Self {
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            child_size: ComputedData::ZERO,
            visible_size: ComputedData::ZERO,
            override_child_size: None,
            last_frame_nanos: None,
            scrollbar_state_v: ScrollBarState::default(),
            scrollbar_state_h: ScrollBarState::default(),
            velocity_tracker: None,
            active_inertia: None,
        }
    }

    /// Returns the current child position relative to the scrollable container.
    ///
    /// This is primarily useful for components that need to implement custom
    /// virtualization strategies (e.g. lazy lists) and must know the current
    /// scroll offset. Values are clamped by the scroll logic, so consumers
    /// can safely derive their offset from the returned position.
    pub fn child_position(&self) -> PxPosition {
        self.child_position
    }

    /// Returns the currently visible viewport size of the scrollable container.
    pub fn visible_size(&self) -> ComputedData {
        self.visible_size
    }

    pub(crate) fn child_size(&self) -> ComputedData {
        self.child_size
    }

    /// Overrides the child size used for scroll extent calculation.
    pub fn override_child_size(&mut self, size: ComputedData) {
        self.override_child_size = Some(size);
    }

    pub(crate) fn target_position(&self) -> PxPosition {
        self.target_position
    }

    pub(crate) fn set_target_position(&mut self, target: PxPosition) {
        self.target_position = target;
    }

    /// Instantly sets the scroll position without animation.
    ///
    /// This is useful for restoring a saved scroll position when remounting
    /// a component.
    pub fn set_scroll_position(&mut self, position: PxPosition) {
        self.child_position = position;
        self.target_position = position;
    }

    /// Updates the scroll position based on time-based interpolation
    /// Returns true if the position changed (needs redraw)
    pub(crate) fn update_scroll_position(&mut self, frame_nanos: u64, smoothing: f32) -> bool {
        // Calculate delta time
        let delta_time = if let Some(last_frame_nanos) = self.last_frame_nanos {
            frame_nanos.saturating_sub(last_frame_nanos) as f32 / 1_000_000_000.0
        } else {
            0.016 // Assume 60fps for first frame
        };

        self.last_frame_nanos = Some(frame_nanos);

        // Calculate the difference between target and current position
        let diff_x = self.target_position.x.to_f32() - self.child_position.x.to_f32();
        let diff_y = self.target_position.y.to_f32() - self.child_position.y.to_f32();

        // If we're close enough to target, snap to it
        if diff_x.abs() < 1.0 && diff_y.abs() < 1.0 {
            if self.child_position != self.target_position {
                self.child_position = self.target_position;
                return true;
            }
            return false;
        }

        // Use simple velocity-based movement for consistent behavior
        // Higher smoothing = slower movement
        let mut movement_factor = (1.0 - smoothing) * delta_time * 60.0;

        // CRITICAL FIX: Clamp the movement factor to a maximum of 1.0.
        if movement_factor > 1.0 {
            movement_factor = 1.0;
        }
        let old_position = self.child_position;

        self.child_position = PxPosition {
            x: Px::saturating_from_f32(self.child_position.x.to_f32() + diff_x * movement_factor),
            y: Px::saturating_from_f32(self.child_position.y.to_f32() + diff_y * movement_factor),
        };

        // Return true if position changed significantly
        old_position != self.child_position
    }

    fn cancel_inertia(&mut self) {
        self.active_inertia = None;
    }

    fn push_touch_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        self.cancel_inertia();
        let tracker = self
            .velocity_tracker
            .get_or_insert_with(|| ScrollVelocityTracker::new(now));
        tracker.push_delta(now, dx, dy);
    }

    fn end_touch_scroll(&mut self, now: Instant) {
        let Some(mut tracker) = self.velocity_tracker.take() else {
            return;
        };
        if let Some((avg_vx, avg_vy)) = tracker.resolve(now) {
            let velocity_magnitude = (avg_vx * avg_vx + avg_vy * avg_vy).sqrt();
            if velocity_magnitude > SCROLL_INERTIA_START_THRESHOLD {
                let (vx, vy) = clamp_inertia_velocity(avg_vx, avg_vy);
                self.active_inertia = Some(ActiveInertia {
                    velocity_x: vx,
                    velocity_y: vy,
                    last_tick_time: now,
                });
            }
        }
    }

    fn should_trigger_idle_inertia(&self, now: Instant) -> bool {
        self.active_inertia.is_none()
            && self
                .velocity_tracker
                .as_ref()
                .is_some_and(|tracker| tracker.is_idle(now))
    }

    fn advance_inertia(
        &mut self,
        now: Instant,
        container_size: &ComputedData,
        vertical_scrollable: bool,
        horizontal_scrollable: bool,
    ) {
        let Some(mut inertia) = self.active_inertia.take() else {
            return;
        };
        let delta_time = now.duration_since(inertia.last_tick_time).as_secs_f32();
        if delta_time <= 0.0 {
            self.active_inertia = Some(inertia);
            return;
        }

        let delta_x = inertia.velocity_x * delta_time;
        let delta_y = inertia.velocity_y * delta_time;
        if delta_x.abs() > 0.01 || delta_y.abs() > 0.01 {
            let new_target = self.target_position.saturating_offset(
                Px::saturating_from_f32(delta_x),
                Px::saturating_from_f32(delta_y),
            );
            let constrained_target = constrain_position(
                new_target,
                &self.child_size,
                container_size,
                vertical_scrollable,
                horizontal_scrollable,
            );
            let consumed_x = constrained_target.x.to_f32() - self.target_position.x.to_f32();
            let consumed_y = constrained_target.y.to_f32() - self.target_position.y.to_f32();
            self.target_position = constrained_target;
            if consumed_x.abs() <= f32::EPSILON {
                inertia.velocity_x = 0.0;
            }
            if consumed_y.abs() <= f32::EPSILON {
                inertia.velocity_y = 0.0;
            }
        }

        let decay = (-SCROLL_INERTIA_DECAY_CONSTANT * delta_time).exp();
        inertia.velocity_x *= decay;
        inertia.velocity_y *= decay;
        inertia.last_tick_time = now;

        if inertia.velocity_x.abs() >= SCROLL_INERTIA_MIN_VELOCITY
            || inertia.velocity_y.abs() >= SCROLL_INERTIA_MIN_VELOCITY
        {
            self.active_inertia = Some(inertia);
        }
    }

    fn has_pending_animation_frame(&self) -> bool {
        self.child_position != self.target_position
            || self.active_inertia.is_some()
            || self.velocity_tracker.is_some()
    }

    pub(crate) fn scrollbar_state_v(&self) -> ScrollBarState {
        self.scrollbar_state_v.clone()
    }

    pub(crate) fn scrollbar_state_h(&self) -> ScrollBarState {
        self.scrollbar_state_h.clone()
    }
}

impl ScrollVelocityTracker {
    fn new(now: Instant) -> Self {
        Self {
            samples: VecDeque::new(),
            last_sample_time: now,
        }
    }

    fn push_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        let delta_time = now.duration_since(self.last_sample_time).as_secs_f32();
        self.last_sample_time = now;
        if delta_time <= 0.0 {
            return;
        }

        let vx = dx / delta_time;
        let vy = dy / delta_time;
        let (vx, vy) = clamp_inertia_velocity(vx, vy);
        self.samples.push_back((now, vx, vy));
        self.prune(now);
    }

    fn resolve(&mut self, now: Instant) -> Option<(f32, f32)> {
        self.prune(now);

        if self.samples.is_empty() {
            return None;
        }

        let idle_time = now.duration_since(self.last_sample_time);

        let mut weighted_sum_x = 0.0f32;
        let mut weighted_sum_y = 0.0f32;
        let mut total_weight = 0.0f32;
        let window_secs = SCROLL_VELOCITY_SAMPLE_WINDOW
            .as_secs_f32()
            .max(f32::EPSILON);

        for &(timestamp, vx, vy) in &self.samples {
            let age_secs = now
                .duration_since(timestamp)
                .as_secs_f32()
                .clamp(0.0, window_secs);
            let weight = (window_secs - age_secs).max(0.0);
            if weight > 0.0 {
                weighted_sum_x += vx * weight;
                weighted_sum_y += vy * weight;
                total_weight += weight;
            }
        }

        if total_weight <= f32::EPSILON {
            self.samples.clear();
            return None;
        }

        let avg_x = weighted_sum_x / total_weight;
        let avg_y = weighted_sum_y / total_weight;

        let damping = 1.0 - idle_time.as_secs_f32() / SCROLL_VELOCITY_IDLE_CUTOFF.as_secs_f32();
        let damping = damping.clamp(0.0, 1.0);
        let (avg_x, avg_y) = clamp_inertia_velocity(avg_x * damping, avg_y * damping);

        Some((avg_x, avg_y))
    }

    fn is_idle(&self, now: Instant) -> bool {
        now.duration_since(self.last_sample_time) >= SCROLL_VELOCITY_IDLE_CUTOFF
    }

    fn prune(&mut self, now: Instant) {
        while let Some(&(timestamp, _, _)) = self.samples.front() {
            if now.duration_since(timestamp) > SCROLL_VELOCITY_SAMPLE_WINDOW {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ScrollableAlongsideLayout {
    vertical: bool,
    horizontal: bool,
}

impl LayoutSpec for ScrollableAlongsideLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let mut final_size = ComputedData::ZERO;
        let mut content_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        if self.vertical {
            let scrollbar_node_id = input.children_ids()[1];
            let size = input.measure_child_in_parent_constraint(scrollbar_node_id)?;
            content_constraint.width -= size.width;
            final_size.width += size.width;
        }

        if self.horizontal {
            let scrollbar_node_id = if self.vertical {
                input.children_ids()[2]
            } else {
                input.children_ids()[1]
            };
            let size = input.measure_child_in_parent_constraint(scrollbar_node_id)?;
            content_constraint.height -= size.height;
            final_size.height += size.height;
        }

        let content_node_id = input.children_ids()[0];
        let content_measurement = input.measure_child(content_node_id, &content_constraint)?;
        final_size.width += content_measurement.width;
        final_size.height += content_measurement.height;

        output.place_child(content_node_id, PxPosition::ZERO);
        if self.vertical {
            output.place_child(
                input.children_ids()[1],
                PxPosition::new(content_measurement.width, Px::ZERO),
            );
        }
        if self.horizontal {
            let scrollbar_node_id = if self.vertical {
                input.children_ids()[2]
            } else {
                input.children_ids()[1]
            };
            output.place_child(
                scrollbar_node_id,
                PxPosition::new(Px::ZERO, content_measurement.height),
            );
        }

        Ok(final_size)
    }
}

#[derive(Clone)]
struct ScrollableInnerLayout {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    child_position: PxPosition,
    has_override: bool,
}

impl PartialEq for ScrollableInnerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.child_position == other.child_position
            && self.has_override == other.has_override
    }
}

impl LayoutSpec for ScrollableInnerLayout {
    fn measure(
        &self,
        input: &LayoutInput<'_>,
        output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        let merged_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );
        let mut child_constraint = merged_constraint;

        if self.vertical {
            child_constraint.height = DimensionValue::Wrap {
                min: None,
                max: None,
            };
        }
        if self.horizontal {
            child_constraint.width = DimensionValue::Wrap {
                min: None,
                max: None,
            };
        }

        let child_node_id = input.children_ids()[0];
        let child_measurement = input.measure_child(child_node_id, &child_constraint)?;
        let current_child_position = self.child_position;
        self.controller.with_mut(|c| {
            if let Some(override_size) = c.override_child_size.take() {
                c.child_size = override_size;
            } else {
                c.child_size = child_measurement;
            }
        });

        output.place_child(child_node_id, current_child_position);

        let mut width = resolve_dimension(merged_constraint.width, child_measurement.width);
        let mut height = resolve_dimension(merged_constraint.height, child_measurement.height);

        if let Some(parent_max_width) = input.parent_constraint().width().get_max() {
            width = width.min(parent_max_width);
        }
        if let Some(parent_max_height) = input.parent_constraint().height().get_max() {
            height = height.min(parent_max_height);
        }

        let computed_data = ComputedData { width, height };
        self.controller.with_mut(|c| c.visible_size = computed_data);
        Ok(computed_data)
    }

    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().clips_children = true;
    }
}

#[derive(Clone, PartialEq)]
struct ScrollableAlongsideArgs {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: RenderSlot,
}

#[derive(Clone, PartialEq)]
struct ScrollableOverlayArgs {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_args_v: ScrollBarArgs,
    scrollbar_args_h: ScrollBarArgs,
    child: RenderSlot,
}

#[derive(Clone, PartialEq)]
struct ScrollableInnerArgs {
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    controller: State<ScrollableController>,
    scrollbar_state_v: ScrollBarState,
    scrollbar_state_h: ScrollBarState,
    child: RenderSlot,
}

/// # scrollable
///
/// Creates a container that makes its content scrollable when it overflows.
///
/// ## Usage
///
/// Wrap a large component or a long list of items to allow the user to scroll
/// through them.
///
/// ## Parameters
///
/// - `args` — props for this component; see [`ScrollableArgs`].
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     column::{ColumnArgs, column},
///     modifier::ModifierExt as _,
///     scrollable::{ScrollableArgs, scrollable},
///     text::{TextArgs, text},
/// };
/// use tessera_ui::{Dp, Modifier, tessera};
///
/// #[tessera]
/// fn demo() {
///     let render_args = ScrollableArgs {
///         modifier: Modifier::new().height(Dp(100.0)),
///         ..Default::default()
///     }
///     .child(|| {
///         column(ColumnArgs::default(), |scope| {
///             for i in 0..20 {
///                 let text_content = format!("Item #{}", i + 1);
///                 scope.child(move || {
///                     text(&TextArgs::default().text(text_content.clone()));
///                 });
///             }
///         });
///     });
///     scrollable(&render_args);
/// }
/// ```
#[tessera]
pub fn scrollable(args: &ScrollableArgs) {
    let args = args.clone();
    let controller = args
        .controller
        .unwrap_or_else(|| remember(ScrollableController::new));
    let child = args.child.clone().unwrap_or_else(|| RenderSlot::new(|| {}));
    scrollable_node(args, controller, child);
}

fn scrollable_node(
    args: ScrollableArgs,
    controller: State<ScrollableController>,
    child: RenderSlot,
) {
    let modifier = args.modifier.clone();

    // Create separate ScrollBarArgs for vertical and horizontal scrollbars
    let scrollbar_args_v = ScrollBarArgs {
        total: controller.with(|c| c.child_size().height),
        visible: controller.with(|c| c.visible_size().height),
        offset: controller.with(|c| c.child_position().y),
        thickness: Dp(8.0), // Default scrollbar thickness
        state: controller,
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
        scrollbar_state: Some(controller.with(|c| c.scrollbar_state_v())),
    };

    let scrollbar_args_h = ScrollBarArgs {
        total: controller.with(|c| c.child_size().width),
        visible: controller.with(|c| c.visible_size().width),
        offset: controller.with(|c| c.child_position().x),
        thickness: Dp(8.0), // Default scrollbar thickness
        state: controller,
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        track_color: args.scrollbar_track_color,
        thumb_color: args.scrollbar_thumb_color,
        thumb_hover_color: args.scrollbar_thumb_hover_color,
        scrollbar_state: Some(controller.with(|c| c.scrollbar_state_h())),
    };

    match args.scrollbar_layout {
        ScrollBarLayout::Alongside => {
            let child = child.clone();
            modifier.run(move || {
                let render_args = ScrollableAlongsideArgs {
                    controller,
                    vertical: args.vertical,
                    horizontal: args.horizontal,
                    scroll_smoothing: args.scroll_smoothing,
                    scrollbar_behavior: args.scrollbar_behavior.clone(),
                    scrollbar_args_v: scrollbar_args_v.clone(),
                    scrollbar_args_h: scrollbar_args_h.clone(),
                    child: child.clone(),
                };
                scrollable_with_alongside_scrollbar(&render_args);
            });
        }
        ScrollBarLayout::Overlay => {
            let child = child.clone();
            modifier.run(move || {
                let render_args = ScrollableOverlayArgs {
                    controller,
                    vertical: args.vertical,
                    horizontal: args.horizontal,
                    scroll_smoothing: args.scroll_smoothing,
                    scrollbar_behavior: args.scrollbar_behavior.clone(),
                    scrollbar_args_v: scrollbar_args_v.clone(),
                    scrollbar_args_h: scrollbar_args_h.clone(),
                    child: child.clone(),
                };
                scrollable_with_overlay_scrollbar(&render_args);
            });
        }
    }
}

#[tessera]
fn scrollable_with_alongside_scrollbar(args: &ScrollableAlongsideArgs) {
    let controller = args.controller;
    let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());

    let inner_args = ScrollableInnerArgs {
        vertical: args.vertical,
        horizontal: args.horizontal,
        scroll_smoothing: args.scroll_smoothing,
        scrollbar_behavior: args.scrollbar_behavior.clone(),
        controller,
        scrollbar_state_v: scrollbar_v_state.clone(),
        scrollbar_state_h: scrollbar_h_state.clone(),
        child: args.child.clone(),
    };
    scrollable_inner(&inner_args);

    if args.vertical {
        let mut scrollbar_args = args.scrollbar_args_v.clone();
        scrollbar_args.scrollbar_state = Some(scrollbar_v_state);
        scrollbar_v(&scrollbar_args);
    }

    if args.horizontal {
        let mut scrollbar_args = args.scrollbar_args_h.clone();
        scrollbar_args.scrollbar_state = Some(scrollbar_h_state);
        scrollbar_h(&scrollbar_args);
    }

    layout(ScrollableAlongsideLayout {
        vertical: args.vertical,
        horizontal: args.horizontal,
    });
}

#[tessera]
fn scrollable_with_overlay_scrollbar(args: &ScrollableOverlayArgs) {
    let args = args.clone();
    let controller = args.controller;
    let child = args.child;
    let scrollbar_args_v = args.scrollbar_args_v;
    let scrollbar_args_h = args.scrollbar_args_h;
    let vertical = args.vertical;
    let horizontal = args.horizontal;
    let scroll_smoothing = args.scroll_smoothing;
    let scrollbar_behavior = args.scrollbar_behavior;

    boxed(
        BoxedArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .alignment(Alignment::BottomEnd),
        move |scope| {
            scope.child({
                let child = child.clone();
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                let scrollbar_behavior = scrollbar_behavior.clone();
                move || {
                    let inner_args = ScrollableInnerArgs {
                        vertical,
                        horizontal,
                        scroll_smoothing,
                        scrollbar_behavior: scrollbar_behavior.clone(),
                        controller,
                        scrollbar_state_v: scrollbar_v_state.clone(),
                        scrollbar_state_h: scrollbar_h_state.clone(),
                        child: child.clone(),
                    };
                    scrollable_inner(&inner_args);
                }
            });
            scope.child({
                let scrollbar_args_v = scrollbar_args_v.clone();
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                move || {
                    if vertical {
                        let mut scrollbar_args = scrollbar_args_v.clone();
                        scrollbar_args.scrollbar_state = Some(scrollbar_v_state.clone());
                        scrollbar_v(&scrollbar_args);
                    }
                }
            });
            scope.child({
                let scrollbar_args_h = scrollbar_args_h.clone();
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                move || {
                    if horizontal {
                        let mut scrollbar_args = scrollbar_args_h.clone();
                        scrollbar_args.scrollbar_state = Some(scrollbar_h_state.clone());
                        scrollbar_h(&scrollbar_args);
                    }
                }
            });
        },
    );
}

// Helpers to resolve DimensionValue into concrete Px sizes.
// This reduces duplication in the measurement code and lowers cyclomatic
// complexity.
fn clamp_wrap(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    min.unwrap_or(Px(0))
        .max(measure)
        .min(max.unwrap_or(Px::MAX))
}

fn fill_value(min: Option<Px>, max: Option<Px>, measure: Px) -> Px {
    max.expect("Seems that you are trying to fill an infinite dimension, which is not allowed")
        .max(measure)
        .max(min.unwrap_or(Px(0)))
}

fn resolve_dimension(dim: DimensionValue, measure: Px) -> Px {
    match dim {
        DimensionValue::Fixed(v) => v,
        DimensionValue::Wrap { min, max } => clamp_wrap(min, max, measure),
        DimensionValue::Fill { min, max } => fill_value(min, max, measure),
    }
}

#[tessera]
fn scrollable_inner(args: &ScrollableInnerArgs) {
    let args = args.clone();
    let scrollbar_state_v = args.scrollbar_state_v.clone();
    let scrollbar_state_h = args.scrollbar_state_h.clone();
    let controller = args.controller;
    let frame_nanos = current_frame_nanos();
    controller.with_mut(|c| c.update_scroll_position(frame_nanos, args.scroll_smoothing));
    if controller.with(|c| c.has_pending_animation_frame()) {
        let controller_for_frame = controller;
        let smoothing = args.scroll_smoothing;
        receive_frame_nanos(move |frame_nanos| {
            let has_pending_animation_frame = controller_for_frame.with_mut(|c| {
                c.update_scroll_position(frame_nanos, smoothing);
                c.has_pending_animation_frame()
            });
            if has_pending_animation_frame {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let child_position = controller.with(|c| c.child_position());
    let has_override = controller.with(|c| c.override_child_size.is_some());
    layout(ScrollableInnerLayout {
        controller,
        vertical: args.vertical,
        horizontal: args.horizontal,
        child_position,
        has_override,
    });

    // Handle scroll input and position updates
    input_handler(move |input| {
        let size = input.computed_data;
        let cursor_pos_option = input.cursor_position_rel;
        let is_cursor_in_component = cursor_pos_option
            .map(|pos| is_position_inside_bounds(size, pos))
            .unwrap_or(false);
        let now = Instant::now();
        let frame_nanos = current_frame_nanos();
        let should_handle_scroll = is_cursor_in_component;

        if should_handle_scroll {
            let mut remaining_events: Vec<CursorEvent> = Vec::new();
            for cursor_event in input.cursor_events.iter() {
                match &cursor_event.content {
                    CursorEventContent::Scroll(scroll_event) => {
                        controller.with_mut(|c| c.cancel_inertia());
                        let scroll_delta_x = scroll_event.delta_x;
                        let scroll_delta_y = scroll_event.delta_y;
                        let (consumed_x, consumed_y) = controller.with_mut(|c| {
                            let current_target = c.target_position;
                            let new_target = current_target.saturating_offset(
                                Px::saturating_from_f32(scroll_delta_x),
                                Px::saturating_from_f32(scroll_delta_y),
                            );
                            let child_size = c.child_size;
                            let constrained_target = constrain_position(
                                new_target,
                                &child_size,
                                &input.computed_data,
                                args.vertical,
                                args.horizontal,
                            );
                            c.set_target_position(constrained_target);
                            (
                                constrained_target.x.to_f32() - current_target.x.to_f32(),
                                constrained_target.y.to_f32() - current_target.y.to_f32(),
                            )
                        });

                        let remaining_x = scroll_delta_x - consumed_x;
                        let remaining_y = scroll_delta_y - consumed_y;

                        if scroll_event.source == ScrollEventSource::Touch
                            && (consumed_x.abs() > f32::EPSILON || consumed_y.abs() > f32::EPSILON)
                        {
                            controller.with_mut(|c| {
                                c.push_touch_delta(cursor_event.timestamp, consumed_x, consumed_y);
                            });
                        }

                        if matches!(args.scrollbar_behavior, ScrollBarBehavior::AutoHide)
                            && (consumed_x.abs() > f32::EPSILON || consumed_y.abs() > f32::EPSILON)
                        {
                            if args.vertical {
                                let mut scrollbar_state = scrollbar_state_v.write();
                                scrollbar_state.last_scroll_activity_frame_nanos =
                                    Some(frame_nanos);
                                scrollbar_state.should_be_visible = true;
                            }
                            if args.horizontal {
                                let mut scrollbar_state = scrollbar_state_h.write();
                                scrollbar_state.last_scroll_activity_frame_nanos =
                                    Some(frame_nanos);
                                scrollbar_state.should_be_visible = true;
                            }
                        }

                        if remaining_x.abs() > f32::EPSILON || remaining_y.abs() > f32::EPSILON {
                            let mut event = cursor_event.clone();
                            if let CursorEventContent::Scroll(scroll_event) = &mut event.content {
                                scroll_event.delta_x = remaining_x;
                                scroll_event.delta_y = remaining_y;
                            }
                            remaining_events.push(event);
                        }
                    }
                    CursorEventContent::Pressed(_) => {
                        controller.with_mut(|c| c.cancel_inertia());
                        remaining_events.push(cursor_event.clone());
                    }
                    CursorEventContent::Released(_) => {
                        controller.with_mut(|c| c.end_touch_scroll(cursor_event.timestamp));
                        remaining_events.push(cursor_event.clone());
                    }
                }
            }

            // Apply bound constraints to the child position
            // To make sure we constrain the target position at least once per frame
            let target = controller.with(|c| c.target_position());
            let child_size = controller.with(|c| c.child_size());
            let constrained_position = constrain_position(
                target,
                &child_size,
                &input.computed_data,
                args.vertical,
                args.horizontal,
            );
            controller.with_mut(|c| c.set_target_position(constrained_position));

            input.cursor_events.clear();
            input.cursor_events.extend(remaining_events);
        }

        if !is_cursor_in_component {
            controller.with_mut(|c| {
                if c.should_trigger_idle_inertia(now) {
                    c.end_touch_scroll(now);
                }
            });
        }

        controller.with_mut(|c| {
            c.advance_inertia(now, &input.computed_data, args.vertical, args.horizontal);
        });
    });

    // Add child component
    args.child.render();
}

/// Constrains a position to stay within the scrollable bounds.
///
/// Split per-axis logic into a helper to simplify reasoning and reduce
/// cyclomatic complexity.
fn constrain_axis(pos: Px, child_len: Px, container_len: Px) -> Px {
    if child_len <= container_len {
        return Px::ZERO;
    }

    if pos > Px::ZERO {
        Px::ZERO
    } else if pos.saturating_add(child_len) < container_len {
        container_len.saturating_sub(child_len)
    } else {
        pos
    }
}

fn constrain_position(
    position: PxPosition,
    child_size: &ComputedData,
    container_size: &ComputedData,
    vertical_scrollable: bool,
    horizontal_scrollable: bool,
) -> PxPosition {
    let x = if horizontal_scrollable {
        constrain_axis(position.x, child_size.width, container_size.width)
    } else {
        Px::ZERO
    };

    let y = if vertical_scrollable {
        constrain_axis(position.y, child_size.height, container_size.height)
    } else {
        Px::ZERO
    };

    PxPosition { x, y }
}
