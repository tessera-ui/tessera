use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::Alignment,
    boxed::{BoxedArgs, boxed},
    glass_switch::{GlassSwitchArgsBuilder, GlassSwitchState, glass_switch},
    surface::{SurfaceArgsBuilder, surface},
};

#[tessera]
fn app(switch_state: Arc<Mutex<GlassSwitchState>>) {
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
                        let args = GlassSwitchArgsBuilder::default()
                            .state(Some(switch_state.clone()))
                            .checked(switch_state.lock().checked)
                            .on_toggle(Arc::new(|on| {
                                if on {
                                    println!("Glass Switch toggled to OFF");
                                } else {
                                    println!("Glass Switch toggled to ON");
                                }
                            }))
                            .width(Dp(72.0))
                            .height(Dp(40.0))
                            .track_on_color(Color::new(0.2, 0.7, 1.0, 0.5))
                            .track_off_color(Color::new(0.8, 0.8, 0.8, 0.5))
                            .build()
                            .unwrap();
                        glass_switch(args);
                    });
                },
            )
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let switch_state = Arc::new(Mutex::new(GlassSwitchState::new(false)));
    Renderer::run(
        {
            let switch_state = switch_state.clone();
            move || {
                app(switch_state.clone());
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
