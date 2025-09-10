use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Px, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    dialog::{DialogProviderArgsBuilder, DialogProviderState, dialog_provider},
    ripple_state::RippleState,
    row::{RowArgsBuilder, row},
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
            scope.child(|| {
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
            });
        },
    );
}

#[tessera]
fn dialog_content(app_state: Arc<RwLock<AppState>>, content_alpha: f32) {
    let state = app_state.clone();
    let close_button_ripple = state.read().close_button_ripple.clone();

    let children: [Box<dyn Fn() + Send + Sync>; _] = [
        Box::new(move || {
            text(
                TextArgsBuilder::default()
                    .color(Color::BLACK.with_alpha(content_alpha))
                    .text("This is a Dialog".to_string())
                    .build()
                    .unwrap(),
            );
        }),
        Box::new(|| {
            spacer(
                SpacerArgsBuilder::default()
                    .height(DimensionValue::Fixed(Px(10)))
                    .build()
                    .unwrap(),
            );
        }),
        Box::new(move || {
            // clone captured Arcs inside this child to avoid moving outer captures
            let state_for_click = state.clone();
            let ripple_for_call = close_button_ripple.clone();
            button(
                ButtonArgsBuilder::default()
                    .color(Color::new(0.2, 0.5, 0.8, content_alpha))
                    .on_click(Arc::new(move || {
                        state_for_click.write().dialog_state.write().close();
                    }))
                    .build()
                    .unwrap(),
                ripple_for_call,
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
        }),
    ];
    column(
        ColumnArgsBuilder::default().build().unwrap(),
        move |scope| {
            for child in children {
                scope.child(child);
            }
        },
    );
}

#[tessera]
fn dialog_provider_wrapper(app_state: Arc<RwLock<AppState>>) {
    let state_for_provider = app_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .style(Color::WHITE.into())
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
