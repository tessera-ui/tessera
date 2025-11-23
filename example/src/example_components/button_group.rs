use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    button_group::{ButtonGroupArgsBuilder, ButtonGroupItem, ButtonGroupState, button_group},
    column::{ColumnArgsBuilder, column},
    md3_color::global_md3_scheme,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

struct ButtonGroupShowcaseState {
    scrollable_state: ScrollableState,
    single_state: ButtonGroupState,
    multi_state: ButtonGroupState,
}

impl ButtonGroupShowcaseState {
    fn new() -> Self {
        Self {
            scrollable_state: Default::default(),
            single_state: ButtonGroupState::single_with_initial(Some(0), false),
            multi_state: ButtonGroupState::multiple([0, 2]),
        }
    }
}

impl Default for ButtonGroupShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

#[tessera]
#[shard]
pub fn button_group_showcase(#[state] state: ButtonGroupShowcaseState) {
    let scrollable_state = state.scrollable_state.clone();
    let single_state = state.single_state.clone();
    let multi_state = state.multi_state.clone();

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .expect("builder construction failed"),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .expect("builder construction failed"),
                scrollable_state,
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(24.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .expect("builder construction failed"),
                        None,
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .width(DimensionValue::FILLED)
                                    .build()
                                    .expect("builder construction failed"),
                                |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Button Group (MD3 Segmented)".to_string())
                                                .size(Dp(20.0))
                                                .build()
                                                .expect("builder construction failed"),
                                        );
                                    });

                                    scope.child(|| spacer(SpacerArgs::from(Dp(16.0))));

                                    let single_for_card = single_state.clone();
                                    let multi_for_card = multi_state.clone();
                                    scope.child(move || {
                                        surface(
                                            SurfaceArgsBuilder::default()
                                                .style(global_md3_scheme().surface_variant.into())
                                                .padding(Dp(16.0))
                                                .width(DimensionValue::FILLED)
                                                .build()
                                                .expect("builder construction failed"),
                                            None,
                                            move || {
                                                column(
                                                    ColumnArgsBuilder::default()
                                                        .width(DimensionValue::FILLED)
                                                        .build()
                                                        .expect("builder construction failed"),
                                                    |scope| {
                                                        scope.child(|| {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Single (no deselect)".to_string())
                                                                    .size(Dp(16.0))
                                                                    .color(global_md3_scheme().on_surface)
                                                                    .build()
                                                                    .expect(
                                                                        "builder construction failed",
                                                                    ),
                                                            );
                                                        });
                                                        scope.child(|| spacer(SpacerArgs::from(Dp(
                                                            8.0,
                                                        ))));
                                                        let group_state = single_for_card.clone();
                                                        scope.child(move || {
                                                            let args =
                                                                ButtonGroupArgsBuilder::default()
                                                                    .allow_deselect_single(false)
                                                                    .build()
                                                                    .expect(
                                                                        "builder construction failed",
                                                                    );
                                                            button_group(args, group_state, |s| {
                                                                s.item(ButtonGroupItem::text(
                                                                    "Day",
                                                                ));
                                                                s.item(ButtonGroupItem::text(
                                                                    "Week",
                                                                ));
                                                                s.item(ButtonGroupItem::text(
                                                                    "Month",
                                                                ));
                                                            });
                                                        });
                                                    },
                                                );
                                            },
                                        );
                                    });

                                    scope.child(|| spacer(SpacerArgs::from(Dp(16.0))));

                                    scope.child(move || {
                                        surface(
                                            SurfaceArgsBuilder::default()
                                                .style(global_md3_scheme().surface_variant.into())
                                                .padding(Dp(16.0))
                                                .width(DimensionValue::FILLED)
                                                .build()
                                                .expect("builder construction failed"),
                                            None,
                                            move || {
                                                column(
                                                    ColumnArgsBuilder::default()
                                                        .width(DimensionValue::FILLED)
                                                        .build()
                                                        .expect("builder construction failed"),
                                                    |scope| {
                                                        scope.child(|| {
                                                            text(
                                                                TextArgsBuilder::default()
                                                                    .text("Multi-select".to_string())
                                                                    .size(Dp(16.0))
                                                                    .color(global_md3_scheme().on_surface)
                                                                    .build()
                                                                    .expect(
                                                                        "builder construction failed",
                                                                    ),
                                                            );
                                                        });
                                                        scope.child(|| spacer(SpacerArgs::from(Dp(
                                                            8.0,
                                                        ))));
                                                        let group_state = multi_for_card.clone();
                                                        scope.child(move || {
                                                            let args =
                                                                ButtonGroupArgsBuilder::default()
                                                                    .build()
                                                                    .expect(
                                                                        "builder construction failed",
                                                                    );
                                                            button_group(args, group_state, |s| {
                                                                s.item(ButtonGroupItem::text(
                                                                    "Flights",
                                                                ));
                                                                s.item(ButtonGroupItem::text(
                                                                    "Hotels",
                                                                ));
                                                                s.item(ButtonGroupItem::text(
                                                                    "Cars",
                                                                ));
                                                            });
                                                        });
                                                    },
                                                );
                                            },
                                        );
                                    });

                                    scope.child(|| spacer(SpacerArgs::from(Dp(24.0))));

                                    let single_for_summary = single_state.clone();
                                    let multi_for_summary = multi_state.clone();
                                    scope.child(move || {
                                        row(
                                            RowArgsBuilder::default()
                                                .width(DimensionValue::FILLED)
                                                .build()
                                                .expect("builder construction failed"),
                                            |row_scope| {
                                                let single = single_for_summary.clone();
                                                let multi = multi_for_summary.clone();
                                                row_scope.child(move || {
                                                    text(
                                                        TextArgsBuilder::default()
                                                            .text(format!(
                                                                "Single selection: {:?}",
                                                                single.selected_indices()
                                                            ))
                                                            .size(Dp(14.0))
                                                            .color(
                                                                global_md3_scheme()
                                                                    .on_surface_variant,
                                                            )
                                                            .build()
                                                            .expect("builder construction failed"),
                                                    );
                                                });
                                                row_scope
                                                    .child(|| spacer(SpacerArgs::from(Dp(12.0))));
                                                row_scope.child(move || {
                                                    text(
                                                        TextArgsBuilder::default()
                                                            .text(format!(
                                                                "Multi selection: {:?}",
                                                                multi.selected_indices()
                                                            ))
                                                            .size(Dp(14.0))
                                                            .color(
                                                                global_md3_scheme()
                                                                    .on_surface_variant,
                                                            )
                                                            .build()
                                                            .expect("builder construction failed"),
                                                    );
                                                });
                                            },
                                        );
                                    });
                                },
                            );
                        },
                    );
                },
            );
        },
    );
}
