use std::sync::{Arc, Mutex};

use closure::closure;
use tessera_ui::{DimensionValue, Dp, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    material_icons::filled,
    scrollable::{ScrollableArgsBuilder, scrollable},
    slider::{RangeSliderArgsBuilder, SliderArgsBuilder, centered_slider, range_slider, slider},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};

#[tessera]
#[shard]
pub fn slider_showcase() {
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
    let centered_value = remember(|| Mutex::new(0.5));
    let range_value = remember(|| Mutex::new((0.2, 0.8)));
    let icon_slider_value = remember(|| Mutex::new(0.5));

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Slider Showcase"));

            let value_clone = value.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(clone value_clone, |new_value| {
                    *value_clone.lock().unwrap() = new_value;
                }));
                slider(
                    SliderArgsBuilder::default()
                        .value(*value_clone.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            let value_clone = value.clone();
            scope.child(move || {
                let value = *value_clone.lock().unwrap();
                text(format!("Current value: {:.2}", value));
            });

            // Centered Slider Showcase
            scope.child(|| text("Centered Slider Showcase"));

            let centered_value_clone = centered_value.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(
                    clone centered_value_clone,
                    |new_value| {
                        *centered_value_clone.lock().unwrap() = new_value;
                    }
                ));
                centered_slider(
                    SliderArgsBuilder::default()
                        .value(*centered_value_clone.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            let centered_value_clone = centered_value.clone();
            scope.child(move || {
                let centered_value = *centered_value_clone.lock().unwrap();
                text(format!("Centered value: {:.2}", centered_value));
            });

            // Range Slider Showcase
            scope.child(|| text("Range Slider Showcase"));

            let range_value_clone = range_value.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(
                    clone range_value_clone,
                    |new_value| {
                        *range_value_clone.lock().unwrap() = new_value;
                    }
                ));
                range_slider(
                    RangeSliderArgsBuilder::default()
                        .value(*range_value_clone.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            let range_value_clone = range_value.clone();
            scope.child(move || {
                let (start, end) = *range_value_clone.lock().unwrap();
                text(format!("Range value: {:.2} - {:.2}", start, end));
            });

            // Slider with Inset Icon Showcase
            scope.child(|| text("Slider with Inset Icon Showcase"));

            let icon_slider_value_clone = icon_slider_value.clone();
            scope.child(move || {
                let on_change = Arc::new(closure!(
                    clone icon_slider_value_clone,
                    |new_value| {
                        *icon_slider_value_clone.lock().unwrap() = new_value;
                    }
                ));
                slider(
                    SliderArgsBuilder::default()
                        .value(*icon_slider_value_clone.lock().unwrap())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .size(tessera_ui_basic_components::slider::SliderSize::Medium)
                        .inset_icon(filled::volume_up_icon())
                        .build()
                        .unwrap(),
                );
            });

            let icon_slider_value_clone = icon_slider_value.clone();
            scope.child(move || {
                let value = *icon_slider_value_clone.lock().unwrap();
                text(format!("Icon Slider value: {:.2}", value));
            });
        },
    )
}
