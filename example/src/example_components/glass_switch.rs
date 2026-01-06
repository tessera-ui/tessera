use tessera_ui::{Dp, Modifier, retain, shard, tessera};
use tessera_ui_basic_components::{
    glass_switch::{GlassSwitchArgs, glass_switch},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt as _,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn glass_switch_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    );
}

#[tessera]
fn test_content() {
    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0)),
        controller,
        move |scope| {
            scope.item(|| text("Glass Switch Showcase"));

            scope.item(move || {
                glass_switch(GlassSwitchArgs::default().on_toggle(|value| {
                    println!("Glass Switch toggled to: {}", value);
                }));
            });

            scope.item(|| {
                text(
                    TextArgs::default()
                        .text("Disabled Glass Switch")
                        .size(Dp(16.0)),
                )
            });
            scope.item(|| {
                // Disabled by not providing on_change,
                glass_switch(GlassSwitchArgs::default());
            });
        },
    );
}
