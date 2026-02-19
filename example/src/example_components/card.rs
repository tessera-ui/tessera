use tessera_components::{
    card::{CardArgs, CardDefaults, CardVariant, card},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, shard, tessera};
#[tessera]
#[shard]
pub fn card_showcase() {
    card_showcase_node();
}

#[tessera]
fn card_showcase_node() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(&TextArgs::default().text("Card Showcase").size(Dp(20.0)));
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(&TextArgs::default().text("Filled").size(Dp(14.0)));
                    });

                    scope.child(|| {
                        card(CardArgs::filled(), |card_scope| {
                            card_scope.child(|| {
                                surface(&SurfaceArgs::with_child(
                                    SurfaceArgs::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0))),
                                    || {
                                        text(&TextArgs::from("Filled card body"));
                                    },
                                ));
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(&TextArgs::default().text("Elevated").size(Dp(14.0)));
                    });

                    scope.child(|| {
                        card(CardArgs::elevated(), |card_scope| {
                            card_scope.child(|| {
                                surface(&SurfaceArgs::with_child(
                                    SurfaceArgs::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0))),
                                    || {
                                        text(&TextArgs::from("Elevated card body"));
                                    },
                                ));
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(&TextArgs::default().text("Outlined").size(Dp(14.0)));
                    });

                    scope.child(|| {
                        card(CardArgs::outlined(), |card_scope| {
                            card_scope.child(|| {
                                surface(&SurfaceArgs::with_child(
                                    SurfaceArgs::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0))),
                                    || {
                                        text(&TextArgs::from("Outlined card body"));
                                    },
                                ));
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Clickable (outlined)")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        card(
                            CardArgs::default()
                                .variant(CardVariant::Outlined)
                                .border(CardDefaults::outlined_card_border(true))
                                .on_click(|| {}),
                            |card_scope| {
                                card_scope.child(|| {
                                    surface(&SurfaceArgs::with_child(
                                        SurfaceArgs::default()
                                            .style(tessera_ui::Color::TRANSPARENT.into())
                                            .modifier(Modifier::new().padding_all(Dp(16.0))),
                                        || {
                                            text(&TextArgs::from("Tap me"));
                                        },
                                    ));
                                });
                            },
                        );
                    });
                },
            );
        },
    ));
}
