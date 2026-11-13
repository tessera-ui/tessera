use tessera_components::md3::theme::material_theme;
use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, Node, NodeId},
    testing::UiTest,
};

use super::*;

#[tessera]
fn fixture() {
    material_theme().child(|| {
        configuration_layout(true, true);
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

fn click(renderer: &mut UiTest, label: &str, role: Role) {
    let bounds = node(renderer, label, role).1.bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(3);
}

fn result_count(renderer: &UiTest) -> usize {
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
    let mut renderer = UiTest::new()
        .viewport(600, 600)
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(5);
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
    assert!(renderer.accessibility_action(input, Action::Focus));
    renderer.frames(2);
    for character in "Movie".chars() {
        renderer.inject_text(&character.to_string());
    }
    renderer.frames(3);
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
