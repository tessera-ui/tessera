//! Native radio-button article figures and accessible choice scenarios.
//!
//! ## Usage
//!
//! Compare selection controls and explore radio groups in forms and settings.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    radio_button::{RadioGroupOrientation, radio_button, radio_group},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, LayoutCoordinates, Modifier, RenderSlot, State,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use super::super::{
    ACCENT, gap, media, selection_articles::option_label, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn overview_article() {
    super::prose(
        "• Use radio buttons, not switches, when only one item can be selected from a list.\n• Labels should be scannable.\n• Selected items are more prominent than unselected items.",
    );
    gap(Dp(32.0));
    super::overview();
    gap(Dp(48.0));
    subheading("Resources & availability".into());
    gap(Dp(24.0));
    spec_table(
        vec!["Resource".into(), "Status".into()],
        vec![
            vec![
                "Native radio button and radio group".into(),
                "Available in this example".into(),
            ],
            vec![
                "Theme colors and bundled icons".into(),
                "Available in the interactive figures".into(),
            ],
            vec![
                "Keyboard and accessibility semantics".into(),
                "Available on the live controls".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("What's new".into());
    gap(Dp(24.0));
    super::prose(
        "Radio buttons use new color mappings and support dynamic color. Selected icons use primary; unselected icons use on surface variant. Adjacent text retains the on surface color.",
    );
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "Color changes emphasize the selection without changing the label. These native choices demonstrate the current theme's color roles.",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn usage() {
    context(
        RenderSlot::new(|| {
            super::prose(
                "Radio buttons are the recommended way to make a single selection from a visible list of options. Only one radio button can be selected at a time. Always accompany each radio with a clear inline label.",
            );
        }),
        RenderSlot::new(language),
    );
    gap(Dp(48.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "Use radio buttons to select one option from a set and expose all available options. Checkboxes are different: several independent options can be selected together.",
            );
        }),
        RenderSlot::new(notification_settings),
    );
    gap(Dp(48.0));
    subheading("Choose the right selection control".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            choice_card(0);
            choice_card(1);
            choice_card(2);
        });
    gap(Dp(24.0));
    super::prose(
        "Do: use radio buttons for a single choice, checkboxes for multiple choices, and switches for independent on/off settings. Don't nest radio groups or use several selected radios to represent a multi-select list.",
    );
    gap(Dp(24.0));
    super::prose(
        "Use radio buttons when there are five or fewer options. Consider a drop-down menu when saving space is important, remembering that opening a menu adds clicks and cognitive effort.",
    );
}

#[tessera]
pub(super) fn anatomy(labels: bool) {
    stage(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            if labels {
                let selected = remember(|| Some(0_usize));
                boxed()
                    .modifier(Modifier::new().offset(Dp(400.0), Dp(95.0)).width(Dp(240.0)))
                    .children(move || {
                        options(
                            "Radio anatomy".into(),
                            vec!["Option 1".into(), "Option 2".into()],
                            selected,
                            true,
                            false,
                        );
                    });
                native_label("1".into(), 400.0, 42.0, 48.0);
                native_label("2".into(), 486.0, 42.0, 48.0);
                native_label("3".into(), 400.0, 230.0, 48.0);
            } else {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(488.0), Dp(100.0))
                            .size(Dp(48.0), Dp(48.0)),
                    )
                    .children(|| {
                        radio_button()
                            .selected(true)
                            .accessibility_label("Radio button icon anatomy");
                    });
                spacer().modifier(
                    Modifier::new()
                        .offset(Dp(511.0), Dp(140.0))
                        .size(Dp(1.0), Dp(40.0))
                        .background(
                            use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .color_scheme
                                .outline,
                        ),
                );
                native_label("1".into(), 488.0, 180.0, 48.0);
            }
        }),
    );
    gap(Dp(16.0));
    if labels {
        super::prose("1. Selected icon\n2. Adjacent label text\n3. Unselected icon");
    } else {
        super::prose("1. Radio button icon");
    }
}

