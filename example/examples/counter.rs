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

use tessera::Renderer;
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    row::{AsRowItem, RowArgsBuilder, row},
    surface::{RippleState, SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Shared application state
struct AppState {
    /// Click counter
    click_count: AtomicU32,
    /// Button ripple state
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
        let button_state = app_state.button_state.clone();
        let click_count = app_state.click_count.load(atomic::Ordering::Relaxed);
        surface(SurfaceArgs::default(), None, move || {
            row(
                RowArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                [
                    (move || {
                        button(
                            ButtonArgsBuilder::default()
                                .on_click(Arc::new(move || {
                                    // Increment the click count
                                    app_state
                                        .click_count
                                        .fetch_add(1, atomic::Ordering::Relaxed);
                                }))
                                .build()
                                .unwrap(),
                            button_state,
                            move || text("click me!"),
                        )
                    })
                    .into_row_item(),
                    (move || {
                        text(
                            TextArgsBuilder::default()
                                .text(format!("Count: {}", click_count))
                                .build()
                                .unwrap(),
                        )
                    })
                    .into_row_item(),
                ],
            );
        });
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
    Renderer::run({
        let app_state = app_state.clone();
        move || {
            counter_app(app_state.clone());
        }
    })?;

    Ok(())
}
