//! Scrollable viewports driven by a [`ScrollableController`].
//!
//! ## Usage
//!
//! Turn a layout node into a scroll viewport with `Modifier::vertical_scroll`
//! or `Modifier::horizontal_scroll` to display content that overflows its
//! bounds.
pub mod scrollbar;
use std::time::Duration;

use tessera_foundation::{
    gesture::{ScrollRecognizer, TapRecognizer},
    scroll_physics::{ExponentialInertia, ScrollVelocityTracker as FoundationVelocityTracker},
};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutModifierChild,
    LayoutModifierInput, LayoutModifierNode, LayoutResult, MeasurementError, Modifier,
    PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot, ScrollDeltaUnit,
    ScrollEventSource, State, current_frame_nanos, current_frame_time,
    focus::FocusRevealRequest,
    layout::{MeasureScope, PlacementResult, PlacementScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    normalize_platform_scroll_delta, receive_frame_nanos, remember, tessera,
    time::Instant,
    use_context,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    modifier::ModifierExt,
    nested_scroll::{NestedScrollConnection, ScrollDelta, ScrollVelocity},
    pos_misc::is_position_inside_bounds,
};

pub use crate::scrollable::scrollbar::{
    ScrollBarState, ScrollbarDefaults, scrollbar_h, scrollbar_v,
};

const SCROLL_INERTIA_DECAY_CONSTANT: f32 = 5.0;
const SCROLL_INERTIA_MIN_VELOCITY: f32 = 10.0;
const SCROLL_INERTIA_START_THRESHOLD: f32 = 50.0;
const SCROLL_INERTIA_MAX_VELOCITY: f32 = 6000.0;
const SCROLL_VELOCITY_SAMPLE_WINDOW: Duration = Duration::from_millis(90);
const SCROLL_VELOCITY_IDLE_CUTOFF: Duration = Duration::from_millis(65);

/// Fraction of the remaining distance an animated scroll keeps after one 60Hz
/// frame.
///
/// `1 - DEFAULT_SCROLL_SMOOTHING` of the remaining distance is covered per
/// frame, so `0.75` reaches 95% of a step in roughly 175ms and 99% in roughly
/// 270ms. That reads as an eased movement instead of a jump while staying
/// responsive to rapid wheel input.
pub(crate) const DEFAULT_SCROLL_SMOOTHING: f32 = 0.75;

/// Pixel magnitude at which wheel input counts as a discrete notch.
///
/// Mouse wheels deliver stepwise input while trackpads deliver a continuous
/// stream, and browsers report both as pixel deltas (Chrome normalises a mouse
/// wheel notch to roughly 100px). Deltas at or above this value are animated
/// like a notch; smaller ones are applied directly so touchpads keep tracking
/// the finger 1:1.
const DISCRETE_WHEEL_STEP_PX: f32 = 40.0;

fn clamp_inertia_velocity(vx: f32, vy: f32) -> (f32, f32) {
    let magnitude = (vx * vx + vy * vy).sqrt();
    if !magnitude.is_finite() {
        return (0.0, 0.0);
    }
    if magnitude > SCROLL_INERTIA_MAX_VELOCITY {
        let scale = SCROLL_INERTIA_MAX_VELOCITY / magnitude;
        (vx * scale, vy * scale)
    } else {
        (vx, vy)
    }
}
/// Returns whether a wheel delta represents a discrete notch instead of a
/// continuous stream of pointer movement.
fn is_discrete_wheel_step(unit: ScrollDeltaUnit, delta_x: f32, delta_y: f32) -> bool {
    match unit {
        ScrollDeltaUnit::Line => true,
        ScrollDeltaUnit::Pixel => delta_x.abs().max(delta_y.abs()) >= DISCRETE_WHEEL_STEP_PX,
    }
}

fn clamp_scroll_coordinate(value: f32, boundary: f32, enabled: bool) -> f32 {
    if !enabled {
        0.0
    } else if value < boundary {
        boundary
    } else if value > 0.0 {
        0.0
    } else {
        value
    }
}

