use tessera_ui::{Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    divider::{DividerArgsBuilder, horizontal_divider, vertical_divider},
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn divider_showcase() {
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
                                .text("Divider Showcase")
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
                                .text("Horizontal (default)")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        horizontal_divider(DividerArgsBuilder::default().build().unwrap());
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(16.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Horizontal (hairline)")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        horizontal_divider(
                            DividerArgsBuilder::default()
                                .thickness(Dp::ZERO)
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        spacer(Modifier::new().height(Dp(24.0)));
                    });

                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text("Vertical (fixed row height)")
                                .size(Dp(14.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| {
                        row(
                            RowArgsBuilder::default()
                                .modifier(Modifier::new().fill_max_width().height(Dp(56.0)))
                                .build()
                                .unwrap(),
                            |scope| {
                                scope.child(|| {
                                    text("Left");
                                });

                                scope.child(|| {
                                    spacer(Modifier::new().width(Dp(12.0)));
                                });

                                scope.child(|| {
                                    vertical_divider(
                                        DividerArgsBuilder::default().build().unwrap(),
                                    );
                                });

                                scope.child(|| {
                                    spacer(Modifier::new().width(Dp(12.0)));
                                });

                                scope.child(|| {
                                    text("Right");
                                });
                            },
                        );
                    });
                },
            );
        },
    );
}
