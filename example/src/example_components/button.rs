use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    icon::IconArgs,
    icon_button::{IconButtonArgs, IconButtonVariant, icon_button},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
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
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                        move || {
                            column(
                                ColumnArgs::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start),
                                move |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgs::default()
                                                .text("Button Showcase")
                                                .size(Dp(20.0)),
                                        )
                                    });

                                    scope.child(|| {
                                        text(TextArgs::default().text("Icon Button").size(Dp(16.0)))
                                    });

                                    scope.child(move || {
                                        let icon =
                                            IconArgs::from(state.icon_data.clone()).size(Dp(24.0));

                                        let button_args = IconButtonArgs::new(icon)
                                            .variant(IconButtonVariant::Filled)
                                            .color(
                                                use_context::<MaterialTheme>()
                                                    .expect("MaterialTheme must be provided")
                                                    .get()
                                                    .color_scheme
                                                    .surface_variant,
                                            )
                                            .on_click(move || {
                                                counter.with_mut(|count| *count += 1);
                                                println!("Icon button clicked!");
                                            });

                                        icon_button(button_args);
                                    });

                                    scope.child(|| {
                                        spacer(Modifier::new().height(Dp(20.0)));
                                    });

                                    scope.child(|| {
                                        text(
                                            TextArgs::default()
                                                .text("Button Styles")
                                                .size(Dp(20.0)),
                                        )
                                    });

                                    scope.child(|| {
                                        row(RowArgs::default(), |scope| {
                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::filled(|| {
                                                        println!("Filled clicked")
                                                    }),
                                                    || {
                                                        text(
                                                            TextArgs::default()
                                                                .text("Filled")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .expect("MaterialTheme must be provided")
                                                                        .get()
                                                                        .color_scheme
                                                                        .on_primary,
                                                                ),
                                                        );
                                                    },
                                                );
                                            });
                                            scope.child(|| spacer(Modifier::new().width(Dp(8.0))));

                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::elevated(|| {
                                                        println!("Elevated clicked")
                                                    }),
                                                    || {
                                                        text(
                                                            TextArgs::default()
                                                                .text("Elevated")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .expect("MaterialTheme must be provided")
                                                                        .get()
                                                                        .color_scheme
                                                                        .primary,
                                                                ),
                                                        );
                                                    },
                                                );
                                            });
                                            scope.child(|| spacer(Modifier::new().width(Dp(8.0))));

                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::tonal(|| println!("Tonal clicked")),
                                                    || {
                                                        text(
                                                            TextArgs::default()
                                                                .text("Tonal")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .expect("MaterialTheme must be provided")
                                                                        .get()
                                                                        .color_scheme
                                                                        .on_secondary_container,
                                                                ),
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                    });

                                    scope.child(|| spacer(Modifier::new().height(Dp(8.0))));

                                    scope.child(|| {
                                        row(RowArgs::default(), move |scope| {
                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::outlined(|| {
                                                        println!("Outlined clicked")
                                                    }),
                                                    || {
                                                        text(
                                                            TextArgs::default()
                                                                .text("Outlined")
                                                                .color(
                                                                    use_context::<MaterialTheme>()
                                                                        .expect("MaterialTheme must be provided")
                                                                        .get()
                                                                        .color_scheme
                                                                        .primary,
                                                                ),
                                                        );
                                                    },
                                                );
                                            });
                                            scope.child(|| spacer(Modifier::new().width(Dp(8.0))));

                                            scope.child(|| {
                                                button(
                                                    ButtonArgs::text(|| println!("Text clicked")),
                                                    || {
                                                        text(
                                                            TextArgs::default().text("Text").color(
                                                                use_context::<MaterialTheme>()
                                                                    .expect("MaterialTheme must be provided")
                                                                    .get()
                                                                    .color_scheme
                                                                    .primary,
                                                            ),
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                    });
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
