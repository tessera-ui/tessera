use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::{ColumnArgsBuilder, column},
    dialog::{DialogProviderArgsBuilder, DialogProviderState, DialogStyle, dialog_provider},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    ripple_state::RippleState,
    row::{RowArgsBuilder, row},
    spacer::{SpacerArgsBuilder, spacer},
    text::{TextArgsBuilder, text},
};

#[derive(Default)]
struct AppState {
    dialog_state: DialogProviderState,
    button_ripple: RippleState,
    close_button_ripple: RippleState,
}

#[tessera]
fn dialog_main_content(app_state: Arc<RwLock<AppState>>) {
    let (button_ripple, dialog_state) = {
        let state = app_state.read();
        (state.button_ripple.clone(), state.dialog_state.clone())
    };
    row(
        RowArgsBuilder::default()
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
                glass_button(
                    GlassButtonArgsBuilder::default()
                        .on_click({
                            let dialog_state = dialog_state.clone();
                            Arc::new(move || {
                                dialog_state.open();
                            })
                        })
                        .tint_color(Color::WHITE.with_alpha(0.3))
                        .build()
                        .unwrap(),
                    button_ripple,
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Show Glass Dialog".to_string())
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
fn dialog_content(app_state: Arc<RwLock<AppState>>, content_alpha: f32) {
    let (close_button_ripple, dialog_state) = {
        let state = app_state.read();
        (
            state.close_button_ripple.clone(),
            state.dialog_state.clone(),
        )
    };
    let close_ripple_for_button = close_button_ripple.clone();
    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
        scope.child(move || {
            text(
                TextArgsBuilder::default()
                    .color(Color::WHITE.with_alpha(content_alpha))
                    .text("This is a glass dialog!".to_string())
                    .size(Dp(20.0))
                    .build()
                    .unwrap(),
            )
        });

        scope.child(|| {
            spacer(
                SpacerArgsBuilder::default()
                    .height(DimensionValue::from(Dp(5.0)))
                    .build()
                    .unwrap(),
            )
        });

        scope.child(move || {
            glass_button(
                GlassButtonArgsBuilder::default()
                    .tint_color(Color::RED.with_alpha(content_alpha / 2.5))
                    .on_click({
                        let dialog_state = dialog_state.clone();
                        Arc::new(move || dialog_state.close())
                    })
                    .refraction_amount(32.0 * content_alpha)
                    .build()
                    .unwrap(),
                close_ripple_for_button,
                move || {
                    text(
                        TextArgsBuilder::default()
                            .color(Color::RED.with_alpha(content_alpha))
                            .text("Close".to_string())
                            .build()
                            .unwrap(),
                    )
                },
            );
        });
    });
}

#[tessera]
fn dialog_provider_wrapper(
    app_state: Arc<RwLock<AppState>>,
    image_resource: &tessera_ui_basic_components::pipelines::image::ImageData,
) {
    let image_resource = image_resource.clone();
    let dialog_state = app_state.read().dialog_state.clone();
    boxed(
        tessera_ui_basic_components::boxed::BoxedArgs {
            alignment: tessera_ui_basic_components::alignment::Alignment::Center,
            width: DimensionValue::Fill {
                min: None,
                max: None,
            },
            height: DimensionValue::Fill {
                min: None,
                max: None,
            },
        },
        |scope| {
            scope.child(move || {
                tessera_ui_basic_components::image::image(
                    tessera_ui_basic_components::image::ImageArgsBuilder::default()
                        .data(image_resource.clone())
                        .build()
                        .unwrap(),
                );
            });
            scope.child(move || {
                dialog_provider(
                    DialogProviderArgsBuilder::default()
                        .on_close_request({
                            let dialog_state = dialog_state.clone();
                            Arc::new(move || dialog_state.close())
                        })
                        .style(DialogStyle::Glass)
                        .build()
                        .unwrap(),
                    dialog_state.clone(),
                    {
                        let state = app_state.clone();
                        move || dialog_main_content(state.clone())
                    },
                    {
                        let state = app_state.clone();
                        move |progress| dialog_content(state.clone(), progress)
                    },
                );
            });
        },
    );
}

#[tessera]
fn app(
    app_state: Arc<RwLock<AppState>>,
    image_resource: &tessera_ui_basic_components::pipelines::image::ImageData,
) {
    dialog_provider_wrapper(app_state, image_resource);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = Arc::new(RwLock::new(AppState::default()));
    let image_path = format!(
        "{}/examples/assets/scarlet_ut.jpg",
        env!("CARGO_MANIFEST_DIR")
    );
    let image_data = tessera_ui_basic_components::image::load_image_from_source(
        &tessera_ui_basic_components::image::ImageSource::Path(image_path),
    )?;

    Renderer::run(
        {
            let app_state = app_state.clone();
            let image_data = image_data.clone();
            move || app(app_state.clone(), &image_data)
        },
        |renderer| {
            tessera_ui_basic_components::pipelines::register_pipelines(renderer);
        },
    )?;

    Ok(())
}
