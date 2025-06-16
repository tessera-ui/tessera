use std::sync::Arc;
use tessera::{Px, DimensionValue};
use tessera_basic_components::{
    column::{ColumnItem, column},
    ripple_rect::{
        ripple_rect, RippleState,
        RippleRectArgs, RippleRectArgsBuilder
    },
    spacer::{SpacerArgsBuilder, spacer},
    text::{text, TextArgsBuilder},
};
use tessera_macros::tessera;

use crate::app_state::AppState;

/// Demo component showcasing ripple effect rectangles
#[tessera]
pub fn ripple_demo(app_state: Arc<AppState>) {
    
    column([
        // Title
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Ripple Effect Demo".to_string())
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
        
        // Filled ripple rectangles section
        ColumnItem::wrap(Box::new(|| {
            text(
                TextArgsBuilder::default()
                    .text("Filled Ripple Rectangles:".to_string())
                    .size(tessera::Dp(18.0))
                    .line_height(tessera::Dp(24.0))
                    .color([200, 200, 200])
                    .build()
                    .unwrap(),
            )
        })),

        // Primary blue ripple rect
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.primary.clone();
            move || {
                ripple_rect(
                    RippleRectArgs::primary(Arc::new(|| {
                        println!("Primary ripple rect clicked!");
                    })),
                    state,
                )
            }
        })),

        // Success green ripple rect  
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.success.clone();
            move || {
                ripple_rect(
                    RippleRectArgs::success(Arc::new(|| {
                        println!("Success ripple rect clicked!");
                    })),
                    state,
                )
            }
        })),

        // Danger red ripple rect
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.danger.clone();
            move || {
                ripple_rect(
                    RippleRectArgs::danger(Arc::new(|| {
                        println!("Danger ripple rect clicked!");
                    })),
                    state,
                )
            }
        })),

        // Custom ripple rect
        ColumnItem::wrap(Box::new({
            let state = app_state.ripple_states.custom.clone();
            move || {
                ripple_rect(
                    RippleRectArgsBuilder::default()
                        .color([0.8, 0.3, 0.8, 1.0]) // Purple
                        .ripple_color([1.0, 1.0, 0.0]) // Yellow ripple
                        .width(DimensionValue::Fixed(Px(250)))
                        .height(DimensionValue::Fixed(Px(80)))
                        .corner_radius(20.0)
                        .on_click(Arc::new(|| {
                            println!("Custom purple ripple rect with yellow ripple clicked!");
                        }))
                        .build()
                        .unwrap(),
                    state,
                )
            }
        })),
    ]);
}