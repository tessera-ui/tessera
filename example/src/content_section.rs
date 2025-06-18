use tessera::DimensionValue;
use tessera_basic_components::{
    column::{AsColumnItem, ColumnArgsBuilder, column},
    row::{AsRowItem, RowArgsBuilder, row},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

/// Header row component with two text items
#[tessera]
pub fn header_row() {
    row(
        RowArgsBuilder::default().build().unwrap(),
        [
            ((|| text("Hello, this is tessera")), 1.0f32).into_row_item(),
            ((|| text("Hello, this is another tessera")), 1.0f32).into_row_item(),
        ],
    )
}

/// Vertical text column component
#[tessera]
pub fn text_column() {
    column(
        ColumnArgsBuilder::default().build().unwrap(),
        [
            ((|| text("This is a column")), 1.0f32).into_column_item(),
            ((|| text("Another item in column")), 1.0f32).into_column_item(),
        ],
    )
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
            column(
                ColumnArgsBuilder::default().build().unwrap(),
                [
                    (|| header_row()).into_column_item(),
                    (|| text_column()).into_column_item(),
                ],
            );
        },
    )
}
