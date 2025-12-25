use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    material_icons::round::check_icon,
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    switch::{
        SwitchArgs, SwitchController, switch, switch_with_child_and_controller,
        switch_with_controller,
    },
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn switch_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_size()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                        move || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    column(
        ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Start),
        move |scope| {
            scope.child(|| text(TextArgs::default().text("Switch Showcase").size(Dp(20.0))));

            scope.child(move || {
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

            scope.child(|| text(TextArgs::default().text("Disabled Switch").size(Dp(16.0))));
            scope.child(move || {
                // Disabled by not providing on_change,
                switch(SwitchArgs::default());
            });
        },
    )
}
