use std::time::Duration;

use tessera_ui::{
    Callback, CallbackWith, Dp, FocusRequester, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Node, NodeId, Role, Toggled},
    layout::{Stack, layout},
    tessera,
    testing::UiTest,
    winit::keyboard::NamedKey,
};

use crate::modifier::{
    ClickableArgs, CombinedClickableArgs, InteractionState, ModifierExt as _, SelectableArgs,
    ToggleableArgs,
};

fn node(test: &UiTest, label: &str, role: Role) -> (NodeId, Node) {
    let snapshot = test.inspect_accessibility().expect("published semantics");
    let (id, node) = snapshot.find(label, role).expect("interaction target");
    (id, node.clone())
}

#[tessera]
fn click_target(
    enabled: State<bool>,
    clicks: State<usize>,
    interaction: State<InteractionState>,
    requester: FocusRequester,
) {
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(80.0), Dp(80.0))
            .clickable(ClickableArgs {
                on_click: Callback::new(move || clicks.with_mut(|count| *count += 1)),
                enabled: enabled.get(),
                label: Some("Activate".into()),
                interaction_state: Some(interaction),
                focus_requester: Some(requester),
                ..Default::default()
            }),
    );
}

#[test]
fn clickable_routes_input_preserves_held_press_on_blur_and_disables_activation() {
    let test = UiTest::new();
    let enabled = test.state(true);
    let clicks = test.state(0);
    let interaction = test.state(InteractionState::new());
    let requester = test.run(FocusRequester::new);
    let mut test = test.content(|| click_target(enabled, clicks, interaction, requester));
    test.frame();
    let (id, target) = node(&test, "Activate", Role::Button);
    assert!(target.supports_action(Action::Click));
    assert!(test.accessibility_action(id, Action::Focus));
    test.frame();
    assert_eq!(clicks.get(), 0);
    let point = PxPosition::from_f32_xy(10.0, 10.0);
    test.pointer_move(MOUSE_POINTER_ID, point);
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert!(interaction.get().is_hovered());
    assert!(interaction.get().is_pressed());
    test.run(|| requester.clear_focus());
    test.frame();
    assert!(!interaction.get().is_focused());
    assert!(interaction.get().is_pressed());
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(clicks.get(), 1);
    assert!(!interaction.get().is_pressed());
    test.run(|| requester.request_focus());
    test.frame();
    assert_eq!(
        test.accessibility_snapshot()
            .expect("focus publication")
            .focus,
        id
    );
    test.press_key(NamedKey::Enter);
    test.frame();
    assert_eq!(clicks.get(), 2);
    assert!(test.accessibility_action(id, Action::Click));
    test.frame();
    assert_eq!(clicks.get(), 3);
    test.run(|| enabled.set(false));
    test.frame();
    let (_, disabled) = node(&test, "Activate", Role::Button);
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!test.accessibility_action(id, Action::Click));
    test.press_key(NamedKey::Enter);
    test.click_at(point);
    assert_eq!(clicks.get(), 3);
}

#[tessera]
fn combined_target(
    enabled: State<bool>,
    click_enabled: State<bool>,
    counts: State<(usize, usize)>,
    interaction: State<InteractionState>,
    requester: FocusRequester,
) {
    layout(Stack).modifier(
        Modifier::new()
            .size(Dp(80.0), Dp(80.0))
            .combined_clickable(CombinedClickableArgs {
                on_click: click_enabled
                    .get()
                    .then(|| Callback::new(move || counts.with_mut(|counts| counts.0 += 1))),
                on_long_click: Some(Callback::new(move || {
                    counts.with_mut(|counts| counts.1 += 1)
                })),
                enabled: enabled.get(),
                label: Some("Hold".into()),
                interaction_state: Some(interaction),
                focus_requester: Some(requester),
                ..Default::default()
            }),
    );
}

