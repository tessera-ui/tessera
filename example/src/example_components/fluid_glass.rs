use std::{fmt::Display, sync::Arc};

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    fluid_glass::{FluidGlassArgs, GlassBorder, fluid_glass},
    glass_slider::{GlassSliderArgs, glass_slider},
    image::{ImageArgs, ImageData, ImageSource, image, load_image_from_source},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    shape_def::{RoundedCorner, Shape},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{CallbackWith, Dp, Modifier, remember, retain, shard, tessera};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/grid_background.png",
));

struct CornerRadius(f32);

impl Display for CornerRadius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

struct ExampleGlassState {
    width: ConfigSliderState<Dp>,
    height: ConfigSliderState<Dp>,
    border_width: ConfigSliderState<Dp>,
    corner_radius: ConfigSliderState<CornerRadius>,
    refraction_amount: ConfigSliderState<f32>,
    refraction_height: ConfigSliderState<Dp>,
    blur_radius: ConfigSliderState<Dp>,
    background_image_data: Arc<ImageData>,
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

impl Display for ExampleGlassState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Width: {}", self.width.value)?;
        writeln!(f, "Height: {}", self.height.value)?;
        writeln!(f, "Border Width: {}", self.border_width.value)?;
        writeln!(f, "Corner Radius: {}", self.corner_radius.value)?;
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
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
                .expect("Failed to load image from embedded bytes"),
        );

        Self {
            width: ConfigSliderState::new(Dp(100.0)),
            height: ConfigSliderState::new(Dp(100.0)),
            border_width: ConfigSliderState::new(Dp(1.0)),
            corner_radius: ConfigSliderState::new(CornerRadius(25.0)),
            refraction_amount: ConfigSliderState::new(32.0),
            refraction_height: ConfigSliderState::new(Dp(24.0)),
            blur_radius: ConfigSliderState::new(Dp(0.0)),
            background_image_data: image_data,
        }
    }
}
#[tessera]
#[shard]
pub fn fluid_glass_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let state = remember(ExampleGlassState::default);
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .controller(controller)
            .content(move |scope| {
                scope.item(move || {
                    let (
                        corner_radius,
                        width,
                        height,
                        border_width,
                        state_string,
                        refraction_amount,
                        refraction_height,
                        blur_radius,
                    ) = state.with(|s| {
                        (
                            Dp(s.corner_radius.value.0 as f64),
                            s.width.value,
                            s.height.value,
                            s.border_width.value,
                            s.to_string(),
                            s.refraction_amount.value,
                            s.refraction_height.value,
                            s.blur_radius.value,
                        )
                    });

                    row(
                        RowArgs::default()
                            .modifier(Modifier::new().fill_max_width())
                            .main_axis_alignment(MainAxisAlignment::Center)
                            .cross_axis_alignment(CrossAxisAlignment::Center),
                        move |scope| {
                            scope.child(move || {
                                boxed(BoxedArgs::default().alignment(Alignment::Center), |scope| {
                                    scope.child(move || {
                                        image(&ImageArgs {
                                            data: state.with(|s| s.background_image_data.clone()),
                                            modifier: Modifier::new().size(Dp(250.0), Dp(250.0)),
                                        });
                                    });

                                    scope.child(move || {
                                        fluid_glass(&FluidGlassArgs::with_child(
                                            FluidGlassArgs::default()
                                                .modifier(Modifier::new().size(width, height))
                                                .blur_radius(blur_radius)
                                                .shape(Shape::RoundedRectangle {
                                                    top_left: RoundedCorner::manual(
                                                        corner_radius,
                                                        3.0,
                                                    ),
                                                    top_right: RoundedCorner::manual(
                                                        corner_radius,
                                                        3.0,
                                                    ),
                                                    bottom_left: RoundedCorner::manual(
                                                        corner_radius,
                                                        3.0,
                                                    ),
                                                    bottom_right: RoundedCorner::manual(
                                                        corner_radius,
                                                        3.0,
                                                    ),
                                                })
                                                .border(GlassBorder {
                                                    width: border_width.into(),
                                                })
                                                .on_click(|| {
                                                    println!("Glass clicked");
                                                })
                                                .refraction_amount(refraction_amount)
                                                .refraction_height(refraction_height),
                                            || {},
                                        ));
                                    });
                                });
                            });

                            scope.child(|| {
                                spacer(&SpacerArgs::new(Modifier::new().width(Dp(16.0))))
                            });

                            scope.child(move || {
                                text(
                                    &TextArgs::default()
                                        .text(state_string.clone())
                                        .size(Dp(16.0)),
                                );
                            });
                        },
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Width",
                        state.with(|s| s.width.value.0 as f32 / 500.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.width.value = Dp(f64::from(value) * 500.0));
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Height",
                        state.with(|s| s.height.value.0 as f32 / 500.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.height.value = Dp(f64::from(value) * 500.0));
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Corner Radius",
                        state.with(|s| s.corner_radius.value.0 / 100.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.corner_radius.value = CornerRadius(value * 100.0));
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Border Width",
                        state.with(|s| s.border_width.value.0 as f32 / 20.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.border_width.value = Dp(f64::from(value) * 20.0));
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Refraction Strength",
                        state.with(|s| s.refraction_amount.value / 100.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.refraction_amount.value = value * 100.0);
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Refraction Height",
                        state.with(|s| s.refraction_height.value.0 as f32 / 50.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| {
                                s.refraction_height.value = Dp(f64::from(value * 50.0))
                            })
                        }),
                    );
                });

                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(32.0)))));

                scope.item(move || {
                    glass_config_slider(
                        "Blur Radius",
                        state.with(|s| s.blur_radius.value.0 as f32 / 100.0),
                        CallbackWith::new(move |value| {
                            state.with_mut(|s| s.blur_radius.value = Dp(f64::from(value * 100.0)));
                        }),
                    );
                });
            }),
    );
}
fn glass_config_slider(label: &str, value: f32, on_change: CallbackWith<f32>) {
    let label = label.to_string();
    column(
        ColumnArgs::default()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(move || {
                let label = label.clone();
                let on_change = on_change.clone();
                column(ColumnArgs::default(), |scope| {
                    scope.child(move || {
                        text(&TextArgs::default().text(label.clone()).size(Dp(16.0)));
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(16.0)))));

                    scope.child({
                        move || {
                            glass_slider(
                                &GlassSliderArgs::default()
                                    .value(value)
                                    .on_change_shared(on_change.clone())
                                    .modifier(Modifier::new().width(Dp(300.0))),
                            );
                        }
                    });
                });
            });
        },
    );
}
