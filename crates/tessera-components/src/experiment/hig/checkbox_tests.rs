use std::time::Duration;

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, Role, Toggled},
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
    CheckboxState, checkbox,
};

#[test]
fn label_is_placed_beside_the_glyph_at_fractional_density() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::checkbox::tests::label_is_placed_beside_the_glyph_at_fractional_density",
        || {
            let frame = UiTest::new()
                .viewport(300, 100)
                .scale_factor(1.5)
                .content(|| {
                    crate::boxed::boxed()
                        .alignment(crate::alignment::Alignment::TopStart)
                        .children(|| {
                            checkbox(CheckboxState::Off)
                                .label("Option")
                                .modifier(Modifier::new().size(Dp(160.0), Dp(20.0)));
                        });
                })
                .frame();
            let glyph = frame.node("choice_glyph").bounds();
            let text = frame.node("text_node").bounds();
            assert_eq!(glyph.width.0, 24);
            assert_eq!(glyph.height.0, 24);
            assert_eq!(glyph.y.0, 3);
            assert_eq!(text.x.0, glyph.x.0 + glyph.width.0);
            assert!(
                text.width.0 > 50,
                "title must retain its measured text width instead of being ellipsized inside the glyph"
            );
            assert_eq!(text.y.0, 3);
        },
    );
}

#[derive(Clone, Copy)]
struct Controls {
    value: State<CheckboxState>,
    enabled: State<bool>,
    mixed: State<bool>,
    changes: State<Vec<CheckboxState>>,
    focus: FocusRequester,
    other_focus: FocusRequester,
    submits: State<u32>,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let value = remember(|| CheckboxState::Off);
    let enabled = remember(|| true);
    let mixed = remember(|| false);
    let changes = remember(Vec::new);
    let focus = remember(FocusRequester::new).get();
    let other_focus = remember(FocusRequester::new).get();
    let submits = remember(|| 0);
    probe.set(Some(Controls {
        value,
        enabled,
        mixed,
        changes,
        focus,
        other_focus,
        submits,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                checkbox(value.get())
                    .label("Option")
                    .enabled(enabled.get())
                    .allows_mixed(mixed.get())
                    .focus_requester(focus)
                    .on_value_change(move |next| {
                        changes.with_mut(|changes| changes.push(next));
                        value.set(next);
                    })
                    .modifier(
                        Modifier::new()
                            .size(Dp(160.0), Dp(20.0))
                            .semantics(SemanticsArgs {
                                test_tag: Some("option".into()),
                                ..Default::default()
                            }),
                    );
                button("Save")
                    .scope(scope)
                    .default_action(true)
                    .on_click(move || submits.set(submits.get() + 1));
                text("Other").modifier(Modifier::new().focus_requester(other_focus).focusable());
            });
    });
}

#[test]
fn label_clicks_cycle_controlled_selection_and_read_external_updates() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 120)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let label = PxPosition::from_f32_xy(50.0, 10.0);
    ui.click_at(label);
    assert_eq!(controls.value.get(), CheckboxState::On);
    ui.click_at(label);
    assert_eq!(controls.value.get(), CheckboxState::Off);
    controls.mixed.set(true);
    ui.frames(1);
    for expected in [CheckboxState::Mixed, CheckboxState::On, CheckboxState::Off] {
        ui.click_at(label);
        assert_eq!(controls.value.get(), expected);
    }
    controls.value.set(CheckboxState::On);
    ui.frames(1);
    ui.click_at(label);
    assert_eq!(controls.value.get(), CheckboxState::Off);
    assert_eq!(
        controls.changes.get(),
        [
            CheckboxState::On,
            CheckboxState::Off,
            CheckboxState::Mixed,
            CheckboxState::On,
            CheckboxState::Off,
            CheckboxState::Off
        ]
    );
}

