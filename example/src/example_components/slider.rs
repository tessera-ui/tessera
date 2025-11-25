use std::sync::{Arc, Mutex};

use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    slider::{
        RangeSliderArgsBuilder, RangeSliderState, SliderArgsBuilder, SliderState, centered_slider,
        range_slider, slider,
    },
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

struct SliderShowcaseState {
    scrollable_state: ScrollableState,
    value: Arc<Mutex<f32>>,
    slider_state: SliderState,
    centered_value: Arc<Mutex<f32>>,
    centered_slider_state: SliderState,
    range_value: Arc<Mutex<(f32, f32)>>,
    range_slider_state: RangeSliderState,
}

impl Default for SliderShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            value: Arc::new(Mutex::new(0.5)),
            slider_state: SliderState::new(),
            centered_value: Arc::new(Mutex::new(0.5)),
            centered_slider_state: SliderState::new(),
            range_value: Arc::new(Mutex::new((0.2, 0.8))),
            range_slider_state: RangeSliderState::new(),
        }
    }
}

#[tessera]
#[shard]
pub fn slider_showcase(#[state] state: SliderShowcaseState) {
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
fn test_content(state: Arc<SliderShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Slider Showcase"));

            let state_for_slider = state.clone();
            scope.child(move || {
                let value_clone = state_for_slider.value.clone();
                let on_change = Arc::new(move |new_value| {
                    *value_clone.lock().unwrap() = new_value;
                });
                slider(
                    SliderArgsBuilder::default()
                        .value(*state_for_slider.value.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                    state_for_slider.slider_state.clone(),
                );
            });

            let state_for_value_display = state.clone();
            scope.child(move || {
                let value = *state_for_value_display.value.lock().unwrap();
                text(format!("Current value: {:.2}", value));
            });

            // Centered Slider Showcase
            scope.child(|| text("Centered Slider Showcase"));

            let state_for_centered_slider = state.clone();
            scope.child(move || {
                let centered_value_clone = state_for_centered_slider.centered_value.clone();
                let on_change = Arc::new(move |new_value| {
                    *centered_value_clone.lock().unwrap() = new_value;
                });
                centered_slider(
                    SliderArgsBuilder::default()
                        .value(*state_for_centered_slider.centered_value.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                    state_for_centered_slider.centered_slider_state.clone(),
                );
            });

            let state_for_centered_value_display = state.clone();
            scope.child(move || {
                let centered_value = *state_for_centered_value_display
                    .centered_value
                    .lock()
                    .unwrap();
                text(format!("Centered value: {:.2}", centered_value));
            });

            // Range Slider Showcase
            scope.child(|| text("Range Slider Showcase"));

            let state_for_range_slider = state.clone();
            scope.child(move || {
                let range_value_clone = state_for_range_slider.range_value.clone();
                let on_change = Arc::new(move |new_value| {
                    *range_value_clone.lock().unwrap() = new_value;
                });
                range_slider(
                    RangeSliderArgsBuilder::default()
                        .value(*state_for_range_slider.range_value.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                    state_for_range_slider.range_slider_state.clone(),
                );
            });

            let state_for_range_value_display = state.clone();
            scope.child(move || {
                let (start, end) = *state_for_range_value_display.range_value.lock().unwrap();
                text(format!("Range value: {:.2} - {:.2}", start, end));
            });
        },
    )
}
