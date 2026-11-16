use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, Modifier, PxPosition, State,
    accesskit::{Action, Role},
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest, WorkPhase},
};

use crate::{alignment::Alignment, boxed::boxed, modifier::ModifierExt as _};

use super::{
    super::{
        button::button,
        theme::{Appearance, HigTheme, hig_theme},
    },
    group_box,
};

#[derive(Clone, Copy)]
struct Controls {
    title: State<Option<String>>,
    size: State<Option<(Dp, Dp)>>,
    dark: State<bool>,
    clipped: State<bool>,
    count: State<u32>,
    focus: FocusRequester,
}

fn tag(name: &'static str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        ..Default::default()
    })
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let title = remember(|| Some("General".to_owned()));
    let size = remember(|| Some((Dp(200.0), Dp(80.0))));
    let dark = remember(|| false);
    let clipped = remember(|| false);
    let count = remember(|| 0);
    let focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        title,
        size,
        dark,
        clipped,
        count,
        focus,
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
                    let mut modifier = tag("group");
                    if let Some((width, height)) = size.get() {
                        modifier = modifier.size(width, height);
                    }
                    if clipped.get() {
                        modifier = modifier.clip_to_bounds();
                    }
                    let group = group_box().modifier(modifier);
                    if let Some(title) = title.get() {
                        group.title(title);
                    }
                    group.child(move || {
                        button("Increment")
                            .accessibility_label("Increment count")
                            .focus_requester(focus)
                            .modifier(tag("control").size(Dp(100.0), Dp(24.0)))
                            .on_click(move || count.set(count.get() + 1));
                    });
                });
        });
}

#[test]
fn title_changes_reposition_content_and_preserve_caller_state_and_action_targets() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .content(|| fixture(probe.clone()));
    let frame = ui.frame();
    let controls = probe.get().unwrap();
    let button = frame.node("control").bounds();
    assert_eq!(
        (button.x.0, button.y.0, button.width.0, button.height.0),
        (5, 17, 100, 24)
    );
    let (id, node) = ui.accessibility_node("Increment count");
    assert_eq!(node.role(), Role::Button);
    let tree = ui.inspect_accessibility().unwrap();
    let (_, group) = tree.find("General", Role::Group).unwrap();
    assert_eq!(group.role(), Role::Group);
    assert!(!group.supports_action(Action::Focus));
    ui.click_at(PxPosition::from_f32_xy(55.0, 29.0));
    assert_eq!(controls.count.get(), 1);
    controls.title.set(None);
    let frame = ui.frame();
    assert_eq!(frame.node("control").bounds().y.0, 5);
    assert_eq!(ui.accessibility_node("Increment count").0, id);
    ui.click_at(PxPosition::from_f32_xy(55.0, 17.0));
    assert_eq!(controls.count.get(), 2);
    controls.title.set(Some("Advanced".into()));
    ui.frame();
    assert!(
        ui.inspect_accessibility()
            .unwrap()
            .find("Advanced", Role::Group)
            .is_some()
    );
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frame();
    assert_eq!(controls.count.get(), 3);
    controls.title.set(Some(String::new()));
    assert_eq!(ui.frame().node("control").bounds().y.0, 5);
    controls.size.set(None);
    let natural = ui.frame().node("group").bounds();
    assert_eq!((natural.width.0, natural.height.0), (110, 34));
    controls.title.set(Some("General".into()));
    let natural = ui.frame().node("group").bounds();
    assert_eq!((natural.width.0, natural.height.0), (110, 46));
}

#[test]
fn constrained_and_collapsed_groups_bound_content_and_restore_input() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    for size in [(0.0, 0.0), (3.0, 4.0), (8.0, 20.0), (60.0, 30.0)] {
        controls.size.set(Some((Dp(size.0), Dp(size.1))));
        let frame = ui.frame();
        let group = frame.node("group").bounds();
        assert_eq!(
            (group.width.0, group.height.0),
            (size.0 as i32, size.1 as i32)
        );
        let snapshot = ui.inspect_components();
        let child = snapshot
            .parent(snapshot.tagged("control"))
            .unwrap()
            .bounds()
            .unwrap();
        assert!(child.width <= (size.0 as f32 - 10.0).max(0.0));
        assert!(child.height <= (size.1 as f32 - 22.0).max(0.0));
        ui.click_at(PxPosition::from_f32_xy(55.0, 29.0));
        assert_eq!(controls.count.get(), 0);
    }
    controls.size.set(Some((Dp(200.0), Dp(80.0))));
    ui.frame();
    ui.click_at(PxPosition::from_f32_xy(55.0, 29.0));
    assert_eq!(controls.count.get(), 1);
}

fn check_scaled_geometry() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(640, 320)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let bounds = ui.accessibility_node("Increment count").1.bounds().unwrap();
    assert_eq!(
        (
            bounds.x0,
            bounds.y0,
            bounds.x1 - bounds.x0,
            bounds.y1 - bounds.y0
        ),
        (9.375, 31.875, 187.5, 45.0)
    );
    ui.click_at(PxPosition::from_f32_xy(100.0, 50.0));
    assert_eq!(controls.count.get(), 1);
    controls.title.set(None);
    ui.frame();
    assert_eq!(
        ui.accessibility_node("Increment count")
            .1
            .bounds()
            .unwrap()
            .y0,
        9.375
    );
    controls.dark.set(true);
    ui.frame();
    assert_eq!(ui.work().phase(WorkPhase::Measure).count(), 0);
}

#[test]
fn fractional_density_and_ancestor_scaling_keep_content_insets_and_input_in_window_coordinates() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::group_box::tests::fractional_density_and_ancestor_scaling_keep_content_insets_and_input_in_window_coordinates",
        check_scaled_geometry,
    );
}

fn check_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(640, 320)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    assert!(ui.pixel(300, 140)[0].abs_diff(235) <= 1);
    assert!(ui.pixel(300, 10)[0].abs_diff(242) <= 1);
    let (id, _) = ui.accessibility_node("Increment count");
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert!(ui.run(|| controls.focus.is_focused()));
    assert_eq!(controls.count.get(), 0);
    controls.dark.set(true);
    ui.frame();
    assert!(ui.pixel(300, 140)[0].abs_diff(32) <= 1);
    controls.title.set(None);
    ui.frame();
    assert!(ui.pixel(300, 10)[0].abs_diff(32) <= 1);
    controls.clipped.set(true);
    controls.size.set(Some((Dp::ZERO, Dp::ZERO)));
    ui.frame();
    assert!(ui.pixel(300, 140)[0].abs_diff(25) <= 1);
    controls.size.set(Some((Dp(200.0), Dp(80.0))));
    ui.frame();
    assert!(ui.pixel(300, 140)[0].abs_diff(32) <= 1);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
}

#[test]
#[ignore = "requires a native GPU adapter for group container pixels and focus"]
fn inherited_appearance_title_removal_focus_and_collapsed_recovery_update_real_pixels() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::group_box::tests::inherited_appearance_title_removal_focus_and_collapsed_recovery_update_real_pixels",
        check_pixels,
    );
}
