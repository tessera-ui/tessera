use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    tabs::{TabsArgsBuilder, TabsState, tabs},
    text::{TextArgsBuilder, text},
};

#[derive(Default)]
struct TabsShowcaseState {
    scrollable_state: ScrollableState,
    tabs_state: TabsState,
}

#[tessera]
#[shard]
pub fn tabs_showcase(#[state] state: TabsShowcaseState) {
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
                        move || {
                            test_content(state);
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content(state: Arc<TabsShowcaseState>) {
    let tabs_state = state.tabs_state.clone();
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
                    tabs_state,
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
