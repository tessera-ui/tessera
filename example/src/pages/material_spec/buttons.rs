//! Interactive Expressive button specifications.
//!
//! ## Usage
//!
//! Compare button selection, sizes, shapes, colors and input feedback.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _, remember, tessera,
    use_context,
};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const STYLES: [ButtonVariant; 5] = [
    ButtonVariant::Elevated,
    ButtonVariant::Filled,
    ButtonVariant::Tonal,
    ButtonVariant::Outlined,
    ButtonVariant::Text,
];
const SIZES: [ButtonSize; 5] = [
    ButtonSize::ExtraSmall,
    ButtonSize::Small,
    ButtonSize::Medium,
    ButtonSize::Large,
    ButtonSize::ExtraLarge,
];

fn style_name(style: ButtonVariant) -> &'static str {
    match style {
        ButtonVariant::Elevated => "Elevated",
        ButtonVariant::Filled => "Filled",
        ButtonVariant::Tonal => "Tonal",
        ButtonVariant::Outlined => "Outlined",
        ButtonVariant::Text => "Text",
    }
}

#[tessera]
pub(super) fn buttons_spec() {
    article(
        "Buttons",
        "Buttons prompt most actions in a UI",
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Shape morph",
                content: RenderSlot::new(shape_morph),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "Baseline tokens",
                content: RenderSlot::new(baseline_tokens),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Toggle buttons",
                    content: RenderSlot::new(toggle_buttons),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "Color styles",
                    content: RenderSlot::new(color_styles),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
pub(super) fn button_preview(
    style: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    toggle: bool,
    enabled: bool,
    #[prop(into)] label: String,
    initial_selected: Option<bool>,
    with_icon: Option<bool>,
) {
    let selected = remember(|| initial_selected.unwrap_or(false));
    let activations = remember(|| 0_u32);
    button()
        .variant(style)
        .size(size)
        .shape_style(shape)
        .selected_optional(if toggle && style != ButtonVariant::Text {
            Some(selected.get())
        } else {
            None
        })
        .enabled(enabled)
        .accessibility_label(label.clone())
        .accessibility_description(format!("Activations: {}", activations.get()))
        .leading_icon_optional(with_icon.unwrap_or(false).then(|| {
            RenderSlot::new(|| {
                icon().painter(filled::EDIT_SVG);
            })
        }))
        .on_click(move || {
            activations.set(activations.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        })
        .child(move || {
            text().content(label.clone());
        });
}

#[tessera]
fn overview_art() {
    let width = remember(|| 1024.0_f64);
    let scale = width.get() / 1024.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(300.0 * scale))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32()).0;
                    if width.is_alive() && width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            for (index, style) in STYLES.into_iter().enumerate() {
                let center = [159.0, 342.0, 515.0, 699.0, 879.0][index];
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp((center - 80.0) * scale), Dp(76.0 * scale))
                            .size(Dp(160.0 * scale), Dp(80.0 * scale)),
                    )
                    .children(move || {
                        overview_button(style, index, scale);
                    });
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp((center - 20.0) * scale), Dp(168.0 * scale))
                            .size(Dp(40.0 * scale), Dp(40.0 * scale))
                            .background_with_shape(scheme.on_surface, Shape::Ellipse)
                            .semantics(SemanticsArgs {
                                role: Some(Role::Note),
                                label: Some(format!("{} {} button", index + 1, style_name(style))),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        text()
                            .content((index + 1).to_string())
                            .style(type_style(16.0 * scale, 24.0 * scale))
                            .color(scheme.surface);
                    });
            }
        });
    gap(Dp(24.0));
    for (index, caption) in [
        "Elevated button",
        "Filled button",
        "Filled tonal button",
        "Outlined button",
        "Text button",
    ]
    .into_iter()
    .enumerate()
    {
        text()
            .content(format!("{}  {caption}", index + 1))
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn overview_button(style: ButtonVariant, index: usize, scale: f64) {
    let actions = remember(|| 0_u32);
    let mut label_style = type_style(14.0 * scale, 20.0 * scale);
    label_style.font_weight = 500;
    button()
        .variant(style)
        .size(ButtonSize::Small)
        .shape_style(ButtonShape::Round)
        .content_padding(Padding::default())
        .modifier(Modifier::new().size(
            Dp([90.0, 68.0, 68.0, 90.0, 64.0][index] * scale),
            Dp(40.0 * scale),
        ))
        .border_width(Dp(if style == ButtonVariant::Outlined {
            scale
        } else {
            0.0
        }))
        .accessibility_label(format!("Overview {} button", style_name(style)))
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || actions.set(actions.get() + 1))
        .child(move || {
            text().content(style_name(style)).style(label_style);
        });
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                column().children(|| {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Medium,
                        ButtonShape::Round,
                        false,
                        true,
                        "Default button",
                    );
                    gap(Dp(24.0));
                    text()
                        .content("1  Default button")
                        .style(type_style(16.0, 24.0));
                });
                column().children(|| {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Medium,
                        ButtonShape::Round,
                        true,
                        true,
                        "Toggle button",
                    );
                    gap(Dp(24.0));
                    text()
                        .content("2  Toggle button")
                        .style(type_style(16.0, 24.0));
                });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["Default".into(), "Available".into(), "Available".into()],
            vec!["Toggle (selection)".into(), "--".into(), "Available".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "A default button performs an action. A toggle button keeps a selected state: the label names the option, and selection is shown by container color, content color, and shape.",
    );
}

