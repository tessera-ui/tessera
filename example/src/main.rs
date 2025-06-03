use tessera::{DimensionValue, Renderer}; // Added DimensionValue
use tessera_basic_components::{
    column::{ColumnItem, column}, // Changed to ColumnItem
    row::{RowItem, row},          // Changed to RowItem
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run(app)
}

#[tessera]
fn app() {
    surface(
        SurfaceArgsBuilder::default()
            .color([1.0, 1.0, 1.0])
            .width(DimensionValue::Fill { max: None }) // Make surface fill width
            .height(DimensionValue::Fill { max: None }) // Make surface fill height
            .build()
            .unwrap(),
        || {
            column([
                ColumnItem::wrap(&|| {
                    // Assuming this surface wraps its content or has its own size
                    surface(
                        SurfaceArgsBuilder::default().padding(20.0).build().unwrap(),
                        || {
                            column([
                                ColumnItem::wrap(&|| {
                                    // Row container, assume wrap or specific size
                                    row([
                                        RowItem::fill(
                                            &|| text("Hello, this is tessera"),
                                            Some(1.0),
                                            None,
                                        ), // weight 1.0, fill available width
                                        RowItem::fill(
                                            &|| text("Hello, this is another tessera"),
                                            Some(1.0),
                                            None,
                                        ), // weight 1.0, fill available width
                                    ])
                                }),
                                ColumnItem::wrap(&|| {
                                    // Column container, assume wrap or specific size
                                    column([
                                        ColumnItem::fill(
                                            &|| text("This is a column"),
                                            Some(1.0),
                                            None,
                                        ), // weight 1.0, fill available height
                                        ColumnItem::fill(
                                            &|| text("Another item in column"),
                                            Some(1.0),
                                            None,
                                        ), // weight 1.0, fill available height
                                    ])
                                }),
                            ]);
                        },
                    )
                }),
                ColumnItem::wrap(&|| {
                    // Spacer container, assume wrap or specific size
                    spacer(
                        SpacerArgsBuilder::default()
                            .height(DimensionValue::Fixed(10)) // Explicitly Fixed height for spacer
                            .build()
                            .unwrap(),
                    )
                }),
                ColumnItem::wrap(&|| {
                    // Surface container, assume wrap or specific size
                    surface(
                        SurfaceArgsBuilder::default()
                            .corner_radius(25.0)
                            // This surface will wrap its text content by default
                            .build()
                            .unwrap(),
                        || {
                            text("Hello, this is a surface with text");
                        },
                    )
                }),
            ]);
        },
    );
}
