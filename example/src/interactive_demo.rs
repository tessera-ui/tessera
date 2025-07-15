use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, Px};
use tessera_ui_basic_components::{
    button::{ButtonArgsBuilder, button},
    checkbox::{CheckboxArgsBuilder, checkbox},
    column::ColumnArgsBuilder,
    column_ui,
    row::RowArgsBuilder,
    row_ui,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_ui_macros::tessera;

use crate::{app_state::AppState, material_colors::md_colors, misc::create_spacer};

/// Demo component showcasing interactive surfaces and buttons
#[tessera]
pub fn interactive_demo(app_state: Arc<AppState>) {
    column_ui!(
        ColumnArgsBuilder::default().build().unwrap(),
        // Title
        || {
            text(
                TextArgsBuilder::default()
                    .text("Interactive Components Demo".to_string())
                    .size(tessera_ui::Dp(24.0))
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
                    .size(tessera_ui::Dp(18.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        },
        // Primary button with hover effect
        {
            let app_state = app_state.clone();
            move || {
                let state = app_state.ripple_states.primary.clone();
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::PRIMARY) // Material Design primary color
                        .hover_color(Some(Color::new(0.3, 0.6, 0.9, 1.0))) // Lighter blue on hover
                        .shape(Shape::RoundedRectangle {
                            corner_radius: 25.0,
                        })
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
            let app_state = app_state.clone();
            move || {
                let state = app_state.ripple_states.success.clone();
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::TERTIARY) // Material Design tertiary color
                        .hover_color(Some(Color::new(0.2, 0.8, 0.4, 1.0))) // Lighter green on hover
                        .shape(Shape::RoundedRectangle {
                            corner_radius: 25.0,
                        })
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
            let app_state = app_state.clone();
            move || {
                let state = app_state.ripple_states.danger.clone();
                button(
                    ButtonArgsBuilder::default()
                        .color(md_colors::ERROR) // Material Design error color
                        .hover_color(Some(Color::new(0.9, 0.3, 0.3, 1.0))) // Lighter red on hover
                        .shape(Shape::RoundedRectangle {
                            corner_radius: 25.0,
                        })
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
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
        || (create_spacer(16))(),
        // Checkbox section
        {
            let app_state = app_state.clone();
            move || {
                let checked = *app_state.checkbox_state.checked.read();
                let on_toggle = {
                    let checked_arc = app_state.checkbox_state.checked.clone();
                    Arc::new(move |new_checked| {
                        *checked_arc.write() = new_checked;
                    })
                };

                row_ui!(
                    RowArgsBuilder::default()
                        .cross_axis_alignment(
                            tessera_ui_basic_components::alignment::CrossAxisAlignment::Center
                        )
                        .build()
                        .unwrap(),
                    move || checkbox(
                        CheckboxArgsBuilder::default()
                            .checked(checked)
                            .on_toggle(on_toggle)
                            .build()
                            .unwrap()
                    ),
                    || create_spacer(8)(),
                    move || {
                        let label = if checked {
                            "Checkbox is ON"
                        } else {
                            "Checkbox is OFF"
                        };
                        text(label)
                    }
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
                    .size(tessera_ui::Dp(18.0))
                    .color(md_colors::ON_SURFACE_VARIANT)
                    .build()
                    .unwrap(),
            )
        },
        // Custom interactive surface with hover effect
        {
            let app_state = app_state.clone();
            move || {
                let state = app_state.ripple_states.custom.clone();
                surface(
                    SurfaceArgsBuilder::default()
                        .color(md_colors::SECONDARY) // Material Design secondary color
                        .hover_color(Some(Color::new(0.6, 0.7, 0.9, 1.0))) // Lighter color on hover
                        .ripple_color(md_colors::RIPPLE) // Material Design ripple
                        .shape(Shape::RoundedRectangle {
                            corner_radius: 25.0,
                        })
                        .padding(Dp(16.0))
                        .width(DimensionValue::Fixed(Px(250)))
                        .height(DimensionValue::Fixed(Px(80)))
                        .border_width(2.0)
                        .border_color(Some(Color::new(1.0, 1.0, 1.0, 0.8))) // White border
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
                    .shape(Shape::RoundedRectangle {
                        corner_radius: 25.0,
                    })
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
                            .build()
                            .unwrap(),
                    )
                },
            )
        }
    )
}
