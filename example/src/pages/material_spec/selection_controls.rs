//! Checkbox and switch specification examples.
//!
//! ## Usage
//!
//! Compare independent selection controls and their optional icons.

use tessera_components::{
    alignment::CrossAxisAlignment,
    button::button,
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlot, State, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    article::SpecSection,
    gap, panel,
    selection_articles::{Kind, option_label, selection_article},
    spec_table, type_style,
};

#[tessera]
pub(super) fn checkbox_spec() {
    selection_article(
        Kind::Checkbox,
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(|| {
                    tokens(false);
                }),
            },
            SpecSection {
                title: "Checkbox",
                content: RenderSlot::new(checkbox_anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(|| {
                    colors(false);
                }),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(|| {
                    states(false);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(|| {
                    measurements(false);
                }),
            },
        ],
        RenderSlot::new(|| {
            settings(false);
        }),
        RenderSlot::new(checkbox_anatomy),
    );
}

#[tessera]
pub(super) fn switch_spec() {
    selection_article(
        Kind::Switch,
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(|| {
                    tokens(true);
                }),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(|| {
                    colors(true);
                }),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(|| {
                    states(true);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(|| {
                    measurements(true);
                }),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
        ],
        RenderSlot::new(|| {
            settings(true);
        }),
        RenderSlot::new(configurations),
    );
}

#[tessera]
pub(super) fn selection_control(
    is_switch: bool,
    initial: bool,
    indeterminate: bool,
    enabled: bool,
    icons: usize,
    #[prop(into)] label: String,
    state: Option<State<bool>>,
    focus_requester: Option<FocusRequester>,
) {
    let checked = if let Some(state) = state {
        state
    } else {
        remember(|| initial)
    };
    let mixed = remember(|| indeterminate);
    if is_switch {
        let mut control = switch()
            .focus_requester_optional(focus_requester)
            .checked(checked.get())
            .enabled(enabled)
            .accessibility_label(label)
            .on_toggle(move |value| checked.set(value));
        if icons == 2 || (icons == 1 && checked.get()) {
            control = control.child(move || {
                icon().painter(if checked.get() {
                    filled::CHECK_SVG
                } else {
                    filled::CLOSE_SVG
                });
            });
        }
        drop(control);
    } else {
        checkbox()
            .focus_requester_optional(focus_requester)
            .checked(checked.get())
            .indeterminate(mixed.get())
            .disabled(!enabled)
            .accessibility_label(label)
            .on_toggle(move |value| {
                checked.set(value);
                mixed.set(false);
            });
    }
}

#[tessera]
fn setting_row(is_switch: bool, initial: bool, #[prop(into)] label: String) {
    let checked = remember(|| initial);
    let focus = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(72.0)))
        .children(move || {
            option_label(
                label.clone(),
                Callback::new(move || {
                    checked.with_mut(|checked| *checked = !*checked);
                    focus.request_focus();
                }),
                true,
            )
            .style(type_style(20.0, 28.0))
            .color(scheme.on_surface);
            spacer().modifier(Modifier::new().weight(1.0));
            selection_control(
                is_switch,
                initial,
                false,
                true,
                usize::from(is_switch),
                label.clone(),
            )
            .state(checked)
            .focus_requester(focus);
        });
}

#[tessera]
fn settings(is_switch: bool) {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().width(Dp(400.0)))
            .children(move || {
                text()
                    .content(if is_switch {
                        "Settings"
                    } else {
                        "Allow access"
                    })
                    .style(type_style(24.0, 32.0))
                    .color(INK);
                gap(Dp(24.0));
                if is_switch {
                    setting_row(true, true, "Wi-Fi");
                    setting_row(true, false, "Bluetooth");
                } else {
                    setting_row(false, true, "Microphone access");
                    setting_row(false, true, "Location access");
                    setting_row(false, false, "Haptics");
                }
            });
    }));
}

fn group_checked(items: State<[bool; 3]>, index: Option<usize>) -> bool {
    items.with(|items| {
        index.map_or_else(
            || items.iter().all(|selected| *selected),
            |index| items[index],
        )
    })
}

fn set_group_value(items: State<[bool; 3]>, index: Option<usize>, value: bool) {
    items.with_mut(|items| {
        if let Some(index) = index {
            items[index] = value;
        } else {
            items.fill(value);
        }
    });
}

#[tessera]
pub(super) fn checkbox_group() {
    let items = remember(|| [true, false, false]);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().width(Dp(400.0)).semantics(SemanticsArgs {
                role: Some(tessera_ui::accesskit::Role::Group),
                label: Some("Related checkbox choices".into()),
                merge_descendants: true,
                ..Default::default()
            }))
            .children(move || {
                checkbox_group_row(items, "All items".into());
                column()
                    .modifier(Modifier::new().padding_symmetric(Dp(24.0), Dp::ZERO))
                    .children(move || {
                        for (index, label) in
                            ["Documents", "Photos", "Videos"].into_iter().enumerate()
                        {
                            checkbox_group_row(items, label.into()).index(index);
                        }
                    });
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "Selected items: {}",
                        items.with(|items| items.iter().filter(|selected| **selected).count())
                    ))
                    .style(type_style(16.0, 24.0));
            });
    }));
}

#[tessera]
fn checkbox_group_row(items: State<[bool; 3]>, index: Option<usize>, label: String) {
    let focus = remember(FocusRequester::new).get();
    let checked = group_checked(items, index);
    let mixed =
        index.is_none() && !checked && items.with(|items| items.iter().any(|selected| *selected));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            checkbox()
                .checked(checked)
                .indeterminate(mixed)
                .accessibility_label(label.clone())
                .focus_requester(focus)
                .on_toggle(move |next| set_group_value(items, index, next));
            option_label(
                label.clone(),
                Callback::new(move || {
                    set_group_value(items, index, !group_checked(items, index));
                    focus.request_focus();
                }),
                true,
            );
        });
}

