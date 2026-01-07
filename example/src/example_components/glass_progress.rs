use tessera_components::{
    glass_progress::{GlassProgressArgs, glass_progress},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt as _,
    slider::{SliderArgs, slider},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, remember, retain, shard, tessera, use_context};

#[tessera]
#[shard]
pub fn glass_progress_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    );
}

#[tessera]
fn test_content() {
    let progress = remember(|| 0.5);

    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .content_padding(Dp(16.0)),
        controller,
        move |scope| {
            scope.item(|| text("Glass Progress Showcase"));

            scope.item(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.item(|| {
                text(TextArgs::default()
                    .text("This is the glass progress, adjust the slider below to change its value.")
                    .size(Dp(20.0))
                    .color(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .on_surface_variant,
                    )
                    );
            });

            scope.item(move || {
                glass_progress(
                    GlassProgressArgs::default()
                        .value(progress.get())
                        .modifier(Modifier::new().width(Dp(250.0))),
                );
            });

            scope.item(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.item(move || {
                slider(
                    SliderArgs::default()
                        .value(progress.get())
                        .on_change(move |new_value| {
                            progress.set(new_value);
                        })
                        .modifier(Modifier::new().width(Dp(250.0))),
                );
            });
        },
    );
}