#[tessera]
fn size_examples() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            for (index, size) in SIZES.into_iter().enumerate() {
                text()
                    .content(["Extra small", "Small", "Medium", "Large", "Extra large"][index])
                    .style(type_style(16.0, 24.0));
                gap(Dp(16.0));
                button_preview(
                    ButtonVariant::Filled,
                    size,
                    ButtonShape::Round,
                    false,
                    true,
                    "Label",
                );
                gap(Dp(32.0));
            }
        });
}

#[tessera]
fn configurations() {
    prose(
        "Choose a size, shape, and color style. Small is the default size; round is the default shape. The 16dp horizontal padding of small buttons is recommended, while the earlier 24dp padding is no longer recommended.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("1  Size".into());
                gap(Dp(16.0));
                size_examples();
                gap(Dp(32.0));
                subheading("2  Shape".into());
                gap(Dp(16.0));
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(16.0))
                    .children(|| {
                        for shape in [ButtonShape::Round, ButtonShape::Square] {
                            button_preview(
                                ButtonVariant::Filled,
                                ButtonSize::Medium,
                                shape,
                                false,
                                true,
                                if shape == ButtonShape::Round {
                                    "Round"
                                } else {
                                    "Square"
                                },
                            );
                        }
                    });
                gap(Dp(32.0));
                subheading("3  Color".into());
                gap(Dp(16.0));
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(|| {
                        for style in STYLES {
                            button_preview(
                                style,
                                ButtonSize::Medium,
                                ButtonShape::Round,
                                false,
                                true,
                                style_name(style),
                            );
                        }
                    });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec![
            "Category".into(),
            "Configuration".into(),
            "M3".into(),
            "M3 Expressive".into(),
        ],
        vec![
            vec![
                "Size".into(),
                "Small (default)".into(),
                "Available".into(),
                "Available".into(),
            ],
            vec![
                "Size".into(),
                "XS, M, L, XL".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Shape".into(),
                "Round (default)".into(),
                "Available".into(),
                "Available".into(),
            ],
            vec![
                "Shape".into(),
                "Square".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Color".into(),
                "Elevated, filled (default), tonal, outlined, text".into(),
                "Available".into(),
                "Available".into(),
            ],
            vec![
                "Small button padding".into(),
                "24dp".into(),
                "Available".into(),
                "Not recommended. Use 16dp".into(),
            ],
            vec![
                "Small button padding".into(),
                "16dp".into(),
                "--".into(),
                "Available".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("Live configuration".into());
    gap(Dp(8.0));
    prose(
        "Change the configuration and activate the button. Focus it with Tab and activate it with Space or Enter to check keyboard and assistive-technology behavior.",
    );
    gap(Dp(24.0));
    playground();
}

#[tessera]
fn tokens() {
    let style = remember(|| ButtonVariant::Filled);
    prose(
        "Select a color style to inspect its container, content, border, elevation, typography, and shape roles. Values resolve from the active Material theme, so the same roles map to different color values in light and dark schemes.",
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for variant in STYLES {
                button()
                    .tonal()
                    .selected(style.get() == variant)
                    .accessibility_label(format!("{} color tokens", style_name(variant)))
                    .on_click(move || style.set(variant))
                    .child(move || {
                        text().content(style_name(variant));
                    });
            }
        });
    gap(Dp(24.0));
    let mut rows = match style.get() {
        ButtonVariant::Elevated => vec![
            vec!["Container".into(), "Surface container low".into()],
            vec!["Icon and label".into(), "Primary".into()],
            vec!["Border".into(), "None".into()],
            vec!["Elevation".into(), "1dp enabled · 0dp disabled".into()],
        ],
        ButtonVariant::Filled => vec![
            vec!["Container".into(), "Primary".into()],
            vec!["Icon and label".into(), "On primary".into()],
            vec!["Border".into(), "None".into()],
            vec!["Elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Tonal => vec![
            vec!["Container".into(), "Secondary container".into()],
            vec!["Icon and label".into(), "On secondary container".into()],
            vec!["Border".into(), "None".into()],
            vec!["Elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Outlined => vec![
            vec!["Container".into(), "Transparent".into()],
            vec!["Icon and label".into(), "On surface variant".into()],
            vec!["Border".into(), "Outline variant · 1dp".into()],
            vec!["Elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Text => vec![
            vec!["Container".into(), "Transparent".into()],
            vec!["Icon and label".into(), "Primary".into()],
            vec!["Border".into(), "None".into()],
            vec!["Elevation".into(), "0dp".into()],
        ],
    };
    rows.push(vec![
        "Label type".into(),
        "Label large · 14sp / 20sp".into(),
    ]);
    rows.push(vec![
        "Shape".into(),
        "Full capsule; size-specific corners when square".into(),
    ]);
    rows.push(vec![
        "Height".into(),
        "40dp for the default small size".into(),
    ]);
    rows.push(vec!["Disabled label opacity".into(), "38%".into()]);
    spec_table(vec!["Element".into(), "Token value".into()], rows);
    gap(Dp(32.0));
    subheading("Toggle colors".into());
    gap(Dp(16.0));
    prose(
        "Toggle buttons change their container and content roles with selection. Text buttons do not support selection, so they keep the default text colors.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Color style".into(),
            "Toggle unselected".into(),
            "Toggle selected".into(),
        ],
        vec![
            vec![
                "Elevated".into(),
                "Surface container low · primary".into(),
                "Primary · on primary".into(),
            ],
            vec![
                "Filled".into(),
                "Surface container · on surface variant".into(),
                "Primary · on primary".into(),
            ],
            vec![
                "Tonal".into(),
                "Secondary container · on secondary container".into(),
                "Secondary · on secondary".into(),
            ],
            vec![
                "Outlined".into(),
                "Outline variant · on surface variant".into(),
                "Inverse surface · inverse on surface".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        column().children(|| {
            button_preview(
                ButtonVariant::Filled,
                ButtonSize::Large,
                ButtonShape::Round,
                false,
                true,
                "Label",
            )
            .with_icon(true);
            gap(Dp(40.0));
            for label in ["1  Container", "2  Label text", "3  Icon (optional)"] {
                text().content(label).style(type_style(16.0, 28.0));
            }
        });
    }));
}

#[tessera]
fn color_examples(dark: bool) {
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
                            flow_row()
                                .item_spacing(Dp(16.0))
                                .line_spacing(Dp(16.0))
                                .children(|| {
                                    for style in STYLES {
                                        button_preview(
                                            style,
                                            ButtonSize::Small,
                                            ButtonShape::Round,
                                            false,
                                            true,
                                            style_name(style),
                                        );
                                    }
                                });
                            gap(Dp(32.0));
                            text()
                                .content("Toggle: unselected and selected")
                                .style(type_style(16.0, 24.0));
                            gap(Dp(16.0));
                            flow_row()
                                .item_spacing(Dp(16.0))
                                .line_spacing(Dp(16.0))
                                .children(|| {
                                    for style in STYLES
                                        .into_iter()
                                        .filter(|style| *style != ButtonVariant::Text)
                                    {
                                        for selected in [false, true] {
                                            button_preview(
                                                style,
                                                ButtonSize::Small,
                                                ButtonShape::Round,
                                                true,
                                                true,
                                                style_name(style),
                                            )
                                            .initial_selected(selected);
                                        }
                                    }
                                });
                        });
                });
        });
}

