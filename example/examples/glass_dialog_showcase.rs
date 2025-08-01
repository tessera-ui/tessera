use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, Px, Renderer};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::ColumnArgsBuilder,
    column_ui,
    fluid_glass::{FluidGlassArgsBuilder, fluid_glass},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    glass_dialog::{
        GlassDialogProviderArgsBuilder, GlassDialogProviderState, glass_dialog_provider,
    },
    ripple_state::RippleState,
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    spacer::{SpacerArgsBuilder, spacer},
    text::{TextArgsBuilder, text},
};
use tessera_ui_macros::tessera;

#[derive(Default)]
struct AppState {
    dialog_state: Arc<RwLock<GlassDialogProviderState>>,
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
            glass_button(
                GlassButtonArgsBuilder::default()
                    .on_click(Arc::new(move || {
                        state.write().dialog_state.write().open();
                    }))
                    .tint_color(Color::new(0.2, 0.5, 0.8, 0.5))
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
            fluid_glass(
                FluidGlassArgsBuilder::default()
                    .tint_color(Color::WHITE.with_alpha(content_alpha / 2.0))
                    .blur_radius(10.0 * content_alpha)
                    .shape(Shape::RoundedRectangle {
                        corner_radius: 25.0,
                        g2_k_value: 3.0,
                    })
                    .refraction_height(50.0 * content_alpha)
                    .block_input(true)
                    .padding(Dp(20.0))
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
                                    .text("This is a Glass Dialog".to_string())
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
                            glass_button(
                                GlassButtonArgsBuilder::default()
                                    .tint_color(Color::new(0.2, 0.5, 0.8, content_alpha / 2.0))
                                    .on_click(Arc::new(move || {
                                        state.write().dialog_state.write().close();
                                    }))
                                    .refraction_height(24.0 * content_alpha)
                                    .build()
                                    .unwrap(),
                                close_button_ripple,
                                move || {
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
fn dialog_provider_wrapper(
    app_state: Arc<RwLock<AppState>>,
    image_resource: &tessera_ui_basic_components::pipelines::image::ImageData,
) {
    let state_for_provider = app_state.clone();
    let image_resource = image_resource.clone();
    tessera_ui_basic_components::boxed_ui!(
        tessera_ui_basic_components::boxed::BoxedArgs {
            alignment: tessera_ui_basic_components::alignment::Alignment::Center,
            width: DimensionValue::Fill {
                min: None,
                max: None
            },
            height: DimensionValue::Fill {
                min: None,
                max: None
            },
        },
        move || {
            tessera_ui_basic_components::image::image(
                tessera_ui_basic_components::image::ImageArgsBuilder::default()
                    .data(image_resource.clone())
                    .build()
                    .unwrap(),
            );
        },
        move || {
            glass_dialog_provider(
                GlassDialogProviderArgsBuilder::default()
                    .on_close_request(Arc::new(move || {
                        state_for_provider.write().dialog_state.write().close();
                    }))
                    .blur_radius(20.0)
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
fn app(
    app_state: Arc<RwLock<AppState>>,
    image_resource: &tessera_ui_basic_components::pipelines::image::ImageData,
) {
    dialog_provider_wrapper(app_state, image_resource);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    let app_state = Arc::new(RwLock::new(AppState::default()));

    // 加载图片资源
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
