use tessera::DimensionValue;
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

/// Header row component with two text items
#[tessera]
pub fn header_row() {
    row([
        RowItem::fill(Box::new(|| text("Hello, this is tessera")), Some(1.0), None),
        RowItem::fill(
            Box::new(|| text("Hello, this is another tessera")),
            Some(1.0),
            None,
        ),
    ])
}

/// Vertical text column component
#[tessera]
pub fn text_column() {
    column([
        ColumnItem::fill(Box::new(|| text("This is a column")), Some(1.0), None),
        ColumnItem::fill(Box::new(|| text("Another item in column")), Some(1.0), None),
    ])
}

/// Content section with header and text column
#[tessera]
pub fn content_section() {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .padding(20.0.into())
            .color([0.8, 0.8, 0.9, 1.0]) // Light purple fill, RGBA
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive content section
        || {
            column([
                ColumnItem::wrap(Box::new(header_row)),
                ColumnItem::wrap(Box::new(text_column)),
            ]);
        },
    )
}