#[tessera]
fn tokens(is_switch: bool) {
    let dimensions = remember(|| false);
    flow_row().item_spacing(Dp(16.0)).children(move || {
        button()
            .text()
            .on_click(move || dimensions.set(false))
            .child(|| {
                text().content("Color tokens");
            });
        button()
            .text()
            .on_click(move || dimensions.set(true))
            .child(|| {
                text().content("Size tokens");
            });
    });
    gap(Dp(24.0));
    if dimensions.get() {
        measurement_table(is_switch);
    } else {
        spec_table(
            vec!["Element".into(), "Selected".into(), "Unselected".into()],
            if is_switch {
                vec![
                    vec![
                        "Track".into(),
                        "Primary".into(),
                        "Surface container highest".into(),
                    ],
                    vec!["Handle".into(), "On primary".into(), "Outline".into()],
                    vec![
                        "Icon".into(),
                        "On primary container".into(),
                        "Surface container highest".into(),
                    ],
                ]
            } else {
                vec![
                    vec!["Container".into(), "Primary".into(), "Transparent".into()],
                    vec![
                        "Outline".into(),
                        "Primary".into(),
                        "On surface variant".into(),
                    ],
                    vec!["Icon".into(), "On primary".into(), "—".into()],
                ]
            },
        );
    }
}

#[tessera]
fn checkbox_anatomy() {
    panel(RenderSlot::new(|| {
        column().children(|| {
            flow_row()
                .item_spacing(Dp(48.0))
                .line_spacing(Dp(16.0))
                .children(|| {
                    selection_control(false, false, false, true, 0, "Unselected checkbox");
                    selection_control(false, true, false, true, 0, "Selected checkbox");
                    selection_control(false, false, true, true, 0, "Indeterminate checkbox");
                });
            gap(Dp(24.0));
            text().content("1  Container").style(type_style(16.0, 28.0));
            text().content("2  Icon").style(type_style(16.0, 28.0));
        });
    }));
}

#[tessera]
fn color_scheme(is_switch: bool, dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            surface()
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(32.0)))
                        .children(move || {
                            text()
                                .content(if dark { "Dark scheme" } else { "Light scheme" })
                                .style(type_style(24.0, 32.0));
                            gap(Dp(24.0));
                            setting_row(is_switch, true, "Selected");
                            setting_row(is_switch, false, "Unselected");
                        });
                });
        });
}

#[tessera]
fn colors(is_switch: bool) {
    color_scheme(is_switch, false);
    gap(Dp(24.0));
    color_scheme(is_switch, true);
    gap(Dp(32.0));
    text()
        .content("Adjacent labels use On surface in either selection state.")
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn states(is_switch: bool) {
    text()
        .content("Hover a control, focus with Tab, or press Space to inspect the actual feedback.")
        .style(type_style(16.0, 24.0))
        .color(INK);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            for enabled in [true, false] {
                text()
                    .content(if enabled { "Enabled" } else { "Disabled" })
                    .style(type_style(24.0, 32.0));
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        selection_control(is_switch, false, false, enabled, 0, "Unselected");
                        selection_control(is_switch, true, false, enabled, 0, "Selected");
                    });
                gap(Dp(32.0));
            }
        });
    }));
}

#[tessera]
fn measurement_table(is_switch: bool) {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        if is_switch {
            vec![
                vec!["Track size".into(), "52 × 32dp".into()],
                vec!["Track outline".into(), "2dp".into()],
                vec!["Unselected handle".into(), "16dp".into()],
                vec!["Selected handle / handle with icon".into(), "24dp".into()],
                vec!["Pressed handle".into(), "28dp".into()],
                vec!["Icon".into(), "16dp".into()],
                vec!["State layer".into(), "40dp".into()],
                vec!["Target".into(), "48dp".into()],
            ]
        } else {
            vec![
                vec!["Container".into(), "18 × 18dp".into()],
                vec!["Corner radius".into(), "2dp".into()],
                vec!["Icon".into(), "18dp · centered".into()],
                vec!["State layer".into(), "40dp".into()],
                vec!["Target".into(), "48 × 48dp".into()],
            ]
        },
    );
}

#[tessera]
fn measurements(is_switch: bool) {
    panel(RenderSlot::new(move || {
        column().children(move || {
            surface()
                .style(SurfaceStyle::Outlined {
                    color: ACCENT,
                    width: Dp(1.0),
                })
                .modifier(
                    Modifier::new().size(if is_switch { Dp(52.0) } else { Dp(48.0) }, Dp(48.0)),
                )
                .child(move || {
                    selection_control(is_switch, true, false, true, 0, "Measured control");
                });
            gap(Dp(24.0));
            text()
                .content(if is_switch {
                    "52 × 48dp interaction target"
                } else {
                    "48 × 48dp interaction target"
                })
                .style(type_style(16.0, 24.0));
        });
    }));
    gap(Dp(32.0));
    measurement_table(is_switch);
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(|| {
        column().children(|| {
            for (icons, label) in [
                "Without icons",
                "Icon when selected",
                "Icons in both states",
            ]
            .into_iter()
            .enumerate()
            {
                text().content(label).style(type_style(20.0, 28.0));
                gap(Dp(16.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        selection_control(
                            true,
                            false,
                            false,
                            true,
                            icons,
                            format!("{label}: unselected"),
                        );
                        selection_control(
                            true,
                            true,
                            false,
                            true,
                            icons,
                            format!("{label}: selected"),
                        );
                    });
                gap(Dp(32.0));
            }
        });
    }));
}
