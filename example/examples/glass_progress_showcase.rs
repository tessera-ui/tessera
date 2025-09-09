use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{Color, DimensionValue, Dp, renderer::Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgsBuilder, column},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderState, glass_slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

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
            .style(Color::WHITE.into())
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

                let children: [Box<dyn Fn() + Send + Sync>; 9] = [
                    Box::new({
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
                    }),
                    Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Glass Progress ↑".to_string())
                                .color(Color::WHITE)
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new({
                        let state = state_for_column.clone();
                        move || {
                            glass_slider(
                                GlassSliderArgsBuilder::default()
                                    .value(*state.value.lock())
                                    .on_change(on_change.clone())
                                    .width(Dp(250.0))
                                    .build()
                                    .unwrap(),
                                state.slider_state.clone(),
                            )
                        }
                    }),
                    Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(tessera_ui::DimensionValue::Fixed(Dp(10.0).to_px()))
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Glass Slider ↑".to_string())
                                .color(Color::WHITE)
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(tessera_ui::DimensionValue::Fixed(Dp(20.0).to_px()))
                                .build()
                                .unwrap(),
                        )
                    }),
                    Box::new({
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
                    }),
                ];

                column(
                    ColumnArgsBuilder::default()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .build()
                        .unwrap(),
                    |scope| {
                        for child in children {
                            scope.child(child);
                        }
                    },
                )
            }
        },
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
