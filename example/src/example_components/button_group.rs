use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    button_groups::{ButtonGroupsArgs, ButtonGroupsStyle, button_groups},
    lazy_list::{LazyColumnArgs, lazy_column},
    modifier::ModifierExt as _,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn button_group_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .expect("builder construction failed"),
        move || {
            lazy_column(
                LazyColumnArgs {
                    content_padding: Dp::new(16.0),
                    ..Default::default()
                },
                move |scope| {
                    scope.item(move || {
                        text("Button Groups");
                    });

                    scope.item(move || {
                        button_groups(ButtonGroupsArgs::default(), |scope| {
                            scope.child(
                                |color| {
                                    text(TextArgs {
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
                                    text(TextArgs {
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
                                    text(TextArgs {
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

                    scope.item(|| spacer(Modifier::new().height(Dp(5.0))));

                    scope.item(move || {
                        button_groups(
                            ButtonGroupsArgs {
                                style: ButtonGroupsStyle::Connected,
                                ..Default::default()
                            },
                            |scope| {
                                scope.child(
                                    |color| {
                                        text(TextArgs {
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
                                        text(TextArgs {
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
                                        text(TextArgs {
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
                },
            );
        },
    );
}
