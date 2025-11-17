use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    pipelines::image::ImageData,
    ripple_state::RippleState,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

#[tessera]
fn app(ripple_state: RippleState, image_resource: &ImageData) {
    let image_resource = image_resource.clone();
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
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            boxed(
                BoxedArgs {
                    alignment: Alignment::Center,
                    width: DimensionValue::Fill {
                        min: None,
                        max: None,
                    },
                    height: DimensionValue::Fill {
                        min: None,
                        max: None,
                    },
                },
                |scope| {
                    scope.child(move || {
                        image(
                            ImageArgsBuilder::default()
                                .data(image_resource)
                                .build()
                                .unwrap(),
                        );
                    });
                    scope.child(move || {
                        let button_args = GlassButtonArgsBuilder::default()
                            .on_click(Arc::new(|| println!("Glass Button 1 clicked!")))
                            .width(DimensionValue::Fixed(Dp(50.0).into()))
                            .height(DimensionValue::Fixed(Dp(50.0).into()))
                            .noise_amount(0.0)
                            .padding(Dp(15.0))
                            .shape(Shape::Ellipse)
                            .contrast(0.6)
                            .build()
                            .unwrap();

                        glass_button(button_args, ripple_state.clone(), move || {});
                    });
                },
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ripple_state = RippleState::new();
    let image_path = format!(
        "{}/examples/assets/scarlet_ut.jpg",
        env!("CARGO_MANIFEST_DIR")
    );
    let image_data = load_image_from_source(&ImageSource::Path(image_path))?;

    Renderer::run(
        {
            move || {
                app(ripple_state.clone(), &image_data);
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
