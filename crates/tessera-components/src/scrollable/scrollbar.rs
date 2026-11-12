//! Scrollbar components for scrollable viewports.
//!
//! ## Usage
//!
//! Show and drag scroll thumbs beside overflowing content.

use std::time::Duration;

use tessera_foundation::{
    gesture::{DragRecognizer, ScrollRecognizer, TapRecognizer},
    modifier::ModifierExt as _,
};
use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, AxisConstraint, Color, ComputedData, Constraint,
    Dp, LayoutResult, MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode, State,
    accesskit::{Action, Role},
    launch_once,
    layout::{MeasureScope, PlacementResult, PlacementScope},
    modifier::ModifierCapabilityExt as _,
    remember, tessera,
    time::Instant,
    use_context, with_frame_time,
};

use crate::{
    animation::FrameDelta,
    modifier::with_pointer_input,
    nested_scroll::NestedScrollConnection,
    scrollable::{ScrollBarBehavior, ScrollableController, ScrollbarWheelModifierNode},
    shape_def::Shape,
    surface::surface,
    theme::{MaterialColorScheme, MaterialTheme},
};

/// Material Design 3 defaults for the scrollbar components.
///
/// A Material 3 scrollbar is a rounded thumb over the content; it has no
/// persistent track. The thumb uses the `on-surface` hue at a low opacity so
/// it stays legible on both light and dark surfaces.
///
/// The opacities are tuned for the renderer's linear-space compositing so the
/// resting thumb keeps a comparable weight to a Material scrollbar on a light
/// surface.
pub struct ScrollbarDefaults;

impl ScrollbarDefaults {
    /// Default scrollbar thickness.
    pub const THICKNESS: Dp = Dp(8.0);

    /// Default thumb opacity over the surface.
    pub const THUMB_ALPHA: f32 = 0.5;

    /// Thumb opacity while the pointer hovers or drags the thumb.
    pub const THUMB_HOVER_ALPHA: f32 = 0.7;

    /// Returns the default thumb color for the provided scheme.
    pub fn thumb_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface.with_alpha(Self::THUMB_ALPHA)
    }

    /// Returns the thumb color used while hovering or dragging.
    pub fn thumb_hover_color(scheme: &MaterialColorScheme) -> Color {
        scheme.on_surface.with_alpha(Self::THUMB_HOVER_ALPHA)
    }
}

#[derive(Clone, PartialEq, Copy, Default)]
enum ScrollOrientation {
    #[default]
    Vertical,
    Horizontal,
}

impl ScrollOrientation {
    fn position(self, main: Px, cross: Px) -> PxPosition {
        match self {
            Self::Vertical => PxPosition::new(cross, main),
            Self::Horizontal => PxPosition::new(main, cross),
        }
    }

    fn extent(self, size: ComputedData) -> Px {
        match self {
            Self::Vertical => size.height,
            Self::Horizontal => size.width,
        }
    }

    fn size(self, main: Px, cross: Px) -> tessera_ui::PxSize {
        match self {
            Self::Vertical => tessera_ui::PxSize::new(cross, main),
            Self::Horizontal => tessera_ui::PxSize::new(main, cross),
        }
    }

    fn axis_px(self, point: PxPosition) -> Px {
        match self {
            Self::Vertical => point.y_px(),
            Self::Horizontal => point.x_px(),
        }
    }

    fn cursor(self, point: PxPosition) -> (f32, f32) {
        match self {
            Self::Vertical => (point.y, point.x),
            Self::Horizontal => (point.x, point.y),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct ZeroLayout;

impl tessera_ui::LayoutPolicy for ZeroLayout {
    fn measure(&self, _input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        Ok(LayoutResult::new(ComputedData::ZERO))
    }
}

const HOVER_FADE_DURATION_SECS: f32 = 0.2;
const AUTO_HIDE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, PartialEq)]
struct ScrollBarLayout {
    thumb_offset: PxPosition,
}

impl tessera_ui::LayoutPolicy for ScrollBarLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let track = children[0];
        let size = track.measure(input, &Constraint::NONE)?;
        result.place_child(track, PxPosition::ZERO);

