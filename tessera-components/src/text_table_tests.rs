//! Content-based table geometry, merged cells and accessibility.
//!
//! ## Usage
//!
//! Preserve table tracks and merged-cell semantics during reflow.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    time::Duration,
};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, LayoutCoordinates, PlacementModifierNode, State,
    accesskit::{Node, NodeId},
    modifier::ModifierCapabilityExt as _,
    scaled_box,
    testing::{layout_test, pump_layout_test_session, start_layout_test_session},
};

use super::*;

thread_local! {
    pub(super) static CELLS:RefCell<BTreeMap<(usize,usize),LayoutCoordinates>>=const{RefCell::new(BTreeMap::new())};
    static WIDTH:Cell<Option<State<f64>>>=const{Cell::new(None)};
    static FONT_SIZE:Cell<Option<State<f64>>>=const{Cell::new(None)};
    static HEADER_COLOR:Cell<Option<State<Option<Color>>>>=const{Cell::new(None)};
}

fn data() -> Vec<TextTableCell> {
    let mut cells = ["Category", "Configuration", "Availability"]
        .into_iter()
        .enumerate()
        .map(|(column, text)| TextTableCell {
            text: text.into(),
            column,
            heading: TextTableHeading::Column,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    cells.extend([
        TextTableCell {
            text: "Inset icon".into(),
            row: 1,
            row_span: 2,
            heading: TextTableHeading::Row,
            ..Default::default()
        },
        TextTableCell {
            text: "No (default)".into(),
            row: 1,
            column: 1,
            ..Default::default()
        },
        TextTableCell {
            text: "Available".into(),
            row: 1,
            column: 2,
            ..Default::default()
        },
        TextTableCell {
            text: "Yes".into(),
            row: 2,
            column: 1,
            ..Default::default()
        },
        TextTableCell {
            text: "Available on every configured platform".into(),
            row: 2,
            column: 2,
            ..Default::default()
        },
        TextTableCell {
            text: "Summary".into(),
            row: 3,
            column_span: 3,
            heading: TextTableHeading::Row,
            ..Default::default()
        },
    ]);
    cells
}

#[derive(Clone, Copy, PartialEq)]
struct Origin;
impl PlacementModifierNode for Origin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25, 20.5)
    }
}

#[tessera]
fn fixture() {
    let width = remember(|| 1000.0_f64);
    let font_size = remember(|| 16.0_f64);
    let header_color = remember(|| Some(Color::RED));
    WIDTH.set(Some(width));
    FONT_SIZE.set(Some(font_size));
    HEADER_COLOR.set(Some(header_color));
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().push_placement(Origin))
        .children(move || {
            scaled_box(0.625).child(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().width(Dp(width.get())))
                    .children(move || {
                        text_table(data())
                            .columns(3)
                            .color(Color::BLUE)
                            .header_color_optional(header_color.get())
                            .modifier(Modifier::new().fill_max_width())
                            .style(TextStyle {
                                font_family: Some("Roboto"),
                                font_size: Dp(font_size.get()),
                                line_height: Some(Dp(24.0)),
                                ..Default::default()
                            });
                    });
            });
        });
}

fn coordinates() -> BTreeMap<(usize, usize), LayoutCoordinates> {
    CELLS.with_borrow(Clone::clone)
}

