//! A container that allows its content to be scrolled.
//!
//! ## Usage
//!
//! Use to display content that might overflow the available space.
pub(crate) mod scrollbar;
use std::{collections::VecDeque, time::Duration};

use tessera_ui::{
    CallbackWith, Color, ComputedData, Constraint, DimensionValue, Dp, MeasurementError, Modifier,
    PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot, ScrollEventSource, State,
    current_frame_nanos,
    focus::FocusRevealRequest,
    layout::{
        LayoutInput, LayoutOutput, LayoutPolicy, PlacementInput, RenderInput, RenderPolicy,
        layout_primitive,
    },
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    receive_frame_nanos, remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    gesture_recognizer::{ScrollRecognizer, TapRecognizer},
    modifier::ModifierExt,
    nested_scroll::{NestedScrollConnection, ScrollDelta, ScrollVelocity},
    pos_misc::is_position_inside_bounds,
    scrollable::scrollbar::{ScrollBarState, scrollbar_h, scrollbar_v},
};

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
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScrollBarBehavior {
    /// The scrollbar is always visible.
    #[default]
    AlwaysVisible,
    /// The scrollbar is only visible when scrolling.
    AutoHide,
    /// No scrollbar at all.
    Hidden,
}

/// Defines the layout of the scrollbar relative to the scrollable content.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ScrollBarLayout {
    /// The scrollbar is placed alongside the content (takes up space in the
    /// layout).
    #[default]
    Alongside,
    /// The scrollbar is overlaid on top of the content (doesn't take up space).
    Overlay,
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

    #[cfg(test)]
    pub(crate) fn set_visible_size_for_test(&mut self, size: ComputedData) {
        self.visible_size = size;
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
        if diff_x.abs() <= 1.0 && diff_y.abs() <= 1.0 {
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

        // If interpolation rounds back to the same pixel, snap to target to
        // avoid an endless pending-animation loop.
        if old_position == self.child_position && self.child_position != self.target_position {
            self.child_position = self.target_position;
            return true;
        }

        // Return true if position changed significantly
        old_position != self.child_position
    }

    fn cancel_inertia(&mut self) {
        self.active_inertia = None;
    }

    fn apply_scroll_delta(
        &mut self,
        delta: ScrollDelta,
        container_size: &ComputedData,
        vertical_scrollable: bool,
        horizontal_scrollable: bool,
    ) -> ScrollDelta {
        let current_target = self.target_position;
        let new_target = current_target.saturating_offset(
            Px::saturating_from_f32(delta.x),
            Px::saturating_from_f32(delta.y),
        );
        let constrained_target = constrain_position(
            new_target,
            &self.child_size,
            container_size,
            vertical_scrollable,
            horizontal_scrollable,
        );
        self.target_position = constrained_target;
        ScrollDelta::new(
            constrained_target.x.to_f32() - current_target.x.to_f32(),
            constrained_target.y.to_f32() - current_target.y.to_f32(),
        )
    }

    fn push_touch_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        self.cancel_inertia();
        let tracker = self
            .velocity_tracker
            .get_or_insert_with(|| ScrollVelocityTracker::new(now));
        tracker.push_delta(now, dx, dy);
    }

    fn resolve_touch_velocity(&mut self, now: Instant) -> ScrollVelocity {
        let Some(mut tracker) = self.velocity_tracker.take() else {
            return ScrollVelocity::ZERO;
        };
        if let Some((avg_vx, avg_vy)) = tracker.resolve(now) {
            let velocity_magnitude = (avg_vx * avg_vx + avg_vy * avg_vy).sqrt();
            if velocity_magnitude > SCROLL_INERTIA_START_THRESHOLD {
                let (vx, vy) = clamp_inertia_velocity(avg_vx, avg_vy);
                return ScrollVelocity::new(vx, vy);
            }
        }
        ScrollVelocity::ZERO
    }

    fn start_inertia(&mut self, now: Instant, velocity: ScrollVelocity) {
        if velocity.is_zero() {
            return;
        }
        self.active_inertia = Some(ActiveInertia {
            velocity_x: velocity.x,
            velocity_y: velocity.y,
            last_tick_time: now,
        });
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

impl LayoutPolicy for ScrollableAlongsideLayout {
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
    has_override: bool,
}

impl PartialEq for ScrollableInnerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.has_override == other.has_override
    }
}

