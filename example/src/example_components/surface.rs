use std::{fmt::Display, sync::Arc};

use tessera_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    shape_def::{RoundedCorner, Shape},
    slider::{SliderArgs, slider},
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, State, remember, shard, tessera, use_context};

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
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0))),
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
        ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Center),
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
                    RowArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center),
                    move |scope| {
                        scope.child(move || {
                            let style = if border_width.to_pixels_f32() > 0.1 {
                                SurfaceStyle::FilledOutlined {
                                    fill_color: use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .primary_container,
                                    border_color: use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .outline,
                                    border_width,
                                }
                            } else {
                                SurfaceStyle::Filled {
                                    color: use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .primary_container,
                                }
                            };
                            surface(
                                SurfaceArgs::default()
                                    .modifier(Modifier::new().size(width, height))
                                    .shape(Shape::RoundedRectangle {
                                        top_left: RoundedCorner::manual(corner_radius, 3.0),
                                        top_right: RoundedCorner::manual(corner_radius, 3.0),
                                        bottom_left: RoundedCorner::manual(corner_radius, 3.0),
                                        bottom_right: RoundedCorner::manual(corner_radius, 3.0),
                                    })
                                    .style(style)
                                    .on_click(|| {
                                        println!("Surface clicked");
                                    }),
                                || {},
                            );
                        });

                        scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

                        scope.child(move || {
                            text(TextArgs::default().text(state_string).size(Dp(16.0)));
                        });
                    },
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

            scope.child(move || {
                surface_config_slider(
                    "Width",
                    state.with(|s| s.width.value.0 as f32 / 500.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.width.value = Dp(f64::from(value) * 500.0));
                    }),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

            scope.child(move || {
                surface_config_slider(
                    "Height",
                    state.with(|s| s.height.value.0 as f32 / 500.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.height.value = Dp(f64::from(value) * 500.0));
                    }),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

            scope.child(move || {
                surface_config_slider(
                    "Corner Radius",
                    state.with(|s| s.corner_radius.value.0 / 100.0),
                    Arc::new(move |value| {
                        state.with_mut(|s| s.corner_radius.value = CornerRadius(value * 100.0));
                    }),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

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
        ColumnArgs::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(move || {
                column(ColumnArgs::default(), |scope| {
                    scope.child(move || {
                        text(TextArgs::default().text(label).size(Dp(16.0)));
                    });

                    scope.child(|| spacer(Modifier::new().height(Dp(16.0))));

                    scope.child(move || {
                        slider(
                            SliderArgs::default()
                                .value(value)
                                .on_change_shared(on_change)
                                .modifier(Modifier::new().width(Dp(300.0))),
                        );
                    });
                });
            });
        },
    );
}
