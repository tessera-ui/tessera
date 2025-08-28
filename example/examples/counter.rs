//! Simple counter example demonstrating Tessera UI framework basics
//!
//! This example shows how to create a simple interactive counter with:
//! - A button to increment the counter
//! - A display showing the current count
//! - Horizontal layout using row component

use std::sync::{
    Arc,
    atomic::{self, AtomicU32},
};

use tessera_ui::{
    Color, Dp, Renderer, RouteController, renderer::TesseraConfig, router::router_root, shard,
    tessera,
};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    ripple_state::RippleState,
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};

/// Shared application state
struct AppState {
    /// Click counter
    click_count: AtomicU32,
    /// button ripple state
    button_state: Arc<RippleState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            click_count: AtomicU32::new(0),
            button_state: Arc::new(RippleState::new()),
        }
    }
}

/// Main counter application component
#[tessera]
#[shard]
fn counter_app(#[state] app_state: AppState, #[route_controller] controller: RouteController) {
    let button_state_clone = app_state.button_state.clone(); // Renamed for clarity
    let click_count = app_state.click_count.load(atomic::Ordering::Relaxed);
    let app_state_clone = app_state.clone(); // Clone app_state for the button's on_click

    surface(
        SurfaceArgs {
            color: Color::WHITE, // White background
            padding: Dp(25.0),
            ..Default::default()
        },
        None,
        move || {
            row_ui![
                RowArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                move || {
                    button(
                        ButtonArgsBuilder::default()
                            .on_click(Arc::new(move || {
                                // Increment the click count
                                app_state_clone // Use the cloned app_state
                                    .click_count
                                    .fetch_add(1, atomic::Ordering::Relaxed);
                                // Navigate to the counter_app2 route if click_count > 5
                                if app_state_clone.click_count.load(atomic::Ordering::Relaxed) > 5 {
                                    app_state_clone
                                        .click_count
                                        .store(0, atomic::Ordering::Relaxed); // Reset count
                                    controller.push(CounterApp2Destination {});
                                }
                            }))
                            .build()
                            .unwrap(),
                        button_state_clone, // Use the cloned button_state
                        move || text("click me!"),
                    )
                },
                move || {
                    text(
                        TextArgsBuilder::default()
                            .text(format!("Count: {click_count}"))
                            .build()
                            .unwrap(),
                    )
                }
            ];
        },
    );
}

/// Main counter application component, but this one's button is red :)
#[tessera]
#[shard]
fn counter_app2(#[state] app_state: AppState, #[route_controller] controller: RouteController) {
    let button_state_clone = app_state.button_state.clone(); // Renamed for clarity
    let click_count = app_state.click_count.load(atomic::Ordering::Relaxed);
    let app_state_clone = app_state.clone(); // Clone app_state for the button's on_click

    surface(
        SurfaceArgs {
            color: Color::WHITE, // White background
            padding: Dp(25.0),
            ..Default::default()
        },
        None,
        move || {
            row_ui![
                RowArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                move || {
                    button(
                        ButtonArgsBuilder::default()
                            .color(Color::RED) // Set button color to red
                            .on_click(Arc::new(move || {
                                // Increment the click count
                                app_state_clone // Use the cloned app_state
                                    .click_count
                                    .fetch_add(1, atomic::Ordering::Relaxed);
                                // Navigate back to the counter_app route if click_count > 5
                                if app_state_clone.click_count.load(atomic::Ordering::Relaxed) > 5 {
                                    app_state_clone
                                        .click_count
                                        .store(0, atomic::Ordering::Relaxed); // Reset count
                                    controller.pop();
                                }
                            }))
                            .build()
                            .unwrap(),
                        button_state_clone, // Use the cloned button_state
                        move || text("click me!"),
                    )
                },
                move || {
                    text(
                        TextArgsBuilder::default()
                            .text(format!("Count: {click_count}"))
                            .build()
                            .unwrap(),
                    )
                }
            ];
        },
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Starting Counter Example");
    println!("Click the blue button to increment the counter!");
    // Run the application
    let config = TesseraConfig {
        window_title: "Tessera Counter Example".to_string(),
        ..Default::default()
    };
    Renderer::run_with_config(
        {
            move || {
                router_root(CounterAppDestination {});
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
        config,
    )?;

    Ok(())
}
