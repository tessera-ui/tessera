//! Native list baseline figure and single-selection regressions.
//!
//! ## Usage
//!
//! Verify the baseline rows and single-selection feedback of the live list
//! article.

use std::time::Duration;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, NodeId, Toggled},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn baseline_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                baseline();
            });
    });
}

#[tessera]
fn selection_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                mode_demo(
                    ListItemSelectionMode::Single,
                    false,
                    true,
                    true,
                    "Test list",
                );
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

fn labeled(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
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
fn baseline_rows_stay_flush_and_publish_their_measurements() {
    let renderer = renderer(baseline_fixture);
    let rows = [
        "Baseline two-line row",
        "Baseline one-line row",
        "Baseline three-line row",
    ]
    .map(|label| labeled(&renderer, label).1.bounds().unwrap());
    assert!(
        (rows[1].y0 - rows[0].y1 - 1.0).abs() <= 0.01
            && (rows[2].y0 - rows[1].y1 - 1.0).abs() <= 0.01,
        "baseline rows must stack with only their 1dp dividers: {rows:?}"
    );
    for row in rows {
        assert!(
            row.height() >= 56.0 - 0.01,
            "a baseline row must keep at least the one-line height: {row:?}"
        );
    }
    let values = labels(&renderer);
    assert!(
        values
            .iter()
            .any(|label| label == "0dp, with an optional outline variant divider"),
        "the baseline section must document its flush row spacing"
    );
    assert!(
        values
            .iter()
            .any(|label| label == "Surface without segmented corners"),
        "the baseline section must document its container"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn single_selection_moves_between_segmented_rows() {
    let mut renderer = renderer(selection_fixture);
    assert_eq!(
        labeled(&renderer, "Test list: Notes").1.toggled(),
        Some(Toggled::True)
    );
    let projects = labeled(&renderer, "Test list: Projects")
        .1
        .bounds()
        .unwrap();
    click(&mut renderer, projects);
    assert_eq!(
        labeled(&renderer, "Test list: Projects").1.toggled(),
        Some(Toggled::True)
    );
    assert_eq!(
        labeled(&renderer, "Test list: Notes").1.toggled(),
        Some(Toggled::False)
    );
}
