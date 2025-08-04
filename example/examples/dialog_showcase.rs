use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    column::ColumnArgsBuilder,
    column_ui,
    dialog::{DialogProviderArgsBuilder, DialogProviderState, dialog_provider},
    ripple_state::RippleState,
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default)]
struct AppState {
    dialog_state: Arc<RwLock<DialogProviderState>>,
    button_ripple: Arc<RippleState>,
    close_button_ripple: Arc<RippleState>,
}

#[tessera]
fn dialog_main_content(app_state: Arc<RwLock<AppState>>) {
    let state = app_state.clone();
    let button_ripple = state.read().button_ripple.clone();
    row_ui!(
        RowArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .width(DimensionValue::Fill {
                min: None,
                max: None
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None
            })
            .build()
            .unwrap(),
        || {
            button(
                ButtonArgsBuilder::default()
                    .on_click(Arc::new(move || {
                        state.write().dialog_state.write().open();
                    }))
                    .build()
                    .unwrap(),
                button_ripple,
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Show Dialog".to_string())
                            .build()
                            .unwrap(),
                    )
                },
            );
        }
    );
}

#[tessera]
fn dialog_content(app_state: Arc<RwLock<AppState>>, content_alpha: f32) {
    let state = app_state.clone();
    let close_button_ripple = state.read().close_button_ripple.clone();
    row_ui!(
        RowArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .width(DimensionValue::Fill {
                min: None,
                max: None
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None
            })
            .build()
            .unwrap(),
        move || {
            surface(
                SurfaceArgsBuilder::default()
                    .color(Color::new(0.2, 0.2, 0.2, 1.0).with_alpha(content_alpha))
                    .shape(Shape::RoundedRectangle {
                        corner_radius: 10.0,
                        g2_k_value: 3.0,
                    })
                    .padding(Dp(20.0))
                    .block_input(true)
                    .build()
                    .unwrap(),
                None,
                move || {
                    column_ui!(
                        ColumnArgsBuilder::default().build().unwrap(),
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .color(Color::BLACK.with_alpha(content_alpha))
                                    .text("This is a Dialog".to_string())
                                    .build()
                                    .unwrap(),
                            );
                        },
                        || {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .height(DimensionValue::Fixed(Px(10)))
                                    .build()
                                    .unwrap(),
                            );
                        },
                        move || {
                            button(
                                ButtonArgsBuilder::default()
                                    .color(Color::new(0.2, 0.5, 0.8, content_alpha))
                                    .on_click(Arc::new(move || {
                                        state.write().dialog_state.write().close();
                                    }))
                                    .build()
                                    .unwrap(),
                                close_button_ripple,
                                || {
                                    text(
                                        TextArgsBuilder::default()
                                            .color(Color::BLACK.with_alpha(content_alpha))
                                            .text("Close".to_string())
                                            .build()
                                            .unwrap(),
                                    )
                                },
                            );
                        }
                    );
                },
            );
        }
    );
}

#[tessera]
fn dialog_provider_wrapper(app_state: Arc<RwLock<AppState>>) {
    let state_for_provider = app_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .color(Color::WHITE)
            .build()
            .unwrap(),
        None,
        move || {
            dialog_provider(
                DialogProviderArgsBuilder::default()
                    .on_close_request(Arc::new(move || {
                        state_for_provider.write().dialog_state.write().close();
                    }))
                    .build()
                    .unwrap(),
                app_state.read().dialog_state.clone(),
                {
                    let state = app_state.clone();
                    move || dialog_main_content(state.clone())
                },
                {
                    let state = app_state.clone();
                    move |progress| dialog_content(state.clone(), progress)
                },
            );
        },
    );
}

#[tessera]
fn app(app_state: Arc<RwLock<AppState>>) {
    dialog_provider_wrapper(app_state);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    let app_state = Arc::new(RwLock::new(AppState::default()));

    Renderer::run(
        {
            let app_state = app_state.clone();
            move || app(app_state.clone())
        },
        |renderer| {
            tessera_ui_basic_components::pipelines::register_pipelines(renderer);
        },
    )?;

    Ok(())
}
