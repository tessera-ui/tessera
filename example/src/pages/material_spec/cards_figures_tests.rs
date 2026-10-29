//! Native card article navigation and token group regressions.
//!
//! ## Usage
//!
//! Verify the four article pages and the per-variant token groups of the live
//! cards article.

use std::time::Duration;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, NodeId, Role},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn tokens_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                tokens();
            });
    });
}

fn renderer(fixture: fn()) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 1024,
            height: 1000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(6);
    renderer
}

fn labels(renderer: &HeadlessRenderer) -> Vec<String> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn node(renderer: &HeadlessRenderer, role: Role, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label} ({role:?})"))
}

fn click(renderer: &mut HeadlessRenderer, bounds: tessera_ui::accesskit::Rect) {
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(5);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn cards_token_selector_switches_container_and_outline_values() {
    let mut renderer = renderer(tokens_fixture);
    assert!(
        labels(&renderer)
            .iter()
            .any(|label| label == "Surface container low"),
        "the elevated card container token must be visible"
    );
    assert!(
        labels(&renderer).iter().any(|label| label == "#000000"),
        "the elevated card shadow color token must be visible"
    );

    let filled = node(&renderer, Role::Button, "Filled card tokens")
        .1
        .bounds()
        .unwrap();
    click(&mut renderer, filled);
    assert!(
        labels(&renderer)
            .iter()
            .any(|label| label == "Surface container highest"),
        "selecting the filled variant must swap the container token"
    );

    let outlined = node(&renderer, Role::Button, "Outlined card tokens")
        .1
        .bounds()
        .unwrap();
    click(&mut renderer, outlined);
    assert!(
        labels(&renderer)
            .iter()
            .any(|label| label == "Outline variant · 1dp"),
        "selecting the outlined variant must publish its outline token"
    );
    assert!(
        labels(&renderer)
            .iter()
            .any(|label| label == "3dp focus indicator at 0dp elevation"),
        "the outlined state tokens must keep the resting elevation"
    );
}
