use tessera_ui::{Color, Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    progress::{
        CircularProgressIndicatorArgs, LinearProgressIndicatorArgs, circular_progress_indicator,
        linear_progress_indicator,
    },
    scrollable::{ScrollableArgs, scrollable},
    slider::{SliderArgs, slider},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::text,
};

#[tessera]
#[shard]
pub fn progress_indicator_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
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
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(|| text("Progress indicators (Material 3).".to_string()));

            scope.child(|| spacer(Modifier::new().height(Dp(12.0))));

            scope.child(|| text("Linear (determinate).".to_string()));
            scope.child(move || {
                linear_progress_indicator(
                    LinearProgressIndicatorArgs::default()
                        .progress(progress_value.get())
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(12.0))));

            scope.child(|| text("Adjust progress value:".to_string()));
            scope.child(move || {
                slider(
                    SliderArgs::default()
                        .value(progress_value.get())
                        .on_change(move |new_value| progress_value.set(new_value))
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

            scope.child(|| text("Linear (indeterminate).".to_string()));
            scope.child(|| {
                linear_progress_indicator(
                    LinearProgressIndicatorArgs::default()
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

            scope.child(|| text("Circular (determinate).".to_string()));
            scope.child(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default().progress(progress_value.get()),
                );
            });

            scope.child(|| spacer(Modifier::new().height(Dp(24.0))));

            scope.child(|| text("Circular (indeterminate).".to_string()));
            scope.child(|| {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default().track_color(Color::TRANSPARENT),
                );
            });
        },
    )
}