impl LayoutPolicy for ScrollableInnerLayout {
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
        let next_child_size = self
            .controller
            .with(|c| c.override_child_size.unwrap_or(child_measurement));
        let needs_child_size_update = self.controller.with(|c| c.child_size != next_child_size);
        if needs_child_size_update {
            self.controller.with_mut(|c| c.child_size = next_child_size);
        }

        let current_child_position = self.controller.with(|c| c.child_position());
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
        let needs_visible_size_update = self.controller.with(|c| c.visible_size != computed_data);
        if needs_visible_size_update {
            self.controller.with_mut(|c| c.visible_size = computed_data);
        }
        Ok(computed_data)
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.has_override == other.has_override
    }

    fn place_children(&self, input: &PlacementInput<'_>, output: &mut LayoutOutput<'_>) -> bool {
        let Some(&child_node_id) = input.children_ids().first() else {
            return true;
        };
        let child_position = self.controller.with(|c| c.child_position());
        output.place_child(child_node_id, child_position);
        true
    }
}

impl RenderPolicy for ScrollableInnerLayout {
    fn record(&self, input: &RenderInput<'_>) {
        input.metadata_mut().set_clips_children(true);
    }
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
/// - `modifier` — optional modifier chain applied to the scrollable subtree.
/// - `vertical` — whether vertical scrolling is enabled.
/// - `horizontal` — whether horizontal scrolling is enabled.
/// - `scroll_smoothing` — optional smoothing factor for animated scrolling.
/// - `scrollbar_behavior` — scrollbar visibility behavior.
/// - `scrollbar_track_color` — optional scrollbar track color.
/// - `scrollbar_thumb_color` — optional scrollbar thumb color.
/// - `scrollbar_thumb_hover_color` — optional scrollbar thumb hover color.
/// - `scrollbar_layout` — layout of the scrollbar relative to content.
/// - `controller` — optional external scroll controller.
/// - `child` — optional scrollable child content.
///
/// ## Examples
///
/// ```
/// use tessera_components::{
///     column::column, modifier::ModifierExt as _, scrollable::scrollable, text::text,
/// };
/// use tessera_ui::{Dp, Modifier, tessera};
///
/// #[tessera]
/// fn demo() {
///     scrollable()
///         .modifier(Modifier::new().height(Dp(100.0)))
///         .child(|| {
///             column().children(|| {
///                 for i in 0..20 {
///                     let text_content = format!("Item #{}", i + 1);
///                     text().content(text_content);
///                 }
///             });
///         });
/// }
/// ```
#[tessera]
pub fn scrollable(
    #[prop(skip_setter)] modifier: Option<Modifier>,
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    #[prop(skip_setter)] scrollbar_track_color: Option<Color>,
    #[prop(skip_setter)] scrollbar_thumb_color: Option<Color>,
    #[prop(skip_setter)] scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: ScrollBarLayout,
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    child: Option<RenderSlot>,
) {
    let controller = controller.unwrap_or_else(|| remember(ScrollableController::new));
    let child = child.unwrap_or_else(RenderSlot::empty);
    let modifier = modifier.unwrap_or_else(|| Modifier::new().fill_max_size());
    let scrollbar_track_color = scrollbar_track_color.unwrap_or(Color::new(0.0, 0.0, 0.0, 0.1));
    let scrollbar_thumb_color = scrollbar_thumb_color.unwrap_or(Color::new(0.0, 0.0, 0.0, 0.3));
    let scrollbar_thumb_hover_color =
        scrollbar_thumb_hover_color.unwrap_or(Color::new(0.0, 0.0, 0.0, 0.5));

    match scrollbar_layout {
        ScrollBarLayout::Alongside => {
            layout_primitive().modifier(modifier).child(move || {
                scrollable_with_alongside_scrollbar()
                    .controller_internal(controller)
                    .vertical(vertical)
                    .horizontal(horizontal)
                    .scroll_smoothing(scroll_smoothing)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .scrollbar_track_color(scrollbar_track_color)
                    .scrollbar_thumb_color(scrollbar_thumb_color)
                    .scrollbar_thumb_hover_color(scrollbar_thumb_hover_color)
                    .child_slot(child);
            });
        }
        ScrollBarLayout::Overlay => {
            layout_primitive().modifier(modifier).child(move || {
                scrollable_with_overlay_scrollbar()
                    .controller_internal(controller)
                    .vertical(vertical)
                    .horizontal(horizontal)
                    .scroll_smoothing(scroll_smoothing)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .scrollbar_track_color(scrollbar_track_color)
                    .scrollbar_thumb_color(scrollbar_thumb_color)
                    .scrollbar_thumb_hover_color(scrollbar_thumb_hover_color)
                    .child_slot(child);
            });
        }
    }
}

#[allow(missing_docs)]
impl ScrollableBuilder {
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.props.modifier = Some(modifier);
        self
    }

