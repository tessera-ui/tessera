use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    divider::{DividerArgsBuilder, horizontal_divider, vertical_divider},
    row::{RowArgsBuilder, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn divider_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .padding(Dp(16.0))
            .build()
            .unwrap(),
        move || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::FILLED)
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
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(Dp(16.0))
                                .build()
                                .unwrap(),
                        );
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
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(Dp(16.0))
                                .build()
                                .unwrap(),
                        );
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
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(Dp(24.0))
                                .build()
                                .unwrap(),
                        );
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
                                .width(DimensionValue::FILLED)
                                .height(DimensionValue::Fixed(Dp(56.0).to_px()))
                                .build()
                                .unwrap(),
                            |scope| {
                                scope.child(|| {
                                    text("Left");
                                });

                                scope.child(|| {
                                    spacer(
                                        SpacerArgsBuilder::default()
                                            .width(Dp(12.0))
                                            .build()
                                            .unwrap(),
                                    );
                                });

                                scope.child(|| {
                                    vertical_divider(
                                        DividerArgsBuilder::default().build().unwrap(),
                                    );
                                });

                                scope.child(|| {
                                    spacer(
                                        SpacerArgsBuilder::default()
                                            .width(Dp(12.0))
                                            .build()
                                            .unwrap(),
                                    );
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
