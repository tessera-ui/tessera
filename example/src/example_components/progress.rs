use tessera_components::{
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt as _,
    progress::{
        CircularProgressIndicatorArgs, LinearProgressIndicatorArgs, ProgressArgs,
        circular_progress_indicator, linear_progress_indicator, progress,
    },
    slider::{SliderArgs, slider},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard};
#[shard]
pub fn progress_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let progress_value = remember(|| 0.5);
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width())
            .controller(controller)
            .content(move |scope| {
                scope.item(|| {
                    text(&TextArgs::from(
                        "This is the progress, adjust the slider below to change its value.",
                    ))
                });
                scope.item(move || {
                    let progress_val = progress_value.get();
                    progress(
                        &ProgressArgs::default()
                            .value(progress_val)
                            .modifier(Modifier::new().width(Dp(240.0))),
                    );
                });
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(10.0)))));
                scope.item(move || {
                    slider(
                        &SliderArgs::default()
                            .value(progress_value.get())
                            .on_change(move |new_value| progress_value.set(new_value))
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));

                scope.item(|| {
                    text(&TextArgs::from(
                        "Linear progress indicator (indeterminate).".to_string(),
                    ));
                });
                scope.item(|| {
                    linear_progress_indicator(&LinearProgressIndicatorArgs::default());
                });
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));
                scope.item(|| {
                    text(&TextArgs::from(
                        "Circular progress indicator (determinate).".to_string(),
                    ));
                });
                scope.item(move || {
                    circular_progress_indicator(
                        &CircularProgressIndicatorArgs::default().progress(progress_value.get()),
                    );
                });
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));
                scope.item(|| {
                    text(&TextArgs::from(
                        "Circular progress indicator (indeterminate).".to_string(),
                    ));
                });
                scope.item(|| {
                    circular_progress_indicator(
                        &CircularProgressIndicatorArgs::default()
                            .track_color(tessera_ui::Color::TRANSPARENT),
                    );
                });
            }),
    );
}
