use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    badge::{BadgeArgsBuilder, badge, badge_with_content, badged_box},
    column::{ColumnArgsBuilder, column},
    icon::{IconArgsBuilder, icon},
    material_icons::filled,
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn badge_showcase() {
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
                                .text("Badge Showcase")
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
                                .text("Dot badge")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        row(
                            RowArgsBuilder::default()
                                .modifier(Modifier::new().fill_max_width())
                                .build()
                                .unwrap(),
                            |row_scope| {
                                row_scope.child(|| {
                                    let icon_content = filled::home_icon();
                                    badged_box(
                                        || {
                                            badge(BadgeArgsBuilder::default().build().unwrap());
                                        },
                                        || {
                                            icon(
                                                IconArgsBuilder::default()
                                                    .content(icon_content)
                                                    .build()
                                                    .unwrap(),
                                            );
                                        },
                                    );
                                });
                            },
                        );
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Badge with content")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        row(
                            RowArgsBuilder::default()
                                .modifier(Modifier::new().fill_max_width())
                                .build()
                                .unwrap(),
                            |row_scope| {
                                row_scope.child(|| {
                                    let icon_content = filled::home_icon();
                                    badged_box(
                                        || {
                                            badge_with_content(
                                                BadgeArgsBuilder::default().build().unwrap(),
                                                |badge_scope| {
                                                    badge_scope.child(|| {
                                                        text(
                                                            TextArgsBuilder::default()
                                                                .text("12".to_string())
                                                                .build()
                                                                .unwrap(),
                                                        );
                                                    });
                                                },
                                            );
                                        },
                                        || {
                                            icon(
                                                IconArgsBuilder::default()
                                                    .content(icon_content)
                                                    .build()
                                                    .unwrap(),
                                            );
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
