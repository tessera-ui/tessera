use std::{fmt::Display, sync::Arc};

use tessera_ui::{DimensionValue, Dp, State, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    shape_def::{RoundedCorner, Shape},
    slider::{SliderArgsBuilder, slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialColorScheme,
};

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
pub fn surface_showcase() {
    let example_surface_state = remember(ExampleSurfaceState::default);
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
fn test_content(state: State<ExampleSurfaceState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(move || {
                let (corner_radius, width, height, border_width, state_string) = state.with(|s| {
                    (
                        Dp(s.corner_radius.value.0 as f64),
                        s.width.value,
                        s.height.value,
                        s.border_width.value,
                        s.to_string(),
                    )
                });

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
                                    fill_color: use_context::<MaterialColorScheme>()
                                        .get()
                                        .primary_container,
                                    border_color: use_context::<MaterialColorScheme>()
                                        .get()
                                        .outline,
                                    border_width,
                                }
                            } else {
                                SurfaceStyle::Filled {
                                    color: use_context::<MaterialColorScheme>()
                                        .get()
                                        .primary_container,
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

            scope.child(move || {
                surface_config_slider(
                    "Width",
                    state.with(|s| s.width.value.0 as f32 / 500.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.width.value = Dp(f64::from(value) * 500.0));
                    }),
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

            scope.child(move || {
                surface_config_slider(
                    "Height",
                    state.with(|s| s.height.value.0 as f32 / 500.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.height.value = Dp(f64::from(value) * 500.0));
                    }),
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

            scope.child(move || {
                surface_config_slider(
                    "Corner Radius",
                    state.with(|s| s.corner_radius.value.0 / 100.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.corner_radius.value = CornerRadius(value * 100.0));
                    }),
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

            scope.child(move || {
                surface_config_slider(
                    "Border Width",
                    state.with(|s| s.border_width.value.0 as f32 / 20.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.border_width.value = Dp(f64::from(value) * 20.0));
                    }),
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
