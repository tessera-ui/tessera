use tessera_ui::{Color, Dp, Modifier, remember, retain, shard, tessera};
use tessera_ui_basic_components::{
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt as _,
    progress::{
        CircularProgressIndicatorArgs, LinearProgressIndicatorArgs, circular_progress_indicator,
        linear_progress_indicator,
    },
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
        test_content,
    );
}

#[tessera]
fn test_content() {
    let progress_value = remember(|| 0.6);
    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width()),
        controller,
        move |scope| {
            scope.item(|| text("Progress indicators (Material 3).".to_string()));
            scope.item(|| spacer(Modifier::new().height(Dp(12.0))));
            scope.item(|| text("Linear (determinate).".to_string()));
            scope.item(move || {
                linear_progress_indicator(
                    LinearProgressIndicatorArgs::default()
                        .progress(progress_value.get())
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });
            scope.item(|| spacer(Modifier::new().height(Dp(12.0))));
            scope.item(|| text("Adjust progress value:".to_string()));
            scope.item(move || {
                slider(
                    SliderArgs::default()
                        .value(progress_value.get())
                        .on_change(move |new_value| progress_value.set(new_value))
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });
            scope.item(|| spacer(Modifier::new().height(Dp(24.0))));
            scope.item(|| text("Linear (indeterminate).".to_string()));
            scope.item(|| {
                linear_progress_indicator(
                    LinearProgressIndicatorArgs::default()
                        .modifier(Modifier::new().width(Dp(240.0))),
                );
            });
            scope.item(|| spacer(Modifier::new().height(Dp(24.0))));
            scope.item(|| text("Circular (determinate).".to_string()));
            scope.item(move || {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default().progress(progress_value.get()),
                );
            });
            scope.item(|| spacer(Modifier::new().height(Dp(24.0))));
            scope.item(|| text("Circular (indeterminate).".to_string()));
            scope.item(|| {
                circular_progress_indicator(
                    CircularProgressIndicatorArgs::default().track_color(Color::TRANSPARENT),
                );
            });
        },
    );
}
