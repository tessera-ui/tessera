use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    tabs::{TabsArgsBuilder, tabs},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn tabs_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        move || {
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
                        .text("Tabs Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(move || {
                tabs(
                    TabsArgsBuilder::default()
                        .width(DimensionValue::FILLED)
                        .build()
                        .unwrap(),
                    |scope| {
                        scope.child_with_color(
                            |color| {
                                text(
                                    TextArgsBuilder::default()
                                        .text("Flights")
                                        .size(Dp(14.0))
                                        .color(color)
                                        .build()
                                        .unwrap(),
                                )
                            },
                            || text("Fly in the air..."),
                        );
                        scope.child_with_color(
                            |color| {
                                text(
                                    TextArgsBuilder::default()
                                        .text("Hotel")
                                        .size(Dp(14.0))
                                        .color(color)
                                        .build()
                                        .unwrap(),
                                )
                            },
                            || text("Sleep well..."),
                        );
                        scope.child_with_color(
                            |color| {
                                text(
                                    TextArgsBuilder::default()
                                        .text("Cars")
                                        .size(Dp(14.0))
                                        .color(color)
                                        .build()
                                        .unwrap(),
                                )
                            },
                            || text("Beep beep..."),
                        );
                    },
                );
            });
        },
    )
}
