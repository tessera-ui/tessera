use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, Modifier, tessera, testing::UiTest};

use crate::{column::column, spacer::spacer};

use super::{GridItem, grid};

#[tessera]
fn tile(index: usize) {
    spacer().modifier(
        Modifier::new()
            .size(Dp(40.0), Dp(30.0))
            .semantics(SemanticsArgs {
                test_tag: Some(format!("tile-{index}")),
                ..Default::default()
            }),
    );
}

fn positions(columns: Vec<GridItem>, width: f64) -> Vec<(i32, i32)> {
    let mut ui = UiTest::new().viewport(400, 400).content(move || {
        let columns = columns.clone();
        column().children(move || {
            grid(columns.clone())
                .modifier(Modifier::new().width(Dp(width)))
                .content(|| {
                    for index in 0..5 {
                        tile(index);
                    }
                });
        });
    });
    let frame = ui.frame();
    (0..5)
        .map(|index| {
            let bounds = frame.node(&format!("tile-{index}")).bounds();
            (bounds.x.0, bounds.y.0)
        })
        .collect()
}

#[test]
fn adaptive_items_fit_as_many_columns_as_the_width_allows() {
    // Three 96pt columns fit in 304pt with 8pt spacing; tiles centre in their columns.
    let placed = positions(vec![GridItem::Adaptive { min: Dp(90.0) }], 304.0);
    assert_eq!(&placed[..4], [(28, 0), (132, 0), (236, 0), (28, 38)]);
}

#[test]
fn fixed_columns_keep_their_width_and_flexible_columns_share_the_rest() {
    let placed = positions(vec![GridItem::Fixed(Dp(50.0)), GridItem::flexible()], 208.0);
    // The fixed column is 50pt; the flexible one takes the remaining 150pt.
    assert_eq!(&placed[..2], [(5, 0), (113, 0)]);
    assert_eq!(placed[2], (5, 38));
}