#[tessera]
fn colors() {
    color_examples(false);
    gap(Dp(24.0));
    color_examples(true);
}

#[tessera]
fn states() {
    for style in STYLES {
        subheading(format!("{} button style states", style_name(style)));
        gap(Dp(24.0));
        subheading("Default".into());
        gap(Dp(24.0));
        button_state_matrix(style, false);
        if style != ButtonVariant::Text {
            gap(Dp(40.0));
            subheading("Toggle".into());
            gap(Dp(24.0));
            button_state_matrix(style, true);
        }
        gap(Dp(64.0));
    }
}

#[tessera]
fn button_state_matrix(style: ButtonVariant, toggle: bool) {
    let width = remember(|| 1024.0_f64);
    let scale = width.get() / 1024.0;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(600.0 * scale))
                .clip_to_bounds()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32()).0;
                    if width.is_alive() && width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(move || {
                        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp::ZERO, Dp(if dark { 300.0 } else { 0.0 } * scale))
                                    .size(Dp(1024.0 * scale), Dp(300.0 * scale))
                                    .background(scheme.surface_container_low),
                            )
                            .children(move || {
                                for state in 0..5 {
                                    let center = if toggle {
                                        [169.0, 353.0, 537.0, 722.0, 909.0][state]
                                    } else {
                                        [144.0, 328.0, 512.0, 696.0, 880.0][state]
                                    };
                                    button_state_marker(
                                        (state + 1).to_string(),
                                        center,
                                        if toggle {
                                            if dark { 74.0 } else { 79.0 }
                                        } else {
                                            116.0
                                        },
                                        scale,
                                    );
                                    for selected in
                                        [false, true].into_iter().take(if toggle { 2 } else { 1 })
                                    {
                                        let mut center_y = if toggle {
                                            if selected { 208.0 } else { 136.0 }
                                        } else {
                                            172.0
                                        };
                                        if toggle && dark {
                                            center_y -= 4.0;
                                        }
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(
                                                Modifier::new()
                                                    .offset(
                                                        Dp((center - 80.0) * scale),
                                                        Dp((center_y - 40.0) * scale),
                                                    )
                                                    .size(Dp(160.0 * scale), Dp(80.0 * scale)),
                                            )
                                            .children(move || {
                                                button_state_item(
                                                    style,
                                                    toggle,
                                                    selected,
                                                    state,
                                                    scale,
                                                    format!(
                                                        "{} {} {} {} {}",
                                                        if dark { "Dark" } else { "Light" },
                                                        style_name(style),
                                                        if toggle { "toggle" } else { "default" },
                                                        if selected {
                                                            "selected"
                                                        } else {
                                                            "unselected"
                                                        },
                                                        [
                                                            "enabled", "disabled", "hovered",
                                                            "focused", "pressed"
                                                        ][state]
                                                    ),
                                                );
                                            });
                                    }
                                }
                                if toggle {
                                    button_state_marker(
                                        "A".into(),
                                        86.0,
                                        if dark { 132.0 } else { 136.0 },
                                        scale,
                                    );
                                    button_state_marker(
                                        "B".into(),
                                        86.0,
                                        if dark { 204.0 } else { 208.0 },
                                        scale,
                                    );
                                }
                            });
                    });
            }
        });
    gap(Dp(24.0));
    if toggle {
        text()
            .content("A. Unselected, B. Selected")
            .style(type_style(16.0, 24.0));
    }
    for (index, label) in ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"]
        .into_iter()
        .enumerate()
    {
        text()
            .content(format!("{}  {label}", index + 1))
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn button_state_marker(label: String, x: f64, y: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((x - 12.0) * scale), Dp((y - 12.0) * scale))
                .size(Dp(24.0 * scale), Dp(24.0 * scale))
                .background_with_shape(scheme.on_surface, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(label.clone())
                .style(type_style(14.0 * scale, 20.0 * scale))
                .color(scheme.surface);
        });
}

