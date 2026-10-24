use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, Rect, Role, TreeId},
    remember, tessera,
};

use crate::{modifier::ModifierExt as _, spacer::spacer, surface::surface, theme::material_theme};

use super::{BottomSheetController, BottomSheetExpansion, bottom_sheet_provider};

thread_local! {
    static CONTROLLER: Cell<Option<State<BottomSheetController>>> = const { Cell::new(None) };
    static PARTIAL: Cell<Option<Dp>> = const { Cell::new(Some(Dp(240.0))) };
    static CLOSES: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let controller = remember(|| BottomSheetController::new(true));
    CONTROLLER.set(Some(controller));
    material_theme().child(move || {
        bottom_sheet_provider()
            .controller(controller)
            .partial_height_optional(PARTIAL.get())
            .accessibility_label("Resizable sheet")
            .on_close_request(move || {
                CLOSES.set(CLOSES.get() + 1);
                controller.with_mut(BottomSheetController::close);
            })
            .main_content(|| {
                surface()
                    .modifier(Modifier::new().fill_max_size())
                    .child(|| {});
            })
            .bottom_sheet_content(|| {
                spacer().modifier(Modifier::new().height(Dp(120.0)));
            });
    });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() != Role::Label)
        .unwrap_or_else(|| panic!("missing {label}"))
        .1
        .bounds()
        .unwrap()
}

fn tap(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn check_handle() {
    PARTIAL.set(Some(Dp(240.0)));
    CLOSES.set(0);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 400,
            height: 640,
            sample_count: 1,
            frame_time: Duration::from_millis(20),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let controller = CONTROLLER.get().unwrap();
    let tree = renderer.accessibility_snapshot().unwrap();
    let (handle_id, handle) = tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Drag handle"))
        .unwrap();
    assert_eq!(handle.role(), Role::Button);
    assert!(handle.supports_action(Action::Click));
    assert_eq!(handle.bounds().unwrap().height(), 48.0);
    assert_eq!(handle.bounds().unwrap().width(), 400.0);
    assert_eq!(bounds(&renderer, "Resizable sheet").height(), 240.0);
    assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
    renderer.render_frames(4);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, *handle_id);
    let handle = bounds(&renderer, "Drag handle");
    let point = PxPosition::from_f32_xy(
        ((handle.x0 + handle.x1) / 2.0) as f32,
        ((handle.y0 + handle.y1) / 2.0) as f32,
    );
    tap(&mut renderer, point);
    assert_eq!(
        controller.with(BottomSheetController::expansion),
        BottomSheetExpansion::Expanded
    );
    assert_eq!(bounds(&renderer, "Resizable sheet").height(), 568.0);
    assert!(controller.with(BottomSheetController::is_open));
    assert_eq!(CLOSES.get(), 0);
    let tree = renderer.accessibility_snapshot().unwrap();
    let handle_id = tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Drag handle"))
        .unwrap()
        .0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: handle_id,
        data: None,
    }));
    renderer.render_frames(4);
    assert_eq!(
        controller.with(BottomSheetController::expansion),
        BottomSheetExpansion::PartiallyExpanded
    );
    assert_eq!(bounds(&renderer, "Resizable sheet").height(), 240.0);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, handle_id);
    let handle = bounds(&renderer, "Drag handle");
    let point = PxPosition::from_f32_xy(
        ((handle.x0 + handle.x1) / 2.0) as f32,
        ((handle.y0 + handle.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    let moved = PxPosition::from_f32_xy(point.x, point.y + 20.0);
    renderer.pointer_move(MOUSE_POINTER_ID, moved);
    renderer.render_frames(2);
    assert!(bounds(&renderer, "Drag handle").y0 > handle.y0 + 5.0);
    // Hold past the velocity tracker's idle cutoff so release is not a fling.
    renderer.render_frames(6);
    renderer.pointer_release(MOUSE_POINTER_ID, moved, PressKeyEventType::Left);
    renderer.render_frames(40);
    assert_eq!(
        controller.with(BottomSheetController::expansion),
        BottomSheetExpansion::PartiallyExpanded
    );
    assert_eq!(bounds(&renderer, "Resizable sheet").height(), 240.0);
    assert_eq!(
        CLOSES.get(),
        0,
        "a short drag must settle without toggling or dismissing"
    );
    tap(&mut renderer, PxPosition::from_f32_xy(10.0, 10.0));
    assert_eq!(CLOSES.get(), 1);
    assert!(!controller.with(BottomSheetController::is_open));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_bottom_sheet_handle_cycles_real_heights_without_activating_after_a_drag() {
    crate::test_utils::run_isolated_test(
        "bottom_sheet::accessibility_tests::gpu_bottom_sheet_handle_cycles_real_heights_without_activating_after_a_drag",
        check_handle,
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_bottom_sheet_natural_height_and_small_viewport_remain_bounded() {
    crate::test_utils::run_isolated_test(
        "bottom_sheet::accessibility_tests::gpu_bottom_sheet_natural_height_and_small_viewport_remain_bounded",
        || {
            PARTIAL.set(None);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 400,
                    height: 640,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(8);
            assert_eq!(bounds(&renderer, "Resizable sheet").height(), 168.0);
            let controller = CONTROLLER.get().unwrap();
            controller.with_mut(BottomSheetController::expand);
            renderer.render_frames(4);
            assert_eq!(bounds(&renderer, "Resizable sheet").height(), 568.0);
            controller.with_mut(BottomSheetController::partially_expand);
            renderer.render_frames(4);
            assert_eq!(bounds(&renderer, "Resizable sheet").height(), 168.0);
            renderer.resize(400, 100);
            renderer.render_frames(8);
            assert_eq!(bounds(&renderer, "Resizable sheet").height(), 28.0);
            let tree = renderer.accessibility_snapshot().unwrap();
            let handle = tree
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Drag handle"))
                .unwrap()
                .1
                .clone();
            assert!(
                handle.is_disabled(),
                "one available height cannot offer a false resize action"
            );
            assert!(!handle.supports_action(Action::Click));
            assert!(controller.with(BottomSheetController::is_open));
            controller.with_mut(BottomSheetController::close);
            renderer.render_frames(40);
            controller.with_mut(BottomSheetController::expand);
            assert!(!controller.with(BottomSheetController::is_open));
            controller.with_mut(BottomSheetController::partially_expand);
            assert!(!controller.with(BottomSheetController::is_open));
        },
    );
}
