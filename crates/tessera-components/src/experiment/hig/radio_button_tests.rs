use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role, Toggled},
    key,
    modifier::FocusModifierExt as _,
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{
        event::ElementState,
        keyboard::{ModifiersState, NamedKey},
    },
};

use crate::{column::column, modifier::ModifierExt as _};

use super::{
    super::{
        button::{button, button_scope},
        text::text,
    },
    radio_button, radio_group,
};

#[derive(Clone, Copy)]
struct Controls {
    selected: State<usize>,
    accept: State<bool>,
    middle_enabled: State<bool>,
    show_last: State<bool>,
    requests: State<Vec<usize>>,
    focuses: [FocusRequester; 3],
    before: FocusRequester,
    after: FocusRequester,
    submits: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let selected = remember(|| 0usize);
    let accept = remember(|| true);
    let middle_enabled = remember(|| false);
    let show_last = remember(|| true);
    let requests = remember(Vec::new);
    let focuses = remember(|| {
        [
            FocusRequester::new(),
            FocusRequester::new(),
            FocusRequester::new(),
        ]
    })
    .get();
    let before = remember(FocusRequester::new).get();
    let after = remember(FocusRequester::new).get();
    let submits = remember(|| 0);
    probe.set(Some(Controls {
        selected,
        accept,
        middle_enabled,
        show_last,
        requests,
        focuses,
        before,
        after,
        submits,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                text("Before").modifier(Modifier::new().focus_requester(before).focusable());
                radio_group()
                    .accessibility_label("Transport")
                    .child(move |group| {
                        column().children(move || {
                            for (index, label) in
                                ["First", "Middle", "Last"].into_iter().enumerate()
                            {
                                if index == 2 && !show_last.get() {
                                    continue;
                                }
                                key(index, move || {
                                    radio_button(selected.get() == index)
                                        .label(label)
                                        .group(group)
                                        .enabled(index != 1 || middle_enabled.get())
                                        .focus_requester(focuses[index])
                                        .on_select(move || {
                                            requests.with_mut(|requests| requests.push(index));
                                            if accept.get() {
                                                selected.set(index);
                                            }
                                        })
                                        .modifier(
                                            Modifier::new().size(Dp(160.0), Dp(20.0)).semantics(
                                                SemanticsArgs {
                                                    test_tag: Some(label.into()),
                                                    ..Default::default()
                                                },
                                            ),
                                        );
                                });
                            }
                        });
                    });
                button("Save")
                    .scope(scope)
                    .default_action(true)
                    .on_click(move || submits.set(submits.get() + 1));
                text("After").modifier(Modifier::new().focus_requester(after).focusable());
            });
    });
}

#[test]
fn pointer_selection_reselection_and_rejected_requests_keep_caller_owned_values() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 200)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.node("Last").click();
    assert_eq!(controls.selected.get(), 2);
    ui.node("Last").click();
    assert_eq!(controls.requests.get(), [2, 2]);
    assert_eq!(
        ui.accessibility_node("First").1.toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        ui.accessibility_node("Last").1.toggled(),
        Some(Toggled::True)
    );
    let bounds = ui.frames(1).node("First").bounds();
    let position = PxPosition::from_f32_xy(bounds.x.to_f32() + 40.0, bounds.y.to_f32() + 10.0);
    ui.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.selected.get(), 2);
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(300.0, 190.0),
        PressKeyEventType::Left,
    );
    ui.frames(1);
    assert_eq!(controls.requests.get(), [2, 2]);
    ui.node("Middle").click();
    assert_eq!(controls.requests.get(), [2, 2]);
    controls.accept.set(false);
    ui.frames(1);
    ui.node("First").click();
    assert_eq!(controls.requests.get(), [2, 2, 0]);
    assert_eq!(controls.selected.get(), 2);
    assert_eq!(
        ui.accessibility_node("Last").1.toggled(),
        Some(Toggled::True)
    );
    controls.selected.set(1);
    ui.frames(1);
    let (_, middle) = ui.accessibility_node("Middle");
    assert!(middle.is_disabled());
    assert_eq!(middle.toggled(), Some(Toggled::True));
    assert!(!middle.supports_action(Action::Click));
}

fn key_up(ui: &mut UiTest<'_>, key: NamedKey) {
    ui.inject_key_event(KeyboardEvent::named(key, ElementState::Released));
    ui.frames(1);
}

