use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    badge::{BadgeArgs, badge, badge_with_content, badged_box},
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    material_icons::filled,
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn badge_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(TextArgs::default().text("Badge Showcase").size(Dp(20.0)));
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(TextArgs::default().text("Dot badge").size(Dp(14.0)));
                    });

                    scope.child(|| {
                        row(
                            RowArgs::default().modifier(Modifier::new().fill_max_width()),
                            |row_scope| {
                                row_scope.child(|| {
                                    let icon_content = filled::home_icon();
                                    badged_box(
                                        || {
                                            badge(BadgeArgs::default());
                                        },
                                        || {
                                            icon(IconArgs::from(icon_content));
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
                            TextArgs::default()
                                .text("Badge with content")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        row(
                            RowArgs::default().modifier(Modifier::new().fill_max_width()),
                            |row_scope| {
                                row_scope.child(|| {
                                    let icon_content = filled::home_icon();
                                    badged_box(
                                        || {
                                            badge_with_content(
                                                BadgeArgs::default(),
                                                |badge_scope| {
                                                    badge_scope.child(|| {
                                                        text(
                                                            TextArgs::default()
                                                                .text("12".to_string()),
                                                        );
                                                    });
                                                },
                                            );
                                        },
                                        || {
                                            icon(IconArgs::from(icon_content));
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
