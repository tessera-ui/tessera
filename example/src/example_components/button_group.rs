use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    button_groups::{ButtonGroupsArgs, ButtonGroupsState, ButtonGroupsStyle, button_groups},
    lazy_list::{LazyColumnArgs, LazyListState, lazy_column},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgs, text},
};

#[derive(Default)]
struct ButtonGroupShowcaseState {
    lazy_list_state: LazyListState,
    button_groups_state: ButtonGroupsState,
    button_groups_state2: ButtonGroupsState,
}

#[tessera]
#[shard]
pub fn button_group_showcase(#[state] state: ButtonGroupShowcaseState) {
    let lazy_list_state = state.lazy_list_state.clone();
    let button_groups_state = state.button_groups_state.clone();
    let button_groups_state2 = state.button_groups_state2.clone();

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .expect("builder construction failed"),
        move || {
            lazy_column(
                LazyColumnArgs {
                    content_padding: Dp::new(16.0),
                    ..Default::default()
                },
                lazy_list_state,
                move |scope| {
                    scope.item(move || {
                        text("Button Groups");
                    });

                    scope.item(move || {
                        button_groups(
                            ButtonGroupsArgs::default(),
                            button_groups_state.clone(),
                            |scope| {
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
                            },
                        );
                    });

                    scope.item(|| {
                        spacer(SpacerArgs {
                            width: Dp(5.0).into(),
                            ..Default::default()
                        })
                    });

                    scope.item(move || {
                        button_groups(
                            ButtonGroupsArgs {
                                style: ButtonGroupsStyle::Connected,
                                ..Default::default()
                            },
                            button_groups_state2.clone(),
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
