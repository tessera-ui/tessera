use tessera_components::{
    column::{ColumnArgs, column},
    icon::IconArgs,
    material_icons::filled,
    modifier::ModifierExt as _,
    segmented_buttons::{
        SegmentedButtonArgs, SegmentedButtonDefaults, SegmentedButtonRowArgs,
        multi_choice_segmented_button_row, segmented_button, single_choice_segmented_button_row,
    },
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard};
#[shard]
pub fn segmented_buttons_showcase() {
    let selected_index = remember(|| 0usize);
    let selected_filters = remember(|| vec![true, false, false]);
    let base_shape = SegmentedButtonDefaults::shape();

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Segmented Buttons Showcase")
                                .size(Dp(20.0)),
                        )
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));

                    scope.child(|| text(&TextArgs::default().text("Single choice").size(Dp(14.0))));

                    scope.child(move || {
                        let options = ["Day", "Week", "Month"];
                        let count = options.len();
                        single_choice_segmented_button_row(
                            &SegmentedButtonRowArgs::default(),
                            move || {
                                for (index, label) in options.iter().enumerate() {
                                    let selected_index = selected_index;
                                    let label = (*label).to_string();
                                    segmented_button(
                                        &SegmentedButtonArgs::new(label)
                                            .selected(selected_index.get() == index)
                                            .shape(SegmentedButtonDefaults::item_shape(
                                                index, count, base_shape,
                                            ))
                                            .on_click(move || {
                                                selected_index.set(index);
                                            }),
                                    );
                                }
                            },
                        );
                    });

                    scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));

                    scope.child(|| text(&TextArgs::default().text("Multi choice").size(Dp(14.0))));

                    scope.child(move || {
                        let options = ["Inbox", "Alerts", "Settings"];
                        let count = options.len();
                        multi_choice_segmented_button_row(
                            &SegmentedButtonRowArgs::default(),
                            move || {
                                for (index, label) in options.iter().enumerate() {
                                    let icon_args = match index {
                                        0 => IconArgs::from(filled::inbox_icon()),
                                        1 => IconArgs::from(filled::notifications_icon()),
                                        _ => IconArgs::from(filled::settings_icon()),
                                    };
                                    let is_selected = selected_filters
                                        .with(|values| values.get(index).copied().unwrap_or(false));
                                    let selected_filters = selected_filters;
                                    let label = (*label).to_string();
                                    segmented_button(
                                        &SegmentedButtonArgs::new(label)
                                            .icon(icon_args)
                                            .selected(is_selected)
                                            .shape(SegmentedButtonDefaults::item_shape(
                                                index, count, base_shape,
                                            ))
                                            .on_click(move || {
                                                selected_filters.with_mut(|values| {
                                                    if let Some(value) = values.get_mut(index) {
                                                        *value = !*value;
                                                    }
                                                });
                                            }),
                                    );
                                }
                            },
                        );
                    });
                },
            );
        },
    ));
}
