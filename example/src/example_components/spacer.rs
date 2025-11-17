use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default, Clone)]
pub struct SpacerShowcaseState {
    scrollable_state: ScrollableState,
}

#[tessera]
#[shard]
pub fn spacer_showcase(#[state] state: SpacerShowcaseState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                state.scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.into())
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Spacer Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(|| text("Horizontal Spacer (in a Row):"));
            scope.child(|| {
                row(RowArgsBuilder::default().build().unwrap(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    });
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(Dp(40.0))
                                .build()
                                .unwrap(),
                        )
                    });
                    scope.child(|| colored_box(Color::BLUE));
                })
            });

            scope.child(|| text("Vertical Spacer (in a Column):"));
            scope.child(|| {
                column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(Dp(20.0))
                                .build()
                                .unwrap(),
                        )
                    });
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(Dp(40.0))
                                .build()
                                .unwrap(),
                        )
                    });
                    scope.child(|| colored_box(Color::BLUE));
                })
            });
        },
    )
}

#[tessera]
fn colored_box(color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .style(color.into())
            .width(Dp(50.0))
            .height(Dp(50.0))
            .build()
            .unwrap(),
        None,
        || {},
    );
}
