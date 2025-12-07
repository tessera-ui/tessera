use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
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
    let selected_index = remember(|| AtomicUsize::new(0));
    let radio_a = remember(|| RadioButtonController::new(true));
    let radio_b = remember(|| RadioButtonController::new(false));
    let radio_c = remember(|| RadioButtonController::new(false));
    let disabled_selected = remember(|| RadioButtonController::new(true));
    let disabled_unselected = remember(|| RadioButtonController::new(false));

    let select = Arc::new({
        let selected_index = selected_index.clone();
        let radio_a = radio_a.clone();
        let radio_b = radio_b.clone();
        let radio_c = radio_c.clone();
        move |index: usize| {
            selected_index.store(index, Ordering::SeqCst);
            radio_a.set_selected(index == 0);
            radio_b.set_selected(index == 1);
            radio_c.set_selected(index == 2);
        }
    });

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        {
            let selected_index = selected_index.clone();
            let radio_a = radio_a.clone();
            let radio_b = radio_b.clone();
            let radio_c = radio_c.clone();
            let disabled_selected = disabled_selected.clone();
            let disabled_unselected = disabled_unselected.clone();
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

                let selected = selected_index.load(Ordering::Acquire);
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
                    let radio_a = radio_a.clone();
                    move || {
                        option_row(
                            "Cat".to_string(),
                            radio_a.clone(),
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
                    let radio_b = radio_b.clone();
                    move || {
                        option_row(
                            "Dog".to_string(),
                            radio_b.clone(),
                            selected == 1,
                            move |_| select(1),
                            true,
                        );
                    }
                });

                scope.child({
                    let radio_c = radio_c.clone();
                    let select = select.clone();
                    move || {
                        option_row(
                            "Red Panda".to_string(),
                            radio_c.clone(),
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
                    let disabled_selected = disabled_selected.clone();
                    move || {
                        option_row(
                            "Selected (disabled)".to_string(),
                            disabled_selected.clone(),
                            true,
                            |_| {},
                            false,
                        );
                    }
                });

                scope.child({
                    let disabled_unselected = disabled_unselected.clone();
                    move || {
                        option_row(
                            "Unselected (disabled)".to_string(),
                            disabled_unselected.clone(),
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
    controller: Arc<RadioButtonController>,
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
                let controller = controller.clone();
                move || {
                    radio_button_with_controller(
                        RadioButtonArgsBuilder::default()
                            .on_select(on_select)
                            .enabled(enabled)
                            .build()
                            .unwrap(),
                        controller.clone(),
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
