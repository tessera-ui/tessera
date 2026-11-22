use std::time::Duration;

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
    IndeterminateBarSize, SpinnerSize, indeterminate_progress_bar, progress_spinner, progress_view,
};

#[tessera]
fn bar_fixture(probe: TestProbe<Option<State<bool>>>) {
    let running = remember(|| false);
    probe.set(Some(running));
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().background(Color::WHITE))
        .children(move || {
            indeterminate_progress_bar(running.get()).accessibility_label("Shown bar");
            indeterminate_progress_bar(running.get())
                .size(IndeterminateBarSize::Small)
                .window_active(false)
                .displayed_when_stopped(false)
                .accessibility_label("Hidden bar")
                .accessibility_description("Preparing files")
                .modifier(Modifier::new().width(Dp(180.0)).offset(Dp::ZERO, Dp(24.0)));
        });
}

#[test]
fn indeterminate_bar_keeps_intrinsic_height_and_exposes_no_value_or_actions() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(200, 42)
        .scale_factor(1.0)
        .content(|| bar_fixture(probe.clone()));
    ui.frame();
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
    let (_, shown) = ui.accessibility_node("Shown bar");
    let bounds = shown.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (160.0, 20.0));
    let missing = |ui: &UiTest<'_>| {
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.label() != Some("Hidden bar"))
    };
    assert!(missing(&ui));
    let running = probe.get().unwrap();
    running.set(true);
    ui.frame();
    let (id, hidden) = ui.accessibility_node("Hidden bar");
    let bounds = hidden.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (180.0, 12.0));
    assert_eq!(hidden.role(), Role::ProgressIndicator);
    assert_eq!(hidden.description(), Some("Preparing files"));
    assert_eq!(hidden.numeric_value(), None);
    assert_eq!(hidden.min_numeric_value(), None);
    assert_eq!(hidden.max_numeric_value(), None);
    assert!(!hidden.supports_action(Action::Focus));
    assert!(!ui.accessibility_action(id, Action::Click));
    running.set(false);
    ui.frame();
    assert!(missing(&ui));
}

fn check_bar_start_stop_restart() {
    const ROW_BYTES: usize = 200 * 4;
    const SPLIT: usize = ROW_BYTES * 24;
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(200, 42)
        .scale_factor(1.0)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(|| bar_fixture(probe.clone()));
    ui.frame();
    let stopped = ui.read_pixels();
    assert!(ui.pixel(1, 10)[0] < 30);
    assert!(ui.pixel(80, 10)[0].abs_diff(239) <= 1);
    assert_eq!(ui.pixel(1, 30), [255; 4]);
    let running = probe.get().unwrap();
    running.set(true);
    for _ in 0..4 {
        ui.frame();
    }
    let first = ui.read_pixels();
    assert!(ui.pixel(1, 30)[0].abs_diff(170) <= 1);
    ui.advance(Duration::from_millis(440));
    let moving = ui.read_pixels();
    assert_ne!(moving, first);
    running.set(false);
    ui.frame();
    ui.frame();
    assert_eq!(&ui.read_pixels()[SPLIT..], &stopped[SPLIT..]);
    ui.advance(Duration::from_millis(100));
    let draining = ui.read_pixels();
    assert_ne!(&draining[..SPLIT], &moving[..SPLIT]);

    running.set(true);
    for _ in 0..4 {
        ui.frame();
    }
    let restarted = ui.read_pixels();
    assert_eq!(&restarted[..SPLIT], &draining[..SPLIT]);
    assert_eq!(&restarted[SPLIT..], &first[SPLIT..]);

    // The retained sweep is now 1.70 seconds into its original period.
    ui.advance(Duration::from_millis(1160));
    running.set(false);
    ui.frame();
    ui.frame();
    ui.advance(Duration::from_millis(400));
    assert_ne!(ui.read_pixels(), stopped);
    ui.advance(Duration::from_millis(1700));
    assert_eq!(ui.read_pixels(), stopped);
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);

    running.set(true);
    for _ in 0..4 {
        ui.frame();
    }
    assert_eq!(ui.read_pixels(), first);
    ui.advance(Duration::from_millis(440));
    running.set(false);
    ui.frame();
    ui.frame();
    ui.advance(Duration::from_millis(1460));
    assert_eq!(ui.read_pixels(), stopped);
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
}

