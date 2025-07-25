use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, Constraint, CursorEventContent, Dp, PressKeyEventType, Px, PxPosition};
use tessera_ui_macros::tessera;

use crate::{
    scrollable::ScrollableStateInner,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

#[derive(Clone, Debug)]
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
    pub state: Arc<RwLock<ScrollableStateInner>>,
    /// The color of the scrollbar track.
    pub track_color: Color,
    /// The color of the scrollbar thumb.
    pub thumb_color: Color,
    /// The color of the scrollbar thumb when hovered.
    pub thumb_hover_color: Color,
}

#[derive(Default)]
pub struct ScrollBarState {
    /// Whether the scrollbar's thumb is currently being dragged.
    pub is_dragging: bool,
    /// Whether the scrollbar's thumb is currently being hovered.
    pub is_hovered: bool,
    /// The instant when the hover state last changed.
    pub hover_instant: Option<std::time::Instant>,
}

#[tessera]
pub fn scrollbar_v(args: impl Into<ScrollBarArgs>, state: Arc<RwLock<ScrollBarState>>) {
    let args: ScrollBarArgs = args.into();

    // Ensure the scrollbar is visible
    if args.visible <= Px::ZERO || args.total <= Px::ZERO || args.thickness <= Dp::ZERO {
        return;
    }

    let width = args.thickness.to_px();
    let track_height = args.visible;
    let thumb_height = args.visible * args.visible / args.total;

    // track surface
    surface(
        SurfaceArgsBuilder::default()
            .width(width.into())
            .height(track_height.into())
            .color(args.track_color)
            .shape(Shape::RoundedRectangle {
                corner_radius: width.to_f32() / 2.0,
                g2_k_value: 2.0,
            })
            .build()
            .unwrap(),
        None,
        || {},
    );

    let thumb_color = {
        let state = state.read();
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
    };

    // thumb surface
    surface(
        SurfaceArgsBuilder::default()
            .width(width.into())
            .height(thumb_height.into())
            .shape(Shape::RoundedRectangle {
                corner_radius: width.to_f32() / 2.0,
                g2_k_value: 2.0,
            })
            .color(thumb_color)
            .build()
            .unwrap(),
        None,
        || {},
    );

    // Calculate the position of the thumb based on the scroll offset and total size
    let progress = args.offset.to_f32().abs() / (args.total).to_f32();
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

    state_handler(Box::new(move |input| {
        // A helper function to calculate the target position based on cursor's y
        let calculate_target_pos = |cursor_y: Px| -> PxPosition {
            // The scrollable range of the thumb within the track
            let thumb_scrollable_range = track_height - thumb_height;
            if thumb_scrollable_range <= Px::ZERO {
                return args.state.read().target_position;
            }

            // Adjust cursor position to be relative to the thumb's center
            let cursor_y_adjusted = cursor_y - thumb_height / 2;

            // Calculate scroll progress (0.0 to 1.0)
            let progress =
                (cursor_y_adjusted.to_f32() / thumb_scrollable_range.to_f32()).clamp(0.0, 1.0);

            // Calculate the total scrollable range of the content
            let content_scrollable_range = args.total - args.visible;
            if content_scrollable_range <= Px::ZERO {
                return PxPosition::ZERO;
            }

            // Calculate the new absolute target Y position for the content
            let new_target_y = Px::from_f32(-progress * content_scrollable_range.to_f32());

            PxPosition {
                x: Px::ZERO, // Vertical scrollbar doesn't affect X
                y: new_target_y,
            }
        };

        if state.read().is_dragging {
            // Check for left mouse button release to stop dragging
            if input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Left)
                )
            }) {
                state.write().is_dragging = false;
                return;
            }

            // If dragging, update target position based on cursor
            if let Some(cursor_pos) = input.cursor_position {
                let new_target_pos = calculate_target_pos(cursor_pos.y);
                args.state.write().set_target_position(new_target_pos);
            } else {
                // Cursor is outside the window, stop dragging
                state.write().is_dragging = false;
            }
        } else {
            // Not dragging, check for interactions to start dragging or jump
            let Some(cursor_pos) = input.cursor_position else {
                state.write().is_hovered = false; // Reset hover state if no cursor
                return; // No cursor, do nothing
            };

            // Check if the cursor is on the thumb
            let is_on_thumb = cursor_pos.x >= Px::ZERO
                && cursor_pos.x <= width
                && cursor_pos.y >= Px::from_f32(thumb_y)
                && cursor_pos.y <= Px::from_f32(thumb_y + thumb_height.to_f32());

            if is_on_thumb && !state.read().is_hovered {
                let mut state = state.write();
                state.is_hovered = true;
                state.hover_instant = Some(std::time::Instant::now());
            } else if !is_on_thumb && state.read().is_hovered {
                let mut state = state.write();
                state.is_hovered = false;
                state.hover_instant = Some(std::time::Instant::now());
            }

            // Check for left mouse button press
            if !input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            }) {
                return; // No press, do nothing
            }

            if is_on_thumb {
                // Start dragging
                state.write().is_dragging = true;
                return;
            }

            // Check if the press is on the track
            let is_on_track = cursor_pos.x >= Px::ZERO
                && cursor_pos.x <= width
                && cursor_pos.y >= Px::ZERO
                && cursor_pos.y <= track_height;

            if is_on_track {
                // Jump to the clicked position
                let new_target_pos = calculate_target_pos(cursor_pos.y);
                args.state.write().set_target_position(new_target_pos);
            }
        }
    }));
}

