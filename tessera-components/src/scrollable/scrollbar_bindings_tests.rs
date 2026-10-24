//! Controller-bound scrollbar metrics and physical input.
//!
//! ## Usage
//!
//! Operate independent scrollbar tracks beside scaled scroll viewports.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    CallbackWith, Color, CursorEventContent, Dp, GestureState, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PointerChange, PressKeyEventType, Px, PxPosition, ScrollDeltaUnit,
    ScrollEventContent, ScrollEventSource, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    current_frame_time, provide_context, remember, scaled_box, tessera,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    modifier::ModifierExt as _,
    nested_scroll::{NestedScrollConnection, PostScrollInput, ScrollDelta},
    row::row,
    scrollable::{ScrollableController, scrollbar_h, scrollbar_v},
    theme::material_theme,
};

thread_local! {
    static CASE: Cell<(bool, bool)> = const { Cell::new((false, false)) };
    static BOTH_AXES: Cell<bool> = const { Cell::new(false) };
    static CONTROLLER: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
    static ACTIVATIONS: Cell<usize> = const { Cell::new(0) };
    static REMAINDER: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        scrollbar_fixture();
    });
}

#[tessera]
fn scrollbar_fixture() {
    let controller = remember(ScrollableController::new);
    CONTROLLER.set(Some(controller));
    let (custom, horizontal) = CASE.get();
    let both_axes = BOTH_AXES.get();
    let connection = NestedScrollConnection::new().with_post_scroll_handler(CallbackWith::new(
        |input: PostScrollInput| {
            REMAINDER.set((input.available.x, input.available.y));
            ScrollDelta::ZERO
        },
    ));
    provide_context(
        move || connection.clone(),
        move || {
            row().children(move || {
                scaled_box(0.625).child(move || {
                    let modifier = Modifier::new().size(Dp(200.0), Dp(200.0));
                    let modifier = if both_axes {
                        crate::scrollable::apply_scroll_modifier(modifier, controller, true, true)
                    } else if horizontal {
                        modifier.horizontal_scroll(controller)
                    } else {
                        modifier.vertical_scroll(controller)
                    };
                    boxed()
                        .alignment(Alignment::TopStart)
                        .modifier(modifier)
                        .children(move || {
                            boxed()
                                .alignment(if both_axes {
                                    Alignment::BottomEnd
                                } else if horizontal {
                                    Alignment::CenterEnd
                                } else {
                                    Alignment::BottomStart
                                })
                                .modifier(Modifier::new().size(
                                    Dp(if horizontal || both_axes {
                                        600.0
                                    } else {
                                        200.0
                                    }),
                                    Dp(if !horizontal || both_axes {
                                        600.0
                                    } else {
                                        200.0
                                    }),
                                ))
                                .children(|| {
                                    button()
                                        .accessibility_label("Reveal target")
                                        .on_click(|| ACTIVATIONS.set(ACTIVATIONS.get() + 1));
                                });
                        });
                });
                if horizontal {
                    let bar = scrollbar_h()
                        .state(controller)
                        .track_color(Color::BLACK)
                        .thumb_color(Color::WHITE)
                        .thumb_hover_color(Color::WHITE);
                    if custom {
                        bar.total(Px(1000)).visible(Px(100)).offset(Px(-25));
                    }
                } else {
                    let bar = scrollbar_v()
                        .state(controller)
                        .track_color(Color::BLACK)
                        .thumb_color(Color::WHITE)
                        .thumb_hover_color(Color::WHITE);
                    if custom {
                        bar.total(Px(1000)).visible(Px(100)).offset(Px(-25));
                    }
                }
            });
        },
    );
}

fn control(renderer: &HeadlessRenderer, role: Role, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap()
}

#[tessera]
fn unbound_fixture() {
    material_theme().child(|| {
        unbound_scrollbar_fixture();
    });
}

#[tessera]
fn unbound_scrollbar_fixture() {
    let controller = remember(|| {
        let mut controller = ScrollableController::new();
        controller.set_scroll_position(PxPosition::from_f32_xy(-10.0, -20.0));
        controller
    });
    CONTROLLER.set(Some(controller));
    scrollbar_v()
        .state(controller)
        .total(Px(1000))
        .visible(Px(200));
}

fn renderer(custom: bool, horizontal: bool) -> HeadlessRenderer {
    CASE.set((custom, horizontal));
    ACTIVATIONS.set(0);
    REMAINDER.set((0.0, 0.0));
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 500,
            height: 400,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    renderer
}

fn click(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(3);
}

