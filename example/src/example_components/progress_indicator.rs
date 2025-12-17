use std::sync::Arc;

use tessera_ui::{Color, DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    progress::{
        CircularProgressIndicatorArgsBuilder, LinearProgressIndicatorArgsBuilder,
        circular_progress_indicator, linear_progress_indicator,
    },
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

#[tessera]
#[shard]
pub fn progress_indicator_showcase() {
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
                        test_content,
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    let progress_value = remember(|| 0.6);

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Progress indicators (Material 3).".to_string()));

            scope.child(|| spacer(Dp(12.0)));

            scope.child(|| text("Linear (determinate).".to_string()));
            scope.child(move || {
                linear_progress_indicator(
                    LinearProgressIndicatorArgsBuilder::default()
                        .progress(progress_value.get())
                        .width(DimensionValue::Fixed(Dp(240.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Dp(12.0)));

            scope.child(|| text("Adjust progress value:".to_string()));
            scope.child(move || {
                let on_change = Arc::new(move |new_value| progress_value.set(new_value));
                slider(
                    SliderArgsBuilder::default()
                        .value(progress_value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(260.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Dp(24.0)));

            scope.child(|| text("Linear (indeterminate).".to_string()));
            scope.child(|| {
                linear_progress_indicator(
                    LinearProgressIndicatorArgsBuilder::default()
                        .width(DimensionValue::Fixed(Dp(240.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Dp(24.0)));

            scope.child(|| text("Circular (determinate).".to_string()));
            scope.child(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgsBuilder::default()
                        .progress(progress_value.get())
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Dp(24.0)));

            scope.child(|| text("Circular (indeterminate).".to_string()));
            scope.child(|| {
                circular_progress_indicator(
                    CircularProgressIndicatorArgsBuilder::default()
                        .track_color(Color::TRANSPARENT)
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
