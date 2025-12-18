use std::sync::Arc;

use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, icon},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgsBuilder, scrollable},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/scarlet_ut.jpg",
));
const VECTOR_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));

pub struct ImageShowcaseState {
    image_data: Arc<ImageData>,
    image_vector_data: Arc<ImageVectorData>,
}

impl Default for ImageShowcaseState {
    fn default() -> Self {
        let image_data = Arc::new(
            load_image_from_source(&ImageSource::Bytes(Arc::from(IMAGE_BYTES)))
                .expect("Failed to load image from embedded bytes"),
        );

        let image_vector_data = Arc::new(
            load_image_vector_from_source(&ImageVectorSource::Bytes(Arc::from(VECTOR_BYTES)))
                .expect("Failed to load SVG from embedded bytes"),
        );

        Self {
            image_data,
            image_vector_data,
        }
    }
}

#[tessera]
#[shard]
pub fn image_showcase(#[state] state: ImageShowcaseState) {
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
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        move || {
                            test_content(state);
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<ImageShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| text("Image Showcase"));

            scope.child(|| {
                spacer(Modifier::new().height(Dp(10.0)));
            });

            scope.child(move || {
                column(
                    ColumnArgsBuilder::default()
                        .modifier(Modifier::new().fill_max_width())
                        .build()
                        .unwrap(),
                    |column_scope| {
                        column_scope.child(|| text("Raster image"));
                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(8.0)));
                        });
                        let raster = state.image_data.clone();
                        column_scope.child(move || {
                            image(
                                ImageArgsBuilder::default()
                                    .data(raster.clone())
                                    .build()
                                    .unwrap(),
                            )
                        });

                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(24.0)));
                        });

                        column_scope.child(|| text("Icon (vector source)"));
                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(8.0)));
                        });
                        let vector = state.image_vector_data.clone();
                        column_scope.child(move || {
                            icon(
                                IconArgsBuilder::default()
                                    .content(vector.clone())
                                    .size(Dp(160.0))
                                    .build()
                                    .unwrap(),
                            )
                        });
                    },
                )
            });
        },
    )
}

#[tessera]
#[shard]
pub fn icon_showcase(#[state] state: ImageShowcaseState) {
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
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .modifier(Modifier::new().fill_max_width())
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| text("Icon Showcase"));
                                    scope.child(|| spacer(Modifier::new().height(Dp(10.0))));
                                    let vector = state.image_vector_data.clone();
                                    scope.child(move || {
                                        icon(
                                            IconArgsBuilder::default()
                                                .content(vector.clone())
                                                .size(Dp(100.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });
                                },
                            );
                        },
                    );
                },
            )
        },
    );
}
