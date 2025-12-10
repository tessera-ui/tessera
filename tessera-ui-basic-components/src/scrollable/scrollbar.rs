use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    Color, Constraint, CursorEventContent, Dp, PressKeyEventType, Px, PxPosition,
    accesskit::{Action, Role},
    tessera,
};

use crate::{
    scrollable::{ScrollBarBehavior, ScrollableController},
    shape_def::{RoundedCorner, Shape},
    surface::{SurfaceArgsBuilder, surface},
};

#[derive(Clone, Copy)]
enum ScrollOrientation {
    Vertical,
    Horizontal,
}

#[derive(Clone)]
pub struct ScrollBarArgs {
    /// The total size of the scrollable content.
    pub total: Px,
    /// The size of the visible area.
    pub visible: Px,
    /// The current scroll offset.
    pub offset: Px,
    /// The thickness of the scrollbar
    pub thickness: Dp,
    /// The scrollable's state, used for interaction.
    pub state: Arc<ScrollableController>,
    /// The behavior of the scrollbar visibility.
    pub scrollbar_behavior: ScrollBarBehavior,
    /// The color of the scrollbar track.
    pub track_color: Color,
    /// The color of the scrollbar thumb.
    pub thumb_color: Color,
    /// The color of the scrollbar thumb when hovered.
    pub thumb_hover_color: Color,
}

#[derive(Default)]
pub struct ScrollBarStateInner {
    /// Whether the scrollbar's thumb is currently being dragged.
    pub is_dragging: bool,
    /// Whether the scrollbar's thumb is currently being hovered.
    pub is_hovered: bool,
    /// The instant when the hover state last changed.
    pub hover_instant: Option<std::time::Instant>,
    /// The instant when the last scroll activity occurred (for AutoHide
    /// behavior).
    pub last_scroll_activity: Option<std::time::Instant>,
    /// Whether the scrollbar should be visible (for AutoHide behavior).
    pub should_be_visible: bool,
}

/// Public wrapper for ScrollBarStateInner that stores the internal
/// Arc<RwLock<..>>.
#[derive(Clone)]
pub struct ScrollBarState {
    inner: Arc<RwLock<ScrollBarStateInner>>,
}

impl ScrollBarState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ScrollBarStateInner::default())),
        }
    }

    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, ScrollBarStateInner> {
        self.inner.read()
    }

    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, ScrollBarStateInner> {
        self.inner.write()
    }
}

