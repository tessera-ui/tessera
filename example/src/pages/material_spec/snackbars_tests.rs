//! Snackbar article queue and automatic dismissal regressions.
//!
//! ## Usage
//!
//! Verify consecutive notifications through the live article controls.

use std::time::Duration;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        queued();
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label} {role:?}"))
}

fn action(renderer: &mut HeadlessRenderer, label: &str, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: node(renderer, label, Role::Button).0,
        data: None,
    }));
    renderer.render_frames(2);
}

fn assert_message(renderer: &HeadlessRenderer, message: &str) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let statuses: Vec<_> = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Status)
        .collect();
    assert_eq!(statuses.len(), 1, "queued notifications must never stack");
    assert_eq!(statuses[0].1.label(), Some(message));
    assert!(statuses[0].1.bounds().unwrap().height() > 0.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn queued_article_message_only_timeout_advances_to_persistent_action_without_moving_focus() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 500,
            sample_count: 1,
            frame_time: Duration::from_millis(100),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(3);
    assert_message(&renderer, "Photo saved");
    action(&mut renderer, "Next snackbar action", Action::Click);
    action(&mut renderer, "Next snackbar dismiss", Action::Click);
    action(&mut renderer, "Next snackbar two lines", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Focus);
    let origin = node(&renderer, "Enqueue snackbar", Role::Button).0;
    renderer.render_frames(50);
    assert_message(&renderer, "Photo saved");
    node(&renderer, "Undo", Role::Button);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    action(&mut renderer, "Undo", Action::Click);
    assert_message(&renderer, "Your photo was saved.\nIt is ready to share.");
    action(&mut renderer, "Next snackbar action", Action::Click);
    action(&mut renderer, "Next snackbar two lines", Action::Click);
    action(&mut renderer, "Enqueue snackbar", Action::Click);
    renderer.render_frames(20);
    assert_message(&renderer, "Your photo was saved.\nIt is ready to share.");
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);

    renderer.render_frames(30);
    assert_message(&renderer, "Photo saved");
    node(&renderer, "Undo", Role::Button);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, origin);
    renderer.render_frames(120);
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
