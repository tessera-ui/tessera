//! Switch motion wiring and frame-clock regressions.
//!
//! ## Usage
//!
//! Preserve interrupted thumb motion and stop work after controls settle.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, TreeId},
    remember, tessera,
};

use crate::{column::column, icon::icon, material_icons::filled, theme::material_theme};

use super::{SwitchController, THUMB_TEST_TAG, switch};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: Option<bool>,
    icons: bool,
    activations: usize,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CONTROLLER: Cell<Option<State<SwitchController>>> = const { Cell::new(None) };
    pub(super) static TICKS: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let controller = remember(|| SwitchController::new(false));
    SETTINGS.set(Some(settings));
    CONTROLLER.set(Some(controller));
    material_theme().child(move || {
        column().children(move || {
            switch()
                .controller(controller)
                .checked_optional(settings.get().checked)
                .accessibility_label("Animated switch")
                .on_toggle(move |_| settings.with_mut(|value| value.activations += 1))
                .child_optional(if settings.get().icons {
                    Some(tessera_ui::RenderSlot::new(|| {
                        icon().painter(filled::CHECK_SVG);
                    }))
                } else {
                    None
                });
        });
    });
}

fn thumb(renderer: &HeadlessRenderer) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(value) = pending.pop() {
        if value["accessibility"]["key"] == THUMB_TEST_TAG {
            let bounds = &value["bounds"];
            return ["x", "y", "width", "height"].map(|key| bounds[key].as_i64().unwrap());
        }
        if let Some(children) = value["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("placed thumb not found");
}

fn check_motion(samples: u32) {
    TICKS.set(0);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 200,
            height: 140,
            scale_factor: 2.0,
            sample_count: samples,
            frame_time: Duration::from_millis(10),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(30);
    let settings = SETTINGS.get().unwrap();
    let controller = CONTROLLER.get().unwrap();
    let initial = thumb(&renderer);
    assert_eq!((initial[0], initial[2], initial[3]), (16, 32, 32));
    let initial_pixels = renderer.read_pixels().unwrap();
    settings.with_mut(|value| value.checked = Some(true));
    renderer.step_frame();
    assert_eq!(
        thumb(&renderer),
        initial,
        "selection starts from the rendered geometry"
    );
    renderer.render_frames(5);
    let halfway = thumb(&renderer);
    assert_eq!((halfway[0], halfway[2]), (32, 40));
    assert_ne!(renderer.read_pixels().unwrap(), initial_pixels);
    settings.with_mut(|value| value.checked = Some(false));
    renderer.step_frame();
    assert!(
        thumb(&renderer)[0] >= halfway[0],
        "reversal retains incoming movement"
    );
    renderer.render_frames(100);
    assert_eq!(thumb(&renderer), initial);
    assert_eq!(renderer.read_pixels().unwrap(), initial_pixels);
    assert_eq!(settings.get().activations, 0);

    let point = PxPosition::from_f32_xy(
        (initial[0] + initial[2] / 2) as f32,
        (initial[1] + initial[3] / 2) as f32,
    );
    let outside = PxPosition::from_f32_xy(190.0, 130.0);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(2);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (4, 56));
    renderer.pointer_move(MOUSE_POINTER_ID, outside);
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.render_frames(2);
    renderer.render_frames(5);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (10, 43));
    renderer.render_frames(100);
    assert_eq!(settings.get().activations, 0);
    assert_eq!(thumb(&renderer), initial);

    settings.with_mut(|value| value.icons = true);
    renderer.step_frame();
    renderer.render_frames(5);
    assert_eq!(
        thumb(&renderer)[2],
        40,
        "content changes animate thumb size"
    );
    controller.with_mut(|controller| controller.set_checked(false));
    renderer.step_frame();
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (8, 48));
    settings.with_mut(|value| {
        value.icons = false;
        value.checked = None;
    });
    renderer.render_frames(100);
    let target = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Animated switch"))
        .unwrap()
        .0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: target,
        data: None
    }));
    renderer.step_frame();
    renderer.render_frames(15);
    assert!(thumb(&renderer)[0] > 48, "spatial overshoot reaches layout");
    assert!(thumb(&renderer)[2] > 48, "size overshoot reaches layout");
    renderer.render_frames(100);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (48, 48));
    assert_eq!(settings.get().activations, 1);
    assert!(!controller.with(SwitchController::is_animating));
    let ticks = TICKS.get();
    renderer.render_frames(100);
    assert_eq!(TICKS.get(), ticks, "settled switch frame receivers stop");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn switch_spatial_motion_press_release_and_idle_msaa1() {
    crate::test_utils::run_isolated_test(
        "switch::motion_tests::switch_spatial_motion_press_release_and_idle_msaa1",
        || check_motion(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn switch_spatial_motion_press_release_and_idle_msaa4() {
    crate::test_utils::run_isolated_test(
        "switch::motion_tests::switch_spatial_motion_press_release_and_idle_msaa4",
        || check_motion(4),
    );
}