    pub fn scrollbar_track_color(mut self, color: Color) -> Self {
        self.props.scrollbar_track_color = Some(color);
        self
    }

    pub fn scrollbar_thumb_color(mut self, color: Color) -> Self {
        self.props.scrollbar_thumb_color = Some(color);
        self
    }

    pub fn scrollbar_thumb_hover_color(mut self, color: Color) -> Self {
        self.props.scrollbar_thumb_hover_color = Some(color);
        self
    }

    pub fn controller(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }
}

#[allow(missing_docs)]
impl ScrollableWithAlongsideScrollbarBuilder {
    fn controller_internal(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    fn child_slot(mut self, child: RenderSlot) -> Self {
        self.props.child = Some(child);
        self
    }
}

#[allow(missing_docs)]
impl ScrollableWithOverlayScrollbarBuilder {
    fn controller_internal(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    fn child_slot(mut self, child: RenderSlot) -> Self {
        self.props.child = Some(child);
        self
    }
}

#[allow(missing_docs)]
impl ScrollableViewportBuilder {
    fn controller_internal(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    fn child_slot(mut self, child: RenderSlot) -> Self {
        self.props.child = Some(child);
        self
    }

    fn scrollbar_state_v_internal(mut self, scrollbar_state_v: ScrollBarState) -> Self {
        self.props.scrollbar_state_v = Some(scrollbar_state_v);
        self
    }

    fn scrollbar_state_h_internal(mut self, scrollbar_state_h: ScrollBarState) -> Self {
        self.props.scrollbar_state_h = Some(scrollbar_state_h);
        self
    }
}

impl ScrollbarVBoundBuilder {
    fn controller_internal(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    fn scrollbar_state_internal(mut self, scrollbar_state: ScrollBarState) -> Self {
        self.props.scrollbar_state = Some(scrollbar_state);
        self
    }
}

impl ScrollbarHBoundBuilder {
    fn controller_internal(mut self, controller: State<ScrollableController>) -> Self {
        self.props.controller = Some(controller);
        self
    }

