use tessera_components::theme::material_theme;
use tessera_ui::{
    accesskit::{Action, Node, NodeId, Role, Toggled},
    testing::UiTest,
};

use super::*;

#[tessera]
fn fixture() {
    material_theme().child(|| {
        dialogs_spec();
    });
}

fn node(renderer: &UiTest, label: &str, role: Role) -> (NodeId, Node) {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    let (id, node) = snapshot
        .find(&crate::res::I18N.text(label), role)
        .unwrap_or_else(|| panic!("missing {label} {role:?}"));
    (id, node.clone())
}

fn exists(renderer: &UiTest, label: &str, role: Role) -> bool {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.role() == role && crate::has_label(&node, label))
}

fn action(renderer: &mut UiTest, label: &str, role: Role, action: Action) {
    assert!(renderer.accessibility_action(node(renderer, label, role).0, action));
    renderer.frames(20);
}

fn edit(renderer: &mut UiTest, label: &str, value: &str) {
    action(renderer, label, Role::TextInput, Action::Focus);
    for character in value.chars() {
        renderer.inject_text(&character.to_string());
    }
    renderer.frames(3);
}

fn assert_default_draft(renderer: &UiTest) {
    for (label, value) in [
        ("Event name", ""),
        ("From", "Mon, May 18"),
        ("To", "Mon, May 18"),
        ("Time zone", "GMT-07:00"),
    ] {
        assert_eq!(
            node(renderer, label, Role::TextInput)
                .1
                .value()
                .unwrap_or_default(),
            value
        );
    }
    assert_eq!(
        node(renderer, "All-day", Role::CheckBox).1.toggled(),
        Some(Toggled::False)
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn event_dialog_keeps_edits_when_discard_is_cancelled_and_resets_after_discard_or_save() {
    let mut renderer = UiTest::new()
        .viewport(900, 1000)
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(5);
    action(&mut renderer, "Overview", Role::Tab, Action::Click);
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Focus,
    );
    let origin = node(&renderer, "Open full-screen dialog", Role::Button).0;
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    node(&renderer, "New event", Role::Dialog);
    assert_default_draft(&renderer);
    edit(&mut renderer, "Event name", "Planning 🦀");
    for (label, prefix) in [
        ("From", "Start: "),
        ("To", "End: "),
        ("Time zone", "Zone: "),
    ] {
        edit(&mut renderer, label, prefix);
    }
    action(&mut renderer, "All-day", Role::CheckBox, Action::Click);
    action(&mut renderer, "Close dialog", Role::Button, Action::Focus);
    let close = node(&renderer, "Close dialog", Role::Button).0;
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    node(&renderer, "Discard changes?", Role::AlertDialog);
    action(&mut renderer, "Keep editing", Role::Button, Action::Click);
    assert!(!exists(&renderer, "Discard changes?", Role::AlertDialog));
    node(&renderer, "New event", Role::Dialog);
    assert_eq!(
        node(&renderer, "Event name", Role::TextInput).1.value(),
        Some("Planning 🦀")
    );
    for (label, value) in [
        ("From", "Start: Mon, May 18"),
        ("To", "End: Mon, May 18"),
        ("Time zone", "Zone: GMT-07:00"),
    ] {
        assert_eq!(
            node(&renderer, label, Role::TextInput).1.value(),
            Some(value)
        );
    }
    assert_eq!(
        node(&renderer, "All-day", Role::CheckBox).1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, close);
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    node(&renderer, "Discard changes?", Role::AlertDialog);
    action(&mut renderer, "Discard", Role::Button, Action::Click);
    assert!(!exists(&renderer, "New event", Role::Dialog));
    assert!(!exists(&renderer, "Changes saved", Role::Label));
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    assert_default_draft(&renderer);
    edit(&mut renderer, "Event name", "Release review");
    for (label, prefix) in [
        ("From", "Start: "),
        ("To", "End: "),
        ("Time zone", "Zone: "),
    ] {
        edit(&mut renderer, label, prefix);
    }
    action(&mut renderer, "All-day", Role::CheckBox, Action::Click);
    action(&mut renderer, "Save", Role::Button, Action::Click);
    assert!(!exists(&renderer, "New event", Role::Dialog));
    node(&renderer, "Changes saved", Role::Label);
    action(
        &mut renderer,
        "Open full-screen dialog",
        Role::Button,
        Action::Click,
    );
    assert_default_draft(&renderer);
    action(&mut renderer, "Close dialog", Role::Button, Action::Click);
    assert!(!exists(&renderer, "Discard changes?", Role::AlertDialog));
    assert!(!exists(&renderer, "New event", Role::Dialog));
}
