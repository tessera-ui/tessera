use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    progress::{
        CircularProgressIndicatorArgsBuilder, LinearProgressIndicatorArgsBuilder,
        ProgressArgsBuilder, circular_progress_indicator, linear_progress_indicator, progress,
    },
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
            .modifier(Modifier::new().fill_max_width())
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
                        .modifier(Modifier::new().width(Dp(240.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| progress_value.set(new_value));
                slider(
                    SliderArgsBuilder::default()
                        .value(progress_value.get())
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Linear progress indicator (indeterminate).".to_string());
            });
            scope.child(|| {
                linear_progress_indicator(
                    LinearProgressIndicatorArgsBuilder::default()
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Circular progress indicator (determinate).".to_string());
            });
            scope.child(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgsBuilder::default()
                        .progress(progress_value.get())
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Circular progress indicator (indeterminate).".to_string());
            });
            scope.child(|| {
                circular_progress_indicator(
                    CircularProgressIndicatorArgsBuilder::default()
                        .track_color(tessera_ui::Color::TRANSPARENT)
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
