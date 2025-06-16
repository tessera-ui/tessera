use std::sync::Arc;
use tessera::{DimensionValue, Dp};
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::{
    animated_spacer::anim_spacer,
    app_state::AppState,
    interactive_demo::interactive_demo,
    layout_examples::{outlined_surface_example, transparent_surface_example},
    misc::create_spacer,
    performance_display::perf_display,
    text_editors::{text_editor_1, text_editor_2},
};

/// Surface examples showcase
#[tessera]
fn surface_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .color([0.25, 0.25, 0.3, 1.0])
            .corner_radius(10.0)
            .padding(Dp(20.0))
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
            column([
                // Title inside the card
                ColumnItem::wrap(Box::new(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Surface Components".to_string())
                            .size(tessera::Dp(24.0))
                            .line_height(tessera::Dp(32.0))
                            .color([255, 255, 255])
                            .build()
                            .unwrap(),
                    )
                })),
                // Spacer
                ColumnItem::wrap(Box::new(create_spacer(15))),
                // Content
                ColumnItem::wrap(Box::new(|| {
                    row([
                        RowItem::wrap(Box::new(outlined_surface_example)),
                        RowItem::wrap(Box::new(create_spacer(20))),
                        RowItem::wrap(Box::new(transparent_surface_example)),
                    ])
                })),
            ])
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
            .color([0.25, 0.25, 0.3, 1.0])
            .corner_radius(10.0)
            .padding(Dp(20.0))
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
        move || {
            column([
                // Title inside the card
                ColumnItem::wrap(Box::new(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Text Editor Components".to_string())
                            .size(tessera::Dp(24.0))
                            .line_height(tessera::Dp(32.0))
                            .color([255, 255, 255])
                            .build()
                            .unwrap(),
                    )
                })),
                // Spacer
                ColumnItem::wrap(Box::new(create_spacer(15))),
                // Content
                ColumnItem::wrap(Box::new(move || text_editor_1(editor_state_clone.clone()))),
                ColumnItem::wrap(Box::new(create_spacer(15))),
                ColumnItem::wrap(Box::new(move || {
                    text_editor_2(editor_state_2_clone.clone())
                })),
            ])
        },
    )
}

/// Animation showcase
#[tessera]
fn animation_showcase(state: Arc<AppState>) {
    let anim_state_clone = state.anim_spacer_state.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color([0.25, 0.25, 0.3, 1.0])
            .corner_radius(10.0)
            .padding(Dp(20.0))
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
        None, // Non-interactive
        move || {
            column([
                // Title inside the card
                ColumnItem::wrap(Box::new(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Animation Components".to_string())
                            .size(tessera::Dp(24.0))
                            .line_height(tessera::Dp(32.0))
                            .color([255, 255, 255])
                            .build()
                            .unwrap(),
                    )
                })),
                ColumnItem::wrap(Box::new(create_spacer(15))),
                // Content
                ColumnItem::wrap(Box::new(|| text("Animated Spacer:"))),
                ColumnItem::wrap(Box::new(move || anim_spacer(anim_state_clone.clone()))),
                ColumnItem::wrap(Box::new(|| text("↑ Height animation effect"))),
            ])
        },
    )
}

/// Interactive components showcase
#[tessera]
fn interactive_showcase(state: Arc<AppState>) {
    let state_clone = state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .color([0.25, 0.25, 0.3, 1.0])
            .corner_radius(10.0)
            .padding(Dp(20.0))
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
        move || interactive_demo(state_clone.clone()),
    )
}

/// Performance showcase
#[tessera]
fn performance_showcase(state: Arc<AppState>) {
    let metrics_clone = state.metrics.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color([0.25, 0.25, 0.3, 1.0])
            .corner_radius(10.0)
            .padding(Dp(20.0))
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
        None, // Non-interactive
        move || {
            column([
                // Title inside the card
                ColumnItem::wrap(Box::new(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Performance Monitoring".to_string())
                            .size(tessera::Dp(24.0))
                            .line_height(tessera::Dp(32.0))
                            .color([255, 255, 255])
                            .build()
                            .unwrap(),
                    )
                })),
                ColumnItem::wrap(Box::new(create_spacer(15))),
                // Content
                ColumnItem::wrap(Box::new(move || perf_display(metrics_clone.clone()))),
            ])
        },
    )
}

/// Main component showcase that organizes all components
#[tessera]
pub fn component_showcase(state: Arc<AppState>) {
    column([
        // Welcome section
        ColumnItem::wrap(Box::new(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .color([0.3, 0.4, 0.5, 1.0])
                    .corner_radius(8.0)
                    .padding(Dp(12.0))
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
                            .color([255, 255, 255])
                            .build()
                            .unwrap(),
                    );
                },
            )
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Surface components
        ColumnItem::wrap(Box::new(surface_showcase)),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Text editor components
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || text_editor_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Interactive components
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || interactive_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Performance monitoring
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || performance_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Animation components (放在最下面避免跳动)
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || animation_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
    ])
}
