//! Switch motion wiring and frame-clock regressions.
//!
//! ## Usage
//!
//! Preserve interrupted thumb motion and stop work after controls settle.

use std::time::Duration;

use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition, State,
    accesskit::Action,
    remember, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{column::column, icon::icon, material_icons::filled, theme::material_theme};

use super::{THUMB_TEST_TAG, switch};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: bool,
    icons: bool,
    activations: usize,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
}

#[tessera]
fn fixture(probe: TestProbe<FixtureHandles>) {
    let settings = remember(Settings::default);
    probe.set(FixtureHandles {
        settings: Some(settings),
    });
    material_theme().child(move || {
        column().children(move || {
            switch(settings.get().checked)
                .accessibility_label("Animated switch")
                .on_toggle(move |checked| {
                    settings.with_mut(|value| {
                        value.checked = checked;
                        value.activations += 1;
                    })
                })
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

fn thumb(renderer: &UiTest) -> [i64; 4] {
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

fn check_motion() {
    let probe = TestProbe::default();
    let observed = probe.clone();
    let mut renderer = UiTest::new()
        .viewport(200, 140)
        .scale_factor(2.0)
        .frame_step(Duration::from_millis(10))
        .rendered(crate::TesseraComponents)
        .content(move || fixture(probe.clone()));
    renderer.frames(30);
    let settings = observed.get().settings.unwrap();
    let initial = thumb(&renderer);
    assert_eq!((initial[0], initial[2], initial[3]), (16, 32, 32));
    let initial_pixels = renderer.read_pixels();
    settings.with_mut(|value| value.checked = true);
    renderer.frames(1);
    assert_eq!(
        thumb(&renderer),
        initial,
        "selection starts from the rendered geometry"
    );
    renderer.frames(6);
    let halfway = thumb(&renderer);
    assert_eq!((halfway[0], halfway[2]), (32, 40));
    assert_ne!(renderer.read_pixels(), initial_pixels);
    settings.with_mut(|value| value.checked = false);
    renderer.frames(1);
    assert!(
        thumb(&renderer)[0] >= halfway[0],
        "reversal retains incoming movement"
    );
    renderer.frames(100);
    assert_eq!(thumb(&renderer), initial);
    assert_eq!(renderer.read_pixels(), initial_pixels);
    assert_eq!(settings.get().activations, 0);

    let point = PxPosition::from_f32_xy(
        (initial[0] + initial[2] / 2) as f32,
        (initial[1] + initial[3] / 2) as f32,
    );
    let outside = PxPosition::from_f32_xy(190.0, 130.0);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.frames(2);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (4, 56));
    renderer.pointer_move(MOUSE_POINTER_ID, outside);
    renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    renderer.frames(1);
    assert_eq!(
        (thumb(&renderer)[0], thumb(&renderer)[2]),
        (4, 56),
        "release starts from the pressed geometry without advancing the clock twice"
    );
    renderer.frames(6);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (10, 43));
    renderer.frames(100);
    assert_eq!(settings.get().activations, 0);
    assert_eq!(thumb(&renderer), initial);

    settings.with_mut(|value| value.icons = true);
    renderer.frames(1);
    renderer.frames(6);
    assert_eq!(
        thumb(&renderer)[2],
        40,
        "content changes animate thumb size"
    );
    renderer.frames(100);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (8, 48));
    settings.with_mut(|value| {
        value.icons = false;
        value.checked = false;
    });
    renderer.frames(100);
    let target = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Animated switch"))
        .unwrap()
        .0;
    assert!(renderer.accessibility_action(target, Action::Click));
    renderer.frames(1);
    renderer.frames(15);
    assert!(thumb(&renderer)[0] > 48, "spatial overshoot reaches layout");
    assert!(thumb(&renderer)[2] > 48, "size overshoot reaches layout");
    renderer.frames(100);
    assert_eq!((thumb(&renderer)[0], thumb(&renderer)[2]), (48, 48));
    assert_eq!(settings.get().activations, 1);
    for _ in 0..100 {
        renderer.frames(1);
        let clock = renderer.frame_clock_diagnostics();
        assert_eq!(
            clock.resumed_tasks, 0,
            "settled switch frame receivers stop"
        );
        assert_eq!(clock.waiting_tasks, 0);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn switch_spatial_motion_press_release_and_idle() {
    crate::test_utils::run_isolated_test(
        "switch::motion_tests::switch_spatial_motion_press_release_and_idle",
        check_motion,
    );
}
