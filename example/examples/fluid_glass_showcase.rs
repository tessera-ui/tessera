//! Fluid Glass Showcase

use tessera_ui::{Color, DimensionValue, Dp, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// Create a small colored box
#[tessera]
fn small_box(color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .style(color.into())
            .shape(Shape::RoundedRectangle {
                top_left: Dp(25.0),
                top_right: Dp(25.0),
                bottom_right: Dp(25.0),
                bottom_left: Dp(25.0),
                g2_k_value: 3.0,
            })
            .padding(Dp(8.0))
            .width(DimensionValue::Fixed(Px(40)))
            .height(DimensionValue::Fixed(Px(40)))
            .build()
            .unwrap(),
        None,
        move || {},
    );
}

/// Main App
#[tessera]
fn app(image_resource: &ImageData) {
    let image_resource = image_resource.clone();
    // A surface to hold everything
    surface(
        SurfaceArgsBuilder::default()
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
        None,
        || {
            // Use boxed to stack background content and fluid glass
            boxed(
                BoxedArgs {
                    alignment: Alignment::Center,
                    ..Default::default()
                },
                |scope| {
                    // Background content layer
                    scope.child(move || {
                        image(
                            ImageArgsBuilder::default()
                                .data(image_resource)
                                .build()
                                .unwrap(),
                        );
                    });
                    // Fluid glass overlay
                    scope.child(move || {
                        row(
                            RowArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .width(DimensionValue::Fill {
                                    min: None,
                                    max: None,
                                })
                                .build()
                                .unwrap(),
                            |scope| {
                                scope.child(move || {
                                    fluid_glass(
                                        FluidGlassArgsBuilder::default()
                                            .width(DimensionValue::Fixed(Px(350)))
                                            .height(DimensionValue::Fixed(Px(250)))
                                            .shape(Shape::RoundedRectangle {
                                                top_left: Dp(20.0),
                                                top_right: Dp(20.0),
                                                bottom_right: Dp(20.0),
                                                bottom_left: Dp(20.0),
                                                g2_k_value: 3.0,
                                            })
                                            .refraction_amount(50.0)
                                            .tint_color(Color::TRANSPARENT)
                                            .build()
                                            .unwrap(),
                                        None,
                                        || {},
                                    )
                                });
                                scope.child(move || {
                                    fluid_glass(
                                        FluidGlassArgsBuilder::default()
                                            .blur_radius(10.0)
                                            .width(DimensionValue::Fixed(Px(350)))
                                            .height(DimensionValue::Fixed(Px(250)))
                                            .shape(Shape::RoundedRectangle {
                                                top_left: Dp(20.0),
                                                top_right: Dp(20.0),
                                                bottom_right: Dp(20.0),
                                                bottom_left: Dp(20.0),
                                                g2_k_value: 3.0,
                                            })
                                            .tint_color(Color::TRANSPARENT)
                                            .build()
                                            .unwrap(),
                                        None,
                                        || {},
                                    )
                                });
                            },
                        )
                    });
                },
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_path = format!(
        "{}/examples/assets/scarlet_ut.jpg",
        env!("CARGO_MANIFEST_DIR")
    );
    let image_data = load_image_from_source(&ImageSource::Path(image_path))?;
    Renderer::run(
        {
            move || {
                app(&image_data);
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
