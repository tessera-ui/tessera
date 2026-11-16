use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PlacementModifierNode,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
    winit::{
        event::ElementState,
        keyboard::{ModifiersState, NamedKey},
    },
};

use crate::{alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _};

use super::{
    super::{
        text::text,
        theme::{Appearance, HigTheme, hig_theme},
    },
    disclosure_button,
};

#[derive(Clone, Copy)]
struct Controls {
    expanded: State<bool>,
    enabled: State<bool>,
    accepted: State<bool>,
    visible: State<bool>,
    active: State<bool>,
    dark: State<bool>,
    width: State<Dp>,
    shift: State<Dp>,
    changes: State<Vec<bool>>,
    focus: FocusRequester,
    other_focus: FocusRequester,
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOffset(Dp);

impl PlacementModifierNode for FractionalOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        let shift = self.0.to_pixels_f32();
        position + PxPosition::from_f32_xy(shift, shift)
    }
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let expanded = remember(|| false);
    let enabled = remember(|| true);
    let accepted = remember(|| true);
    let visible = remember(|| true);
    let active = remember(|| true);
    let dark = remember(|| false);
    let width = remember(|| Dp(20.0));
    let shift = remember(|| Dp::ZERO);
    let changes = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    let other_focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        expanded,
        enabled,
        accepted,
        visible,
        active,
        dark,
        width,
        shift,
        changes,
        focus,
        other_focus,
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
                    column().children(move || {
                        if visible.get() {
                            disclosure_button(expanded.get())
                                .enabled(enabled.get())
                                .window_active(active.get())
                                .focus_requester(focus)
                                .accessibility_label("Details")
                                .accessibility_description("Additional settings")
                                .on_expanded_change(move |next| {
                                    changes.with_mut(|changes| changes.push(next));
                                    if accepted.get() {
                                        expanded.set(next);
                                    }
                                })
                                .modifier(
                                    Modifier::new()
                                        .push_placement(FractionalOffset(shift.get()))
                                        .size(width.get(), Dp(20.0))
                                        .semantics(SemanticsArgs {
                                            test_tag: Some("disclosure".into()),
                                            ..Default::default()
                                        }),
                                );
                        }
                        text("Other")
                            .modifier(Modifier::new().focus_requester(other_focus).focusable());
                    });
                });
        });
}

#[test]
fn pointer_release_requests_latest_expansion_and_rejected_updates_remain_controlled() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(160, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let inside = PxPosition::from_f32_xy(10.0, 10.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert!(controls.changes.get().is_empty());
    controls.expanded.set(true);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.changes.get(), [false]);
    assert_eq!(
        ui.accessibility_node("Details").1.is_expanded(),
        Some(false)
    );
    controls.accepted.set(false);
    ui.frame();
    ui.click_at(inside);
    ui.click_at(inside);
    assert_eq!(controls.changes.get(), [false, true, true]);
    assert!(!controls.expanded.get());
    assert_eq!(
        ui.accessibility_node("Details").1.is_expanded(),
        Some(false)
    );
    controls.accepted.set(true);
    ui.frame();
    ui.click_at(inside);
    assert!(controls.expanded.get());
}

#[test]
fn leaving_reentering_disabling_and_removal_cancel_pending_pointer_activation() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(160, 80)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let inside = PxPosition::from_f32_xy(10.0, 10.0);
    let outside = PxPosition::from_f32_xy(100.0, 60.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.pointer_move(MOUSE_POINTER_ID, outside);
    ui.frame();
    ui.pointer_move(MOUSE_POINTER_ID, inside);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.changes.get(), [true]);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(controls.changes.get(), [true]);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    controls.enabled.set(false);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.click_at(inside);
    assert_eq!(controls.changes.get(), [true]);
    assert!(!ui.run(|| controls.focus.is_focused()));
    controls.enabled.set(true);
    ui.frame();
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    controls.visible.set(false);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    controls.visible.set(true);
    ui.frame();
    assert_eq!(controls.changes.get(), [true]);
    ui.click_at(inside);
    assert_eq!(controls.changes.get(), [true, false]);
}

fn check_scaled_input() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .scale_factor(1.5)
        .content(|| {
            let probe = probe.clone();
            scaled_box(1.25).child(move || fixture(probe.clone()));
        });
    ui.frame();
    let controls = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Details");
    let bounds = node.bounds().unwrap();
    assert_eq!((bounds.x1 - bounds.x0, bounds.y1 - bounds.y0), (37.5, 37.5));
    ui.click_at(PxPosition::from_f32_xy(36.0, 36.0));
    assert!(controls.expanded.get());
    assert_eq!(ui.accessibility_node("Details").0, id);
    controls.width.set(Dp::ZERO);
    ui.frame();
    ui.click_at(PxPosition::from_f32_xy(10.0, 10.0));
    assert_eq!(controls.changes.get(), [true]);
    controls.width.set(Dp(20.0));
    ui.frame();
    ui.click_at(PxPosition::from_f32_xy(36.0, 36.0));
    assert_eq!(controls.changes.get(), [true, false]);
    let mut natural = UiTest::new()
        .viewport(100, 100)
        .scale_factor(1.5)
        .content(|| {
            disclosure_button(false).accessibility_label("Natural");
        });
    let frame = natural.frame();
    let bounds = frame.node("disclosure_glyph").bounds();
    assert_eq!((bounds.width.0, bounds.height.0), (19, 19));
}

