use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{Color, DimensionValue, Dp, renderer::Renderer};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgsBuilder, column_ui},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderState, glass_slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_ui_macros::tessera;

struct AppState {
    value: Arc<Mutex<f32>>,
    slider_state: Arc<Mutex<GlassSliderState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            value: Arc::new(Mutex::new(0.5)),
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
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
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
                            glass_progress(
                                GlassProgressArgsBuilder::default()
                                    .value(*state.value.lock())
                                    .width(Dp(250.0))
                                    .height(Dp(16.0))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    },
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    || text(
                        TextArgsBuilder::default()
                            .text("Glass Progress ↑".to_string())
                            .color(Color::WHITE)
                            .build()
                            .unwrap()
                    ),
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    {
                        let state = state_for_column.clone();
                        move || {
                            glass_slider(
                                GlassSliderArgsBuilder::default()
                                    .value(*state.value.lock())
                                    .on_change(on_change)
                                    .width(Dp(250.0))
                                    .build()
                                    .unwrap(),
                                state.slider_state.clone(),
                            )
                        }
                    },
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera_ui::DimensionValue::Fixed(Dp(10.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    || text(
                        TextArgsBuilder::default()
                            .text("Glass Slider ↑".to_string())
                            .color(Color::WHITE)
                            .build()
                            .unwrap()
                    ),
                    || spacer(
                        SpacerArgsBuilder::default()
                            .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                            .build()
                            .unwrap()
                    ),
                    {
                        let state = state_for_column.clone();
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(format!("Value: {:.2}", *state.value.lock()))
                                    .color(Color::WHITE)
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
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
