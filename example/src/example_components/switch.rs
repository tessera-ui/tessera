use tessera_components::{
    icon::{IconArgs, icon},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    material_icons::round::check_icon,
    modifier::ModifierExt as _,
    surface::{SurfaceArgs, surface},
    switch::{SwitchArgs, SwitchController, switch},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard};
#[shard]
pub fn switch_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width())
            .controller(controller)
            .content(move |scope| {
                scope.item(|| text(&TextArgs::default().text("Switch Showcase").size(Dp(20.0))));
                scope.item(move || {
                    let controller = remember(|| SwitchController::new(false));
                    if controller.with(|c| c.is_checked()) {
                        let args = SwitchArgs::default()
                            .on_toggle(|value| {
                                println!("Switch toggled to: {}", value);
                            })
                            .controller(controller)
                            .child(move || {
                                icon(&IconArgs::from(check_icon()).size(Dp(16.0)));
                            });
                        switch(&args);
                    } else {
                        let args = SwitchArgs::default()
                            .on_toggle(|value| {
                                println!("Switch toggled to: {}", value);
                            })
                            .controller(controller);
                        switch(&args);
                    }
                });
                scope.item(|| text(&TextArgs::default().text("Disabled Switch").size(Dp(16.0))));
                scope.item(move || {
                    // Disabled by not providing on_change,
                    switch(&SwitchArgs::default());
                });
            }),
    );
}
