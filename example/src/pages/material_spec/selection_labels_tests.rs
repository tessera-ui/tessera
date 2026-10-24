//! Native adjacent label placement and activation regressions.
//!
//! ## Usage
//!
//! Preserve independent label activation through canonical artboard scaling.

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
fn checkbox_fixture() {
    super::super::super::media::load_fonts();
    tessera_components::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::figure(false);
        });
    });
}

#[tessera]
fn switch_fixture() {
    super::super::super::media::load_fonts();
    tessera_components::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::figure(true);
        });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() == role)
        .unwrap_or_else(|| panic!("missing native {role:?} {label}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, x: f32, y: f32) {
    let position = PxPosition::from_f32_xy(x, y);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(60);
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

#[test]
#[ignore = "requires a native GPU adapter for adjacent label illustrations"]
fn adjacent_labels_activate_their_native_control_and_preserve_canonical_scaled_geometry() {
    for is_switch in [false, true] {
        let mut renderer = HeadlessRenderer::new(
            if is_switch {
                Box::new(switch_fixture)
            } else {
                Box::new(checkbox_fixture)
            },
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 676,
                height: 400,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let app = node(&renderer, "Adjacent labels app viewport", Role::Group)
            .bounds()
            .unwrap();
        assert_eq!(
            (app.x0, app.y0, app.width(), app.height()),
            (132.0, 0.0, 412.0, 316.0)
        );
        let (role, first, peer, last, initial, label_point, control_point) = if is_switch {
            (
                Role::Switch,
                "Permission manager",
                "Camera access",
                "Show password",
                (476.0, 104.0, 52.0, 48.0),
                (250.0, 128.0),
                (502.0, 200.0),
            )
        } else {
            (
                Role::CheckBox,
                "Pickles",
                "Tomato",
                "Cheese",
                (136.0, 112.0, 48.0, 48.0),
                (220.0, 136.0),
                (160.0, 192.0),
            )
        };
        let tree = renderer.accessibility_snapshot().unwrap();
        assert_eq!(
            tree.nodes
                .iter()
                .filter(|(_, node)| node.role() == role)
                .count(),
            if is_switch { 3 } else { 4 }
        );
        let bounds = node(&renderer, first, role).bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            initial
        );
        assert_eq!(node(&renderer, first, role).toggled(), Some(Toggled::True));
        assert_eq!(node(&renderer, last, role).toggled(), Some(Toggled::False));
        save_pixels(
            &renderer,
            if is_switch {
                "switch-labels-canonical-676.png"
            } else {
                "checkbox-labels-canonical-676.png"
            },
        );
        click(&mut renderer, label_point.0, label_point.1);
        assert_eq!(node(&renderer, first, role).toggled(), Some(Toggled::False));
        assert_eq!(node(&renderer, peer, role).toggled(), Some(Toggled::True));
        let tree = renderer.accessibility_snapshot().unwrap();
        let id = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(first))
            .unwrap()
            .0;
        assert_eq!(
            tree.focus, id,
            "label activation must focus its actual native control"
        );
        click(&mut renderer, control_point.0, control_point.1);
        assert_eq!(node(&renderer, peer, role).toggled(), Some(Toggled::False));
        assert_eq!(node(&renderer, first, role).toggled(), Some(Toggled::False));
        click(&mut renderer, 220.0, if is_switch { 264.0 } else { 304.0 });
        assert_eq!(node(&renderer, last, role).toggled(), Some(Toggled::True));
        renderer.resize(338, 400);
        renderer.render_frames(8);
        let bounds = node(&renderer, first, role).bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            (
                initial.0 * 0.5,
                initial.1 * 0.5,
                initial.2 * 0.5,
                initial.3 * 0.5
            )
        );
        click(
            &mut renderer,
            label_point.0 as f32 * 0.5,
            label_point.1 as f32 * 0.5,
        );
        assert_eq!(node(&renderer, first, role).toggled(), Some(Toggled::True));
        assert_eq!(node(&renderer, peer, role).toggled(), Some(Toggled::False));
        save_pixels(
            &renderer,
            if is_switch {
                "switch-labels-narrow-338.png"
            } else {
                "checkbox-labels-narrow-338.png"
            },
        );
    }
}
