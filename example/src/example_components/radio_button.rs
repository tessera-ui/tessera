use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    radio_button::{RadioButtonArgsBuilder, RadioButtonState, radio_button},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct RadioButtonShowcaseState {
    selected_index: Arc<AtomicUsize>,
    radio_a: RadioButtonState,
    radio_b: RadioButtonState,
    radio_c: RadioButtonState,
    disabled_selected: RadioButtonState,
    disabled_unselected: RadioButtonState,
}

impl Default for RadioButtonShowcaseState {
    fn default() -> Self {
        let selected_index = Arc::new(AtomicUsize::new(0));
        Self {
            selected_index,
            radio_a: RadioButtonState::new(true),
            radio_b: RadioButtonState::new(false),
            radio_c: RadioButtonState::new(false),
            disabled_selected: RadioButtonState::new(true),
            disabled_unselected: RadioButtonState::new(false),
        }
    }
}

#[tessera]
#[shard]
pub fn radio_button_showcase(#[state] state: RadioButtonShowcaseState) {
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
                    let state = state.clone();
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        move || {
                            content(state.clone());
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn content(state: Arc<RadioButtonShowcaseState>) {
    let select = Arc::new({
        let state = state.clone();
        move |index: usize| {
            state.selected_index.store(index, Ordering::SeqCst);
            state.radio_a.set_selected(index == 0);
            state.radio_b.set_selected(index == 1);
            state.radio_c.set_selected(index == 2);
        }
    });

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        {
            let state = state.clone();
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

                let selected = state.selected_index.load(Ordering::Acquire);
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
                    let state = state.clone();
                    move || {
                        option_row(
                            "Cat".to_string(),
                            state.radio_a.clone(),
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
                    let state = state.clone();
                    move || {
                        option_row(
                            "Dog".to_string(),
                            state.radio_b.clone(),
                            selected == 1,
                            move |_| select(1),
                            true,
                        );
                    }
                });

                scope.child({
                    let state = state.clone();
                    let select = select.clone();
                    move || {
                        option_row(
                            "Red Panda".to_string(),
                            state.radio_c.clone(),
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
                    let state = state.clone();
                    move || {
                        option_row(
                            "Selected (disabled)".to_string(),
                            state.disabled_selected.clone(),
                            true,
                            |_| {},
                            false,
                        );
                    }
                });

                scope.child({
                    let state = state.clone();
                    move || {
                        option_row(
                            "Unselected (disabled)".to_string(),
                            state.disabled_unselected.clone(),
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
    radio_state: RadioButtonState,
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
                let radio_state = radio_state.clone();
                move || {
                    radio_button(
                        RadioButtonArgsBuilder::default()
                            .on_select(on_select)
                            .enabled(enabled)
                            .build()
                            .unwrap(),
                        radio_state.clone(),
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
