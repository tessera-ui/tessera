//! Pointer gestures for side-sheet content.
//!
//! ## Usage
//!
//! Drag supporting panels while preserving interactive child input priority.

use std::time::Duration;

use tessera_foundation::{modifier::ModifierExt as _, scroll_physics::ScrollVelocityTracker};
use tessera_ui::{
    Callback, CursorEventContent, Dp, Modifier, PointerEventPass, PointerInputScope,
    PressKeyEventType, PxPosition, RenderSlot, State,
    layout::{Stack, layout},
    tessera,
};

use crate::{boxed::boxed, nested_scroll::NestedScrollConnection};

use super::{SideSheetController, SideSheetPosition, side_sheet_dimension};

async fn sheet_pointer_detector(
    mut scope: PointerInputScope,
    controller: State<SideSheetController>,
    position: State<SideSheetPosition>,
    on_close: Callback,
) {
    struct DragLease {
        controller: State<SideSheetController>,
        claimed: bool,
    }
    impl Drop for DragLease {
        fn drop(&mut self) {
            if self.claimed && self.controller.is_alive() {
                self.controller.with_mut(SideSheetController::settle_drag);
            }
        }
    }
    loop {
        let Some(down) = scope
            .await_first_down(
                PointerEventPass::Initial,
                Some(PressKeyEventType::Left),
                tessera_ui::DownRequirement::Any,
            )
            .await
        else {
            return;
        };
        let Some(start) = down.position else { continue };
        let mut previous = start;
        let mut drag = DragLease {
            controller,
            claimed: false,
        };
        let mut velocity = ScrollVelocityTracker::new(
            down.change.timestamp,
            Duration::from_millis(90),
            Duration::from_millis(65),
        );
        loop {
            let event = scope.await_pointer_event(PointerEventPass::Initial).await;
            if event.is_cancelled() {
                return;
            }
            let Some(change) = event
                .changes
                .iter()
                .find(|change| change.pointer_id == down.change.pointer_id)
            else {
                continue;
            };
            match change.content {
                CursorEventContent::Cancelled(_) => {
                    break;
                }
                CursorEventContent::Released(_) => {
                    if drag.claimed && !change.is_consumed() {
                        drag.claimed = false;
                        let velocity = velocity
                            .resolve(change.timestamp)
                            .map(|v| {
                                event
                                    .local_to_window
                                    .inverse_vector(PxPosition::from_f32_xy(v.x, 0.0))
                                    .x
                            })
                            .unwrap_or(0.0);
                        if controller.with_mut(|c| c.complete_drag(velocity, position.get())) {
                            on_close.call();
                        }
                    }
                    break;
                }
                CursorEventContent::Moved(point) => {
                    let travel = point - start;
                    let delta = point - previous;
                    previous = point;
                    if !drag.claimed
                        && (travel.x.abs() < Dp(8.0).to_pixels_f32()
                            || travel.x.abs() < travel.y.abs())
                    {
                        continue;
                    }
                    let event = scope.await_pointer_event(PointerEventPass::Main).await;
                    if event.is_cancelled() {
                        return;
                    }
                    if let Some(mut change) = event.changes.into_iter().find(|change| {
                        change.pointer_id == down.change.pointer_id && !change.is_consumed()
                    }) {
                        let window_delta = delta;
                        let local = event.local_to_window.inverse_vector(window_delta);
                        controller.with_mut(|controller| {
                            if !drag.claimed {
                                controller.grab();
                            }
                            controller.apply_drag_delta(local.x, position.get());
                        });
                        drag.claimed = true;
                        velocity.push_delta(change.timestamp, window_delta.x, 0.0);
                        change.consume();
                    } else {
                        break;
                    }
                }
                _ => {}
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
    let modifier = Modifier::new()
        .fill_max_size()
        .pointer_input((controller, position, on_close), move |scope| {
            sheet_pointer_detector(scope, controller, position, on_close)
        });
    boxed().modifier(modifier).children(move || {
        let nested_scroll_connection = nested_scroll_connection.clone();
        layout(Stack)
            .modifier(
                Modifier::new()
                    .padding_all(content_padding)
                    .nested_scroll(nested_scroll_connection.clone()),
            )
            .child(move || {
                content.render();
            });
    });
}
