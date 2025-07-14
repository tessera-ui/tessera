use std::sync::Arc;

use tessera::{Color, DimensionValue, Dp, Renderer};
use tessera_basic_components::{
    alignment::Alignment,
    boxed::BoxedArgs,
    boxed_ui,
    glass_button::{GlassButtonArgsBuilder, glass_button},
    ripple_state::RippleState,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

#[tessera]
fn app(ripple_state: Arc<RippleState>) {
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
                    let button_args = GlassButtonArgsBuilder::default()
                        .on_click(Arc::new(|| println!("Glass Button 1 clicked!")))
                        .tint_color(Color::from_rgba_u8(0, 0, 255, 25))
                        .width(DimensionValue::Fixed(Dp(200.0).into()))
                        .height(DimensionValue::Fixed(Dp(100.0).into()))
                        .padding(Dp(15.0))
                        .corner_radius(25.0)
                        .inner_shadow_radius(0.0)
                        .highlight_size(0.0)
                        .blur_radius(4.0)
                        .contrast(0.6)
                        .build()
                        .unwrap();

                    glass_button(button_args, ripple_state.clone(), move || {
                        text(
                            TextArgsBuilder::default()
                                .text("Shimmery Button".to_string())
                                .build()
                                .unwrap(),
                        );
                    });
                },
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ripple_state = Arc::new(RippleState::new());

    Renderer::run(
        {
            move || {
                app(ripple_state.clone());
            }
        },
        |app| {
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
