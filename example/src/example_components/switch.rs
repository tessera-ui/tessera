use tessera_components::{
    icon::{IconArgs, icon},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    material_icons::round::check_icon,
    modifier::ModifierExt as _,
    surface::{SurfaceArgs, surface},
    switch::{
        SwitchArgs, SwitchController, switch, switch_with_child_and_controller,
        switch_with_controller,
    },
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera};

#[tessera]
#[shard]
pub fn switch_showcase() {
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
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width()),
        controller,
        move |scope| {
            scope.item(|| text(TextArgs::default().text("Switch Showcase").size(Dp(20.0))));
            scope.item(move || {
                let controller = remember(|| SwitchController::new(false));
                if controller.with(|c| c.is_checked()) {
                    switch_with_child_and_controller(
                        SwitchArgs::default().on_toggle(|value| {
                            println!("Switch toggled to: {}", value);
                        }),
                        controller,
                        move || {
                            icon(IconArgs::from(check_icon()).size(Dp(16.0)));
                        },
                    );
                } else {
                    switch_with_controller(
                        SwitchArgs::default().on_toggle(|value| {
                            println!("Switch toggled to: {}", value);
                        }),
                        controller,
                    );
                }
            });
            scope.item(|| text(TextArgs::default().text("Disabled Switch").size(Dp(16.0))));
            scope.item(move || {
                // Disabled by not providing on_change,
                switch(SwitchArgs::default());
            });
        },
    );
}