#[test]
fn dragging_out_and_back_tracks_the_press_and_disabling_cancels_activation() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 120)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let inside = PxPosition::from_f32_xy(50.0, 10.0);
    let outside = PxPosition::from_f32_xy(240.0, 100.0);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_move(MOUSE_POINTER_ID, outside);
    ui.frames(1);
    ui.pointer_move(MOUSE_POINTER_ID, inside);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
    ui.frames(1);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frames(1);
    ui.click_at(inside);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    assert!(!ui.run(|| controls.focus.is_focused()));
    controls.enabled.set(true);
    ui.frames(1);
    ui.click_at(inside);
    assert_eq!(
        controls.changes.get(),
        [CheckboxState::On, CheckboxState::Off]
    );
}

fn check_keyboard_and_semantics() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 120)
        .scale_factor(1.0)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    let initial = ui.read_pixels();
    let (id, node) = ui.accessibility_node("Option");
    assert_eq!(node.role(), Role::CheckBox);
    assert_eq!(node.toggled(), Some(Toggled::False));
    assert!(node.supports_action(Action::Click));
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert!(controls.changes.get().is_empty());
    assert!(ui.run(|| controls.focus.is_focused()));
    assert_ne!(initial, ui.read_pixels());
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.submits.get(), 1);
    assert!(controls.changes.get().is_empty());
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.advance(Duration::from_millis(120));
    ui.frames(1);
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Space);
    ui.frames(2);
    assert!(controls.changes.get().is_empty());
    assert_ne!(focused, ui.read_pixels());
    let mut repeated = KeyboardEvent::named(NamedKey::Space, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    assert!(controls.changes.get().is_empty());
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(2);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    assert_eq!(
        ui.accessibility_node("Option").1.toggled(),
        Some(Toggled::True)
    );
    let pixels = ui.read_pixels();
    let glyph = ui.frames(1).node("choice_glyph").bounds();
    let red = |x: i32, y: i32| pixels[(((glyph.y.0 + y) * 300 + glyph.x.0 + x) * 4) as usize];
    assert!(
        red(6, 11) > 200,
        "selected checkbox must paint its check stroke"
    );
    assert!(
        red(2, 8) < 30,
        "selected checkbox must paint its accent fill"
    );
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.set_key_modifiers(ModifiersState::CONTROL);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    ui.set_key_modifiers(ModifiersState::empty());
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.run(|| controls.other_focus.request_focus());
    ui.frames(1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.changes.get(), [CheckboxState::On]);
    controls.enabled.set(true);
    ui.frames(1);
    controls.mixed.set(true);
    controls.value.set(CheckboxState::Mixed);
    ui.frames(2);
    assert_eq!(
        ui.accessibility_node("Option").1.toggled(),
        Some(Toggled::Mixed)
    );
    let pixels = ui.read_pixels();
    let mark = (((glyph.y.0 + 8) * 300 + glyph.x.0 + 8) * 4) as usize;
    assert!(
        pixels[mark] > 200,
        "mixed checkbox must paint its horizontal dash"
    );
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.value.get(), CheckboxState::On);
    controls.value.set(CheckboxState::Mixed);
    controls.mixed.set(false);
    ui.frames(2);
    assert_eq!(
        ui.accessibility_node("Option").1.toggled(),
        Some(Toggled::True)
    );
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.value.get(), CheckboxState::Off);
    controls.enabled.set(false);
    ui.frames(2);
    let (_, disabled) = ui.accessibility_node("Option");
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    ui.accessibility_action(id, Action::Click);
    ui.frames(1);
    assert_eq!(controls.value.get(), CheckboxState::Off);
    assert_ne!(focused, ui.read_pixels());
    assert_eq!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some("Option"))
            .count(),
        1
    );
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch and pixels"]
fn keyboard_release_cancellation_default_routing_and_accessibility_use_the_control() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::checkbox::tests::keyboard_release_cancellation_default_routing_and_accessibility_use_the_control",
        check_keyboard_and_semantics,
    );
}
