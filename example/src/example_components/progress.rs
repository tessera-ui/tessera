use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    progress::{
        CircularProgressIndicatorArgs, LinearProgressIndicatorArgs, ProgressArgs,
        circular_progress_indicator, linear_progress_indicator, progress,
    },
    scrollable::{ScrollableArgs, scrollable},
    slider::{SliderArgs, slider},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::text,
};

#[tessera]
#[shard]
pub fn progress_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
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
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(|| {
                text("This is the progress, adjust the slider below to change its value.")
            });

            scope.child(move || {
                let progress_val = progress_value.get();
                progress(
                    ProgressArgs::default()
                        .value(progress_val)
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

            scope.child(move || {
                slider(
                    SliderArgs::default()
                        .value(progress_value.get())
                        .on_change(move |new_value| progress_value.set(new_value))
                        .modifier(Modifier::new().width(Dp(250.0))),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Linear progress indicator (indeterminate).".to_string());
            });
            scope.child(|| {
                linear_progress_indicator(LinearProgressIndicatorArgs::default());
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Circular progress indicator (determinate).".to_string());
            });
            scope.child(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default().progress(progress_value.get()),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(20.0))));

            scope.child(|| {
                text("Circular progress indicator (indeterminate).".to_string());
            });
            scope.child(|| {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default()
                        .track_color(tessera_ui::Color::TRANSPARENT),
                );
            });
        },
    )
}
