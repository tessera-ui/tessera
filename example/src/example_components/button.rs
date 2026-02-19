use std::sync::Arc;

use tessera_components::{
    button::{ButtonArgs, button},
    icon::IconArgs,
    icon_button::{IconButtonArgs, IconButtonVariant, icon_button},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, use_context};

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
#[shard]
pub fn button_showcase(#[state] state: ButtonShowcaseState) {
    let counter = remember(|| 0i32);
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            let state = state.clone();
            let controller = retain(LazyListController::new);
            lazy_column(
                &LazyColumnArgs::default()
                    .content_padding(Dp(16.0))
                    .controller(controller)
                    .content(move |scope| {
                        let state = state.clone();
                        scope.item(|| {
                            text(&TextArgs::default().text("Button Showcase").size(Dp(20.0)))
                        });

                        scope
                            .item(|| text(&TextArgs::default().text("Icon Button").size(Dp(16.0))));

                        scope.item(move || {
                            let icon = IconArgs::from(state.icon_data.clone()).size(Dp(24.0));

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

                            icon_button(&button_args);
                        });

                        scope.item(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0))));
                        });

                        scope.item(|| {
                            text(&TextArgs::default().text("Button Styles").size(Dp(20.0)))
                        });

                        scope.item(|| {
                            row(RowArgs::default(), |scope| {
                                scope.child(|| {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::filled(|| println!("Filled clicked")),
                                        || {
                                            text(
                                                &TextArgs::default().text("Filled").color(
                                                    use_context::<MaterialTheme>()
                                                        .expect("MaterialTheme must be provided")
                                                        .get()
                                                        .color_scheme
                                                        .on_primary,
                                                ),
                                            );
                                        },
                                    ));
                                });
                                scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(8.0))))
                                });

                                scope.child(|| {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::elevated(|| println!("Elevated clicked")),
                                        || {
                                            text(
                                                &TextArgs::default().text("Elevated").color(
                                                    use_context::<MaterialTheme>()
                                                        .expect("MaterialTheme must be provided")
                                                        .get()
                                                        .color_scheme
                                                        .primary,
                                                ),
                                            );
                                        },
                                    ));
                                });
                                scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(8.0))))
                                });

                                scope.child(|| {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::tonal(|| println!("Tonal clicked")),
                                        || {
                                            text(
                                                &TextArgs::default().text("Tonal").color(
                                                    use_context::<MaterialTheme>()
                                                        .expect("MaterialTheme must be provided")
                                                        .get()
                                                        .color_scheme
                                                        .on_secondary_container,
                                                ),
                                            );
                                        },
                                    ));
                                });
                            });
                        });

                        scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));

                        scope.item(|| {
                            row(RowArgs::default(), move |scope| {
                                scope.child(|| {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::outlined(|| println!("Outlined clicked")),
                                        || {
                                            text(
                                                &TextArgs::default().text("Outlined").color(
                                                    use_context::<MaterialTheme>()
                                                        .expect("MaterialTheme must be provided")
                                                        .get()
                                                        .color_scheme
                                                        .primary,
                                                ),
                                            );
                                        },
                                    ));
                                });
                                scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(8.0))))
                                });

                                scope.child(|| {
                                    button(&ButtonArgs::with_child(
                                        ButtonArgs::text(|| println!("Text clicked")),
                                        || {
                                            text(
                                                &TextArgs::default().text("Text").color(
                                                    use_context::<MaterialTheme>()
                                                        .expect("MaterialTheme must be provided")
                                                        .get()
                                                        .color_scheme
                                                        .primary,
                                                ),
                                            );
                                        },
                                    ));
                                });
                            });
                        });
                    }),
            );
        },
    ))
}
