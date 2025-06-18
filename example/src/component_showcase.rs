use std::sync::Arc;
use tessera::{DimensionValue, Dp};
use tessera_basic_components::{
    column::{AsColumnItem, ColumnArgsBuilder, column},
    row::{AsRowItem, RowArgsBuilder, row},
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

/// Surface examples showcase
#[tessera]
fn surface_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
            .corner_radius(10.0)
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
            column(
                ColumnArgsBuilder::default().build().unwrap(),
                [
                    // Title inside the card
                    (|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Surface Components".to_string())
                                .size(tessera::Dp(24.0))
                                .line_height(tessera::Dp(32.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    })
                    .into_column_item(),
                    // Spacer
                    (|| (create_spacer(12))()).into_column_item(),
                    // Content
                    (|| {
                        row(
                            RowArgsBuilder::default().build().unwrap(),
                            [
                                (|| outlined_surface_example()).into_row_item(),
                                (|| (create_spacer(20))()).into_row_item(),
                                (|| transparent_surface_example()).into_row_item(),
                            ],
                        )
                    })
                    .into_column_item(),
                ],
            )
        },
    )
}

/// Text editor showcase
#[tessera]
fn text_editor_showcase(state: Arc<AppState>) {
    let editor_state_clone = state.text_editors_state.editor_state.clone();
    let editor_state_2_clone = state.text_editors_state.editor_state_2.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER)
            .corner_radius(10.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive container
        move || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                [
                    // Title inside the card
                    (|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Text Editor Components".to_string())
                                .size(tessera::Dp(24.0))
                                .line_height(tessera::Dp(32.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    })
                    .into_column_item(),
                    // Spacer
                    (|| (create_spacer(12))()).into_column_item(),
                    // Content
                    (move || text_editor_1(editor_state_clone.clone())).into_column_item(),
                    (|| (create_spacer(16))()).into_column_item(),
                    (move || text_editor_2(editor_state_2_clone.clone())).into_column_item(),
                ],
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
            .corner_radius(10.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive
        move || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                [
                    // Title inside the card
                    (|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Animation Components".to_string())
                                .size(tessera::Dp(24.0))
                                .line_height(tessera::Dp(32.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    })
                    .into_column_item(),
                    (|| (create_spacer(12))()).into_column_item(),
                    // Content
                    (|| text("Animated Spacer:")).into_column_item(),
                    (|| (create_spacer(8))()).into_column_item(),
                    (move || anim_spacer(anim_state_clone.clone())).into_column_item(),
                    (|| (create_spacer(8))()).into_column_item(),
                    (|| text("↑ Height animation effect")).into_column_item(),
                ],
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
            .corner_radius(10.0)
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
            .corner_radius(10.0)
            .padding(Dp(24.0))
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        None, // Non-interactive
        move || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::Fill {
                        min: None,
                        max: None,
                    })
                    .build()
                    .unwrap(),
                [
                    // Title inside the card
                    (|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Performance Monitoring".to_string())
                                .size(tessera::Dp(24.0))
                                .line_height(tessera::Dp(32.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    })
                    .into_column_item(),
                    (|| (create_spacer(12))()).into_column_item(),
                    // Content
                    (move || perf_display(metrics_clone.clone())).into_column_item(),
                ],
            )
        },
    )
}

/// Main component showcase that organizes all components
#[tessera]
pub fn component_showcase(state: Arc<AppState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .build()
            .unwrap(),
        [
            // Welcome section
            (|| {
                surface(
                    SurfaceArgsBuilder::default()
                        .color(md_colors::PRIMARY_CONTAINER)
                        .corner_radius(8.0)
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
                                .line_height(tessera::Dp(32.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        );
                    },
                )
            })
            .into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
            // Surface components
            (|| surface_showcase()).into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
            // Text editor components
            ({
                let state_clone = state.clone();
                move || text_editor_showcase(state_clone.clone())
            })
            .into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
            // Interactive components
            ({
                let state_clone = state.clone();
                move || interactive_showcase(state_clone.clone())
            })
            .into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
            // Performance monitoring
            ({
                let state_clone = state.clone();
                move || performance_showcase(state_clone.clone())
            })
            .into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
            // Animation components (Place at the bottom to avoid jumping)
            ({
                let state_clone = state.clone();
                move || animation_showcase(state_clone.clone())
            })
            .into_column_item(),
            (|| create_spacer(24)()).into_column_item(),
        ],
    )
}
