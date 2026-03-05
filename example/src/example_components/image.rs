use tessera_components::{
    icon::{IconArgs, icon},
    image::{ImageArgs, TryIntoImageData, image},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{AssetExt, Dp, Modifier, remember, retain, shard};
#[shard]
pub fn image_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}

fn test_content() {
    let image_data = remember(|| {
        let image_bytes = crate::res::SCARLET_UT_JPG
            .read()
            .expect("Failed to read raster asset bytes");
        image_bytes
            .as_ref()
            .try_into_image_data()
            .expect("Failed to load image data")
    });
    let icon_args = remember(|| {
        IconArgs::default()
            .try_vector_asset(crate::res::EMOJI_U1F416_SVG)
            .expect("Failed to load image vector icon")
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
                scope.item(move || icon(&icon_args.get().size(Dp(160.0))));
            }),
    )
}
#[shard]
pub fn icon_showcase() {
    let icon_args = remember(|| {
        IconArgs::default()
            .try_vector_asset(crate::res::EMOJI_U1F416_SVG)
            .expect("Failed to load image vector icon")
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
                        scope.item(move || icon(&icon_args.get().size(Dp(100.0))));
                    }),
            );
        },
    ));
}
