use std::time::Duration;

use tessera_foundation::modifier::{InteractionState, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, FocusRequester, KeyboardEvent, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    modifier::FocusModifierExt as _,
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::{column::column, modifier::ModifierExt as _};

use super::{super::text::text, button, button_scope};

#[derive(Clone, Copy)]
struct Controls {
    enabled: State<bool>,
    show_default: State<bool>,
    ordinary: State<u32>,
    primary: State<u32>,
    interaction: State<InteractionState>,
    focus: FocusRequester,
    sibling_focus: FocusRequester,
}

#[tessera]
fn fixture(probe: TestProbe<Option<Controls>>) {
    let enabled = remember(|| true);
    let show_default = remember(|| true);
    let ordinary = remember(|| 0);
    let primary = remember(|| 0);
    let interaction = remember(InteractionState::new);
    let focus = remember(FocusRequester::new).get();
    let sibling_focus = remember(FocusRequester::new).get();
    probe.set(Some(Controls {
        enabled,
        show_default,
        ordinary,
        primary,
        interaction,
        focus,
        sibling_focus,
    }));
    button_scope().child(move |scope| {
        column()
            .modifier(Modifier::new().background(Color::WHITE))
            .children(move || {
                button("Cancel")
                    .scope(scope)
                    .enabled(enabled.get())
                    .interaction_state(interaction)
                    .focus_requester(focus)
                    .on_click(move || ordinary.set(ordinary.get() + 1))
                    .modifier(Modifier::new().width(Dp(100.0)).semantics(SemanticsArgs {
                        test_tag: Some("ordinary".into()),
                        ..Default::default()
                    }));
                if show_default.get() {
                    button("Save")
                        .scope(scope)
                        .default_action(true)
                        .enabled(enabled.get())
                        .on_click(move || primary.set(primary.get() + 1))
                        .modifier(Modifier::new().width(Dp(100.0)).semantics(SemanticsArgs {
                            test_tag: Some("primary".into()),
                            ..Default::default()
                        }));
                }
                text("Other content")
                    .modifier(Modifier::new().focus_requester(sibling_focus).focusable());
            });
    });
}

#[test]
fn pointer_press_release_cancel_and_disabled_transitions_use_the_button_input_path() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 160)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    let position = PxPosition::from_f32_xy(50.0, 12.0);
    ui.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    ui.frames(1);
    assert!(controls.interaction.with(InteractionState::is_pressed));
    assert_eq!(controls.ordinary.get(), 0);
    ui.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    ui.frames(1);
    assert!(!controls.interaction.with(InteractionState::is_pressed));
    assert_eq!(controls.ordinary.get(), 1);
    ui.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    ui.frames(1);
    ui.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(200.0, 100.0));
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(200.0, 100.0),
        PressKeyEventType::Left,
    );
    ui.frames(1);
    assert_eq!(controls.ordinary.get(), 1);
    controls.enabled.set(false);
    ui.frames(1);
    ui.click_at(position);
    assert_eq!(controls.ordinary.get(), 1);
    assert!(!ui.run(|| controls.focus.is_focused()));
}

fn check_scoped_keyboard() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    assert!(
        ui.run(|| controls.focus.is_focused()),
        "keyboard fixture must own focus before input"
    );
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert!(
        ui.run(|| controls.focus.is_focused()),
        "key press must preserve focus"
    );
    assert_eq!(controls.ordinary.get(), 0);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    assert_eq!(controls.ordinary.get(), 0);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.ordinary.get(), 1);
    assert!(!controls.interaction.with(InteractionState::is_pressed));
    assert!(ui.run(|| controls.focus.is_focus_visible()));
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.primary.get(), 1);
    let mut repeated = KeyboardEvent::named(NamedKey::Enter, ElementState::Pressed);
    repeated.repeat = true;
    ui.inject_key_event(repeated);
    ui.frames(1);
    assert_eq!(controls.primary.get(), 2);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.primary.get(), 2);
    assert_eq!(controls.ordinary.get(), 1);
    ui.run(|| controls.sibling_focus.request_focus());
    ui.frames(1);
    assert!(ui.run(|| controls.sibling_focus.is_focused()));
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.primary.get(), 3);
    controls.show_default.set(false);
    ui.frames(1);
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.primary.get(), 3);
    assert_eq!(controls.ordinary.get(), 1);
}

fn check_keyboard_cancellation() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frame();
    let controls = probe.get().unwrap();
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
    assert_eq!(controls.ordinary.get(), 0);
    assert!(!controls.interaction.with(InteractionState::is_pressed));
    controls.enabled.set(true);
    ui.frames(1);
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.set_key_modifiers(tessera_ui::winit::keyboard::ModifiersState::CONTROL);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    ui.set_key_modifiers(tessera_ui::winit::keyboard::ModifiersState::empty());
    assert_eq!(controls.ordinary.get(), 0);
    ui.press_key(NamedKey::Space);
    ui.frames(1);
    ui.run(|| controls.sibling_focus.request_focus());
    ui.frames(1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.ordinary.get(), 0);
    ui.run(|| controls.focus.request_focus());
    ui.frames(1);
    ui.press_key(NamedKey::Enter);
    ui.frames(1);
    assert_eq!(controls.primary.get(), 1);
    controls.show_default.set(false);
    ui.frames(1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frames(1);
    assert_eq!(controls.primary.get(), 1);
    assert_eq!(controls.ordinary.get(), 0);
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch"]
fn focused_space_activates_on_release_and_return_targets_the_scoped_default() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::button::tests::focused_space_activates_on_release_and_return_targets_the_scoped_default",
        check_scoped_keyboard,
    );
}

