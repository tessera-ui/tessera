use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    menus::{
        MenuAnchor, MenuController, MenuItemArgsBuilder, MenuPlacement, MenuProviderArgsBuilder,
        menu_provider_with_controller,
    },
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
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
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .expect("builder construction failed"),
        move || {
            menu_provider_with_controller(
                MenuProviderArgsBuilder::default()
                    .placement(MenuPlacement::BelowStart)
                    .offset([Dp(0.0), Dp(6.0)])
                    .build()
                    .expect("builder construction failed"),
                menu_controller,
                {
                    move || {
                        column(
                            ColumnArgsBuilder::default()
                                .modifier(Modifier::new().fill_max_size().padding_all(Dp(20.0)))
                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                .build()
                                .expect("builder construction failed"),
                            |scope| {
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text("Menus Showcase")
                                            .size(Dp(20.0))
                                            .build()
                                            .expect("builder construction failed"),
                                    );
                                });

                                scope.child(move || {
                                    text(
                                        TextArgsBuilder::default()
                                            .text(format!(
                                                "Selected: {} | Pinned: {}",
                                                selected_label.get(),
                                                if pinned.get() { "Yes" } else { "No" }
                                            ))
                                            .size(Dp(14.0))
                                            .color(
                                                use_context::<MaterialTheme>()
                                                    .get()
                                                    .color_scheme
                                                    .on_surface_variant,
                                            )
                                            .build()
                                            .expect("builder construction failed"),
                                    );
                                });

                                scope.child(|| {
                                    spacer(Modifier::new().height(Dp(12.0)));
                                });

                                scope.child(move || {
                                    row(
                                        RowArgsBuilder::default()
                                            .modifier(Modifier::new().fill_max_width())
                                            .build()
                                            .expect("builder construction failed"),
                                        |row_scope| {
                                            row_scope.child(move || {
                                                button(
                                                    ButtonArgsBuilder::default()
                                                        .modifier(Modifier::new().width(Dp(180.0)))
                                                        .on_click(move || {
                                                            menu_controller
                                                                .with_mut(|c| c.open_at(anchor));
                                                        })
                                                        .build()
                                                        .expect("builder construction failed"),
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
                                            TextArgsBuilder::default()
                                                .text(
                                                    "Click to open at the button's anchor point.",
                                                )
                                                .size(Dp(14.0))
                                                .color(
                                                    use_context::<MaterialTheme>()
                                                        .get()
                                                        .color_scheme
                                                        .on_surface_variant,
                                                )
                                                .build()
                                                .expect("builder construction failed"),
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
                    menu_scope.menu_item(
                        MenuItemArgsBuilder::default()
                            .label("Revert")
                            .on_click(move || {
                                selected_label.set("Revert".to_string());
                            })
                            .build()
                            .expect("builder construction failed"),
                    );

                    menu_scope.menu_item(
                        MenuItemArgsBuilder::default()
                            .label("Settings")
                            .on_click(move || {
                                selected_label.set("Settings".to_string());
                            })
                            .build()
                            .expect("builder construction failed"),
                    );

                    menu_scope.menu_item(
                        MenuItemArgsBuilder::default()
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
                            })
                            .build()
                            .expect("builder construction failed"),
                    );

                    menu_scope.menu_item(
                        MenuItemArgsBuilder::default()
                            .label("Help")
                            .build()
                            .expect("builder construction failed"),
                    );
                },
            );
        },
    );
}
