//! An example showcasing the `Boxed` component.
//! This demonstrates how multiple `Surface` components are stacked
//! within a `Boxed` container, with the container's size being
//! determined by the largest child. It also shows how to use
//! the `alignment` property to position the children.

use tessera::{DimensionValue, Px, Renderer};
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
                        width: Some(DimensionValue::Fixed(Px(200))),
                        height: Some(DimensionValue::Fixed(Px(120))),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // A medium green surface in the middle
                || surface(
                    SurfaceArgs {
                        color: [0.2, 1.0, 0.2, 0.8],
                        width: Some(DimensionValue::Fixed(Px(120))),
                        height: Some(DimensionValue::Fixed(Px(80))),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // A small blue surface on top
                || surface(
                    SurfaceArgs {
                        color: [0.2, 0.4, 1.0, 0.7],
                        width: Some(DimensionValue::Fixed(Px(60))),
                        height: Some(DimensionValue::Fixed(Px(40))),
                        ..Default::default()
                    },
                    None,
                    || {},
                ),
                // Add a FluidGlass component to test the multi-pass rendering
                || fluid_glass(
                    FluidGlassArgsBuilder::default()
                        .width(DimensionValue::Fixed(Px(100)))
                        .height(DimensionValue::Fixed(Px(100)))
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