#[tessera]
pub(super) fn placement() {
    context(
        RenderSlot::new(|| {
            super::prose(
                "Radio buttons are often arranged in stacked layouts. List them vertically with an appropriate option preselected. Use an adjacent label for every option; short, distinct labels are easier to scan.",
            );
        }),
        RenderSlot::new(|| {
            language_settings();
        }),
    );
    gap(Dp(40.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            layout_comparison(false);
            layout_comparison(true);
        });
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            subheading("Adjacent label text".into());
            gap(Dp(16.0));
            super::prose(
                "Always pair a radio button with a label describing what it selects. Each choice needs its own label. The label and the radio select the same option; label text keeps the on surface color during interaction.",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn behavior() {
    super::prose(
        "Selecting a radio button deselects the previous option. Activating the selected option does not clear it. A group can start with a selection or with no selection, but should provide a suitable default whenever possible.",
    );
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "Apply a settings change immediately when appropriate. In a form or dialog, keep a draft selection until the person confirms. Cancel restores the saved value; Continue commits the selected language in this example.",
            );
        }),
        RenderSlot::new(language),
    );
}

#[tessera]
pub(super) fn accessibility(section: String) {
    match section.as_str() {
        "Use cases" => {
            super::prose(
                "People using assistive technology should be able to:\n• Navigate to a radio button\n• Select a radio button\n• Get appropriate feedback for the input type",
            );
        }
        "Interaction & style" => {
            context(
                RenderSlot::new(|| {
                    super::prose(
                        "A radio button can be selected or unselected. Selecting one deselects the others. A group may start with one selection or none. Once an option is selected, selecting it again doesn't clear it.",
                    );
                    gap(Dp(16.0));
                    super::prose(
                        "To let people opt out, provide a Not applicable or No option choice, or a separate Clear selection action. People must be able to select either the radio or its text label.",
                    );
                }),
                RenderSlot::new(|| {
                    checkout(true).initial(0);
                }),
            );
            gap(Dp(32.0));
            subheading("Avoid applying density by default".into());
            gap(Dp(16.0));
            super::prose(
                "Keep targets at least 48 × 48dp by default. Let people explicitly choose a denser layout or theme. Keep the controls that reverse a density setting at least 48 × 48dp so the setting remains easy to undo.",
            );
        }
        "Initial focus" => {
            super::prose(
                "Tab enters the group at the selected radio, or the first option if none is selected. Shift+Tab enters at the selected radio, or the last option if none is selected. Arrow keys move focus and selection between options.",
            );
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    checkout(false).initial(1);
                    checkout(true);
                });
        }
        "Keyboard navigation" => {
            spec_table(vec!["Keys".into(), "Actions".into()], vec![
                vec!["Tab".into(), "Enter at the selected radio, or the first if none is selected; Tab again leaves the group".into()],
                vec!["Shift + Tab".into(), "Enter at the selected radio, or the last if none is selected; move backward out of the group".into()],
                vec!["Space".into(), "Select the focused option".into()],
                vec!["Right / Down arrow".into(), "Focus and select the next option, wrapping to the first".into()],
                vec!["Left / Up arrow".into(), "Focus and select the previous option, wrapping to the last".into()],
            ]);
            gap(Dp(24.0));
            checkout(false).initial(0);
        }
        _ => {
            super::prose(
                "Give the group a category name and each radio its own adjacent label. Assistive technology should announce the option's label, radio role, selected or unselected state, and its relationship to the group. Do not label a radio only by its appearance.",
            );
            gap(Dp(24.0));
            checkout(false).initial(0);
        }
    }
}

#[tessera]
pub(super) fn checkout(initial: Option<usize>, clear: bool) {
    let selected = remember(move || initial);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(360.0)).background(scheme.surface))
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_all(Dp(16.0))
                                .background(scheme.surface_container),
                        )
                        .children(|| {
                            icon().vector(filled::ARROW_BACK_SVG).size(Dp(24.0));
                            spacer().modifier(Modifier::new().width(Dp(16.0)));
                            text().content("Checkout").style(type_style(22.0, 28.0));
                        });
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                        .children(move || {
                            super::prose("Ship to");
                            gap(Dp(16.0));
                            options(
                                "Ship to".into(),
                                vec!["Home".into(), "Office".into()],
                                selected,
                                true,
                                false,
                            );
                            gap(Dp(16.0));
                            super::prose(format!(
                                "Shipping destination: {}",
                                match selected.get() {
                                    Some(0) => "Home",
                                    Some(_) => "Office",
                                    None => "Not selected",
                                }
                            ));
                            if clear {
                                button()
                                    .text()
                                    .accessibility_label("Clear shipping selection")
                                    .on_click(move || selected.set(None))
                                    .child(|| {
                                        text().content("Clear selection");
                                    });
                            }
                        });
                });
        });
}

