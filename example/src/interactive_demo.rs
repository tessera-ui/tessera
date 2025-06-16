use std::sync::Arc;
use tessera::{DimensionValue, Dp, Px};
use tessera_basic_components::{
    button::{ButtonArgsBuilder, button},
    column::{ColumnItem, column},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

use crate::app_state::AppState;

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
                    .color([255, 255, 255])
                    .build()
                    .unwrap(),
            )
        })),
        // Spacer
        ColumnItem::wrap(Box::new(|| {
            spacer(
                SpacerArgsBuilder::default()
                    .height(DimensionValue::Fixed(Px(20)))
                    .build()
                    .unwrap(),
            )
        })),
        // Buttons section
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Buttons:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color([200, 200, 200])
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
                        .color([0.2, 0.5, 0.8, 1.0]) // Blue
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
                                .color([255, 255, 255])
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Success button
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.success.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color([0.1, 0.7, 0.3, 1.0]) // Green
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
                                .color([255, 255, 255])
                                .size(Dp(16.0))
                                .line_height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Danger button
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.danger.clone();
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .color([0.8, 0.2, 0.2, 1.0]) // Red
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
                                .color([255, 255, 255])
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
        ColumnItem::wrap(Box::new(|| {
            spacer(
                SpacerArgsBuilder::default()
                    .height(DimensionValue::Fixed(Px(20)))
                    .build()
                    .unwrap(),
            )
        })),
        // Interactive surfaces section
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Surfaces:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color([200, 200, 200])
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
                        .color([0.8, 0.3, 0.8, 1.0]) // Purple
                        .ripple_color([1.0, 1.0, 0.0]) // Yellow ripple
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
                                .color([255, 255, 255])
                                .size(Dp(14.0))
                                .line_height(Dp(18.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        })),
        // Non-interactive surface for comparison
        ColumnItem::wrap(Box::new(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .color([0.4, 0.4, 0.4, 1.0]) // Gray
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
                            .color([200, 200, 200])
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
