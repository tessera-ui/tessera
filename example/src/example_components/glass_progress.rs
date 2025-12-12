use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera, use_context};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_progress::{GlassProgressArgsBuilder, glass_progress},
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{SliderArgsBuilder, slider},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialColorScheme,
};

#[tessera]
#[shard]
pub fn glass_progress_showcase() {
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
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Progress Showcase"));

            scope.child(|| {
                spacer(Dp(20.0));
            });

            scope.child(|| {
                text(TextArgsBuilder::default()
                    .text("This is the glass progress, adjust the slider below to change its value.")
                    .size(Dp(20.0))
                    .color(use_context::<MaterialColorScheme>().get().on_surface_variant)
                    .build()
                    .unwrap());
            });

            scope.child(move || {
                glass_progress(
                    GlassProgressArgsBuilder::default()
                        .value(progress.get())
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(|| {
                spacer(Dp(20.0));
            });

            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    progress.set(new_value);
                });
                slider(
                    SliderArgsBuilder::default()
                        .value(progress.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
