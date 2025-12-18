use std::sync::Arc;

use tessera_ui::{Dp, Modifier, remember, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    material_icons::filled,
    modifier::ModifierExt as _,
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
    let value = remember(|| 0.5);
    let centered_value = remember(|| 0.5);
    let range_value = remember(|| (0.2, 0.8));
    let icon_slider_value = remember(|| 0.5);
    let step_value = remember(|| 0.5);
    let step_range_value = remember(|| (0.2, 0.8));

    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        move |scope| {
            scope.child(|| text("Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| value.set(new_value));
                slider(
                    SliderArgsBuilder::default()
                        .value(value.get())
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(250.0)))
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
                        .modifier(Modifier::new().width(Dp(250.0)))
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
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let (start, end) = range_value.get();
                text(format!("Range value: {:.2} - {:.2}", start, end));
            });

            // Discrete Slider Showcase
            scope.child(|| text("Discrete Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| step_value.set(new_value));
                slider(
                    SliderArgsBuilder::default()
                        .value(step_value.get())
                        .steps(5)
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let current_value = step_value.get();
                text(format!("Discrete value (steps=5): {:.2}", current_value));
            });

            scope.child(|| text("Discrete Range Slider Showcase"));

            scope.child(move || {
                let on_change = Arc::new(move |new_value| step_range_value.set(new_value));
                range_slider(
                    RangeSliderArgsBuilder::default()
                        .value(step_range_value.get())
                        .steps(5)
                        .on_change(on_change)
                        .modifier(Modifier::new().width(Dp(250.0)))
                        .build()
                        .unwrap(),
                );
            });

            scope.child(move || {
                let (start, end) = step_range_value.get();
                text(format!(
                    "Discrete range value (steps=5): {:.2} - {:.2}",
                    start, end
                ));
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
                        .modifier(Modifier::new().width(Dp(250.0)))
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
