use std::sync::Arc;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    glass_button::{GlassButtonArgs, glass_button},
    icon::IconArgs,
    icon_button::{GlassIconButtonArgs, glass_icon_button},
    image::{ImageArgs, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt as _,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::text,
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};

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
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
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
    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .content_padding(Dp(16.0)),
        controller,
        move |scope| {
            scope.item(|| text("Glass Button Showcase"));

            scope.item(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });
            scope.item(move || {
                boxed(
                    BoxedArgs::default().alignment(Alignment::Center),
                    move |scope| {
                        scope.child(move || {
                            image(ImageArgs {
                                data: image_data.get(),
                                modifier: Modifier::new().size(Dp(250.0), Dp(250.0)),
                            });
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
                                            GlassButtonArgs::default()
                                                .on_click_shared(on_click)
                                                .shape(Shape::rounded_rectangle(Dp(25.0))),
                                            || text("Click Me!"),
                                        );
                                    });

                                    scope.child(move || {
                                        spacer(Modifier::new().height(Dp(20.0)));
                                    });

                                    scope.child(move || {
                                        let icon =
                                            IconArgs::from(icon_data.get().clone()).size(Dp(28.0));

                                        let on_click = Arc::new(move || {
                                            counter.with_mut(|c| *c += 1);
                                        });

                                        let args = GlassIconButtonArgs::new(icon).button(
                                            GlassButtonArgs::default()
                                                .on_click_shared(on_click)
                                                .shape(Shape::Ellipse),
                                        );

                                        glass_icon_button(args);
                                    });
                                },
                            );
                        });
                    },
                );
            });

            scope.item(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.item(move || {
                text(format!("Click count: {}", counter.get()));
            });
        },
    );
}
