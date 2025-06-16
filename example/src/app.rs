use std::sync::Arc;

use tessera::DimensionValue;
use tessera_basic_components::{
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
};
use tessera_macros::tessera;

use crate::{app_state::AppState, component_showcase::component_showcase};

/// Creates the main content area with organized component showcase
#[tessera]
fn main_content(state: Arc<AppState>) {
    surface(
        // Main background surface
        SurfaceArgsBuilder::default()
            .color([0.15, 0.15, 0.2, 1.0]) // Darker, more elegant background
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
        move || {
            // Use the new organized component showcase
            component_showcase(state.clone());
        },
    )
}

/// Main application component
#[tessera]
pub fn app(state: Arc<AppState>) {
    let scroller_state_clone = state.scrollable_state.clone();
    let state_clone = state.clone();

    // Main scrollable container
    scrollable(
        ScrollableArgsBuilder::default().build().unwrap(),
        scroller_state_clone,
        move || {
            main_content(state_clone.clone());
        },
    );
}
