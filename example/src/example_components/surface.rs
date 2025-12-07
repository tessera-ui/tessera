use std::{fmt::Display, sync::Arc};

use closure::closure;
use parking_lot::RwLock;
use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgsBuilder, column},
    material_color::global_material_scheme,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::{RoundedCorner, Shape},
    slider::{SliderArgsBuilder, slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default)]
struct SurfaceShowcaseState {
    scrollable_state: ScrollableState,
    example_surface_state: Arc<RwLock<ExampleSurfaceState>>,
}

struct CornerRadius(f32);

impl Display for CornerRadius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

struct ExampleSurfaceState {
    width: ConfigSliderState<Dp>,
    height: ConfigSliderState<Dp>,
    border_width: ConfigSliderState<Dp>,
    corner_radius: ConfigSliderState<CornerRadius>,
}

struct ConfigSliderState<T: Display> {
    value: T,
}

impl<T: Display> ConfigSliderState<T> {
    fn new(initial_value: T) -> Self {
        Self {
            value: initial_value,
        }
    }
}

impl Display for ExampleSurfaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Width: {}", self.width.value)?;
        writeln!(f, "Height: {}", self.height.value)?;
        writeln!(f, "Border Width: {}", self.border_width.value)?;
        writeln!(f, "Corner Radius: {}", self.corner_radius.value)?;
        Ok(())
    }
}

impl Default for ExampleSurfaceState {
    fn default() -> Self {
        Self {
            width: ConfigSliderState::new(Dp(100.0)),
            height: ConfigSliderState::new(Dp(100.0)),
            border_width: ConfigSliderState::new(Dp(0.0)),
            corner_radius: ConfigSliderState::new(CornerRadius(25.0)),
        }
    }
}

#[tessera]
#[shard]
pub fn surface_showcase(#[state] state: SurfaceShowcaseState) {
    let example_surface_state = state.example_surface_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                state.scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(16.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        move || {
                            test_content(example_surface_state);
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<RwLock<ExampleSurfaceState>>) {
    let state_for_surface = state.clone();
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            let state = state_for_surface.clone();
            scope.child(move || {
                let state = state.read();
                let corner_radius = Dp(state.corner_radius.value.0 as f64);
                let width = state.width.value;
                let height = state.height.value;
                let border_width = state.border_width.value;
                let state_string = (*state).to_string();

                row(
                    RowArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .build()
                        .unwrap(),
                    move |scope| {
                        scope.child(move || {
                            let style = if border_width.to_pixels_f32() > 0.1 {
                                SurfaceStyle::FilledOutlined {
                                    fill_color: global_material_scheme().primary_container,
                                    border_color: global_material_scheme().outline,
                                    border_width,
                                }
                            } else {
                                SurfaceStyle::Filled {
                                    color: global_material_scheme().primary_container,
                                }
                            };
                            surface(
                                SurfaceArgsBuilder::default()
                                    .width(DimensionValue::from(width))
                                    .height(DimensionValue::from(height))
                                    .shape(Shape::RoundedRectangle {
                                        top_left: RoundedCorner::manual(corner_radius, 3.0),
                                        top_right: RoundedCorner::manual(corner_radius, 3.0),
                                        bottom_left: RoundedCorner::manual(corner_radius, 3.0),
                                        bottom_right: RoundedCorner::manual(corner_radius, 3.0),
                                    })
                                    .style(style)
                                    .on_click(Arc::new(|| {
                                        println!("Surface clicked");
                                    }))
                                    .build()
                                    .unwrap(),
                                || {},
                            );
                        });

                        scope.child(|| {
                            spacer(
                                SpacerArgsBuilder::default()
                                    .width(DimensionValue::from(Dp(16.0)))
                                    .build()
                                    .unwrap(),
                            )
                        });

                        scope.child(move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(state_string)
                                    .size(Dp(16.0))
                                    .build()
                                    .unwrap(),
                            );
                        });
                    },
                );
            });

            scope.child(|| {
                spacer(
                    SpacerArgsBuilder::default()
                        .height(DimensionValue::from(Dp(16.0)))
                        .build()
                        .unwrap(),
                )
            });

            let state = state_for_surface.clone();
            scope.child(move || {
                surface_config_slider(
                    "Width",
                    state.read().width.value.0 as f32 / 500.0,
                    Arc::new(closure!(clone state, |value| {
                        state.write().width.value = Dp(f64::from(value) * 500.0);
                    })),
                );
            });

            scope.child(|| {
                spacer(
                    SpacerArgsBuilder::default()
                        .height(DimensionValue::from(Dp(16.0)))
                        .build()
                        .unwrap(),
                )
            });

            let state = state_for_surface.clone();

            scope.child(move || {
                surface_config_slider(
                    "Height",
                    state.read().height.value.0 as f32 / 500.0,
                    Arc::new(closure!(clone state, |value| {
                        state.write().height.value = Dp(f64::from(value) * 500.0);
                    })),
                );
            });

            scope.child(|| {
                spacer(
                    SpacerArgsBuilder::default()
                        .height(DimensionValue::from(Dp(16.0)))
                        .build()
                        .unwrap(),
                )
            });

            let state = state_for_surface.clone();
            scope.child(move || {
                surface_config_slider(
                    "Corner Radius",
                    state.read().corner_radius.value.0 / 100.0,
                    Arc::new(closure!(clone state, |value| {
                        state.write().corner_radius.value = CornerRadius(value * 100.0);
                    })),
                );
            });

            scope.child(|| {
                spacer(
                    SpacerArgsBuilder::default()
                        .height(DimensionValue::from(Dp(16.0)))
                        .build()
                        .unwrap(),
                )
            });

            let state = state_for_surface.clone();
            scope.child(move || {
                surface_config_slider(
                    "Border Width",
                    state.read().border_width.value.0 as f32 / 20.0,
                    Arc::new(closure!(clone state, |value| {
                        state.write().border_width.value = Dp(f64::from(value) * 20.0);
                    })),
                );
            });
        },
    );
}

#[tessera]
fn surface_config_slider(label: &str, value: f32, on_change: Arc<dyn Fn(f32) + Send + Sync>) {
    let label = label.to_string();
    column(
        ColumnArgsBuilder::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(move || {
                column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                    scope.child(move || {
                        text(
                            TextArgsBuilder::default()
                                .text(label)
                                .size(Dp(16.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(DimensionValue::from(Dp(16.0)))
                                .build()
                                .unwrap(),
                        )
                    });

                    scope.child(move || {
                        slider(
                            SliderArgsBuilder::default()
                                .value(value)
                                .on_change(on_change)
                                .width(DimensionValue::Fixed(Dp(300.0).to_px()))
                                .build()
                                .unwrap(),
                        );
                    });
                });
            });
        },
    );
}
