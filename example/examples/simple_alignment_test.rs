//! Simple Alignment Test
//!
//! Direct Testrowalignment functionality of the component

use tessera::{DimensionValue, Dp, Px, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    row::{RowArgsBuilder, row},
    surface::{SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Create a small colored box
#[tessera]
fn small_box(text_content: &'static str, color: [f32; 4]) {
    surface(
        SurfaceArgs {
            color,
            corner_radius: 4.0,
            padding: Dp(4.0),
            width: Some(DimensionValue::Fixed(Px(40))),
            height: Some(DimensionValue::Fixed(Px(40))),
            ..Default::default()
        },
        None,
        move || {
            text(
                TextArgsBuilder::default()
                    .text(text_content.to_string())
                    .color([255, 255, 255])
                    .size(Dp(12.0))
                    .build()
                    .unwrap(),
            )
        },
    );
}

/// Main App
#[tessera]
fn app() {
    surface(
        SurfaceArgs {
            color: [1.0, 1.0, 1.0, 1.0], // White Background
            padding: Dp(20.0),
            ..Default::default()
        },
        None,
        || {
            // Directly create a large fixed-width container for testingrowAlignment
            surface(
                SurfaceArgs {
                    color: [0.9, 0.9, 0.9, 1.0], // Gray background to show container borders
                    corner_radius: 4.0,
                    padding: Dp(10.0),
                    width: Some(DimensionValue::Fixed(Px(800))), // Very Large Fixed Width
                    height: Some(DimensionValue::Fixed(Px(100))), // Fixed Height
                    ..Default::default()
                },
                None,
                || {
                    row(
                        RowArgsBuilder::default()
                            .main_axis_alignment(MainAxisAlignment::End) // TestEndAlignment
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .build()
                            .unwrap(),
                        [Box::new(|| small_box("X", [0.9, 0.2, 0.2, 1.0]))
                            as Box<dyn FnOnce() + Send + Sync>],
                    );
                },
            );
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    println!("Simple Alignment Test - Two boxes should be displayed on the right");

    Renderer::run(|| {
        app();
    })?;

    Ok(())
}