fn normalize_scroll_delta(
    delta_x: f32,
    delta_y: f32,
    unit: ScrollDeltaUnit,
    source: ScrollEventSource,
) -> ScrollDelta {
    let (delta_x, delta_y) = normalize_platform_scroll_delta(delta_x, delta_y, unit, source);
    ScrollDelta::new(delta_x, delta_y)
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
    density: f64,
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// Floating-point target retained between rendered pixel positions.
    target_position_f32: (f32, f32),
    /// Floating-point current position retained between rendered pixel
    /// positions.
    child_position_f32: (f32, f32),
    // Preserve interpolation steps smaller than f32 precision in long virtualized lists.
    animation_position: (f64, f64),
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
    velocity_tracker: Option<FoundationVelocityTracker>,
    /// Active inertia state after a touch release.
    active_inertia: Option<ExponentialInertia>,
    inertia_last_tick: Option<Instant>,
    /// Whether anything other than construction has positioned this viewport.
    ///
    /// Virtualized containers need to tell "this viewport has not been set up
    /// yet" apart from "the viewport sits at the top", because a fresh
    /// viewport and a viewport scrolled back to the top both report a zero
    /// offset.
    positioned: bool,
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
            density: tessera_ui::dp::scale_factor(),
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            target_position_f32: (0.0, 0.0),
            child_position_f32: (0.0, 0.0),
            animation_position: (0.0, 0.0),
            child_size: ComputedData::ZERO,
            visible_size: ComputedData::ZERO,
            override_child_size: None,
            last_frame_nanos: None,
            scrollbar_state_v: ScrollBarState::default(),
            scrollbar_state_h: ScrollBarState::default(),
            velocity_tracker: None,
            active_inertia: None,
            inertia_last_tick: None,
            positioned: false,
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

    /// Returns whether this viewport has been positioned by scrolling yet.
    ///
    /// A freshly created controller reports `false`. Any scroll input,
    /// programmatic position or animation marks it as positioned, so
    /// consumers can distinguish an untouched viewport from one that
    /// currently sits at the top.
    pub fn is_positioned(&self) -> bool {
        self.positioned
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
    ///
    /// The new extent is immediately available to scrolling and scrollbars.
    pub fn override_child_size(&mut self, size: ComputedData) {
        self.update_density(tessera_ui::dp::scale_factor());
        self.override_child_size = Some(size);
        self.child_size = size;
    }

    pub(crate) fn target_position(&self) -> PxPosition {
        self.target_position
    }

    pub(crate) fn set_target_position(&mut self, target: PxPosition) {
        self.update_density(tessera_ui::dp::scale_factor());
        self.cancel_inertia();
        self.velocity_tracker = None;
        self.positioned = true;
        self.target_position = target;
        self.target_position_f32 = (target.x, target.y);
    }

    /// Instantly sets the scroll position without animation.
    ///
    /// This is useful for restoring a saved scroll position when remounting
    /// a component.
    pub fn set_scroll_position(&mut self, position: PxPosition) {
        self.update_density(tessera_ui::dp::scale_factor());
        self.cancel_inertia();
        self.velocity_tracker = None;
        self.last_frame_nanos = None;
        self.positioned = true;
        self.child_position = position;
        self.target_position = position;
        self.child_position_f32 = (position.x, position.y);
        self.animation_position = (position.x as f64, position.y as f64);
        self.target_position_f32 = (position.x, position.y);
    }

    pub(crate) fn translate_scroll_position(&mut self, delta: PxPosition) {
        // Layout anchoring changes the content origin, not the remaining
        // motion.
        self.child_position = self.child_position + delta;
        self.target_position = self.target_position + delta;
        self.child_position_f32.0 += delta.x;
        self.child_position_f32.1 += delta.y;
        self.target_position_f32.0 += delta.x;
        self.target_position_f32.1 += delta.y;
        self.animation_position.0 += f64::from(delta.x);
        self.animation_position.1 += f64::from(delta.y);
    }

    pub(crate) fn update_density(&mut self, density: f64) -> f64 {
        let ratio = density / self.density;
        if density == self.density {
            return 1.0;
        }
        self.density = density;
        self.animation_position.0 *= ratio;
        self.animation_position.1 *= ratio;
        self.child_position_f32 = (
            self.animation_position.0 as f32,
            self.animation_position.1 as f32,
        );
        self.target_position_f32.0 = (f64::from(self.target_position_f32.0) * ratio) as f32;
        self.target_position_f32.1 = (f64::from(self.target_position_f32.1) * ratio) as f32;
        self.child_position =
            PxPosition::from_f32_xy(self.child_position_f32.0, self.child_position_f32.1);
        self.target_position =
            PxPosition::from_f32_xy(self.target_position_f32.0, self.target_position_f32.1);
        let scale_size = |size: ComputedData| ComputedData {
            width: Px((f64::from(size.width.0) * ratio) as i32),
            height: Px((f64::from(size.height.0) * ratio) as i32),
        };
        self.child_size = scale_size(self.child_size);
        self.visible_size = scale_size(self.visible_size);
        self.override_child_size = self.override_child_size.map(scale_size);
        if let Some(inertia) = &mut self.active_inertia {
            inertia.position_x *= ratio as f32;
            inertia.position_y *= ratio as f32;
            inertia.velocity_x *= ratio as f32;
            inertia.velocity_y *= ratio as f32;
        }
        // Samples collected in the previous physical units cannot be mixed with
        // new input.
        self.velocity_tracker = None;
        ratio
    }

    /// Updates the scroll position based on time-based interpolation
    /// Returns true if the position changed (needs redraw)
    pub(crate) fn update_scroll_position(&mut self, frame_nanos: u64, smoothing: f32) -> bool {
        let delta_time = self
            .last_frame_nanos
            .map(|last| frame_nanos.saturating_sub(last) as f32 / 1_000_000_000.0)
            .unwrap_or(1.0 / 60.0)
            .clamp(0.0, 0.25);
        self.last_frame_nanos = Some(frame_nanos);
        let old = self.child_position;
        let (target_x, target_y) = self.target_position_f32;
        let (current_x, current_y) = self.animation_position;
        let smoothing = smoothing.clamp(0.0, 0.9999);
        let response = if smoothing == 0.0 {
            f32::INFINITY
        } else {
            -(smoothing).ln() * 60.0
        };
        let factor = if response.is_infinite() {
            1.0
        } else {
            1.0 - (-response * delta_time).exp()
        };
        self.animation_position = (
            current_x + (target_x as f64 - current_x) * factor as f64,
            current_y + (target_y as f64 - current_y) * factor as f64,
        );
        if (target_x as f64 - self.animation_position.0).abs() < 0.01 {
            self.animation_position.0 = target_x as f64;
        }
        if (target_y as f64 - self.animation_position.1).abs() < 0.01 {
            self.animation_position.1 = target_y as f64;
        }
        self.child_position_f32 = (
            self.animation_position.0 as f32,
            self.animation_position.1 as f32,
        );
        self.child_position =
            PxPosition::from_f32_xy(self.child_position_f32.0, self.child_position_f32.1);
        if !self.has_pending_animation_frame() {
            self.last_frame_nanos = None;
            self.animation_position = (target_x as f64, target_y as f64);
        }
        if old != self.child_position {
            self.positioned = true;
        }
        old != self.child_position
    }

    fn cancel_inertia(&mut self) {
        self.active_inertia = None;
        self.inertia_last_tick = None;
    }

    // Direct-manipulation input starts at the displayed floating-point
    // position, discarding any pending line-wheel animation rather than
    // jumping to it.
    fn apply_input_delta(
        &mut self,
        delta: ScrollDelta,
        source: ScrollEventSource,
        unit: ScrollDeltaUnit,
        container_size: &ComputedData,
        vertical: bool,
        horizontal: bool,
    ) -> ScrollDelta {
        self.cancel_inertia();
        if source != ScrollEventSource::Touch {
            self.velocity_tracker = None;
        }
        let immediate =
            source == ScrollEventSource::Touch || !is_discrete_wheel_step(unit, delta.x, delta.y);
        if immediate {
            self.target_position_f32 = self.child_position_f32;
            self.target_position = self.child_position;
        }
        let consumed = self.apply_scroll_delta(delta, container_size, vertical, horizontal);
        if immediate {
            self.child_position_f32 = self.target_position_f32;
            self.child_position = self.target_position;
            self.animation_position = (
                self.child_position_f32.0 as f64,
                self.child_position_f32.1 as f64,
            );
        }
        consumed
    }
    fn apply_scroll_delta(
        &mut self,
        delta: ScrollDelta,
        container_size: &ComputedData,
        vertical_scrollable: bool,
        horizontal_scrollable: bool,
    ) -> ScrollDelta {
        self.positioned = true;
        let current_target = self.target_position_f32;
        let proposed = (current_target.0 + delta.x, current_target.1 + delta.y);
        let extent_size = self.override_child_size.unwrap_or(self.child_size);
        let constrained_target = constrain_position(
            PxPosition::new(Px::new(i32::MIN), Px::new(i32::MIN)),
            &extent_size,
            container_size,
            vertical_scrollable,
            horizontal_scrollable,
        );
        let constrained = (constrained_target.x, constrained_target.y);
        let next = (
            clamp_scroll_coordinate(proposed.0, constrained.0, horizontal_scrollable),
            clamp_scroll_coordinate(proposed.1, constrained.1, vertical_scrollable),
        );
        self.target_position_f32 = next;
        self.target_position = PxPosition::from_f32_xy(next.0, next.1);
        ScrollDelta::new(next.0 - current_target.0, next.1 - current_target.1)
    }

    fn push_touch_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        let tracker = self.velocity_tracker.get_or_insert_with(|| {
            FoundationVelocityTracker::new(
                now,
                SCROLL_VELOCITY_SAMPLE_WINDOW,
                SCROLL_VELOCITY_IDLE_CUTOFF,
            )
        });
        tracker.push_delta(now, dx, dy);
    }

    fn resolve_touch_velocity(&mut self, now: Instant) -> ScrollVelocity {
        let Some(mut tracker) = self.velocity_tracker.take() else {
            return ScrollVelocity::ZERO;
        };
        if let Some(velocity) = tracker.resolve(now) {
            let velocity_magnitude = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
            if velocity_magnitude > SCROLL_INERTIA_START_THRESHOLD {
                let (vx, vy) = clamp_inertia_velocity(velocity.x, velocity.y);
                return ScrollVelocity::new(vx, vy);
            }
        }
        ScrollVelocity::ZERO
    }

    fn start_inertia(&mut self, now: Instant, velocity: ScrollVelocity) {
        if velocity.is_zero() {
            return;
        }
        self.inertia_last_tick = Some(now);
        self.active_inertia = Some(ExponentialInertia::new(
            self.target_position_f32.0,
            self.target_position_f32.1,
            tessera_foundation::scroll_physics::ScrollVelocity::new(velocity.x, velocity.y),
            SCROLL_INERTIA_DECAY_CONSTANT,
        ));
    }

    fn has_pending_animation_frame(&self) -> bool {
        self.child_position_f32 != self.target_position_f32 || self.active_inertia.is_some()
    }

    pub(crate) fn scrollbar_state_v(&self) -> ScrollBarState {
        self.scrollbar_state_v.clone()
    }

    pub(crate) fn scrollbar_state_h(&self) -> ScrollBarState {
        self.scrollbar_state_h.clone()
    }
}

