use tessera_components::{
    column::{ColumnArgs, column},
    divider::{DividerArgs, horizontal_divider, vertical_divider},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, shard, tessera};
#[tessera]
#[shard]
pub fn divider_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(&TextArgs::default().text("Divider Showcase").size(Dp(20.0)));
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Horizontal (default)")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        horizontal_divider(&DividerArgs::default());
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Horizontal (hairline)")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        horizontal_divider(&DividerArgs::default().thickness(Dp::ZERO));
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(24.0))));
                    });

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Vertical (fixed row height)")
                                .size(Dp(14.0)),
                        );
                    });

                    scope.child(|| {
                        row(
                            RowArgs::default()
                                .modifier(Modifier::new().fill_max_width().height(Dp(56.0))),
                            |scope| {
                                scope.child(|| {
                                    text(&TextArgs::from("Left"));
                                });

                                scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))));
                                });

                                scope.child(|| {
                                    vertical_divider(&DividerArgs::default());
                                });

                                scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))));
                                });

                                scope.child(|| {
                                    text(&TextArgs::from("Right"));
                                });
                            },
                        );
                    });
                },
            );
        },
    ));
}
