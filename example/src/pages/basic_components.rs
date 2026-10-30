use tessera_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    lazy_list::lazy_column,
    material_icons::filled,
    modifier::{ModifierExt, SemanticsArgs},
    progress::progress,
    row::row,
    slider::slider,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    switch::switch,
    text::text,
    text_input::text_input,
    theme::{MaterialTheme, material_theme},
};
use tessera_shard::shard;
use tessera_ui::{Color, Dp, FocusManager, Modifier, RenderSlot, remember, use_context};

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
    let checkbox_enabled = remember(|| true);
    let switch_checked = remember(|| false);
    let switch_enabled = remember(|| true);
    let switch_icons = remember(|| false);
    let boolean_changes = remember(|| [0_u32; 2]);
    let aggregate_choices = remember(|| [true, false]);
    let aggregate_enabled = remember(|| true);
    let aggregate_changes = remember(|| 0_u32);
    let slider_value = remember(|| 0.68_f32);

    lazy_column()
        .modifier(Modifier::new().fill_max_size())
        .estimated_item_size(Dp(120.0))
        .content_padding(Dp(16.0))
        .item_spacing(Dp(20.0))
        .item({
            let content = RenderSlot::new(move || {
                column().children(move || {
                    text()
                        .content("Basic Components")
                        .style(theme.with(|t| t.typography.headline_large));

                    spacer().modifier(Modifier::new().height(Dp(8.0)));

                    text().content("Foundation blocks for forms and interactions.");
                });
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
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
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
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
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
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
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
                column().children(move || {
                    text()
                        .content("Checkbox")
                        .style(theme.with(|t| t.typography.title_medium));

                    spacer().modifier(Modifier::new().height(Dp(6.0)));

                    text()
                        .content("Toggle a boolean option in forms and settings.")
                        .style(theme.with(|t| t.typography.body_medium));

                    spacer().modifier(Modifier::new().height(Dp(10.0)));

                    flow_row().item_spacing(Dp(8.0)).children(move || {
                        button()
                            .tonal()
                            .on_click(move || {
                                checkbox_enabled.set(!checkbox_enabled.get());
                                FocusManager::current().defer_keyboard_until_layout();
                            })
                            .child(move || {
                                text().content(if checkbox_enabled.get() {
                                    "Disable checkbox"
                                } else {
                                    "Enable checkbox"
                                });
                            });
                        button()
                            .text()
                            .on_click(move || checkbox_checked.set(!checkbox_checked.get()))
                            .child(move || {
                                text().content(if checkbox_checked.get() {
                                    "Set unchecked"
                                } else {
                                    "Set checked"
                                });
                            });
                    });
                    checkbox()
                        .checked(checkbox_checked.get())
                        .disabled(!checkbox_enabled.get())
                        .on_toggle(move |checked| {
                            checkbox_checked.set(checked);
                            boolean_changes.with_mut(|changes| changes[0] += 1);
                        })
                        .accessibility_label("Basic checkbox");

                    spacer().modifier(Modifier::new().height(Dp(8.0)));

                    text()
                        .content(format!(
                            "Checked: {} · Changes: {}",
                            checkbox_checked.get(),
                            boolean_changes.get()[0]
                        ))
                        .style(theme.with(|t| t.typography.label_medium));
                });
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
                column().children(move || {
                    text()
                        .content("Aggregate selection")
                        .style(theme.with(|theme| theme.typography.title_medium));
                    button()
                        .tonal()
                        .on_click(move || {
                            aggregate_enabled.set(!aggregate_enabled.get());
                            FocusManager::current().defer_keyboard_until_layout();
                        })
                        .child(move || {
                            text().content(if aggregate_enabled.get() {
                                "Disable selection"
                            } else {
                                "Enable selection"
                            });
                        });
                    column()
                        .modifier(Modifier::new().semantics(SemanticsArgs {
                            role: Some(tessera_ui::accesskit::Role::Group),
                            label: Some("Aggregate selection".to_string()),
                            ..Default::default()
                        }))
                        .children(move || {
                            for (index, label) in ["Select all", "Option one", "Option two"]
                                .into_iter()
                                .enumerate()
                            {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        if index == 0 {
                                            let choices = aggregate_choices.get();
                                            let checked = choices.iter().all(|value| *value);
                                            let indeterminate =
                                                !checked && choices.iter().any(|value| *value);
                                            checkbox()
                                                .checked(checked)
                                                .indeterminate(indeterminate)
                                                .disabled(!aggregate_enabled.get())
                                                .accessibility_label(label)
                                                .on_toggle(move |value| {
                                                    aggregate_choices.set([value; 2]);
                                                    aggregate_changes
                                                        .with_mut(|changes| *changes += 1);
                                                });
                                        } else {
                                            checkbox()
                                                .checked(aggregate_choices.get()[index - 1])
                                                .disabled(!aggregate_enabled.get())
                                                .accessibility_label(label)
                                                .on_toggle(move |value| {
                                                    aggregate_choices.with_mut(|choices| {
                                                        choices[index - 1] = value
                                                    });
                                                    aggregate_changes
                                                        .with_mut(|changes| *changes += 1);
                                                });
                                        }
                                        text().content(label);
                                    });
                            }
                        });
                    text().content(format!(
                        "Selected: {}/2 · Changes: {}",
                        aggregate_choices
                            .get()
                            .iter()
                            .filter(|value| **value)
                            .count(),
                        aggregate_changes.get()
                    ));
                });
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(|| {
                checkbox_state_gallery();
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
                column().children(move || {
                    text()
                        .content("Switch")
                        .style(theme.with(|t| t.typography.title_medium));

                    spacer().modifier(Modifier::new().height(Dp(6.0)));

                    text()
                        .content("Use for immediate on/off state changes.")
                        .style(theme.with(|t| t.typography.body_medium));

                    spacer().modifier(Modifier::new().height(Dp(10.0)));

                    flow_row().item_spacing(Dp(8.0)).children(move || {
                        button()
                            .tonal()
                            .on_click(move || {
                                switch_enabled.set(!switch_enabled.get());
                                FocusManager::current().defer_keyboard_until_layout();
                            })
                            .child(move || {
                                text().content(if switch_enabled.get() {
                                    "Disable switch"
                                } else {
                                    "Enable switch"
                                });
                            });
                        button()
                            .text()
                            .on_click(move || switch_checked.set(!switch_checked.get()))
                            .child(move || {
                                text().content(if switch_checked.get() {
                                    "Set unchecked"
                                } else {
                                    "Set checked"
                                });
                            });
                        button()
                            .tonal()
                            .on_click(move || switch_icons.set(!switch_icons.get()))
                            .child(move || {
                                text().content(if switch_icons.get() {
                                    "Hide thumb icons"
                                } else {
                                    "Show thumb icons"
                                });
                            });
                    });
                    switch()
                        .checked(switch_checked.get())
                        .enabled(switch_enabled.get())
                        .on_toggle(move |checked| {
                            switch_checked.set(checked);
                            boolean_changes.with_mut(|changes| changes[1] += 1);
                        })
                        .accessibility_label("Basic switch")
                        .child_optional(if switch_icons.get() {
                            Some(RenderSlot::new(move || {
                                icon().painter(if switch_checked.get() {
                                    filled::CHECK_SVG
                                } else {
                                    filled::CLOSE_SVG
                                });
                            }))
                        } else {
                            None
                        });

                    spacer().modifier(Modifier::new().height(Dp(8.0)));

                    text()
                        .content(format!(
                            "Checked: {} · Changes: {}",
                            switch_checked.get(),
                            boolean_changes.get()[1]
                        ))
                        .style(theme.with(|t| t.typography.label_medium));
                });
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
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
            });
            move || content.render()
        })
        .item({
            let content = RenderSlot::new(move || {
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
            move || content.render()
        });
}

#[tessera_ui::tessera]
fn checkbox_state_gallery() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let dark = remember(|| false);
    let error = remember(|| false);
    let enabled = remember(|| true);
    let checked = remember(|| [false, true, false]);
    let mixed = remember(|| [false, false, true]);
    let palettes = remember(|| {
        // Theme seeds use linear sRGB, including the default purple seed.
        let seed = Color::from_rgb(0.135633335, 0.080219820, 0.371237695);
        [
            MaterialTheme::default(),
            MaterialTheme::from_seed(seed, true),
        ]
    });
    column().children(move || {
        text()
            .content("Checkbox states")
            .style(theme.with(|theme| theme.typography.title_medium));
        flow_row().item_spacing(Dp(8.0)).children(move || {
            button()
                .tonal()
                .on_click(move || dark.set(!dark.get()))
                .child(move || {
                    text().content(if dark.get() {
                        "Use light theme"
                    } else {
                        "Use dark theme"
                    });
                });
            button()
                .tonal()
                .on_click(move || error.set(!error.get()))
                .child(move || {
                    text().content(if error.get() {
                        "Clear error"
                    } else {
                        "Show error"
                    });
                });
            button()
                .tonal()
                .on_click(move || {
                    enabled.set(!enabled.get());
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text().content(if enabled.get() {
                        "Disable choices"
                    } else {
                        "Enable choices"
                    });
                });
        });
        spacer().modifier(Modifier::new().height(Dp(8.0)));
        material_theme()
            .theme(move || palettes.with(|palettes| palettes[usize::from(dark.get())].clone()))
            .child(move || {
                let theme = use_context::<MaterialTheme>().unwrap();
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .style(SurfaceStyle::Filled {
                        color: theme.with(|theme| theme.color_scheme.surface),
                    })
                    .content_color(theme.with(|theme| theme.color_scheme.on_surface))
                    .child(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(8.0)))
                            .children(move || {
                                for (index, label) in
                                    ["Choice A", "Choice B", "Choice C"].into_iter().enumerate()
                                {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            checkbox()
                                                .checked(checked.get()[index])
                                                .indeterminate(mixed.get()[index])
                                                .is_error(error.get())
                                                .disabled(!enabled.get())
                                                .accessibility_label(label)
                                                .accessibility_description_optional(
                                                    error.get().then(|| {
                                                        "Review this selection".to_string()
                                                    }),
                                                )
                                                .on_toggle(move |value| {
                                                    checked
                                                        .with_mut(|checked| checked[index] = value);
                                                    mixed.with_mut(|mixed| mixed[index] = false);
                                                });
                                            text().content(label);
                                        });
                                }
                                if error.get() {
                                    text()
                                        .content("Review these selections.")
                                        .color(theme.with(|theme| theme.color_scheme.error));
                                }
                            });
                    });
            });
    });
}
