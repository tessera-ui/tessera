use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonArgs, button},
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{IconButtonArgsBuilder, IconButtonVariant, icon_button},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialColorScheme,
};

const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[derive(Clone)]
struct ButtonShowcaseState {
    counter: Arc<Mutex<i32>>,
    icon_data: Arc<ImageVectorData>,
}

impl ButtonShowcaseState {
    fn new() -> Self {
        let icon_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        );

        Self {
            counter: Default::default(),
            icon_data,
        }
    }
}

impl Default for ButtonShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

#[tessera]
#[shard]
pub fn button_showcase(#[state] state: ButtonShowcaseState) {
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
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .build()
                                    .unwrap(),
                                move |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Showcase")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Icon Button")
                                                .size(Dp(16.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(closure!(|| {
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(state.icon_data.clone()))
                                            .size(Dp(24.0))
                                            .build()
                                            .unwrap();

                                        let button_args = IconButtonArgsBuilder::default()
                                            .variant(IconButtonVariant::Filled)
                                            .color(
                                                use_context::<MaterialColorScheme>()
                                                    .surface_variant,
                                            )
                                            .on_click(closure!(
                                                clone state.counter,
                                                || {
                                                    let mut count =
                                                        counter.lock().unwrap();
                                                    *count += 1;
                                                    println!("Icon button clicked!");
                                                }
                                            ))
                                            .icon(icon)
                                            .build()
                                            .unwrap();

                                        icon_button(button_args);
                                    }));

                                    scope.child(|| {
                                        spacer(
                                            SpacerArgsBuilder::default()
                                                .height(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        );
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Styles")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(|| {
                                        row(RowArgsBuilder::default().build().unwrap(), |scope| {
                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::filled(Arc::new(|| {
                                                        println!("Filled clicked")
                                                    })),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Filled")
                                                                .color(
                                                                    use_context::<
                                                                        MaterialColorScheme,
                                                                    >(
                                                                    )
                                                                    .on_primary,
                                                                )
                                                                .build()
                                                                .unwrap(),
                                                        );
                                                    },
                                                );
                                            });
                                            scope.child(|| {
                                                spacer(
                                                    SpacerArgsBuilder::default()
                                                        .width(Dp(8.0))
                                                        .build()
                                                        .unwrap(),
                                                )
                                            });

                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::elevated(Arc::new(|| {
                                                        println!("Elevated clicked")
                                                    })),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Elevated")
                                                                .color(
                                                                    use_context::<
                                                                        MaterialColorScheme,
                                                                    >(
                                                                    )
                                                                    .primary,
                                                                )
                                                                .build()
                                                                .unwrap(),
                                                        );
                                                    },
                                                );
                                            });
                                            scope.child(|| {
                                                spacer(
                                                    SpacerArgsBuilder::default()
                                                        .width(Dp(8.0))
                                                        .build()
                                                        .unwrap(),
                                                )
                                            });

                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::tonal(Arc::new(|| {
                                                        println!("Tonal clicked")
                                                    })),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Tonal")
                                                                .color(
                                                                    use_context::<
                                                                        MaterialColorScheme,
                                                                    >(
                                                                    )
                                                                    .on_secondary_container,
                                                                )
                                                                .build()
                                                                .unwrap(),
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                    });

                                    scope.child(|| {
                                        spacer(
                                            SpacerArgsBuilder::default()
                                                .height(Dp(8.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    scope.child(closure!(|| {
                                        row(
                                            RowArgsBuilder::default().build().unwrap(),
                                            move |scope| {
                                                scope.child(closure!(|| {
                                                    button(
                                                        ButtonArgs::outlined(Arc::new(|| {
                                                            println!("Outlined clicked")
                                                        })),
                                                        || {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Outlined")
                                                                    .color(
                                                                        use_context::<
                                                                            MaterialColorScheme,
                                                                        >(
                                                                        )
                                                                        .primary,
                                                                    )
                                                                    .build()
                                                                    .unwrap(),
                                                            );
                                                        },
                                                    );
                                                }));
                                                scope.child(|| {
                                                    spacer(
                                                        SpacerArgsBuilder::default()
                                                            .width(Dp(8.0))
                                                            .build()
                                                            .unwrap(),
                                                    )
                                                });

                                                scope.child(closure!(|| {
                                                    button(
                                                        ButtonArgs::text(Arc::new(|| {
                                                            println!("Text clicked")
                                                        })),
                                                        || {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Text")
                                                                    .color(
                                                                        use_context::<
                                                                            MaterialColorScheme,
                                                                        >(
                                                                        )
                                                                        .primary,
                                                                    )
                                                                    .build()
                                                                    .unwrap(),
                                                            );
                                                        },
                                                    );
                                                }));
                                            },
                                        );
                                    }));
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