        let thumb = children[1];
        thumb.measure(input, &Constraint::NONE)?;
        result.place_child(thumb, self.thumb_offset);

        Ok(result.with_size(size.size()))
    }

    fn measure_eq(&self, _other: &Self) -> bool {
        true
    }

    fn placement_eq(&self, other: &Self) -> bool {
        self.thumb_offset == other.thumb_offset
    }

    fn place(
        &self,
        input: &PlacementScope<'_>,
        _inner: &PlacementResult,
    ) -> Option<PlacementResult> {
        let mut result = LayoutResult::default();
        let children = input.children();
        let Some(&track) = children.first() else {
            return Some(result.into_placement_result());
        };
        result.place_child(track, PxPosition::ZERO);
        let Some(&thumb) = children.get(1) else {
            return Some(result.into_placement_result());
        };
        result.place_child(thumb, self.thumb_offset);
        Some(result.into_placement_result())
    }
}

#[derive(Clone, PartialEq)]
struct ScrollBarConfig {
    total: Px,
    visible: Px,
    offset: Px,
    thickness: Dp,
    state: State<ScrollableController>,
    scrollbar_behavior: ScrollBarBehavior,
    track_color: Color,
    thumb_color: Color,
    thumb_hover_color: Color,
    scrollbar_controller: Option<ScrollBarController>,
}

#[derive(Clone, Copy, PartialEq, Default)]
/// Mutable state shared by a scrollbar and its viewport.
pub struct ScrollBarInteractionState {
    /// Whether the scrollbar's thumb is currently being dragged.
    pub is_dragging: bool,
    /// Whether the active pointer press started on the thumb.
    pub pressed_on_thumb: bool,
    /// Whether the scrollbar's thumb is currently being hovered.
    pub is_hovered: bool,
    /// Blend from the resting thumb color (0.0) to the hover color (1.0).
    pub hover_progress: f32,
    /// Counts scroll activity; each change restarts the AutoHide timeout.
    pub scroll_activity: u64,
    /// Whether the scrollbar should be visible (for AutoHide behavior).
    pub should_be_visible: bool,
}

/// An explicit owner of the interaction state shared by a scrollbar and viewport.
#[derive(Clone, Copy, PartialEq)]
pub struct ScrollBarController {
    owner: ScrollBarOwner,
}

#[derive(Clone, Copy, PartialEq)]
enum ScrollBarOwner {
    External(State<ScrollBarInteractionState>),
    Viewport {
        controller: State<ScrollableController>,
        vertical: bool,
    },
}

impl ScrollBarController {
    /// Shares a component-owned scrollbar interaction state.
    pub fn new(state: State<ScrollBarInteractionState>) -> Self {
        Self {
            owner: ScrollBarOwner::External(state),
        }
    }

    pub(crate) fn viewport(controller: State<ScrollableController>, vertical: bool) -> Self {
        Self {
            owner: ScrollBarOwner::Viewport {
                controller,
                vertical,
            },
        }
    }

    /// Reads the current interaction state.
    pub fn read(&self) -> ScrollBarInteractionState {
        match self.owner {
            ScrollBarOwner::External(state) => state.get(),
            ScrollBarOwner::Viewport {
                controller,
                vertical,
            } => controller.with(|controller| {
                if vertical {
                    controller.scrollbar_state_v
                } else {
                    controller.scrollbar_state_h
                }
            }),
        }
    }

    /// Updates the owning state and invalidates its readers.
    pub fn with_mut<R>(&self, update: impl FnOnce(&mut ScrollBarInteractionState) -> R) -> R {
        match self.owner {
            ScrollBarOwner::External(state) => state.with_mut(update),
            ScrollBarOwner::Viewport {
                controller,
                vertical,
            } => controller.with_mut(|controller| {
                update(if vertical {
                    &mut controller.scrollbar_state_v
                } else {
                    &mut controller.scrollbar_state_h
                })
            }),
        }
    }
}

