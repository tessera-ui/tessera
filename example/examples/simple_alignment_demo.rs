//! Simple Layout Alignment Demo
//!
//! Showcasecolumnandrowdifferent alignment options for the component

use tessera::{DimensionValue, Dp, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{AsColumnItem, ColumnArgsBuilder, column},
    row::{AsRowItem, RowArgsBuilder, row},
    surface::{SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Create a colored box
#[tessera]
fn color_box(text_content: &'static str, color: [f32; 4]) {
    surface(
        SurfaceArgs {
            color,
            corner_radius: 8.0,
            padding: Dp(12.0),
            width: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            ..Default::default()
        },
        None,
        move || {
            text(
                TextArgsBuilder::default()
                    .text(text_content.to_string())
                    .color([255, 255, 255])
                    .build()
                    .unwrap(),
            )
        },
    );
}

/// Demonstrate different alignment modes
#[tessera]
fn alignment_demo() {
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
                        .text("Tessera Layout Alignment Demo".to_string())
                        .size(Dp(24.0))
                        .build()
                        .unwrap(),
                )
            })
            .into_column_item(),
            // Row - Start Alignment
            (|| {
                text(
                    TextArgsBuilder::default()
                        .text("Row - Start Alignment:".to_string())
                        .build()
                        .unwrap(),
                )
            })
            .into_column_item(),
            (|| {
                surface(
                    SurfaceArgs {
                        color: [0.9, 0.9, 0.9, 1.0],
                        corner_radius: 4.0,
                        padding: Dp(8.0),
                        width: Some(DimensionValue::Fill {
                            min: None,
                            max: None,
                        }),
                        ..Default::default()
                    },
                    None,
                    || {
                        row(
                            RowArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::Start)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .build()
                                .unwrap(),
                            [
                                (|| color_box("A", [0.2, 0.6, 0.9, 1.0])).into_row_item(),
                                (|| color_box("B", [0.9, 0.2, 0.2, 1.0])).into_row_item(),
                                (|| color_box("C", [0.2, 0.9, 0.2, 1.0])).into_row_item(),
                            ],
                        );
                    },
                );
            })
            .into_column_item(),
            // Row - Center Alignment
            (|| {
                text(
                    TextArgsBuilder::default()
                        .text("Row - Center Alignment:".to_string())
                        .build()
                        .unwrap(),
                )
            })
            .into_column_item(),
            (|| {
                surface(
                    SurfaceArgs {
                        color: [0.9, 0.9, 0.9, 1.0],
                        corner_radius: 4.0,
                        padding: Dp(8.0),
                        width: Some(DimensionValue::Fill {
                            min: None,
                            max: None,
                        }),
                        ..Default::default()
                    },
                    None,
                    || {
                        row(
                            RowArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .build()
                                .unwrap(),
                            [
                                (|| color_box("A", [0.2, 0.6, 0.9, 1.0])).into_row_item(),
                                (|| color_box("B", [0.9, 0.2, 0.2, 1.0])).into_row_item(),
                                (|| color_box("C", [0.2, 0.9, 0.2, 1.0])).into_row_item(),
                            ],
                        );
                    },
                );
            })
            .into_column_item(),
            // Row - SpaceEvenly Alignment
            (|| {
                text(
                    TextArgsBuilder::default()
                        .text("Row - SpaceEvenly Alignment:".to_string())
                        .build()
                        .unwrap(),
                )
            })
            .into_column_item(),
            (|| {
                surface(
                    SurfaceArgs {
                        color: [0.9, 0.9, 0.9, 1.0],
                        corner_radius: 4.0,
                        padding: Dp(8.0),
                        width: Some(DimensionValue::Fill {
                            min: None,
                            max: None,
                        }),
                        ..Default::default()
                    },
                    None,
                    || {
                        row(
                            RowArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .build()
                                .unwrap(),
                            [
                                (|| color_box("A", [0.2, 0.6, 0.9, 1.0])).into_row_item(),
                                (|| color_box("B", [0.9, 0.2, 0.2, 1.0])).into_row_item(),
                                (|| color_box("C", [0.2, 0.9, 0.2, 1.0])).into_row_item(),
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
            width: Some(DimensionValue::Fill {
                min: None,
                max: None,
            }),
            ..Default::default()
        },
        None,
        || {
            alignment_demo();
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    println!("Start Simple Layout Alignment Demo");
    println!("Demorowandcolumndifferent alignment options for the component");

    Renderer::run(
        || {
            app();
        },
        |app| {
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