#[tessera]
fn button_state_item(
    style: ButtonVariant,
    toggle: bool,
    initial: bool,
    state: usize,
    scale: f64,
    name: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    let name = remember(|| name);
    let mut label_style = type_style(14.0 * scale, 20.0 * scale);
    label_style.font_weight = 500;
    button()
        .variant(style)
        .size(ButtonSize::Small)
        .shape_style(ButtonShape::Round)
        .selected_optional(if toggle && style != ButtonVariant::Text {
            Some(selected.get())
        } else {
            None
        })
        .enabled(state != 1)
        .presentation_state(match state {
            2 => InteractionVisualState::Hover,
            3 => InteractionVisualState::Focus,
            4 => InteractionVisualState::Pressed,
            _ => InteractionVisualState::Rest,
        })
        .content_padding(Padding::default())
        .modifier(Modifier::new().size(
            Dp([84.0, 90.0, 86.0, 86.0, 84.0][state] * scale),
            Dp(40.0 * scale),
        ))
        .border_width(Dp(if style == ButtonVariant::Outlined {
            scale
        } else {
            0.0
        }))
        .accessibility_label(name.get())
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || {
            actions.set(actions.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        })
        .child(move || {
            text()
                .content(["Enabled", "Disabled", "Hovered", "Focused", "Pressed"][state])
                .style(label_style);
        });
}

#[tessera]
fn shape_morph() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for shape in [ButtonShape::Round, ButtonShape::Square] {
                    column().children(move || {
                        button_preview(
                            ButtonVariant::Filled,
                            ButtonSize::Large,
                            shape,
                            false,
                            true,
                            "Press and hold",
                        );
                        gap(Dp(32.0));
                        button_preview(
                            ButtonVariant::Filled,
                            ButtonSize::Large,
                            shape,
                            true,
                            true,
                            "Select",
                        );
                    });
                }
            });
    }));
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(size_examples));
    gap(Dp(48.0));
    spec_table(
        ["Attribute", "XS", "S", "M", "L", "XL"]
            .map(str::to_owned)
            .to_vec(),
        vec![
            ["Round corners", "Full", "Full", "Full", "Full", "Full"]
                .map(str::to_owned)
                .to_vec(),
            ["Square corners", "12dp", "12dp", "16dp", "28dp", "28dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Pressed corners", "8dp", "8dp", "12dp", "16dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Container height", "32dp", "40dp", "56dp", "96dp", "136dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Icon size", "20dp", "20dp", "24dp", "32dp", "40dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Icon-label spacing", "8dp", "8dp", "8dp", "12dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Outline width", "1dp", "1dp", "1dp", "2dp", "3dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
    gap(Dp(24.0));
    prose(
        "Pressed corners apply to both shape presets while the button is held. Round buttons return to fully rounded ends; square buttons return to their size-specific radius. Icon and label shapes animate together so the content keeps its alignment.",
    );
    gap(Dp(24.0));
    text()
        .content("Minimum interaction target: 48 × 48dp.")
        .style(type_style(16.0, 24.0))
        .color(INK);
}

fn next_style(style: ButtonVariant) -> ButtonVariant {
    match style {
        ButtonVariant::Elevated => ButtonVariant::Filled,
        ButtonVariant::Filled => ButtonVariant::Tonal,
        ButtonVariant::Tonal => ButtonVariant::Outlined,
        ButtonVariant::Outlined => ButtonVariant::Text,
        ButtonVariant::Text => ButtonVariant::Elevated,
    }
}

fn next_size(size: ButtonSize) -> ButtonSize {
    match size {
        ButtonSize::ExtraSmall => ButtonSize::Small,
        ButtonSize::Small => ButtonSize::Medium,
        ButtonSize::Medium => ButtonSize::Large,
        ButtonSize::Large => ButtonSize::ExtraLarge,
        ButtonSize::ExtraLarge => ButtonSize::ExtraSmall,
    }
}

fn size_name(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::ExtraSmall => "Extra small",
        ButtonSize::Small => "Small",
        ButtonSize::Medium => "Medium",
        ButtonSize::Large => "Large",
        ButtonSize::ExtraLarge => "Extra large",
    }
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children(move || {
                        spacer().modifier(
                            Modifier::new()
                                .size(Dp(8.0), Dp(8.0))
                                .background_with_shape(scheme.on_surface_variant, Shape::Ellipse),
                        );
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text().content(item.clone()).style(type_style(16.0, 24.0));
                    });
            }
        });
}

