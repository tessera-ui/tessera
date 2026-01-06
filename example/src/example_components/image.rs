use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};
use tessera_ui_basic_components::{
    icon::{IconArgs, icon},
    image::{ImageArgs, ImageSource, image, load_image_from_source},
    image_vector::{ImageVectorSource, load_image_vector_from_source},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column, lazy_column_with_controller},
    modifier::ModifierExt as _,
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

#[tessera]
#[shard]
pub fn image_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    );
}

#[tessera]
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
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0)),
        controller,
        move |scope| {
            scope.item(|| text("Image Showcase"));
            scope.item(|| spacer(Modifier::new().height(Dp(10.0))));
            scope.item(|| text("Raster image"));
            scope.item(|| spacer(Modifier::new().height(Dp(8.0))));
            scope.item(move || image(ImageArgs::from(image_data.get())));
            scope.item(|| {
                spacer(Modifier::new().height(Dp(24.0)));
            });
            scope.item(|| text("Icon (vector source)"));
            scope.item(|| spacer(Modifier::new().height(Dp(8.0))));
            scope.item(move || icon(IconArgs::from(image_vector_data.get()).size(Dp(160.0))));
        },
    )
}

#[tessera]
#[shard]
pub fn icon_showcase() {
    let image_vector_data = remember(|| {
        load_image_vector_from_source(&ImageVectorSource::Bytes(VECTOR_BYTES.into()))
            .expect("Failed to load image vector data")
    });
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .content_padding(Dp(16.0)),
                move |scope| {
                    scope.item(|| text("Icon Showcase"));
                    scope.item(|| spacer(Modifier::new().height(Dp(10.0))));
                    scope.item(move || {
                        icon(IconArgs::from(image_vector_data.get()).size(Dp(100.0)))
                    });
                },
            );
        },
    );
}
