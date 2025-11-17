use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    ripple_state::RippleState,
    side_bar::{SideBarProviderArgsBuilder, SideBarProviderState, SideBarStyle, side_bar_provider},
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
    pub open_button_state: RippleState,
    pub style_button_state: RippleState,
    pub side_bar_state: SideBarProviderState,
    pub style: ShowcaseStyle,
}

#[tessera]
fn app(state: Arc<RwLock<AppState>>) {
    let style = state.read().style;
    let side_bar_style = match style {
        ShowcaseStyle::Material => SideBarStyle::Material,
        ShowcaseStyle::Glass => SideBarStyle::Glass,
    };

    let side_bar_state = state.read().side_bar_state.clone();

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
        {
            move || {
                side_bar_provider(
                    SideBarProviderArgsBuilder::default()
                        .on_close_request({
                            let side_bar_state = side_bar_state.clone();
                            Arc::new(move || side_bar_state.close())
                        })
                        .style(side_bar_style)
                        .build()
                        .unwrap(),
                    side_bar_state.clone(),
                    {
                        let state = state.clone();
                        move || {
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
                                    scope.child({
                                        let state = state.clone();
                                        move || {
                                            let open_button_state =
                                                state.read().open_button_state.clone();
                                            button(
                                                ButtonArgsBuilder::default()
                                                    .on_click({
                                                        let side_bar_state = side_bar_state.clone();
                                                        Arc::new(move || side_bar_state.open())
                                                    })
                                                    .build()
                                                    .unwrap(),
                                                open_button_state,
                                                || {
                                                    text(
                                                        TextArgsBuilder::default()
                                                            .text("Open Side Bar".to_string())
                                                            .build()
                                                            .unwrap(),
                                                    )
                                                },
                                            )
                                        }
                                    });

                                    scope.child(|| {
                                        spacer(
                                            SpacerArgsBuilder::default()
                                                .height(DimensionValue::Fixed(Px(20)))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child({
                                        let state = state.clone();
                                        move || {
                                            let style_button_state =
                                                state.read().style_button_state.clone();
                                            button(
                                                ButtonArgsBuilder::default()
                                                    .on_click(Arc::new({
                                                        let state = state.clone();
                                                        move || {
                                                            let mut state = state.write();
                                                            state.style = match state.style {
                                                                ShowcaseStyle::Material => {
                                                                    ShowcaseStyle::Glass
                                                                }
                                                                ShowcaseStyle::Glass => {
                                                                    ShowcaseStyle::Material
                                                                }
                                                            };
                                                        }
                                                    }))
                                                    .build()
                                                    .unwrap(),
                                                style_button_state,
                                                {
                                                    let state = state.clone();
                                                    move || {
                                                        let text_content = match state.read().style
                                                        {
                                                            ShowcaseStyle::Material => {
                                                                "Switch to Glass"
                                                            }
                                                            ShowcaseStyle::Glass => {
                                                                "Switch to Material"
                                                            }
                                                        };
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text(text_content.to_string())
                                                                .build()
                                                                .unwrap(),
                                                        )
                                                    }
                                                },
                                            )
                                        }
                                    });
                                },
                            )
                        }
                    },
                    move || {
                        text("Side Bar Content");
                    },
                )
            }
        },
    );
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