impl Default for ScrollBarState {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate the target content position for a vertical scrollbar given a
/// cursor Y.
///
/// This extracts the logic previously embedded in the `input_handler` closure
/// so the closure becomes smaller and easier to reason about during static
/// analysis.
/// - `cursor_y`: cursor Y within the scrollbar track (in Px).
/// - `track_height`: visible track height (in Px).
/// - `thumb_height`: thumb size (in Px).
/// - `args`: scrollbar args (contains total / visible / state).
fn calculate_target_pos_v(
    cursor_y: Px,
    track_height: Px,
    thumb_height: Px,
    total: Px,
    visible: Px,
    fallback: PxPosition,
) -> PxPosition {
    // If the thumb cannot move, return the provided fallback (avoids locking inside
    // this helper).
    let thumb_scrollable_range = track_height - thumb_height;
    if thumb_scrollable_range <= Px::ZERO {
        return fallback;
    }

    let cursor_y_adjusted = cursor_y - thumb_height / 2;
    let progress = (cursor_y_adjusted.to_f32() / thumb_scrollable_range.to_f32()).clamp(0.0, 1.0);

    let content_scrollable_range = total - visible;
    if content_scrollable_range <= Px::ZERO {
        return PxPosition::ZERO;
    }

    let new_target_y = Px::from_f32(-progress * content_scrollable_range.to_f32());
    PxPosition {
        x: Px::ZERO, // Vertical scrollbar doesn't affect X
        y: new_target_y,
    }
}

/// Calculate the target content position for a horizontal scrollbar given a
/// cursor X. Mirrors `calculate_target_pos_v` for horizontal axis.
fn calculate_target_pos_h(
    cursor_x: Px,
    track_width: Px,
    thumb_width: Px,
    total: Px,
    visible: Px,
    fallback: PxPosition,
) -> PxPosition {
    // If the thumb cannot move, return the provided fallback (avoids locking inside
    // this helper).
    let thumb_scrollable_range = track_width - thumb_width;
    if thumb_scrollable_range <= Px::ZERO {
        return fallback;
    }

    let cursor_x_adjusted = cursor_x - thumb_width / 2;
    let progress = (cursor_x_adjusted.to_f32() / thumb_scrollable_range.to_f32()).clamp(0.0, 1.0);

    let content_scrollable_range = total - visible;
    if content_scrollable_range <= Px::ZERO {
        return PxPosition::ZERO;
    }

    let new_target_x = Px::from_f32(-progress * content_scrollable_range.to_f32());
    PxPosition {
        x: new_target_x,
        y: Px::ZERO, // Horizontal scrollbar doesn't affect Y
    }
}

/// Compute the thumb color with hover interpolation.
/// Extracted to reduce duplication between vertical and horizontal scrollbar
/// implementations.
fn compute_thumb_color(state_lock: &ScrollBarState, args: &ScrollBarArgs) -> Color {
    let state = state_lock.read();
    let (from_color, to_color) = if state.is_hovered {
        (args.thumb_color, args.thumb_hover_color)
    } else {
        (args.thumb_hover_color, args.thumb_color)
    };
    let progress = if let Some(instant) = state.hover_instant {
        (instant.elapsed().as_secs_f32() / 0.2).min(1.0)
    } else {
        0.0
    };
    from_color.lerp(&to_color, progress)
}
/// Render a rounded surface for a vertical track (radius based on width).
fn render_track_surface_v(width: Px, height: Px, color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .width(width)
            .height(height)
            .style(color.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::ZERO,
                bottom_left: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::ZERO,
            })
            .build()
            .expect("builder construction failed"),
        || {},
    );
}

/// Render a rounded surface for a vertical thumb (radius based on width).
fn render_thumb_surface_v(width: Px, height: Px, color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .width(width)
            .height(height)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::ZERO,
                bottom_left: RoundedCorner::Capsule,
                bottom_right: RoundedCorner::ZERO,
            })
            .style(color.into())
            .build()
            .expect("builder construction failed"),
        || {},
    );
}

/// Render a rounded surface for a horizontal track (radius based on height).
fn render_track_surface_h(width: Px, height: Px, color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .width(width)
            .height(height)
            .style(color.into())
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::ZERO,
                bottom_right: RoundedCorner::ZERO,
            })
            .build()
            .expect("builder construction failed"),
        || {},
    );
}

/// Render a rounded surface for a horizontal thumb (radius based on height).
fn render_thumb_surface_h(width: Px, height: Px, color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .width(width)
            .height(height)
            .shape(Shape::RoundedRectangle {
                top_left: RoundedCorner::Capsule,
                top_right: RoundedCorner::Capsule,
                bottom_left: RoundedCorner::ZERO,
                bottom_right: RoundedCorner::ZERO,
            })
            .style(color.into())
            .build()
            .expect("builder construction failed"),
        || {},
    );
}

/// Decide whether the scrollbar should be shown according to behavior and
/// state.
fn should_show_scrollbar(args: &ScrollBarArgs, state: &ScrollBarState) -> bool {
    match args.scrollbar_behavior {
        ScrollBarBehavior::AlwaysVisible => true,
        ScrollBarBehavior::Hidden => false,
        ScrollBarBehavior::AutoHide => {
            let state_guard = state.read();
            state_guard.should_be_visible || state_guard.is_dragging || state_guard.is_hovered
        }
    }
}

/// Handle AutoHide behavior: hide the scrollbar after a timeout if no activity.
fn handle_autohide_if_needed(args: &ScrollBarArgs, state: &ScrollBarState) {
    if matches!(args.scrollbar_behavior, ScrollBarBehavior::AutoHide) {
        let mut state_guard = state.write();
        if let Some(last_activity) = state_guard.last_scroll_activity {
            // Hide scrollbar after 2 seconds of inactivity
            if last_activity.elapsed().as_secs_f32() > 2.0 {
                state_guard.should_be_visible = false;
            }
        }
    }
}