#[tessera]
fn overview() {
    prose(
        "Buttons prompt most actions in a UI. Choose a color style that matches the importance of the action, and keep the label short enough to read at a glance.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Two variants: default and toggle".into(),
        "Can contain an optional leading icon".into(),
        "Five color styles: elevated, filled, tonal, outlined, and text".into(),
        "Five sizes: extra small, small, medium, large, and extra large".into(),
        "Two shapes: round and square".into(),
        "Keep labels concise and use sentence case".into(),
    ]);
    gap(Dp(32.0));
    overview_art();
}

#[tessera]
fn resources() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Available".into()],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native button component. The five color styles, five sizes, both shape presets, optional leading and trailing icons, and toggle selection with animated shape morphing are implemented; states, elevation, and color roles follow the token values shown on this page.",
    );
}

#[tessera]
fn expressive() {
    prose(
        "May 2025: buttons gained a wider variety of shapes and sizes, toggle functionality, and a shape change when selected.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Change".into(), "Details".into()],
        vec![
            vec![
                "Variants and naming".into(),
                "Default and toggle (selection); color styles are now configurations".into(),
            ],
            vec![
                "Shapes".into(),
                "Round and square; the shape morphs when pressed and when selected".into(),
            ],
            vec![
                "Sizes".into(),
                "Extra small, small (default), medium, large, and extra large".into(),
            ],
            vec![
                "Small button padding".into(),
                "16dp is recommended; the earlier 24dp padding is no longer recommended".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    size_figure();
    caption("Five sizes of the filled color style, from extra small to extra large.");
}

#[tessera]
fn size_figure() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(|| {
                for size in SIZES {
                    button_preview(
                        ButtonVariant::Filled,
                        size,
                        ButtonShape::Round,
                        false,
                        true,
                        size_name(size),
                    );
                }
            });
    }));
}

#[tessera]
fn differences() {
    prose(
        "Color: new color mappings support dynamic color, and icons and labels now share the same color. The neutral text button is no longer recommended.",
    );
    gap(Dp(16.0));
    prose(
        "Sizes: buttons gained four additional sizes while small remains the default. Icons: the standard leading and trailing icon size is 20dp. Shape: buttons are fully rounded by default and can use square corners, morphing on press and selection.",
    );
    gap(Dp(24.0));
    m2_m3_figure();
    gap(Dp(16.0));
    for (index, label) in [
        "M2: 36dp height with slightly rounded corners",
        "M3: 40dp height with fully rounded corners",
    ]
    .into_iter()
    .enumerate()
    {
        text()
            .content(format!("{}  {label}", index + 1))
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn m2_m3_figure() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                column().children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .height(Dp(36.0))
                                .padding_symmetric(Dp(16.0), Dp::ZERO)
                                .background_with_shape(
                                    scheme.primary,
                                    Shape::rounded_rectangle(Dp(4.0)),
                                ),
                        )
                        .children(move || {
                            text()
                                .content("Label")
                                .color(scheme.on_primary)
                                .style(type_style(14.0, 20.0));
                        });
                    gap(Dp(12.0));
                    text().content("M2 button").style(type_style(14.0, 20.0));
                });
                column().children(|| {
                    button()
                        .filled()
                        .size(ButtonSize::Small)
                        .accessibility_label("M3 button comparison")
                        .on_click(|| {})
                        .child(|| {
                            text().content("Label");
                        });
                    gap(Dp(12.0));
                    text().content("M3 button").style(type_style(14.0, 20.0));
                });
            });
    }));
}

#[tessera]
fn usage() {
    prose(
        "Buttons communicate actions that people can take. They are typically placed throughout the UI, in places like dialogs, modal windows, forms, cards, and toolbars. They can also be placed within standard button groups.",
    );
    gap(Dp(24.0));
    usage_example();
    caption("Use visually prominent filled buttons for the most important actions");
    gap(Dp(32.0));
    prose(
        "Buttons are just one option for representing actions in a product and shouldn't be overused. Too many buttons on a screen can disrupt the visual hierarchy. Consider placing additional actions in a navigation rail, a set of icon buttons, or an overflow menu.",
    );
    gap(Dp(24.0));
    emphasis_comparison();
    caption(
        "One filled button keeps the primary action clear when several actions share the screen.",
    );
}