fn assert_tracks(values: &BTreeMap<(usize, usize), LayoutCoordinates>, width: i32) {
    let first = values[&(0, 0)];
    assert_eq!(
        first.local_to_window.translation(),
        PxPosition::from_f32_xy(10.25, 20.5)
    );
    let sizes = (0..3)
        .map(|column| values[&(0, column)].local_size.width.0)
        .collect::<Vec<_>>();
    assert_eq!(sizes.iter().sum::<i32>(), width);
    assert_ne!(
        sizes[0], sizes[2],
        "Columns must follow real text dimensions rather than equal weights"
    );
    for column in 0..2 {
        let current = values[&(0, column)].window_bounds();
        let next = values[&(0, column + 1)].window_bounds();
        assert_eq!(
            current.x + current.width,
            next.x,
            "Shared tracks cannot leave pixel gaps or overlap"
        );
    }
    for column in 1..3 {
        for row in 1..3 {
            assert_eq!(
                values[&(row, column)].local_size.width,
                values[&(0, column)].local_size.width
            );
            assert_eq!(
                values[&(row, column)].window_bounds().x,
                values[&(0, column)].window_bounds().x
            );
        }
    }
    assert_eq!(values[&(3, 0)].local_size.width, Px(width));
    assert_eq!(values[&(3, 0)].window_bounds().width, width as f32 * 0.625);
    let spanning = values[&(1, 0)].window_bounds();
    let first_row = values[&(1, 1)].window_bounds();
    let last_row = values[&(2, 1)].window_bounds();
    assert_eq!(spanning.y, first_row.y);
    assert_eq!(spanning.y + spanning.height, last_row.y + last_row.height);
}

#[test]
fn table_measures_shared_intrinsic_columns_and_row_spans_across_reflow() {
    crate::test_utils::run_isolated_test(
        "text_table::tests::table_measures_shared_intrinsic_columns_and_row_spans_across_reflow",
        || {
            tessera_ui::dp::set_scale_factor(1.0);
            CELLS.with_borrow_mut(BTreeMap::clear);
            let mut session = start_layout_test_session(
                layout_test(|| {
                    fixture();
                })
                .viewport_px(800, 600),
            );
            pump_layout_test_session(&mut session, 0);
            let original = coordinates();
            assert_tracks(&original, 1000);
            assert_eq!(
                original[&(0, 0)].local_size.height,
                Px(56),
                "24dp line plus16dp top and bottom insets"
            );
            assert_eq!(original[&(1, 0)].local_size.height, Px(112));
            WIDTH.get().unwrap().set(240.0);
            pump_layout_test_session(&mut session, 16_666_667);
            let narrow = coordinates();
            assert_tracks(&narrow, 240);
            assert!(
                narrow[&(2, 2)].local_size.height > original[&(2, 2)].local_size.height,
                "Real text wrapping must grow the row"
            );
            WIDTH.get().unwrap().set(1000.0);
            pump_layout_test_session(&mut session, 33_333_334);
            assert_eq!(
                coordinates(),
                original,
                "Restored tracks must not reuse narrow cell measurements"
            );
            FONT_SIZE.get().unwrap().set(20.0);
            pump_layout_test_session(&mut session, 50_000_001);
            let larger_text = coordinates();
            assert_tracks(&larger_text, 1000);
            assert_ne!(
                larger_text[&(0, 0)].local_size.width,
                original[&(0, 0)].local_size.width,
                "Typography changes must invalidate intrinsic column measurements"
            );
            FONT_SIZE.get().unwrap().set(16.0);
            pump_layout_test_session(&mut session, 66_666_668);
            assert_eq!(coordinates(), original);
        },
    );
}

#[test]
fn auto_column_growth_and_shrink_preserve_integer_pixel_residuals() {
    assert_eq!(distribute(10, &[1, 1, 1]), [4, 3, 3]);
    assert_eq!(fit_columns(26, &[2, 4], &[6, 12]), [9, 17]);
    assert_eq!(fit_columns(12, &[2, 4], &[6, 12]), [4, 8]);
    assert_eq!(fit_columns(3, &[2, 4], &[6, 12]), [1, 2]);
}

#[test]
#[should_panic(expected = "Table cells overlap")]
fn table_rejects_overlapping_merged_cells_before_layout() {
    validate(
        &[
            TextTableCell {
                row_span: 2,
                ..Default::default()
            },
            TextTableCell {
                row: 1,
                ..Default::default()
            },
        ],
        1,
    );
}

