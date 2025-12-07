use std::sync::{Arc, Mutex};

use closure::closure;
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
    let value = remember(|| Mutex::new(0.5));
    let slider_controller = remember(GlassSliderController::new);

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Glass Slider Showcase"));

            let value_clone = value.clone();
            let slider_controller_clone = slider_controller.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(clone value_clone, |new_value| {
                    *value_clone.lock().unwrap() = new_value;
                }));
                glass_slider_with_controller(
                    GlassSliderArgsBuilder::default()
                        .value(*value_clone.lock().unwrap())
                        .on_change(on_change)
                        .width(Dp(250.0))
                        .build()
                        .unwrap(),
                    slider_controller_clone,
                );
            });

            let value_clone = value.clone();
            scope.child(move || {
                let value = *value_clone.lock().unwrap();
                text(
                    TextArgsBuilder::default()
                        .text(format!("Value: {:.2}", value))
                        .size(Dp(16.0))
                        .build()
                        .unwrap(),
                );
            });
        },
    )
}
