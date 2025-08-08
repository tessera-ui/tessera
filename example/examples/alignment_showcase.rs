//! Layout Alignment Showcase

use tessera_ui::{Color, DimensionValue, Dp, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{AsColumnItem, ColumnArgsBuilder, column},
    row::{AsRowItem, RowArgsBuilder, row},
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};

/// Create a small colored box
#[tessera]
fn small_box(text_content: &'static str, color: Color) {
    surface(
        SurfaceArgs {
            color,
            shape: Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            },
            padding: Dp(8.0),
            width: Some(DimensionValue::Fixed(Px(40))),
            height: Some(DimensionValue::Fixed(Px(40))),
            ..Default::default()
        },
        None,
        move || {
            text(
                TextArgsBuilder::default()
                    .text(text_content.to_string())
                    .color(Color::WHITE)
                    .size(Dp(12.0))
                    .build()
                    .unwrap(),
            )
        },
    );
}

/// Create a demonstration row
#[tessera]
fn row_demo_line(title: &'static str, alignment: MainAxisAlignment) {
    column(
        ColumnArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Start)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        [
            // Title
            (move || {
                text(
                    TextArgsBuilder::default()
                        .text(title.to_string())
                        .size(Dp(14.0))
                        .color(Color::from_rgb_u8(80, 80, 80))
                        .build()
                        .unwrap(),
                )
            })
            .into_column_item(),
            // Alignment Demo Container - Fixed Width，Visible Background Border
            (move || {
                surface(
                    SurfaceArgs {
                        color: Color::new(0.9, 0.9, 0.9, 1.0), // Gray background to see borders clearly
                        shape: Shape::RoundedRectangle {
                            corner_radius: 25.0,
                            g2_k_value: 3.0,
                        },
                        padding: Dp(10.0),
                        width: Some(DimensionValue::Fixed(Px(400))), // Sufficient Fixed Width
                        height: Some(DimensionValue::Fixed(Px(70))),
                        ..Default::default()
                    },
                    None,
                    move || {
                        row(
                            RowArgsBuilder::default()
                                .width(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                }) // row Fill Container Width
                                .height(DimensionValue::Wrap {
                                    min: None,
                                    max: None,
                                }) // row Height Adapts to Content
                                .main_axis_alignment(alignment) // Directly use different main axis alignments
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .build()
                                .unwrap(),
                            [
                                (|| small_box("1", Color::new(0.2, 0.6, 0.9, 1.0))).into_row_item(),
                                (|| small_box("2", Color::new(0.9, 0.2, 0.2, 1.0))).into_row_item(),
                                (|| small_box("3", Color::new(0.2, 0.8, 0.3, 1.0))).into_row_item(),
                            ],
                        );
                    },
                );
            })
            .into_column_item(),
        ],
    );
}

/// Main App
#[tessera]
fn app() {
    surface(
        SurfaceArgs {
            color: Color::WHITE, // White Background
            padding: Dp(20.0),
            width: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }), // Fill Width
            height: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }), // Fill Height
            ..Default::default()
        },
        None,
        || {
            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    }) // Fill Width
                    .height(DimensionValue::Fill {
                        min: None,
                        max: None,
                    }) // Fill Height
                    .build()
                    .unwrap(),
                [
                    // Main Title
                    Box::new(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Tessera Alignment Demo".to_string())
                                .size(Dp(24.0))
                                .color(Color::from_rgb_u8(40, 40, 40))
                                .build()
                                .unwrap(),
                        )
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    // Spacing
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(30)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    // row Alignment Demo Title
                    Box::new(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("row Main Axis Alignment:".to_string())
                                .size(Dp(18.0))
                                .color(Color::from_rgb_u8(60, 60, 60))
                                .build()
                                .unwrap(),
                        )
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    // Spacing
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(15)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    // RowAlignment Demo
                    Box::new(|| row_demo_line("Start", MainAxisAlignment::Start))
                        as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(20)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| row_demo_line("Center", MainAxisAlignment::Center))
                        as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(20)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| row_demo_line("End", MainAxisAlignment::End))
                        as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(20)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| row_demo_line("SpaceEvenly", MainAxisAlignment::SpaceEvenly))
                        as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(20)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| row_demo_line("SpaceBetween", MainAxisAlignment::SpaceBetween))
                        as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| {
                        spacer(SpacerArgs {
                            width: DimensionValue::Fixed(Px(0)),
                            height: DimensionValue::Fixed(Px(20)),
                        })
                    }) as Box<dyn FnOnce() + Send + Sync>,
                    Box::new(|| row_demo_line("SpaceAround", MainAxisAlignment::SpaceAround))
                        as Box<dyn FnOnce() + Send + Sync>,
                ],
            );
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    Renderer::run(app, |app| {
        tessera_ui_basic_components::pipelines::register_pipelines(app);
    })?;
    Ok(())
}
