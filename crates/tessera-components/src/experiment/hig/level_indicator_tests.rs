use std::{ops::RangeInclusive, time::Duration};

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, Modifier, PlacementModifierNode, PxPosition, State,
    accesskit::{Action, Role},
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest, WorkPhase},
};

use crate::{alignment::Alignment, boxed::boxed, modifier::ModifierExt as _};

use super::{
    super::{
        button::button,
        theme::{Appearance, HigTheme, hig_theme},
    },
    capacity_indicator,
};

#[derive(Clone, Copy)]
struct Controls {
    value: State<f64>,
    range: State<RangeInclusive<f64>>,
    warning: State<Option<f64>>,
    critical: State<Option<f64>>,
    size: State<(Dp, Dp)>,
    shift: State<Dp>,
    enabled: State<bool>,
    dark: State<bool>,
}

#[derive(Clone, Copy, PartialEq)]
struct Offset(Dp);

impl PlacementModifierNode for Offset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        let shift = self.0.to_pixels_f32();
        position + PxPosition::from_f32_xy(shift, shift)
    }
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| 0.25);
    let range = remember(|| 0.0..=1.0);
    let warning = remember(|| None);
    let critical = remember(|| None);
    let size = remember(|| (Dp(180.0), Dp(20.0)));
    let shift = remember(|| Dp::ZERO);
    let enabled = remember(|| true);
    let dark = remember(|| false);
    probe.set(Some(Controls {
        value,
        range,
        warning,
        critical,
        size,
        shift,
        enabled,
        dark,
    }));
    hig_theme()
        .theme(HigTheme {
            appearance: if dark.get() {
                Appearance::Dark
            } else {
                Appearance::Light
            },
        })
        .child(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().background(if dark.get() {
                    Color::from_rgb(0.010022826, 0.010022826, 0.010022826)
                } else {
                    Color::from_rgb(0.8900054, 0.8900054, 0.8900054)
                }))
                .children(move || {
                    let (width, height) = size.get();
                    let indicator = capacity_indicator(value.get())
                        .range(range.get())
                        .enabled(enabled.get())
                        .accessibility_label("Storage")
                        .accessibility_description("Used disk space")
                        .modifier(
                            Modifier::new()
                                .push_placement(Offset(shift.get()))
                                .size(width, height),
                        );
                    if let Some(warning) = warning.get() {
                        indicator.warning_value(warning);
                    }
                    if let Some(critical) = critical.get() {
                        indicator.critical_value(critical);
                    }
                });
        });
}

#[test]
fn caller_values_are_clamped_for_read_only_meter_semantics_without_edit_actions() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 100)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Storage");
    assert_eq!(node.role(), Role::Meter);
    assert_eq!(node.numeric_value(), Some(0.25));
    assert_eq!(node.description(), Some("Used disk space"));
    for action in [
        Action::Focus,
        Action::Click,
        Action::Increment,
        Action::Decrement,
        Action::SetValue,
    ] {
        assert!(!node.supports_action(action));
        assert!(!ui.accessibility_action(id, action));
    }
    ui.click_at(PxPosition::from_f32_xy(150.0, 10.0));
    assert_eq!(controls.value.get(), 0.25);
    controls.value.set(2.0);
    ui.frame();
    assert_eq!(
        ui.accessibility_node("Storage").1.numeric_value(),
        Some(1.0)
    );
    assert_eq!(controls.value.get(), 2.0);
    controls.range.set(20.0..=80.0);
    ui.frame();
    let (_, node) = ui.accessibility_node("Storage");
    assert_eq!(
        (
            node.numeric_value(),
            node.min_numeric_value(),
            node.max_numeric_value()
        ),
        (Some(20.0), Some(20.0), Some(80.0))
    );
    controls.value.set(50.0);
    controls.enabled.set(false);
    ui.frame();
    let (current, node) = ui.accessibility_node("Storage");
    assert_eq!(current, id);
    assert!(node.is_disabled());
    assert_eq!(node.numeric_value(), Some(50.0));
    let mut defaults = UiTest::new().content(|| {
        capacity_indicator(2.5).accessibility_label("Default capacity");
    });
    defaults.frame();
    let (_, node) = defaults.accessibility_node("Default capacity");
    assert_eq!(
        (
            node.numeric_value(),
            node.min_numeric_value(),
            node.max_numeric_value()
        ),
        (Some(2.5), Some(0.0), Some(5.0))
    );
}

