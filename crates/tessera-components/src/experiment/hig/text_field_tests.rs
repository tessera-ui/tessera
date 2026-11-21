use tessera_ui::{
    Dp, KeyboardEvent, PxPosition, State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::{event::ElementState, keyboard::NamedKey},
};

use super::{
    BULLET, TextFieldController, apply_secure_edit, search_field, secure_field, text_editor,
    text_field,
};

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

#[test]
fn secure_edits_map_bullet_changes_back_onto_the_real_text() {
    let b = |count: usize| std::iter::repeat_n(BULLET, count).collect::<String>();
    assert_eq!(
        apply_secure_edit("pas", &b(3), &format!("{}s", b(3)), (3, 3)),
        "pass"
    );
    assert_eq!(
        apply_secure_edit("pass", &b(4), &format!("{}X{}", b(2), b(2)), (2, 2)),
        "paXss",
        "typing in the middle"
    );
    assert_eq!(
        apply_secure_edit("pass", &b(4), &b(3), (4, 4)),
        "pas",
        "backspace at the end"
    );
    assert_eq!(
        apply_secure_edit("pass", &b(4), &b(3), (2, 2)),
        "pss",
        "backspace in the middle"
    );
    assert_eq!(
        apply_secure_edit("pass", &b(4), &b(2), (1, 3)),
        "ps",
        "deleting a selection"
    );
    assert_eq!(
        apply_secure_edit("secret", &b(6), &format!("{}new{}", b(2), b(1)), (2, 5)),
        "senewt",
        "replacing a selection with pasted text"
    );
}

#[tessera]
fn secure_fixture(probe: TestProbe<Option<State<Vec<String>>>>) {
    let values = remember(Vec::new);
    probe.set(Some(values));
    secure_field()
        .placeholder("Password")
        .on_change(move |value: String| {
            values.with_mut(|values| values.push(value.clone()));
            value
        });
}

fn focus_center(ui: &mut UiTest, label: &str) {
    let bounds = ui
        .accessibility_node(label)
        .1
        .bounds()
        .expect("field bounds");
    ui.click_at(PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    ));
    ui.frame();
}

#[test]
fn secure_fields_report_real_text_but_show_and_publish_only_bullets() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 80)
        .content(|| secure_fixture(probe.clone()));
    ui.frame();
    let values = probe.get().expect("fixture state");
    focus_center(&mut ui, "Password");
    ui.inject_text("pa");
    ui.frame();
    ui.inject_text("ss");
    ui.frame();
    assert_eq!(values.get().last().map(String::as_str), Some("pass"));
    ui.press_key(NamedKey::Backspace);
    ui.frame();
    assert_eq!(values.get().last().map(String::as_str), Some("pas"));
    let (_, node) = ui.accessibility_node("Password");
    assert_eq!(node.role(), Role::PasswordInput);
    assert!(
        node.value().is_none_or(|value| !value.contains('p')),
        "the secret is never published"
    );
}

#[tessera]
fn search_fixture(probe: TestProbe<Option<(State<Vec<String>>, State<TextFieldController>)>>) {
    let values = remember(Vec::new);
    let controller = remember(|| TextFieldController::new(Dp(13.0), Some(Dp(16.0))));
    probe.set(Some((values, controller)));
    search_field()
        .controller(controller)
        .modifier(tessera_ui::Modifier::new())
        .on_change(move |value: String| {
            values.with_mut(|values| values.push(value.clone()));
            value
        });
}

#[test]
fn the_search_clear_button_empties_the_query() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 80)
        .content(|| search_fixture(probe.clone()));
    ui.frame();
    let (values, controller) = probe.get().expect("fixture state");
    focus_center(&mut ui, "Search");
    ui.inject_text("tessera");
    ui.frames(2);
    assert_eq!(
        controller.with(|editor| editor.text().to_string()),
        "tessera"
    );
    let bounds = ui
        .accessibility_node("Search")
        .1
        .bounds()
        .expect("field bounds");
    ui.click_at(PxPosition::from_f32_xy(
        (bounds.x1 - 10.0) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    ));
    ui.frames(2);
    assert_eq!(controller.with(|editor| editor.text().to_string()), "");
    assert_eq!(values.get().last().map(String::as_str), Some(""));
}

#[tessera]
fn editor_fixture(probe: TestProbe<Option<State<TextFieldController>>>) {
    let controller = remember(|| TextFieldController::new(Dp(13.0), Some(Dp(16.0))));
    probe.set(Some(controller));
    text_editor()
        .controller(controller)
        .accessibility_label("Notes");
}

#[test]
fn text_editors_take_several_lines() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(300, 200)
        .content(|| editor_fixture(probe.clone()));
    ui.frame();
    let controller = probe.get().expect("fixture state");
    let bounds = ui
        .accessibility_node("Notes")
        .1
        .bounds()
        .expect("editor bounds");
    assert!(bounds.y1 - bounds.y0 >= 80.0);
    focus_center(&mut ui, "Notes");
    ui.inject_text("first");
    ui.frame();
    ui.press_key(NamedKey::Enter);
    ui.frame();
    ui.inject_text("second");
    ui.frame();
    assert_eq!(
        controller.with(|editor| editor.text().to_string()),
        "first\nsecond"
    );
}
