use tessera_components::{
    button::button, checkbox::checkbox, column::column, lazy_list::lazy_column,
    modifier::ModifierExt, progress::progress, slider::slider, spacer::spacer, switch::switch,
    text::text, text_input::text_input, theme::MaterialTheme,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, remember, use_context};

#[shard]
pub fn basic_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();
    let button_click_count = remember(|| 0_u32);
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