#[tessera]
pub fn scrollbar_h(args: impl Into<ScrollBarArgs>, state: Arc<RwLock<ScrollBarState>>) {
    let args: ScrollBarArgs = args.into();

    // Ensure the scrollbar is visible
    if args.visible <= Px::ZERO || args.total <= Px::ZERO || args.thickness <= Dp::ZERO {
        return;
    }

    let height = args.thickness.to_px();
    let track_width = args.visible;
    let thumb_width = args.visible * args.visible / args.total;

    // track surface
    surface(
        SurfaceArgsBuilder::default()
            .width(track_width.into())
            .height(height.into())
            .color(args.track_color)
            .shape(Shape::RoundedRectangle {
                corner_radius: height.to_f32() / 2.0,
                g2_k_value: 2.0,
            })
            .build()
            .unwrap(),
        None,
        || {},
    );

    let thumb_color = {
        let state = state.read();
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
    };

    // thumb surface
    surface(
        SurfaceArgsBuilder::default()
            .width(thumb_width.into())
            .height(height.into())
            .shape(Shape::RoundedRectangle {
                corner_radius: height.to_f32() / 2.0,
                g2_k_value: 2.0,
            })
            .color(thumb_color)
            .build()
            .unwrap(),
        None,
        || {},
    );

    // Calculate the position of the thumb based on the scroll offset and total size
    let progress = args.offset.to_f32().abs() / (args.total).to_f32();
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

    state_handler(Box::new(move |input| {
        // A helper function to calculate the target position based on cursor's x
        let calculate_target_pos = |cursor_x: Px| -> PxPosition {
            // The scrollable range of the thumb within the track
            let thumb_scrollable_range = track_width - thumb_width;
            if thumb_scrollable_range <= Px::ZERO {
                return args.state.read().target_position;
            }

            // Adjust cursor position to be relative to the thumb's center
            let cursor_x_adjusted = cursor_x - thumb_width / 2;

            // Calculate scroll progress (0.0 to 1.0)
            let progress =
                (cursor_x_adjusted.to_f32() / thumb_scrollable_range.to_f32()).clamp(0.0, 1.0);

            // Calculate the total scrollable range of the content
            let content_scrollable_range = args.total - args.visible;
            if content_scrollable_range <= Px::ZERO {
                return PxPosition::ZERO;
            }

            // Calculate the new absolute target X position for the content
            let new_target_x = Px::from_f32(-progress * content_scrollable_range.to_f32());

            PxPosition {
                x: new_target_x,
                y: Px::ZERO, // Horizontal scrollbar doesn't affect Y
            }
        };

        if state.read().is_dragging {
            // Check for left mouse button release to stop dragging
            if input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Released(PressKeyEventType::Left)
                )
            }) {
                state.write().is_dragging = false;
                return;
            }

            // If dragging, update target position based on cursor
            if let Some(cursor_pos) = input.cursor_position {
                let new_target_pos = calculate_target_pos(cursor_pos.x);
                args.state.write().set_target_position(new_target_pos);
            } else {
                // Cursor is outside the window, stop dragging
                state.write().is_dragging = false;
            }
        } else {
            // Not dragging, check for interactions to start dragging or jump
            let Some(cursor_pos) = input.cursor_position else {
                state.write().is_hovered = false; // Reset hover state if no cursor
                return; // No cursor, do nothing
            };

            // Check if the press is on the thumb
            let is_on_thumb = cursor_pos.y >= Px::ZERO
                && cursor_pos.y <= height
                && cursor_pos.x >= Px::from_f32(thumb_x)
                && cursor_pos.x <= Px::from_f32(thumb_x + thumb_width.to_f32());

            if is_on_thumb && !state.read().is_hovered {
                let mut state = state.write();
                state.is_hovered = true;
                state.hover_instant = Some(std::time::Instant::now());
            } else if !is_on_thumb && state.read().is_hovered {
                let mut state = state.write();
                state.is_hovered = false;
                state.hover_instant = Some(std::time::Instant::now());
            }

            if is_on_thumb {
                // Start dragging
                state.write().is_dragging = true;
                return;
            }

            // Check for left mouse button press
            if !input.cursor_events.iter().any(|event| {
                matches!(
                    event.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            }) {
                return; // No press, do nothing
            }

            // Check if the press is on the track
            let is_on_track = cursor_pos.y >= Px::ZERO
                && cursor_pos.y <= height
                && cursor_pos.x >= Px::ZERO
                && cursor_pos.x <= track_width;

            if is_on_track {
                // Jump to the clicked position
                let new_target_pos = calculate_target_pos(cursor_pos.x);
                args.state.write().set_target_position(new_target_pos);
            }
        }
    }));
}
