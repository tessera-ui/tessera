use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{Color, Dp, Renderer};
use tessera_ui_basic_components::{
    alignment::MainAxisAlignment,
    column::ColumnArgsBuilder,
    column_ui,
    glass_slider::{GlassSliderArgsBuilder, GlassSliderState, glass_slider},
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_ui_macros::tessera;

struct AppState {
    slider_value: Arc<Mutex<f32>>,
    slider_state: Arc<Mutex<GlassSliderState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            slider_value: Arc::new(Mutex::new(0.5)),
            slider_state: Arc::new(Mutex::new(GlassSliderState::new())),
        }
    }
}

#[tessera]
fn app(state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::WHITE)
            .padding(Dp(20.0))
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

            column_ui!(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .build()
                    .unwrap(),
                move || {
                    let on_change_clone = on_change.clone();
                    let state_clone = state.clone();
                    row_ui!(
                        RowArgsBuilder::default()
                            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                            .cross_axis_alignment(
                                tessera_ui_basic_components::alignment::CrossAxisAlignment::Center
                            )
                            .width(tessera_ui::DimensionValue::Fixed(Dp(300.0).to_px()))
                            .build()
                            .unwrap(),
                        move || {
                            glass_slider(
                                GlassSliderArgsBuilder::default()
                                    .value(value)
                                    .on_change(on_change_clone)
                                    .track_tint_color(Color::new(0.3, 0.3, 0.3, 0.15))
                                    .progress_tint_color(Color::new(0.2, 0.7, 1.0, 0.25))
                                    .blur_radius(8.0)
                                    .build()
                                    .unwrap(),
                                state_clone.slider_state.clone(),
                            )
                        },
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(format!("{value:.2}"))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    )
                },
                move || {
                    text(
                        TextArgsBuilder::default()
                            .text("Glass Slider Demo".to_string())
                            .build()
                            .unwrap(),
                    )
                }
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
