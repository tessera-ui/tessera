//! Native state-matrix selection and scaling regressions.
//!
//! ## Usage
//!
//! Preserve interactive calendar choices during uniform scaling.

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Role},
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
    super::super::super::media::load_fonts();
    material_theme().child(|| {
        states();
    });
}

fn choice(renderer: &HeadlessRenderer, group: &str, label: &str) -> Node {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let bounds = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Group && node.label() == Some(group))
        .unwrap_or_else(|| panic!("missing native state context {group}"))
        .1
        .bounds()
        .unwrap();
    snapshot
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == Role::Button
                && node.label() == Some(label)
                && node.bounds().is_some_and(|cell| {
                    cell.x0 >= bounds.x0
                        && cell.y0 >= bounds.y0
                        && cell.x1 <= bounds.x1
                        && cell.y1 <= bounds.y1
                })
        })
        .unwrap_or_else(|| panic!("missing actual {label} inside {group}"))
        .1
}

fn click(renderer: &mut HeadlessRenderer, group: &str, label: &str) {
    let bounds = choice(renderer, group, label).bounds().unwrap();
    let pointer = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, pointer);
    renderer.pointer_press(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, pointer, PressKeyEventType::Left);
    renderer.render_frames(4);
}

#[test]
#[ignore = "requires a native GPU adapter for the date States matrix"]
fn states_matrix_preserves_themed_native_choices_and_scaled_pointer_input() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 1024,
            height: 1400,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(6);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    for (dark, y, height) in [(false, 0.0, 624.0), (true, 624.0, 622.0)] {
        let name = format!("{} calendar states", if dark { "Dark" } else { "Light" });
        let panel = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Group && node.label() == Some(name.as_str()))
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (panel.width(), panel.height(), panel.y0),
            (1024.0, height, y)
        );
        for column in 0..5 {
            for row in 0..6 {
                let group = cell_name(dark, column, row);
                let exists = snapshot.nodes.iter().any(|(_, node)| {
                    node.role() == Role::Group && node.label() == Some(group.as_str())
                });
                assert_eq!(
                    exists,
                    column != 1 || matches!(row, 0 | 4),
                    "source-absent states must remain absent: {group}"
                );
            }
        }
    }
    let range_name = cell_name(false, 0, 3);
    let range_bounds = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Group && node.label() == Some(range_name.as_str()))
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert_eq!(range_bounds.width(), 78.0);
    let pixels = renderer.read_pixels().unwrap();
    let background =
        (((range_bounds.y0 + 1.0) as usize * 1024 + (range_bounds.x0 + 1.0) as usize) * 4)..;
    for x in [range_bounds.x0 + 1.0, range_bounds.x1 - 2.0] {
        let band = (((range_bounds.y0 + 29.0) as usize * 1024 + x as usize) * 4)..;
        assert_ne!(
            &pixels[band][..3],
            &pixels[background.clone()][..3],
            "native adjacent range cells must continue the band across its full78dp context"
        );
    }
    let label = "August 17, 2025";
    let light = cell_name(false, 0, 0);
    let dark = cell_name(true, 0, 0);
    assert_eq!(choice(&renderer, &light, label).is_selected(), Some(false));
    assert_eq!(choice(&renderer, &dark, label).is_selected(), Some(false));
    assert_eq!(
        choice(&renderer, &light, label).bounds().unwrap().width(),
        48.0
    );
    click(&mut renderer, &light, label);
    assert_eq!(choice(&renderer, &light, label).is_selected(), Some(true));
    assert_eq!(choice(&renderer, &dark, label).is_selected(), Some(false));
    let disabled = cell_name(false, 1, 0);
    assert!(choice(&renderer, &disabled, label).is_disabled());
    click(&mut renderer, &disabled, label);
    assert_eq!(
        choice(&renderer, &disabled, label).is_selected(),
        Some(false)
    );
    renderer.resize(512, 1400);
    renderer.render_frames(8);
    assert_eq!(
        choice(&renderer, &light, label).bounds().unwrap().width(),
        24.0
    );
    assert_eq!(
        choice(&renderer, &dark, label).bounds().unwrap().height(),
        24.0
    );
    let year = cell_name(true, 0, 4);
    let light_year = cell_name(false, 0, 4);
    assert_eq!(
        choice(&renderer, &year, "2025").bounds().unwrap().width(),
        56.0
    );
    assert_eq!(
        choice(&renderer, &year, "2025").bounds().unwrap().height(),
        26.0
    );
    click(&mut renderer, &year, "2025");
    assert_eq!(choice(&renderer, &year, "2025").is_selected(), Some(true));
    assert_eq!(
        choice(&renderer, &light_year, "2025").is_selected(),
        Some(false)
    );
    assert_eq!(choice(&renderer, &light, label).is_selected(), Some(true));
}
