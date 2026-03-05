use tessera_components::{
    badge::{BadgeArgs, BadgedBoxArgs, badge, badge_with_content, badged_box},
    column::{ColumnArgs, column},
    icon::{IconArgs, icon},
    material_icons::filled,
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, shard};
#[shard]
pub fn badge_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                &ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0)))
                    .children(|scope| {
                        scope.child(|| {
                            text(&TextArgs::default().text("Badge Showcase").size(Dp(20.0)));
                        });

                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                        });

                        scope.child(|| {
                            text(&TextArgs::default().text("Dot badge").size(Dp(14.0)));
                        });

                        scope.child(|| {
                            row(&RowArgs::default()
                                .modifier(Modifier::new().fill_max_width())
                                .children(|row_scope| {
                                    row_scope.child(|| {
                                        let icon_content = filled::HOME_SVG;
                                        badged_box(
                                            &BadgedBoxArgs::default()
                                                .badge(|| {
                                                    badge(&BadgeArgs::default());
                                                })
                                                .content(move || {
                                                    icon(&IconArgs::default().vector(icon_content));
                                                }),
                                        );
                                    });
                                }));
                        });

                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                        });

                        scope.child(|| {
                            text(
                                &TextArgs::default()
                                    .text("Badge with content")
                                    .size(Dp(14.0)),
                            );
                        });

                        scope.child(|| {
                            row(&RowArgs::default()
                                .modifier(Modifier::new().fill_max_width())
                                .children(|row_scope| {
                                    row_scope.child(|| {
                                        let icon_content = filled::HOME_SVG;
                                        badged_box(
                                            &BadgedBoxArgs::default()
                                                .badge(|| {
                                                    badge_with_content(
                                                        &BadgeArgs::default().content(|| {
                                                            text(
                                                                &TextArgs::default()
                                                                    .text("12".to_string()),
                                                            );
                                                        }),
                                                    );
                                                })
                                                .content(move || {
                                                    icon(&IconArgs::default().vector(icon_content));
                                                }),
                                        );
                                    });
                                }));
                        });
                    }),
            );
        },
    ));
}
