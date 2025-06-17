use std::sync::Arc;
use tessera::{DimensionValue, Dp, Px};
use tessera_basic_components::{
    button::{ButtonArgsBuilder, button},
    column::{ColumnItem, column},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::{app_state::AppState, material_colors::md_colors, misc::create_spacer};

/// Demo component showcasing interactive surfaces and buttons
#[tessera]
pub fn interactive_demo(app_state: Arc<AppState>) {
    column([
        // Title
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Components Demo".to_string())
                    .size(tessera::Dp(24.0))
                    .line_height(tessera::Dp(32.0))
                    .color(md_colors::ON_SURFACE)
                    .build()
                    .unwrap(),
            )
        })),
        // Spacer
        ColumnItem::wrap(Box::new(create_spacer(16))),
        // Buttons section
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Buttons:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        })),
        // Primary button
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.primary.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::PRIMARY) // Material Design primary color
                        .corner_radius(8.0)
                        .padding(Dp(12.0))
                        .on_click(Arc::new(|| {
                            println!("Primary button clicked!");
                        }))
                        .build()
                        .unwrap(),
                    state,
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Primary Button".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Small spacer between buttons
        ColumnItem::wrap(Box::new(create_spacer(8))),
        // Success button
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.success.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::TERTIARY) // Material Design tertiary color
                        .corner_radius(8.0)
                        .padding(Dp(12.0))
                        .on_click(Arc::new(|| {
                            println!("Success button clicked!");
                        }))
                        .build()
                        .unwrap(),
                    state,
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Success Button".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Small spacer between buttons
        ColumnItem::wrap(Box::new(create_spacer(8))),
        // Danger button
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.danger.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::ERROR) // Material Design error color
                        .corner_radius(8.0)
                        .padding(Dp(12.0))
                        .on_click(Arc::new(|| {
                            println!("Danger button clicked!");
                        }))
                        .build()
                        .unwrap(),
                    state,
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Danger Button".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Spacer
        ColumnItem::wrap(Box::new(create_spacer(16))),
        // Interactive surfaces section
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Surfaces:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        })),
        // Custom interactive surface with border
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.custom.clone();
            move || {
                surface(
                    SurfaceArgsBuilder::default()
                        .color(md_colors::SECONDARY) // Material Design secondary color
                        .ripple_color(md_colors::RIPPLE) // Material Design ripple
                        .corner_radius(12.0)
                        .padding(Dp(16.0))
                        .width(DimensionValue::Fixed(Px(250)))
                        .height(DimensionValue::Fixed(Px(80)))
                        .border_width(2.0)
                        .border_color(Some([1.0, 1.0, 1.0, 0.8])) // White border
                        .on_click(Some(Arc::new(|| {
                            println!("Custom interactive surface clicked!");
                        })))
                        .build()
                        .unwrap(),
                    Some(state),
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Interactive Surface\nwith custom styling".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(14.0))
                                .line_height(Dp(18.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Small spacer between surfaces
        ColumnItem::wrap(Box::new(create_spacer(12))),
        // Non-interactive surface for comparison
        ColumnItem::wrap(Box::new(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .color(md_colors::SURFACE_VARIANT) // Material Design surface-variant
                    .corner_radius(8.0)
                    .padding(Dp(12.0))
                    .width(DimensionValue::Fixed(Px(200)))
                    .height(DimensionValue::Fixed(Px(60)))
                    .build()
                    .unwrap(),
                None, // No ripple state - non-interactive
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Non-interactive Surface".to_string())
                            .color(md_colors::ON_SURFACE_VARIANT)
                            .size(Dp(14.0))
                            .line_height(Dp(18.0))
                            .build()
                            .unwrap(),
                    )
                },
            )
        })),
    ]);
}
