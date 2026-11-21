use tessera_ui::{
    KeyboardEvent, MOUSE_POINTER_ID, PressKeyEventType, PxPosition, State,
    accesskit::{Action, Role},
    provide_context, remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::{column::column, experiment::hig::theme::Appearance};

use super::{OpenUrlAction, link, link_color};

#[test]
fn appearance_link_colors_match_macos_27_appkit_semantic_colors() {
    let expected = [
        tessera_ui::Color::from_rgb(0.0, 0.13843162, 0.701102),
        tessera_ui::Color::from_rgb(0.052860647, 0.33245155, 1.0),
    ];
    for (actual, expected) in [Appearance::Light, Appearance::Dark]
        .map(link_color)
        .into_iter()
        .zip(expected)
    {
        assert!(
            actual
                .to_array()
                .into_iter()
                .zip(expected.to_array())
                .all(|(actual, expected)| (actual - expected).abs() < 0.000001),
            "AppKit link color mismatch: {actual:?} != {expected:?}"
        );
    }
}

#[tessera]
fn fixture(probe: TestProbe<Option<(State<bool>, State<u32>, State<Vec<String>>)>>) {
    let enabled = remember(|| true);
    let opened = remember(|| 0);
    let overridden = remember(Vec::new);
    probe.set(Some((enabled, opened, overridden)));
    provide_context(
        OpenUrlAction::new(move |url| {
            assert_eq!(url, "https://example.com/help");
            opened.set(opened.get() + 1);
        }),
        move || {
            column().children(move || {
                link("Learn more", "https://example.com/help")
                    .enabled(enabled.get())
                    .accessibility_label("Open help")
                    .accessibility_description("Documentation");
                link("Release notes", "app://notes")
                    .on_open(move |url| overridden.with_mut(|urls| urls.push(url)));
            });
        },
    );
}

#[test]
fn link_exposes_its_accessible_name_and_cancels_pointer_activation_when_disabled() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().content(|| fixture(probe.clone()));
    ui.frame();
    let (enabled, opened, overridden) = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Open help");
    assert_eq!(node.role(), Role::Link);
    assert_eq!(node.description(), Some("Documentation"));
    assert!(node.supports_action(Action::Click));
    let bounds = node.bounds().unwrap();
    let inside = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    ui.click_at(inside);
    assert_eq!(opened.get(), 1);

    ui.pointer_press(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.frame();
    assert_eq!(opened.get(), 1);
    enabled.set(false);
    ui.frame();
    ui.pointer_release(MOUSE_POINTER_ID, inside, PressKeyEventType::Left);
    ui.click_at(inside);
    let (_, node) = ui.accessibility_node("Open help");
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Click));
    ui.accessibility_action(id, Action::Click);
    ui.frame();
    assert_eq!(opened.get(), 1);
    let (notes, _) = ui.accessibility_node("Release notes");
    assert!(ui.accessibility_action(notes, Action::Click));
    assert_eq!(overridden.get(), ["app://notes"]);
    assert_eq!(opened.get(), 1, "on_open replaces the action in context");
}

#[test]
fn accessible_focus_does_not_open_link_but_keyboard_activation_does() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().content(|| fixture(probe.clone()));
    ui.frame();
    let (_, opened, _) = probe.get().unwrap();
    let (id, _) = ui.accessibility_node("Open help");
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frame();
    assert_eq!(opened.get(), 0);

    for (index, key) in [NamedKey::Enter, NamedKey::Space].into_iter().enumerate() {
        ui.press_key(key);
        ui.inject_key_event(KeyboardEvent::named(key, ElementState::Released));
        ui.frame();
        assert_eq!(opened.get(), index as u32 + 1);
    }
}
