//! A container that allows its content to be scrolled.
//!
//! ## Usage
//!
//! Use to display content that might overflow the available space.
pub(crate) mod scrollbar;
use std::time::Duration;

use tessera_foundation::{
    gesture::{ScrollRecognizer, TapRecognizer},
    scroll_physics::{ExponentialInertia, ScrollVelocityTracker as FoundationVelocityTracker},
};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult,
    MeasurementError, Modifier, PointerInput, PointerInputModifierNode, Px, PxPosition, RenderSlot,
    ScrollDeltaUnit, ScrollEventSource, State, current_frame_nanos,
    focus::FocusRevealRequest,
    layout::{LayoutPolicy, MeasureScope, PlacementScope, RenderInput, RenderPolicy, layout},
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
    scrollable::scrollbar::{ScrollBarState, ScrollbarDefaults, scrollbar_h, scrollbar_v},
};

const SCROLL_INERTIA_DECAY_CONSTANT: f32 = 5.0;
const SCROLL_INERTIA_MIN_VELOCITY: f32 = 10.0;
const SCROLL_INERTIA_START_THRESHOLD: f32 = 50.0;
const SCROLL_INERTIA_MAX_VELOCITY: f32 = 6000.0;
const SCROLL_VELOCITY_SAMPLE_WINDOW: Duration = Duration::from_millis(90);
const SCROLL_VELOCITY_IDLE_CUTOFF: Duration = Duration::from_millis(65);

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
    /// The current position of the child component (for rendering)
    child_position: PxPosition,
    /// The target position of the child component (scrolling destination)
    target_position: PxPosition,
    /// Floating-point target retained between rendered pixel positions.
    target_position_f32: (f32, f32),
    /// Floating-point current position retained between rendered pixel
    /// positions.
    child_position_f32: (f32, f32),
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
            child_position: PxPosition::ZERO,
            target_position: PxPosition::ZERO,
            target_position_f32: (0.0, 0.0),
            child_position_f32: (0.0, 0.0),
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
    pub fn override_child_size(&mut self, size: ComputedData) {
        self.override_child_size = Some(size);
    }

    pub(crate) fn target_position(&self) -> PxPosition {
        self.target_position
    }

    pub(crate) fn set_target_position(&mut self, target: PxPosition) {
        self.cancel_inertia();
        self.velocity_tracker = None;
        self.positioned = true;
        self.target_position = target;
        self.target_position_f32 = (target.x.to_f32(), target.y.to_f32());
    }

    /// Instantly sets the scroll position without animation.
    ///
    /// This is useful for restoring a saved scroll position when remounting
    /// a component.
    pub fn set_scroll_position(&mut self, position: PxPosition) {
        self.cancel_inertia();
        self.velocity_tracker = None;
        self.last_frame_nanos = None;
        self.positioned = true;
        self.child_position = position;
        self.target_position = position;
        self.child_position_f32 = (position.x.to_f32(), position.y.to_f32());
        self.target_position_f32 = (position.x.to_f32(), position.y.to_f32());
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
        let (current_x, current_y) = self.child_position_f32;
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
        self.child_position_f32 = (
            current_x + (target_x - current_x) * factor,
            current_y + (target_y - current_y) * factor,
        );
        if (target_x - self.child_position_f32.0).abs() < 0.01 {
            self.child_position_f32.0 = target_x;
        }
        if (target_y - self.child_position_f32.1).abs() < 0.01 {
            self.child_position_f32.1 = target_y;
        }
        self.child_position = PxPosition {
            x: Px::saturating_from_f32(self.child_position_f32.0),
            y: Px::saturating_from_f32(self.child_position_f32.1),
        };
        if !self.has_pending_animation_frame() {
            self.last_frame_nanos = None;
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
        let immediate = source == ScrollEventSource::Touch || unit == ScrollDeltaUnit::Pixel;
        if immediate {
            self.target_position_f32 = self.child_position_f32;
            self.target_position = self.child_position;
        }
        let consumed = self.apply_scroll_delta(delta, container_size, vertical, horizontal);
        if immediate {
            self.child_position_f32 = self.target_position_f32;
            self.child_position = self.target_position;
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
        let constrained_target = constrain_position(
            PxPosition::new(Px::new(i32::MIN), Px::new(i32::MIN)),
            &self.child_size,
            container_size,
            vertical_scrollable,
            horizontal_scrollable,
        );
        let constrained = (constrained_target.x.to_f32(), constrained_target.y.to_f32());
        let next = (
            clamp_scroll_coordinate(proposed.0, constrained.0, horizontal_scrollable),
            clamp_scroll_coordinate(proposed.1, constrained.1, vertical_scrollable),
        );
        self.target_position_f32 = next;
        self.target_position = PxPosition::new(
            Px::saturating_from_f32(next.0),
            Px::saturating_from_f32(next.1),
        );
        ScrollDelta::new(next.0 - current_target.0, next.1 - current_target.1)
    }

    fn push_touch_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        self.cancel_inertia();
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
        let last_tick = self.inertia_last_tick.replace(now).unwrap_or(now);
        let displacement = inertia.advance(now.duration_since(last_tick));
        let proposed = (
            self.target_position_f32.0 + displacement.x,
            self.target_position_f32.1 + displacement.y,
        );
        let constrained = constrain_position(
            PxPosition::new(Px::new(i32::MIN), Px::new(i32::MIN)),
            &self.child_size,
            container_size,
            vertical_scrollable,
            horizontal_scrollable,
        );
        let next = (
            clamp_scroll_coordinate(proposed.0, constrained.x.to_f32(), horizontal_scrollable),
            clamp_scroll_coordinate(proposed.1, constrained.y.to_f32(), vertical_scrollable),
        );
        self.target_position_f32 = next;
        self.target_position = PxPosition::new(
            Px::saturating_from_f32(next.0),
            Px::saturating_from_f32(next.1),
        );
        self.child_position_f32 = next;
        self.child_position = self.target_position;
        if (next.0 - proposed.0).abs() > f32::EPSILON {
            inertia.velocity_x = 0.0;
        }
        if (next.1 - proposed.1).abs() > f32::EPSILON {
            inertia.velocity_y = 0.0;
        }
        if inertia.velocity().x.abs() >= SCROLL_INERTIA_MIN_VELOCITY
            || inertia.velocity().y.abs() >= SCROLL_INERTIA_MIN_VELOCITY
        {
            self.active_inertia = Some(inertia);
        } else {
            self.inertia_last_tick = None;
        }
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

#[derive(Clone, PartialEq)]
struct ScrollableAlongsideLayout {
    vertical: bool,
    horizontal: bool,
}

impl LayoutPolicy for ScrollableAlongsideLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
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
            let size = scrollbar.measure(&child_constraint)?;
            content_constraint.width -= size.width;
            final_size.width += size.width;
        }

        if self.horizontal {
            let scrollbar = if self.vertical {
                children[2]
            } else {
                children[1]
            };
            let size = scrollbar.measure(&child_constraint)?;
            content_constraint.height -= size.height;
            final_size.height += size.height;
        }

        let content = children[0];
        let content_measurement = content.measure(&content_constraint)?;
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

#[derive(Clone)]
struct ScrollableInnerLayout {
    controller: State<ScrollableController>,
    vertical: bool,
    horizontal: bool,
    has_override: bool,
    apply_child_offset: bool,
}

impl PartialEq for ScrollableInnerLayout {
    fn eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.has_override == other.has_override
            && self.apply_child_offset == other.apply_child_offset
    }
}