#[tessera]
pub(super) fn options(
    name: String,
    values: Vec<String>,
    selected: State<Option<usize>>,
    enabled: bool,
    horizontal: bool,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    radio_group()
        .accessibility_label(name)
        .orientation(if horizontal {
            RadioGroupOrientation::Horizontal
        } else {
            RadioGroupOrientation::Vertical
        })
        .content(move || {
            let values = values.clone();
            let content = RenderSlot::new(move || {
                for (index, value) in values.iter().enumerate() {
                    let value = value.clone();
                    let focus = remember(FocusRequester::new).get();
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            radio_button()
                                .focus_requester(focus)
                                .selected(selected.get() == Some(index))
                                .enabled(enabled)
                                .accessibility_label(value.clone())
                                .on_select(move |_| selected.set(Some(index)));
                            option_label(
                                value.clone(),
                                Callback::new(move || {
                                    selected.set(Some(index));
                                    focus.request_focus();
                                }),
                                enabled,
                            )
                            .color(if enabled {
                                scheme.on_surface
                            } else {
                                scheme.on_surface.with_alpha(0.38)
                            })
                            .style(type_style(16.0, 24.0));
                        });
                }
            });
            if horizontal {
                flow_row().item_spacing(Dp(16.0)).children_shared(content);
            } else {
                column().children_shared(content);
            }
        });
}

#[tessera]
pub(super) fn language() {
    let draft = remember(|| Some(2_usize));
    let saved = remember(|| Some(2_usize));
    let status = remember(|| "Current language: Spanish".to_owned());
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.scrim.with_alpha(0.5),
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(320.0))
                        .padding_all(Dp(24.0))
                        .background_with_shape(
                            scheme.surface_container_high,
                            Shape::rounded_rectangle(Dp(16.0)),
                        ),
                )
                .children(move || {
                    icon().vector(filled::LANGUAGE_SVG).size(Dp(24.0));
                    gap(Dp(12.0));
                    text()
                        .content("Choose your language")
                        .style(type_style(20.0, 28.0));
                    super::prose("You can always change this later");
                    gap(Dp(16.0));
                    options(
                        "Choose your language".into(),
                        vec![
                            "English".into(),
                            "Chinese (Mandarin)".into(),
                            "Spanish".into(),
                        ],
                        draft,
                        true,
                        false,
                    );
                    gap(Dp(16.0));
                    flow_row().item_spacing(Dp(12.0)).children(move || {
                        button()
                            .text()
                            .accessibility_label("Cancel language selection")
                            .on_click(move || {
                                draft.set(saved.get());
                                status.set("Selection restored".into());
                            })
                            .child(|| {
                                text().content("Cancel");
                            });
                        button()
                            .text()
                            .accessibility_label("Confirm language selection")
                            .on_click(move || {
                                saved.set(draft.get());
                                status.set(format!(
                                    "Current language: {}",
                                    match draft.get() {
                                        Some(0) => "English",
                                        Some(1) => "Chinese (Mandarin)",
                                        _ => "Spanish",
                                    }
                                ));
                            })
                            .child(|| {
                                text().content("Continue");
                            });
                    });
                    gap(Dp(12.0));
                    super::prose(status.get());
                });
        });
}

#[tessera]
fn language_settings() {
    let selected = remember(|| Some(0_usize));
    card(
        "Settings".into(),
        RenderSlot::new(move || {
            super::prose("Language");
            gap(Dp(16.0));
            options(
                "Language".into(),
                vec![
                    "English".into(),
                    "Chinese (Mandarin)".into(),
                    "Spanish".into(),
                ],
                selected,
                true,
                false,
            );
        }),
    );
}

#[tessera]
fn notification_settings() {
    let selected = remember(|| Some(0_usize));
    card(
        "Notifications".into(),
        RenderSlot::new(move || {
            options(
                "Notifications".into(),
                vec![
                    "Allow notifications".into(),
                    "Turn off notifications".into(),
                ],
                selected,
                true,
                false,
            );
            gap(Dp(24.0));
            super::prose("Allow access");
            check_option("Microphone access".into());
            check_option("Location access".into());
        }),
    );
}

