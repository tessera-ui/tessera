use std::collections::BTreeMap;

use tessera_components::md3::theme::material_theme;
use tessera_ui::{
    MOUSE_POINTER_ID, PxPosition,
    accesskit::{Node, NodeId, Role},
    tessera,
    testing::UiTest,
};

#[tessera]
fn fixture() {
    material_theme().child(super::figure);
}

fn snapshot(renderer: &UiTest) -> BTreeMap<NodeId, Node> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .collect()
}

fn table(nodes: &BTreeMap<NodeId, Node>) -> &Node {
    nodes
        .values()
        .find(|node| {
            node.role() == Role::Table
                && crate::has_label(&node, "Icon button corner radius values")
        })
        .expect("Native radius table")
}

fn header<'a>(nodes: &'a BTreeMap<NodeId, Node>, label: &str) -> &'a Node {
    nodes
        .values()
        .find(|node| node.role() == Role::ColumnHeader && crate::has_label(&node, label))
        .expect("Native radius column header")
}

fn assert_wide_grid(nodes: &BTreeMap<NodeId, Node>) {
    let table = table(nodes);
    assert_eq!(
        (table.row_count(), table.column_count()),
        (Some(4), Some(6))
    );
    assert_eq!(table.children().len(), 4);
    let mut reference = None;
    for (row_index, row_id) in table.children().iter().enumerate() {
        let row = &nodes[row_id];
        assert_eq!(row.role(), Role::Row);
        assert_eq!(row.children().len(), 6);
        let bounds = row
            .children()
            .iter()
            .enumerate()
            .map(|(column, id)| {
                let cell = &nodes[id];
                assert_eq!(cell.row_index(), Some(row_index));
                assert_eq!(cell.column_index(), Some(column));
                assert_eq!(
                    cell.role(),
                    if row_index == 0 && column > 0 {
                        Role::ColumnHeader
                    } else if row_index > 0 && column == 0 {
                        Role::RowHeader
                    } else {
                        Role::Cell
                    }
                );
                cell.bounds().unwrap()
            })
            .collect::<Vec<_>>();
        assert!(bounds.iter().all(|bounds| bounds.height() == 56.0));
        assert!((bounds[0].width() - 2.0 * bounds[1].width()).abs() <= 2.0);
        for pair in bounds.windows(2) {
            assert_eq!(pair[0].x1, pair[1].x0);
        }
        let tracks = bounds.iter().map(|bounds| bounds.x0).collect::<Vec<_>>();
        if let Some(reference) = &reference {
            assert_eq!(&tracks, reference, "All rows share the same native tracks");
        } else {
            reference = Some(tracks);
        }
    }
    let bounds = table.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (1024.0, 224.0));
}

#[test]
#[ignore = "requires a native GPU adapter for actual corner-radius table layout and scrolling"]
fn radius_table_preserves_native_grid_and_reveals_columns_with_horizontal_input_after_resize() {
    let mut renderer = UiTest::new()
        .viewport(1024, 400)
        .rendered(crate::TestPipelines)
        .content(fixture);
    renderer.frames(16);
    assert_wide_grid(&snapshot(&renderer));

    renderer.resize(390, 400);
    renderer.frames(16);
    let before = snapshot(&renderer);
    assert_eq!(table(&before).row_count(), Some(4));
    let before_xs = header(&before, "XS").bounds().unwrap();
    let before_xl = header(&before, "XL").bounds().unwrap();
    assert_eq!(
        before_xs.height(),
        56.0,
        "Text rows must not be scaled down"
    );
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(195.0, 100.0),
        -226.0,
        0.0,
    );
    renderer.frames(40);
    let after = snapshot(&renderer);
    let after_xs = header(&after, "XS").bounds().unwrap();
    let after_xl = header(&after, "XL").bounds().unwrap();
    assert!(
        after_xs.x0 < before_xs.x0 - 100.0,
        "Real horizontal input must move the table"
    );
    assert!(
        after_xl.width() > before_xl.width() + 50.0,
        "Horizontal scrolling must reveal the final column"
    );
    assert_eq!(after_xl.height(), 56.0);

    renderer.resize(1024, 400);
    renderer.frames(16);
    assert_wide_grid(&snapshot(&renderer));
}
