use std::sync::Arc;

use tessera_ui::{Dp, Modifier, State, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    radio_button::{RadioButtonArgsBuilder, RadioButtonController, radio_button_with_controller},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn radio_button_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        move || {
                            content();
                        },
                    );
                },
            )
        },
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

    let select = Arc::new({
        move |index: usize| {
            selected_index.set(index);
            radio_a.with_mut(|c| c.set_selected(index == 0));
            radio_b.with_mut(|c| c.set_selected(index == 1));
            radio_c.with_mut(|c| c.set_selected(index == 2));
        }
    });

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        {
            let select = select.clone();
            move |scope| {
                scope.child(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Radio Button Showcase")
                            .size(Dp(20.0))
                            .build()
                            .unwrap(),
                    )
                });

                let selected = selected_index.get();
                scope.child(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Pick a favorite animal:")
                            .size(Dp(16.0))
                            .build()
                            .unwrap(),
                    );
                });

                scope.child({
                    let select = select.clone();
                    move || {
                        option_row(
                            "Cat".to_string(),
                            radio_a,
                            selected == 0,
                            {
                                let select = select.clone();
                                move |_| select(0)
                            },
                            true,
                        );
                    }
                });

                scope.child({
                    let select = select.clone();
                    move || {
                        option_row(
                            "Dog".to_string(),
                            radio_b,
                            selected == 1,
                            move |_| select(1),
                            true,
                        );
                    }
                });

                scope.child({
                    let select = select.clone();
                    move || {
                        option_row(
                            "Red Panda".to_string(),
                            radio_c,
                            selected == 2,
                            move |_| select(2),
                            true,
                        );
                    }
                });

                let selected_label = match selected {
                    0 => "Cat",
                    1 => "Dog",
                    _ => "Red Panda",
                };
                scope.child(move || {
                    text(
                        TextArgsBuilder::default()
                            .text(format!("Selected: {}", selected_label))
                            .size(Dp(14.0))
                            .build()
                            .unwrap(),
                    );
                });

                scope.child(|| {
                    text(
                        TextArgsBuilder::default()
                            .text("Disabled states")
                            .size(Dp(16.0))
                            .build()
                            .unwrap(),
                    );
                });

                scope.child({
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

                scope.child({
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
            }
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
        RowArgsBuilder::default()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .build()
            .unwrap(),
        move |scope| {
            let on_select = Arc::new(on_select);
            scope.child({
                let on_select = on_select.clone();
                move || {
                    radio_button_with_controller(
                        RadioButtonArgsBuilder::default()
                            .on_select(on_select)
                            .enabled(enabled)
                            .build()
                            .unwrap(),
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
