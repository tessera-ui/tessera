use std::sync::{Arc, Mutex};

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgs, ColumnArgsBuilder, column},
    glass_button::{GlassButtonArgsBuilder, glass_button},
    icon::{IconArgsBuilder, IconContent},
    icon_button::{GlassIconButtonArgsBuilder, glass_icon_button},
    image::{ImageArgsBuilder, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
    scrollable::{ScrollableArgsBuilder, scrollable},
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
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
    let counter = remember(|| Mutex::new(0));
    let image_data = remember(|| {
        load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
            .expect("Failed to load image from embedded bytes")
    });
    let icon_data = remember(|| {
        load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(ICON_BYTES)))
            .expect("Failed to load icon SVG")
    });

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Button Showcase"));

            scope.child(|| {
                spacer(Dp(20.0));
            });

            // Glass Button on top
            let image_data = image_data.clone();
            let counter_btn = counter.clone();
            let icon_data = icon_data.clone();
            scope.child(move || {
                let image_data = image_data.clone();
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
                                    .data(image_data)
                                    .build()
                                    .unwrap(),
                            );
                        });

                        let counter = counter_btn.clone();
                        let icon_data = icon_data.clone();
                        scope.child(move || {
                            column(
                                ColumnArgs {
                                    cross_axis_alignment: CrossAxisAlignment::Center,
                                    ..Default::default()
                                },
                                move |scope| {
                                    let on_click = Arc::new({
                                        let counter_clone = counter.clone();
                                        move || {
                                            let mut count = counter_clone.lock().unwrap();
                                            *count += 1;
                                        }
                                    });

                                    scope.child(move || {
                                        glass_button(
                                            GlassButtonArgsBuilder::default()
                                                .on_click(on_click)
                                                .shape(Shape::rounded_rectangle(Dp(25.0)))
                                                .build()
                                                .unwrap(),
                                            || text("Click Me!"),
                                        );
                                    });

                                    scope.child(move || {
                                        spacer(SpacerArgs {
                                            height: Dp(20.0).into(),
                                            ..Default::default()
                                        });
                                    });

                                    let counter = counter.clone();
                                    let icon_data = icon_data.clone();
                                    scope.child(move || {
                                        let icon = IconArgsBuilder::default()
                                            .content(IconContent::from(icon_data.clone()))
                                            .size(Dp(28.0))
                                            .build()
                                            .unwrap();

                                        let on_click = Arc::new({
                                            let counter_clone = counter.clone();
                                            move || {
                                                let mut count = counter_clone.lock().unwrap();
                                                *count += 1;
                                                println!("Glass icon button clicked!");
                                            }
                                        });

                                        let args = GlassIconButtonArgsBuilder::default()
                                            .button(
                                                GlassButtonArgsBuilder::default()
                                                    .on_click(on_click)
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
                spacer(Dp(20.0));
            });

            let counter = counter.clone();
            scope.child(move || {
                let count = counter.lock().unwrap();
                text(format!("Click count: {}", *count));
            });
        },
    )
}