fn calculate_target_pos(
    orientation: ScrollOrientation,
    cursor: Px,
    track_extent: Px,
    thumb_extent: Px,
    total: Px,
    visible: Px,
    fallback: PxPosition,
) -> PxPosition {
    // If the thumb cannot move, return the provided fallback (avoids locking
    // inside this helper).
    let thumb_scrollable_range = track_extent - thumb_extent;
    if thumb_scrollable_range <= Px::ZERO {
        return fallback;
    }

    let cursor_adjusted = cursor - thumb_extent / 2;
    let progress = (cursor_adjusted.to_f32() / thumb_scrollable_range.to_f32()).clamp(0.0, 1.0);

    let content_scrollable_range = total - visible;
    if content_scrollable_range <= Px::ZERO {
        return PxPosition::ZERO;
    }

    let target = Px::from_f32(-progress * content_scrollable_range.to_f32());
    orientation.position(target, Px::ZERO)
}

fn compute_thumb_color(
    state: &ScrollBarInteractionState,
    thumb_color: Color,
    thumb_hover_color: Color,
) -> Color {
    thumb_color.lerp(&thumb_hover_color, state.hover_progress.clamp(0.0, 1.0))
}

fn hover_target(state: &ScrollBarInteractionState) -> f32 {
    if state.is_hovered { 1.0 } else { 0.0 }
}
fn should_show_scrollbar(args: &ScrollBarConfig, state: &ScrollBarController) -> bool {
    match args.scrollbar_behavior {
        ScrollBarBehavior::AlwaysVisible => true,
        ScrollBarBehavior::Hidden => false,
        ScrollBarBehavior::AutoHide => {
            let state_guard = state.read();
            state_guard.should_be_visible || state_guard.is_dragging || state_guard.is_hovered
        }
    }
}

fn needs_scrollbar_frame_tick(state: &ScrollBarInteractionState, auto_hide: bool) -> bool {
    state.hover_progress != hover_target(state) || (auto_hide && state.should_be_visible)
}

/// Frame-driven hover fades and AutoHide timeouts of one scrollbar.
#[derive(Default)]
struct ScrollBarFrames {
    delta: FrameDelta,
    activity: Option<(u64, Instant)>,
}

impl ScrollBarFrames {
    fn advance(
        &mut self,
        state: &mut ScrollBarInteractionState,
        auto_hide: bool,
        now: Instant,
    ) -> bool {
        let dt = self.delta.step(now);
        let target = hover_target(state);
        let step = dt / HOVER_FADE_DURATION_SECS;
        state.hover_progress = if state.hover_progress < target {
            (state.hover_progress + step).min(target)
        } else {
            (state.hover_progress - step).max(target)
        };
        if auto_hide && state.should_be_visible {
            let since = match self.activity {
                Some((activity, since)) if activity == state.scroll_activity => since,
                _ => {
                    self.activity = Some((state.scroll_activity, now));
                    now
                }
            };
            if now.saturating_duration_since(since) > AUTO_HIDE_TIMEOUT {
                state.should_be_visible = false;
            }
        }
        needs_scrollbar_frame_tick(state, auto_hide)
    }
}

async fn run_scrollbar_frames(state: ScrollBarController, auto_hide: bool) {
    let mut frames = ScrollBarFrames::default();
    loop {
        let running = with_frame_time(|now| {
            let mut next = state.read();
            let running = frames.advance(&mut next, auto_hide, now);
            if next != state.read() {
                state.with_mut(|state| *state = next);
            }
            running
        })
        .await;
        if !running {
            break;
        }
    }
}

fn mark_scroll_activity(state: &ScrollBarController, behavior: &ScrollBarBehavior) {
    if matches!(*behavior, ScrollBarBehavior::AutoHide) {
        state.with_mut(|state_guard| {
            state_guard.scroll_activity = state_guard.scroll_activity.wrapping_add(1);
            state_guard.should_be_visible = true;
        });
    }
}

fn compute_thumb_progress(offset: Px, total: Px) -> f32 {
    if total <= Px::ZERO {
        0.0
    } else {
        offset.to_f32().abs() / total.to_f32()
    }
}

