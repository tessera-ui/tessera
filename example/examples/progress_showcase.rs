use parking_lot::Mutex;
use std::sync::Arc;
use tessera::{Color, Dp, renderer::Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgsBuilder, column_ui},
    progress::{ProgressArgsBuilder, progress},
    slider::{SliderArgsBuilder, SliderState, slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

struct AppState {
    value: Arc<Mutex<f32>>,
    slider_state: Arc<Mutex<SliderState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0.5)),
            slider_state: Arc::new(Mutex::new(SliderState::new())),
        }
    }
}

#[tessera]
fn app(state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::new(0.1, 0.1, 0.1, 1.0))
            .build()
            .unwrap(),
        None,
        {
            let state_for_column = state.clone();
            move || {
                let on_change = {
                    let state = state_for_column.clone();
                    Arc::new(move |new_value| {
                        *state.value.lock() = new_value;
                    })
                };

                column_ui!(
                    ColumnArgsBuilder::default()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .build()
                        .unwrap(),
                    {
                        let state = state_for_column.clone();
                        move || {
                            progress(
                                ProgressArgsBuilder::default()
                                    .value(*state.value.lock())
                                    .build()
                                    .unwrap(),
                            )
                        }
                    },
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera::DimensionValue::Fixed(Dp(20.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    {
                        let state = state_for_column.clone();
                        move || {
                            slider(
                                SliderArgsBuilder::default()
                                    .value(*state.value.lock())
                                    .on_change(on_change)
                                    .build()
                                    .unwrap(),
                                state.slider_state.clone(),
                            )
                        }
                    },
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera::DimensionValue::Fixed(Dp(10.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    {
                        let state = state_for_column.clone();
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(format!("Value: {:.2}", *state.value.lock()))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    },
                )
            }
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
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
