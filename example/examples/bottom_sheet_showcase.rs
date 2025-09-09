use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    bottom_sheet::{
        BottomSheetProviderArgsBuilder, BottomSheetProviderState, BottomSheetStyle,
        bottom_sheet_provider,
    },
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    ripple_state::RippleState,
    row::{RowArgsBuilder, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default, Clone, Copy, PartialEq)]
enum ShowcaseStyle {
    #[default]
    Material,
    Glass,
}

#[derive(Default)]
struct AppState {
    bottom_sheet_state: Arc<RwLock<BottomSheetProviderState>>,
    button_ripple: Arc<RippleState>,
    close_button_ripple: Arc<RippleState>,
    style_button_ripple: Arc<RippleState>,
    style: ShowcaseStyle,
}

#[tessera]
fn bottom_sheet_main_content(app_state: Arc<RwLock<AppState>>) {
    let state = app_state.clone();
    let button_ripple = state.read().button_ripple.clone();
    let style_button_ripple = state.read().style_button_ripple.clone();

    column(
        ColumnArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        |scope| {
            scope.child(move || {
                button(
                    ButtonArgsBuilder::default()
                        .on_click(Arc::new(move || {
                            state.write().bottom_sheet_state.write().open();
                        }))
                        .build()
                        .unwrap(),
                    button_ripple,
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Show Bottom Sheet".to_string())
                                .build()
                                .unwrap(),
                        )
                    },
                );
            });
            scope.child(|| {
                spacer(
                    SpacerArgsBuilder::default()
                        .height(DimensionValue::Fixed(Px(20)))
                        .build()
                        .unwrap(),
                )
            });
            scope.child(move || {
                let state = app_state.clone();
                button(
                    ButtonArgsBuilder::default()
                        .on_click(Arc::new(move || {
                            let mut state = state.write();
                            state.style = match state.style {
                                ShowcaseStyle::Material => ShowcaseStyle::Glass,
                                ShowcaseStyle::Glass => ShowcaseStyle::Material,
                            };
                        }))
                        .build()
                        .unwrap(),
                    style_button_ripple,
                    move || {
                        let state = app_state.clone();
                        let text_content = match state.read().style {
                            ShowcaseStyle::Material => "Switch to Glass",
                            ShowcaseStyle::Glass => "Switch to Material",
                        };
                        text(
                            TextArgsBuilder::default()
                                .text(text_content.to_string())
                                .build()
                                .unwrap(),
                        )
                    },
                );
            });
        },
    );
}

#[tessera]
fn bottom_sheet_content(app_state: Arc<RwLock<AppState>>) {
    let state = app_state.clone();
    let close_button_ripple = state.read().close_button_ripple.clone();
    row(
        RowArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::End)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        |scope| {
            scope.child({
                let state = app_state.clone();
                move || {
                    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                        scope.child(move || {
                            text(
                                TextArgsBuilder::default()
                                    .color(Color::BLACK)
                                    .text("This is a Bottom Sheet".to_string())
                                    .build()
                                    .unwrap(),
                            );
                        });
                        scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .height(DimensionValue::Fixed(Px(10)))
                                    .build()
                                    .unwrap(),
                            );
                        });
                        scope.child(move || {
                            glass_button(
                                GlassButtonArgsBuilder::default()
                                    .tint_color(Color::new(0.2, 0.5, 0.8, 0.3))
                                    .on_click(Arc::new(move || {
                                        state.write().bottom_sheet_state.write().close();
                                    }))
                                    .build()
                                    .unwrap(),
                                close_button_ripple,
                                move || {
                                    text(
                                        TextArgsBuilder::default()
                                            .color(Color::BLACK)
                                            .text("Close".to_string())
                                            .build()
                                            .unwrap(),
                                    )
                                },
                            );
                        });
                    });
                }
            });
        },
    );
}

#[tessera]
fn bottom_sheet_provider_wrapper(app_state: Arc<RwLock<AppState>>) {
    let state_for_provider = app_state.clone();
    let style = match app_state.read().style {
        ShowcaseStyle::Material => BottomSheetStyle::Material,
        ShowcaseStyle::Glass => BottomSheetStyle::Glass,
    };
    surface(
        SurfaceArgsBuilder::default()
            .style(Color::WHITE.into())
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None,
        move || {
            bottom_sheet_provider(
                BottomSheetProviderArgsBuilder::default()
                    .on_close_request(Arc::new(move || {
                        state_for_provider
                            .write()
                            .bottom_sheet_state
                            .write()
                            .close();
                    }))
                    .style(style)
                    .build()
                    .unwrap(),
                app_state.read().bottom_sheet_state.clone(),
                {
                    let state = app_state.clone();
                    move || bottom_sheet_main_content(state.clone())
                },
                {
                    let state = app_state.clone();
                    move || bottom_sheet_content(state.clone())
                },
            );
        },
    );
}

#[tessera]
fn app(app_state: Arc<RwLock<AppState>>) {
    bottom_sheet_provider_wrapper(app_state);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
