//! Showcase for the Tabs component

use std::sync::Arc;

use parking_lot::Mutex;
use tessera_ui::{
    Color, DimensionValue, Dp,
    renderer::{Renderer, TesseraConfig},
    shard, tessera,
};
use tessera_ui_basic_components::{
    button::{ButtonArgsBuilder, button},
    ripple_state::RippleState,
    surface::{SurfaceArgs, surface},
    tabs::{TabsArgsBuilder, TabsState, tabs},
    text::{TextArgsBuilder, text},
};

/// Shared application state
#[derive(Clone)]
struct AppState {
    /// Ripple states for each tab title button
    title_ripple_states: Vec<Arc<RippleState>>,
    tabs_state: Arc<Mutex<TabsState>>,
}

const NUM_TABS: usize = 3;

impl Default for AppState {
    fn default() -> Self {
        Self {
            title_ripple_states: (0..NUM_TABS)
                .map(|_| Arc::new(RippleState::new()))
                .collect(),
            tabs_state: Arc::new(Mutex::new(TabsState::new(0))),
        }
    }
}

/// Main tabs showcase component
#[tessera]
#[shard]
fn tabs_showcase_app(#[state] app_state: AppState) {
    let active_tab = app_state.tabs_state.lock().active_tab;

    surface(
        SurfaceArgs {
            style: Color::WHITE.into(),
            padding: Dp(25.0),
            width: DimensionValue::FILLED,
            height: DimensionValue::FILLED,
            ..Default::default()
        },
        None,
        move || {
            tabs(
                TabsArgsBuilder::default()
                    .active_tab(active_tab)
                    .state(Some(app_state.tabs_state.clone()))
                    .build()
                    .unwrap(),
                |scope| {
                    for i in 0..NUM_TABS {
                        let app_state_clone = app_state.clone();
                        let title_ripple_state = app_state.title_ripple_states[i].clone();

                        let color = if i == active_tab {
                            Color::new(0.9, 0.9, 0.9, 1.0) // Active tab color
                        } else {
                            Color::TRANSPARENT
                        };

                        scope.tab(
                            move || {
                                surface(
                                    SurfaceArgs {
                                        style: color.into(),
                                        ..Default::default()
                                    },
                                    None,
                                    move || {
                                        button(
                                            ButtonArgsBuilder::default()
                                                .color(Color::TRANSPARENT)
                                                .on_click(Arc::new(move || {
                                                    app_state_clone.tabs_state.lock().set_active_tab(i);
                                                }))
                                                .build()
                                                .unwrap(),
                                            title_ripple_state,
                                            move || {
                                                text(
                                                    TextArgsBuilder::default()
                                                        .text(format!("Tab {}", i + 1))
                                                        .build()
                                                        .unwrap(),
                                                )
                                            },
                                        )
                                    },
                                )
                            },
                            move || {
                                text(
                                    TextArgsBuilder::default()
                                        .text(format!("This is the content for tab {}.\nTry clicking the other tabs!", i + 1))
                                        .build()
                                        .unwrap(),
                                )
                            },
                        );
                    }
                },
            );
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Starting Tabs Showcase Example");

    let config = TesseraConfig {
        window_title: "Tessera Tabs Showcase".to_string(),
        ..Default::default()
    };

    Renderer::run_with_config(
        tabs_showcase_app,
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
        config,
    )?;

    Ok(())
}
