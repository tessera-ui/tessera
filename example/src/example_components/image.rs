use tessera_components::{
    icon::{IconArgs, icon},
    image::{ImageArgs, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};

const IMAGE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/scarlet_ut.jpg",
));
const VECTOR_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../assets/emoji_u1f416.svg"
));
#[tessera]
#[shard]
pub fn image_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let image_data = remember(|| {
        load_image_from_source(&ImageSource::Bytes(IMAGE_BYTES.into()))
            .expect("Failed to load image data")
    });
    let image_vector_data = remember(|| {
        load_image_vector_from_source(&ImageVectorSource::Bytes(VECTOR_BYTES.into()))
            .expect("Failed to load image vector data")
    });
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0))
            .controller(controller)
            .content(move |scope| {
                scope.item(|| text(&TextArgs::from("Image Showcase")));
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(10.0)))));
                scope.item(|| text(&TextArgs::from("Raster image")));
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(move || image(&ImageArgs::from(image_data.get())));
                scope.item(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(24.0))));
                });
                scope.item(|| text(&TextArgs::from("Icon (vector source)")));
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(8.0)))));
                scope.item(move || icon(&IconArgs::from(image_vector_data.get()).size(Dp(160.0))));
            }),
    )
}
#[tessera]
#[shard]
pub fn icon_showcase() {
    let image_vector_data = remember(|| {
        load_image_vector_from_source(&ImageVectorSource::Bytes(VECTOR_BYTES.into()))
            .expect("Failed to load image vector data")
    });
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .content_padding(Dp(16.0))
                    .content(move |scope| {
                        scope.item(|| text(&TextArgs::from("Icon Showcase")));
                        scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(10.0)))));
                        scope.item(move || {
                            icon(&IconArgs::from(image_vector_data.get()).size(Dp(100.0)))
                        });
                    }),
            );
        },
    ));
}
