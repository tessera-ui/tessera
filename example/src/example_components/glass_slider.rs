use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderController, glass_slider_with_controller},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct GlassSliderShowcaseState {
    scrollable_state: ScrollableState,
    value: Arc<Mutex<f32>>,
    slider_controller: Arc<GlassSliderController>,
}

impl Default for GlassSliderShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            value: Arc::new(Mutex::new(0.5)),
            slider_controller: Arc::new(GlassSliderController::new()),
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
                let on_change = Arc::new(closure!(clone state_clone.value, |new_value| {
                    *value.lock().unwrap() = new_value;
                }));
                glass_slider_with_controller(
                    GlassSliderArgsBuilder::default()
                        .value(*state_clone.value.lock().unwrap())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    state_clone.slider_controller.clone(),
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
