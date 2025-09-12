use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
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
    scrollable_state: Arc<ScrollableState>,
    switch_state: Arc<RwLock<SwitchState>>,
    switch2_state: Arc<RwLock<SwitchState>>,
}

#[tessera]
#[shard]
pub fn switch_showcase(#[state] state: SwitchShowcaseState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
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
                            .style(Color::WHITE.into())
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
