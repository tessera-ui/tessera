use tessera_ui::{
    Dp, KeyboardEvent, PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use super::{TextFieldController, text_field};

#[tessera]
fn fixture(probe: TestProbe<Option<(State<bool>, State<TextFieldController>, State<u32>)>>) {
    let enabled = remember(|| true);
    let submits = remember(|| 0);
    let controller = remember(|| {
        let mut controller = TextFieldController::new(Dp(13.0), Some(Dp(16.0)));
        controller.set_text("System settings");
        controller
    });
    probe.set(Some((enabled, controller, submits)));
    text_field()
        .controller(controller)
        .enabled(enabled.get())
        .placeholder("Enter a name")
        .accessibility_description("Text value")
        .on_submit(move || submits.set(submits.get() + 1));
}

#[test]
fn field_exposes_editable_text_semantics_and_preserves_its_value_when_disabled() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().content(|| fixture(probe.clone()));
    ui.frame();

    let (enabled, controller, _) = probe.get().unwrap();
    let (id, node) = ui.accessibility_node("Enter a name");
    assert_eq!(node.role(), Role::TextInput);
    assert_eq!(node.description(), Some("Text value"));
    assert_eq!(node.value(), Some("System settings"));
    assert!(node.supports_action(Action::Focus));
    assert!(ui.accessibility_action(id, Action::Focus));
    ui.frame();
    assert_eq!(
        controller.with(|controller| controller.text()),
        "System settings"
    );

    enabled.set(false);
    ui.frame();
    let (_, node) = ui.accessibility_node("Enter a name");
    assert!(node.is_disabled());
    assert!(!node.supports_action(Action::Focus));
    assert_eq!(node.value(), Some("System settings"));
    assert_eq!(
        controller.with(|controller| controller.text()),
        "System settings"
    );
}

#[test]
fn pointer_focus_ime_commit_and_submit_update_the_owned_editor() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().content(|| fixture(probe.clone()));
    ui.frame();
    let (_, controller, submits) = probe.get().unwrap();
    let (_, node) = ui.accessibility_node("Enter a name");
    let bounds = node.bounds().unwrap();
    let center = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    ui.click_at(center);
    ui.frame();
    assert!(controller.with(|editor| editor.focus_handler().is_focused()));
    ui.inject_text("x");
    ui.frame();
    assert_eq!(
        controller.with(|editor| editor.text().len()),
        "System settings".len() + 1
    );
    assert!(controller.with(|editor| editor.text().contains('x')));
    ui.press_key(NamedKey::Enter);
    ui.inject_key_event(KeyboardEvent::named(
        NamedKey::Enter,
        ElementState::Released,
    ));
    ui.frame();
    assert_eq!(submits.get(), 1);
}

#[tessera]
fn empty_field() {
    text_field()
        .placeholder("Enter a name")
        .accessibility_label("Name");
}

#[test]
fn empty_field_keeps_placeholder_out_of_its_accessible_value() {
    let mut ui = UiTest::new().content(empty_field);
    ui.frame();
    let (_, node) = ui.accessibility_node("Name");
    assert_eq!(node.role(), Role::TextInput);
    assert_eq!(node.value(), None);
    assert!(node.supports_action(Action::Focus));
}

#[tessera]
fn read_only_field(probe: TestProbe<Option<State<TextFieldController>>>) {
    let controller = remember(|| {
        let mut controller = TextFieldController::new(Dp(13.0), Some(Dp(16.0)));
        controller.set_text("Managed value");
        controller
    });
    probe.set(Some(controller));
    text_field()
        .controller(controller)
        .read_only(true)
        .accessibility_label("Managed value");
}

#[test]
fn read_only_field_keeps_its_value_when_focused_and_given_text_input() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new().content(|| read_only_field(probe.clone()));
    ui.frame();
    let controller = probe.get().unwrap();
    let (_, node) = ui.accessibility_node("Managed value");
    let bounds = node.bounds().unwrap();
    ui.click_at(PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    ));
    ui.frame();
    assert!(controller.with(|editor| editor.focus_handler().is_focused()));
    ui.inject_text(" changed");
    ui.frame();
    assert_eq!(controller.with(|editor| editor.text()), "Managed value");
    assert_eq!(
        ui.accessibility_node("Managed value").1.value(),
        Some("Managed value")
    );
}
