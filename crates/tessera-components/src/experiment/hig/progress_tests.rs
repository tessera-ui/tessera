use std::{ops::RangeInclusive, time::Duration};

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, Modifier, PxPosition, State,
    accesskit::{Action, Role},
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{alignment::Alignment, boxed::boxed, modifier::ModifierExt as _};

use super::{
    super::theme::{Appearance, HigTheme, hig_theme},
    progress_bar,
};

#[derive(Clone, Copy)]
struct Controls {
    value: State<f64>,
    range: State<RangeInclusive<f64>>,
    width: State<Dp>,
    height: State<Dp>,
    dark: State<bool>,
    active: State<bool>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| 0.25);
    let range = remember(|| 0.0..=1.0);
    let width = remember(|| Dp(240.0));
    let height = remember(|| Dp(20.0));
    let dark = remember(|| false);
    let active = remember(|| false);
    probe.set(Some(Controls {
        value,
        range,
        width,
        height,
        dark,
        active,
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
                    progress_bar(value.get())
                        .range(range.get())
                        .window_active(active.get())
                        .accessibility_label("Download")
                        .accessibility_description("Downloaded bytes")
                        .modifier(Modifier::new().size(width.get(), height.get()));
                });
        });
}

#[test]
fn retained_values_and_ranges_update_read_only_progress_semantics() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Download");
    assert_eq!(node.role(), Role::ProgressIndicator);
    assert_eq!(node.numeric_value(), Some(0.25));
    assert_eq!(node.description(), Some("Downloaded bytes"));
    assert!(!node.supports_action(Action::Focus));
    assert!(!node.supports_action(Action::Increment));
    assert!(!ui.accessibility_action(id, Action::Increment));
    ui.click_at(PxPosition::from_f32_xy(200.0, 10.0));
    assert_eq!(controls.value.get(), 0.25);
    controls.value.set(1.5);
    ui.frame();
    assert_eq!(
        ui.accessibility_node("Download").1.numeric_value(),
        Some(1.0)
    );
    assert_eq!(controls.value.get(), 1.5);
    controls.range.set(20.0..=80.0);
    ui.frame();
    let (_, node) = ui.accessibility_node("Download");
    assert_eq!(
        (
            node.numeric_value(),
            node.min_numeric_value(),
            node.max_numeric_value()
        ),
        (Some(20.0), Some(20.0), Some(80.0))
    );
    controls.value.set(50.0);
    ui.frame();
    assert_eq!(
        ui.accessibility_node("Download").1.numeric_value(),
        Some(50.0)
    );
}

fn check_fractional_constraints() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(600, 120)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let bounds = ui.accessibility_node("Download").1.bounds().unwrap();
    assert_eq!(
        (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
        (450.0, 37.5)
    );
    controls.width.set(Dp(0.0));
    controls.height.set(Dp(0.0));
    ui.frame();
    let bounds = ui.accessibility_node("Download").1.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (0.0, 0.0));
    controls.width.set(Dp(3.0));
    controls.height.set(Dp(3.0));
    ui.frame();
    controls.value.set(1.0);
    ui.frame();
    assert_eq!(
        ui.accessibility_node("Download").1.numeric_value(),
        Some(1.0)
    );
    controls.width.set(Dp(240.0));
    controls.height.set(Dp(20.0));
    ui.frame();
    let bounds = ui.accessibility_node("Download").1.bounds().unwrap();
    assert_eq!(
        (bounds.x1 - bounds.x0, bounds.y1 - bounds.y0),
        (450.0, 37.5)
    );
}

#[test]
fn fractional_density_scaled_geometry_and_collapsed_layout_recover() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::progress::tests::fractional_density_scaled_geometry_and_collapsed_layout_recover",
        check_fractional_constraints,
    );
}

fn check_progress_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 80)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    controls.width.set(Dp(120.0));
    ui.frame();
    let first = ui.read_pixels();
    let filled = ui.pixel(30, 20);
    assert!(filled[0].abs_diff(170) <= 2, "{filled:?}");
    assert_eq!(filled[0], filled[1]);
    assert_eq!(filled[0], filled[2]);
    let track = ui.pixel(180, 20);
    assert!(track[0].abs_diff(227) <= 2, "{track:?}");
    let trailing_edge = ui.pixel(58, 20);
    assert!(trailing_edge[0].abs_diff(204) <= 16, "{trailing_edge:?}");
    controls.value.set(0.75);
    ui.frame();
    assert_eq!(ui.pixel(120, 20), filled);
    assert_ne!(first, ui.read_pixels());
    let stable = ui.read_pixels();
    ui.advance(Duration::from_secs(1));
    assert_eq!(stable, ui.read_pixels());
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
    controls.value.set(0.0);
    ui.frame();
    let cap = ui.pixel(3, 20);
    assert!(
        cap[0] < 200,
        "minimum fill cap must remain visible: {cap:?}"
    );
    assert!(
        ui.pixel(7, 13)[0] > cap[0],
        "a narrow cap must remain elliptical"
    );
    let clipped_corner = ui.pixel(5, 12);
    assert!(clipped_corner[0].abs_diff(218) <= 16, "{clipped_corner:?}");
    controls.dark.set(true);
    ui.frame();
    assert!(ui.pixel(180, 20)[0].abs_diff(43) <= 2);
    let dark_corner = ui.pixel(5, 12);
    assert!(dark_corner[0].abs_diff(57) <= 16, "{dark_corner:?}");
    controls.active.set(true);
    controls.value.set(0.75);
    ui.frame();
    let accent = ui.pixel(120, 20);
    assert!(accent[2] > accent[1] && accent[1] > accent[0], "{accent:?}");
    controls.width.set(Dp(0.0));
    ui.frame();
    controls.width.set(Dp(120.0));
    ui.frame();
    assert_eq!(ui.pixel(120, 20), accent);
}

#[test]
#[ignore = "requires a native GPU adapter for progress pixels"]
fn retained_fill_track_cap_appearance_and_zero_width_recovery_update_pixels_without_a_clock() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::progress::tests::retained_fill_track_cap_appearance_and_zero_width_recovery_update_pixels_without_a_clock",
        check_progress_pixels,
    );
}

fn check_active_single_density_cap() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 80)
        .scale_factor(1.0)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    controls.active.set(true);
    controls.dark.set(true);
    for width in [180.0, 240.0] {
        controls.width.set(Dp(width));
        controls.value.set(0.0);
        ui.frame();
        let minimum = ui.read_pixels();
        assert!(ui.pixel(2, 9)[2].abs_diff(255) <= 6);
        assert!(ui.pixel(6, 9)[2].abs_diff(43) <= 6);

        controls.value.set(8.0 / width);
        ui.frame();
        assert_eq!(ui.read_pixels(), minimum);
        controls.value.set(8.2 / width);
        ui.frame();
        assert!(ui.pixel(6, 9)[2].abs_diff(255) <= 16);
    }
    controls.value.set(0.25);
    ui.frame();
    for y in [6, 13] {
        let edge = ui.pixel(20, y);
        assert!(
            edge[..3]
                .iter()
                .zip([0_u8, 122, 255])
                .all(|(a, b)| a.abs_diff(b) <= 6),
            "{edge:?}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter for progress pixels"]
fn active_single_density_fill_keeps_its_minimum_cap_and_covers_flat_edges() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::progress::tests::active_single_density_fill_keeps_its_minimum_cap_and_covers_flat_edges",
        check_active_single_density_cap,
    );
}
