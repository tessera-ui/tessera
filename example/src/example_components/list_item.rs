use tessera_components::{
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    list_item::{ListItemArgs, list_item},
    material_icons::filled,
    modifier::{ModifierExt, Padding},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard, tessera};
#[tessera]
#[shard]
pub fn list_item_showcase() {
    let selected = remember(|| false);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default().modifier(Modifier::new().fill_max_size()),
                |scope| {
                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("List Item Showcase")
                                .modifier(
                                    Modifier::new()
                                        .padding(Padding::left(Dp(16.0)))
                                        .padding(Padding::top(Dp(16.0))),
                                )
                                .size(Dp(20.0)),
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("One-line")
                                .modifier(Modifier::new().padding(Padding::left(Dp(16.0))))
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        list_item(
                            &ListItemArgs::new("Inbox")
                                .leading(|| {
                                    icon(&IconArgs::from(filled::inbox_icon()).size(Dp(24.0)));
                                })
                                .trailing(|| {
                                    icon(
                                        &IconArgs::from(filled::chevron_right_icon())
                                            .size(Dp(20.0)),
                                    );
                                }),
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Two-line (click to toggle selection)")
                                .modifier(Modifier::new().padding(Padding::left(Dp(16.0))))
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(move || {
                        let is_selected = selected.get();
                        let status = if is_selected { "On" } else { "Off" };
                        list_item(
                            &ListItemArgs::new("Notifications")
                                .supporting_text(format!("Alerts: {status}"))
                                .selected(is_selected)
                                .on_click(move || {
                                    selected.with_mut(|value| *value = !*value);
                                })
                                .leading(|| {
                                    icon(
                                        &IconArgs::from(filled::notifications_icon())
                                            .size(Dp(24.0)),
                                    );
                                })
                                .trailing(|| {
                                    icon(
                                        &IconArgs::from(filled::chevron_right_icon())
                                            .size(Dp(20.0)),
                                    );
                                }),
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Three-line")
                                .modifier(Modifier::new().padding(Padding::left(Dp(16.0))))
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        list_item(
                            &ListItemArgs::new("Security")
                                .overline_text("Account")
                                .supporting_text("Two-factor auth and recovery options")
                                .leading(|| {
                                    icon(&IconArgs::from(filled::settings_icon()).size(Dp(24.0)));
                                }),
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .modifier(Modifier::new().padding(Padding::left(Dp(16.0))))
                                .text("Disabled")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        list_item(
                            &ListItemArgs::new("Do not disturb")
                                .supporting_text("Disabled item")
                                .enabled(false)
                                .leading(|| {
                                    icon(
                                        &IconArgs::from(filled::notifications_icon())
                                            .size(Dp(24.0)),
                                    );
                                }),
                        );
                    });
                },
            );
        },
    ));
}