/// Mark recent scroll activity and make the scrollbar visible (used by AutoHide
/// behavior).
fn mark_scroll_activity(state: &ScrollBarState, behavior: &ScrollBarBehavior) {
    if matches!(*behavior, ScrollBarBehavior::AutoHide) {
        let mut state_guard = state.write();
        state_guard.last_scroll_activity = Some(std::time::Instant::now());
        state_guard.should_be_visible = true;
    }
}

/// Compute normalized thumb progress (0.0..1.0) from offset/total.
///
/// Returns 0.0 if total is zero or non-positive to avoid division by zero.
fn compute_thumb_progress(offset: Px, total: Px) -> f32 {
    if total <= Px::ZERO {
        0.0
    } else {
        offset.to_f32().abs() / total.to_f32()
    }
}

/// Compute the thumb size (Px) from visible and total content sizes using the
/// proportional formula: thumb = visible * visible / total. When `total` is
/// zero or non-positive, fall back to using `visible` to avoid division by
/// zero.
fn compute_thumb_size(visible: Px, total: Px) -> Px {
    if total <= Px::ZERO {
        return visible.max(Px::ZERO);
    }
    let visible_len = visible.to_f32().abs();
    let total_len = total.to_f32().abs().max(1.0);
    let thumb = (visible_len * visible_len) / total_len;

    // Clamp the thumb size to ensure it's always visible and provides a reasonable
    // drag target.
    let min_thumb = (visible_len * 0.05).clamp(8.0, 32.0);
    Px::saturating_from_f32(thumb.max(min_thumb))
}

/// Helper to check whether a cursor position overlaps the vertical thumb.
fn cursor_on_thumb_v(cursor_pos: PxPosition, width: Px, thumb_y: f32, thumb_height: Px) -> bool {
    cursor_pos.x >= Px::ZERO
        && cursor_pos.x <= width
        && cursor_pos.y >= Px::from_f32(thumb_y)
        && cursor_pos.y <= Px::from_f32(thumb_y + thumb_height.to_f32())
}

/// Helper to check whether a cursor position overlaps the horizontal thumb.
fn cursor_on_thumb_h(cursor_pos: PxPosition, height: Px, thumb_x: f32, thumb_width: Px) -> bool {
    cursor_pos.y >= Px::ZERO
        && cursor_pos.y <= height
        && cursor_pos.x >= Px::from_f32(thumb_x)
        && cursor_pos.x <= Px::from_f32(thumb_x + thumb_width.to_f32())
}

/// Return true if the cursor press position is on vertical track area.
fn is_on_track_v(cursor_pos: PxPosition, thickness: Px, track_height: Px) -> bool {
    cursor_pos.x >= Px::ZERO
        && cursor_pos.x <= thickness
        && cursor_pos.y >= Px::ZERO
        && cursor_pos.y <= track_height
}

/// Return true if the cursor press position is on horizontal track area.
fn is_on_track_h(cursor_pos: PxPosition, thickness: Px, track_width: Px) -> bool {
    cursor_pos.y >= Px::ZERO
        && cursor_pos.y <= thickness
        && cursor_pos.x >= Px::ZERO
        && cursor_pos.x <= track_width
}

/// Handle the input handler logic for the vertical scrollbar.
/// Extracted from the inline closure to reduce function/closure complexity.
fn check_and_handle_release(input: &tessera_ui::InputHandlerInput, state: &ScrollBarState) -> bool {
    if input.cursor_events.iter().any(|event| {
        matches!(
            event.content,
            CursorEventContent::Released(PressKeyEventType::Left)
        )
    }) {
        state.write().is_dragging = false;
        true
    } else {
        false
    }
}