#[tessera]
fn input_fixture(probe: TestProbe<Option<State<u32>>>) {
    let clicks = remember(|| 0);
    probe.set(Some(clicks));
    boxed().alignment(Alignment::TopStart).children(move || {
        button("Continue")
            .accessibility_label("Continue action")
            .modifier(Modifier::new().size(Dp(180.0), Dp(24.0)))
            .on_click(move || clicks.set(clicks.get() + 1));
        capacity_indicator(0.5)
            .accessibility_label("Capacity")
            .modifier(Modifier::new().size(Dp(180.0), Dp(24.0)));
    });
}

#[test]
fn read_only_capacity_does_not_consume_pointer_input_above_an_action() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(240, 80)
        .content(|| input_fixture(probe.clone()));
    ui.frame();
    ui.click_at(PxPosition::from_f32_xy(60.0, 12.0));
    assert_eq!(probe.get().unwrap().get(), 1);
    assert_eq!(
        ui.accessibility_node("Continue action").1.role(),
        Role::Button
    );
}

fn check_scaled_geometry() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(600, 200)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let bounds = ui.accessibility_node("Storage").1.bounds().unwrap();
    assert_eq!(
        (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
        (337.5, 37.5)
    );
    controls.size.set((Dp::ZERO, Dp::ZERO));
    ui.frame();
    let bounds = ui.accessibility_node("Storage").1.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (0.0, 0.0));
    controls.size.set((Dp(3.0), Dp(3.0)));
    ui.frame();
    controls.size.set((Dp(180.0), Dp(20.0)));
    ui.frame();
    let bounds = ui.accessibility_node("Storage").1.bounds().unwrap();
    assert_eq!(
        (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
        (337.5, 37.5)
    );
    controls.value.set(0.9);
    controls.dark.set(true);
    ui.frame();
    assert_eq!(ui.work().phase(WorkPhase::Measure).count(), 0);
    let mut natural = UiTest::new()
        .viewport(600, 200)
        .scale_factor(1.5)
        .content(|| {
            boxed().alignment(Alignment::TopStart).children(|| {
                capacity_indicator(0.25);
            });
        });
    let bounds = natural.frame().node("capacity_indicator").bounds();
    assert_eq!((bounds.width.0, bounds.height.0), (270, 24));
}

#[test]
fn fractional_density_ancestor_scaling_and_collapsed_layout_preserve_local_geometry() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::level_indicator::tests::fractional_density_ancestor_scaling_and_collapsed_layout_preserve_local_geometry",
        check_scaled_geometry,
    );
}