impl LayoutPolicy for ScrollableInnerLayout {
    fn measure(&self, input: &MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let mut child_constraint = *input.parent_constraint().as_ref();

        if self.vertical {
            child_constraint.height = AxisConstraint::NONE;
        }
        if self.horizontal {
            child_constraint.width = AxisConstraint::NONE;
        }

        let child = children[0];
        let child_measurement = child.measure(&child_constraint)?;
        let child_measurement = child_measurement.size();
        let next_child_size = self
            .controller
            .with(|c| c.override_child_size.unwrap_or(child_measurement));
        let needs_child_size_update = self.controller.with(|c| c.child_size != next_child_size);
        if needs_child_size_update {
            self.controller.with_mut(|c| c.child_size = next_child_size);
        }

        let current_child_position = if self.apply_child_offset {
            self.controller.with(|c| c.child_position())
        } else {
            PxPosition::ZERO
        };
        result.place_child(child, current_child_position);

        let width = input
            .parent_constraint()
            .width()
            .clamp(child_measurement.width);
        let height = input
            .parent_constraint()
            .height()
            .clamp(child_measurement.height);

        let computed_data = ComputedData { width, height };
        let needs_visible_size_update = self.controller.with(|c| c.visible_size != computed_data);
        if needs_visible_size_update {
            self.controller.with_mut(|c| c.visible_size = computed_data);
        }
        Ok(result.with_size(computed_data))
    }

