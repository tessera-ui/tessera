use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    menus::{
        MenuAnchor, MenuController, MenuItemArgs, MenuPlacement, MenuProviderArgs,
        menu_provider_with_controller,
    },
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn menus_showcase() {
    let menu_controller = remember(MenuController::new);
    let selected_label = remember(|| "None".to_string());
    let pinned = remember(|| false);
    // Anchor near the trigger button (padding 20dp + title/subtitle + spacer).
    let anchor = MenuAnchor::from_dp((Dp(20.0), Dp(72.0)), (Dp(180.0), Dp(48.0)));

    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            menu_provider_with_controller(
                MenuProviderArgs::default()
                    .placement(MenuPlacement::BelowStart)
                    .offset([Dp(0.0), Dp(6.0)]),
                menu_controller,
                {
                    move || {
                        column(
                            ColumnArgs::default()
                                .modifier(Modifier::new().fill_max_size().padding_all(Dp(20.0)))
                                .cross_axis_alignment(CrossAxisAlignment::Start),
                            |scope| {
                                scope.child(|| {
                                    text(TextArgs::default().text("Menus Showcase").size(Dp(20.0)));
                                });

                                scope.child(move || {
                                    text(
                                        TextArgs::default()
                                            .text(format!(
                                                "Selected: {} | Pinned: {}",
                                                selected_label.get(),
                                                if pinned.get() { "Yes" } else { "No" }
                                            ))
                                            .size(Dp(14.0))
                                            .color(
                                                use_context::<MaterialTheme>()
                                                    .expect("MaterialTheme must be provided")
                                                    .get()
                                                    .color_scheme
                                                    .on_surface_variant,
                                            ),
                                    );
                                });

                                scope.child(|| {
                                    spacer(Modifier::new().height(Dp(12.0)));
                                });

                                scope.child(move || {
                                    row(
                                        RowArgs::default()
                                            .modifier(Modifier::new().fill_max_width()),
                                        |row_scope| {
                                            row_scope.child(move || {
                                                button(
                                                    ButtonArgs::default()
                                                        .modifier(Modifier::new().width(Dp(180.0)))
                                                        .on_click(move || {
                                                            menu_controller
                                                                .with_mut(|c| c.open_at(anchor));
                                                        }),
                                                    || {
                                                        text("Open anchored menu");
                                                    },
                                                );
                                            });

                                            row_scope.child(|| {
                                                spacer(Modifier::new().width(Dp(12.0)));
                                            });

                                            row_scope.child(|| {
                                                text(
                                                    TextArgs::default()
                                                        .text("Click to open at the button's anchor point.")
                                                        .size(Dp(14.0))
                                                        .color(
                                                            use_context::<MaterialTheme>()
                                                                .expect("MaterialTheme must be provided")
                                                                .get()
                                                                .color_scheme
                                                                .on_surface_variant,
                                                        ),
                                                );
                                            });
                                        },
                                    );
                                });
                            },
                        );
                    }
                },
                move |menu_scope| {
                    menu_scope.menu_item(MenuItemArgs::default().label("Revert").on_click(
                        move || {
                            selected_label.set("Revert".to_string());
                        },
                    ));

                    menu_scope.menu_item(MenuItemArgs::default().label("Settings").on_click(
                        move || {
                            selected_label.set("Settings".to_string());
                        },
                    ));

                    menu_scope.menu_item(
                        MenuItemArgs::default()
                            .label("Send Feedback")
                            .selected(pinned.get())
                            .on_click(move || {
                                let flag = pinned.with_mut(|p| {
                                    *p = !*p;
                                    *p
                                });
                                selected_label.set(if flag {
                                    "Send Feedback".to_string()
                                } else {
                                    "Unpinned".to_string()
                                });
                            }),
                    );

                    menu_scope.menu_item(MenuItemArgs::default().label("Help"));
                },
            );
        },
    );
}