    fn scrollbar_state_internal(mut self, scrollbar_state: ScrollBarState) -> Self {
        self.props.scrollbar_state = Some(scrollbar_state);
        self
    }
}

#[tessera]
fn scrollbar_v_bound(
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    thickness: Dp,
    scrollbar_behavior: ScrollBarBehavior,
    track_color: Color,
    thumb_color: Color,
    thumb_hover_color: Color,
    #[prop(skip_setter)] scrollbar_state: Option<ScrollBarState>,
) {
    let controller = controller.expect("scrollbar_v_bound requires controller");
    scrollbar_v()
        .total(controller.with(|c| c.child_size().height))
        .visible(controller.with(|c| c.visible_size().height))
        .offset(controller.with(|c| c.child_position().y))
        .thickness(thickness)
        .state_internal(controller)
        .scrollbar_behavior(scrollbar_behavior)
        .track_color(track_color)
        .thumb_color(thumb_color)
        .thumb_hover_color(thumb_hover_color)
        .scrollbar_state_internal(
            scrollbar_state.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_v())),
        );
}

#[tessera]
fn scrollbar_h_bound(
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    thickness: Dp,
    scrollbar_behavior: ScrollBarBehavior,
    track_color: Color,
    thumb_color: Color,
    thumb_hover_color: Color,
    #[prop(skip_setter)] scrollbar_state: Option<ScrollBarState>,
) {
    let controller = controller.expect("scrollbar_h_bound requires controller");
    scrollbar_h()
        .total(controller.with(|c| c.child_size().width))
        .visible(controller.with(|c| c.visible_size().width))
        .offset(controller.with(|c| c.child_position().x))
        .thickness(thickness)
        .state_internal(controller)
        .scrollbar_behavior(scrollbar_behavior)
        .track_color(track_color)
        .thumb_color(thumb_color)
        .thumb_hover_color(thumb_hover_color)
        .scrollbar_state_internal(
            scrollbar_state.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_h())),
        );
}

#[tessera]
fn scrollable_with_alongside_scrollbar(
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    scrollbar_thumb_hover_color: Color,
    child: Option<RenderSlot>,
) {
    let controller = controller.expect("scrollable_with_alongside_scrollbar requires controller");
    let child = child.unwrap_or_else(RenderSlot::empty);
    let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());

    layout_primitive()
        .layout_policy(ScrollableAlongsideLayout {
            vertical,
            horizontal,
        })
        .child(move || {
            scrollable_viewport()
                .vertical(vertical)
                .horizontal(horizontal)
                .scroll_smoothing(scroll_smoothing)
                .scrollbar_behavior(scrollbar_behavior.clone())
                .controller_internal(controller)
                .scrollbar_state_v_internal(scrollbar_v_state.clone())
                .scrollbar_state_h_internal(scrollbar_h_state.clone())
                .child_slot(child);

            if vertical {
                scrollbar_v_bound()
                    .controller_internal(controller)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .thickness(Dp(8.0))
                    .track_color(scrollbar_track_color)
                    .thumb_color(scrollbar_thumb_color)
                    .thumb_hover_color(scrollbar_thumb_hover_color)
                    .scrollbar_state_internal(scrollbar_v_state.clone());
            }

            if horizontal {
                scrollbar_h_bound()
                    .controller_internal(controller)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .thickness(Dp(8.0))
                    .track_color(scrollbar_track_color)
                    .thumb_color(scrollbar_thumb_color)
                    .thumb_hover_color(scrollbar_thumb_hover_color)
                    .scrollbar_state_internal(scrollbar_h_state.clone());
            }
        });
}

