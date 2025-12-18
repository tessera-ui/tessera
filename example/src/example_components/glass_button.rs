use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgs, ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{GlassIconButtonArgsBuilder, glass_icon_button},
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgsBuilder, scrollable},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/grid_background.png",
));
const ICON_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

#[tessera]
#[shard]
pub fn glass_button_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                            .build()
                            .unwrap(),
                        move || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    let counter = remember(|| 0);
    let image_data = remember(|| {
        Arc::new(
            load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
                .expect("Failed to load image from embedded bytes"),
        )
    });
    let icon_data = remember(|| {
        Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
                .expect("Failed to load icon SVG"),
        )
    });

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Button Showcase"));

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });
            scope.child(move || {
                boxed(
                    BoxedArgsBuilder::default()
                        .alignment(Alignment::Center)
                        .build()
                        .unwrap(),
                    move |scope| {
                        scope.child(move || {
                            image(
                                ImageArgsBuilder::default()
                                    .width(DimensionValue::from(Dp(250.0)))
                                    .height(DimensionValue::from(Dp(250.0)))
                                    .data(image_data.get())
                                    .build()
                                    .unwrap(),
                            );
                        });

                        scope.child(move || {
                            column(
                                ColumnArgs {
                                    cross_axis_alignment: CrossAxisAlignment::Center,
                                    ..Default::default()
                                },
                                move |scope| {
                                    let on_click = Arc::new(move || {
                                        counter.with_mut(|c| *c += 1);
                                    });

                                    scope.child(move || {
                                        glass_button(
                                            GlassButtonArgsBuilder::default()
                                                .on_click_shared(on_click)
                                                .shape(Shape::rounded_rectangle(Dp(25.0)))
                                                .build()
                                                .unwrap(),
                                            || text("Click Me!"),
                                        );
                                    });

                                    scope.child(move || {
                                        spacer(Modifier::new().height(Dp(20.0)));
                                    });

                                    scope.child(move || {
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(icon_data.get().clone()))
                                            .size(Dp(28.0))
                                            .build()
                                            .unwrap();

                                        let on_click = Arc::new(move || {
                                            counter.with_mut(|c| *c += 1);
                                        });

                                        let args = GlassIconButtonArgsBuilder::default()
                                            .button(
                                                GlassButtonArgsBuilder::default()
                                                    .on_click_shared(on_click)
                                                    .shape(Shape::Ellipse)
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .icon(icon)
                                            .build()
                                            .unwrap();

                                        glass_icon_button(args);
                                    });
                                },
                            );
                        });
                    },
                );
            });

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.child(move || {
                text(format!("Click count: {}", counter.get()));
            });
        },
    )
}