fn compute_thumb_size(visible: Px, total: Px) -> Px {
    if total <= Px::ZERO {
        return visible.max(Px::ZERO);
    }
    let visible_len = visible.to_f32().abs();
    let total_len = total.to_f32().abs().max(1.0);
    let thumb = (visible_len * visible_len) / total_len;

    // Clamp the thumb size to ensure it's always visible and provides a
    // reasonable drag target.
    let min_thumb = (visible_len * 0.05).clamp(8.0, 32.0);
    Px::saturating_from_f32(thumb.max(min_thumb))
}

fn cursor_on_thumb(
    orientation: ScrollOrientation,
    cursor: PxPosition,
    thickness: Px,
    offset: f32,
    extent: Px,
) -> bool {
    let (main, cross) = orientation.cursor(cursor);
    cross >= 0.0
        && cross <= thickness.to_f32()
        && main >= offset
        && main <= offset + extent.to_f32()
}

fn apply_scrollbar_semantics(
    accessibility: &mut AccessibilityNode,
    action_handler: &mut Option<AccessibilityActionHandler>,
    args: &ScrollBarConfig,
    state: &ScrollBarController,
    orientation: ScrollOrientation,
) {
    accessibility.role = Some(Role::ScrollBar);
    accessibility.label = Some(
        match orientation {
            ScrollOrientation::Vertical => "Vertical scrollbar",
            ScrollOrientation::Horizontal => "Horizontal scrollbar",
        }
        .to_string(),
    );

    let progress = compute_thumb_progress(args.offset, args.total).clamp(0.0, 1.0);
    accessibility.numeric_value = Some(progress as f64);
    accessibility.min_numeric_value = Some(0.0);
    accessibility.max_numeric_value = Some(1.0);
    accessibility.focusable = true;
    accessibility.actions.clear();
    accessibility.actions.push(Action::Increment);
    accessibility.actions.push(Action::Decrement);

    let args_clone = args.clone();
    let state_clone = *state;
    *action_handler = Some(Box::new(move |action| match action {
        Action::Increment => {
            scroll_accessibility_step(&args_clone, &state_clone, orientation, true)
        }
        Action::Decrement => {
            scroll_accessibility_step(&args_clone, &state_clone, orientation, false)
        }
        _ => {}
    }));
}

struct ScrollbarSemanticsModifierNode {
    args: ScrollBarConfig,
    state: ScrollBarController,
    orientation: ScrollOrientation,
}

impl SemanticsModifierNode for ScrollbarSemanticsModifierNode {
    fn apply(
        &self,
        accessibility: &mut AccessibilityNode,
        action_handler: &mut Option<AccessibilityActionHandler>,
    ) {
        apply_scrollbar_semantics(
            accessibility,
            action_handler,
            &self.args,
            &self.state,
            self.orientation,
        );
    }
}

fn scroll_accessibility_step(
    args: &ScrollBarConfig,
    state: &ScrollBarController,
    orientation: ScrollOrientation,
    increment: bool,
) {
    let step_amount = (args.visible.to_f32() * 0.1).max(1.0);
    let step_px = Px::from_f32(step_amount);
    let delta = match orientation {
        ScrollOrientation::Vertical => {
            let dy = if increment { -step_px } else { step_px };
            PxPosition::new(Px::ZERO, dy)
        }
        ScrollOrientation::Horizontal => {
            let dx = if increment { -step_px } else { step_px };
            PxPosition::new(dx, Px::ZERO)
        }
    };

    let new_target = args.state.with(|c| {
        c.target_position()
            .saturating_offset(delta.x_px(), delta.y_px())
    });
    args.state.with_mut(|c| c.set_target_position(new_target));

    mark_scroll_activity(state, &args.scrollbar_behavior);
}

