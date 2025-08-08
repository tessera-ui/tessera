use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::MainAxisAlignment,
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    slider::{SliderArgsBuilder, SliderState, slider},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

struct AppState {
    slider_value: Arc<Mutex<f32>>,
    slider_state: Arc<Mutex<SliderState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            slider_value: Arc::new(Mutex::new(0.5)),
            slider_state: Arc::new(Mutex::new(SliderState::new())),
        }
    }
}

#[tessera]
fn app(state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::WHITE)
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
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
                            slider(
                                SliderArgsBuilder::default()
                                    .value(value)
                                    .on_change(on_change_clone)
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
                            .text("Slide me!".to_string())
                            .build()
                            .unwrap(),
                    )
                }
            )
        },
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

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
