//! An example showcasing the `Boxed` component.
//!
//! This example demonstrates:
//! 1. Stacking multiple `surface` components within a `Boxed` container.
//! 2. How the `Boxed` container's size is determined by its largest child.
//! 3. Using the `alignment` property to position children within the container.
//! 4. Stacking `fluid_glass` components to test multi-pass rendering.

use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    surface::{SurfaceArgs, surface},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Renderer::run(
        || {
            boxed(
                BoxedArgs {
                    alignment: Alignment::Center,
                    ..Default::default()
                },
                |scope| {
                    // A large red surface at the bottom
                    scope.child(|| {
                        surface(
                            SurfaceArgs {
                                style: Color::new(1.0, 0.2, 0.2, 1.0).into(),
                                width: DimensionValue::Fixed(Dp(1000.0).into()),
                                height: DimensionValue::Fixed(Dp(600.0).into()),
                                ..Default::default()
                            },
                            None,
                            || {},
                        )
                    });
                    // A medium green surface in the middle
                    scope.child(|| {
                        surface(
                            SurfaceArgs {
                                style: Color::new(0.2, 1.0, 0.2, 0.8).into(),
                                width: DimensionValue::Fixed(Dp(600.0).into()),
                                height: DimensionValue::Fixed(Dp(400.0).into()),
                                ..Default::default()
                            },
                            None,
                            || {},
                        )
                    });
                    // A small blue surface on top
                    scope.child(|| {
                        surface(
                            SurfaceArgs {
                                style: Color::new(0.2, 0.4, 1.0, 0.7).into(),
                                width: DimensionValue::Fixed(Dp(300.0).into()),
                                height: DimensionValue::Fixed(Dp(200.0).into()),
                                ..Default::default()
                            },
                            None,
                            || {},
                        )
                    });
                    // Add multiple FluidGlass components to test multi-pass rendering.
                    scope.child(|| {
                        fluid_glass(
                            FluidGlassArgsBuilder::default()
                                .blur_radius(5.0)
                                .width(DimensionValue::Fixed(Dp(500.0).into()))
                                .height(DimensionValue::Fixed(Dp(500.0).into()))
                                .contrast(1.5)
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    });
                    scope.child(|| {
                        fluid_glass(
                            FluidGlassArgsBuilder::default()
                                .blur_radius(5.0)
                                .width(DimensionValue::Fixed(Dp(250.0).into()))
                                .height(DimensionValue::Fixed(Dp(250.0).into()))
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        )
                    });
                },
            );
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
