use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    material_color::global_material_scheme,
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct GlassProgressShowcaseState {
    progress: Arc<Mutex<f32>>,
}

impl Default for GlassProgressShowcaseState {
    fn default() -> Self {
        Self {
            progress: Arc::new(Mutex::new(0.5)),
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
                    .color(global_material_scheme().on_surface_variant)
                    .build()
                    .unwrap());
            });

            let state_clone = state.clone();
            scope.child(move || {
                let progress_val = *state_clone.progress.lock().unwrap();
                glass_progress(
                    GlassProgressArgsBuilder::default()
                        .value(progress_val)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                spacer(Dp(20.0));
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