fn update_drag(
    orientation: ScrollOrientation,
    input: &tessera_ui::PointerInput,
    calculate_target: &dyn Fn(Px) -> PxPosition,
    args: &ScrollBarConfig,
    state: &ScrollBarController,
) {
    if let Some(cursor_pos) = input.cursor_position_rel {
        let new_target_pos = calculate_target(orientation.axis_px(cursor_pos));
        // Dragging the thumb is direct manipulation: the content has to follow
        // the pointer without the eased follow used for wheel notches.
        args.state
            .with_mut(|c| c.set_scroll_position(new_target_pos));
        mark_scroll_activity(state, &args.scrollbar_behavior);
    } else {
        // Cursor left window: stop dragging.
        state.with_mut(|state| state.is_dragging = false);
    }
}

fn update_hover_state(is_on_thumb: bool, state: &ScrollBarController) {
    if is_on_thumb != state.read().is_hovered {
        state.with_mut(|state_guard| state_guard.is_hovered = is_on_thumb);
    }
}

#[derive(Clone, Copy)]
struct ScrollBarTrackMetrics {
    track_extent: Px,
    thumb_extent: Px,
}

fn handle_state(
    orientation: ScrollOrientation,
    args: &ScrollBarConfig,
    state: &ScrollBarController,
    metrics: ScrollBarTrackMetrics,
    tap_recognizer: State<TapRecognizer>,
    drag_recognizer: State<DragRecognizer>,
    input: &mut tessera_ui::PointerInput<'_>,
) {
    // Capture current target position once to avoid locking inside helper on
    // every call.
    let fallback_pos = args.state.with(|c| c.target_position());
    let calculate_target_pos = |cursor: Px| -> PxPosition {
        calculate_target_pos(
            orientation,
            cursor,
            metrics.track_extent,
            metrics.thumb_extent,
            args.total,
            args.visible,
            fallback_pos,
        )
    };
    let thickness = args.thickness.to_px();
    let cursor_pos = input.cursor_position_rel;
    let within_track = cursor_pos
        .map(|pos| cursor_on_thumb(orientation, pos, thickness, 0.0, metrics.track_extent))
        .unwrap_or(false);
    let thumb_offset = args.visible.to_f32() * (args.offset.to_f32().abs() / args.total.to_f32());
    let is_on_thumb = cursor_pos
        .map(|pos| {
            cursor_on_thumb(
                orientation,
                pos,
                thickness,
                thumb_offset,
                metrics.thumb_extent,
            )
        })
        .unwrap_or(false);
    update_hover_state(is_on_thumb, state);
    let tap_result = tap_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            within_track,
        )
    });
    let drag_result = drag_recognizer.with_mut(|recognizer| {
        recognizer.update(
            input.pass,
            input.pointer_changes.as_mut_slice(),
            within_track,
        )
    });

    if tap_result.pressed {
        if is_on_thumb {
            state.with_mut(|state| state.pressed_on_thumb = true);
        } else if within_track && let Some(cursor_pos) = cursor_pos {
            let new_target_pos = calculate_target_pos(orientation.axis_px(cursor_pos));
            args.state
                .with_mut(|c| c.set_target_position(new_target_pos));
            mark_scroll_activity(state, &args.scrollbar_behavior);
        }
    }

    if drag_result.started && state.read().pressed_on_thumb {
        state.with_mut(|state| state.is_dragging = true);
    }

    if state.read().is_dragging && drag_result.updated {
        update_drag(orientation, input, &calculate_target_pos, args, state);
    } else if state.read().is_dragging && cursor_pos.is_none() {
        state.with_mut(|state_guard| {
            state_guard.is_dragging = false;
            state_guard.pressed_on_thumb = false;
        });
    }

    if tap_result.released || drag_result.ended {
        state.with_mut(|state_guard| {
            state_guard.is_dragging = false;
            state_guard.pressed_on_thumb = false;
        });
    }
}

