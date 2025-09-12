//! Showcase for the Tabs component

use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    Color, DimensionValue, Dp,
    renderer::{Renderer, TesseraConfig},
    shard, tessera,
};
use tessera_ui_basic_components::{
    surface::{SurfaceArgs, surface},
    tabs::{TabsArgsBuilder, TabsState, tabs},
    text::{TextArgsBuilder, text},
};

/// Shared application state
#[derive(Default)]
struct AppState {
    tabs_state: Arc<RwLock<TabsState>>,
}

const NUM_TABS: usize = 5;

/// Main tabs showcase component
#[tessera]
#[shard]
fn tabs_showcase_app(#[state] app_state: AppState) {
    let tabs_state = app_state.tabs_state.clone();
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
                TabsArgsBuilder::default().build().unwrap(),
                tabs_state,
                |scope| {
                    for i in 0..NUM_TABS {
                        scope.child(
                            move || {
                                text(
                                    TextArgsBuilder::default()
                                        .text(format!("Tab {}", i + 1))
                                        .build()
                                        .unwrap(),
                                );
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
