use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    progress::{ProgressArgsBuilder, progress},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

#[derive(Clone)]
struct ProgressShowcaseState {
    scrollable_state: ScrollableState,
    progress: Arc<Mutex<f32>>,
}

impl Default for ProgressShowcaseState {
    fn default() -> Self {
        Self {
            scrollable_state: Default::default(),
            progress: Arc::new(Mutex::new(0.5)),
        }
    }
}

#[tessera]
#[shard]
pub fn progress_showcase(#[state] state: ProgressShowcaseState) {
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
fn test_content(state: Arc<ProgressShowcaseState>) {
    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| {
                text("This is the progress, adjust the slider below to change its value.")
            });

            let state_clone = state.clone();
            scope.child(move || {
                let progress_val = *state_clone.progress.lock().unwrap();
                progress(
                    ProgressArgsBuilder::default()
                        .value(progress_val)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                spacer(Dp(10.0));
            });

            let state_clone = state.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(clone state_clone.progress, |new_value| {
                    *progress.lock().unwrap() = new_value;
                }));
                slider(
                    SliderArgsBuilder::default()
                        .value(*state_clone.progress.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