/// # scrollbar_v
///
/// Draw a vertical scrollbar for a scroll viewport.
///
/// ## Usage
///
/// Show and drag the vertical offset of overflowing content.
///
/// ## Parameters
///
/// - `total` — content extent in physical pixels; defaults to controller
///   content.
/// - `visible` — viewport extent in physical pixels; defaults to controller
///   viewport.
/// - `offset` — current scroll offset; defaults to the controller position.
/// - `thickness` — thickness of the scrollbar track.
/// - `controller` — scroll controller the scrollbar reads and drives.
/// - `scrollbar_behavior` — visibility behavior (always, auto-hide, hidden).
/// - `track_color`, `thumb_color`, `thumb_hover_color` — themed colors.
/// - `scrollbar_controller` — shared interaction state; created internally when
///   absent.
///
/// Wheel input follows the attached viewport. A custom track without a viewport
/// retains tap, drag and accessibility actions and leaves wheel input to
/// ancestors.
///
/// ## Examples
///
/// ```
/// use tessera_components::scrollable::{ScrollableController, scrollbar_v};
/// use tessera_components::theme::material_theme;
/// use tessera_ui::{Px, PxPosition, remember, tessera};
/// #[tessera]
/// fn tracked_content() {
///     material_theme().child(|| {
///         let controller = remember(ScrollableController::new);
///         let offset = PxPosition::new(Px::ZERO, Px(-40));
///         controller.with_mut(|controller| controller.set_scroll_position(offset));
///         assert_eq!(
///             controller.with(|controller| controller.child_position()),
///             offset
///         );
///         scrollbar_v()
///             .controller(controller)
///             .total(Px(600))
///             .visible(Px(200));
///     });
/// }
/// # tessera_ui::testing::UiTest::new().content(tracked_content).frame();
/// ```
#[tessera]
pub fn scrollbar_v(
    total: Option<Px>,
    visible: Option<Px>,
    offset: Option<Px>,
    thickness: Option<Dp>,
    controller: Option<State<ScrollableController>>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    track_color: Option<Color>,
    thumb_color: Option<Color>,
    thumb_hover_color: Option<Color>,
    scrollbar_controller: Option<ScrollBarController>,
) {
    scrollbar(
        ScrollOrientation::Vertical,
        controller.expect("scrollbar_v requires controller"),
    )
    .total_optional(total)
    .visible_optional(visible)
    .offset_optional(offset)
    .thickness_optional(thickness)
    .scrollbar_behavior_optional(scrollbar_behavior)
    .track_color_optional(track_color)
    .thumb_color_optional(thumb_color)
    .thumb_hover_color_optional(thumb_hover_color)
    .scrollbar_controller_optional(scrollbar_controller);
}

/// # scrollbar_h
///
/// Draw a horizontal scrollbar for a scroll viewport.
///
/// ## Usage
///
/// Show and drag the horizontal offset of overflowing content.
///
/// ## Parameters
///
/// - `total` — content extent in physical pixels; defaults to controller
///   content.
/// - `visible` — viewport extent in physical pixels; defaults to controller
///   viewport.
/// - `offset` — current scroll offset; defaults to the controller position.
/// - `thickness` — thickness of the scrollbar track.
/// - `controller` — scroll controller the scrollbar reads and drives.
/// - `scrollbar_behavior` — visibility behavior (always, auto-hide, hidden).
/// - `track_color`, `thumb_color`, `thumb_hover_color` — themed colors.
/// - `scrollbar_controller` — shared interaction state; created internally when
///   absent.
///
/// Wheel input follows the attached viewport. A custom track without a viewport
/// retains tap, drag and accessibility actions and leaves wheel input to
/// ancestors.
///
/// ## Examples
///
/// ```
/// use tessera_components::scrollable::{ScrollableController, scrollbar_h};
/// use tessera_components::theme::material_theme;
/// use tessera_ui::{Px, PxPosition, remember, tessera};
/// #[tessera]
/// fn tracked_content() {
///     material_theme().child(|| {
///         let controller = remember(ScrollableController::new);
///         let offset = PxPosition::new(Px(-40), Px::ZERO);
///         controller.with_mut(|controller| controller.set_scroll_position(offset));
///         assert_eq!(
///             controller.with(|controller| controller.child_position()),
///             offset
///         );
///         scrollbar_h()
///             .controller(controller)
///             .total(Px(600))
///             .visible(Px(200));
///     });
/// }
/// # tessera_ui::testing::UiTest::new().content(tracked_content).frame();
/// ```
#[tessera]
pub fn scrollbar_h(
    total: Option<Px>,
    visible: Option<Px>,
    offset: Option<Px>,
    thickness: Option<Dp>,
    controller: Option<State<ScrollableController>>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    track_color: Option<Color>,
    thumb_color: Option<Color>,
    thumb_hover_color: Option<Color>,
    scrollbar_controller: Option<ScrollBarController>,
) {
    scrollbar(
        ScrollOrientation::Horizontal,
        controller.expect("scrollbar_h requires controller"),
    )
    .total_optional(total)
    .visible_optional(visible)
    .offset_optional(offset)
    .thickness_optional(thickness)
    .scrollbar_behavior_optional(scrollbar_behavior)
    .track_color_optional(track_color)
    .thumb_color_optional(thumb_color)
    .thumb_hover_color_optional(thumb_hover_color)
    .scrollbar_controller_optional(scrollbar_controller);
}