#[tessera]
fn scrollable_with_overlay_scrollbar(
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_track_color: Color,
    scrollbar_thumb_color: Color,
    scrollbar_thumb_hover_color: Color,
    child: Option<RenderSlot>,
) {
    let controller = controller.expect("scrollable_with_overlay_scrollbar requires controller");
    let child = child.unwrap_or_else(RenderSlot::empty);

    boxed()
        .modifier(Modifier::new().fill_max_size())
        .alignment(Alignment::BottomEnd)
        .children(move || {
            {
                let child = child;
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                let scrollbar_behavior = scrollbar_behavior.clone();
                scrollable_viewport()
                    .vertical(vertical)
                    .horizontal(horizontal)
                    .scroll_smoothing(scroll_smoothing)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .controller_internal(controller)
                    .scrollbar_state_v_internal(scrollbar_v_state.clone())
                    .scrollbar_state_h_internal(scrollbar_h_state.clone())
                    .child_slot(child);
            };
            {
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                let scrollbar_behavior = scrollbar_behavior.clone();
                if vertical {
                    scrollbar_v_bound()
                        .controller_internal(controller)
                        .scrollbar_behavior(scrollbar_behavior.clone())
                        .thickness(Dp(8.0))
                        .track_color(scrollbar_track_color)
                        .thumb_color(scrollbar_thumb_color)
                        .thumb_hover_color(scrollbar_thumb_hover_color)
                        .scrollbar_state_internal(scrollbar_v_state.clone());
                }
            };
            {
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                let scrollbar_behavior = scrollbar_behavior.clone();
                if horizontal {
                    scrollbar_h_bound()
                        .controller_internal(controller)
                        .scrollbar_behavior(scrollbar_behavior.clone())
                        .thickness(Dp(8.0))
                        .track_color(scrollbar_track_color)
                        .thumb_color(scrollbar_thumb_color)
                        .thumb_hover_color(scrollbar_thumb_hover_color)
                        .scrollbar_state_internal(scrollbar_h_state.clone());
                }
            };
        });
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

struct ScrollableViewportPointerModifierNode {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_state_v: ScrollBarState,
    scrollbar_state_h: ScrollBarState,
    tap_recognizer: State<TapRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
    nested_scroll_connection: Option<NestedScrollConnection>,
}

struct ScrollableViewportInputArgs {
    base: Modifier,
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    scrollbar_behavior: ScrollBarBehavior,
    scrollbar_state_v: ScrollBarState,
    scrollbar_state_h: ScrollBarState,
    tap_recognizer: State<TapRecognizer>,
    scroll_recognizer: State<ScrollRecognizer>,
    nested_scroll_connection: Option<NestedScrollConnection>,
}

fn apply_scrollable_viewport_input_modifier(args: ScrollableViewportInputArgs) -> Modifier {
    let ScrollableViewportInputArgs {
        base,
        controller,
        vertical,
        horizontal,
        scrollbar_behavior,
        scrollbar_state_v,
        scrollbar_state_h,
        tap_recognizer,
        scroll_recognizer,
        nested_scroll_connection,
    } = args;
    base.push_pointer_input(ScrollableViewportPointerModifierNode {
        controller,
        vertical,
        horizontal,
        scrollbar_behavior,
        scrollbar_state_v,
        scrollbar_state_h,
        tap_recognizer,
        scroll_recognizer,
        nested_scroll_connection,
    })
}

impl PointerInputModifierNode for ScrollableViewportPointerModifierNode {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        let is_cursor_in_component = input
            .cursor_position_rel
            .map(|pos| is_position_inside_bounds(input.computed_data, pos))
            .unwrap_or(false);
        let now = Instant::now();
        let frame_nanos = current_frame_nanos();
        let should_handle_scroll = is_cursor_in_component;
        let tap_result = self.tap_recognizer.with_mut(|recognizer| {
            recognizer.update(
                input.pass,
                input.pointer_changes.as_mut_slice(),
                input.cursor_position_rel,
                is_cursor_in_component,
            )
        });

        if tap_result.pressed && self.controller.with(|c| c.active_inertia.is_some()) {
            self.controller.with_mut(|c| c.cancel_inertia());
        }

        if let Some(release_timestamp) = tap_result.release_timestamp {
            let available_velocity = self
                .controller
                .with_mut(|c| c.resolve_touch_velocity(release_timestamp));
            if !available_velocity.is_zero() {
                let consumed_velocity = self
                    .nested_scroll_connection
                    .as_ref()
                    .map(|connection| connection.pre_fling(available_velocity))
                    .unwrap_or(ScrollVelocity::ZERO);
                let remaining_velocity = available_velocity - consumed_velocity;
                self.controller
                    .with_mut(|c| c.start_inertia(release_timestamp, remaining_velocity));
            }
        }

        if should_handle_scroll {
            self.scroll_recognizer.with_mut(|recognizer| {
                recognizer.for_each(
                    input.pass,
                    input.pointer_changes.as_mut_slice(),
                    |context, scroll_event| {
                        if self.controller.with(|c| c.active_inertia.is_some()) {
                            self.controller.with_mut(|c| c.cancel_inertia());
                        }
                        let available =
                            ScrollDelta::new(scroll_event.delta_x, scroll_event.delta_y);
                        let parent_pre_consumed = self
                            .nested_scroll_connection
                            .as_ref()
                            .map(|connection| connection.pre_scroll(available, scroll_event.source))
                            .unwrap_or(ScrollDelta::ZERO);
                        let available_after_pre = available - parent_pre_consumed;
                        let child_consumed = self.controller.with_mut(|c| {
                            c.apply_scroll_delta(
                                available_after_pre,
                                &input.computed_data,
                                self.vertical,
                                self.horizontal,
                            )
                        });
                        let available_after_child = available_after_pre - child_consumed;
                        let parent_post_consumed = self
                            .nested_scroll_connection
                            .as_ref()
                            .map(|connection| {
                                connection.post_scroll(
                                    child_consumed,
                                    available_after_child,
                                    scroll_event.source,
                                )
                            })
                            .unwrap_or(ScrollDelta::ZERO);
                        let remaining = available_after_child - parent_post_consumed;

                        if scroll_event.source == ScrollEventSource::Touch
                            && !child_consumed.is_zero()
                        {
                            self.controller.with_mut(|c| {
                                c.push_touch_delta(
                                    context.timestamp,
                                    child_consumed.x,
                                    child_consumed.y,
                                );
                            });
                        }

                        if matches!(self.scrollbar_behavior, ScrollBarBehavior::AutoHide)
                            && !child_consumed.is_zero()
                        {
                            if self.vertical {
                                let mut scrollbar_state = self.scrollbar_state_v.write();
                                scrollbar_state.last_scroll_activity_frame_nanos =
                                    Some(frame_nanos);
                                scrollbar_state.should_be_visible = true;
                            }
                            if self.horizontal {
                                let mut scrollbar_state = self.scrollbar_state_h.write();
                                scrollbar_state.last_scroll_activity_frame_nanos =
                                    Some(frame_nanos);
                                scrollbar_state.should_be_visible = true;
                            }
                        }

                        scroll_event.delta_x = remaining.x;
                        scroll_event.delta_y = remaining.y;
                    },
                );
            });

            let target = self.controller.with(|c| c.target_position());
            let child_size = self.controller.with(|c| c.child_size());
            let constrained_position = constrain_position(
                target,
                &child_size,
                &input.computed_data,
                self.vertical,
                self.horizontal,
            );
            if target != constrained_position {
                self.controller
                    .with_mut(|c| c.set_target_position(constrained_position));
            }
        }

        if !is_cursor_in_component {
            let should_trigger_idle_inertia =
                self.controller.with(|c| c.should_trigger_idle_inertia(now));
            if should_trigger_idle_inertia {
                let available_velocity =
                    self.controller.with_mut(|c| c.resolve_touch_velocity(now));
                if !available_velocity.is_zero() {
                    let consumed_velocity = self
                        .nested_scroll_connection
                        .as_ref()
                        .map(|connection| connection.pre_fling(available_velocity))
                        .unwrap_or(ScrollVelocity::ZERO);
                    let remaining_velocity = available_velocity - consumed_velocity;
                    self.controller
                        .with_mut(|c| c.start_inertia(now, remaining_velocity));
                }
            }
        }

        if self.controller.with(|c| c.active_inertia.is_some()) {
            self.controller.with_mut(|c| {
                c.advance_inertia(now, &input.computed_data, self.vertical, self.horizontal);
            });
        }
    }
}

