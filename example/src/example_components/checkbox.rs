use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
};

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    checkbox::{CheckboxArgsBuilder, checkbox},
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Default, Clone)]
struct CheckboxShowcaseState {
    scrollable_state: ScrollableState,
    is_checked: Arc<AtomicBool>,
}

#[tessera]
#[shard]
pub fn checkbox_showcase(#[state] state: CheckboxShowcaseState) {
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
                state.scrollable_state.clone(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        move || {
                            column(
                                ColumnArgsBuilder::default()
                                    .cross_axis_alignment(CrossAxisAlignment::Start)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("Checkbox Showcase")
                                                .size(Dp(20.0))
                                                .build()
                                                .unwrap(),
                                        )
                                    });

                                    // Interactive Checkbox
                                    let state_clone = state.clone();
                                    scope.child(move || {
                                        row(
                                            RowArgsBuilder::default()
                                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                                .build()
                                                .unwrap(),
                                            |scope| {
                                                let state = state_clone.clone();
                                                scope.child(move || {
                                                    let on_toggle = Arc::new({
                                                        move |new_value| {
                                                            state.is_checked.store(
                                                                new_value,
                                                                atomic::Ordering::SeqCst,
                                                            );
                                                        }
                                                    });
                                                    checkbox(
                                                        CheckboxArgsBuilder::default()
                                                            .checked(true)
                                                            .on_toggle(on_toggle)
                                                            .build()
                                                            .unwrap(),
                                                    );
                                                });
                                                let state = state_clone.clone();
                                                scope.child(move || {
                                                    let checked_str = if state
                                                        .is_checked
                                                        .load(atomic::Ordering::Acquire)
                                                    {
                                                        "Checked"
                                                    } else {
                                                        "Unchecked"
                                                    };
                                                    text(format!("State: {}", checked_str));
                                                });
                                            },
                                        );
                                    });
                                },
                            )
                        },
                    );
                },
            )
        },
    );
}
