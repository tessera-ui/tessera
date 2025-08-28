//! An example showcasing the `ScrollBarBehavior` functionality.
//!
//! This example demonstrates:
//! 1. AlwaysVisible: Scrollbar is always shown
//! 2. AutoHide: Scrollbar appears when scrolling and hides after inactivity
//! 3. Hidden: No scrollbar is shown at all
//!
//! The example creates three scrollable areas with different behaviors
//! and plenty of content to demonstrate scrolling.

use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Renderer};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column_ui},
    row::{RowArgs, row_ui},
    scrollable::{ScrollBarBehavior, ScrollableArgs, ScrollableState, scrollable},
    surface::{SurfaceArgs, surface},
    text::text,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    // Create states for each scrollable area
    let state_always_visible = Arc::new(ScrollableState::new());
    let state_auto_hide = Arc::new(ScrollableState::new());
    let state_hidden = Arc::new(ScrollableState::new());

    Renderer::run(
        || {
            let state_always_visible = state_always_visible.clone();
            let state_auto_hide = state_auto_hide.clone();
            let state_hidden = state_hidden.clone();
            row_ui!(
                RowArgs::default(),
                // AlwaysVisible scrollbar
                move || {
                    let state = state_always_visible.clone();
                    scrollable(
                        ScrollableArgs {
                            width: DimensionValue::Fixed(Dp(300.0).into()),
                            height: DimensionValue::Fixed(Dp(400.0).into()),
                            scrollbar_behavior: ScrollBarBehavior::AlwaysVisible,
                            scrollbar_track_color: Color::new(0.2, 0.2, 0.2, 0.3),
                            scrollbar_thumb_color: Color::new(0.4, 0.4, 0.4, 0.7),
                            scrollbar_thumb_hover_color: Color::new(0.6, 0.6, 0.6, 0.9),
                            ..Default::default()
                        },
                        state,
                        || {
                            surface(
                                SurfaceArgs {
                                    color: Color::new(0.9, 0.9, 0.9, 1.0),
                                    ..Default::default()
                                },
                                None,
                                || {
                                    column_ui!(
                                        ColumnArgs::default(),
                                        || text("AlwaysVisible Scrollbar".to_string()),
                                        || text("This scrollbar is always visible.".to_string()),
                                        || text("Scroll to see the behavior.".to_string()),
                                        || text("Item 1".to_string()),
                                        || text("Item 2".to_string()),
                                        || text("Item 3".to_string()),
                                        || text("Item 4".to_string()),
                                        || text("Item 5".to_string()),
                                        || text("Item 6".to_string()),
                                        || text("Item 7".to_string()),
                                        || text("Item 8".to_string()),
                                        || text("Item 9".to_string()),
                                        || text("Item 10".to_string()),
                                        || text("Item 11".to_string()),
                                        || text("Item 12".to_string()),
                                        || text("Item 13".to_string()),
                                        || text("Item 14".to_string()),
                                        || text("Item 15".to_string()),
                                        || text("Item 16".to_string()),
                                        || text("Item 17".to_string()),
                                        || text("Item 18".to_string()),
                                        || text("Item 19".to_string()),
                                        || text("Item 20".to_string()),
                                    );
                                },
                            );
                        },
                    );
                },
                // AutoHide scrollbar
                move || {
                    let state = state_auto_hide.clone();
                    scrollable(
                        ScrollableArgs {
                            width: DimensionValue::Fixed(Dp(300.0).into()),
                            height: DimensionValue::Fixed(Dp(400.0).into()),
                            scrollbar_behavior: ScrollBarBehavior::AutoHide,
                            scrollbar_track_color: Color::new(0.2, 0.6, 0.2, 0.3),
                            scrollbar_thumb_color: Color::new(0.4, 0.8, 0.4, 0.7),
                            scrollbar_thumb_hover_color: Color::new(0.6, 1.0, 0.6, 0.9),
                            ..Default::default()
                        },
                        state,
                        || {
                            surface(
                                SurfaceArgs {
                                    color: Color::new(0.9, 1.0, 0.9, 1.0),
                                    ..Default::default()
                                },
                                None,
                                || {
                                    column_ui!(
                                        ColumnArgs::default(),
                                        || text("AutoHide Scrollbar".to_string()),
                                        || text(
                                            "This scrollbar hides after 2 seconds of inactivity."
                                                .to_string()
                                        ),
                                        || text("Scroll or hover to see it appear.".to_string()),
                                        || text("Item 1".to_string()),
                                        || text("Item 2".to_string()),
                                        || text("Item 3".to_string()),
                                        || text("Item 4".to_string()),
                                        || text("Item 5".to_string()),
                                        || text("Item 6".to_string()),
                                        || text("Item 7".to_string()),
                                        || text("Item 8".to_string()),
                                        || text("Item 9".to_string()),
                                        || text("Item 10".to_string()),
                                        || text("Item 11".to_string()),
                                        || text("Item 12".to_string()),
                                        || text("Item 13".to_string()),
                                        || text("Item 14".to_string()),
                                        || text("Item 15".to_string()),
                                        || text("Item 16".to_string()),
                                        || text("Item 17".to_string()),
                                        || text("Item 18".to_string()),
                                        || text("Item 19".to_string()),
                                        || text("Item 20".to_string()),
                                    );
                                },
                            );
                        },
                    );
                },
                // Hidden scrollbar
                move || {
                    let state = state_hidden.clone();
                    scrollable(
                        ScrollableArgs {
                            width: DimensionValue::Fixed(Dp(300.0).into()),
                            height: DimensionValue::Fixed(Dp(400.0).into()),
                            scrollbar_behavior: ScrollBarBehavior::Hidden,
                            ..Default::default()
                        },
                        state,
                        || {
                            surface(
                                SurfaceArgs {
                                    color: Color::new(1.0, 0.9, 0.9, 1.0),
                                    ..Default::default()
                                },
                                None,
                                || {
                                    column_ui!(
                                        ColumnArgs::default(),
                                        || text("Hidden Scrollbar".to_string()),
                                        || text(
                                            "No scrollbar is shown, but scrolling still works."
                                                .to_string()
                                        ),
                                        || text("Use mouse wheel or touch gestures.".to_string()),
                                        || text("Item 1".to_string()),
                                        || text("Item 2".to_string()),
                                        || text("Item 3".to_string()),
                                        || text("Item 4".to_string()),
                                        || text("Item 5".to_string()),
                                        || text("Item 6".to_string()),
                                        || text("Item 7".to_string()),
                                        || text("Item 8".to_string()),
                                        || text("Item 9".to_string()),
                                        || text("Item 10".to_string()),
                                        || text("Item 11".to_string()),
                                        || text("Item 12".to_string()),
                                        || text("Item 13".to_string()),
                                        || text("Item 14".to_string()),
                                        || text("Item 15".to_string()),
                                        || text("Item 16".to_string()),
                                        || text("Item 17".to_string()),
                                        || text("Item 18".to_string()),
                                        || text("Item 19".to_string()),
                                        || text("Item 20".to_string()),
                                    );
                                },
                            );
                        },
                    );
                },
            );
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;
    Ok(())
}