#[tessera]
fn scrollbar(
    orientation: ScrollOrientation,
    state: State<ScrollableController>,
    total: Option<Px>,
    visible: Option<Px>,
    offset: Option<Px>,
    thickness: Option<Dp>,
    scrollbar_behavior: Option<ScrollBarBehavior>,
    track_color: Option<Color>,
    thumb_color: Option<Color>,
    thumb_hover_color: Option<Color>,
    scrollbar_controller: Option<ScrollBarController>,
) {
    let total = total
        .unwrap_or_else(|| state.with(|controller| orientation.extent(controller.child_size())));
    let visible = visible
        .unwrap_or_else(|| state.with(|controller| orientation.extent(controller.visible_size())));
    let offset = offset.unwrap_or_else(|| {
        state.with(|controller| orientation.axis_px(controller.child_position()))
    });
    let thickness = thickness.unwrap_or(ScrollbarDefaults::THICKNESS);
    let scrollbar_behavior = scrollbar_behavior.unwrap_or_default();
    let scheme = use_context::<MaterialTheme>()
        .map(|theme| theme.get().color_scheme)
        .unwrap_or_default();
    let track_color = track_color.unwrap_or(Color::TRANSPARENT);
    let thumb_color = thumb_color.unwrap_or_else(|| ScrollbarDefaults::thumb_color(&scheme));
    let thumb_hover_color =
        thumb_hover_color.unwrap_or_else(|| ScrollbarDefaults::thumb_hover_color(&scheme));
    let args = ScrollBarConfig {
        total,
        visible,
        offset,
        thickness,
        state,
        scrollbar_behavior,
        track_color,
        thumb_color,
        thumb_hover_color,
        scrollbar_controller,
    };
    let state = if let Some(state) = args.scrollbar_controller {
        state
    } else {
        ScrollBarController::new(remember(ScrollBarInteractionState::default))
    };

    let tap_recognizer = remember(TapRecognizer::default);
    let drag_recognizer = remember(DragRecognizer::default);
    let scroll_recognizer = remember(ScrollRecognizer::default);
    let nested_scroll_connection = NestedScrollConnection::nearest();
    let auto_hide = matches!(args.scrollbar_behavior, ScrollBarBehavior::AutoHide);
    if needs_scrollbar_frame_tick(&state.read(), auto_hide) {
        launch_once(run_scrollbar_frames(state, auto_hide));
    }

    let should_show = should_show_scrollbar(&args, &state);

    if !should_show {
        tessera_ui::layout::layout(ZeroLayout);
        return;
    }

    if args.visible <= Px::ZERO || args.total <= Px::ZERO || args.thickness <= Dp::ZERO {
        tessera_ui::layout::layout(ZeroLayout);
        return;
    }

    let thickness = args.thickness.to_px();
    let track_extent = args.visible;
    let thumb_extent = compute_thumb_size(args.visible, args.total);
    let has_overflow = args.total > args.visible;
    let track_color = if has_overflow {
        args.track_color
    } else {
        args.track_color.with_alpha(0.0)
    };

    let handler_args = args.clone();
    let handler_state = state;
    let modifier = with_pointer_input(Modifier::new(), move |mut input| {
        handle_state(
            orientation,
            &handler_args,
            &handler_state,
            ScrollBarTrackMetrics {
                track_extent: track_extent,
                thumb_extent: thumb_extent,
            },
            tap_recognizer,
            drag_recognizer,
            &mut input,
        );
    })
    .push_pointer_input(ScrollbarWheelModifierNode {
        controller: args.state,
        recognizer: scroll_recognizer,
        connection: nested_scroll_connection,
        scrollbar_controller: state,
        behavior: args.scrollbar_behavior.clone(),
    })
    .push_semantics(ScrollbarSemanticsModifierNode {
        args: args.clone(),
        state: state,
        orientation,
    });

    let thumb_color = if has_overflow {
        compute_thumb_color(&state.read(), args.thumb_color, args.thumb_hover_color)
    } else {
        args.thumb_color.with_alpha(0.0)
    };
    let progress = compute_thumb_progress(args.offset, args.total);
    let thumb_offset = args.visible.to_f32() * progress;

    tessera_ui::layout::layout(ScrollBarLayout {
        thumb_offset: orientation.position(Px::from_f32(thumb_offset), Px::ZERO),
    })
    .modifier(modifier)
    .child(move || {
        for (extent, color) in [(track_extent, track_color), (thumb_extent, thumb_color)] {
            let size = orientation.size(extent, thickness);
            surface()
                .modifier(Modifier::new().constrain(
                    Some(AxisConstraint::exact(size.width)),
                    Some(AxisConstraint::exact(size.height)),
                ))
                .style(color.into())
                .shape(Shape::CAPSULE)
                .child(|| {});
        }
    });
}