#[tessera]
fn check_option(value: String) {
    let checked = remember(|| true);
    let focus = remember(FocusRequester::new).get();
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            checkbox()
                .checked(checked.get())
                .focus_requester(focus)
                .accessibility_label(value.clone())
                .on_toggle(move |value| checked.set(value));
            option_label(
                value.clone(),
                Callback::new(move || {
                    checked.set(!checked.get());
                    focus.request_focus();
                }),
                true,
            );
        });
}

#[tessera]
fn choice_card(kind: usize) {
    card(
        ["Filters", "Meal options", "Settings"][kind].into(),
        RenderSlot::new(move || {
            if kind == 0 {
                let selected = remember(|| Some(0_usize));
                options(
                    "Sort by".into(),
                    vec![
                        "Relevance".into(),
                        "Lowest price".into(),
                        "Highest rating".into(),
                        "Most reviewed".into(),
                    ],
                    selected,
                    true,
                    false,
                );
            } else if kind == 1 {
                for value in ["Pickles", "Tomato", "Lettuce", "Cheese"] {
                    check_option(value.into());
                }
            } else {
                let checked = remember(|| true);
                super::prose("Allow notifications");
                switch()
                    .checked(checked.get())
                    .accessibility_label("Allow notifications independently")
                    .on_toggle(move |value| checked.set(value));
            }
            gap(Dp(16.0));
            super::prose(
                [
                    "Do: one selected option",
                    "Do: multiple independent options",
                    "Do: one setting on or off",
                ][kind],
            );
        }),
    );
}

#[tessera]
fn layout_comparison(horizontal: bool) {
    let selected = remember(|| Some(0_usize));
    card(
        if horizontal {
            "Caution: horizontal list".into()
        } else {
            "Do: stacked list".into()
        },
        RenderSlot::new(move || {
            options(
                if horizontal {
                    "Horizontal placement example".into()
                } else {
                    "Stacked placement example".into()
                },
                vec!["Option 1".into(), "Option 2".into(), "Option 3".into()],
                selected,
                true,
                horizontal,
            );
        }),
    );
}

#[tessera]
fn card(title: String, content: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .width(Dp(320.0))
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            text().content(title.clone()).style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            content.render();
        });
}

#[tessera]
pub(super) fn colors() {
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                stage(
                    1024.0,
                    220.0,
                    RenderSlot::new(move || {
                        for (index, selected) in [true, false].into_iter().enumerate() {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .offset(Dp(376.0 + index as f64 * 224.0), Dp(66.0))
                                        .size(Dp(48.0), Dp(48.0)),
                                )
                                .children(move || {
                                    radio_button()
                                        .selected(selected)
                                        .accessibility_label(format!(
                                            "{} {} color role",
                                            if dark { "Dark" } else { "Light" },
                                            if selected { "Selected" } else { "Unselected" }
                                        ));
                                });
                            native_label(
                                (index + 1).to_string(),
                                376.0 + index as f64 * 224.0,
                                142.0,
                                48.0,
                            );
                        }
                    }),
                );
            });
    }
    gap(Dp(16.0));
    super::prose(
        "Radio button color roles in light and dark themes:\n1. Primary\n2. On surface variant",
    );
    gap(Dp(40.0));
    context(
        RenderSlot::new(|| {
            subheading("Adjacent text label color".into());
            gap(Dp(16.0));
            super::prose(
                "Use on surface for adjacent text labels. This color remains the same when the label or radio is selected, hovered, focused or pressed.",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn states() {
    super::prose(
        "States communicate the status of a component. These examples retain real radio controls; their native surrounding circles illustrate hover, focus and pressed state layers.",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                stage(
                    1024.0,
                    300.0,
                    RenderSlot::new(move || {
                        for state in 0..5 {
                            native_label(
                                (state + 1).to_string(),
                                152.0 + state as f64 * 176.0,
                                34.0,
                                48.0,
                            );
                            for (row, selected) in [true, false].into_iter().enumerate() {
                                state_item(
                                    selected,
                                    state,
                                    dark,
                                    176.0 + state as f64 * 176.0,
                                    142.0 + row as f64 * 80.0,
                                );
                            }
                        }
                    }),
                );
            });
    }
    gap(Dp(16.0));
    super::prose("1. Enabled\n2. Hover\n3. Focused\n4. Pressed\n5. Disabled");
}

#[tessera]
fn state_item(initial: bool, state: usize, dark: bool, x: f64, y: f64) {
    let selected = remember(move || initial);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let color = if initial {
        scheme.primary
    } else {
        scheme.on_surface
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x - 24.0), Dp(y - 24.0))
                .size(Dp(48.0), Dp(48.0)),
        )
        .children(move || {
            if state == 1 || state == 3 {
                boxed().modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(
                            color.with_alpha(if state == 1 { 0.08 } else { 0.10 }),
                            Shape::Ellipse,
                        ),
                );
            }
            if state == 2 {
                boxed().modifier(Modifier::new().size(Dp(30.0), Dp(30.0)).border_with_shape(
                    Dp(2.0),
                    color,
                    Shape::Ellipse,
                ));
            }
            radio_button()
                .selected(selected.get())
                .enabled(state != 4)
                .accessibility_label(format!(
                    "{} {} {} radio",
                    if dark { "Dark" } else { "Light" },
                    if initial { "Selected" } else { "Unselected" },
                    [
                        "enabled",
                        "hover preview",
                        "focus preview",
                        "pressed preview",
                        "disabled"
                    ][state]
                ))
                .on_select(move |_| selected.set(true));
        });
}

