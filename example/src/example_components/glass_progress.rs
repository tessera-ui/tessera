use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    glass_progress::{GlassProgressArgs, glass_progress},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgs, scrollable},
    slider::{SliderArgs, slider},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn glass_progress_showcase() {
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
    let progress = remember(|| 0.5);

    column(
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            scope.child(|| text("Glass Progress Showcase"));

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.child(|| {
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

            scope.child(move || {
                glass_progress(
                    GlassProgressArgs::default()
                        .value(progress.get())
                        .modifier(Modifier::new().width(Dp(250.0))),
                );
            });

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.child(move || {
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
    )
}
