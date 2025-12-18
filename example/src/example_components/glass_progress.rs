use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    modifier::ModifierExt as _,
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn glass_progress_showcase() {
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
    let progress = remember(|| 0.5);

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Progress Showcase"));

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.child(|| {
                text(TextArgsBuilder::default()
                    .text("This is the glass progress, adjust the slider below to change its value.")
                    .size(Dp(20.0))
                    .color(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .on_surface_variant,
                    )
                    .build()
                    .unwrap());
            });

            scope.child(move || {
                glass_progress(
                    GlassProgressArgsBuilder::default()
                        .value(progress.get())
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                spacer(Modifier::new().height(Dp(20.0)));
            });

            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    progress.set(new_value);
                });
                slider(
                    SliderArgsBuilder::default()
                        .value(progress.get())
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
