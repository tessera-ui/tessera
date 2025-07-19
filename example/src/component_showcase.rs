use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp};
use tessera_ui_basic_components::{
    column::ColumnArgsBuilder,
    column_ui,
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};
use tessera_ui_macros::tessera;

use crate::{
    animated_spacer::anim_spacer,
    app_state::AppState,
    interactive_demo::interactive_demo,
    material_colors::md_colors,
    misc::create_spacer,
    performance_display::perf_display,
    switch_showcase::switch_showcase,
    text_editors::{text_editor_1, text_editor_2},
};

/// surface examples showcase
#[tessera]
fn surface_showcase(state: Arc<AppState>) {
    let state = state.clone();
    {
        surface(
            SurfaceArgsBuilder::default()
                .color(md_colors::SURFACE_CONTAINER) // Material Design surface-container color
                .shape(Shape::RoundedRectangle {
                    corner_radius: 25.0,
                    g2_k_value: 3.0,
                })
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
                                .text("Surface Components".to_string())
                                .size(tessera_ui::Dp(24.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    },
                    // Spacer
                    || (create_spacer(12))(),
                    // Content
                    || text("Surface looks like an enhanced rect"),
                    || (create_spacer(8))(),
                    || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .width(DimensionValue::Fixed(Dp(100.0).into()))
                                .height(DimensionValue::Fixed(Dp(100.0).into()))
                                .color(Color::RED)
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        );
                    },
                    || (create_spacer(8))(),
                    || text("surface can be clicked"),
                    || (create_spacer(8))(),
                    || {
                        surface(
                            SurfaceArgsBuilder::default()
                                .color(Color::RED)
                                .width(DimensionValue::Fixed(Dp(100.0).into()))
                                .height(DimensionValue::Fixed(Dp(100.0).into()))
                                .on_click(Some(Arc::new(|| {
                                    println!("Surface clicked!");
                                })))
                                .build()
                                .unwrap(),
                            None,
                            || {},
                        );
                    },
                    || (create_spacer(8))(),
                    || text("surface can have ripple anim on click"),
                    || (create_spacer(8))(),
                    {
                        let state = state.clone();
                        move || {
                            surface(
                                SurfaceArgsBuilder::default()
                                    .color(Color::RED)
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .on_click(Some(Arc::new(|| {
                                        println!("Surface with ripple clicked!")
                                    })))
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    },
                    || (create_spacer(8))(),
                    || text("surface can have rounded corners"),
                    {
                        let state = state.clone();
                        move || {
                            surface(
                                SurfaceArgsBuilder::default()
                                    .color(Color::RED)
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .on_click(Some(Arc::new(|| {
                                        println!("Surface with ripple clicked!")
                                    })))
                                    .shape(Shape::RoundedRectangle {
                                        corner_radius: 25.0,
                                        g2_k_value: 3.0,
                                    })
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    },
                    || (create_spacer(8))(),
                    || text("surface can be an ellipse"),
                    {
                        let state = state.clone();
                        move || {
                            surface(
                                SurfaceArgsBuilder::default()
                                    .color(Color::RED)
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .on_click(Some(Arc::new(|| {
                                        println!("Surface with ripple clicked!")
                                    })))
                                    .shape(Shape::Ellipse)
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    },
                )
            },
        )
    }
}

/// fluid glass examples showcase
#[tessera]
fn fluid_glass_showcase(state: Arc<AppState>) {
    let state = state.clone();
    {
        surface(
            SurfaceArgsBuilder::default()
                .color(md_colors::SURFACE_CONTAINER)
                .shape(Shape::RoundedRectangle {
                    corner_radius: 25.0,
                    g2_k_value: 3.0,
                })
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
            None,
            || {
                column_ui!(
                    ColumnArgsBuilder::default().build().unwrap(),
                    // Title
                    || {
                        text(
                            TextArgsBuilder::default()
                                .text("Fluid Glass Components".to_string())
                                .size(tessera_ui::Dp(24.0))
                                .color(md_colors::ON_SURFACE)
                                .build()
                                .unwrap(),
                        )
                    },
                    || (create_spacer(12))(),
                    // Content
                    || text("fluid_glass is glass-like surface"),
                    || (create_spacer(8))(),
                    // Basic fluid glass with ripple
                    {
                        move || {
                            fluid_glass(
                                FluidGlassArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .tint_color(Color::new(0.8, 0.9, 1.0, 0.2))
                                    .build()
                                    .unwrap(),
                                None,
                                || {},
                            );
                        }
                    },
                    || (create_spacer(8))(),
                    || text("fluid_glass with border and ripple"),
                    {
                        let state = state.clone();
                        move || {
                            fluid_glass(
                                FluidGlassArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .tint_color(Color::new(0.8, 0.9, 1.0, 0.2))
                                    .border(GlassBorder::new(Dp(2.0), Color::BLUE.with_alpha(0.3)))
                                    .on_click(Arc::new(|| {
                                        println!("Fluid glass with border clicked!");
                                    }))
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    },
                    || (create_spacer(8))(),
                    || text("fluid_glass with rounded corners and ripple"),
                    {
                        let state = state.clone();
                        move || {
                            fluid_glass(
                                FluidGlassArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .shape(Shape::RoundedRectangle {
                                        corner_radius: 25.0,
                                        g2_k_value: 3.0,
                                    })
                                    .on_click(Arc::new(|| {
                                        println!("Fluid glass with rounded corners clicked!");
                                    }))
                                    .tint_color(Color::new(0.8, 0.9, 1.0, 0.2))
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    },
                    || (create_spacer(8))(),
                    || text("fluid_glass as ellipse with ripple and border"),
                    {
                        let state = state.clone();
                        move || {
                            fluid_glass(
                                FluidGlassArgsBuilder::default()
                                    .width(DimensionValue::Fixed(Dp(100.0).into()))
                                    .height(DimensionValue::Fixed(Dp(100.0).into()))
                                    .shape(Shape::Ellipse)
                                    .tint_color(Color::new(0.8, 0.9, 1.0, 0.2))
                                    .on_click(Arc::new(|| {
                                        println!("Fluid glass ellipse clicked!");
                                    }))
                                    .border(GlassBorder::new(Dp(2.0), Color::BLUE.with_alpha(0.3)))
                                    .build()
                                    .unwrap(),
                                Some(state.ripple_states.primary.clone()),
                                || {},
                            );
                        }
                    }
                )
            },
        )
    }
}

/// text editor showcase
#[tessera]
fn text_editor_showcase(state: Arc<AppState>) {
    let editor_state_clone = state.text_editors_state.editor_state.clone();
    let editor_state_2_clone = state.text_editors_state.editor_state_2.clone();

    surface(
        SurfaceArgsBuilder::default()
            .color(md_colors::SURFACE_CONTAINER)
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            })
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
                            .text("Text Editor Components".to_string())
                            .size(tessera_ui::Dp(24.0))
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
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            })
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
                            .size(tessera_ui::Dp(24.0))
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
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            })
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
            .shape(Shape::RoundedRectangle {
                corner_radius: 25.0,
                g2_k_value: 3.0,
            })
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
                            .size(tessera_ui::Dp(24.0))
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
                    .shape(Shape::RoundedRectangle {
                        corner_radius: 25.0,
                        g2_k_value: 3.0,
                    })
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
                            .size(tessera_ui::Dp(24.0))
                            .color(md_colors::ON_SURFACE)
                            .build()
                            .unwrap(),
                    );
                },
            )
        },
        || create_spacer(24)(),
        // surface components
        {
            let state_clone = state.clone();
            move || surface_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // fluid glass components
        {
            let state_clone = state.clone();
            move || fluid_glass_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // text editor components
        {
            let state_clone = state.clone();
            move || text_editor_showcase(state_clone.clone())
        },
        || create_spacer(24)(),
        // Switch component
        {
            let state_clone = state.clone();
            move || switch_showcase(state_clone.switch_state.state.clone())
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
