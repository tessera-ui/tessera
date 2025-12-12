use std::sync::Arc;

use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    glass_slider::{GlassSliderArgsBuilder, GlassSliderController, glass_slider_with_controller},
    scrollable::{ScrollableArgsBuilder, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn glass_slider_showcase() {
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
    let value = remember(|| 0.5);
    let slider_controller = remember(GlassSliderController::new);

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Slider Showcase"));
            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    value.set(new_value);
                });
                glass_slider_with_controller(
                    GlassSliderArgsBuilder::default()
                        .value(value.get())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    slider_controller,
                );
            });

            scope.child(move || {
                text(
                    TextArgsBuilder::default()
                        .text(format!("Value: {:.2}", value.get()))
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
