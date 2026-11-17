use std::time::Duration;

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role},
    modifier::FocusModifierExt as _,
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
    winit::{
        event::ElementState,
        keyboard::{ModifiersState, NamedKey},
    },
};

use crate::{alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _};

use super::{
    super::theme::{Appearance, HigTheme, hig_theme},
    help_button,
};

#[derive(Clone, Copy)]
struct Controls {
    count: State<u32>,
    enabled: State<bool>,
    visible: State<bool>,
    width: State<Dp>,
    active: State<bool>,
    dark: State<bool>,
    focus: FocusRequester,
    other: FocusRequester,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>, transparent: Option<bool>) {
    let count = remember(|| 0);
    let enabled = remember(|| true);
    let visible = remember(|| true);
    let width = remember(|| Dp(24.0));
    let active = remember(|| true);
    let dark = remember(|| false);
    let focus = remember(FocusRequester::new).get();
    let other = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        count,
        enabled,
        visible,
        width,
        active,
        dark,
        focus,
        other,
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
                .modifier(if transparent.unwrap_or(false) {
                    Modifier::new()
                } else {
                    Modifier::new().background(if dark.get() {
                        Color::BLACK
                    } else {
                        Color::WHITE
                    })
                })
                .children(move || {
                    column().children(move || {
                        if visible.get() {
                            help_button()
                                .enabled(enabled.get())
                                .window_active(active.get())
                                .focus_requester(focus)
                                .accessibility_description("Documentation")
                                .modifier(Modifier::new().size(width.get(), Dp(24.0)))
                                .on_click(move || count.set(count.get() + 1));
                        }
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(24.0), Dp(24.0))
                                .focus_requester(other)
                                .focusable(),
                        );
                    });
                });
        });
}

#[test]
fn pointer_release_reentry_cancellation_and_disabled_removal_use_the_help_target() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(160, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let inside = PxPosition::from_f32_xy(12.0, 12.0);
    let outside = PxPosition::from_f32_xy(100.0, 60.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.count.get(), 0);
    ui.pointer_move(MOUSE_POINTER_ID, outside);
    ui.pointer_move(MOUSE_POINTER_ID, inside);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    controls.enabled.set(false);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.click_at(inside);
    assert_eq!(controls.count.get(), 1);
    controls.enabled.set(true);
    ui.frame();
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    controls.visible.set(false);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    controls.visible.set(true);
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    ui.click_at(inside);
    assert_eq!(controls.count.get(), 2);
}

fn check_scaled_target() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Help");
    let bounds = node.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (45.0, 45.0));
    ui.click_at(PxPosition::from_f32_xy(43.0, 43.0));
    assert_eq!(controls.count.get(), 1);
    controls.width.set(Dp::ZERO);
    ui.frame();
    ui.click_at(PxPosition::from_f32_xy(12.0, 12.0));
    assert_eq!(controls.count.get(), 1);
    controls.width.set(Dp(24.0));
    ui.frame();
    assert_eq!(ui.accessibility_node("Help").0, id);
    ui.click_at(PxPosition::from_f32_xy(43.0, 43.0));
    assert_eq!(controls.count.get(), 2);
    let mut natural = UiTest::new()
        .viewport(160, 80)
        .scale_factor(1.5)
        .content(|| {
            boxed().alignment(Alignment::TopStart).children(|| {
                help_button();
            });
        });
    natural.frame();
    let bounds = natural.accessibility_node("Help").1.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (36.0, 36.0));
}

#[test]
fn fractional_density_scaled_targets_and_collapsed_recovery_preserve_actions() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::help_button::tests::fractional_density_scaled_targets_and_collapsed_recovery_preserve_actions",
        check_scaled_target,
    );
}

