//! Simplest Alignment Test
//!
//! TestCenterAlignment

use tessera::{DimensionValue, Dp, Px, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgs, surface},
};
use tessera_macros::tessera;

/// Main App
#[tessera]
fn app() {
    // Create a fixed-size outer container
    surface(
        SurfaceArgs {
            color: [0.8, 0.8, 0.8, 1.0], // Gray Background
            width: Some(DimensionValue::Fixed(Px(400))),
            height: Some(DimensionValue::Fixed(Px(100))),
            padding: Dp(0.0), // Nonepadding
            ..Default::default()
        },
        None,
        || {
            // Nest another one insidesurface，ForrowProvideFillConstraint
            surface(
                SurfaceArgs {
                    color: [0.8, 0.8, 0.8, 1.0], // Same Gray Background
                    width: Some(DimensionValue::Fill {
                        min: None,
                        max: None,
                    }), // FillWidth
                    height: Some(DimensionValue::Fill {
                        min: None,
                        max: None,
                    }), // FillHeight
                    padding: Dp(0.0),
                    ..Default::default()
                },
                None,
                || {
                    row_ui![
                        RowArgsBuilder::default()
                            .main_axis_alignment(MainAxisAlignment::Center) // Main Axis Alignment Mode
                            .cross_axis_alignment(CrossAxisAlignment::Center) // Cross Axis Alignment Mode
                            .build()
                            .unwrap(),
                        || {
                            surface(
                                SurfaceArgs {
                                    color: [1.0, 0.0, 0.0, 1.0], // Red
                                    width: Some(DimensionValue::Fixed(Px(50))),
                                    height: Some(DimensionValue::Fixed(Px(50))),
                                    ..Default::default()
                                },
                                None,
                                || {},
                            );
                        }
                    ];
                },
            );
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Simplest Alignment Test：A red square should be in the center of the gray container");

    Renderer::run(
        || {
            app();
        },
        |gpu, gpu_queue, config, registry| {
            tessera_basic_components::pipelines::register_pipelines(
                gpu, gpu_queue, config, registry,
            );
        },
    )?;

    Ok(())
}