pub(crate) fn synchronize_scroll_density(controller: State<ScrollableController>) {
    let density = tessera_ui::dp::scale_factor();
    if controller.with(|controller| controller.density != density) {
        controller.with_mut(|controller| {
            controller.update_density(density);
        });
    }
}

#[derive(Clone, PartialEq)]
struct ScrollableAlongsideLayout {
    vertical: bool,
    horizontal: bool,
}

impl tessera_ui::LayoutModifierNode for ScrollableAlongsideLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let mut final_size = ComputedData::ZERO;
        let child_constraint = input.parent_constraint().without_min();
        let mut content_constraint = Constraint::new(
            input.parent_constraint().width(),
            input.parent_constraint().height(),
        );

        if self.vertical {
            let scrollbar = children[1];
            let size = scrollbar.measure(input, &child_constraint)?;
            content_constraint.width -= size.width;
            final_size.width += size.width;
        }

        if self.horizontal {
            let scrollbar = if self.vertical {
                children[2]
            } else {
                children[1]
            };
            let size = scrollbar.measure(input, &child_constraint)?;
            content_constraint.height -= size.height;
            final_size.height += size.height;
        }

        let content = children[0];
        let content_measurement = content.measure(input, &content_constraint)?;
        final_size.width += content_measurement.width;
        final_size.height += content_measurement.height;

        result.place_child(content, PxPosition::ZERO);
        if self.vertical {
            result.place_child(
                children[1],
                PxPosition::new(content_measurement.width, Px::ZERO),
            );
        }
        if self.horizontal {
            let scrollbar = if self.vertical {
                children[2]
            } else {
                children[1]
            };
            result.place_child(
                scrollbar,
                PxPosition::new(Px::ZERO, content_measurement.height),
            );
        }

        Ok(result.with_size(final_size))
    }
}

/// Layout modifier that turns a node into a scroll viewport.
///
/// Measures the wrapped content without constraining the scroll axes, reports
/// the constrained viewport size, and places the content at the current scroll
/// offset. Reading the controller during measure registers a layout dependency,
/// so the offset is re-applied when the controller moves.
struct ScrollLayoutModifierNode {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    apply_child_offset: bool,
    has_override: bool,
}

impl PartialEq for ScrollLayoutModifierNode {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.apply_child_offset == other.apply_child_offset
            && self.has_override == other.has_override
    }
}

impl LayoutModifierNode for ScrollLayoutModifierNode {
    fn measure_content(
        &self,
        input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let parent = input.layout_input.parent_constraint();
        let mut child_constraint = *parent.as_ref();
        if self.vertical {
            child_constraint.height = AxisConstraint::NONE;
        }
        if self.horizontal {
            child_constraint.width = AxisConstraint::NONE;
        }

        let child_measurement = child.measure(&child_constraint)?;
        let current_child_position = input.placement(|| {
            let next_child_size = self
                .controller
                .with(|c| c.override_child_size.unwrap_or(child_measurement));
            if self.controller.with(|c| c.child_size != next_child_size) {
                self.controller.with_mut(|c| c.child_size = next_child_size);
            }

            if self.apply_child_offset {
                self.controller.with(|c| c.child_position())
            } else {
                PxPosition::ZERO
            }
        });
        child.place(current_child_position);

        let width = parent.width().clamp(child_measurement.width);
        let height = parent.height().clamp(child_measurement.height);
        let computed_data = ComputedData { width, height };
        input.placement(|| {
            if self.controller.with(|c| c.visible_size != computed_data) {
                self.controller.with_mut(|c| c.visible_size = computed_data);
            }
        });

        Ok(LayoutResult::new(computed_data))
    }

    fn place_children(
        &self,
        _scope: &PlacementScope<'_>,
        inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        // The viewport only shifts its content by the current scroll offset.
        // Re-applying it from cached placements keeps scrolling from forcing a
        // full re-measure of the content subtree.
        let offset = if self.apply_child_offset {
            self.controller.with(|c| c.child_position())
        } else {
            PxPosition::ZERO
        };
        let mut result = PlacementResult::new();
        for (instance_key, position) in inner.iter() {
            result.place(*instance_key, offset + *position);
        }
        Some(result)
    }
}

