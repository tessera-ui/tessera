use std::sync::Arc;

use tessera_ui::{Color, Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
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
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
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
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Progress indicators (Material 3).".to_string()));

            scope.child(|| spacer(Modifier::new().height(Dp(12.0))));

            scope.child(|| text("Linear (determinate).".to_string()));
            scope.child(move || {
                linear_progress_indicator(
                    LinearProgressIndicatorArgsBuilder::default()
                        .progress(progress_value.get())
                        .modifier(Modifier::new().width(Dp(240.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(12.0))));

            scope.child(|| text("Adjust progress value:".to_string()));
            scope.child(move || {
                let on_change = Arc::new(move |new_value| progress_value.set(new_value));
                slider(
                    SliderArgsBuilder::default()
                        .value(progress_value.get())
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(240.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

            scope.child(|| text("Linear (indeterminate).".to_string()));
            scope.child(|| {
                linear_progress_indicator(
                    LinearProgressIndicatorArgsBuilder::default()
                        .modifier(Modifier::new().width(Dp(240.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

            scope.child(|| text("Circular (determinate).".to_string()));
            scope.child(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgsBuilder::default()
                        .progress(progress_value.get())
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

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
