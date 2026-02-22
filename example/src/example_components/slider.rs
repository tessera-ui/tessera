use tessera_components::{
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    material_icons::filled,
    modifier::ModifierExt as _,
    slider::{RangeSliderArgs, SliderArgs, centered_slider, range_slider, slider},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, retain, shard};
#[shard]
pub fn slider_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let value = remember(|| 0.5);
    let centered_value = remember(|| 0.5);
    let range_value = remember(|| (0.2, 0.8));
    let icon_slider_value = remember(|| 0.5);
    let step_value = remember(|| 0.5);
    let step_range_value = remember(|| (0.2, 0.8));
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width())
            .controller(controller)
            .content(move |scope| {
                scope.item(|| text(&TextArgs::from("Slider Showcase")));

                scope.item(move || {
                    slider(
                        &SliderArgs::default()
                            .value(value.get())
                            .on_change(move |new_value| value.set(new_value))
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });

                scope.item(move || {
                    let current_value = value.get();
                    text(&TextArgs::from(format!(
                        "Current value: {:.2}",
                        current_value
                    )));
                });

                // Centered Slider Showcase,
                scope.item(|| text(&TextArgs::from("Centered Slider Showcase")));

                scope.item(move || {
                    centered_slider(
                        &SliderArgs::default()
                            .value(centered_value.get())
                            .on_change(move |new_value| {
                                centered_value.set(new_value);
                            })
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });

                scope.item(move || {
                    let current_centered_value = centered_value.get();
                    text(&TextArgs::from(format!(
                        "Centered value: {:.2}",
                        current_centered_value
                    )));
                });

                // Range Slider Showcase,
                scope.item(|| text(&TextArgs::from("Range Slider Showcase")));

                scope.item(move || {
                    range_slider(
                        &RangeSliderArgs::default()
                            .value(range_value.get())
                            .on_change(move |new_value| {
                                range_value.set(new_value);
                            })
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });

                scope.item(move || {
                    let (start, end) = range_value.get();
                    text(&TextArgs::from(format!(
                        "Range value: {:.2} - {:.2}",
                        start, end
                    )));
                });

                // Discrete Slider Showcase,
                scope.item(|| text(&TextArgs::from("Discrete Slider Showcase")));

                scope.item(move || {
                    slider(
                        &SliderArgs::default()
                            .value(step_value.get())
                            .steps(5)
                            .on_change(move |new_value| step_value.set(new_value))
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });

                scope.item(move || {
                    let current_value = step_value.get();
                    text(&TextArgs::from(format!(
                        "Discrete value (steps=5): {:.2}",
                        current_value
                    )));
                });

                scope.item(|| text(&TextArgs::from("Discrete Range Slider Showcase")));

                scope.item(move || {
                    range_slider(
                        &RangeSliderArgs::default()
                            .value(step_range_value.get())
                            .steps(5)
                            .on_change(move |new_value| step_range_value.set(new_value))
                            .modifier(Modifier::new().width(Dp(250.0))),
                    );
                });

                scope.item(move || {
                    let (start, end) = step_range_value.get();
                    text(&TextArgs::from(format!(
                        "Discrete range value (steps=5): {:.2} - {:.2}",
                        start, end
                    )));
                });

                // Slider with Inset Icon Showcase,
                scope.item(|| text(&TextArgs::from("Slider with Inset Icon Showcase")));

                scope.item(move || {
                    slider(
                        &SliderArgs::default()
                            .value(icon_slider_value.get())
                            .on_change(move |new_value| {
                                icon_slider_value.set(new_value);
                            })
                            .modifier(Modifier::new().width(Dp(250.0)))
                            .size(tessera_components::slider::SliderSize::Medium)
                            .inset_icon(filled::volume_up_icon()),
                    );
                });

                scope.item(move || {
                    let current_icon_value = icon_slider_value.get();
                    text(&TextArgs::from(format!(
                        "Icon Slider value: {:.2}",
                        current_icon_value
                    )));
                });
            }),
    );
}
