use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    card::{CardArgs, CardArgsBuilder, CardDefaults, CardVariant, card},
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn card_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            column(
                ColumnArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0)))
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Card Showcase")
                                .size(Dp(20.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Filled")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        card(CardArgs::filled(), |card_scope| {
                            card_scope.child(|| {
                                surface(
                                    SurfaceArgsBuilder::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                                        .build()
                                        .unwrap(),
                                    || {
                                        text("Filled card body");
                                    },
                                );
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Elevated")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        card(CardArgs::elevated(), |card_scope| {
                            card_scope.child(|| {
                                surface(
                                    SurfaceArgsBuilder::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                                        .build()
                                        .unwrap(),
                                    || {
                                        text("Elevated card body");
                                    },
                                );
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Outlined")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        card(CardArgs::outlined(), |card_scope| {
                            card_scope.child(|| {
                                surface(
                                    SurfaceArgsBuilder::default()
                                        .style(tessera_ui::Color::TRANSPARENT.into())
                                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                                        .build()
                                        .unwrap(),
                                    || {
                                        text("Outlined card body");
                                    },
                                );
                            });
                        });
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Clickable (outlined)")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        card(
                            CardArgsBuilder::default()
                                .variant(CardVariant::Outlined)
                                .border(CardDefaults::outlined_card_border(true))
                                .on_click(|| {})
                                .build()
                                .unwrap(),
                            |card_scope| {
                                card_scope.child(|| {
                                    surface(
                                        SurfaceArgsBuilder::default()
                                            .style(tessera_ui::Color::TRANSPARENT.into())
                                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                                            .build()
                                            .unwrap(),
                                        || {
                                            text("Tap me");
                                        },
                                    );
                                });
                            },
                        );
                    });
                },
            );
        },
    );
}
