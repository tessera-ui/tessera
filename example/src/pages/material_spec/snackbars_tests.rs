use std::time::Duration;

use tessera_ui::{
    accesskit::{Action, Node, NodeId, Role},
    testing::UiTest,
};

use super::*;

#[tessera]
fn fixture() {
    material_theme().child(|| {
        queued();
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

fn action(renderer: &mut UiTest, label: &str, action: Action) {
    assert!(renderer.accessibility_action(node(renderer, label, Role::Button).0, action));
    renderer.frames(2);
}

fn assert_message(renderer: &UiTest, message: &str) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let statuses: Vec<_> = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Status)
        .collect();
    assert_eq!(statuses.len(), 1, "queued notifications must never stack");
    assert!(crate::text_matches(statuses[0].1.label(), message));
    assert!(statuses[0].1.bounds().unwrap().height() > 0.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn queued_article_message_only_timeout_advances_to_persistent_action_without_moving_focus() {
    let mut renderer = UiTest::new()
        .viewport(900, 500)
        .frame_step(Duration::from_millis(100))
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(3);
    assert_message(&renderer, "Photo saved");
    action(&mut renderer, "Next snackbar action", Action::Click);
    action(&mut renderer, "Next snackbar dismiss", Action::Click);
    action(&mut renderer, "Next snackbar two lines", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Focus);
    let origin = node(&renderer, "Enqueue snackbar", Role::Button).0;
    renderer.frames(50);
    assert_message(&renderer, "Photo saved");
    node(&renderer, "Undo", Role::Button);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    action(&mut renderer, "Undo", Action::Click);
    assert_message(&renderer, "Your photo was saved.\nIt is ready to share.");
    action(&mut renderer, "Next snackbar action", Action::Click);
    action(&mut renderer, "Next snackbar two lines", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Click);
    renderer.frames(20);
    assert_message(&renderer, "Your photo was saved.\nIt is ready to share.");
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    renderer.frames(30);
    assert_message(&renderer, "Photo saved");
    node(&renderer, "Undo", Role::Button);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
    renderer.frames(120);
    assert_message(&renderer, "Photo saved");
    action(&mut renderer, "Undo", Action::Click);
    node(&renderer, "Snackbar visible: false", Role::Label);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.role() != Role::Status)
    );
}

#[tessera]
fn configurations_fixture() {
    material_theme().child(|| {
        configurations();
    });
}

#[tessera]
fn inline_feedback_fixture() {
    material_theme().child(|| {
        inline_feedback();
    });
}

fn renderer<Output: 'static>(root: fn() -> Output) -> UiTest<'static> {
    let mut renderer = UiTest::new()
        .viewport(900, 1400)
        .frame_step(Duration::from_millis(100))
        .rendered(crate::TestPipelines)
        .content(root);
    renderer.frames(3);
    renderer
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn configuration_figure_uses_native_one_two_and_stacked_line_heights() {
    let renderer = renderer(configurations_fixture);
    for (name, height) in [
        ("Single line", 48.0),
        ("Single line with action", 48.0),
        ("Two lines", 68.0),
        ("Two lines with action", 68.0),
        ("Two lines with longer action", 120.0),
    ] {
        let bounds = node(&renderer, name, Role::Group).1.bounds().unwrap();
        assert_eq!(bounds.height(), height, "snackbar configuration {name}");
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn inline_feedback_keeps_saved_label_after_message_only_snackbar_times_out() {
    let mut renderer = renderer(inline_feedback_fixture);
    action(&mut renderer, "Save", Action::Click);
    assert_message(&renderer, "All changes saved");
    node(&renderer, "Saved", Role::Button);
    renderer.frames(60);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.role() != Role::Status),
        "the message-only snackbar must auto-dismiss"
    );
    node(&renderer, "Saved", Role::Button);
}