#[tessera]
fn usage_example() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Appointment - Thea")
                    .style(type_style(24.0, 32.0));
                gap(Dp(8.0));
                text()
                    .content("Your host has not yet joined")
                    .style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(|| {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Ask to join")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Ask to join");
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Share details")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Share details");
                            });
                    });
            });
    }));
}

#[tessera]
fn emphasis_comparison() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(|| {
                for style in STYLES {
                    button_preview(
                        style,
                        ButtonSize::Small,
                        ButtonShape::Round,
                        false,
                        true,
                        style_name(style),
                    );
                }
            });
    }));
}

#[tessera]
fn toggle_buttons() {
    prose(
        "Toggle buttons let people select and deselect an option. The label states the option, and the selected state is communicated by container color, content color, and shape. Elevated, filled, tonal, and outlined color styles support selection; text buttons do not.",
    );
    gap(Dp(24.0));
    toggle_example();
    caption(
        "A selected toggle button changes its container color and moves from fully rounded ends to the size-specific square shape.",
    );
    gap(Dp(32.0));
    spec_table(
        vec!["Interaction".into(), "Result".into()],
        vec![
            vec![
                "Click or tap an unselected toggle".into(),
                "Selects the option".into(),
            ],
            vec![
                "Click or tap a selected toggle".into(),
                "Clears the selection".into(),
            ],
            vec![
                "Press and hold".into(),
                "Morphs to the pressed corner radius until release".into(),
            ],
            vec![
                "Text color style".into(),
                "No selection state; performs an action only".into(),
            ],
        ],
    );
}

#[tessera]
fn toggle_example() {
    let filled = remember(|| false);
    let tonal = remember(|| true);
    let outlined = remember(|| true);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .selected(filled.get())
                            .accessibility_label("Filled toggle")
                            .on_click(move || filled.set(!filled.get()))
                            .child(|| {
                                text().content("Filled");
                            });
                        button()
                            .tonal()
                            .size(ButtonSize::Medium)
                            .selected(tonal.get())
                            .accessibility_label("Tonal toggle")
                            .on_click(move || tonal.set(!tonal.get()))
                            .child(|| {
                                text().content("Tonal");
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .selected(outlined.get())
                            .accessibility_label("Outlined toggle")
                            .on_click(move || outlined.set(!outlined.get()))
                            .child(|| {
                                text().content("Outlined");
                            });
                    });
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "Filled: {} · Tonal: {} · Outlined: {}",
                        if filled.get() {
                            "selected"
                        } else {
                            "unselected"
                        },
                        if tonal.get() {
                            "selected"
                        } else {
                            "unselected"
                        },
                        if outlined.get() {
                            "selected"
                        } else {
                            "unselected"
                        },
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "A button is made of a container, a label, and an optional icon. The container communicates emphasis through color, elevation, and outline; the icon and label share the same content color.",
    );
    gap(Dp(24.0));
    anatomy();
    gap(Dp(24.0));
    prose(
        "Icons are optional and usually lead the label. Keep the label to a single line where possible and use sentence case rather than all caps.",
    );
}

#[tessera]
fn color_styles() {
    prose(
        "Five color styles express emphasis, in order: elevated, filled, tonal (filled tonal), outlined, and text. Use one prominent style for the primary action and lower-emphasis styles for supporting actions.",
    );
    gap(Dp(24.0));
    emphasis_matrix();
    caption(
        "A. Default, B. Toggle unselected, and C. Toggle selected for five color styles, in order of emphasis.",
    );
    gap(Dp(32.0));
    spec_table(
        vec!["Emphasis".into(), "Color style".into()],
        vec![
            vec!["High".into(), "Filled".into()],
            vec!["Medium".into(), "Elevated, tonal, outlined".into()],
            vec!["Low".into(), "Text".into()],
        ],
    );
}

#[tessera]
fn emphasis_matrix() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                flow_row()
                    .item_spacing(Dp(40.0))
                    .line_spacing(Dp(16.0))
                    .children(|| {
                        for header in ["A. Default", "B. Toggle unselected", "C. Toggle selected"] {
                            text().content(header).style(type_style(14.0, 20.0));
                        }
                    });
                gap(Dp(16.0));
                for style in STYLES {
                    flow_row()
                        .item_spacing(Dp(40.0))
                        .line_spacing(Dp(16.0))
                        .children(move || {
                            button_preview(
                                style,
                                ButtonSize::Small,
                                ButtonShape::Round,
                                false,
                                true,
                                style_name(style),
                            );
                            if style == ButtonVariant::Text {
                                text().content("--").style(type_style(14.0, 20.0));
                                text().content("--").style(type_style(14.0, 20.0));
                            } else {
                                button_preview(
                                    style,
                                    ButtonSize::Small,
                                    ButtonShape::Round,
                                    true,
                                    true,
                                    style_name(style),
                                )
                                .initial_selected(false);
                                button_preview(
                                    style,
                                    ButtonSize::Small,
                                    ButtonShape::Round,
                                    true,
                                    true,
                                    style_name(style),
                                )
                                .initial_selected(true);
                            }
                        });
                    gap(Dp(12.0));
                }
            });
    }));
}