#[test]
#[ignore = "requires a native GPU adapter for progress pixels"]
fn indeterminate_bar_drains_its_sweep_and_restarts_from_the_expected_phase() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::progress::tests::indeterminate_bar_drains_its_sweep_and_restarts_from_the_expected_phase",
        check_bar_start_stop_restart,
    );
}

#[tessera]
fn spinner_fixture(probe: TestProbe<Option<State<bool>>>) {
    let running = remember(|| false);
    probe.set(Some(running));
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().background(Color::WHITE))
        .children(move || {
            progress_spinner(running.get()).accessibility_label("Visible");
            progress_spinner(running.get())
                .size(SpinnerSize::Small)
                .displayed_when_stopped(false)
                .accessibility_label("Loading")
                .modifier(Modifier::new().offset(Dp(40.0), Dp::ZERO));
        });
}

fn check_spinner_start_and_stop() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(64, 40)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| spinner_fixture(probe.clone()));
    ui.frames(3);
    let stopped = ui.read_pixels();
    let (_, visible) = ui.accessibility_node("Visible");
    assert_eq!(visible.role(), Role::ProgressIndicator);
    assert_eq!(visible.numeric_value(), None);
    assert!(!visible.supports_action(Action::Click));
    assert!(stopped.chunks_exact(4).any(|pixel| pixel[0] < 160));
    assert!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.label() != Some("Loading"))
    );

    let running = probe.get().unwrap();
    running.set(true);
    ui.frames(3);
    let before = ui.read_pixels();
    ui.advance(Duration::from_millis(220));
    assert_ne!(ui.read_pixels(), before);
    let (_, node) = ui.accessibility_node("Loading");
    assert_eq!(node.role(), Role::ProgressIndicator);
    assert_eq!(node.numeric_value(), None);
    let bounds = node.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (16.0, 16.0));

    running.set(false);
    ui.frames(3);
    assert_eq!(ui.read_pixels(), stopped);
    ui.advance(Duration::from_secs(2));
    assert_eq!(ui.read_pixels(), stopped);
    assert!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.label() != Some("Loading"))
    );
}

#[test]
#[ignore = "requires a native GPU adapter for spinner pixels"]
fn indeterminate_spinner_animates_and_restores_stopped_visibility() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::progress::tests::indeterminate_spinner_animates_and_restores_stopped_visibility",
        check_spinner_start_and_stop,
    );
}

#[derive(Clone, Copy)]
struct Controls {
    value: State<f64>,
    total: State<f64>,
    width: State<Dp>,
    height: State<Dp>,
    dark: State<bool>,
    active: State<bool>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| 0.25);
    let total = remember(|| 1.0);
    let width = remember(|| Dp(240.0));
    let height = remember(|| Dp(20.0));
    let dark = remember(|| false);
    let active = remember(|| false);
    probe.set(Some(Controls {
        value,
        total,
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
                    progress_view()
                        .value(value.get())
                        .total(total.get())
                        .window_active(active.get())
                        .accessibility_label("Download")
                        .accessibility_description("Downloaded bytes")
                        .modifier(Modifier::new().size(width.get(), height.get()));
                });
        });
}

#[test]
fn retained_values_and_totals_update_read_only_progress_semantics() {
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
    controls.total.set(80.0);
    ui.frame();
    let (_, node) = ui.accessibility_node("Download");
    assert_eq!(
        (
            node.numeric_value(),
            node.min_numeric_value(),
            node.max_numeric_value()
        ),
        (Some(1.5), Some(0.0), Some(80.0))
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