#[test]
fn combined_clickable_owns_release_cancels_on_blur_and_supports_long_click_only_actions() {
    let test = UiTest::new();
    let enabled = test.state(true);
    let click_enabled = test.state(true);
    let counts = test.state((0, 0));
    let interaction = test.state(InteractionState::new());
    let requester = test.run(FocusRequester::new);
    let mut test = test.content(|| {
        combined_target(enabled, click_enabled, counts, interaction, requester);
    });
    test.frame();
    let point = PxPosition::from_f32_xy(10.0, 10.0);
    test.click_at(point);
    assert_eq!(counts.get(), (1, 0));
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.advance(Duration::from_millis(499));
    assert_eq!(counts.get(), (1, 0));
    test.advance(Duration::from_millis(1));
    assert_eq!(counts.get(), (1, 1));
    test.advance(Duration::from_secs(1));
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(counts.get(), (1, 1));
    assert!(!interaction.get().is_pressed());
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.run(|| requester.clear_focus());
    test.frame();
    assert!(!interaction.get().is_pressed());
    test.advance(Duration::from_secs(1));
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(counts.get(), (1, 1));
    test.run(|| click_enabled.set(false));
    test.frame();
    let (id, target) = node(&test, "Hold", Role::Button);
    assert!(!target.supports_action(Action::Click));
    assert!(target.supports_action(Action::ShowContextMenu));
    assert!(test.accessibility_action(id, Action::Focus));
    test.frame();
    assert_eq!(
        test.accessibility_snapshot()
            .expect("focus publication")
            .focus,
        id
    );
    test.press_key(NamedKey::Enter);
    test.frame();
    assert_eq!(counts.get(), (1, 1));
    test.press_key(NamedKey::ContextMenu);
    test.frame();
    assert_eq!(counts.get(), (1, 2));
    assert!(test.accessibility_action(id, Action::ShowContextMenu));
    test.frame();
    assert_eq!(counts.get(), (1, 3));
    test.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    test.run(|| enabled.set(false));
    test.frame();
    test.advance(Duration::from_secs(1));
    test.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    test.frame();
    assert_eq!(counts.get(), (1, 3));
    assert!(!interaction.get().is_pressed());
    assert!(
        !node(&test, "Hold", Role::Button)
            .1
            .supports_action(Action::ShowContextMenu)
    );
}

#[tessera]
fn selection_target(kind: u8, value: State<bool>, enabled: State<bool>) {
    let modifier = Modifier::new().size(Dp(80.0), Dp(80.0));
    let modifier = if kind == 0 {
        modifier.toggleable(ToggleableArgs {
            value: value.get(),
            on_value_change: CallbackWith::new(move |next| value.set(next)),
            enabled: enabled.get(),
            label: Some("Choice".into()),
            ..Default::default()
        })
    } else {
        modifier.selectable(SelectableArgs {
            selected: value.get(),
            on_click: Callback::new(move || value.set(true)),
            enabled: enabled.get(),
            role: Some(if kind == 1 {
                Role::Tab
            } else {
                Role::RadioButton
            }),
            label: Some("Choice".into()),
            ..Default::default()
        })
    };
    layout(Stack).modifier(modifier);
}

#[test]
fn toggleable_and_selectable_publish_distinct_states_and_disable_their_actions() {
    for (kind, role) in [(0, Role::CheckBox), (1, Role::Tab), (2, Role::RadioButton)] {
        let test = UiTest::new();
        let value = test.state(false);
        let enabled = test.state(true);
        let mut test = test.content(|| selection_target(kind, value, enabled));
        test.frame();
        let (id, _) = node(&test, "Choice", role);
        assert!(test.accessibility_action(id, Action::Focus));
        test.frame();
        assert_eq!(
            test.accessibility_snapshot()
                .expect("focus publication")
                .focus,
            id
        );
        assert!(!value.get());
        test.press_key(NamedKey::Space);
        test.frame();
        assert!(value.get());
        let (_, selected) = node(&test, "Choice", role);
        assert_eq!(selected.is_selected(), (kind == 1).then_some(true));
        assert_eq!(selected.toggled(), (kind != 1).then_some(Toggled::True));
        test.click_at(PxPosition::from_f32_xy(10.0, 10.0));
        assert_eq!(value.get(), kind != 0);
        test.run(|| enabled.set(false));
        test.frame();
        let (_, disabled) = node(&test, "Choice", role);
        assert!(disabled.is_disabled());
        assert!(!disabled.supports_action(Action::Click));
        assert!(!test.accessibility_action(id, Action::Click));
        test.click_at(PxPosition::from_f32_xy(10.0, 10.0));
        assert_eq!(value.get(), kind != 0);
    }
}
