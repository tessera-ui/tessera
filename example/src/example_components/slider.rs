use std::sync::Arc;

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
    let value = remember(|| 0.5);
    let centered_value = remember(|| 0.5);
    let range_value = remember(|| (0.2, 0.8));
    let icon_slider_value = remember(|| 0.5);

    column(
        ColumnArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(closure!(clone value, |new_value| {
                    value.set(new_value);
                }));
                slider(
                    SliderArgsBuilder::default()
                        .value(value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let current_value = value.get();
                text(format!("Current value: {:.2}", current_value));
            });

            // Centered Slider Showcase
            scope.child(|| text("Centered Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    centered_value.set(new_value);
                });
                centered_slider(
                    SliderArgsBuilder::default()
                        .value(centered_value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let current_centered_value = centered_value.get();
                text(format!("Centered value: {:.2}", current_centered_value));
            });

            // Range Slider Showcase
            scope.child(|| text("Range Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    range_value.set(new_value);
                });
                range_slider(
                    RangeSliderArgsBuilder::default()
                        .value(range_value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let (start, end) = range_value.get();
                text(format!("Range value: {:.2} - {:.2}", start, end));
            });

            // Slider with Inset Icon Showcase
            scope.child(|| text("Slider with Inset Icon Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| {
                    icon_slider_value.set(new_value);
                });
                slider(
                    SliderArgsBuilder::default()
                        .value(icon_slider_value.get())
                        .on_change(on_change)
                        .width(DimensionValue::Fixed(Dp(250.0).to_px()))
                        .size(tessera_ui_basic_components::slider::SliderSize::Medium)
                        .inset_icon(filled::volume_up_icon())
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let current_icon_value = icon_slider_value.get();
                text(format!("Icon Slider value: {:.2}", current_icon_value));
            });
        },
    )
}