fn apply_scrollbar_accessibility(
    input: &mut tessera_ui::InputHandlerInput<'_>,
    args: &ScrollBarArgs,
    state: &ScrollBarState,
    orientation: ScrollOrientation,
) {
    let mut builder = input.accessibility().role(Role::ScrollBar);
    let label = match orientation {
        ScrollOrientation::Vertical => "Vertical scrollbar",
        ScrollOrientation::Horizontal => "Horizontal scrollbar",
    };
    builder = builder.label(label.to_string());

    let progress = compute_thumb_progress(args.offset, args.total).clamp(0.0, 1.0);
    builder = builder
        .numeric_value(progress as f64)
        .numeric_range(0.0, 1.0)
        .focusable()
        .action(Action::Increment)
        .action(Action::Decrement);

    builder.commit();

    let args_clone = args.clone();
    let state_clone = state.clone();
    input.set_accessibility_action_handler(move |action| match action {
        Action::Increment => {
            scroll_accessibility_step(&args_clone, &state_clone, orientation, true)
        }
        Action::Decrement => {
            scroll_accessibility_step(&args_clone, &state_clone, orientation, false)
        }
        _ => {}
    });
}

fn scroll_accessibility_step(
    args: &ScrollBarArgs,
    state: &ScrollBarState,
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

    {
        let mut scroll_state = args.state.write();
        let new_target = scroll_state
            .target_position
            .saturating_offset(delta.x, delta.y);
        scroll_state.set_target_position(new_target);
    }

    if matches!(args.scrollbar_behavior, ScrollBarBehavior::AutoHide) {
        let mut scroll_state = state.write();
        scroll_state.last_scroll_activity = Some(std::time::Instant::now());
        scroll_state.should_be_visible = true;
    }
}

/// Return true if there is a left-press event in the input.
/// Extracted to reduce duplication and simplify input handlers.
fn is_pressed_left(input: &tessera_ui::InputHandlerInput) -> bool {
    input.cursor_events.iter().any(|event| {
        matches!(
            event.content,
            CursorEventContent::Pressed(PressKeyEventType::Left)
        )
    })
}

/// Update dragging behavior for vertical axis.
fn update_drag_vertical(
    input: &tessera_ui::InputHandlerInput,
    calculate_target: &dyn Fn(Px) -> PxPosition,
    args: &ScrollBarArgs,
    state: &ScrollBarState,
) {
    if let Some(cursor_pos) = input.cursor_position_rel {
        let new_target_pos = calculate_target(cursor_pos.y);
        args.state.write().set_target_position(new_target_pos);
        mark_scroll_activity(state, &args.scrollbar_behavior);
    } else {
        // Cursor left window: stop dragging.
        state.write().is_dragging = false;
    }
}

/// Update hovered state uniformly.
fn update_hover_state(is_on_thumb: bool, state: &ScrollBarState) {
    if is_on_thumb && !state.read().is_hovered {
        let mut state_guard = state.write();
        state_guard.is_hovered = true;
        state_guard.hover_instant = Some(std::time::Instant::now());
    } else if !is_on_thumb && state.read().is_hovered {
        let mut state_guard = state.write();
        state_guard.is_hovered = false;
        state_guard.hover_instant = Some(std::time::Instant::now());
    }
}

fn handle_state_v(
    args: &ScrollBarArgs,
    state: &ScrollBarState,
    track_height: Px,
    thumb_height: Px,
    input: &mut tessera_ui::InputHandlerInput<'_>,
) {
    // Handle AutoHide behavior - hide scrollbar after inactivity
    handle_autohide_if_needed(args, state);

    // Capture current target position once to avoid locking inside helper on every
    // call.
    let fallback_pos = args.state.target_position();
    let calculate_target_pos = |cursor_y: Px| -> PxPosition {
        calculate_target_pos_v(
            cursor_y,
            track_height,
            thumb_height,
            args.total,
            args.visible,
            fallback_pos,
        )
    };

    if state.read().is_dragging {
        // If mouse released, stop dragging (extracted helper reduces branching
        // complexity).
        if check_and_handle_release(input, state) {
            return;
        }

        // Update dragging position or stop if cursor left.
        update_drag_vertical(input, &calculate_target_pos, args, state);
    } else {
        // Not dragging, check for interactions to start dragging or jump
        let Some(cursor_pos) = input.cursor_position_rel else {
            state.write().is_hovered = false; // Reset hover state if no cursor
            return; // No cursor, do nothing
        };

        // Check if the cursor is on the thumb
        let is_on_thumb = cursor_on_thumb_v(
            cursor_pos,
            args.thickness.to_px(),
            args.visible.to_f32() * (args.offset.to_f32().abs() / args.total.to_f32()),
            thumb_height,
        );

        // Update hover state (extracted).
        update_hover_state(is_on_thumb, state);

        // Check for left mouse button press
        if !is_pressed_left(input) {
            return; // No press, do nothing
        }

        if is_on_thumb {
            // Start dragging
            state.write().is_dragging = true;
            return;
        }

        // Check if the press is on the track
        if is_on_track_v(cursor_pos, args.thickness.to_px(), track_height) {
            // Jump to the clicked position
            let new_target_pos = calculate_target_pos(cursor_pos.y);
            args.state.write().set_target_position(new_target_pos);
        }
    }
}

