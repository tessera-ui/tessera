use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, icon},
    image::{ImageArgsBuilder, ImageData, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
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
    scrollable_state: ScrollableState,
    vector_scrollable_state: ScrollableState,
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
            scrollable_state: ScrollableState::default(),
            vector_scrollable_state: ScrollableState::default(),
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
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                state.vector_scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
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
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| text("Image Showcase"));

            scope.child(|| {
                spacer(Dp(10.0));
            });

            scope.child(move || {
                column(
                    ColumnArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .build()
                        .unwrap(),
                    |column_scope| {
                        column_scope.child(|| text("Raster image"));
                        column_scope.child(|| {
                            spacer(Dp(8.0));
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
                            spacer(Dp(24.0));
                        });

                        column_scope.child(|| text("Icon (vector source)"));
                        column_scope.child(|| {
                            spacer(Dp(8.0));
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
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
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
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .width(DimensionValue::FILLED)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| text("Icon Showcase"));
                                    scope.child(|| spacer(Dp(10.0)));
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
