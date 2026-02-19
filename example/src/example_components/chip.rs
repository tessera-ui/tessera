use tessera_components::{
    alignment::CrossAxisAlignment,
    chip::{ChipArgs, ChipStyle, chip},
    column::{ColumnArgs, column},
    flow_row::{FlowRowArgs, flow_row},
    icon::IconArgs,
    material_icons::filled,
    modifier::ModifierExt as _,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard, tessera};
#[tessera]
#[shard]
pub fn chip_showcase() {
    let favorites_selected = remember(|| false);
    let recent_selected = remember(|| true);
    let input_selected = remember(|| true);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                move |scope| {
                    scope.child(|| {
                        text(&TextArgs::default().text("Chip Showcase").size(Dp(20.0)));
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Assist and Suggestion")
                                .size(Dp(16.0)),
                        );
                    });

                    scope.child(|| {
                        flow_row(
                            FlowRowArgs::default()
                                .item_spacing(Dp(8.0))
                                .line_spacing(Dp(8.0))
                                .cross_axis_alignment(CrossAxisAlignment::Center),
                            |row_scope| {
                                row_scope.child(|| {
                                    chip(
                                        &ChipArgs::assist("Calendar")
                                            .leading_icon(IconArgs::from(filled::info_icon()))
                                            .on_click(|| {}),
                                    );
                                });
                                row_scope.child(|| {
                                    chip(
                                        &ChipArgs::suggestion("Road Trip")
                                            .leading_icon(IconArgs::from(
                                                filled::directions_car_icon(),
                                            ))
                                            .style(ChipStyle::Elevated)
                                            .on_click(|| {}),
                                    );
                                });
                            },
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                    scope.child(|| {
                        text(&TextArgs::default().text("Filter Chips").size(Dp(16.0)));
                    });

                    scope.child(move || {
                        let favorites_selected = favorites_selected;
                        let recent_selected = recent_selected;
                        flow_row(
                            FlowRowArgs::default()
                                .item_spacing(Dp(8.0))
                                .line_spacing(Dp(8.0))
                                .cross_axis_alignment(CrossAxisAlignment::Center),
                            move |row_scope| {
                                row_scope.child(move || {
                                    let selected = favorites_selected.with(|value| *value);
                                    chip(
                                        &ChipArgs::filter("Favorites")
                                            .selected(selected)
                                            .leading_icon(IconArgs::from(filled::home_icon()))
                                            .on_click(move || {
                                                favorites_selected.with_mut(|value| {
                                                    *value = !*value;
                                                });
                                            }),
                                    );
                                });

                                row_scope.child(move || {
                                    let selected = recent_selected.with(|value| *value);
                                    chip(
                                        &ChipArgs::filter("Recent")
                                            .style(ChipStyle::Elevated)
                                            .selected(selected)
                                            .leading_icon(IconArgs::from(filled::info_icon()))
                                            .on_click(move || {
                                                recent_selected.with_mut(|value| {
                                                    *value = !*value;
                                                });
                                            }),
                                    );
                                });
                            },
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                    scope.child(|| {
                        text(&TextArgs::default().text("Input Chips").size(Dp(16.0)));
                    });

                    scope.child(move || {
                        let input_selected = input_selected;
                        flow_row(
                            FlowRowArgs::default()
                                .item_spacing(Dp(8.0))
                                .line_spacing(Dp(8.0))
                                .cross_axis_alignment(CrossAxisAlignment::Center),
                            move |row_scope| {
                                row_scope.child(move || {
                                    let selected = input_selected.with(|value| *value);
                                    chip(
                                        &ChipArgs::input("Budget")
                                            .selected(selected)
                                            .leading_icon(IconArgs::from(filled::home_icon()))
                                            .trailing_icon(IconArgs::from(filled::info_icon()))
                                            .on_click(move || {
                                                input_selected.with_mut(|value| {
                                                    *value = !*value;
                                                });
                                            }),
                                    );
                                });
                            },
                        );
                    });
                },
            );
        },
    ));
}
