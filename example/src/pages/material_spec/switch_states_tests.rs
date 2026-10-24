//! Native switch state artboard interaction regressions.
//!
//! ## Usage
//!
//! Preserve themed state controls and pointer mapping across responsive
//! scaling.

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, Role, Toggled, TreeId},
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
    tessera_components::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::states();
        });
    });
}

fn save_pixels(renderer: &HeadlessRenderer, name: &str) {
    if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
        let size = renderer.physical_size();
        tessera_ui::headless::write_png_file(
            std::path::PathBuf::from(directory).join(name),
            size.width,
            size.height,
            &renderer.read_pixels().unwrap(),
        )
        .unwrap();
    }
}

fn node(renderer: &HeadlessRenderer, label: &str) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
        .1
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = node(renderer, label).bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(60);
}

#[test]
#[ignore = "requires a native GPU adapter for switch state illustrations"]
fn switch_states_keep_twenty_independent_native_controls_and_scaled_pointer_targets() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 1024,
            height: 760,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    save_pixels(&renderer, "switch-states-canonical-1024.png");
    let tree = renderer.accessibility_snapshot().unwrap();
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Switch)
            .count(),
        20
    );
    let light = node(&renderer, "Light switch states").bounds().unwrap();
    let dark = node(&renderer, "Dark switch states").bounds().unwrap();
    assert_eq!(
        (light.x0, light.y0, light.width(), light.height()),
        (0.0, 0.0, 1024.0, 300.0)
    );
    assert_eq!((dark.y0, dark.height()), (300.0, 300.0));
    let control = node(&renderer, "Light Enabled selected switch state")
        .bounds()
        .unwrap();
    assert_eq!(
        (control.x0, control.y0, control.width(), control.height()),
        (132.0, 109.0, 52.0, 48.0)
    );
    click(&mut renderer, "Light Enabled selected switch state");
    assert_eq!(
        node(&renderer, "Light Enabled selected switch state").toggled(),
        Some(Toggled::False)
    );
    assert_eq!(
        node(&renderer, "Dark Enabled selected switch state").toggled(),
        Some(Toggled::True)
    );
    let pointer = PxPosition::from_f32_xy(332.0, 145.0);
    renderer.pointer_move(MOUSE_POINTER_ID, pointer);
    renderer.pointer_press(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.render_frames(60);
    assert_eq!(
        node(&renderer, "Light Hovered selected switch state").toggled(),
        Some(Toggled::False),
        "the decorative hand must not intercept a real switch press"
    );
    assert_eq!(
        node(&renderer, "Dark Hovered selected switch state").toggled(),
        Some(Toggled::True)
    );
    click(&mut renderer, "Light Disabled selected switch state");
    assert_eq!(
        node(&renderer, "Light Disabled selected switch state").toggled(),
        Some(Toggled::True)
    );
    let (id, _) = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("Light Focused selected switch state"))
        .unwrap();
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None
    }));
    renderer.render_frames(4);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
    assert_eq!(
        node(&renderer, "Light Focused selected switch state").toggled(),
        Some(Toggled::True),
        "requesting accessibility focus must not toggle the switch"
    );
    renderer.resize(512, 760);
    renderer.render_frames(8);
    save_pixels(&renderer, "switch-states-narrow-512.png");
    let narrow = node(&renderer, "Light Enabled selected switch state")
        .bounds()
        .unwrap();
    assert_eq!(
        (narrow.x0, narrow.y0, narrow.width(), narrow.height()),
        (66.0, 54.5, 26.0, 24.0)
    );
    click(&mut renderer, "Light Enabled selected switch state");
    assert_eq!(
        node(&renderer, "Light Enabled selected switch state").toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        node(&renderer, "Dark Enabled selected switch state").toggled(),
        Some(Toggled::True)
    );
}
