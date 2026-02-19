use tessera_components::{
    glass_switch::{GlassSwitchArgs, glass_switch},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, retain, shard, tessera};
#[tessera]
#[shard]
pub fn glass_switch_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0))
            .controller(controller)
            .content(move |scope| {
                scope.item(|| text(&TextArgs::from("Glass Switch Showcase")));

                scope.item(move || {
                    glass_switch(&GlassSwitchArgs::default().on_toggle(|value| {
                        println!("Glass Switch toggled to: {}", value);
                    }));
                });

                scope.item(|| {
                    text(
                        &TextArgs::default()
                            .text("Disabled Glass Switch")
                            .size(Dp(16.0)),
                    )
                });
                scope.item(|| {
                    // Disabled by not providing on_change,
                    glass_switch(&GlassSwitchArgs::default());
                });
            }),
    );
}
