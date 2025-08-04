use tessera_ui::{Color, DimensionValue, tessera};
use tessera_ui_basic_components::{
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

/// Header row component with two text items
#[tessera]
pub fn header_row() {
    row_ui![
        RowArgsBuilder::default().build().unwrap(),
        (|| text("Hello, this is tessera"), 1.0f32),
        (|| text("Hello, this is another tessera"), 1.0f32)
    ]
}

/// Vertical text column component
#[tessera]
pub fn text_column() {
    column_ui!(
        ColumnArgsBuilder::default().build().unwrap(),
        (|| text("This is a column"), 1.0f32),
        (|| text("Another item in column"), 1.0f32)
    )
}

/// Content section with header and text column
#[tessera]
pub fn content_section() {
    surface(
        SurfaceArgsBuilder::default()
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            })
            .padding(20.0.into())
            .color(Color::new(0.8, 0.8, 0.9, 1.0)) // Light purple fill, RGBA
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive content section
        || {
            column_ui!(
                ColumnArgsBuilder::default().build().unwrap(),
                || header_row(),
                || text_column()
            );
        },
    )
}
