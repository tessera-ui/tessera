use std::sync::Arc;

use tessera_ui::{Dp, Modifier, State, remember, retain, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt as _,
    radio_button::{RadioButtonArgs, RadioButtonController, radio_button_with_controller},
    row::{RowArgs, row},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn radio_button_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        content,
    );
}

#[tessera]
fn content() {
    let selected_index = remember(|| 0usize);
    let radio_a = remember(|| RadioButtonController::new(true));
    let radio_b = remember(|| RadioButtonController::new(false));
    let radio_c = remember(|| RadioButtonController::new(false));
    let disabled_selected = remember(|| RadioButtonController::new(true));
    let disabled_unselected = remember(|| RadioButtonController::new(false));

    selected_index.with(move |&index| {
        radio_a.with_mut(|c| c.set_selected(index == 0));
        radio_b.with_mut(|c| c.set_selected(index == 1));
        radio_c.with_mut(|c| c.set_selected(index == 2));
    });

    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0)),
        controller,
        move |scope| {
            scope.item(|| {
                text(
                    TextArgs::default()
                        .text("Radio Button Showcase")
                        .size(Dp(20.0)),
                )
            });

            let selected = selected_index.get();
            scope.item(|| {
                text(
                    TextArgs::default()
                        .text("Pick a favorite animal:")
                        .size(Dp(16.0)),
                );
            });

            scope.item(move || {
                option_row(
                    "Cat".to_string(),
                    radio_a,
                    selected == 0,
                    move |_| selected_index.set(0),
                    true,
                );
            });

            scope.item(move || {
                option_row(
                    "Dog".to_string(),
                    radio_b,
                    selected == 1,
                    move |_| selected_index.set(1),
                    true,
                );
            });

            scope.item(move || {
                option_row(
                    "Red Panda".to_string(),
                    radio_c,
                    selected == 2,
                    move |_| selected_index.set(2),
                    true,
                );
            });

            let selected_label = match selected {
                0 => "Cat",
                1 => "Dog",
                _ => "Red Panda",
            };
            scope.item(move || {
                text(
                    TextArgs::default()
                        .text(format!("Selected: {}", selected_label))
                        .size(Dp(14.0)),
                );
            });

            scope.item(|| {
                text(TextArgs::default().text("Disabled states").size(Dp(16.0)));
            });

            scope.item({
                move || {
                    option_row(
                        "Selected (disabled)".to_string(),
                        disabled_selected,
                        true,
                        |_| {},
                        false,
                    );
                }
            });

            scope.item({
                move || {
                    option_row(
                        "Unselected (disabled)".to_string(),
                        disabled_unselected,
                        false,
                        |_| {},
                        false,
                    );
                }
            });
        },
    );
}

fn option_row(
    label: String,
    controller: State<RadioButtonController>,
    is_selected: bool,
    on_select: impl Fn(bool) + Clone + Send + Sync + 'static,
    enabled: bool,
) {
    row(
        RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
        move |scope| {
            let on_select = Arc::new(on_select);
            scope.child({
                let on_select = on_select.clone();
                move || {
                    radio_button_with_controller(
                        RadioButtonArgs::default()
                            .on_select_shared(on_select)
                            .enabled(enabled),
                        controller,
                    );
                }
            });
            scope.child(move || {
                let status = if is_selected { "(selected)" } else { "" };
                text(format!("{label} {status}").trim().to_string());
            });
        },
    );
}