/// Handle the input handler logic for the horizontal scrollbar.
/// Extracted from the inline closure to reduce function/closure complexity.
fn update_drag_horizontal(
    input: &tessera_ui::InputHandlerInput,
    calculate_target: &dyn Fn(Px) -> PxPosition,
    args: &ScrollBarArgs,
    state: &ScrollBarState,
) {
    if let Some(cursor_pos) = input.cursor_position_rel {
        let new_target_pos = calculate_target(cursor_pos.x);
        args.state.write().set_target_position(new_target_pos);
        mark_scroll_activity(state, &args.scrollbar_behavior);
    } else {
        // Cursor left window: stop dragging.
        state.write().is_dragging = false;
    }
}

fn handle_state_h(
    args: &ScrollBarArgs,
    state: &ScrollBarState,
    track_width: Px,
    thumb_width: Px,
    input: &mut tessera_ui::InputHandlerInput<'_>,
) {
    // Handle AutoHide behavior - hide scrollbar after inactivity
    handle_autohide_if_needed(args, state);

    // Capture current target position once to avoid locking inside helper on every
    // call.
    let fallback_pos = args.state.target_position();
    let calculate_target_pos = |cursor_x: Px| -> PxPosition {
        calculate_target_pos_h(
            cursor_x,
            track_width,
            thumb_width,
            args.total,
            args.visible,
            fallback_pos,
        )
    };

    if state.read().is_dragging {
        // If mouse released, stop dragging (extracted helper).
        if check_and_handle_release(input, state) {
            return;
        }

        // Update dragging position or stop if cursor left.
        update_drag_horizontal(input, &calculate_target_pos, args, state);
    } else {
        // Not dragging, check for interactions to start dragging or jump
        let Some(cursor_pos) = input.cursor_position_rel else {
            state.write().is_hovered = false; // Reset hover state if no cursor
            return; // No cursor, do nothing
        };

        // Check if the cursor is on the thumb
        let is_on_thumb = cursor_on_thumb_h(
            cursor_pos,
            args.thickness.to_px(),
            args.visible.to_f32() * (args.offset.to_f32().abs() / args.total.to_f32()),
            thumb_width,
        );

        // Update hover state (re-use helper).
        update_hover_state(is_on_thumb, state);

        if !is_pressed_left(input) {
            return;
        }

        if is_on_thumb {
            // Start dragging
            state.write().is_dragging = true;
            return;
        }

        // Check if the press is on the track
        if is_on_track_h(cursor_pos, args.thickness.to_px(), track_width) {
            // Jump to the clicked position
            let new_target_pos = calculate_target_pos(cursor_pos.x);
            args.state.write().set_target_position(new_target_pos);
        }
    }
}

