use tessera_components::{
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    lazy_list::lazy_column,
    material_icons::filled,
    modifier::ModifierExt,
    progress::progress,
    slider::slider,
    spacer::spacer,
    switch::switch,
    text::text,
    text_input::text_input,
    theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, remember, use_context};

#[shard]
pub fn basic_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let button_click_count = remember(|| 0_u32);
    let button_variant = remember(ButtonVariant::default);
    let buttons_enabled = remember(|| true);
    let button_shape = remember(ButtonShape::default);
    let button_toggle = remember(|| false);
    let button_selected = remember(|| [false; 5]);
    let input_value = remember(|| String::from("Try typing here"));
    let checkbox_checked = remember(|| true);
    let switch_checked = remember(|| false);
    let slider_value = remember(|| 0.68_f32);

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(16.0))
        .item_spacing(Dp(20.0))
        .item(move || {
            column().children(move || {
                text()
                    .content("Basic Components")
                    .style(theme.with(|t| t.typography.headline_large));

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text().content("Foundation blocks for forms and interactions.");
            });
        })
        .item(move || {
            column().children(move || {
                text()
                    .content("Button")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Trigger actions in response to user intent.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                button()
                    .filled()
                    .on_click(move || {
                        button_click_count.with_mut(|count| *count += 1);
                    })
                    .child(|| {
                        text().content("Primary Action");
                    });

                spacer().modifier(Modifier::new().height(Dp(12.0)));

                button().outlined().on_click(|| {}).child(|| {
                    text().content("Secondary Action");
                });

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Clicks: {}", button_click_count.get()))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        })
        .item(move || {
            column().children(move || {
                text()
                    .content("Expressive button sizes")
                    .style(theme.with(|t| t.typography.title_medium));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                text().content("Extra-small through extra-large actions with icons.");
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .on_click(move || {
                                button_variant.set(match button_variant.get() {
                                    ButtonVariant::Filled => ButtonVariant::Tonal,
                                    ButtonVariant::Tonal => ButtonVariant::Outlined,
                                    ButtonVariant::Outlined if button_toggle.get() => {
                                        ButtonVariant::Elevated
                                    }
                                    ButtonVariant::Outlined => ButtonVariant::Text,
                                    ButtonVariant::Text => ButtonVariant::Elevated,
                                    ButtonVariant::Elevated => ButtonVariant::Filled,
                                })
                            })
                            .child(move || {
                                text().content(format!("Style: {:?}", button_variant.get()));
                            });
                        button()
                            .tonal()
                            .on_click(move || {
                                button_shape.set(match button_shape.get() {
                                    ButtonShape::Round => ButtonShape::Square,
                                    ButtonShape::Square => ButtonShape::Round,
                                });
                            })
                            .child(move || {
                                text().content(format!("Shape: {:?}", button_shape.get()));
                            });
                        button()
                            .tonal()
                            .on_click(move || {
                                let toggle = !button_toggle.get();
                                if toggle && button_variant.get() == ButtonVariant::Text {
                                    button_variant.set(ButtonVariant::Filled);
                                }
                                button_toggle.set(toggle);
                            })
                            .child(move || {
                                text().content(if button_toggle.get() {
                                    "Mode: Toggle"
                                } else {
                                    "Mode: Action"
                                });
                            });
                        button()
                            .text()
                            .on_click(move || buttons_enabled.set(!buttons_enabled.get()))
                            .child(move || {
                                text().content(if buttons_enabled.get() {
                                    "Disable buttons"
                                } else {
                                    "Enable buttons"
                                });
                            });
                    });
                spacer().modifier(Modifier::new().height(Dp(12.0)));
                flow_row()
                    .item_spacing(Dp(12.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        for (index, (size, label)) in [
                            (ButtonSize::ExtraSmall, "XS"),
                            (ButtonSize::Small, "S"),
                            (ButtonSize::Medium, "M"),
                            (ButtonSize::Large, "L"),
                            (ButtonSize::ExtraLarge, "XL"),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            button()
                                .size(size)
                                .variant(button_variant.get())
                                .shape_style(button_shape.get())
                                .selected_optional(if button_toggle.get() {
                                    Some(button_selected.get()[index])
                                } else {
                                    None
                                })
                                .enabled(buttons_enabled.get())
                                .accessibility_label(format!("{label} action"))
                                .on_click(move || {
                                    if button_toggle.get() {
                                        button_selected.with_mut(|selected| {
                                            selected[index] = !selected[index]
                                        });
                                    }
                                    button_click_count.with_mut(|count| *count += 1);
                                })
                                .leading_icon(|| {
                                    icon().vector(filled::ADD_SVG);
                                })
                                .child(move || {
                                    text().content(label);
                                });
                        }
                    });
                text().content(format!("Actions: {}", button_click_count.get()));
            });
        })
        .item(move || {
            column().children(move || {
                text()
                    .content("Text Input")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Collect short text from the user.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                text_input()
                    .initial_text(input_value.get())
                    .on_change(move |new_value| {
                        input_value.set(new_value.clone());
                        new_value
                    })
                    .accessibility_label("Basic text input");

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Current text: {}", input_value.get()))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        })
        .item(move || {
            column().children(move || {
                text()
                    .content("Checkbox")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Toggle a boolean option in forms and settings.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                checkbox()
                    .checked(checkbox_checked.get())
                    .on_toggle(move |checked| {
                        checkbox_checked.set(checked);
                    })
                    .accessibility_label("Basic checkbox");

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Checked: {}", checkbox_checked.get()))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        })
        .item(move || {
            column().children(move || {
                text()
                    .content("Switch")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Use for immediate on/off state changes.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                switch()
                    .checked(switch_checked.get())
                    .on_toggle(move |checked| {
                        switch_checked.set(checked);
                    })
                    .accessibility_label("Basic switch");

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Enabled: {}", switch_checked.get()))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        })
        .item(move || {
            column().children(move || {
                let current_slider_value = slider_value.get();

                text()
                    .content("Slider")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Select a value in a continuous range.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                slider()
                    .modifier(Modifier::new().width(Dp(240.0)))
                    .value(current_slider_value)
                    .on_change(move |new_value| {
                        slider_value.set(new_value);
                    })
                    .accessibility_label("Basic slider");

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Value: {:.2}", current_slider_value))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        })
        .item(move || {
            column().children(move || {
                let current_slider_value = slider_value.get();

                text()
                    .content("Progress")
                    .style(theme.with(|t| t.typography.title_medium));

                spacer().modifier(Modifier::new().height(Dp(6.0)));

                text()
                    .content("Communicate completion state to users.")
                    .style(theme.with(|t| t.typography.body_medium));

                spacer().modifier(Modifier::new().height(Dp(10.0)));

                progress()
                    .modifier(Modifier::new().width(Dp(240.0)))
                    .value(current_slider_value);

                spacer().modifier(Modifier::new().height(Dp(8.0)));

                text()
                    .content(format!("Progress: {:.0}%", current_slider_value * 100.0))
                    .style(theme.with(|t| t.typography.label_medium));
            });
        });
}
