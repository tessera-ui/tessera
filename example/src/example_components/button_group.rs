use tessera_components::{
    button_groups::{ButtonGroupsArgs, ButtonGroupsStyle, button_groups},
    lazy_list::{LazyColumnArgs, lazy_column},
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, shard, tessera};
#[shard]
pub fn button_group_showcase() {
    button_group_showcase_node();
}

#[tessera]
fn button_group_showcase_node() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                &LazyColumnArgs {
                    content_padding: Dp::new(16.0),
                    ..Default::default()
                }
                .content(move |scope| {
                    scope.item(move || {
                        text(&TextArgs::from("Button Groups"));
                    });

                    scope.item(move || {
                        button_groups(&ButtonGroupsArgs::default(), |scope| {
                            scope.child(
                                |color| {
                                    text(&TextArgs {
                                        text: "Button 1".to_string(),
                                        color,
                                        ..Default::default()
                                    })
                                },
                                |_| {
                                    println!("Button 1 clicked");
                                },
                            );

                            scope.child(
                                |color| {
                                    text(&TextArgs {
                                        text: "Button 2".to_string(),
                                        color,
                                        ..Default::default()
                                    })
                                },
                                |_actived| {
                                    println!("Button 2 clicked");
                                },
                            );

                            scope.child(
                                |color| {
                                    text(&TextArgs {
                                        text: "Button 3".to_string(),
                                        color,
                                        ..Default::default()
                                    })
                                },
                                |_actived| {
                                    println!("Button 3 clicked");
                                },
                            );
                        });
                    });

                    scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(5.0)))));

                    scope.item(move || {
                        button_groups(
                            &ButtonGroupsArgs {
                                style: ButtonGroupsStyle::Connected,
                                ..Default::default()
                            },
                            |scope| {
                                scope.child(
                                    |color| {
                                        text(&TextArgs {
                                            text: "Button 1".to_string(),
                                            color,
                                            ..Default::default()
                                        })
                                    },
                                    |_actived| {
                                        println!("Button 1 clicked");
                                    },
                                );

                                scope.child(
                                    |color| {
                                        text(&TextArgs {
                                            text: "Button 2".to_string(),
                                            color,
                                            ..Default::default()
                                        })
                                    },
                                    |_actived| {
                                        println!("Button 2 clicked");
                                    },
                                );

                                scope.child(
                                    |color| {
                                        text(&TextArgs {
                                            text: "Button 3".to_string(),
                                            color,
                                            ..Default::default()
                                        })
                                    },
                                    |_actived| {
                                        println!("Button 3 clicked");
                                    },
                                );
                            },
                        );
                    });
                }),
            );
        },
    ));
}