    fn measure_eq(&self, other: &Self) -> bool {
        self.vertical == other.vertical
            && self.horizontal == other.horizontal
            && self.has_override == other.has_override
            && self.apply_child_offset == other.apply_child_offset
    }

    fn place_children(&self, input: &PlacementScope<'_>) -> Option<Vec<(u64, PxPosition)>> {
        let mut result = LayoutResult::default();
        let Some(&child) = input.children().first() else {
            return Some(result.into_placements());
        };
        let child_position = if self.apply_child_offset {
            self.controller.with(|c| c.child_position())
        } else {
            PxPosition::ZERO
        };
        result.place_child(child, child_position);
        Some(result.into_placements())
    }
}

impl RenderPolicy for ScrollableInnerLayout {
    fn record(&self, input: &mut RenderInput<'_>) {
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
/// - `apply_child_offset` — whether the viewport shifts its child by the
///   current scroll position or leaves placement to the child layout.
/// - `scrollbar_behavior` — scrollbar visibility behavior.
/// - `scrollbar_track_color` — optional scrollbar track color; transparent by
///   default.
/// - `scrollbar_thumb_color` — optional scrollbar thumb color; defaults to the
///   Material `on-surface` color at the resting scrollbar opacity.
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
/// use tessera_ui::{Dp, LayoutResult, Modifier, tessera};
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
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
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
        .offset(controller.with(|c| c.child_position().y))
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
        .offset(controller.with(|c| c.child_position().x))
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
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let controller = controller.expect("scrollable_with_alongside_scrollbar requires controller");
    let child = child.unwrap_or_else(RenderSlot::empty);
    let scrollbar_v_state = controller.with(|c| c.scrollbar_state_v());
    let scrollbar_h_state = controller.with(|c| c.scrollbar_state_h());

    layout()
        .layout_policy(ScrollableAlongsideLayout {
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
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
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
            Instant::now(),
            current_frame_nanos(),
        );
    }
}

impl ScrollableViewportPointerModifierNode {
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
            let tap_result = self.tap_recognizer.with_mut(|recognizer| {
                recognizer.update(pass, changes, cursor_position_rel, is_cursor_in_component)
            });
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
            } else if let Some(release_timestamp) = tap_result.release_timestamp {
                let available_velocity = self
                    .controller
                    .with_mut(|c| c.resolve_touch_velocity(release_timestamp));
                if !available_velocity.is_zero() {
                    let consumed_velocity = self
                        .nested_scroll_connection
                        .as_ref()
                        .map(|connection| connection.pre_fling(available_velocity))
                        .unwrap_or(ScrollVelocity::ZERO);
                    self.controller.with_mut(|c| {
                        c.start_inertia(release_timestamp, available_velocity - consumed_velocity)
                    });
                }
            }
        }
        if self.controller.with(|c| c.active_inertia.is_some()) {
            self.controller.with_mut(|c| {
                c.advance_inertia(now, &computed_data, self.vertical, self.horizontal);
            });
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
    let scroll_smoothing = scroll_smoothing.unwrap_or(0.12);
    let apply_child_offset = apply_child_offset.unwrap_or(true);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let controller = controller.expect("scrollable_viewport requires controller");
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
    let policy = ScrollableInnerLayout {
        controller,
        vertical,
        horizontal,
        has_override,
        apply_child_offset,
    };
    layout()
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
        assert_eq!(controller.child_position.y, Px(-100));
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
        while controller.child_position.y != Px(0) && steps < 200 {
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
                controller.update_scroll_position(frame * 16_666_667, 0.12);
            }
        }
        assert_eq!(
            controller.child_position.y,
            Px(0),
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
        assert_eq!(
            controller.child_position.y,
            Px(0),
            "current={:?}",
            controller.child_position
        );
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
        assert!(controller.is_positioned());
        controller.apply_input_delta(
            ScrollDelta::new(0.0, 500.0),
            ScrollEventSource::Wheel,
            ScrollDeltaUnit::Pixel,
            &viewport,
            true,
            false,
        );
        assert_eq!(controller.child_position(), PxPosition::ZERO);
        assert!(
            controller.is_positioned(),
            "a viewport at the top must not look like a fresh one"
        );
    }
}