#[tessera]
fn adaptive() {
    prose(
        "Buttons can change size for different window classes. Choose a size that matches the density of the surrounding layout: extra small and small suit compact toolbars, while large and extra large give primary actions more presence in wide layouts.",
    );
    gap(Dp(24.0));
    size_figure();
    caption(
        "The same label at five sizes. Container height and icon size grow together, and the pressed corner radius scales with the size.",
    );
    gap(Dp(32.0));
    prose(
        "Keep labels short so buttons reflow without truncation. When a label needs more than one line, prefer a wider container or a shorter string rather than shrinking the text.",
    );
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Use a button to perform an action".into(),
        "Navigate to and activate a button".into(),
    ]);
    gap(Dp(24.0));
    activation_demo();
    caption(
        "The native button exposes an accessible label and a description that reports its activation count. Activate it with a click, or focus it with Tab and press Space or Enter.",
    );
}

#[tessera]
fn activation_demo() {
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label("Perform action")
                    .accessibility_description(format!("Activations: {}", activations.get()))
                    .on_click(move || activations.set(activations.get() + 1))
                    .child(|| {
                        text().content("Perform action");
                    });
                gap(Dp(16.0));
                text()
                    .content(format!("Activations: {}", activations.get()))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn interaction_style() {
    subheading("Color contrast".into());
    gap(Dp(16.0));
    prose(
        "Enabled buttons need a 3:1 contrast ratio with the background to meet accessibility best practices. This is measured from the container for elevated, filled, and tonal color styles, and from the label text for outlined and text color styles.",
    );
    gap(Dp(24.0));
    contrast_examples();
    caption("Higher contrast helps differentiate elements.");
    gap(Dp(48.0));
    subheading("200% text size".into());
    gap(Dp(16.0));
    prose(
        "Avoid excessive text wrapping or truncation by choosing concise strings. Keep labels short enough to fit within two lines after the text size is increased to 200%.",
    );
    gap(Dp(24.0));
    text_scale_example();
    gap(Dp(48.0));
    subheading("Rapid clicks".into());
    gap(Dp(16.0));
    prose(
        "On the web, a modified motion curve avoids resonant effects from overlapping animations. This provides a smoother experience for interactions where multiple clicks or taps are expected in succession.",
    );
    gap(Dp(24.0));
    rapid_click_example();
}

#[tessera]
fn contrast_examples() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                column().children(move || {
                    button()
                        .filled()
                        .size(ButtonSize::Medium)
                        .accessibility_label("Above three to one")
                        .on_click(|| {})
                        .child(|| {
                            text().content("Share");
                        });
                    gap(Dp(12.0));
                    text()
                        .content("Above 3:1 — primary container on surface")
                        .style(type_style(14.0, 20.0));
                });
                column().children(move || {
                    button()
                        .size(ButtonSize::Medium)
                        .color(scheme.surface_container)
                        .content_color(scheme.outline)
                        .accessibility_label("Below three to one")
                        .on_click(|| {})
                        .child(|| {
                            text().content("Share");
                        });
                    gap(Dp(12.0));
                    text()
                        .content("Below 3:1 — low-contrast container and label")
                        .style(type_style(14.0, 20.0));
                });
            });
    }));
}

#[tessera]
fn text_scale_example() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label("Save")
                    .on_click(|| {})
                    .child(|| {
                        text().content("Save");
                    });
                gap(Dp(12.0));
                prose("A concise label keeps its single-line shape when text is scaled up.");
                gap(Dp(24.0));
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label("Save the current document to your device")
                    .on_click(|| {})
                    .child(|| {
                        text().content("Save the current document to your device");
                    });
                gap(Dp(12.0));
                prose("A long label wraps or truncates once the text size grows. Prefer a shorter string or a wider container.");
            });
    }));
}

#[tessera]
fn rapid_click_example() {
    let clicks = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .size(ButtonSize::Medium)
                    .accessibility_label("Skip forward")
                    .accessibility_description(format!("Activations: {}", clicks.get()))
                    .on_click(move || clicks.set(clicks.get() + 1))
                    .child(|| {
                        text().content("Skip forward");
                    });
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "Press repeatedly to check the shape and state animations. Activations: {}",
                        clicks.get()
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate to a button".into()],
            vec!["Space or Enter".into(), "Activate a button".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "In this native example, Tab moves focus between the two buttons and Space or Enter activates the focused one. The focused button draws a focus ring, and the status text reports the last activation.",
    );
    gap(Dp(24.0));
    keyboard_demo();
}

