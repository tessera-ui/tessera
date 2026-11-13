//! Checkbox animation wiring and idle-clock regressions.
//!
//! ## Usage
//!
//! Preserve timed state transitions, direct updates, and disabled appearance.

use std::time::Duration;

use tessera_ui::{
    Color, State,
    accesskit::{Action, Toggled},
    remember, tessera,
    testing::UiTest,
};

use crate::{
    column::column,
    md3::theme::{MaterialTheme, material_theme},
};

use super::checkbox;

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    checked: bool,
    mixed: bool,
    disabled: bool,
    activations: usize,
}

#[derive(Clone, Default)]
struct FixtureHandles {
    settings: Option<State<Settings>>,
}

#[tessera]
fn fixture(probe: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let settings = remember(Settings::default);
    probe.set(FixtureHandles {
        settings: Some(settings),
    });
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
                checkbox(value.checked)
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

fn selection(renderer: &UiTest) -> (tessera_ui::accesskit::NodeId, tessera_ui::accesskit::Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Animated checkbox"))
        .unwrap()
}

fn fill(renderer: &UiTest) -> [u8; 4] {
    let bounds = selection(renderer).1.bounds().unwrap();
    let x = ((bounds.x0 + bounds.x1) / 2.0) as u32 + 8;
    let y = ((bounds.y0 + bounds.y1) / 2.0) as u32 + 8;
    let offset = ((y * renderer.physical_size().width + x) * 4) as usize;
    renderer.read_pixels()[offset..offset + 4]
        .try_into()
        .unwrap()
}

fn check_motion() {
    let probe = tessera_ui::testing::TestProbe::default();
    let mut renderer = UiTest::new()
        .viewport(160, 160)
        .scale_factor(2.0)
        .frame_step(Duration::from_millis(10))
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    renderer.frames(30);
    let settings = probe.get().settings.unwrap();
    settings.with_mut(|value| value.checked = true);
    renderer.frames(1);
    assert_eq!(selection(&renderer).1.toggled(), Some(Toggled::True));
    renderer.frames(6);
    assert!(renderer.frame_clock_diagnostics().resumed_tasks > 0);
    let alpha = fill(&renderer)[3];
    assert!((145..=158).contains(&alpha), "50ms fill alpha: {alpha}");
    renderer.frames(5);
    assert!((225..=240).contains(&fill(&renderer)[3]));
    renderer.frames(100);
    assert_eq!(fill(&renderer), [255, 0, 0, 255]);
    renderer.frames(3);
    for _ in 0..30 {
        renderer.frames(1);
        let clock = renderer.frame_clock_diagnostics();
        assert_eq!(
            clock.resumed_tasks, 0,
            "settled controls must stop their frame receiver"
        );
        assert_eq!(clock.waiting_tasks, 0);
    }

    assert!(renderer.accessibility_action(selection(&renderer).0, Action::Click));
    renderer.frames(8);
    assert_eq!(settings.get().activations, 1);
    assert_eq!(selection(&renderer).1.toggled(), Some(Toggled::False));
    assert!(
        fill(&renderer)[3] < 100,
        "removal uses the faster color effect"
    );
    renderer.frames(4);
    renderer.frames(30);
    settings.with_mut(|value| value.mixed = true);
    renderer.frames(1);
    renderer.frames(40);
    let dash_pixels = renderer.read_pixels();
    settings.with_mut(|value| {
        value.mixed = false;
        value.checked = true;
    });
    renderer.frames(7);
    assert_ne!(
        renderer.read_pixels(),
        dash_pixels,
        "the morph must reach the draw pipeline"
    );
    settings.with_mut(|value| {
        value.mixed = true;
        value.checked = false;
    });
    renderer.frames(1);
    renderer.frames(40);
    assert_eq!(
        renderer.read_pixels(),
        dash_pixels,
        "reversing the morph must restore the dash pixels"
    );
    settings.with_mut(|value| value.disabled = true);
    renderer.frames(1);
    let disabled = fill(&renderer);
    assert!(disabled[0] < 5 && disabled[1] < 5 && disabled[2] > 150);
    assert!(
        (95..=99).contains(&disabled[3]),
        "disabled fill snaps: {disabled:?}"
    );
    assert!(!renderer.accessibility_action(selection(&renderer).0, Action::Click));
    assert_eq!(settings.get().activations, 1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn checkbox_spatial_and_color_motion_retargeting_and_idle() {
    crate::test_utils::run_isolated_test(
        "md3::checkbox::motion_tests::checkbox_spatial_and_color_motion_retargeting_and_idle",
        check_motion,
    );
}
