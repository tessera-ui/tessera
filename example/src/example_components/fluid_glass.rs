use std::{fmt::Display, sync::Arc};

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    fluid_glass::{FluidGlassArgsBuilder, GlassBorder, fluid_glass},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    shape_def::Shape,
    slider::{SliderArgsBuilder, SliderState, slider},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

const IMAGE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/scarlet_ut.jpg",
);

#[derive(Default)]
struct FluidGlassShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    example_glass_state: Arc<RwLock<ExampleGlassState>>,
}

struct ConerRadius(f32);

impl Display for ConerRadius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

struct ExampleGlassState {
    ripple_state: Arc<RippleState>,
    width: ConfigSliderState<Dp>,
    height: ConfigSliderState<Dp>,
    border_width: ConfigSliderState<Dp>,
    coner_radius: ConfigSliderState<ConerRadius>,
    refraction_amount: ConfigSliderState<f32>,
    refraction_height: ConfigSliderState<f32>,
    background_image_data: Arc<ImageData>,
}

struct ConfigSliderState<T: Display> {
    value: T,
    slider_state: Arc<RwLock<SliderState>>,
}

impl<T: Display> ConfigSliderState<T> {
    fn new(initial_value: T) -> Self {
        Self {
            value: initial_value,
            slider_state: Default::default(),
        }
    }
}

impl Display for ExampleGlassState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Width: {}", self.width.value)?;
        writeln!(f, "Height: {}", self.height.value)?;
        writeln!(f, "Border Width: {}", self.border_width.value)?;
        writeln!(f, "Coner Radius: {}", self.coner_radius.value)?;
        writeln!(
            f,
            "Refraction Strength: {:.2}",
            self.refraction_amount.value
        )?;
        writeln!(f, "Refraction Height: {:.2}", self.refraction_height.value)?;
        Ok(())
    }
}

impl Default for ExampleGlassState {
    fn default() -> Self {
        println!("Loading image from path: {}", IMAGE_PATH);
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Path(IMAGE_PATH.to_string()))
                .expect("Failed to load image"),
        );

        Self {
            ripple_state: Default::default(),
            width: ConfigSliderState::new(Dp(100.0)),
            height: ConfigSliderState::new(Dp(100.0)),
            border_width: ConfigSliderState::new(Dp(1.0)),
            coner_radius: ConfigSliderState::new(ConerRadius(25.0)),
            refraction_amount: ConfigSliderState::new(32.0),
            refraction_height: ConfigSliderState::new(24.0),
            background_image_data: image_data,
        }
    }
}

#[tessera]
#[shard]
pub fn fluid_glass_showcase(#[state] state: FluidGlassShowcaseState) {
    let example_surface_state = state.example_glass_state.clone();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
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
                            .style(Color::WHITE.into())
                            .padding(Dp(16.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
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
fn test_content(state: Arc<RwLock<ExampleGlassState>>) {
    let state_for_glass = state.clone();
    let image_data = state.read().background_image_data.clone();
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(move || {
                let state = state.read();
                let coner_radius = state.coner_radius.value.0;
                let width = state.width.value;
                let height = state.height.value;
                let border_width = state.border_width.value;
                let state_string = (*state).to_string();
                let ripple_state = state.ripple_state.clone();
                let refraction_amount = state.refraction_amount.value;
                let refraction_height = state.refraction_height.value;

                row(
                    RowArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .build()
                        .unwrap(),
                    move |scope| {
                        scope.child(move || {
                            boxed(
                                BoxedArgsBuilder::default()
                                    .alignment(Alignment::Center)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| {
                                        image(
                                            ImageArgsBuilder::default()
                                                .width(DimensionValue::from(Dp(250.0)))
                                                .height(DimensionValue::from(Dp(250.0)))
                                                .data(image_data)
                                                .build()
                                                .unwrap(),
                                        );
                                    });

                                    scope.child(move || {
                                        fluid_glass(
                                            FluidGlassArgsBuilder::default()
                                                .width(DimensionValue::from(width))
                                                .height(DimensionValue::from(height))
                                                .tint_color(Color::new(0.2, 0.5, 0.8, 0.1))
                                                .shape(Shape::RoundedRectangle {
                                                    top_left: coner_radius,
                                                    top_right: coner_radius,
                                                    bottom_left: coner_radius,
                                                    bottom_right: coner_radius,
                                                    g2_k_value: 3.0,
                                                })
                                                .border(GlassBorder {
                                                    width: border_width.into(),
                                                })
                                                .on_click(Arc::new(|| {
                                                    println!("Glass clicked");
                                                }))
                                                .refraction_amount(refraction_amount)
                                                .refraction_height(refraction_height)
                                                .build()
                                                .unwrap(),
                                            Some(ripple_state),
                                            || {},
                                        );
                                    });
                                },
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

            let state = state_for_glass.clone();
            scope.child(move || {
                surface_config_slider(
                    "Width",
                    state.read().width.value.0 as f32 / 500.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().width.value = Dp(f64::from(value) * 500.0);
                        })
                    },
                    state.read().width.slider_state.clone(),
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

            let state = state_for_glass.clone();

            scope.child(move || {
                surface_config_slider(
                    "Height",
                    state.read().height.value.0 as f32 / 500.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().height.value = Dp(f64::from(value) * 500.0);
                        })
                    },
                    state.read().height.slider_state.clone(),
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

            let state = state_for_glass.clone();
            scope.child(move || {
                surface_config_slider(
                    "Coner Radius",
                    state.read().coner_radius.value.0 / 100.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().coner_radius.value = ConerRadius(value * 100.0);
                        })
                    },
                    state.read().coner_radius.slider_state.clone(),
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

            let state = state_for_glass.clone();
            scope.child(move || {
                surface_config_slider(
                    "Border Width",
                    state.read().border_width.value.0 as f32 / 20.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().border_width.value = Dp(f64::from(value) * 20.0);
                        })
                    },
                    state.read().border_width.slider_state.clone(),
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

            let state = state_for_glass.clone();
            scope.child(move || {
                surface_config_slider(
                    "Refraction Strength",
                    state.read().refraction_amount.value / 100.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().refraction_amount.value = value * 100.0;
                        })
                    },
                    state.read().refraction_amount.slider_state.clone(),
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

            let state = state_for_glass.clone();
            scope.child(move || {
                surface_config_slider(
                    "Refraction Height",
                    state.read().refraction_height.value / 50.0,
                    {
                        let state = state.clone();
                        Arc::new(move |value| {
                            state.write().refraction_height.value = value * 50.0;
                        })
                    },
                    state.read().refraction_height.slider_state.clone(),
                );
            });
        },
    );
}

#[tessera]
fn surface_config_slider(
    label: &str,
    value: f32,
    on_change: Arc<dyn Fn(f32) + Send + Sync>,
    state: Arc<RwLock<SliderState>>,
) {
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
                                .width(Dp(300.0))
                                .build()
                                .unwrap(),
                            state,
                        );
                    });
                });
            });
        },
    );
}