#[test]
fn fractional_density_scaled_targets_and_zero_width_recovery_use_the_real_control() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::disclosure_button::tests::fractional_density_scaled_targets_and_zero_width_recovery_use_the_real_control",
        check_scaled_input,
    );
}

fn check_keyboard_pixels_and_semantics() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 160)
        .scale_factor(2.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(2);
    let controls = probe.get().unwrap();
    let initial = ui.read_pixels();
    let (id, node) = ui.accessibility_node("Details");
    assert_eq!(node.role(), Role::DisclosureTriangle);
    assert_eq!(node.description(), Some("Additional settings"));
    assert_eq!(node.is_expanded(), Some(false));
    assert_eq!(node.toggled(), None);
    assert!(node.supports_action(Action::Expand));
    assert!(!node.supports_action(Action::Collapse));
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert!(controls.changes.get().is_empty());
    let focused = ui.read_pixels();
    assert_ne!(initial, focused);
    ui.press_key(NamedKey::Space);
    ui.frame();
    assert!(controls.changes.get().is_empty());
    assert_ne!(focused, ui.read_pixels());
    let mut repeat = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeat.repeat = true;
    ui.inject_key_event(repeat);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.changes.get(), [true]);
    let (_, expanded) = ui.accessibility_node("Details");
    assert_eq!(expanded.is_expanded(), Some(true));
    assert!(expanded.supports_action(Action::Collapse));
    assert!(!expanded.supports_action(Action::Expand));
    ui.press_key(NamedKey::Space);
    ui.frame();
    ui.run(|| controls.other_focus.request_focus());
    ui.frame();
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(controls.changes.get(), [true]);
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
    assert_eq!(controls.changes.get(), [true]);
    assert!(ui.accessibility_action(id, Action::Collapse));
    ui.frame();
    assert_eq!(controls.changes.get(), [true, false]);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frame();
    assert_eq!(controls.changes.get(), [true, false, true]);
    ui.press_key(NamedKey::Space);
    ui.frame();
    controls.enabled.set(false);
    ui.frame();
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frame();
    let (_, disabled) = ui.accessibility_node("Details");
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!disabled.supports_action(Action::Collapse));
    ui.accessibility_action(id, Action::Click);
    ui.frame();
    assert_eq!(controls.changes.get(), [true, false, true]);
    controls.enabled.set(true);
    controls.active.set(false);
    ui.frame();
    let resting = ui.read_pixels();
    assert!(ui.pixel(19, 24)[0].abs_diff(191) <= 2);
    controls.expanded.set(false);
    ui.frame();
    assert_ne!(resting, ui.read_pixels());
    assert!(ui.pixel(24, 19)[0].abs_diff(191) <= 2);
    controls.enabled.set(false);
    ui.frame();
    assert!(ui.pixel(24, 19)[0].abs_diff(229) <= 2);
    controls.enabled.set(true);
    controls.dark.set(true);
    ui.frame();
    assert!(ui.pixel(24, 19)[0].abs_diff(128) <= 2);
    assert!(ui.pixel(17, 12)[0].abs_diff(128) <= 2);
    controls.shift.set(Dp(0.25));
    ui.frame();
    assert!(ui.pixel(17, 12)[0].abs_diff(91) <= 5);
    controls.shift.set(Dp::ZERO);
    ui.frame();
    let dark = ui.read_pixels();
    assert_ne!(resting, dark);
    controls.width.set(Dp::ZERO);
    ui.frame();
    assert_ne!(dark, ui.read_pixels());
    controls.width.set(Dp(20.0));
    ui.frame();
    assert_eq!(dark, ui.read_pixels());
    ui.advance(Duration::from_secs(1));
    assert_eq!(dark, ui.read_pixels());
    assert_eq!(ui.frame_clock_diagnostics().waiting_tasks, 0);
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
    ui.frame();
    controls.visible.set(true);
    ui.frame();
    assert_eq!(controls.changes.get(), [true, false, true]);
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and disclosure pixels"]
fn keyboard_feedback_cancellation_accessibility_and_retained_pixels_use_the_disclosure_button() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::disclosure_button::tests::keyboard_feedback_cancellation_accessibility_and_retained_pixels_use_the_disclosure_button",
        check_keyboard_pixels_and_semantics,
    );
}
