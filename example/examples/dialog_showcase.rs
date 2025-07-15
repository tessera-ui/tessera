use std::sync::{Arc, RwLock};

use tessera::{Color, DimensionValue, Dp, Px, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    column::ColumnArgsBuilder,
    column_ui,
    dialog::{DialogProviderArgsBuilder, dialog_provider},
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
    show_dialog: bool,
    button_ripple: Arc<RippleState>,
    close_button_ripple: Arc<RippleState>,
}

fn app(app_state: Arc<RwLock<AppState>>) {
    let state_for_provider = app_state.clone();
    let state_for_main_content = app_state.clone();
    let state_for_dialog_content = app_state.clone();

    dialog_provider(
        DialogProviderArgsBuilder::default()
            .is_open(app_state.read().unwrap().show_dialog)
            .on_close_request(Arc::new(move || {
                state_for_provider.write().unwrap().show_dialog = false;
            }))
            .build()
            .unwrap(),
        // Main Content Closure
        move || {
            let button_ripple = state_for_main_content.read().unwrap().button_ripple.clone();
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
                                state_for_main_content.write().unwrap().show_dialog = true;
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
        },
        // Dialog Content Closure
        move || {
            let close_button_ripple = state_for_dialog_content
                .read()
                .unwrap()
                .close_button_ripple
                .clone();
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
                    surface(
                        SurfaceArgsBuilder::default()
                            .color(Color::new(0.2, 0.2, 0.2, 1.0))
                            .shape(Shape::RoundedRectangle {
                                corner_radius: 10.0,
                            })
                            .padding(Dp(20.0))
                            .build()
                            .unwrap(),
                        None,
                        || {
                            column_ui!(
                                ColumnArgsBuilder::default().build().unwrap(),
                                || {
                                    text(
                                        TextArgsBuilder::default()
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
                                || {
                                    button(
                                        ButtonArgsBuilder::default()
                                            .on_click(Arc::new(move || {
                                                state_for_dialog_content
                                                    .write()
                                                    .unwrap()
                                                    .show_dialog = false;
                                            }))
                                            .build()
                                            .unwrap(),
                                        close_button_ripple,
                                        || {
                                            text(
                                                TextArgsBuilder::default()
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
        },
    );
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
            tessera_basic_components::pipelines::register_pipelines(renderer);
        },
    )?;

    Ok(())
}
