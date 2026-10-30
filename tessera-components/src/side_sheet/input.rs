//! Pointer gestures for side-sheet content.
//!
//! ## Usage
//!
//! Drag supporting panels while preserving interactive child input priority.

use std::time::Duration;

use tessera_foundation::{
    gesture::{DragAxis, DragRecognizer, DragSettings},
    scroll_physics::ScrollVelocityTracker,
};
use tessera_ui::{
    Callback, CursorEventContent, Dp, Modifier, PointerEventPass, PointerInput,
    PointerInputModifierNode, PressKeyEventType, PxPosition, RenderSlot, State,
    current_frame_nanos, current_frame_time, layout::layout, modifier::ModifierCapabilityExt as _,
    provide_context, remember, tessera, time::Instant,
};

use crate::{
    boxed::boxed, modifier::ModifierExt as _, nested_scroll::NestedScrollConnection,
    pos_misc::is_position_inside_bounds,
};

use super::{SideSheetController, SideSheetPosition, side_sheet_dimension};

struct SheetPointerGesture {
    recognizer: DragRecognizer,
    velocity: Option<ScrollVelocityTracker>,
    pointer: Option<u64>,
    pending_delta: f32,
    pending_motion: bool,
    claimed: bool,
}

#[derive(Default)]
struct GestureEnd {
    released: bool,
    cancelled: bool,
    velocity: f32,
}

impl SheetPointerGesture {
    fn new() -> Self {
        Self {
            recognizer: DragRecognizer::new(DragSettings {
                axis: Some(DragAxis::Horizontal),
                consume_when_dragging: false,
                ..Default::default()
            }),
            velocity: None,
            pointer: None,
            pending_delta: 0.0,
            pending_motion: false,
            claimed: false,
        }
    }

    fn preview(&mut self, input: &mut PointerInput<'_>) -> GestureEnd {
        let now = current_frame_time().unwrap_or_else(Instant::now);
        let within = input
            .cursor_position_rel
            .map(|p| is_position_inside_bounds(input.computed_data, p))
            .unwrap_or(false);
        self.recognizer.set_settings(DragSettings {
            axis: Some(DragAxis::Horizontal),
            slop_px: Dp(8.0).to_pixels_f32(),
            consume_when_dragging: false,
        });
        for change in input.pointer_changes.iter() {
            if self.pointer.is_none()
                && within
                && matches!(
                    change.content,
                    CursorEventContent::Pressed(PressKeyEventType::Left)
                )
            {
                self.pointer = Some(change.pointer_id);
                self.claimed = false;
                self.velocity = Some(ScrollVelocityTracker::new(
                    now,
                    Duration::from_millis(90),
                    Duration::from_millis(65),
                ));
            }
        }
        let cancelled = input.pointer_changes.iter().any(|c| {
            Some(c.pointer_id) == self.pointer
                && matches!(
                    c.content,
                    CursorEventContent::Cancelled(PressKeyEventType::Left)
                )
        });
        let sample = input
            .pointer_changes
            .iter_mut()
            .find(|change| Some(change.pointer_id) == self.pointer);
        let result = self.recognizer.update(
            PointerEventPass::Main,
            sample.map(std::slice::from_mut).unwrap_or(&mut []),
            within,
        );
        self.pending_delta = input.local_delta(result.delta_x.to_f32(), 0.0).0;
        self.pending_motion = result.updated;
        if cancelled || result.ended {
            let end = GestureEnd {
                released: result.ended && self.claimed,
                cancelled,
                velocity: self
                    .velocity
                    .take()
                    .and_then(|mut v| v.resolve(now))
                    .map(|v| input.local_delta(v.x, 0.0).0)
                    .unwrap_or(0.0),
            };
            self.claimed = false;
            self.pointer = None;
            return end;
        }
        GestureEnd::default()
    }

    fn main(&mut self, input: &mut PointerInput<'_>) -> Option<(bool, f32)> {
        let moved = input.pointer_changes.iter().any(|c| {
            !c.is_consumed()
                && Some(c.pointer_id) == self.pointer
                && matches!(c.content, CursorEventContent::Moved(_))
        });
        if !moved || (!self.claimed && !self.pending_motion) {
            return None;
        }
        let started = !self.claimed;
        self.claimed = true;
        let now = current_frame_time().unwrap_or_else(Instant::now);
        if let Some(velocity) = &mut self.velocity {
            let window = input
                .local_to_window
                .map_vector(PxPosition::from_f32_xy(self.pending_delta, 0.0));
            velocity.push_delta(now, window.x, window.y);
        }
        input.consume_pointer_changes();
        Some((started, self.pending_delta))
    }
}

#[derive(Clone, Copy)]
struct SheetPointerNode {
    gesture: State<SheetPointerGesture>,
    controller: State<SideSheetController>,
    position: State<SideSheetPosition>,
    on_close: Callback,
}

impl PointerInputModifierNode for SheetPointerNode {
    fn on_pointer_input(&self, mut input: PointerInput<'_>) {
        if input.pointer_changes.is_empty() {
            return;
        }
        if input.pass == PointerEventPass::Initial {
            let end = self.gesture.with_mut(|gesture| gesture.preview(&mut input));
            if end.cancelled {
                self.controller
                    .with_mut(|c| c.drag.settle(current_frame_nanos()));
            }
            if end.released
                && self
                    .controller
                    .with_mut(|c| c.complete_drag(end.velocity, self.position.get()))
            {
                self.on_close.call();
            }
        } else if input.pass == PointerEventPass::Main {
            if let Some((started, delta)) =
                self.gesture.with_mut(|gesture| gesture.main(&mut input))
            {
                self.controller.with_mut(|controller| {
                    if started {
                        controller.drag.grab(current_frame_nanos());
                    }
                    controller.apply_drag_delta(delta, self.position.get());
                });
            }
        }
    }
}

#[tessera]
pub(super) fn side_sheet_body(
    content_padding: Option<Dp>,
    controller: Option<State<SideSheetController>>,
    position: Option<State<SideSheetPosition>>,
    on_close: Option<Callback>,
    nested_scroll_connection: Option<NestedScrollConnection>,
    content: Option<RenderSlot>,
) {
    let content_padding = side_sheet_dimension(content_padding, Dp(24.0));
    let controller = controller.expect("sheet body requires its controller");
    let position = position.expect("sheet body requires its edge state");
    let on_close = on_close.unwrap_or_default();
    let content = content.unwrap_or_else(RenderSlot::empty);
    let nested_scroll_connection = nested_scroll_connection.unwrap_or_default();
    let gesture = remember(SheetPointerGesture::new);
    let node = SheetPointerNode {
        gesture,
        controller,
        position,
        on_close,
    };
    let modifier = Modifier::new()
        .fill_max_size()
        .push_pointer_preview_input(node)
        .push_pointer_input(node);
    boxed().modifier(modifier).children(move || {
        let nested_scroll_connection = nested_scroll_connection.clone();
        layout()
            .modifier(Modifier::new().padding_all(content_padding))
            .child(move || {
                provide_context(
                    || nested_scroll_connection.clone(),
                    move || {
                        content.render();
                    },
                );
            });
    });
}
