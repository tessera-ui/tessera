use std::sync::Arc;
use tessera::{DimensionValue, Dp, Px};
use tessera_basic_components::{
    button::{ButtonArgsBuilder, button},
    column::ColumnArgsBuilder,
    column_ui,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::{app_state::AppState, material_colors::md_colors, misc::create_spacer};

/// Demo component showcasing interactive surfaces and buttons
#[tessera]
pub fn interactive_demo(app_state: Arc<AppState>) {
    column_ui!(
        ColumnArgsBuilder::default().build().unwrap(),
        // Title
        move || {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Components Demo".to_string())
                    .size(tessera::Dp(24.0))
                    .line_height(tessera::Dp(32.0))
                    .color(md_colors::ON_SURFACE)
                    .build()
                    .unwrap(),
            )
        },
        // Spacer
        || (create_spacer(16))(),
        // Buttons section
        || {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Buttons with Hover Effects:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        },
        // Primary button with hover effect
        {
            let state = app_state.ripple_states.primary.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::PRIMARY) // Material Design primary color
                        .hover_color(Some([0.3, 0.6, 0.9, 1.0])) // Lighter blue on hover
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
                                .text("Primary Button (Hover Effect)".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
        // Small spacer between buttons
        || (create_spacer(8))(),
        // Success button with hover effect
        {
            let state = app_state.ripple_states.success.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::TERTIARY) // Material Design tertiary color
                        .hover_color(Some([0.2, 0.8, 0.4, 1.0])) // Lighter green on hover
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
                                .text("Success Button (Hover Effect)".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
        // Small spacer between buttons
        || (create_spacer(8))(),
        // Danger button with hover effect
        {
            let state = app_state.ripple_states.danger.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::ERROR) // Material Design error color
                        .hover_color(Some([0.9, 0.3, 0.3, 1.0])) // Lighter red on hover
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
                                .text("Danger Button (Hover Effect)".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
        // Spacer
        || (create_spacer(16))(),
        // Interactive surfaces section
        || {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Surfaces with Hover Effects:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        },
        // Custom interactive surface with hover effect
        {
            let state = app_state.ripple_states.custom.clone();
            move || {
                surface(
                    SurfaceArgsBuilder::default()
                        .color(md_colors::SECONDARY) // Material Design secondary color
                        .hover_color(Some([0.6, 0.7, 0.9, 1.0])) // Lighter color on hover
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
                                .text("Interactive Surface\nwith Hover Effect".to_string())
                                .color(md_colors::ON_SURFACE)
                                .size(Dp(14.0))
                                .line_height(Dp(18.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
        // Small spacer between surfaces
        || (create_spacer(12))(),
        // Non-interactive surface for comparison
        || {
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
                            .text("Non-interactive Surface\n(No Hover Effect)".to_string())
                            .color(md_colors::ON_SURFACE_VARIANT)
                            .size(Dp(14.0))
                            .line_height(Dp(18.0))
                            .build()
                            .unwrap(),
                    )
                },
            )
        }
    )
}