#[tessera]
pub fn scrollbar_v(args: impl Into<ScrollBarArgs>, state: ScrollBarState) {
    let args: ScrollBarArgs = args.into();

    // Check if scrollbar should be visible based on behavior
    let should_show = should_show_scrollbar(&args, &state);

    // If scrollbar should be hidden, don't render anything
    if !should_show {
        return;
    }

    // Ensure the scrollbar is visible
    if args.visible <= Px::ZERO || args.total <= Px::ZERO || args.thickness <= Dp::ZERO {
        return;
    }

    let width = args.thickness.to_px();
    let track_height = args.visible;
    let thumb_height = compute_thumb_size(args.visible, args.total);
    let has_vertical_overflow = args.total > args.visible;
    let track_color = if has_vertical_overflow {
        args.track_color
    } else {
        args.track_color.with_alpha(0.0)
    };

    // track surface
    render_track_surface_v(width, track_height, track_color);

    let thumb_color = if has_vertical_overflow {
        compute_thumb_color(&state, &args)
    } else {
        args.thumb_color.with_alpha(0.0)
    };

    // thumb surface
    render_thumb_surface_v(width, thumb_height, thumb_color);

    // Calculate the position of the thumb based on the scroll offset and total size
    let progress = compute_thumb_progress(args.offset, args.total);
    let thumb_y = args.visible.to_f32() * progress;

    measure(Box::new(move |input| {
        // measure track
        let track_node_id = input.children_ids[0];
        let size = input.measure_child(track_node_id, &Constraint::NONE)?; // No constraints need since it's size is fixed
        // place track at the top left corner
        input.place_child(track_node_id, [0, 0].into());
        // measure thumb
        let thumb_node_id = input.children_ids[1];
        input.measure_child(thumb_node_id, &Constraint::NONE)?; // No constraints need since it's size is fixed
        // place thumb
        input.place_child(thumb_node_id, [0, thumb_y as i32].into());
        // Return the size of the scrollbar track
        Ok(size)
    }));

    let args_for_handler = args.clone();
    let state_for_handler = state.clone();
    input_handler(Box::new(move |mut input| {
        handle_state_v(
            &args_for_handler,
            &state_for_handler,
            track_height,
            thumb_height,
            &mut input,
        );
        apply_scrollbar_accessibility(
            &mut input,
            &args_for_handler,
            &state_for_handler,
            ScrollOrientation::Vertical,
        );
    }));
}

#[tessera]
pub fn scrollbar_h(args: impl Into<ScrollBarArgs>, state: ScrollBarState) {
    let args: ScrollBarArgs = args.into();

    // Check if scrollbar should be visible based on behavior
    let should_show = should_show_scrollbar(&args, &state);

    // If scrollbar should be hidden, don't render anything
    if !should_show {
        return;
    }

    // Ensure the scrollbar is visible
    if args.visible <= Px::ZERO || args.total <= Px::ZERO || args.thickness <= Dp::ZERO {
        return;
    }

    let height = args.thickness.to_px();
    let track_width = args.visible;
    let thumb_width = compute_thumb_size(args.visible, args.total);
    let has_horizontal_overflow = args.total > args.visible;
    let track_color = if has_horizontal_overflow {
        args.track_color
    } else {
        args.track_color.with_alpha(0.0)
    };

    // track surface
    render_track_surface_h(track_width, height, track_color);

    let thumb_color = if has_horizontal_overflow {
        compute_thumb_color(&state, &args)
    } else {
        args.thumb_color.with_alpha(0.0)
    };

    // thumb surface
    render_thumb_surface_h(thumb_width, height, thumb_color);

    // Calculate the position of the thumb based on the scroll offset and total size
    let progress = compute_thumb_progress(args.offset, args.total);
    let thumb_x = args.visible.to_f32() * progress;

    measure(Box::new(move |input| {
        // measure track
        let track_node_id = input.children_ids[0];
        let size = input.measure_child(track_node_id, &Constraint::NONE)?;
        // place track at the top left corner
        input.place_child(track_node_id, [0, 0].into());
        // measure thumb
        let thumb_node_id = input.children_ids[1];
        input.measure_child(thumb_node_id, &Constraint::NONE)?;
        // place thumb
        input.place_child(thumb_node_id, [thumb_x as i32, 0].into());
        // Return the size of the scrollbar track
        Ok(size)
    }));

    let args_for_handler = args.clone();
    let state_for_handler = state.clone();
    input_handler(Box::new(move |mut input| {
        handle_state_h(
            &args_for_handler,
            &state_for_handler,
            track_width,
            thumb_width,
            &mut input,
        );
        apply_scrollbar_accessibility(
            &mut input,
            &args_for_handler,
            &state_for_handler,
            ScrollOrientation::Horizontal,
        );
    }));
}
