use std::sync::{Arc, Mutex};

use parking_lot::RwLock;
use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    slider::{SliderArgsBuilder, SliderState, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct GlassProgressShowcaseState {
    scrollable_state: Arc<ScrollableState>,
    progress: Arc<Mutex<f32>>,
    slider_state: Arc<RwLock<SliderState>>,
}

impl Default for GlassProgressShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            progress: Arc::new(Mutex::new(0.5)),
            slider_state: Default::default(),
        }
    }
}

#[tessera]
#[shard]
pub fn glass_progress_showcase(#[state] state: GlassProgressShowcaseState) {
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
fn test_content(state: Arc<GlassProgressShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Progress Showcase"));

            scope.child(|| {
                spacer(Dp(20.0));
            });

            scope.child(|| {
                text(TextArgsBuilder::default()
                    .text("This is the glass progress, adjust the slider below to change its value.")
                    .size(Dp(20.0))
                    .color(Color::GRAY)
                    .build()
                    .unwrap());
            });

            let state_clone = state.clone();
            scope.child(move || {
                let progress_val = *state_clone.progress.lock().unwrap();
                glass_progress(
                    GlassProgressArgsBuilder::default()
                        .value(progress_val)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                spacer(Dp(20.0));
            });

            let state_clone = state.clone();
            scope.child(move || {
                let progress_clone = state_clone.progress.clone();
                let on_change = Arc::new(move |new_value| {
                    *progress_clone.lock().unwrap() = new_value;
                });
                slider(
                    SliderArgsBuilder::default()
                        .value(*state_clone.progress.lock().unwrap())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    state_clone.slider_state.clone(),
                );
            });
        },
    )
}