#[tessera]
pub(super) fn measurements() {
    stage(
        1024.0,
        330.0,
        RenderSlot::new(|| {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            for (index, selected) in [true, false].into_iter().enumerate() {
                let x = 376.0 + index as f64 * 224.0;
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(x), Dp(64.0))
                            .size(Dp(48.0), Dp(48.0))
                            .border(Dp(1.0), scheme.outline),
                    )
                    .children(move || {
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(40.0), Dp(40.0))
                                .background_with_shape(
                                    scheme.primary.with_alpha(0.10),
                                    Shape::Ellipse,
                                ),
                        );
                        radio_button()
                            .selected(selected)
                            .accessibility_label(format!(
                                "{} radio dimensions",
                                if selected { "Selected" } else { "Unselected" }
                            ));
                    });
                for (row, length) in [20.0, 40.0, 48.0].into_iter().enumerate() {
                    spacer().modifier(
                        Modifier::new()
                            .offset(Dp(x + 24.0 - length * 0.5), Dp(148.0 + row as f64 * 50.0))
                            .size(Dp(length), Dp(1.0))
                            .background(scheme.outline),
                    );
                }
            }
            native_label("20dp icon".into(), 680.0, 134.0, 200.0);
            native_label("40dp state layer".into(), 680.0, 184.0, 200.0);
            native_label("48 × 48dp target".into(), 680.0, 234.0, 200.0);
        }),
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Icon diameter".into(), "20dp".into()],
            vec!["Stroke".into(), "2dp".into()],
            vec!["Selected dot".into(), "10dp".into()],
            vec!["State layer".into(), "40dp".into()],
            vec!["Minimum target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn stage(width: f64, height: f64, content: RenderSlot) {
    media::load_fonts();
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box((measured.get().0 / width) as f32).child(move || {
                boxed()
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(
                                scheme.surface,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                    )
                    .children_shared(content);
            });
        });
}

#[tessera]
fn native_label(value: String, x: f64, y: f64, width: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(width), Dp(32.0)),
        )
        .children(move || {
            text()
                .content(value.clone())
                .style(type_style(16.0, 24.0))
                .color(scheme.on_surface);
        });
}

#[tessera]
fn context(prose: RenderSlot, figure: RenderSlot) {
    let width = remember(|| Dp(1024.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            if width.get().0 < 760.0 {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        prose.render();
                        gap(Dp(24.0));
                        figure.render();
                    });
            } else {
                let track = (width.get().0 - 40.0) / 3.0;
                row().children(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(track)))
                        .children_shared(prose);
                    spacer().modifier(Modifier::new().width(Dp(20.0)));
                    column()
                        .modifier(Modifier::new().width(Dp(track * 2.0 + 20.0)))
                        .children_shared(figure);
                });
            }
        });
}
