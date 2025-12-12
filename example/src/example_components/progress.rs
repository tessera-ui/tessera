use std::sync::Arc;

use closure::closure;
use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    progress::{ProgressArgsBuilder, progress},
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

#[tessera]
#[shard]
pub fn progress_showcase() {
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
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    let progress_value = remember(|| 0.5);

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| {
                text("This is the progress, adjust the slider below to change its value.")
            });

            scope.child(move || {
                let progress_val = progress_value.get();
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

            scope.child(move || {
                let on_change = Arc::new(closure!(clone progress_value, |new_value| {
                    progress_value.set(new_value);
                }));
                slider(
                    SliderArgsBuilder::default()
                        .value(progress_value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