#[tessera]
fn keyboard_demo() {
    let status = remember(|| "No activation yet".to_string());
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Save")
                            .on_click(move || status.set("Save activated".to_string()))
                            .child(|| {
                                text().content("Save");
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Discard")
                            .on_click(move || status.set("Discard activated".to_string()))
                            .child(|| {
                                text().content("Discard");
                            });
                    });
                gap(Dp(16.0));
                text().content(status.get()).style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn labeling() {
    prose(
        "The accessibility label for a button should match the visible label text on the button, such as Done, Send, or Reply. It can contain extra contextual information if necessary.",
    );
    gap(Dp(24.0));
    labeling_example();
    caption("Label: Next · Role: Button. The accessible label repeats the visible text.");
    gap(Dp(32.0));
    prose(
        "An icon inside a button is decorative when the label already names the action. Provide the accessible label on the button rather than describing the icon.",
    );
}

#[tessera]
fn labeling_example() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Maze of Mycelium")
                    .style(type_style(20.0, 28.0));
                gap(Dp(8.0));
                text()
                    .content("From truffle hunting in the forests of France to digging deep underground to find vast mycelium networks, this documentary shows a side of the fungi family you never knew existed.")
                    .style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(|| {
                        button()
                            .text()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Back")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Back");
                            });
                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label("Next")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Next");
                            });
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        column().children(|| {
                            text()
                                .content("Label: Next")
                                .style(type_style(14.0, 20.0));
                            text()
                                .content("Role: Button")
                                .style(type_style(14.0, 20.0));
                        });
                    });
            });
    }));
}

#[tessera]
fn baseline_tokens() {
    prose(
        "Baseline token sets describe the reference color, typography, and size roles for a small filled button. These values match the native component defaults used on this page.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "md.comp.filled-button.container.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.filled-button.label-text.color".into(),
                "On primary".into(),
            ],
            vec![
                "md.comp.filled-button.container.height".into(),
                "40dp".into(),
            ],
            vec![
                "md.comp.filled-button.container.shape".into(),
                "Full capsule".into(),
            ],
            vec![
                "md.comp.filled-button.label-text".into(),
                "Label large · 14sp / 20sp".into(),
            ],
            vec!["md.comp.filled-button.icon.size".into(), "20dp".into()],
            vec!["md.comp.filled-button.leading-space".into(), "16dp".into()],
            vec!["md.comp.filled-button.trailing-space".into(), "16dp".into()],
            vec![
                "md.comp.filled-button.icon-label-space".into(),
                "8dp".into(),
            ],
            vec![
                "md.comp.filled-button.pressed.state-layer.opacity".into(),
                "10%".into(),
            ],
        ],
    );
}

#[tessera]
fn playground() {
    let style = remember(|| ButtonVariant::Filled);
    let size = remember(|| ButtonSize::Small);
    let shape = remember(|| ButtonShape::Round);
    let toggle = remember(|| false);
    let selected = remember(|| false);
    let enabled = remember(|| true);
    let with_icon = remember(|| false);
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .text()
                            .accessibility_label("Cycle color style")
                            .on_click(move || style.set(next_style(style.get())))
                            .child(move || {
                                text().content(format!("Style: {}", style_name(style.get())));
                            });
                        button()
                            .text()
                            .accessibility_label("Cycle size")
                            .on_click(move || size.set(next_size(size.get())))
                            .child(move || {
                                text().content(format!("Size: {}", size_name(size.get())));
                            });
                        button()
                            .tonal()
                            .selected(shape.get() == ButtonShape::Square)
                            .accessibility_label("Square shape")
                            .on_click(move || {
                                shape.set(if shape.get() == ButtonShape::Square {
                                    ButtonShape::Round
                                } else {
                                    ButtonShape::Square
                                });
                            })
                            .child(|| {
                                text().content("Square");
                            });
                        button()
                            .tonal()
                            .selected(toggle.get())
                            .accessibility_label("Toggle variant")
                            .on_click(move || {
                                let next = !toggle.get();
                                toggle.set(next);
                                if !next {
                                    selected.set(false);
                                }
                            })
                            .child(|| {
                                text().content("Toggle");
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label("Disabled")
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(|| {
                                text().content("Disabled");
                            });
                        button()
                            .tonal()
                            .selected(with_icon.get())
                            .accessibility_label("Leading icon")
                            .on_click(move || with_icon.set(!with_icon.get()))
                            .child(|| {
                                text().content("Icon");
                            });
                    });
                gap(Dp(32.0));
                button()
                    .variant(style.get())
                    .size(size.get())
                    .shape_style(shape.get())
                    .selected_optional(if toggle.get() && style.get() != ButtonVariant::Text {
                        Some(selected.get())
                    } else {
                        None
                    })
                    .enabled(enabled.get())
                    .accessibility_label("Live button")
                    .accessibility_description(format!("Activations: {}", activations.get()))
                    .leading_icon_optional(with_icon.get().then(|| {
                        RenderSlot::new(|| {
                            icon().painter(filled::EDIT_SVG);
                        })
                    }))
                    .on_click(move || {
                        activations.set(activations.get() + 1);
                        if toggle.get() && style.get() != ButtonVariant::Text {
                            selected.set(!selected.get());
                        }
                    })
                    .child(move || {
                        text().content("Enable");
                    });
                gap(Dp(24.0));
                text()
                    .content(format!(
                        "Activations: {} · {}",
                        activations.get(),
                        if toggle.get() && style.get() != ButtonVariant::Text {
                            if selected.get() {
                                "Selected"
                            } else {
                                "Unselected"
                            }
                        } else {
                            "Default action"
                        }
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
}
