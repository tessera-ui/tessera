use std::sync::{Arc, Mutex};

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderState, glass_slider},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct GlassSliderShowcaseState {
    scrollable_state: ScrollableState,
    value: Arc<Mutex<f32>>,
    slider_state: GlassSliderState,
}

impl Default for GlassSliderShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            value: Arc::new(Mutex::new(0.5)),
            slider_state: Default::default(),
        }
    }
}

#[tessera]
#[shard]
pub fn glass_slider_showcase(#[state] state: GlassSliderShowcaseState) {
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
fn test_content(state: Arc<GlassSliderShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Slider Showcase"));

            let state_clone = state.clone();
            scope.child(move || {
                let value_clone = state_clone.value.clone();
                let on_change = Arc::new(move |new_value| {
                    *value_clone.lock().unwrap() = new_value;
                });
                glass_slider(
                    GlassSliderArgsBuilder::default()
                        .value(*state_clone.value.lock().unwrap())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    state_clone.slider_state.clone(),
                );
            });

            scope.child(move || {
                let value = *state.value.lock().unwrap();
                text(
                    TextArgsBuilder::default()
                        .text(format!("Value: {:.2}", value))
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
