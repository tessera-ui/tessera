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

use tessera::{Dp, Renderer};
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    row::RowArgsBuilder,
    row_ui,
    surface::{RippleState, SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Shared application state
struct AppState {
    /// Click counter
    click_count: AtomicU32,
    /// button ripple state
    button_state: Arc<RippleState>,
}

impl AppState {
    fn new() -> Self {
        Self {
            click_count: AtomicU32::new(0),
            button_state: Arc::new(RippleState::new()),
        }
    }
}

/// Main counter application component
#[tessera]
fn counter_app(app_state: Arc<AppState>) {
    {
        let button_state_clone = app_state.button_state.clone(); // Renamed for clarity
        let click_count = app_state.click_count.load(atomic::Ordering::Relaxed);
        let app_state_clone = app_state.clone(); // Clone app_state for the button's on_click

        surface(
            SurfaceArgs {
                color: [1.0, 1.0, 1.0, 1.0], // White background
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = flexi_logger::Logger::try_with_env()?
        .write_mode(flexi_logger::WriteMode::Async)
        .start()?;

    // Create application state
    let app_state = Arc::new(AppState::new());

    println!("Starting Counter Example");
    println!("Click the blue button to increment the counter!");

    // Run the application
    Renderer::run(
        {
            let app_state_main = app_state.clone(); // Clone for the main app loop
            move || {
                counter_app(app_state_main.clone());
            }
        },
        |app| {
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
