//! Native switch anatomy and configuration regressions.
//!
//! ## Usage
//!
//! Preserve responsive control placement, annotations and independent toggles.

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
fn anatomy_fixture() {
    super::super::super::media::load_fonts();
    tessera_components::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::anatomy();
        });
    });
}

#[tessera]
fn configuration_fixture() {
    super::super::super::media::load_fonts();
    tessera_components::theme::material_theme().child(|| {
        tessera_components::column::column().children(|| {
            super::configurations();
        });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing native {role:?} {label}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = node(renderer, label, Role::Switch).bounds().unwrap();
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
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
#[ignore = "requires a native GPU adapter for switch anatomy and configurations"]
fn switch_figures_preserve_native_parts_numbered_markers_and_independent_scaled_toggles() {
    for configuration in [false, true] {
        let width = if configuration { 676 } else { 1024 };
        let mut renderer = HeadlessRenderer::new(
            if configuration {
                Box::new(configuration_fixture)
            } else {
                Box::new(anatomy_fixture)
            },
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width,
                height: 480,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let group = node(
            &renderer,
            if configuration {
                "Switch configurations diagram"
            } else {
                "Switch anatomy diagram"
            },
            Role::Group,
        )
        .bounds()
        .unwrap();
        assert_eq!(
            (group.x0, group.y0, group.width(), group.height()),
            (0.0, 0.0, f64::from(width), 300.0)
        );
        let tree = renderer.accessibility_snapshot().unwrap();
        assert_eq!(
            tree.nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Switch)
                .count(),
            if configuration { 6 } else { 2 }
        );
        for marker in 1..=3 {
            let marker = node(
                &renderer,
                &format!(
                    "Switch {} marker {marker}",
                    if configuration {
                        "configuration"
                    } else {
                        "anatomy"
                    }
                ),
                Role::Note,
            )
            .bounds()
            .unwrap();
            assert_eq!(
                (marker.width(), marker.height()),
                (24.0, if configuration { 24.0 } else { 32.0 })
            );
        }
        let (first, peer, expected) = if configuration {
            (
                "Without icons: unselected",
                "Without icons: selected",
                (292.0, 62.0, 52.0, 48.0),
            )
        } else {
            (
                "Anatomy unselected switch",
                "Anatomy selected switch",
                (378.0, 126.0, 52.0, 48.0),
            )
        };
        let bounds = node(&renderer, first, Role::Switch).bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            expected
        );
        save_pixels(
            &renderer,
            if configuration {
                "switch-configurations-canonical-676.png"
            } else {
                "switch-anatomy-canonical-1024.png"
            },
        );
        click(&mut renderer, first);
        assert_eq!(
            node(&renderer, first, Role::Switch).toggled(),
            Some(Toggled::True)
        );
        assert_eq!(
            node(&renderer, peer, Role::Switch).toggled(),
            Some(Toggled::True)
        );
        if configuration {
            for label in [
                "Icon when selected: unselected",
                "Icons in both states: unselected",
            ] {
                click(&mut renderer, label);
                assert_eq!(
                    node(&renderer, label, Role::Switch).toggled(),
                    Some(Toggled::True)
                );
            }
            assert_eq!(
                node(&renderer, "Icons in both states: selected", Role::Switch).toggled(),
                Some(Toggled::True)
            );
        }
        renderer.resize(width / 2, 480);
        renderer.render_frames(8);
        let bounds = node(&renderer, first, Role::Switch).bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
            (expected.0 * 0.5, expected.1 * 0.5, 26.0, 24.0)
        );
        click(&mut renderer, first);
        assert_eq!(
            node(&renderer, first, Role::Switch).toggled(),
            Some(Toggled::False)
        );
        assert_eq!(
            node(&renderer, peer, Role::Switch).toggled(),
            Some(Toggled::True)
        );
        save_pixels(
            &renderer,
            if configuration {
                "switch-configurations-narrow-338.png"
            } else {
                "switch-anatomy-narrow-512.png"
            },
        );
    }
}