/// Makes `base` behave as a scroll viewport driven by `controller`.
///
/// Composes the layout (viewport measure + content offset), clipping, wheel and
/// drag input, smoothing frame ticking, focus reveal, and nested-scroll
/// participation.
pub(crate) fn apply_scroll_modifier(
    base: Modifier,
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
) -> Modifier {
    if !(vertical || horizontal) {
        return base;
    }

    synchronize_scroll_density(controller);

    let scrollbar_state_v = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_state_h = controller.with(|c| c.scrollbar_state_h());
    let tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    if controller.with(|c| c.has_pending_animation_frame()) {
        receive_frame_nanos(move |frame_nanos| {
            let pending = controller.with_mut(|c| {
                c.update_scroll_position(frame_nanos, DEFAULT_SCROLL_SMOOTHING);
                c.has_pending_animation_frame()
            });
            if pending {
                tessera_ui::FrameNanosControl::Continue
            } else {
                tessera_ui::FrameNanosControl::Stop
            }
        });
    }
    let has_override = controller.with(|c| c.override_child_size.is_some());
    let nested_scroll_connection =
        use_context::<NestedScrollConnection>().map(|context| context.get());
    let modifier = apply_scrollable_viewport_input_modifier(ScrollableViewportInputArgs {
        base,
        controller,
        vertical,
        horizontal,
        scrollbar_behavior: ScrollBarBehavior::Hidden,
        scrollbar_state_v,
        scrollbar_state_h,
        tap_recognizer,
        scroll_recognizer,
        nested_scroll_connection,
    });
    let modifier =
        apply_scrollable_focus_reveal_modifier(modifier, controller, vertical, horizontal);
    modifier
        .clip_to_bounds()
        .push_layout(ScrollLayoutModifierNode {
            controller,
            vertical,
            horizontal,
            apply_child_offset: true,
            has_override,
        })
}

/// Internal scroll container: a sized viewport with scrollbars around the
/// scroll modifier. Public scrolling is expressed through
/// `Modifier::vertical_scroll` / `Modifier::horizontal_scroll`.
///
/// Retained as a crate-internal helper because it also lays out scrollbar
/// siblings, which a modifier cannot add.
#[tessera]
pub(crate) fn scrollable(
    modifier: Option<Modifier>,
    vertical: Option<bool>,
    horizontal: Option<bool>,
    scroll_smoothing: Option<f32>,
    apply_child_offset: Option<bool>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    scrollbar_layout: Option<ScrollBarLayout>,
    controller: Option<State<ScrollableController>>,
    child: Option<RenderSlot>,
) {
    let vertical = vertical.unwrap_or(false);
    let horizontal = horizontal.unwrap_or(false);
    let scroll_smoothing = scroll_smoothing.unwrap_or(DEFAULT_SCROLL_SMOOTHING);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let scrollbar_layout = scrollbar_layout.unwrap_or_default();
    let controller = controller.unwrap_or_else(|| remember(ScrollableController::new));
    let child = child.unwrap_or_else(RenderSlot::empty);
    let modifier = modifier.unwrap_or_else(|| Modifier::new().fill_max_size());

    match scrollbar_layout {
        ScrollBarLayout::Alongside => {
            layout().modifier(modifier).child(move || {
                scrollable_with_alongside_scrollbar()
                    .controller(controller)
                    .vertical(vertical)
                    .horizontal(horizontal)
                    .scroll_smoothing(scroll_smoothing)
                    .apply_child_offset(apply_child_offset)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .scrollbar_track_color_optional(scrollbar_track_color)
                    .scrollbar_thumb_color_optional(scrollbar_thumb_color)
                    .scrollbar_thumb_hover_color_optional(scrollbar_thumb_hover_color)
                    .child_shared(child);
            });
        }
        ScrollBarLayout::Overlay => {
            layout().modifier(modifier).child(move || {
                scrollable_with_overlay_scrollbar()
                    .controller(controller)
                    .vertical(vertical)
                    .horizontal(horizontal)
                    .scroll_smoothing(scroll_smoothing)
                    .apply_child_offset(apply_child_offset)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .scrollbar_track_color_optional(scrollbar_track_color)
                    .scrollbar_thumb_color_optional(scrollbar_thumb_color)
                    .scrollbar_thumb_hover_color_optional(scrollbar_thumb_hover_color)
                    .child_shared(child);
            });
        }
    }
}

#[tessera]
fn scrollbar_v_bound(
    controller: Option<State<ScrollableController>>,
    thickness: Option<Dp>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    track_color: Option<Color>,
    thumb_color: Option<Color>,
    thumb_hover_color: Option<Color>,
    scrollbar_state: Option<ScrollBarState>,
) {
    let controller = controller.expect("scrollbar_v_bound requires controller");
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    scrollbar_v()
        .total(controller.with(|c| c.child_size().height))
        .visible(controller.with(|c| c.visible_size().height))
        .offset(controller.with(|c| c.child_position().y_px()))
        .thickness_optional(thickness)
        .state(controller)
        .scrollbar_behavior(scrollbar_behavior)
        .track_color_optional(track_color)
        .thumb_color_optional(thumb_color)
        .thumb_hover_color_optional(thumb_hover_color)
        .scrollbar_state(
            scrollbar_state.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_v())),
        );
}

#[tessera]
fn scrollbar_h_bound(
    controller: Option<State<ScrollableController>>,
    thickness: Option<Dp>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    track_color: Option<Color>,
    thumb_color: Option<Color>,
    thumb_hover_color: Option<Color>,
    scrollbar_state: Option<ScrollBarState>,
) {
    let controller = controller.expect("scrollbar_h_bound requires controller");
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    scrollbar_h()
        .total(controller.with(|c| c.child_size().width))
        .visible(controller.with(|c| c.visible_size().width))
        .offset(controller.with(|c| c.child_position().x_px()))
        .thickness_optional(thickness)
        .state(controller)
        .scrollbar_behavior(scrollbar_behavior)
        .track_color_optional(track_color)
        .thumb_color_optional(thumb_color)
        .thumb_hover_color_optional(thumb_hover_color)
        .scrollbar_state(
            scrollbar_state.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_h())),
        );
}

