//! Fluid Glass Showcase

use tessera::{Color, DimensionValue, Dp, Px, Renderer};
use tessera_basic_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{AsBoxedItem, BoxedArgs, boxed},
    column::{AsColumnItem, ColumnArgsBuilder, column},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    row::{AsRowItem, RowArgsBuilder, row},
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Create a small colored box
#[tessera]
fn small_box(color: Color) {
    surface(
        SurfaceArgs {
            color,
            shape: Shape::RoundedRectangle {
                corner_radius: 25.0,
            },
            padding: Dp(8.0),
            width: Some(DimensionValue::Fixed(Px(40))),
            height: Some(DimensionValue::Fixed(Px(40))),
            ..Default::default()
        },
        None,
        move || {},
    );
}

/// Main App
#[tessera]
fn app() {
    // A surface to hold everything
    surface(
        SurfaceArgs {
            color: Color::new(0.1, 0.1, 0.2, 1.0), // Dark background to make the effect more visible
            width: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            height: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            ..Default::default()
        },
        None,
        || {
            // Use boxed to stack background content and fluid glass
            boxed(
                BoxedArgs {
                    alignment: Alignment::Center,
                    ..Default::default()
                },
                [
                    // Background content layer
                    (move || {
                        column(
                            ColumnArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .width(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                })
                                .height(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                })
                                .build()
                                .unwrap(),
                            [
                                // Colorful boxes row
                                (move || {
                                    row(
                                        RowArgsBuilder::default()
                                            .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                            .width(DimensionValue::Fixed(Px(400)))
                                            .build()
                                            .unwrap(),
                                        [
                                            (|| small_box(Color::new(0.2, 0.6, 0.9, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(0.9, 0.2, 0.2, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(0.2, 0.8, 0.3, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(0.9, 0.8, 0.2, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(0.8, 0.2, 0.8, 1.0)))
                                                .into_row_item(),
                                        ],
                                    )
                                })
                                .into_column_item(),
                                // Spacer
                                (|| {
                                    spacer(SpacerArgs {
                                        width: DimensionValue::Fixed(Px(0)),
                                        height: DimensionValue::Fixed(Px(30)),
                                    })
                                })
                                .into_column_item(),
                                // Text content
                                (move || {
                                    text(
                                        TextArgsBuilder::default()
                                            .text(
                                                "This text should appear blurred through the glass"
                                                    .to_string(),
                                            )
                                            .size(Dp(18.0))
                                            .color(Color::WHITE)
                                            .build()
                                            .unwrap(),
                                    )
                                })
                                .into_column_item(),
                                // More colorful elements
                                (|| {
                                    spacer(SpacerArgs {
                                        width: DimensionValue::Fixed(Px(0)),
                                        height: DimensionValue::Fixed(Px(20)),
                                    })
                                })
                                .into_column_item(),
                                (move || {
                                    row(
                                        RowArgsBuilder::default()
                                            .main_axis_alignment(MainAxisAlignment::Center)
                                            .build()
                                            .unwrap(),
                                        [
                                            (|| small_box(Color::new(0.3, 0.5, 1.0, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(1.0, 0.3, 0.3, 1.0)))
                                                .into_row_item(),
                                            (|| small_box(Color::new(0.3, 1.0, 0.3, 1.0)))
                                                .into_row_item(),
                                        ],
                                    )
                                })
                                .into_column_item(),
                            ],
                        )
                    })
                    .into_boxed_item(),
                    // Fluid glass overlay
                    (move || {
                        fluid_glass(
                            FluidGlassArgsBuilder::default()
                                .blur_radius(10.0)
                                .width(DimensionValue::Fixed(Px(350)))
                                .height(DimensionValue::Fixed(Px(250)))
                                .shape(Shape::RoundedRectangle {
                                    corner_radius: 20.0,
                                })
                                .highlight_color(Color::new(1.0, 1.0, 1.0, 0.3))
                                .tint_color(Color::new(0.8, 0.9, 1.0, 0.2))
                                .inner_shadow_radius(0.0)
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    })
                    .into_boxed_item(),
                ],
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