#[test]
#[ignore = "requires a native GPU adapter for keyboard dispatch"]
fn disabling_held_space_and_removing_the_default_prevent_later_activation() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::button::tests::disabling_held_space_and_removing_the_default_prevent_later_activation",
        check_keyboard_cancellation,
    );
}

fn check_accessibility_and_pixels() {
    use tessera_ui::accesskit::{Action, Role};

    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 120)
        .rendered(crate::TesseraComponents)
        .content(|| fixture(probe.clone()));
    ui.frames(3);
    let controls = probe.get().unwrap();
    let (id, ordinary) = ui.accessibility_node("Cancel");
    assert_eq!(ordinary.role(), Role::Button);
    assert!(ordinary.supports_action(Action::Click));
    assert_eq!(
        ui.accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| !node.is_hidden() && node.label() == Some("Cancel"))
            .count(),
        1
    );
    let initial = ui.read_pixels();
    ui.pointer_press(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(50.0, 12.0),
        PressKeyEventType::Left,
    );
    ui.frames(2);
    assert_ne!(initial, ui.read_pixels());
    ui.pointer_release(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(50.0, 12.0),
        PressKeyEventType::Left,
    );
    ui.frames(2);
    assert_eq!(controls.ordinary.get(), 1);
    assert_eq!(initial, ui.read_pixels());
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frames(2);
    assert_eq!(controls.ordinary.get(), 1);
    assert_ne!(initial, ui.read_pixels());
    let focused = ui.read_pixels();
    ui.press_key(NamedKey::Enter);
    ui.frames(2);
    assert_ne!(focused, ui.read_pixels());
    assert_eq!(controls.primary.get(), 1);
    ui.advance(Duration::from_millis(100));
    ui.frames(1);
    assert_eq!(focused, ui.read_pixels());
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frames(2);
    assert_eq!(controls.primary.get(), 1);
    assert_eq!(focused, ui.read_pixels());
    ui.press_key(NamedKey::Space);
    ui.frames(2);
    assert_ne!(focused, ui.read_pixels());
    assert_eq!(controls.ordinary.get(), 1);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Space,
        ElementState::Released,
    ));
    ui.frames(2);
    assert_eq!(controls.ordinary.get(), 2);
    assert_eq!(focused, ui.read_pixels());
    assert!(ui.accessibility_action(id, Action::Click));
    ui.frames(2);
    assert_eq!(controls.ordinary.get(), 3);
    controls.enabled.set(false);
    ui.frames(2);
    let (id, disabled) = ui.accessibility_node("Cancel");
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    ui.accessibility_action(id, Action::Click);
    ui.frames(1);
    assert_eq!(controls.ordinary.get(), 3);
    assert_ne!(initial, ui.read_pixels());
}

#[tessera]
fn backdrop_fixture() {
    use super::super::theme::{Appearance, HigTheme, hig_theme};
    use crate::{alignment::Alignment, boxed::boxed};

    boxed().alignment(Alignment::TopStart).children(|| {
        for (index, (gray, dark)) in [
            (1.0, false),
            (0.8900054, false),
            (0.010022826, true),
            (0.051269457, true),
        ]
        .into_iter()
        .enumerate()
        {
            hig_theme()
                .theme(HigTheme {
                    appearance: if dark {
                        Appearance::Dark
                    } else {
                        Appearance::Light
                    },
                })
                .child(move || {
                    for (row, enabled) in [true, false].into_iter().enumerate() {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(index as f64 * 100.0), Dp(row as f64 * 40.0))
                                    .size(Dp(100.0), Dp(24.0))
                                    .background(Color::from_rgb(gray, gray, gray)),
                            )
                            .children(move || {
                                button("Save")
                                    .enabled(enabled)
                                    .window_active(false)
                                    .modifier(Modifier::new().fill_max_size());
                            });
                    }
                });
        }
    });
}

fn check_backdrop_pixels() {
    let mut ui = UiTest::new()
        .viewport(800, 160)
        .scale_factor(2.0)
        .samples(4)
        .rendered(crate::TesseraComponents)
        .content(backdrop_fixture);
    ui.frames(3);
    let pixels = ui.read_pixels();
    for (column, values) in [[236, 245], [224, 232], [43, 33], [79, 72]]
        .into_iter()
        .enumerate()
    {
        for (row, expected) in values.into_iter().enumerate() {
            let offset = ((row * 80 + 24) * 800 + column * 200 + 10) * 4;
            for channel in 0..3 {
                assert!(
                    (i32::from(pixels[offset + channel]) - expected).abs() <= 2,
                    "backdrop column {column}, enabled row {row}, channel {channel}: {} != {expected}",
                    pixels[offset + channel]
                );
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn inactive_button_tints_follow_light_and_dark_backdrops() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::button::tests::inactive_button_tints_follow_light_and_dark_backdrops",
        check_backdrop_pixels,
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn pointer_and_accessibility_update_pressed_disabled_default_and_focus_pixels() {
    crate::test_utils::run_isolated_test(
        "experiment::hig::button::tests::pointer_and_accessibility_update_pressed_disabled_default_and_focus_pixels",
        check_accessibility_and_pixels,
    );
}
