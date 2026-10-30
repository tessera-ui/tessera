//! Interactive token-set selection and native preview regressions.
//!
//! ## Usage
//!
//! Keep token menus connected to readable values and real icon-button actions.

use tessera_components::{column::column, theme::material_theme};
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role, Toggled},
    tessera,
};

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        column().children(super::figure);
    });
}

fn node(renderer: &HeadlessRenderer, role: Role, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("Missing {role:?}: {label}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, node: Node) {
    let bounds = node.bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(20);
}

#[test]
#[ignore = "requires a native GPU adapter for token menu selection and icon-button input"]
fn token_sets_select_size_values_and_keep_the_preview_live() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 2000,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(20);
    let selector = node(&renderer, Role::Button, "Show Extra large size tokens");
    click(&mut renderer, selector);
    assert_eq!(
        node(&renderer, Role::Button, "Show Extra large size tokens").toggled(),
        Some(Toggled::True)
    );
    node(&renderer, Role::Cell, "136dp");
    node(&renderer, Role::Cell, "40dp");
    node(&renderer, Role::Cell, "184dp");
    let target = node(&renderer, Role::Button, "Token preview favorite");
    assert_eq!(target.toggled(), Some(Toggled::False));
    let bounds = target.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (136.0, 136.0));
    let disabled = node(&renderer, Role::Button, "Token preview disabled favorite");
    assert!(disabled.is_disabled());
    click(&mut renderer, target);
    assert_eq!(
        node(&renderer, Role::Button, "Token preview favorite").toggled(),
        Some(Toggled::True)
    );
    node(
        &renderer,
        Role::Label,
        "Filled preview · Selected · Activations: 1",
    );
    let shape = node(&renderer, Role::Button, "Use square resting shape");
    click(&mut renderer, shape);
    node(&renderer, Role::Button, "Use round resting shape");
}