fn check_capacity_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(440, 100)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    let green = ui.pixel(30, 15);
    assert_eq!(&green[..3], &[52, 199, 89]);
    assert!(ui.pixel(300, 15)[0].abs_diff(225) <= 1);
    assert!(ui.pixel(30, 2)[1].abs_diff(173) <= 2);
    controls.warning.set(Some(0.6));
    controls.critical.set(Some(0.8));
    controls.value.set(0.5999);
    ui.frame();
    assert_eq!(ui.pixel(30, 15), green);
    controls.size.set((Dp(180.0), Dp(16.0)));
    ui.frame();
    assert_eq!(ui.pixel(30, 0), green);
    assert!(ui.pixel(30, 28)[0].abs_diff(242) <= 1);
    controls.size.set((Dp(180.0), Dp(8.0)));
    ui.frame();
    assert_eq!(ui.pixel(30, 0), green);
    assert!(ui.pixel(30, 12)[0].abs_diff(242) <= 1);
    controls.size.set((Dp(180.0), Dp(20.0)));
    ui.frame();
    controls.value.set(0.6);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[255, 204, 0]);
    controls.value.set(0.8);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[255, 56, 60]);
    controls.warning.set(Some(0.9));
    controls.critical.set(Some(0.95));
    ui.frame();
    assert_eq!(ui.pixel(30, 15), green);
    controls.warning.set(None);
    controls.critical.set(None);
    controls.range.set(20.0..=80.0);
    controls.warning.set(Some(56.0));
    controls.critical.set(Some(68.0));
    controls.value.set(56.0);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[255, 204, 0]);
    controls.value.set(68.0);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[255, 56, 60]);
    controls.range.set(0.0..=1.0);
    controls.warning.set(Some(0.6));
    controls.critical.set(Some(0.8));
    controls.value.set(0.0);
    ui.frame();
    let empty = ui.read_pixels();
    controls.value.set(0.001);
    ui.frame();
    assert_eq!(empty, ui.read_pixels());
    controls.value.set(1.0);
    ui.frame();
    assert_eq!(&ui.pixel(350, 15)[..3], &[255, 56, 60]);
    controls.warning.set(None);
    controls.critical.set(None);
    ui.frame();
    assert_eq!(ui.pixel(350, 15), green);
    controls.range.set(0.0..=5.0);
    controls.value.set(0.6);
    ui.frame();
    assert_eq!(ui.pixel(43, 15), green);
    assert!(ui.pixel(44, 15)[0].abs_diff(225) <= 1);
    controls.range.set(0.0..=1.0);
    controls.value.set(1.0);
    controls.warning.set(Some(0.0));
    controls.critical.set(Some(0.0));
    ui.frame();
    assert_eq!(ui.pixel(350, 15), green);
    controls.warning.set(Some(0.6));
    controls.critical.set(Some(0.8));
    controls.dark.set(true);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[255, 66, 69]);
    controls.value.set(0.25);
    ui.frame();
    assert_eq!(&ui.pixel(30, 15)[..3], &[48, 209, 88]);
    assert!(ui.pixel(300, 15)[0].abs_diff(48) <= 1);
    let enabled = ui.read_pixels();
    controls.enabled.set(false);
    ui.frame();
    assert_eq!(enabled, ui.read_pixels());
    controls.shift.set(Dp(0.25));
    ui.frame();
    assert!(ui.pixel(0, 15)[0].abs_diff(25) <= 1);
    assert!(ui.pixel(1, 15)[1].abs_diff(182) <= 2);
    controls.shift.set(Dp::ZERO);
    ui.frame();
    assert_eq!(enabled, ui.read_pixels());
    controls.size.set((Dp::ZERO, Dp::ZERO));
    ui.frame();
    assert!(ui.pixel(30, 15)[0].abs_diff(25) <= 1);
    controls.size.set((Dp(180.0), Dp(20.0)));
    ui.frame();
    assert_eq!(enabled, ui.read_pixels());
    ui.advance(Duration::from_secs(1));
    assert_eq!(enabled, ui.read_pixels());
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
    let mut defaults = UiTest::new()
        .viewport(440, 100)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| {
            boxed().alignment(Alignment::TopStart).children(|| {
                capacity_indicator(5.0).modifier(Modifier::new().size(Dp(180.0), Dp(20.0)));
            });
        });
    defaults.frames(2);
    assert_eq!(&defaults.pixel(350, 15)[..3], &[52, 199, 89]);
}

#[test]
#[ignore = "requires a native GPU adapter for capacity pixels"]
fn thresholds_outline_pixel_alignment_and_collapsed_recovery_update_retained_pixels() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::level_indicator::tests::thresholds_outline_pixel_alignment_and_collapsed_recovery_update_retained_pixels",
        check_capacity_pixels,
    );
}
