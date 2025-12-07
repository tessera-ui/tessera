use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
};

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    checkbox::{CheckboxArgsBuilder, checkbox},
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn checkbox_showcase() {
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
                                    scope.child(move || {
                                        let is_checked = remember(|| AtomicBool::new(true));
                                        row(
                                            RowArgsBuilder::default()
                                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                                .build()
                                                .unwrap(),
                                            |scope| {
                                                let is_checked_clone = is_checked.clone();
                                                scope.child(move || {
                                                    let on_toggle = Arc::new({
                                                        move |new_value| {
                                                            is_checked_clone.store(
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
                                                let is_checked_clone = is_checked.clone();
                                                scope.child(move || {
                                                    let checked_str = if is_checked_clone
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
