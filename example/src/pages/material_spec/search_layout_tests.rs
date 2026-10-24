//! Search configuration email result interaction regressions.
//!
//! ## Usage
//!
//! Verify native filtering and navigation in the full-screen divided example.

use std::time::Duration;

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
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
        configuration_layout(true, true);
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

fn click(renderer: &mut HeadlessRenderer, label: &str, role: Role) {
    let bounds = node(renderer, label, role).1.bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(3);
}

fn result_count(renderer: &HeadlessRenderer) -> usize {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::ListItem)
        .count()
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn divided_full_screen_configuration_filters_native_emails_and_preserves_activated_query() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 600,
            height: 600,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    assert_eq!(
        node(&renderer, "Search your messages", Role::TextInput)
            .1
            .value(),
        Some("Eli")
    );
    assert_eq!(
        result_count(&renderer),
        3,
        "initial email configuration: {:?}",
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .map(|(_, node)| (node.role(), node.label(), node.value(), node.bounds()))
            .collect::<Vec<_>>()
    );
    node(&renderer, "Email search results", Role::List);
    let result = node(&renderer, "Eli, me", Role::ListItem)
        .1
        .bounds()
        .unwrap();
    let query = node(&renderer, "Search your messages", Role::TextInput)
        .1
        .bounds()
        .unwrap();
    assert!(result.y0 >= query.y1);
    click(&mut renderer, "Clear search", Role::Button);
    let input = node(&renderer, "Search your messages", Role::TextInput).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: input,
        data: None,
    }));
    renderer.render_frames(2);
    for character in "Movie".chars() {
        renderer.inject_text(&character.to_string());
    }
    renderer.render_frames(3);
    assert_eq!(
        node(&renderer, "Search your messages", Role::TextInput)
            .1
            .value(),
        Some("Movie")
    );
    assert_eq!(result_count(&renderer), 1);
    node(&renderer, "Email search suggestions", Role::List);
    click(&mut renderer, "Zita, Odette, Dagmar", Role::ListItem);
    node(&renderer, "Opened message: Movie marathon", Role::Label);
    node(&renderer, "Email search results", Role::List);
    assert_eq!(
        node(&renderer, "Search your messages", Role::TextInput)
            .1
            .value(),
        Some("Movie")
    );
    click(&mut renderer, "Clear search", Role::Button);
    assert_eq!(
        node(&renderer, "Search your messages", Role::TextInput)
            .1
            .value()
            .unwrap_or_default(),
        ""
    );
    assert_eq!(result_count(&renderer), 3);
    node(&renderer, "Email search suggestions", Role::List);
    click(&mut renderer, "Close search", Role::Button);
    assert_eq!(result_count(&renderer), 0);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .all(|(_, node)| node.label() != Some("Close search"))
    );
    assert_eq!(
        node(&renderer, "Search your messages", Role::TextInput)
            .1
            .value()
            .unwrap_or_default(),
        ""
    );
}
