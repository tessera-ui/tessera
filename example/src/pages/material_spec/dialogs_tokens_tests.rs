use std::time::Duration;

use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _,
    theme::material_theme,
};
use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, Modifier,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, Toggled, TreeId},
    tessera,
};

use super::tokens;

#[tessera]
fn fixture() {
    material_theme().child(|| {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(tokens);
            });
    });
}

fn nodes(renderer: &HeadlessRenderer) -> Vec<(NodeId, Node)> {
    renderer
        .accessibility_snapshot()
        .expect("dialog token semantics")
        .nodes
}

fn activate(renderer: &mut HeadlessRenderer, target_node: NodeId) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(6);
}

fn click_button(renderer: &mut HeadlessRenderer, label: &str) {
    let target = nodes(renderer)
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing dialog token button {label}"))
        .0;
    activate(renderer, target);
}

fn contains_text(renderer: &HeadlessRenderer, expected: &str) -> bool {
    nodes(renderer).iter().any(|(_, node)| {
        node.label().is_some_and(|label| label.contains(expected))
            || node.value().is_some_and(|value| value.contains(expected))
    })
}

fn menu_options(renderer: &HeadlessRenderer) -> Vec<(NodeId, Node)> {
    nodes(renderer)
        .into_iter()
        .filter(|(_, node)| node.role() == Role::MenuItemRadio)
        .collect()
}

fn select(renderer: &mut HeadlessRenderer, label: &str) {
    click_button(renderer, "Dialog token set selector");
    let options = menu_options(renderer);
    assert_eq!(options.len(), 2, "both dialog token sets are offered");
    let target = options
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing token set {label}"))
        .0;
    activate(renderer, target);
    assert!(
        menu_options(renderer).is_empty(),
        "selection closes the menu"
    );
}

#[test]
#[ignore = "Requires a native GPU adapter for actual token menus, tables and accordion controls"]
fn dialog_token_sets_switch_variants_and_accordion_controls_reveal_their_own_rows() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 1100,
            height: 2400,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("native dialog token renderer");
    renderer.render_frames(6);

    assert!(
        !contains_text(&renderer, "Dialog container color"),
        "token groups begin collapsed"
    );
    click_button(&mut renderer, "Enabled");
    assert!(contains_text(&renderer, "Dialog container color"));
    assert!(contains_text(&renderer, "#ECE6F0"));
    assert!(
        !contains_text(&renderer, "Surface container high · #ECE6F0"),
        "published hex values are not merged with inferred role names"
    );
    assert!(contains_text(
        &renderer,
        "Surface container high (inferred role)"
    ));
    assert!(
        !contains_text(&renderer, "Dialog action hover state layer opacity"),
        "other state groups stay collapsed"
    );

    click_button(&mut renderer, "Expand all");
    assert!(contains_text(
        &renderer,
        "Dialog action hover state layer opacity"
    ));
    assert!(contains_text(
        &renderer,
        "Dialog action pressed state layer opacity"
    ));

    select(&mut renderer, "Dialog - Full screen");
    assert!(
        !contains_text(&renderer, "Dialog container color"),
        "basic rows are replaced by the full-screen set"
    );
    assert!(
        !contains_text(&renderer, "Full screen dialog header container height"),
        "the full-screen set keeps its own collapsed groups"
    );
    click_button(&mut renderer, "Enabled");
    assert!(contains_text(
        &renderer,
        "Full screen dialog header container height"
    ));

    click_button(&mut renderer, "Dialog token set selector");
    for (_, node) in menu_options(&renderer) {
        assert_eq!(
            node.toggled(),
            Some(if node.label() == Some("Dialog - Full screen") {
                Toggled::True
            } else {
                Toggled::False
            })
        );
    }
    let basic = menu_options(&renderer)
        .into_iter()
        .find(|(_, node)| node.label() == Some("Dialog - Basic"))
        .unwrap()
        .0;
    activate(&mut renderer, basic);
    assert!(
        contains_text(&renderer, "Dialog action hover state layer opacity"),
        "returning to the basic set restores its expanded groups"
    );
    click_button(&mut renderer, "Collapse all");
    assert!(!contains_text(&renderer, "Dialog container color"));
}