#[cfg(test)]
mod tests {
    use tessera_ui::Color;

    use crate::animation::test_instant;

    use super::{
        AUTO_HIDE_TIMEOUT, HOVER_FADE_DURATION_SECS, ScrollBarFrames, ScrollBarInteractionState,
        compute_thumb_color,
    };

    fn nanos(seconds: f32) -> u64 {
        (seconds * 1_000_000_000.0) as u64
    }

    #[test]
    fn thumb_color_fades_towards_the_hover_color_over_frames() {
        let thumb = Color::new(1.0, 0.0, 0.0, 0.4);
        let hovered = Color::new(1.0, 0.0, 0.0, 0.6);
        let mut state = ScrollBarInteractionState::default();
        let mut frames = ScrollBarFrames::default();

        assert_eq!(compute_thumb_color(&state, thumb, hovered), thumb);

        state.is_hovered = true;
        assert!(frames.advance(&mut state, false, test_instant(0)));
        assert_eq!(compute_thumb_color(&state, thumb, hovered), thumb);

        let half = frames.advance(
            &mut state,
            false,
            test_instant(nanos(HOVER_FADE_DURATION_SECS / 2.0)),
        );
        assert!(half);
        let midway = compute_thumb_color(&state, thumb, hovered);
        assert!(midway.a > thumb.a + 1e-3 && midway.a < hovered.a - 1e-3);

        let running = frames.advance(
            &mut state,
            false,
            test_instant(nanos(HOVER_FADE_DURATION_SECS)),
        );
        assert!(!running);
        let faded = compute_thumb_color(&state, thumb, hovered);
        assert!((faded.a - hovered.a).abs() < 1e-5);
    }

    #[test]
    fn auto_hide_timeout_restarts_with_scroll_activity() {
        let mut state = ScrollBarInteractionState {
            should_be_visible: true,
            ..Default::default()
        };
        let mut frames = ScrollBarFrames::default();
        let timeout = AUTO_HIDE_TIMEOUT.as_nanos() as u64;

        assert!(frames.advance(&mut state, true, test_instant(0)));
        state.scroll_activity += 1;
        assert!(frames.advance(&mut state, true, test_instant(timeout / 2)));
        assert!(frames.advance(&mut state, true, test_instant(timeout + 1)));
        assert!(state.should_be_visible);
        assert!(!frames.advance(&mut state, true, test_instant(timeout / 2 + timeout + 1)));
        assert!(!state.should_be_visible);
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "scrollbar_bindings_tests.rs"]
mod bindings_tests;