#[test]
#[ignore = "requires a native GPU adapter for real scrollbar input and accessibility"]
fn controller_metrics_overrides_scaled_track_wheel_drag_and_focus_reveal() {
    crate::test_utils::run_isolated_test(
        "scrollable::scrollbar::bindings_tests::controller_metrics_overrides_scaled_track_wheel_drag_and_focus_reveal",
        || {
            tessera_ui::dp::set_scale_factor(1.0);
            for horizontal in [false, true] {
                let label = if horizontal {
                    "Horizontal scrollbar"
                } else {
                    "Vertical scrollbar"
                };
                {
                    let renderer = renderer(true, horizontal);
                    let (_, bar) = control(&renderer, Role::ScrollBar, label);
                    let bounds = bar.bounds().unwrap();
                    assert_eq!(
                        if horizontal {
                            bounds.width()
                        } else {
                            bounds.height()
                        },
                        100.0
                    );
                    assert!((bar.numeric_value().unwrap() - 0.025).abs() < 0.000001);
                }
                let mut renderer = renderer(false, horizontal);
                let (bar_id, bar) = control(&renderer, Role::ScrollBar, label);
                let bounds = bar.bounds().unwrap();
                assert_eq!(
                    (bounds.width(), bounds.height()),
                    if horizontal {
                        (200.0, 8.0)
                    } else {
                        (8.0, 200.0)
                    }
                );
                let track =
                    PxPosition::from_f32_xy((bounds.x0 + 4.0) as f32, (bounds.y0 + 4.0) as f32);
                let controller = CONTROLLER.get().unwrap();
                assert_eq!(
                    controller.with(|controller| controller.viewport_scale),
                    0.625
                );
                let offset = || {
                    controller.with(|controller| {
                        if horizontal {
                            controller.child_position().x
                        } else {
                            controller.child_position().y
                        }
                    })
                };
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    track,
                    if horizontal { -20.0 } else { 0.0 },
                    if horizontal { 0.0 } else { -20.0 },
                );
                renderer.render_frames(2);
                assert_eq!(
                    offset(),
                    -32.0,
                    "20 window pixels become 32 viewport pixels"
                );
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    PxPosition::from_f32_xy(50.0, 50.0),
                    if horizontal { -10.0 } else { 0.0 },
                    if horizontal { 0.0 } else { -10.0 },
                );
                renderer.render_frames(2);
                assert_eq!(offset(), -48.0);
                let (_, bar) = control(&renderer, Role::ScrollBar, label);
                assert!((bar.numeric_value().unwrap() - 0.08).abs() < 0.000001);
                renderer.inject_pointer(
                    PointerChange::new(
                        current_frame_time().unwrap(),
                        MOUSE_POINTER_ID,
                        CursorEventContent::Scroll(ScrollEventContent {
                            delta_x: if horizontal { -1.0 } else { 0.0 },
                            delta_y: if horizontal { 0.0 } else { -1.0 },
                            unit: ScrollDeltaUnit::Line,
                            source: ScrollEventSource::Wheel,
                        }),
                        GestureState::TapCandidate,
                    )
                    .with_position(track),
                );
                renderer.render_frames(120);
                assert_eq!(
                    offset(),
                    -112.0,
                    "One native wheel line maps to 64 viewport pixels exactly once"
                );
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    track,
                    if horizontal { 3.125 } else { 0.0 },
                    if horizontal { 0.0 } else { 3.125 },
                );
                renderer.render_frames(2);
                assert_eq!(
                    offset(),
                    -107.0,
                    "Positive touchpad deltas reverse direction without smoothing"
                );
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    track,
                    if horizontal { -0.3125 } else { 0.0 },
                    if horizontal { 0.0 } else { -0.3125 },
                );
                renderer.render_frames(2);
                assert_eq!(
                    offset(),
                    -107.5,
                    "Fractional touchpad motion survives scale conversion"
                );
                let before = renderer.read_pixels().unwrap();
                let thumb_pixel = |pixels: &[u8], along: usize| {
                    let x = bounds.x0 as usize + if horizontal { along } else { 4 };
                    let y = bounds.y0 as usize + if horizontal { 4 } else { along };
                    pixels[(y * 500 + x) * 4]
                };
                assert_eq!(thumb_pixel(&before, 70), 255);
                assert_eq!(thumb_pixel(&before, 166), 0);
                let bottom = PxPosition::from_f32_xy(
                    (bounds.x0 + if horizontal { 190.0 } else { 4.0 }) as f32,
                    (bounds.y0 + if horizontal { 4.0 } else { 190.0 }) as f32,
                );
                click(&mut renderer, bottom);
                renderer.render_frames(120);
                assert!(
                    offset() < -300.0,
                    "Real track tap moves content toward its end"
                );
                let after = renderer.read_pixels().unwrap();
                assert_eq!(thumb_pixel(&after, 70), 0);
                assert_eq!(thumb_pixel(&after, 166), 255);
                assert!(renderer.dispatch_accessibility_action(ActionRequest {
                    action: Action::Decrement,
                    target_tree: TreeId::ROOT,
                    target_node: bar_id,
                    data: None
                }));
                renderer.render_frames(120);
                assert!(offset() > -400.0);
                let after_decrement = offset();
                assert!(renderer.dispatch_accessibility_action(ActionRequest {
                    action: Action::Increment,
                    target_tree: TreeId::ROOT,
                    target_node: bar_id,
                    data: None,
                }));
                renderer.render_frames(120);
                assert!(offset() < after_decrement);
                let before_drag = offset();
                let thumb_center = before_drag.abs() / 600.0 * 200.0 + 100.0 / 3.0;
                let start = PxPosition::from_f32_xy(
                    (bounds.x0 + if horizontal { thumb_center as f64 } else { 4.0 }) as f32,
                    (bounds.y0 + if horizontal { 4.0 } else { thumb_center as f64 }) as f32,
                );
                let end = start
                    + PxPosition::from_f32_xy(
                        if horizontal { -35.0 } else { 0.0 },
                        if horizontal { 0.0 } else { -35.0 },
                    );
                renderer.pointer_press(MOUSE_POINTER_ID, start, PressKeyEventType::Left);
                renderer.render_frames(1);
                renderer.pointer_move(MOUSE_POINTER_ID, end);
                renderer.render_frames(1);
                renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
                renderer.render_frames(120);
                assert!(
                    offset() > before_drag,
                    "Dragging the real thumb backward moves content"
                );
                let (target, _) = control(&renderer, Role::Button, "Reveal target");
                assert!(renderer.dispatch_accessibility_action(ActionRequest {
                    action: Action::Focus,
                    target_tree: TreeId::ROOT,
                    target_node: target,
                    data: None
                }));
                renderer.render_frames(8);
                assert_eq!(ACTIVATIONS.get(), 0);
                let (_, target) = control(&renderer, Role::Button, "Reveal target");
                let target = target.bounds().unwrap();
                assert!(
                    target.x0 >= 0.0
                        && target.y0 >= 0.0
                        && target.x1 <= 125.0
                        && target.y1 <= 125.0
                );
                click(
                    &mut renderer,
                    PxPosition::from_f32_xy(
                        ((target.x0 + target.x1) * 0.5) as f32,
                        ((target.y0 + target.y1) * 0.5) as f32,
                    ),
                );
                assert_eq!(ACTIVATIONS.get(), 1);
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    track,
                    if horizontal { -1000.0 } else { 0.0 },
                    if horizontal { 0.0 } else { -1000.0 },
                );
                renderer.render_frames(120);
                assert_eq!(offset(), -400.0);
                REMAINDER.set((0.0, 0.0));
                renderer.pointer_scroll(
                    MOUSE_POINTER_ID,
                    track,
                    if horizontal { -10.0 } else { 0.0 },
                    if horizontal { 0.0 } else { -10.0 },
                );
                renderer.render_frames(1);
                assert_eq!(
                    REMAINDER.get(),
                    if horizontal {
                        (-10.0, 0.0)
                    } else {
                        (0.0, -10.0)
                    },
                    "Boundary remainder returns in parent window units exactly once"
                );
                assert_eq!(offset(), -400.0);
            }
            BOTH_AXES.set(true);
            let mut renderer = renderer(false, false);
            let controller = CONTROLLER.get().unwrap();
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(50.0, 50.0),
                -10.0,
                -10.0,
            );
            renderer.render_frames(2);
            assert_eq!(
                controller.with(|controller| controller.child_position()),
                PxPosition::from_f32_xy(-16.0, -16.0)
            );
            let (_, bar) = control(&renderer, Role::ScrollBar, "Vertical scrollbar");
            let bounds = bar.bounds().unwrap();
            let track = PxPosition::from_f32_xy((bounds.x0 + 4.0) as f32, (bounds.y0 + 4.0) as f32);
            renderer.pointer_scroll(MOUSE_POINTER_ID, track, 0.0, -10.0);
            renderer.render_frames(2);
            assert_eq!(
                controller.with(|controller| controller.child_position()),
                PxPosition::from_f32_xy(-16.0, -32.0),
                "Vertical track wheel must preserve the both-axis viewport's horizontal offset"
            );
            BOTH_AXES.set(false);
            drop(renderer);
            let mut renderer = HeadlessRenderer::new(
                Box::new(unbound_fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 200,
                    height: 240,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(4.0, 100.0),
                0.0,
                -10.0,
            );
            renderer.render_frames(2);
            assert_eq!(
                CONTROLLER
                    .get()
                    .unwrap()
                    .with(|controller| controller.child_position()),
                PxPosition::from_f32_xy(-10.0, -20.0),
                "A custom track without an attached viewport must leave wheel input to its ancestors"
            );
        },
    );
}