fn check_keyboard_at_samples(samples: u32) {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(160, 100)
        .samples(samples)
        .rendered(crate::TesseraComponents)
        .content(|| {
            fixture(probe.clone()).transparent(true);
        });
    ui.frames(2);
    let controls = probe.get().unwrap();
    let initial = ui.read_pixels();
    let (id, node) = ui.accessibility_node("Help");
    assert_eq!(node.role(), Role::Button);
    assert_eq!(node.description(), Some("Documentation"));
    assert_eq!(node.toggled(), None);
    assert_eq!(node.is_expanded(), None);
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert_eq!(controls.count.get(), 0);
    let focused = ui.read_pixels();
    assert_ne!(initial, focused);
    ui.press_key(NamedKey::Enter);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.count.get(), 0);
    ui.press_key(NamedKey::Space);
    ui.frame();
    assert_eq!(controls.count.get(), 0);
    assert_ne!(focused, ui.read_pixels());
    let mut repeat = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeat.repeat = true;
    ui.inject_key_event(repeat);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    ui.press_key(NamedKey::Space);
    ui.frame();
    ui.run(|| controls.other.request_focus());
    ui.frame();
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.count.get(), 1);
    ui.run(|| controls.focus.request_focus());
    ui.frame();
    ui.press_key(NamedKey::Space);
    ui.frame();
    ui.set_key_modifiers(ModifiersState::CONTROL);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    ui.set_key_modifiers(ModifiersState::empty());
    assert_eq!(controls.count.get(), 1);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frame();
    assert_eq!(controls.count.get(), 2);
    ui.press_key(NamedKey::Space);
    ui.frame();
    controls.enabled.set(false);
    ui.frame();
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    let (_, node) = ui.accessibility_node("Help");
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Click));
    ui.accessibility_action(id, Action::Click);
    ui.frame();
    assert_eq!(controls.count.get(), 2);
    controls.enabled.set(true);
    ui.frame();
    ui.run(|| controls.focus.request_focus());
    ui.frame();
    ui.press_key(NamedKey::Space);
    ui.frame();
    controls.visible.set(false);
    ui.frame();
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    controls.visible.set(true);
    ui.frame();
    assert_eq!(controls.count.get(), 2);
    let restored = ui.read_pixels();
    ui.advance(Duration::from_secs(1));
    let idle = ui.read_pixels();
    assert!(
        restored == idle,
        "transparent focus pixels changed while idle at MSAA{samples}"
    );
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
}

fn check_keyboard() {
    for samples in [1, 4] {
        check_keyboard_at_samples(samples);
    }
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and help pixels"]
fn keyboard_release_focus_cancellation_accessibility_and_removal_use_the_help_button() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::help_button::tests::keyboard_release_focus_cancellation_accessibility_and_removal_use_the_help_button",
        check_keyboard,
    );
}

fn check_retained_face() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(160, 100)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    controls.active.set(false);
    ui.frame();
    let light = ui.read_pixels();
    assert!(ui.pixel(10, 24)[0].abs_diff(236) <= 1);
    assert!(ui.pixel(23, 35)[0] < 50);
    controls.enabled.set(false);
    ui.frame();
    assert!(ui.pixel(10, 24)[0].abs_diff(245) <= 1);
    controls.enabled.set(true);
    ui.frame();
    assert_eq!(light, ui.read_pixels());
    controls.dark.set(true);
    ui.frame();
    assert!(ui.pixel(10, 24)[0].abs_diff(19) <= 1);
    assert!(ui.pixel(23, 35)[0] > 210);
    let dark = ui.read_pixels();
    controls.width.set(Dp::ZERO);
    ui.frame();
    assert_ne!(dark, ui.read_pixels());
    controls.width.set(Dp(24.0));
    ui.frame();
    assert_eq!(dark, ui.read_pixels());
    controls.enabled.set(false);
    ui.frame();
    assert!(ui.pixel(10, 24)[0].abs_diff(10) <= 1);
    controls.enabled.set(true);
    ui.frame();
    ui.advance(Duration::from_secs(1));
    assert_eq!(dark, ui.read_pixels());
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
}

#[test]
#[ignore = "requires a native GPU adapter for retained help pixels"]
fn inactive_face_theme_disabled_and_collapsed_recovery_update_retained_pixels() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::help_button::tests::inactive_face_theme_disabled_and_collapsed_recovery_update_retained_pixels",
        check_retained_face,
    );
}
