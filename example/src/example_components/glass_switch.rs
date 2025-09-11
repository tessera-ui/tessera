use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    glass_switch::{GlassSwitchArgsBuilder, GlassSwitchState, glass_switch},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone, Default)]
struct GlassSwitchShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    glass_switch_state: Arc<RwLock<GlassSwitchState>>,
    glass_switch_state2: Arc<RwLock<GlassSwitchState>>,
}

#[tessera]
#[shard]
pub fn glass_switch_showcase(#[state] state: GlassSwitchShowcaseState) {
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
fn test_content(state: Arc<GlassSwitchShowcaseState>) {
    let glass_switch_state = state.glass_switch_state.clone();
    let glass_switch_state2 = state.glass_switch_state2.clone();
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Switch Showcase"));

            scope.child(move || {
                glass_switch(
                    GlassSwitchArgsBuilder::default()
                        .on_toggle(Arc::new(|value| {
                            println!("Glass Switch toggled to: {}", value);
                        }))
                        .build()
                        .unwrap(),
                    glass_switch_state,
                );
            });

            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Disabled Glass Switch")
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                )
            });
            scope.child(|| {
                // Disabled by not providing on_change
                glass_switch(
                    GlassSwitchArgsBuilder::default().build().unwrap(),
                    glass_switch_state2,
                );
            });
        },
    )
}
