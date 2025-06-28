//! An example showcasing the `Boxed` component.
//! This demonstrates how multiple `Surface` components are stacked
//! within a `Boxed` container, with the container's size being
//! determined by the largest child. It also shows how to use
//! the `alignment` property to position the children.

use tessera::{DimensionValue, Dp, Renderer};
use tessera_basic_components::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed_ui},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    surface::{SurfaceArgs, surface},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    Renderer::run(
        || {
            boxed_ui!(
                BoxedArgs {
                    alignment: Alignment::Center,
                },
                // A large red surface at the bottom
                || surface(
                    SurfaceArgs {
                        color: [1.0, 0.2, 0.2, 1.0],
                        width: Some(DimensionValue::Fixed(Dp(1000.0).into())),
                        height: Some(DimensionValue::Fixed(Dp(600.0).into())),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // A medium green surface in the middle
                || surface(
                    SurfaceArgs {
                        color: [0.2, 1.0, 0.2, 0.8],
                        width: Some(DimensionValue::Fixed(Dp(600.0).into())),
                        height: Some(DimensionValue::Fixed(Dp(400.0).into())),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // A small blue surface on top
                || surface(
                    SurfaceArgs {
                        color: [0.2, 0.4, 1.0, 0.7],
                        width: Some(DimensionValue::Fixed(Dp(300.0).into())),
                        height: Some(DimensionValue::Fixed(Dp(200.0).into())),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // Add a FluidGlass component to test the multi-pass rendering
                || fluid_glass(
                    FluidGlassArgsBuilder::default()
                        .width(DimensionValue::Fixed(Dp(500.0).into()))
                        .height(DimensionValue::Fixed(Dp(500.0).into()))
                        .build()
                        .unwrap(),
                ),
                // Add a FluidGlass component to test the multi-pass rendering
                || fluid_glass(
                    FluidGlassArgsBuilder::default()
                        .width(DimensionValue::Fixed(Dp(280.0).into()))
                        .height(DimensionValue::Fixed(Dp(280.0).into()))
                        .build()
                        .unwrap(),
                ),
            );
        },
        |app| {
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
