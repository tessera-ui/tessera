use std::sync::Arc;
use tessera::{DimensionValue, Dp};
use tessera_basic_components::{
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::{
    animated_spacer::anim_spacer,
    app_state::AppState,
    interactive_demo::interactive_demo,
    layout_examples::{outlined_surface_example, transparent_surface_example},
    material_colors::md_colors,
    misc::create_spacer,
    performance_display::perf_display,
    text_editors::{text_editor_1, text_editor_2},
};

/// surface examples showcase
#[tessera]
fn surface_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Wrap {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive container
        || {
            column_ui!(
                ColumnArgsBuilder::default().build().unwrap(),
                // Title inside the card
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("surface Components".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    )
                },
                // Spacer
                || (create_spacer(12))(),
                // Content
                || {
                    row_ui![
                        RowArgsBuilder::default().build().unwrap(),
                        || outlined_surface_example(),
                        || (create_spacer(20))(),
                        || transparent_surface_example()
                    ]
                }
            )
        },
    )
}

/// text editor showcase
#[tessera]
fn text_editor_showcase(state: Arc<AppState>) {
    let editor_state_clone = state.text_editors_state.editor_state.clone();
    let editor_state_2_clone = state.text_editors_state.editor_state_2.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER)
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive container
        move || {
            column_ui!(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                // Title inside the card
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("text Editor Components".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    )
                },
                // Spacer
                || (create_spacer(12))(),
                // Content
                move || text_editor_1(editor_state_clone.clone()),
                || (create_spacer(16))(),
                move || text_editor_2(editor_state_2_clone.clone())
            )
        },
    )
}

/// Animation showcase
#[tessera]
fn animation_showcase(state: Arc<AppState>) {
    let anim_state_clone = state.anim_spacer_state.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive
        move || {
            column_ui!(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                // Title inside the card
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Animation Components".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    )
                },
                || (create_spacer(12))(),
                // Content
                || text("Animated Spacer:"),
                || (create_spacer(8))(),
                move || anim_spacer(anim_state_clone.clone()),
                || (create_spacer(8))(),
                || text("↑ Height animation effect")
            )
        },
    )
}

/// Interactive components showcase
#[tessera]
fn interactive_showcase(state: Arc<AppState>) {
    let state_clone = state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive container
        move || interactive_demo(state_clone.clone()),
    )
}

/// Performance showcase
#[tessera]
fn performance_showcase(state: Arc<AppState>) {
    let metrics_clone = state.metrics.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
            .corner_radius(25.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive
        move || {
            column_ui!(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                // Title inside the card
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Performance Monitoring".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    )
                },
                || (create_spacer(12))(),
                // Content
                move || perf_display(metrics_clone.clone())
            )
        },
    )
}

/// Main component showcase that organizes all components
#[tessera]
pub fn component_showcase(state: Arc<AppState>) {
    column_ui!(
        ColumnArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        // Welcome section
        || {
            surface(
                SurfaceArgsBuilder::default()
                    .color(md_colors::PRIMARY_CONTAINER)
                    .corner_radius(25.0)
                    .padding(Dp(24.0))
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                None, // Non-interactive
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Tessera UI Framework Component Showcase".to_string())
                            .size(tessera::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    );
                },
            )
        },
        || create_spacer(24)(),
        // surface components
        || surface_showcase(),
        || create_spacer(24)(),
        // text editor components
        {
            let state_clone = state.clone();
            move || text_editor_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // Interactive components
        {
            let state_clone = state.clone();
            move || interactive_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // Performance monitoring
        {
            let state_clone = state.clone();
            move || performance_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // Animation components (Place at the bottom to avoid jumping)
        {
            let state_clone = state.clone();
            move || animation_showcase(state_clone.clone())
        },
        || create_spacer(24)()
    )
}
