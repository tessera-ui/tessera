use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp};
use tessera_ui_basic_components::{
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
};
use tessera_ui_macros::tessera;

use crate::{
    app_state::AppState, component_showcase::component_showcase, material_colors::md_colors,
};

/// Creates the main content area with organized component showcase
#[tessera]
fn main_content(state: Arc<AppState>) {
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE)
            .padding(Dp(16.0))
            .build()
            .unwrap(),
        None,
        move || {
            component_showcase(state.clone());
        },
    );
}

/// Main application component
#[tessera]
pub fn app(state: Arc<AppState>) {
    let scroller_state_clone = state.scrollable_state.clone();
    let state_clone = state.clone();

    surface(
        // Main background surface
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE)
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive background
        move || {
            // Main scrollable container
            scrollable(
                ScrollableArgsBuilder::default().build().unwrap(),
                scroller_state_clone,
                move || {
                    main_content(state_clone.clone());
                },
            );
        },
    );
}