#[tessera]
fn scrollable_with_alongside_scrollbar(
    controller: Option<State<ScrollableController>>,
    vertical: Option<bool>,
    horizontal: Option<bool>,
    scroll_smoothing: Option<f32>,
    apply_child_offset: Option<bool>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    child: Option<RenderSlot>,
) {
    let vertical = vertical.unwrap_or(false);
    let horizontal = horizontal.unwrap_or(false);
    let scroll_smoothing = scroll_smoothing.unwrap_or(DEFAULT_SCROLL_SMOOTHING);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let controller = controller.expect("scrollable_with_alongside_scrollbar requires controller");
    let child = child.unwrap_or_else(RenderSlot::empty);
    let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());

    layout()
        .policy(ScrollableAlongsideLayout {
            vertical,
            horizontal,
        })
        .child(move || {
            scrollable_viewport()
                .vertical(vertical)
                .horizontal(horizontal)
                .scroll_smoothing(scroll_smoothing)
                .apply_child_offset(apply_child_offset)
                .scrollbar_behavior(scrollbar_behavior.clone())
                .controller(controller)
                .scrollbar_state_v(scrollbar_v_state.clone())
                .scrollbar_state_h(scrollbar_h_state.clone())
                .child_shared(child);

            if vertical {
                scrollbar_v_bound()
                    .controller(controller)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .thickness(ScrollbarDefaults::THICKNESS)
                    .track_color_optional(scrollbar_track_color)
                    .thumb_color_optional(scrollbar_thumb_color)
                    .thumb_hover_color_optional(scrollbar_thumb_hover_color)
                    .scrollbar_state(scrollbar_v_state.clone());
            }

            if horizontal {
                scrollbar_h_bound()
                    .controller(controller)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .thickness(ScrollbarDefaults::THICKNESS)
                    .track_color_optional(scrollbar_track_color)
                    .thumb_color_optional(scrollbar_thumb_color)
                    .thumb_hover_color_optional(scrollbar_thumb_hover_color)
                    .scrollbar_state(scrollbar_h_state.clone());
            }
        });
}

#[tessera]
fn scrollable_with_overlay_scrollbar(
    controller: Option<State<ScrollableController>>,
    vertical: Option<bool>,
    horizontal: Option<bool>,
    scroll_smoothing: Option<f32>,
    apply_child_offset: Option<bool>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    scrollbar_track_color: Option<Color>,
    scrollbar_thumb_color: Option<Color>,
    scrollbar_thumb_hover_color: Option<Color>,
    child: Option<RenderSlot>,
) {
    let vertical = vertical.unwrap_or(false);
    let horizontal = horizontal.unwrap_or(false);
    let scroll_smoothing = scroll_smoothing.unwrap_or(DEFAULT_SCROLL_SMOOTHING);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
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
                    .apply_child_offset(apply_child_offset)
                    .scrollbar_behavior(scrollbar_behavior.clone())
                    .controller(controller)
                    .scrollbar_state_v(scrollbar_v_state.clone())
                    .scrollbar_state_h(scrollbar_h_state.clone())
                    .child_shared(child);
            };
            {
                let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
                let scrollbar_behavior = scrollbar_behavior.clone();
                if vertical {
                    scrollbar_v_bound()
                        .controller(controller)
                        .scrollbar_behavior(scrollbar_behavior.clone())
                        .thickness(ScrollbarDefaults::THICKNESS)
                        .track_color_optional(scrollbar_track_color)
                        .thumb_color_optional(scrollbar_thumb_color)
                        .thumb_hover_color_optional(scrollbar_thumb_hover_color)
                        .scrollbar_state(scrollbar_v_state.clone());
                }
            };
            {
                let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());
                let scrollbar_behavior = scrollbar_behavior.clone();
                if horizontal {
                    scrollbar_h_bound()
                        .controller(controller)
                        .scrollbar_behavior(scrollbar_behavior.clone())
                        .thickness(ScrollbarDefaults::THICKNESS)
                        .track_color_optional(scrollbar_track_color)
                        .thumb_color_optional(scrollbar_thumb_color)
                        .thumb_hover_color_optional(scrollbar_thumb_hover_color)
                        .scrollbar_state(scrollbar_h_state.clone());
                }
            };
        });
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
        self.handle_changes(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            input.cursor_position_rel,
            input.computed_data,
            current_frame_time().unwrap_or_else(Instant::now),
            current_frame_nanos(),
        );
    }
}

impl ScrollableViewportPointerModifierNode {
    fn advance_inertia(&self, now: Instant, viewport: &ComputedData) {
        let step = self.controller.with_mut(|controller| {
            let mut inertia = controller.active_inertia.take()?;
            let last = controller.inertia_last_tick.replace(now).unwrap_or(now);
            let displacement = inertia.advance(now.duration_since(last));
            Some((inertia, ScrollDelta::new(displacement.x, displacement.y)))
        });
        let Some((mut inertia, displacement)) = step else {
            return;
        };
        let pre = self
            .nested_scroll_connection
            .as_ref()
            .map(|connection| connection.pre_scroll(displacement, ScrollEventSource::Touch))
            .unwrap_or(ScrollDelta::ZERO);
        let child = self.controller.with_mut(|controller| {
            controller.apply_input_delta(
                displacement - pre,
                ScrollEventSource::Touch,
                ScrollDeltaUnit::Pixel,
                viewport,
                self.vertical,
                self.horizontal,
            )
        });
        let available = displacement - pre - child;
        let post = self
            .nested_scroll_connection
            .as_ref()
            .map(|connection| connection.post_scroll(child, available, ScrollEventSource::Touch))
            .unwrap_or(ScrollDelta::ZERO);
        let remaining = available - post;
        // Subtracting large scroll coordinates loses fractional precision;
        // that rounding residue does not indicate a scroll boundary.
        let target = self
            .controller
            .with(|controller| controller.target_position_f32);
        let tolerance_x = (target.0.abs() * f32::EPSILON * 2.0).max(0.001);
        let tolerance_y = (target.1.abs() * f32::EPSILON * 2.0).max(0.001);
        let mut velocity = ScrollVelocity::ZERO;
        if remaining.x.abs() > tolerance_x {
            velocity.x = inertia.velocity_x;
            inertia.velocity_x = 0.0;
        }
        if remaining.y.abs() > tolerance_y {
            velocity.y = inertia.velocity_y;
            inertia.velocity_y = 0.0;
        }
        let active = inertia.velocity_x.abs() >= SCROLL_INERTIA_MIN_VELOCITY
            || inertia.velocity_y.abs() >= SCROLL_INERTIA_MIN_VELOCITY;
        self.controller.with_mut(|controller| {
            if active {
                controller.active_inertia = Some(inertia);
                controller.inertia_last_tick = Some(now);
            }
        });
        if !active && let Some(connection) = &self.nested_scroll_connection {
            connection.post_fling(velocity);
        }
    }

