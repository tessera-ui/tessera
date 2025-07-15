use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::BoxedArgs,
    boxed_ui,
    glass_button::{GlassButtonArgsBuilder, glass_button},
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    pipelines::image::ImageData,
    ripple_state::RippleState,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};
use tessera_ui_macros::tessera;

#[tessera]
fn app(ripple_state: Arc<RippleState>, image_resource: &ImageData) {
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
            .color(Color::WHITE)
            .build()
            .unwrap(),
        None,
        move || {
            boxed_ui!(
                BoxedArgs {
                    alignment: Alignment::Center,
                    width: DimensionValue::Fill {
                        min: None,
                        max: None
                    },
                    height: DimensionValue::Fill {
                        min: None,
                        max: None
                    },
                },
                move || {
                    image(
                        ImageArgsBuilder::default()
                            .data(image_resource)
                            .build()
                            .unwrap(),
                    );
                },
                move || {
                    let button_args = GlassButtonArgsBuilder::default()
                        .on_click(Arc::new(|| println!("Glass Button 1 clicked!")))
                        .tint_color(Color::GREEN.with_alpha(0.2))
                        .width(DimensionValue::Fixed(Dp(50.0).into()))
                        .height(DimensionValue::Fixed(Dp(50.0).into()))
                        .noise_amount(0.0)
                        .padding(Dp(15.0))
                        .shape(Shape::Ellipse)
                        .inner_shadow_radius(0.0)
                        .highlight_size(0.0)
                        .contrast(0.6)
                        .build()
                        .unwrap();

                    glass_button(button_args, ripple_state.clone(), move || {});
                },
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ripple_state = Arc::new(RippleState::new());
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
