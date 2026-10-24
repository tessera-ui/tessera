//! Live specification version tables with native merged cells.
//!
//! ## Usage
//!
//! Compare Material versions with native table geometry and semantics.

use std::{collections::BTreeMap, time::Duration};

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, Modifier, PlacementModifierNode, PxPosition,
    accesskit::{Node, NodeId, Role},
    modifier::ModifierCapabilityExt as _,
    tessera,
};

use super::*;

#[derive(Clone, Copy, PartialEq)]
struct Origin;
impl PlacementModifierNode for Origin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        boxed()
            .modifier(Modifier::new().fill_max_width().push_placement(Origin))
            .children(|| {
                sliders::slider_spec();
            });
    });
}

struct Pipelines;
impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

fn table<'a>(nodes: &'a BTreeMap<NodeId, Node>, headers: &[&str]) -> &'a Node {
    nodes
        .values()
        .find(|node| {
            if node.role() != Role::Table {
                return false;
            }
            let Some(row) = node.children().first().and_then(|id| nodes.get(id)) else {
                return false;
            };
            row.children()
                .iter()
                .map(|id| nodes[id].label().unwrap_or(""))
                .collect::<Vec<_>>()
                == headers
        })
        .expect("Actual upstream version table")
}

fn assert_tracks(nodes: &BTreeMap<NodeId, Node>, table: &Node) {
    let header = &nodes[&table.children()[0]];
    let bounds = header
        .children()
        .iter()
        .map(|id| nodes[id].bounds().unwrap())
        .collect::<Vec<_>>();
    assert!(
        bounds[0].width() != bounds.last().unwrap().width(),
        "Real text must produce unequal auto columns"
    );
    assert!(
        bounds.iter().all(|bounds| bounds.height() == 56.0),
        "Official16/24 text plus16dp top/bottom cell padding"
    );
    for pair in bounds.windows(2) {
        assert_eq!(pair[0].x1, pair[1].x0);
    }
    let body = table.bounds().unwrap();
    assert_eq!(
        (bounds[0].x0, bounds.last().unwrap().x1),
        (body.x0, body.x1)
    );
}

#[test]
#[ignore = "requires a native GPU adapter for live Slider version tables"]
fn slider_version_tables_publish_native_auto_tracks_and_true_category_row_spans() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 900,
            height: 8000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(16);
    let nodes = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let variants = table(&nodes, &["Variant", "M3", "M3 Expressive"]);
    assert_eq!(
        (variants.row_count(), variants.column_count()),
        (Some(5), Some(3))
    );
    assert_tracks(&nodes, variants);
    let configuration = table(
        &nodes,
        &["Category", "Configuration", "M3", "M3 Expressive"],
    );
    assert_eq!(
        (configuration.row_count(), configuration.column_count()),
        (Some(9), Some(4))
    );
    assert_tracks(&nodes, configuration);
    let rows = configuration
        .children()
        .iter()
        .map(|id| &nodes[id])
        .collect::<Vec<_>>();
    for (label, index) in [("Inset icon", 1), ("Orientation", 3), ("Size", 5)] {
        let cells = rows[index]
            .children()
            .iter()
            .map(|id| &nodes[id])
            .collect::<Vec<_>>();
        let category = cells
            .iter()
            .find(|cell| cell.label() == Some(label))
            .unwrap();
        assert_eq!(
            (
                category.row_index(),
                category.row_span(),
                category.column_index()
            ),
            (Some(index), Some(2), Some(0))
        );
        assert!(
            !rows[index + 1]
                .children()
                .iter()
                .any(|id| nodes[id].label() == Some(label)),
            "Merged category must be a single real cell"
        );
        let last = &nodes[rows[index + 1].children().last().unwrap()];
        assert_eq!(category.bounds().unwrap().y1, last.bounds().unwrap().y1);
    }
    let upstream=nodes.values().find(|node|node.role()==Role::Cell&&node.label()==Some("Available on Android Views (MDC-Android).\nAvailable as tokens on other platforms.*")).expect("Verbatim upstream M3 Expressive availability, not a Tessera platform declaration");
    assert_eq!(
        (upstream.row_index(), upstream.column_index()),
        (Some(6), Some(3))
    );
}