fn check_keyboard_and_accessibility() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(320, 200)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    assert_eq!(
        ui.accessibility_node("Transport").1.role(),
        Role::RadioGroup
    );
    let (last_id, last) = ui.accessibility_node("Last");
    assert_eq!(last.role(), Role::RadioButton);
    assert!(last.supports_action(Action::Click));
    ui.run(|| controls.before.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Tab);
    ui.frames(2);
    key_up(&mut ui, NamedKey::Tab);
    assert!(ui.run(|| controls.focuses[0].is_focused()));
    assert!(controls.requests.get().is_empty());
    ui.press_key(NamedKey::ArrowDown);
    ui.frames(2);
    key_up(&mut ui, NamedKey::ArrowDown);
    assert!(ui.run(|| controls.focuses[2].is_focused()));
    assert_eq!(controls.selected.get(), 2);
    assert_eq!(controls.requests.get(), [2]);
    ui.press_key(NamedKey::ArrowRight);
    ui.frames(2);
    key_up(&mut ui, NamedKey::ArrowRight);
    assert!(ui.run(|| controls.focuses[0].is_focused()));
    assert_eq!(controls.requests.get(), [2, 0]);
    controls.middle_enabled.set(true);
    ui.frames(2);
    ui.press_key(NamedKey::ArrowDown);
    ui.frames(2);
    key_up(&mut ui, NamedKey::ArrowDown);
    assert!(ui.run(|| controls.focuses[1].is_focused()));
    assert_eq!(controls.selected.get(), 1);
    ui.press_key(NamedKey::ArrowUp);
    ui.frames(2);
    key_up(&mut ui, NamedKey::ArrowUp);
    assert_eq!(controls.selected.get(), 0);
    ui.press_key(NamedKey::ArrowLeft);
    ui.frames(2);
    key_up(&mut ui, NamedKey::ArrowLeft);
    assert_eq!(controls.selected.get(), 2);
    let count = controls.requests.get().len();
    ui.press_key(NamedKey::Tab);
    ui.frames(2);
    key_up(&mut ui, NamedKey::Tab);
    assert!(!ui.run(|| controls.focuses.iter().any(FocusRequester::is_focused)));
    assert_eq!(controls.requests.get().len(), count);
    ui.run(|| controls.after.request_focus());
    ui.frames(1);
    assert!(ui.accessibility_action(last_id, Action::Focus));
    ui.frames(2);
    assert!(ui.run(|| controls.focuses[2].is_focused()));
    assert_eq!(controls.requests.get().len(), count);
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert_ne!(focused, ui.read_pixels());
    assert_eq!(controls.requests.get().len(), count);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get().len(), count + 1);
    assert_eq!(controls.selected.get(), 2);
    assert!(ui.accessibility_action(last_id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.requests.get().len(), count + 2);
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.submits.get(), 1);
    key_up(&mut ui, NamedKey::Enter);
    ui.advance(Duration::from_millis(120));
    ui.frames(1);
    assert_eq!(controls.requests.get().len(), count + 2);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    controls.show_last.set(false);
    ui.frames(2);
    key_up(&mut ui, NamedKey::Space);
    assert_eq!(controls.requests.get().len(), count + 2);
    assert!(!ui.run(|| controls.focuses[2].is_focused()));
    controls.selected.set(1);
    controls.show_last.set(true);
    ui.frames(2);
    ui.run(|| controls.before.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Tab);
    ui.frames(2);
    key_up(&mut ui, NamedKey::Tab);
    assert!(ui.run(|| controls.focuses[1].is_focused()));
    assert_eq!(controls.requests.get().len(), count + 2);
    assert_eq!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some("Middle"))
            .count(),
        1
    );
    ui.set_key_modifiers(ModifiersState::CONTROL);
    ui.press_key(NamedKey::Space);
    key_up(&mut ui, NamedKey::Space);
    ui.set_key_modifiers(ModifiersState::empty());
    assert_eq!(controls.requests.get().len(), count + 2);
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn group_navigation_focus_actions_and_removal_preserve_single_choice_input() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::radio_button::tests::group_navigation_focus_actions_and_removal_preserve_single_choice_input",
        check_keyboard_and_accessibility,
    );
}
