use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    switch::{SwitchArgsBuilder, SwitchState, switch},
    text::{TextArgsBuilder, text},
};

#[derive(Default)]
struct SwitchShowcaseState {
    scrollable_state: ScrollableState,
    switch_state: SwitchState,
    switch2_state: SwitchState,
}

#[tessera]
#[shard]
pub fn switch_showcase(#[state] state: SwitchShowcaseState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
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
fn test_content(state: Arc<SwitchShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Switch Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            let state_clone = state.clone();
            scope.child(move || {
                switch(
                    SwitchArgsBuilder::default()
                        .on_toggle(Arc::new(|value| {
                            println!("Switch toggled to: {}", value);
                        }))
                        .build()
                        .unwrap(),
                    state_clone.switch_state.clone(),
                );
            });

            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Disabled Switch")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });
            scope.child(move || {
                // Disabled by not providing on_change
                switch(
                    SwitchArgsBuilder::default().build().unwrap(),
                    state.switch2_state.clone(),
                );
            });
        },
    )
}
