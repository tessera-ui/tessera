use std::sync::Arc;
use tessera::{DimensionValue, Dp};
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

use crate::{
    animated_spacer::anim_spacer,
    app_state::AppState,
    button_demo::button_demo,
    layout_examples::{outlined_surface_example, transparent_surface_example},
    misc::create_spacer,
    performance_display::perf_display,
    text_editors::{text_editor_1, text_editor_2},
};

/// Creates a section header with title
#[tessera]
fn section_header(title: &'static str) {
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
        move || {
            text(title);
        },
    )
}

/// Surface examples showcase
#[tessera]
fn surface_showcase() {
    column([
        ColumnItem::wrap(Box::new(|| section_header("Surface Components"))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(|| {
            row([
                RowItem::wrap(Box::new(outlined_surface_example)),
                RowItem::wrap(Box::new(create_spacer(20))),
                RowItem::wrap(Box::new(transparent_surface_example)),
            ])
        })),
    ])
}

/// Text editor showcase
#[tessera]
fn text_editor_showcase(state: Arc<AppState>) {
    let editor_state_clone = state.text_editors_state.editor_state.clone();
    let editor_state_2_clone = state.text_editors_state.editor_state_2.clone();

    column([
        ColumnItem::wrap(Box::new(|| section_header("Text Editor Components"))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(move || text_editor_1(editor_state_clone.clone()))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(move || {
            text_editor_2(editor_state_2_clone.clone())
        })),
    ])
}

/// Button showcase
#[tessera]
fn button_showcase(state: Arc<AppState>) {
    let button_data_clone = state.button_demo_data.clone();

    column([
        ColumnItem::wrap(Box::new(|| section_header("Button Components"))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(move || button_demo(button_data_clone.clone()))),
    ])
}

/// Animation showcase
#[tessera]
fn animation_showcase(state: Arc<AppState>) {
    let anim_state_clone = state.anim_spacer_state.clone();

    column([
        ColumnItem::wrap(Box::new(|| section_header("Animation Components"))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(|| {
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
                move || {
                    column([
                        ColumnItem::wrap(Box::new(|| text("Animated Spacer:"))),
                        ColumnItem::wrap(Box::new(move || anim_spacer(anim_state_clone.clone()))),
                        ColumnItem::wrap(Box::new(|| text("↑ Height animation effect"))),
                    ])
                },
            )
        })),
    ])
}

/// Performance showcase
#[tessera]
fn performance_showcase(state: Arc<AppState>) {
    let metrics_clone = state.metrics.clone();

    column([
        ColumnItem::wrap(Box::new(|| section_header("Performance Monitoring"))),
        ColumnItem::wrap(Box::new(create_spacer(15))),
        ColumnItem::wrap(Box::new(move || {
            surface(
                SurfaceArgsBuilder::default()
                    .color([0.2, 0.3, 0.2, 1.0])
                    .corner_radius(8.0)
                    .padding(Dp(15.0))
                    .build()
                    .unwrap(),
                move || perf_display(metrics_clone.clone()),
            )
        })),
    ])
}

/// Main component showcase that organizes all components
#[tessera]
pub fn component_showcase(state: Arc<AppState>) {
    column([
        // Welcome section
        ColumnItem::wrap(Box::new(|| {
            section_header("Tessera UI Framework Component Showcase")
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Surface components
        ColumnItem::wrap(Box::new(surface_showcase)),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Button components
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || button_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Text editor components
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || text_editor_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Animation components
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || animation_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
        // Performance monitoring
        ColumnItem::wrap(Box::new({
            let state_clone = state.clone();
            move || performance_showcase(state_clone.clone())
        })),
        ColumnItem::wrap(Box::new(create_spacer(30))),
    ])
}
