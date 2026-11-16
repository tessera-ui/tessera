use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role, Toggled},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::{column::column, modifier::ModifierExt as _};

use super::{
    super::button::{button, button_scope},
    switch,
};

#[derive(Clone, Copy)]
struct Controls {
    value: State<bool>,
    enabled: State<bool>,
    accept: State<bool>,
    requests: State<Vec<bool>>,
    focus: FocusRequester,
    other_focus: FocusRequester,
    submits: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| false);
    let enabled = remember(|| true);
    let accept = remember(|| true);
    let requests = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    let other_focus = remember(FocusRequester::new).get();
    let submits = remember(|| 0);
    probe.set(Some(Controls {
        value,
        enabled,
        accept,
        requests,
        focus,
        other_focus,
        submits,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                switch(value.get())
                    .enabled(enabled.get())
                    .focus_requester(focus)
                    .accessibility_label("Notifications")
                    .on_value_change(move |next| {
                        requests.with_mut(|requests| requests.push(next));
                        if accept.get() {
                            value.set(next);
                        }
                    })
                    .modifier(
                        Modifier::new()
                            .size(Dp(54.0), Dp(24.0))
                            .semantics(SemanticsArgs {
                                test_tag: Some("setting".into()),
                                ..Default::default()
                            }),
                    );
                button("Save")
                    .scope(scope)
                    .default_action(true)
                    .focus_requester(other_focus)
                    .on_click(move || submits.set(submits.get() + 1));
            });
    });
}

#[test]
fn pointer_release_uses_the_current_controlled_value_and_disabled_presses_cancel() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.node("setting").click();
    assert!(controls.value.get());
    controls.value.set(false);
    controls.accept.set(false);
    ui.frames(1);
    ui.node("setting").click();
    ui.node("setting").click();
    assert!(!controls.value.get());
    assert_eq!(controls.requests.get(), [true, true, true]);
    controls.value.set(true);
    ui.frames(1);
    ui.node("setting").click();
    assert_eq!(controls.requests.get(), [true, true, true, false]);
    let bounds = ui.frames(1).node("setting").bounds();
    let inside = PxPosition::from_f32_xy(bounds.x.to_f32() + 10.0, bounds.y.to_f32() + 10.0);
    let outside = PxPosition::from_f32_xy(300.0, 110.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), 4);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.node("setting").click();
    assert_eq!(controls.requests.get().len(), 4);
    let (_, node) = ui.accessibility_node("Notifications");
    assert_eq!(node.role(), Role::Switch);
    assert_eq!(node.toggled(), Some(Toggled::True));
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Click));
}

fn key_up(ui: &mut UiTest<'_>, key: NamedKey) {
    ui.inject_key_event(KeyboardEvent::named(key, ElementState::Released));
    ui.frames(1);
}

fn check_keyboard_and_pixels() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 120)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    let off = ui.read_pixels();
    let (id, _) = ui.accessibility_node("Notifications");
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert!(controls.requests.get().is_empty());
    assert!(ui.run(|| controls.focus.is_focused()));
    assert_ne!(off, ui.read_pixels());
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert!(controls.requests.get().is_empty());
    assert_ne!(focused, ui.read_pixels());
    let mut repeated = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    assert!(controls.requests.get().is_empty());
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true]);
    assert_eq!(
        ui.accessibility_node("Notifications").1.toggled(),
        Some(Toggled::True)
    );
    let frame = ui.frames(1);
    let bounds = frame.node("setting").bounds();
    let thumb = frame.node("switch_thumb").bounds();
    assert_eq!((bounds.width.0, bounds.height.0), (54, 24));
    assert_eq!(
        (
            thumb.x.0 - bounds.x.0,
            thumb.y.0 - bounds.y.0,
            thumb.width.0,
            thumb.height.0
        ),
        (20, 2, 32, 20)
    );
    let on = ui.read_pixels();
    assert_ne!(off, on);
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.requests.get(), [true, false]);
    assert_eq!(
        ui.accessibility_node("Notifications").1.toggled(),
        Some(Toggled::False)
    );
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.submits.get(), 1);
    key_up(&mut ui, NamedKey::Enter);
    assert_eq!(controls.requests.get(), [true, false]);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.run(|| controls.other_focus.request_focus());
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true, false]);
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get(), [true, false]);
    assert!(!ui.run(|| controls.focus.is_focused()));
    assert_ne!(focused, ui.read_pixels());
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn keyboard_feedback_focus_cancellation_and_accessibility_use_the_switch() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::switch::tests::keyboard_feedback_focus_cancellation_and_accessibility_use_the_switch",
        check_keyboard_and_pixels,
    );
}
