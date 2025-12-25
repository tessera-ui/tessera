use std::sync::Arc;

use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    image::{ImageArgs, ImageData, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorData, ImageVectorSource, load_image_vector_from_source},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
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
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
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
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        |scope| {
            scope.child(|| text("Image Showcase"));

            scope.child(|| {
                spacer(Modifier::new().height(Dp(10.0)));
            });

            scope.child(move || {
                column(
                    ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
                    |column_scope| {
                        column_scope.child(|| text("Raster image"));
                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(8.0)));
                        });
                        let raster = state.image_data.clone();
                        column_scope.child(move || {
                            image(ImageArgs {
                                data: raster.clone(),
                                modifier: Modifier::new(),
                            })
                        });

                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(24.0)));
                        });

                        column_scope.child(|| text("Icon (vector source)"));
                        column_scope.child(|| {
                            spacer(Modifier::new().height(Dp(8.0)));
                        });
                        let vector = state.image_vector_data.clone();
                        column_scope
                            .child(move || icon(IconArgs::from(vector.clone()).size(Dp(160.0))));
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
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                        move || {
                            column(
                                ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
                                |scope| {
                                    scope.child(|| text("Icon Showcase"));
                                    scope.child(|| spacer(Modifier::new().height(Dp(10.0))));
                                    let vector = state.image_vector_data.clone();
                                    scope.child(move || {
                                        icon(IconArgs::from(vector.clone()).size(Dp(100.0)))
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
