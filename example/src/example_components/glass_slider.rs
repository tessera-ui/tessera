use tessera_components::{
    glass_slider::{GlassSliderArgs, GlassSliderController, glass_slider},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};
#[tessera]
#[shard]
pub fn glass_slider_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let value = remember(|| 0.5);
    let slider_controller = remember(GlassSliderController::new);
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0))
            .controller(controller)
            .content(move |scope| {
                scope.item(|| text(&TextArgs::from("Glass Slider Showcase")));
                scope.item(move || {
                    let args = GlassSliderArgs::default()
                        .value(value.get())
                        .on_change(move |new_value| {
                            value.set(new_value);
                        })
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .controller(slider_controller);
                    glass_slider(&args);
                });

                scope.item(move || {
                    text(
                        &TextArgs::default()
                            .text(format!("Value: {:.2}", value.get()))
                            .size(Dp(16.0)),
                    );
                });
            }),
    );
}
