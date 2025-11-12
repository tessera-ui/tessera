use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::MainAxisAlignment,
    column::{ColumnArgsBuilder, column},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderState, glass_slider},
    row::{RowArgsBuilder, row},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

struct AppState {
    slider_value: Arc<Mutex<f32>>,
    slider_state: Arc<RwLock<GlassSliderState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            slider_value: Arc::new(Mutex::new(0.5)),
            slider_state: Default::default(),
        }
    }
}

#[tessera]
fn app(state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .style(Color::WHITE.into())
            .padding(Dp(20.0))
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        None,
        move || {
            let value = *state.slider_value.lock();
            let on_change = {
                let state = state.clone();
                Arc::new(move |new_value| {
                    *state.slider_value.lock() = new_value;
                })
            };

            let children: [Box<dyn Fn() + Send + Sync>; 2] = [
                Box::new(move || {
                    let on_change_clone = on_change.clone();
                    let state_clone = state.clone();
                    row(
                        RowArgsBuilder::default()
                            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                            .cross_axis_alignment(
                                tessera_ui_basic_components::alignment::CrossAxisAlignment::Center,
                            )
                            .width(tessera_ui::DimensionValue::Fixed(Dp(300.0).to_px()))
                            .build()
                            .unwrap(),
                        |scope| {
                            scope.child(move || {
                                glass_slider(
                                    GlassSliderArgsBuilder::default()
                                        .value(value)
                                        .on_change(on_change_clone)
                                        .track_tint_color(Color::new(0.3, 0.3, 0.3, 0.15))
                                        .progress_tint_color(Color::new(0.2, 0.7, 1.0, 0.25))
                                        .blur_radius(Dp(8.0))
                                        .build()
                                        .unwrap(),
                                    state_clone.slider_state.clone(),
                                )
                            });
                            scope.child(move || {
                                text(
                                    TextArgsBuilder::default()
                                        .text(format!("{value:.2}"))
                                        .build()
                                        .unwrap(),
                                )
                            });
                        },
                    )
                }),
                Box::new(move || {
                    text(
                        TextArgsBuilder::default()
                            .text("Glass Slider Demo".to_string())
                            .build()
                            .unwrap(),
                    )
                }),
            ];

            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .build()
                    .unwrap(),
                |scope| {
                    for child in children {
                        scope.child(child);
                    }
                },
            )
        },
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState::new());

    Renderer::run(
        {
            let app_state_main = app_state.clone();
            move || {
                app(app_state_main.clone());
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