#[test]
#[ignore = "requires a native GPU adapter for published table semantics"]
fn table_publishes_merged_cell_semantics_and_independent_heading_ink() {
    crate::test_utils::run_isolated_test(
        "text_table::tests::table_publishes_merged_cell_semantics_and_independent_heading_ink",
        || {
            tessera_ui::dp::set_scale_factor(1.0);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 800,
                    height: 600,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            let map = snapshot
                .nodes
                .into_iter()
                .collect::<BTreeMap<NodeId, Node>>();
            let table = map
                .values()
                .find(|node| node.role() == Role::Table)
                .unwrap();
            assert_eq!(
                (table.row_count(), table.column_count()),
                (Some(4), Some(3))
            );
            assert_eq!(table.children().len(), 4);
            let expected = [
                vec!["Category", "Configuration", "Availability"],
                vec!["Inset icon", "No (default)", "Available"],
                vec!["Yes", "Available on every configured platform"],
                vec!["Summary"],
            ];
            for (row_index, row_id) in table.children().iter().enumerate() {
                let row = &map[row_id];
                assert_eq!(row.role(), Role::Row);
                assert_eq!(row.row_index(), Some(row_index));
                let children = row.children().iter().map(|id| &map[id]).collect::<Vec<_>>();
                assert_eq!(
                    children
                        .iter()
                        .map(|node| node.label().unwrap())
                        .collect::<Vec<_>>(),
                    expected[row_index]
                );
                for child in children {
                    assert_eq!(child.row_index(), Some(row_index));
                    assert!(matches!(
                        child.role(),
                        Role::Cell | Role::RowHeader | Role::ColumnHeader
                    ));
                }
            }
            let spanning = map
                .values()
                .find(|node| node.label() == Some("Inset icon") && node.role() == Role::RowHeader)
                .unwrap();
            assert_eq!(
                (
                    spanning.row_index(),
                    spanning.row_span(),
                    spanning.column_index(),
                    spanning.column_span()
                ),
                (Some(1), Some(2), Some(0), Some(1))
            );
            let summary = map
                .values()
                .find(|node| node.label() == Some("Summary") && node.role() == Role::RowHeader)
                .unwrap();
            assert_eq!(
                (
                    summary.row_index(),
                    summary.column_index(),
                    summary.column_span()
                ),
                (Some(3), Some(0), Some(3))
            );
            assert_eq!(summary.bounds().unwrap().width(), 625.0);
            let bounds = spanning.bounds().unwrap();
            assert_eq!(bounds.height(), 70.0);
            assert_eq!(bounds.y0, 55.5);

            let geometry = coordinates();
            let region = |pixels: &[u8], cell: (usize, usize)| {
                let bounds = geometry[&cell].window_bounds();
                let mut result = Vec::new();
                for y in bounds.y.ceil() as usize..(bounds.y + bounds.height).floor() as usize {
                    for x in bounds.x.ceil() as usize..(bounds.x + bounds.width).floor() as usize {
                        let offset = (y * 800 + x) * 4;
                        result.push([pixels[offset], pixels[offset + 1], pixels[offset + 2]]);
                    }
                }
                result
            };
            let ink = |pixels: &[[u8; 3]], channel: usize| {
                pixels
                    .iter()
                    .filter(|pixel| {
                        (0..3).all(|other| {
                            other == channel || pixel[channel] as i16 - pixel[other] as i16 > 64
                        })
                    })
                    .count()
            };
            let initial = renderer.read_pixels().unwrap();
            let body = region(&initial, (1, 1));
            assert!(ink(&region(&initial, (0, 0)), 0) > 2);
            assert!(ink(&body, 2) > 2);
            HEADER_COLOR.get().unwrap().set(Some(Color::GREEN));
            renderer.render_frames(2);
            let updated = renderer.read_pixels().unwrap();
            assert!(ink(&region(&updated, (0, 0)), 1) > 2);
            assert_eq!(ink(&region(&updated, (0, 0)), 0), 0);
            assert_eq!(region(&updated, (1, 1)), body);
            assert_eq!(coordinates(), geometry);
            HEADER_COLOR.get().unwrap().set(None);
            renderer.render_frames(2);
            let inherited = renderer.read_pixels().unwrap();
            assert!(ink(&region(&inherited, (0, 0)), 2) > 2);
            assert_eq!(region(&inherited, (1, 1)), body);
            assert_eq!(coordinates(), geometry);
        },
    );
}
