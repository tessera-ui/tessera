//! Checkbox animation wiring and idle-clock regressions.
//!
//! ## Usage
//!
//! Preserve timed state transitions, direct updates, and disabled appearance.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    Color, HeadlessConfig, HeadlessRenderer, State,
    accesskit::{Action, ActionRequest, Toggled, TreeId},
    remember, tessera,
};

use crate::{
    column::column,
    theme::{MaterialTheme, material_theme},
};

use super::{CheckboxController, checkbox};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: bool,
    mixed: bool,
    disabled: bool,
    activations: usize,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CONTROLLER: Cell<Option<State<CheckboxController>>> = const { Cell::new(None) };
    pub(super) static TICKS: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let controller = remember(|| CheckboxController::new(false));
    SETTINGS.set(Some(settings));
    CONTROLLER.set(Some(controller));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::RED;
            theme.color_scheme.on_primary = Color::WHITE;
            theme.color_scheme.on_surface = Color::BLUE;
            theme.color_scheme.surface = Color::WHITE;
            theme
        })
        .child(move || {
            column().children(move || {
                let value = settings.get();
                checkbox()
                    .controller(controller)
                    .checked(value.checked)
                    .indeterminate(value.mixed)
                    .disabled(value.disabled)
                    .accessibility_label("Animated checkbox")
                    .on_toggle(move |value| {
                        settings.with_mut(|settings| {
                            settings.checked = value;
                            settings.mixed = false;
                            settings.activations += 1;
                        })
                    });
            });
        });
}

fn selection(
    renderer: &HeadlessRenderer,
) -> (tessera_ui::accesskit::NodeId, tessera_ui::accesskit::Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Animated checkbox"))
        .unwrap()
}

fn fill(renderer: &HeadlessRenderer) -> [u8; 4] {
    let bounds = selection(renderer).1.bounds().unwrap();
    let x = ((bounds.x0 + bounds.x1) / 2.0) as u32 + 8;
    let y = ((bounds.y0 + bounds.y1) / 2.0) as u32 + 8;
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    renderer.read_pixels().unwrap()[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn check_motion(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 160,
            height: 160,
            scale_factor: 2.0,
            sample_count: samples,
            frame_time: Duration::from_millis(10),
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(30);
    let settings = SETTINGS.get().unwrap();
    let controller = CONTROLLER.get().unwrap();
    settings.with_mut(|value| value.checked = true);
    renderer.step_frame();
    assert_eq!(controller.with(CheckboxController::progress), 0.0);
    assert_eq!(selection(&renderer).1.toggled(), Some(Toggled::True));
    renderer.render_frames(5);
    assert!((controller.with(CheckboxController::progress) - 0.28017744).abs() < 0.00001);
    let alpha = fill(&renderer)[3];
    assert!((145..=158).contains(&alpha), "50ms fill alpha: {alpha}");
    renderer.render_frames(5);
    assert!((controller.with(CheckboxController::progress) - 0.65983105).abs() < 0.00001);
    assert!((225..=240).contains(&fill(&renderer)[3]));
    controller.with_mut(|controller| controller.set_checked(true));
    renderer.step_frame();
    assert_eq!(controller.with(CheckboxController::progress), 1.0);
    assert_eq!(fill(&renderer), [255, 0, 0, 255]);
    renderer.render_frames(3);
    let ticks = TICKS.get();
    renderer.render_frames(30);
    assert_eq!(
        TICKS.get(),
        ticks,
        "settled controls must stop their frame receiver"
    );

    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: selection(&renderer).0,
        data: None,
    }));
    renderer.render_frames(8);
    assert_eq!(settings.get().activations, 1);
    assert_eq!(controller.with(CheckboxController::progress), 1.0);
    assert_eq!(selection(&renderer).1.toggled(), Some(Toggled::False));
    assert!(
        fill(&renderer)[3] < 100,
        "removal uses the faster color effect"
    );
    renderer.render_frames(2);
    assert_eq!(controller.with(CheckboxController::progress), 0.0);
    renderer.render_frames(30);
    settings.with_mut(|value| value.mixed = true);
    renderer.step_frame();
    assert_eq!(
        controller.with(|controller| controller.checkmark.morph.value),
        1.0,
        "partial selection enters as a dash"
    );
    renderer.render_frames(40);
    let dash_pixels = renderer.read_pixels().unwrap();
    settings.with_mut(|value| {
        value.mixed = false;
        value.checked = true;
    });
    renderer.step_frame();
    renderer.render_frames(5);
    let morph = controller.with(|controller| controller.checkmark.morph.value);
    assert!((morph - 0.71982256).abs() < 0.00001);
    assert_ne!(
        renderer.read_pixels().unwrap(),
        dash_pixels,
        "the morph must reach the draw pipeline"
    );
    settings.with_mut(|value| {
        value.mixed = true;
        value.checked = false;
    });
    renderer.step_frame();
    assert!(controller.with(|controller| controller.checkmark.morph.value) < morph);
    assert!(controller.with(|controller| controller.checkmark.morph.velocity) < 0.0);
    renderer.render_frames(40);
    assert_eq!(
        controller.with(|controller| controller.checkmark.morph.value),
        1.0
    );
    assert_eq!(
        renderer.read_pixels().unwrap(),
        dash_pixels,
        "reversing the morph must restore the dash pixels"
    );
    settings.with_mut(|value| value.disabled = true);
    renderer.step_frame();
    let disabled = fill(&renderer);
    assert!(disabled[0] < 5 && disabled[1] < 5 && disabled[2] > 150);
    assert!(
        (95..=99).contains(&disabled[3]),
        "disabled fill snaps: {disabled:?}"
    );
    assert!(!renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: selection(&renderer).0,
        data: None,
    }));
    assert_eq!(settings.get().activations, 1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_spatial_and_color_motion_retargeting_and_idle_msaa1() {
    crate::test_utils::run_isolated_test(
        "checkbox::motion_tests::checkbox_spatial_and_color_motion_retargeting_and_idle_msaa1",
        || check_motion(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_spatial_and_color_motion_retargeting_and_idle_msaa4() {
    crate::test_utils::run_isolated_test(
        "checkbox::motion_tests::checkbox_spatial_and_color_motion_retargeting_and_idle_msaa4",
        || check_motion(4),
    );
}