#[tessera]
fn scrollable_viewport(
    vertical: bool,
    horizontal: bool,
    scroll_smoothing: f32,
    scrollbar_behavior: ScrollBarBehavior,
    #[prop(skip_setter)] controller: Option<State<ScrollableController>>,
    #[prop(skip_setter)] scrollbar_state_v: Option<ScrollBarState>,
    #[prop(skip_setter)] scrollbar_state_h: Option<ScrollBarState>,
    child: Option<RenderSlot>,
) {
    let controller = controller.expect("scrollable_viewport requires controller");
    let scrollbar_state_v =
        scrollbar_state_v.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_v()));
    let scrollbar_state_h =
        scrollbar_state_h.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_h()));
    let child = child.unwrap_or_else(RenderSlot::empty);
    if controller.with(|c| c.has_pending_animation_frame()) {
        let smoothing = scroll_smoothing;
        receive_frame_nanos(move |frame_nanos| {
            let has_pending_animation_frame = controller.with_mut(|c| {
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
    let has_override = controller.with(|c| c.override_child_size.is_some());
    let tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    let nested_scroll_connection =
        use_context::<NestedScrollConnection>().map(|context| context.get());
    let modifier = apply_scrollable_viewport_input_modifier(ScrollableViewportInputArgs {
        base: Modifier::new(),
        controller,
        vertical,
        horizontal,
        scrollbar_behavior: scrollbar_behavior.clone(),
        scrollbar_state_v,
        scrollbar_state_h,
        tap_recognizer,
        scroll_recognizer,
        nested_scroll_connection,
    });
    let modifier = if vertical || horizontal {
        apply_scrollable_focus_reveal_modifier(modifier, controller, vertical, horizontal)
    } else {
        modifier
    };
    let policy = ScrollableInnerLayout {
        controller,
        vertical,
        horizontal,
        has_override,
    };
    layout_primitive()
        .modifier(modifier)
        .layout_policy(policy.clone())
        .render_policy(policy)
        .child(move || child.render());
}

fn apply_scrollable_focus_reveal_modifier(
    base: Modifier,
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
) -> Modifier {
    base.focus_reveal_handler(CallbackWith::new(move |request: FocusRevealRequest| {
        let (current_position, child_size, visible_size) =
            controller.with(|c| (c.child_position(), c.child_size(), c.visible_size()));
        let mut desired_position = current_position;

        if horizontal {
            desired_position.x = reveal_axis_position(
                current_position.x,
                request.target_rect.x,
                request.target_rect.x + request.target_rect.width,
                request.viewport_rect.x,
                request.viewport_rect.x + request.viewport_rect.width,
            );
        }

        if vertical {
            desired_position.y = reveal_axis_position(
                current_position.y,
                request.target_rect.y,
                request.target_rect.y + request.target_rect.height,
                request.viewport_rect.y,
                request.viewport_rect.y + request.viewport_rect.height,
            );
        }

        let constrained_position = constrain_position(
            desired_position,
            &child_size,
            &visible_size,
            vertical,
            horizontal,
        );
        if constrained_position == current_position {
            return false;
        }

        controller.with_mut(|c| {
            c.cancel_inertia();
            c.velocity_tracker = None;
            c.set_scroll_position(constrained_position);
        });
        true
    }))
}

fn reveal_axis_position(
    current: Px,
    target_start: Px,
    target_end: Px,
    viewport_start: Px,
    viewport_end: Px,
) -> Px {
    if target_start < viewport_start {
        current + (viewport_start - target_start)
    } else if target_end > viewport_end {
        current - (target_end - viewport_end)
    } else {
        current
    }
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
