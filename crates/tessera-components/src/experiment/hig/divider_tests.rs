use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, Modifier, PlacementModifierNode, PxPosition, State,
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{alignment::Alignment, boxed::boxed, modifier::ModifierExt as _};

use super::{
    super::{
        button::button,
        theme::{Appearance, HigTheme, hig_theme},
    },
    DividerOrientation, divider,
};

#[derive(Clone, Copy)]
struct Controls {
    orientation: State<DividerOrientation>,
    width: State<Option<Dp>>,
    height: State<Option<Dp>>,
    shift: State<Dp>,
    dark: State<bool>,
    clipped: State<bool>,
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOffset(Dp);

impl PlacementModifierNode for FractionalOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        let offset = Dp(20.0).to_pixels_f32() + self.0.to_pixels_f32();
        position + PxPosition::from_f32_xy(offset, offset)
    }
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let orientation = remember(|| DividerOrientation::Horizontal);
    let width = remember(|| Some(Dp(240.0)));
    let height = remember(|| Some(Dp(8.0)));
    let shift = remember(|| Dp::ZERO);
    let dark = remember(|| false);
    let clipped = remember(|| false);
    probe.set(Some(Controls {
        orientation,
        width,
        height,
        shift,
        dark,
        clipped,
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
                    let mut modifier =
                        Modifier::new().push_placement(FractionalOffset(shift.get()));
                    if let Some(width) = width.get() {
                        modifier = modifier.width(width);
                    }
                    if let Some(height) = height.get() {
                        modifier = modifier.height(height);
                    }
                    if clipped.get() {
                        modifier = modifier.clip_to_bounds();
                    }
                    divider().orientation(orientation.get()).modifier(modifier);
                });
        });
}

#[test]
fn orientation_changes_remeasure_the_bounded_length_and_natural_thickness() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    controls.width.set(None);
    controls.height.set(None);
    let horizontal = ui.frame().node("divider").bounds();
    assert_eq!((horizontal.width.0, horizontal.height.0), (320, 1));
    controls.orientation.set(DividerOrientation::Vertical);
    let vertical = ui.frame().node("divider").bounds();
    assert_eq!((vertical.width.0, vertical.height.0), (1, 120));
    controls.width.set(Some(Dp::ZERO));
    controls.height.set(Some(Dp::ZERO));
    let collapsed = ui.frame().node("divider").bounds();
    assert_eq!((collapsed.width.0, collapsed.height.0), (0, 0));
    controls.width.set(Some(Dp(8.0)));
    controls.height.set(Some(Dp(80.0)));
    let restored = ui.frame().node("divider").bounds();
    assert_eq!((restored.width.0, restored.height.0), (8, 80));
    assert_eq!(ui.accessibility_snapshot().unwrap().nodes.len(), 1);
}

#[tessera]
fn input_fixture(probe: TestProbe<Option<(State<u32>, State<bool>)>>) {
    let clicks = remember(|| 0);
    let visible = remember(|| true);
    probe.set(Some((clicks, visible)));
    boxed().alignment(Alignment::TopStart).children(move || {
        button("Continue")
            .accessibility_label("Continue action")
            .modifier(Modifier::new().size(Dp(120.0), Dp(24.0)))
            .on_click(move || clicks.set(clicks.get() + 1));
        if visible.get() {
            divider().modifier(Modifier::new().size(Dp(120.0), Dp(24.0)));
        }
    });
}

#[test]
fn a_divider_above_a_button_does_not_intercept_input_or_add_an_accessibility_target() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(240, 80)
        .content(|| input_fixture(probe.clone()));
    ui.frame();
    let (clicks, visible) = probe.get().unwrap();
    ui.click_at(PxPosition::from_f32_xy(60.0, 12.0));
    assert_eq!(clicks.get(), 1);
    let (_, button) = ui.accessibility_node("Continue action");
    assert_eq!(button.role(), tessera_ui::accesskit::Role::Button);
    let count = ui.accessibility_snapshot().unwrap().nodes.len();
    visible.set(false);
    ui.frame();
    assert_eq!(ui.accessibility_snapshot().unwrap().nodes.len(), count);
}

fn check_scaled_layout() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(640, 240)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let bounds = |ui: &UiTest| {
        ui.inspect_components()
            .nodes()
            .iter()
            .find(|node| node.name().ends_with("divider"))
            .unwrap()
            .bounds()
            .unwrap()
    };
    let horizontal = bounds(&ui);
    assert_eq!((horizontal.width, horizontal.height), (450.0, 15.0));
    controls.orientation.set(DividerOrientation::Vertical);
    controls.width.set(Some(Dp(8.0)));
    controls.height.set(Some(Dp(80.0)));
    ui.frame();
    let vertical = bounds(&ui);
    assert_eq!((vertical.width, vertical.height), (15.0, 150.0));
}

#[test]
fn fractional_density_and_ancestor_scaling_preserve_the_divider_frame() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::divider::tests::fractional_density_and_ancestor_scaling_preserve_the_divider_frame",
        check_scaled_layout,
    );
}

fn check_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(640, 240)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    let background = ui.pixel(80, 45);
    assert!(background[0].abs_diff(242) <= 1);
    assert_eq!(ui.pixel(80, 48), background);
    assert!(ui.pixel(80, 46)[0].abs_diff(218) <= 1);
    assert!(ui.pixel(80, 47)[0].abs_diff(218) <= 1);
    controls.shift.set(Dp(0.25));
    ui.frame();
    assert_eq!(ui.pixel(80, 48), background);
    assert!(ui.pixel(80, 46)[0].abs_diff(218) <= 1);
    controls.shift.set(Dp(0.5));
    ui.frame();
    assert_eq!(ui.pixel(80, 46), background);
    assert!(ui.pixel(80, 48)[0].abs_diff(218) <= 1);
    controls.height.set(None);
    ui.frame();
    assert!(ui.pixel(80, 40)[0].abs_diff(218) <= 1);
    assert_eq!(ui.pixel(80, 48), background);
    controls.clipped.set(true);
    ui.frame();
    assert_eq!(ui.pixel(80, 40), background);
    assert!(ui.pixel(80, 41)[0].abs_diff(218) <= 1);
    controls.clipped.set(false);
    controls.orientation.set(DividerOrientation::Vertical);
    controls.width.set(Some(Dp(8.0)));
    controls.height.set(Some(Dp(80.0)));
    ui.frame();
    assert!(ui.pixel(49, 80)[0].abs_diff(218) <= 1);
    assert_eq!(ui.pixel(48, 80), background);
    controls.dark.set(true);
    ui.frame();
    assert!(ui.pixel(49, 80)[0].abs_diff(53) <= 1);
    let visible = ui.read_pixels();
    controls.width.set(Some(Dp::ZERO));
    ui.frame();
    assert!(ui.pixel(49, 80)[0].abs_diff(25) <= 1);
    controls.width.set(Some(Dp(8.0)));
    ui.frame();
    assert_eq!(ui.read_pixels(), visible);
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
}

#[test]
#[ignore = "requires a native GPU adapter for divider pixels"]
fn native_alignment_fractional_motion_clipping_theme_changes_and_collapsed_recovery_update_pixels()
{
    crate::test_utils::run_isolated_test(
        "experiment::hig::divider::tests::native_alignment_fractional_motion_clipping_theme_changes_and_collapsed_recovery_update_pixels",
        check_pixels,
    );
}