    fn handle_changes(
        &self,
        pass: tessera_ui::PointerEventPass,
        changes: &mut [tessera_ui::PointerChange],
        cursor_position_rel: Option<PxPosition>,
        computed_data: ComputedData,
        now: Instant,
        frame_nanos: u64,
    ) {
        if pass != tessera_ui::PointerEventPass::Main {
            return;
        }
        let is_cursor_in_component = cursor_position_rel
            .map(|pos| is_position_inside_bounds(computed_data, pos))
            .unwrap_or(false);
        for change in changes {
            let changes = std::slice::from_mut(change);
            let should_handle_scroll = is_cursor_in_component
                || matches!(
                    changes[0].content,
                    tessera_ui::CursorEventContent::Scroll(ref scroll)
                        if scroll.source == ScrollEventSource::Touch
                );
            let tap_result = self
                .tap_recognizer
                .with_mut(|recognizer| recognizer.update(pass, changes, is_cursor_in_component));
            if tap_result.pressed {
                self.controller.with_mut(|c| {
                    c.cancel_inertia();
                    c.target_position_f32 = c.child_position_f32;
                    c.target_position = c.child_position;
                    c.velocity_tracker = Some(FoundationVelocityTracker::new(
                        tap_result.press_timestamp.unwrap_or(now),
                        SCROLL_VELOCITY_SAMPLE_WINDOW,
                        SCROLL_VELOCITY_IDLE_CUTOFF,
                    ));
                });
            }

            // Release is intentionally handled after all scroll samples in this
            // frame.
            if should_handle_scroll {
                self.scroll_recognizer.with_mut(|recognizer| {
                    recognizer.for_each(pass, changes, |context, scroll_event| {
                        if self.controller.with(|c| c.active_inertia.is_some()) {
                            self.controller.with_mut(|c| c.cancel_inertia());
                        }
                        let available = normalize_scroll_delta(
                            scroll_event.delta_x,
                            scroll_event.delta_y,
                            scroll_event.unit,
                            scroll_event.source,
                        );
                        let parent_pre_consumed = self
                            .nested_scroll_connection
                            .as_ref()
                            .map(|connection| connection.pre_scroll(available, scroll_event.source))
                            .unwrap_or(ScrollDelta::ZERO);
                        let available_after_pre = available - parent_pre_consumed;
                        let child_consumed = self.controller.with_mut(|c| {
                            c.apply_input_delta(
                                available_after_pre,
                                scroll_event.source,
                                scroll_event.unit,
                                &computed_data,
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

                        let gesture_consumed =
                            parent_pre_consumed + child_consumed + parent_post_consumed;
                        if scroll_event.source == ScrollEventSource::Touch
                            && !gesture_consumed.is_zero()
                        {
                            self.controller.with_mut(|c| {
                                c.push_touch_delta(
                                    context.timestamp,
                                    gesture_consumed.x,
                                    gesture_consumed.y,
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
                        scroll_event.unit = ScrollDeltaUnit::Pixel;
                    });
                });

                let target = self.controller.with(|c| c.target_position());
                let child_size = self.controller.with(|c| c.child_size());
                let constrained_position = constrain_position(
                    target,
                    &child_size,
                    &computed_data,
                    self.vertical,
                    self.horizontal,
                );
                if target != constrained_position {
                    self.controller
                        .with_mut(|c| c.set_target_position(constrained_position));
                }
            }
            // A captured gesture can end outside the viewport. Resolve only
            // after the last scroll sample, independently of the
            // current hit-test result.
            let cancelled = changes.iter().any(|change| {
                matches!(change.content, tessera_ui::CursorEventContent::Cancelled(_))
            });
            if cancelled {
                self.controller.with_mut(|c| {
                    c.velocity_tracker = None;
                    c.cancel_inertia();
                });
                if let Some(connection) = &self.nested_scroll_connection {
                    connection.post_fling(ScrollVelocity::ZERO);
                }
            } else if let Some(release_timestamp) = tap_result.release_timestamp {
                let available_velocity = self
                    .controller
                    .with_mut(|c| c.resolve_touch_velocity(release_timestamp));
                let consumed_velocity = self
                    .nested_scroll_connection
                    .as_ref()
                    .map(|connection| connection.pre_fling(available_velocity))
                    .unwrap_or(ScrollVelocity::ZERO);
                self.controller.with_mut(|c| {
                    c.start_inertia(release_timestamp, available_velocity - consumed_velocity)
                });
                if self.controller.with(|c| c.active_inertia.is_none()) {
                    if let Some(connection) = &self.nested_scroll_connection {
                        connection.post_fling(ScrollVelocity::ZERO);
                    }
                }
            }
        }
        if self.controller.with(|c| c.active_inertia.is_some()) {
            self.advance_inertia(now, &computed_data);
        }
    }
}

#[tessera]
fn scrollable_viewport(
    vertical: Option<bool>,
    horizontal: Option<bool>,
    scroll_smoothing: Option<f32>,
    apply_child_offset: Option<bool>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    controller: Option<State<ScrollableController>>,
    scrollbar_state_v: Option<ScrollBarState>,
    scrollbar_state_h: Option<ScrollBarState>,
    child: Option<RenderSlot>,
) {
    let vertical = vertical.unwrap_or(false);
    let horizontal = horizontal.unwrap_or(false);
    let scroll_smoothing = scroll_smoothing.unwrap_or(DEFAULT_SCROLL_SMOOTHING);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let controller = controller.expect("scrollable_viewport requires controller");
    synchronize_scroll_density(controller);
    let scrollbar_state_v =
        scrollbar_state_v.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_v()));
    let scrollbar_state_h =
        scrollbar_state_h.unwrap_or_else(|| controller.with(|c| c.scrollbar_state_h()));
    let child = child.unwrap_or_else(RenderSlot::empty);
    let tap_recognizer = remember(TapRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
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
    let modifier = modifier
        .clip_to_bounds()
        .push_layout(ScrollLayoutModifierNode {
            controller,
            vertical,
            horizontal,
            apply_child_offset,
            has_override,
        });
    layout().modifier(modifier).child(move || child.render());
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
                request.target_rect.x.0 as f32,
                (request.target_rect.x + request.target_rect.width).0 as f32,
                request.viewport_rect.x.0 as f32,
                (request.viewport_rect.x + request.viewport_rect.width).0 as f32,
            );
        }

        if vertical {
            desired_position.y = reveal_axis_position(
                current_position.y,
                request.target_rect.y.0 as f32,
                (request.target_rect.y + request.target_rect.height).0 as f32,
                request.viewport_rect.y.0 as f32,
                (request.viewport_rect.y + request.viewport_rect.height).0 as f32,
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
    current: f32,
    target_start: f32,
    target_end: f32,
    viewport_start: f32,
    viewport_end: f32,
) -> f32 {
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
fn constrain_axis(pos: f32, child_len: Px, container_len: Px) -> f32 {
    let child_len = child_len.0 as f32;
    let container_len = container_len.0 as f32;
    if child_len <= container_len {
        return 0.0;
    }

    if pos > 0.0 {
        0.0
    } else if pos + child_len < container_len {
        container_len - child_len
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
        0.0
    };

    let y = if vertical_scrollable {
        constrain_axis(position.y, child_size.height, container_size.height)
    } else {
        0.0
    };

    PxPosition::from_f32_xy(x, y)
}

#[cfg(test)]
mod scroll_tests {
    use super::*;
    fn controller() -> ScrollableController {
        let mut controller = ScrollableController::new();
        controller.child_size = ComputedData {
            width: Px(100),
            height: Px(2000),
        };
        controller.visible_size = ComputedData {
            width: Px(100),
            height: Px(100),
        };
        controller
    }
    #[test]
    fn pixel_input_accumulates_subpixels_without_smoothing() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        for _ in 0..20 {
            controller.apply_input_delta(
                ScrollDelta::new(0.0, -0.1),
                ScrollEventSource::Wheel,
                ScrollDeltaUnit::Pixel,
                &viewport,
                true,
                false,
            );
        }
        assert!((controller.child_position_f32.1 + 2.0).abs() < 0.0001);
        assert!(!controller.has_pending_animation_frame());
    }
    #[test]
    fn smoothing_does_not_snap_when_one_frame_rounds_to_same_pixel() {
        let mut controller = controller();
        controller.set_target_position(PxPosition::new(Px(0), Px(-100)));
        controller.last_frame_nanos = Some(0);
        controller.update_scroll_position(1_000_000, 0.99);
        assert!(controller.child_position_f32.1 > -1.0);
        assert!(controller.has_pending_animation_frame());
        for frame in 1..2000 {
            controller.update_scroll_position(frame * 16_666_667, 0.99);
        }
        assert!(!controller.has_pending_animation_frame());
        assert_eq!(controller.child_position.y, -100.0);
    }

    #[test]
    fn translating_layout_origin_preserves_smoothing_and_idle_state() {
        let mut original = controller();
        original.set_scroll_position(PxPosition::from_f32_xy(-100.25, -200.5));
        original.set_target_position(PxPosition::from_f32_xy(-300.25, -500.5));
        original.update_scroll_position(16_666_667, 0.75);
        let mut translated = original.clone();
        let delta = PxPosition::new(Px(-80), Px(-120));
        translated.translate_scroll_position(delta);
        for frame in 2..100 {
            original.update_scroll_position(frame * 16_666_667, 0.75);
            translated.update_scroll_position(frame * 16_666_667, 0.75);
            let expected = original.child_position() + delta;
            let actual = translated.child_position();
            assert!((actual.x - expected.x).abs() < 0.0001);
            assert!((actual.y - expected.y).abs() < 0.0001);
            assert_eq!(
                translated.has_pending_animation_frame(),
                original.has_pending_animation_frame()
            );
        }
        assert_eq!(
            translated.child_position(),
            PxPosition::from_f32_xy(-380.25, -620.5)
        );
        translated.translate_scroll_position(PxPosition::new(Px(80), Px(120)));
        assert_eq!(translated.child_position(), original.child_position());
        assert!(!translated.has_pending_animation_frame());
    }

    #[test]
    fn density_changes_scale_animation_distance_and_inertial_motion() {
        let mut original = controller();
        original.set_scroll_position(PxPosition::from_f32_xy(-100.25, -200.5));
        original.set_target_position(PxPosition::from_f32_xy(-300.25, -500.5));
        original.update_scroll_position(16_666_667, 0.75);
        let now = Instant::now();
        original.start_inertia(now, ScrollVelocity::new(120.0, -240.0));
        let mut scaled = original.clone();
        scaled.update_density(original.density * 1.5);
        assert_eq!(scaled.last_frame_nanos, original.last_frame_nanos);
        assert_eq!(scaled.inertia_last_tick, Some(now));
        assert_eq!(
            scaled.target_position(),
            PxPosition::from_f32_xy(-450.375, -750.75)
        );
        let remaining = original.target_position().y - original.child_position().y;
        let scaled_remaining = scaled.target_position().y - scaled.child_position().y;
        assert!((scaled_remaining - remaining * 1.5).abs() < 0.0001);
        let mut original_motion = original.active_inertia.expect("active inertia");
        let motion = scaled.active_inertia.as_mut().expect("scaled inertia");
        assert_eq!(motion.velocity_x, 180.0);
        assert_eq!(motion.velocity_y, -360.0);
        let before = original_motion.advance(Duration::from_millis(32));
        let after = motion.advance(Duration::from_millis(32));
        assert!((after.x - before.x * 1.5).abs() < 0.0001);
        assert!((after.y - before.y * 1.5).abs() < 0.0001);
        scaled.update_density(original.density);
        assert_eq!(scaled.child_position(), original.child_position());
        assert_eq!(scaled.target_position(), original.target_position());
        assert!(scaled.has_pending_animation_frame());
    }

    #[test]
    fn large_virtual_offsets_reach_the_target_and_stop_the_animation_clock() {
        for smoothing in [0.75, 0.99] {
            let mut controller = controller();
            controller.set_scroll_position(PxPosition::from_f32_xy(-765_072.0, -1_250_500.0));
            let target = PxPosition::from_f32_xy(-765_576.0, -1_251_004.0);
            controller.set_target_position(target);
            for frame in 1..2000 {
                controller.update_scroll_position(frame * 16_666_667, smoothing);
            }
            assert_eq!(controller.child_position(), target);
            assert!(!controller.has_pending_animation_frame());
        }
    }

    #[test]
    fn overridden_child_extent_allows_scrolling_past_measured_child() {
        let mut controller = controller();
        controller.child_size.height = controller.visible_size.height;
        controller.override_child_size(ComputedData {
            width: controller.child_size.width,
            height: Px(2000),
        });
        let viewport = controller.visible_size;
        controller.apply_input_delta(
            ScrollDelta::new(0.0, -300.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        assert!(controller.target_position.y < 0.0);
    }
    #[test]
    fn touch_and_programmatic_position_interrupt_inertia() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        let now = Instant::now();
        controller.start_inertia(now, ScrollVelocity::new(0.0, -1000.0));
        controller.apply_input_delta(
            ScrollDelta::new(0.0, -2.0),
            ScrollEventSource::Touch,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        assert!(controller.active_inertia.is_none());
        controller.start_inertia(now, ScrollVelocity::new(0.0, -1000.0));
        controller.set_scroll_position(PxPosition::ZERO);
        assert!(!controller.has_pending_animation_frame());
    }
    #[test]
    fn holding_still_does_not_start_inertia_and_release_has_no_velocity() {
        let mut controller = controller();
        let now = Instant::now();
        controller.push_touch_delta(now, 0.0, -10.0);
        controller.push_touch_delta(now + Duration::from_millis(10), 0.0, -10.0);
        assert!(controller.active_inertia.is_none());
        assert!(
            controller
                .resolve_touch_velocity(now + Duration::from_millis(100))
                .is_zero()
        );
    }

    #[test]
    fn line_wheel_steps_reach_the_top_from_the_bottom() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        controller.set_scroll_position(PxPosition::new(Px(0), Px(-1900)));
        let mut frame = 0u64;
        let mut steps = 0;
        while controller.child_position.y != 0.0 && steps < 200 {
            controller.apply_input_delta(
                ScrollDelta::new(0.0, 40.0),
                ScrollEventSource::Wheel,
                ScrollDeltaUnit::Line,
                &viewport,
                true,
                false,
            );
            steps += 1;
            for _ in 0..30 {
                frame += 1;
                controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
            }
        }
        assert_eq!(
            controller.child_position.y,
            0.0,
            "steps={steps} current={:?} target={:?} target_f32={:?} pending={}",
            controller.child_position,
            controller.target_position,
            controller.target_position_f32,
            controller.has_pending_animation_frame()
        );
    }

    #[test]
    fn pixel_overshoot_reaches_the_top_from_the_bottom() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        controller.set_scroll_position(PxPosition::new(Px(0), Px(-1900)));
        controller.apply_input_delta(
            ScrollDelta::new(0.0, 1900.5),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        for frame in 1..=60u64 {
            controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
        }
        assert_eq!(
            controller.child_position.y, 0.0,
            "current={:?}",
            controller.child_position
        );
        assert!(!controller.has_pending_animation_frame());
    }

    #[test]
    fn discrete_wheel_notch_eases_towards_its_target() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        controller.apply_input_delta(
            ScrollDelta::new(0.0, -120.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        // A notch must not teleport: the position only moves once frames run.
        assert_eq!(controller.child_position.y, 0.0);
        assert!(controller.has_pending_animation_frame());

        let mut travelled = Vec::new();
        for frame in 1..=4u64 {
            controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
            travelled.push(controller.child_position.y);
        }
        // Each frame covers less than the whole step and moves forwards.
        assert!(travelled[0] < 0.0 && travelled[0] > -120.0, "{travelled:?}");
        assert!(
            travelled.windows(2).all(|pair| pair[1] <= pair[0]),
            "scroll must stay monotonic: {travelled:?}"
        );
        assert!(
            travelled[3] < travelled[0],
            "later frames must keep easing: {travelled:?}"
        );

        for frame in 5..=60u64 {
            controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
        }
        assert_eq!(controller.child_position.y, -120.0);
        assert!(!controller.has_pending_animation_frame());
    }

    #[test]
    fn continuous_trackpad_delta_keeps_tracking_directly() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        controller.apply_input_delta(
            ScrollDelta::new(0.0, -8.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        assert_eq!(controller.child_position.y, -8.0);
        assert!(!controller.has_pending_animation_frame());
    }

    #[test]
    fn fresh_viewport_is_not_positioned_until_it_scrolls() {
        let controller = controller();
        assert!(!controller.is_positioned());
    }

    #[test]
    fn viewport_stays_positioned_after_scrolling_back_to_the_top() {
        let mut controller = controller();
        let viewport = controller.visible_size;
        controller.apply_input_delta(
            ScrollDelta::new(0.0, -500.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        let mut frame = 0u64;
        for _ in 0..40 {
            frame += 1;
            controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
        }
        assert!(controller.is_positioned());
        assert_eq!(controller.child_position.y, -500.0);
        controller.apply_input_delta(
            ScrollDelta::new(0.0, 500.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        for _ in 0..40 {
            frame += 1;
            controller.update_scroll_position(frame * 16_666_667, DEFAULT_SCROLL_SMOOTHING);
        }
        assert_eq!(controller.child_position(), PxPosition::ZERO);
        assert!(
            controller.is_positioned(),
            "a viewport at the top must not look like a fresh one"
        );
    }

    mod scroll_modifier_api {
        use std::cell::RefCell;

        use tessera_ui::{
            Dp, Modifier, Px, PxPosition, State,
            layout::layout,
            remember, tessera,
            testing::{layout_test, pump_layout_test_session, start_layout_test_session},
        };

        use crate::{
            column::column,
            modifier::{ModifierExt as _, SemanticsArgs},
            spacer::spacer,
        };

        use super::ScrollableController;

        thread_local! {
            static CONTROLLER: RefCell<Option<State<ScrollableController>>> =
                const { RefCell::new(None) };
        }

        #[tessera]
        fn scroll_modifier_case() {
            let controller = remember(ScrollableController::new);
            CONTROLLER.with(|slot| *slot.borrow_mut() = Some(controller));
            layout()
                .modifier(Modifier::new().vertical_scroll(controller))
                .child(move || {
                    column().children(move || {
                        spacer().modifier(Modifier::new().height(Dp(400.0)).semantics(
                            SemanticsArgs {
                                test_tag: Some("inner".to_string()),
                                ..Default::default()
                            },
                        ));
                    });
                });
        }

        #[test]
        fn cached_viewport_publishes_updated_scroll_extent() {
            let mut session = start_layout_test_session(
                layout_test(|| {
                    scroll_modifier_case();
                })
                .viewport_px(100, 100),
            );
            let _ = pump_layout_test_session(&mut session, 0);
            let controller = CONTROLLER.with(|slot| slot.borrow().expect("scroll controller"));
            controller.with_mut(|c| {
                c.override_child_size(tessera_ui::ComputedData {
                    width: Px(100),
                    height: Px(400),
                });
            });
            let _ = pump_layout_test_session(&mut session, 16_666_667);
            controller.with_mut(|c| {
                c.override_child_size(tessera_ui::ComputedData {
                    width: Px(100),
                    height: Px(800),
                });
            });
            let _ = pump_layout_test_session(&mut session, 33_333_334);
            assert_eq!(controller.with(|c| c.child_size().height), Px(800));
            controller.with_mut(|c| c.set_scroll_position(PxPosition::new(Px(0), Px(-650))));
            let snapshot = pump_layout_test_session(&mut session, 50_000_001);
            snapshot.node("inner").y(-650);
        }

        #[test]
        fn vertical_scroll_modifier_reapplies_offset() {
            let mut session = start_layout_test_session(
                layout_test(|| {
                    scroll_modifier_case();
                })
                .viewport_px(100, 100),
            );
            let snapshot = pump_layout_test_session(&mut session, 0);
            snapshot.node("inner").y(0);

            let controller = CONTROLLER.with(|slot| (*slot.borrow()).expect("controller"));
            controller.with_mut(|c| {
                c.set_scroll_position(PxPosition::new(Px(0), Px(-40)));
            });

            let snapshot = pump_layout_test_session(&mut session, 16_666_667);
            snapshot.node("inner").y(-40);
        }
    }
}
