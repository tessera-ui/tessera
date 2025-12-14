use std::sync::Arc;

use closure::closure;
use tessera_ui::{DimensionValue, Dp, remember, shard, tessera, use_context};
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
    theme::MaterialTheme,
};

const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[derive(Clone)]
struct ButtonShowcaseState {
    icon_data: Arc<ImageVectorData>,
}

impl ButtonShowcaseState {
    fn new() -> Self {
        let icon_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        );

        Self { icon_data }
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
    let counter = remember(|| 0i32);
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
                                                use_context::<MaterialTheme>()
                                                    .get()
                                                    .color_scheme
                                                    .surface_variant,
                                            )
                                            .on_click(closure!(
                                                clone counter,
                                                || {
                                                    counter.with_mut(|count| *count += 1);
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
                                                    ButtonArgs::filled(|| {
                                                        println!("Filled clicked")
                                                    }),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Filled")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .get()
                                                                        .color_scheme
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
                                                    ButtonArgs::elevated(|| {
                                                        println!("Elevated clicked")
                                                    }),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Elevated")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .get()
                                                                        .color_scheme
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
                                                    ButtonArgs::tonal(|| println!("Tonal clicked")),
                                                    || {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("Tonal")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .get()
                                                                        .color_scheme
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
                                                        ButtonArgs::outlined(|| {
                                                            println!("Outlined clicked")
                                                        }),
                                                        || {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Outlined")
                                                                    .color(
                                                                        use_context::<MaterialTheme>()
                                                                            .get()
                                                                            .color_scheme
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
                                                        ButtonArgs::text(|| {
                                                            println!("Text clicked")
                                                        }),
                                                        || {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Text")
                                                                    .color(
                                                                        use_context::<MaterialTheme>()
                                                                            .get()
                                                                            .color_scheme
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
