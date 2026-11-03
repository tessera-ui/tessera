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
    material_icons::{Asset, filled},
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, RenderSlotWith, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
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
const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const CAUTION_COLOR: Color = Color::from_rgb(0.705, 0.365, 0.0);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CELL: f64 = 120.0;

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
pub(super) fn buttons_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
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
                content: RenderSlot::new(|| {
                    anatomy(vec!["Container", "Label text", "Icon (optional)"]);
                }),
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
    .route_optional(route.clone())
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
    leading: Option<Asset>,
    presentation: Option<InteractionVisualState>,
    padding: Option<Padding>,
    modifier: Option<Modifier>,
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
        .presentation_state_optional(presentation)
        .content_padding_optional(padding)
        .modifier_optional(modifier)
        .accessibility_label(label.clone())
        .accessibility_description(format!("Activations: {}", activations.get()))
        .leading_icon_optional(if let Some(asset) = leading {
            Some(RenderSlot::new(move || {
                icon().painter(asset);
            }))
        } else {
            None
        })
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
    legend(vec![
        "Elevated button",
        "Filled button",
        "Filled tonal button",
        "Outlined button",
        "Text button",
    ]);
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
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        button_preview(
                            ButtonVariant::Filled,
                            ButtonSize::Small,
                            ButtonShape::Round,
                            false,
                            true,
                            "Label",
                        )
                        .leading(filled::EDIT_SVG);
                    }),
                );
                spacer().modifier(Modifier::new().width(Dp(64.0)));
                marked(2, RenderSlot::new(|| config_row(1)));
            });
    }));
    gap(Dp(16.0));
    legend(vec!["Default button", "Toggle button"]);
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["Default".into(), "Available".into(), "Available".into()],
            vec!["Toggle (selection)".into(), "--".into(), "Available".into()],
        ],
    );
}

#[tessera]
fn configurations() {
    config_figure(vec![0, 2, 3, 4]);
    gap(Dp(16.0));
    legend(vec!["Size", "Shape", "Color", "Small button padding"]);
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
}

#[tessera]
fn anatomy(labels: Vec<&'static str>) {
    panel(RenderSlot::new(|| {
        button_preview(
            ButtonVariant::Filled,
            ButtonSize::Large,
            ButtonShape::Round,
            false,
            true,
            "Label",
        )
        .leading(filled::EDIT_SVG);
    }));
    gap(Dp(16.0));
    legend(labels);
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
                            style_grid(
                                vec!["1. Default", "2. Toggle unselected", "3. Toggle selected"],
                                vec!["A", "B", "C", "D", "E"],
                            );
                        });
                });
        });
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. For designers, this means working with color values that correspond with tokens. In implementation, a color value will be a token that references a value.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "There are five built-in button color styles: elevated, filled, tonal, outlined, and text"
            .into(),
        "The default and toggle buttons use different colors".into(),
        "Toggle buttons don’t use the text style".into(),
    ]);
    gap(Dp(24.0));
    note(
        "Note: These color roles were chosen to create design coherence and familiarity. Other color roles can be used as long as the container and text have a 3:1 contrast ratio. For example, tertiary and on tertiary.",
    );
    gap(Dp(32.0));
    color_examples(false);
    gap(Dp(24.0));
    color_examples(true);
    caption("A. Elevated, B. Filled, C. Tonal, D. Outlined, E. Text");
    gap(Dp(16.0));
    legend(vec!["Default", "Toggle: unselected", "Toggle: selected"]);
    gap(Dp(32.0));
    let pair = |container: &str, content: &str| format!("{container}\n{content}");
    spec_table(
        vec![
            "".into(),
            "1. Default".into(),
            "2. Toggle unselected".into(),
            "3. Toggle selected".into(),
        ],
        vec![
            vec![
                pair("Elevated container", "Elevated icon & label"),
                pair("Surface container low", "Primary"),
                pair("Surface container low", "Primary"),
                pair("Primary", "On primary"),
            ],
            vec![
                pair("Filled container", "Filled icon & label"),
                pair("Primary", "On primary"),
                pair("Surface container", "On surface variant"),
                pair("Primary", "On primary"),
            ],
            vec![
                pair("Tonal container", "Tonal icon & label"),
                pair("Secondary container", "On secondary container"),
                pair("Secondary container", "On secondary container"),
                pair("Secondary", "On secondary"),
            ],
            vec![
                pair("Outlined container", "Outlined icon & label"),
                pair("Outline variant (outline)", "On surface variant"),
                pair("Outline variant (outline)", "On surface variant"),
                pair("Inverse surface", "Inverse on surface"),
            ],
            vec![
                "Text icon & label".into(),
                "Primary".into(),
                "--".into(),
                "--".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "States are visual representations used to communicate the status of a component or interactive element.",
    );
    gap(Dp(40.0));
    for style in STYLES {
        subheading(if style == ButtonVariant::Text {
            "Text button style states".into()
        } else {
            format!("{} button states", style_name(style))
        });
        match style {
            ButtonVariant::Elevated => {
                gap(Dp(16.0));
                prose(
                    "The elevated button style has an elevation of 1 by default and 0 when disabled.",
                );
            }
            ButtonVariant::Outlined => {
                gap(Dp(16.0));
                prose(
                    "The outlined button’s container fill is invisible at rest, but the opacity and state layers behave the same as other button styles when disabled, hovered, focused, or pressed.",
                );
            }
            ButtonVariant::Text => {
                gap(Dp(16.0));
                prose(
                    "The text button’s container is invisible at rest, but the opacity and state layers behave the same as other button styles when disabled, hovered, focused, or pressed. There is no toggle text button.",
                );
            }
            _ => {}
        }
        gap(Dp(24.0));
        subheading("Default".into()).level(4);
        gap(Dp(24.0));
        button_state_matrix(style, false);
        if style != ButtonVariant::Text {
            gap(Dp(40.0));
            subheading("Toggle".into()).level(4);
            gap(Dp(16.0));
            caption("A. Unselected, B. Selected");
            gap(Dp(16.0));
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
    subheading("Pressed state".into()).level(4);
    gap(Dp(16.0));
    prose(
        "When pressed, buttons can morph to become more square. Both round and square buttons should have the same pressed shape.",
    );
    gap(Dp(16.0));
    prose(
        "The corner radius value differs for each button size. See the corner sizes under Measurements.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["Enabled", "Hovered", "Pressed"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    [ButtonShape::Round, ButtonShape::Square][row],
                    false,
                    true,
                    "Learn more",
                )
                .presentation(
                    [
                        InteractionVisualState::Rest,
                        InteractionVisualState::Hover,
                        InteractionVisualState::Pressed,
                    ][column],
                );
            }),
        );
    }));
    caption("A. Round button, B. Square button");
    gap(Dp(48.0));
    subheading("When selected".into()).level(4);
    gap(Dp(16.0));
    prose(
        "In addition to changing shape when pressed, toggle buttons also change the resting shape from round (unselected) to square (selected).",
    );
    gap(Dp(16.0));
    prose("If the resting unselected shape is square, the selected shape should be round.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["1", "2", "3", "4"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    [ButtonShape::Round, ButtonShape::Square][row],
                    true,
                    true,
                    "Play",
                )
                .initial_selected(column == 3)
                .leading(if column == 3 {
                    filled::PAUSE_SVG
                } else {
                    filled::PLAY_ARROW_SVG
                })
                .presentation(
                    [
                        InteractionVisualState::Rest,
                        InteractionVisualState::Hover,
                        InteractionVisualState::Pressed,
                        InteractionVisualState::Rest,
                    ][column],
                );
            }),
        );
    }));
    caption("A. Round button, B. Square button");
    gap(Dp(16.0));
    legend(vec!["Enabled", "Hovered", "Pressed", "Selected"]);
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for (index, size) in SIZES.into_iter().enumerate() {
                    if index > 0 {
                        gap(Dp(24.0));
                    }
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    button_preview(
                                        ButtonVariant::Filled,
                                        size,
                                        ButtonShape::Round,
                                        false,
                                        true,
                                        "Label",
                                    )
                                    .leading(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(24.0)));
                                    button_preview(
                                        ButtonVariant::Filled,
                                        size,
                                        ButtonShape::Round,
                                        false,
                                        true,
                                        "Label",
                                    );
                                });
                        }),
                    );
                }
            });
    }));
    caption("Padding and size measurements of each button size");
    gap(Dp(16.0));
    legend(vec![
        "Extra small",
        "Small",
        "Medium",
        "Large",
        "Extra large",
    ]);
    gap(Dp(32.0));
    spec_table(
        ["Attribute", "XS", "S", "M", "L", "XL"]
            .map(str::to_owned)
            .to_vec(),
        vec![
            ["Container height", "32dp", "40dp", "56dp", "96dp", "136dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Horizontal padding", "12dp", "16dp", "24dp", "48dp", "64dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Icon size", "20dp", "20dp", "24dp", "32dp", "40dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Icon-label spacing", "4dp", "8dp", "8dp", "12dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Outline width", "1dp", "1dp", "1dp", "2dp", "3dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
    gap(Dp(48.0));
    subheading("Target areas".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Extra small and small icon buttons must have a target size of 48x48dp or larger to be accessible.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["Round button", "Button with icon", "Square button"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                let preview = button_preview(
                    ButtonVariant::Filled,
                    [ButtonSize::ExtraSmall, ButtonSize::Small][row],
                    if column == 2 {
                        ButtonShape::Square
                    } else {
                        ButtonShape::Round
                    },
                    false,
                    true,
                    "Label",
                );
                if column == 1 {
                    preview.leading(filled::EDIT_SVG);
                }
            }),
        );
    }));
    caption("A. Extra small  B. Small");
    gap(Dp(48.0));
    subheading("Corner sizes".into()).level(4);
    gap(Dp(16.0));
    spec_table(
        ["", "XS", "S", "M", "L", "XL"].map(str::to_owned).to_vec(),
        vec![
            ["A. Round button", "Full", "Full", "Full", "Full", "Full"]
                .map(str::to_owned)
                .to_vec(),
            ["B. Square button", "12dp", "12dp", "16dp", "28dp", "28dp"]
                .map(str::to_owned)
                .to_vec(),
            ["C. Pressed state", "8dp", "8dp", "12dp", "16dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
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
fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

/// Numbered caption list under a figure, matching the figure markers.
#[tessera]
fn legend(items: Vec<&'static str>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, item) in items.clone().into_iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
                    .children(move || {
                        badge(index + 1);
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        text().content(item).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// A figure row with a leading numbered marker.
#[tessera]
fn marked(number: usize, content: RenderSlot) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            content.render();
        });
}

#[tessera]
fn guidance(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text()
                        .content(mark.clone())
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(note.clone())
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0))
                .modifier(Modifier::new().weight(1.0));
        });
}

/// A lettered and numbered figure grid with one cell per row and column.
#[tessera]
fn figure_grid(
    columns: Vec<&'static str>,
    rows: Vec<&'static str>,
    cell: RenderSlotWith<(usize, usize)>,
) {
    let count = columns.len();
    column().children(move || {
        let columns = columns.clone();
        row().children(move || {
            spacer().modifier(Modifier::new().width(Dp(40.0)));
            for header in columns.clone() {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().width(Dp(CELL)))
                    .children(move || {
                        text().content(header).style(type_style(14.0, 20.0));
                    });
            }
        });
        for (index, label) in rows.clone().into_iter().enumerate() {
            gap(Dp(16.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    boxed()
                        .modifier(Modifier::new().width(Dp(40.0)))
                        .children(move || {
                            text().content(label).style(type_style(16.0, 24.0));
                        });
                    for column in 0..count {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().width(Dp(CELL)))
                            .children(move || cell.render((index, column)));
                    }
                });
        }
    });
}

#[tessera]
fn note(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_high,
                    Shape::rounded_rectangle(Dp(16.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            icon().painter(filled::STAR_SVG).size(Dp(24.0));
            spacer().modifier(Modifier::new().width(Dp(16.0)));
            text()
                .content(value.clone())
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().weight(1.0));
        });
}

/// One configuration row: sizes, toggle, shapes, colors, padding or square
/// sizes.
#[tessera]
fn config_row(kind: usize) {
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || match kind {
            0 | 5 => {
                for size in SIZES {
                    button_preview(
                        ButtonVariant::Filled,
                        size,
                        if kind == 0 {
                            ButtonShape::Round
                        } else {
                            ButtonShape::Square
                        },
                        false,
                        true,
                        size_name(size),
                    );
                }
            }
            1 => {
                for selected in [false, true] {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Small,
                        ButtonShape::Round,
                        true,
                        true,
                        if selected { "Selected" } else { "Unselected" },
                    )
                    .initial_selected(selected);
                }
            }
            2 => {
                for (shape, label) in [
                    (ButtonShape::Round, "Round"),
                    (ButtonShape::Square, "Square"),
                ] {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Small,
                        shape,
                        false,
                        true,
                        label,
                    );
                }
            }
            3 => {
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
            }
            _ => {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "24dp padding",
                )
                .padding(Padding::symmetric(Dp(24.0), Dp(10.0)));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "16dp padding",
                );
            }
        });
}

#[tessera]
fn config_figure(rows: Vec<usize>) {
    panel(RenderSlot::new(move || {
        let rows = rows.clone();
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                for (index, kind) in rows.clone().into_iter().enumerate() {
                    if index > 0 {
                        gap(Dp(32.0));
                    }
                    marked(index + 1, RenderSlot::new(move || config_row(kind)));
                }
            });
    }));
}

/// Default and toggle buttons for every color style in a lettered grid.
#[tessera]
fn style_grid(columns: Vec<&'static str>, rows: Vec<&'static str>) {
    figure_grid(
        columns,
        rows,
        RenderSlotWith::new(|(row, column): (usize, usize)| {
            let style = STYLES[row];
            if column > 0 && style == ButtonVariant::Text {
                text().content("--").style(type_style(14.0, 20.0));
            } else {
                button_preview(
                    style,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    column > 0,
                    true,
                    "Label",
                )
                .initial_selected(column == 2)
                .leading(filled::EDIT_SVG);
            }
        }),
    );
}

#[tessera]
fn m2_button(style: ButtonVariant) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (container, content) = match style {
        ButtonVariant::Elevated => (scheme.surface_container_low, scheme.primary),
        ButtonVariant::Filled => (scheme.primary, scheme.on_primary),
        ButtonVariant::Tonal => (scheme.secondary_container, scheme.on_secondary_container),
        ButtonVariant::Outlined | ButtonVariant::Text => (Color::TRANSPARENT, scheme.primary),
    };
    let border = if style == ButtonVariant::Outlined {
        scheme.outline
    } else {
        Color::TRANSPARENT
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(36.0))
                .border_with_shape(Dp(1.0), border, Shape::rounded_rectangle(Dp(4.0)))
                .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            text()
                .content("BUTTON")
                .color(content)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn style_example(style: ButtonVariant, #[prop(into)] label: String) {
    panel(RenderSlot::new(move || {
        button_preview(
            style,
            ButtonSize::Medium,
            ButtonShape::Round,
            false,
            true,
            label.clone(),
        );
    }));
}

#[tessera]
fn flight_example(large: bool) {
    panel(RenderSlot::new(move || {
        if large {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    flight_info();
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    flight_actions();
                });
        } else {
            column()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(|| {
                    boxed()
                        .modifier(Modifier::new().fill_max_width())
                        .children(flight_info);
                    gap(Dp(24.0));
                    flight_actions();
                });
        }
    }));
}

#[tessera]
fn flight_info() {
    column().children(|| {
        text().content("SFO → JFK").style(type_style(24.0, 32.0));
        text()
            .content("Depart 8:05 AM · Nonstop · 5h 32m")
            .style(type_style(14.0, 20.0));
    });
}

#[tessera]
fn flight_actions() {
    row().children(|| {
        button_preview(
            ButtonVariant::Outlined,
            ButtonSize::Small,
            ButtonShape::Round,
            false,
            true,
            "Details",
        );
        spacer().modifier(Modifier::new().width(Dp(8.0)));
        button_preview(
            ButtonVariant::Filled,
            ButtonSize::Small,
            ButtonShape::Round,
            false,
            true,
            "Book",
        )
        .leading(filled::FLIGHT_SVG);
    });
}

#[tessera]
fn overview() {
    bullets(vec![
        "Two variants: default and toggle".into(),
        "Can contain an optional leading icon".into(),
        "Five color options: elevated, filled, tonal, outlined, and text".into(),
        "Five size recommendations: extra small, small, medium, large, and extra large".into(),
        "Two shape options: round and square".into(),
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
}

#[tessera]
fn expressive() {
    subheading("May 2025".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Buttons now have a wider variety of shapes and sizes, toggle functionality, and can change shape when selected.",
    );
    for (title, items) in [
        (
            "Variants and naming:",
            vec![
                "Default and toggle (selection)",
                "Color styles are now configurations (elevated, filled, tonal, outlined, text)",
            ],
        ),
        (
            "Shapes:",
            vec![
                "Round and square",
                "Shape morphs when pressed",
                "Shape morphs when selected",
            ],
        ),
        (
            "Sizes:",
            vec![
                "Extra small",
                "Small (existing, default)",
                "Medium",
                "Large",
                "Extra large",
            ],
        ),
        (
            "New padding for small buttons:",
            vec![
                "16dp (recommended to match padding of new sizes)",
                "24dp (no longer recommended)",
            ],
        ),
    ] {
        gap(Dp(16.0));
        prose(title);
        bullets(items.into_iter().map(String::from).collect());
    }
    gap(Dp(32.0));
    config_figure(vec![0, 1, 2, 4]);
    gap(Dp(16.0));
    legend(vec![
        "Five sizes",
        "Toggle (selection)",
        "Two shapes",
        "Two small padding widths",
    ]);
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color. Icons and labels now share the same color. Neutral text button is no longer recommended.".into(),
        "Icons: Standard size for leading and trailing icons is now 20dp".into(),
        "Shape: Fully-rounded corner radius and additional height options".into(),
    ]);
    gap(Dp(24.0));
    m2_m3_figure();
    gap(Dp(16.0));
    legend(vec![
        "M2: Buttons have a height of 36dp and slightly rounded corner radius",
        "M3: Default buttons are taller at 40dp and have fully rounded corners",
    ]);
}

#[tessera]
fn m2_m3_figure() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        flow_row()
                            .item_spacing(Dp(16.0))
                            .line_spacing(Dp(16.0))
                            .children(|| {
                                for style in STYLES {
                                    m2_button(style);
                                }
                            });
                    }),
                );
                gap(Dp(32.0));
                marked(2, RenderSlot::new(|| config_row(3)));
            });
    }));
}

#[tessera]
fn usage() {
    prose(
        "Buttons communicate actions that people can take. They are typically placed throughout the UI, in places like:",
    );
    gap(Dp(8.0));
    bullets(vec![
        "Dialogs".into(),
        "Modal windows".into(),
        "Forms".into(),
        "Cards".into(),
        "Toolbars".into(),
    ]);
    gap(Dp(8.0));
    prose("They can also be placed within standard button groups.");
    gap(Dp(24.0));
    usage_example();
    caption("Use visually-prominent filled buttons for the most important actions");
    gap(Dp(32.0));
    prose(
        "Buttons are just one option for representing actions in a product and shouldn’t be overused. Too many buttons on a screen can disrupt the visual hierarchy.",
    );
    gap(Dp(16.0));
    prose(
        "Consider placing additional actions in a navigation rail, set of chips, text links, or icon buttons.",
    );
    guidance("Do", "Use buttons for discrete actions", DO_COLOR);
    guidance(
        "Don’t",
        "Don’t clutter your UI with too many buttons. Consider presenting low-priority actions in overflow menus or as icon buttons.",
        DONT_COLOR,
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "Learn more",
                );
                gap(Dp(24.0));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "Learn more",
                )
                .modifier(Modifier::new().fill_max_width());
                gap(Dp(24.0));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "Learn more",
                )
                .modifier(Modifier::new().width(Dp(64.0)).clip_to_bounds());
            });
    }));
    guidance(
        "Do",
        "A button container’s width is dynamically set to fit its label text",
        DO_COLOR,
    );
    guidance(
        "Do",
        "Button container width can be responsive, which allows it to stretch horizontally",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "A button container’s width shouldn’t be narrower than its label text",
        DONT_COLOR,
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        style_grid(vec!["A", "B", "C"], vec!["1", "2", "3", "4", "5"]);
    }));
    caption(
        "A: Default button; B: Toggle (unselected); C: Toggle (selected) for five button styles, in order of emphasis:",
    );
    gap(Dp(16.0));
    legend(vec![
        "Elevated button",
        "Filled button",
        "Filled tonal button",
        "Outlined button",
        "Text button",
    ]);
    gap(Dp(32.0));
    prose(
        "A button group is a collection of buttons that relate to each other and can respond to one another. Both buttons and icon buttons can be used inside a button group.",
    );
    gap(Dp(16.0));
    prose(
        "In some cases, there are primary and secondary actions within a button group. Buttons with primary actions should have a higher visual emphasis through size, color, or shape.",
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
fn toggle_buttons() {
    prose(
        "Toggle buttons should be used for binary selections, such as Save or Favorite. When toggle buttons are pressed, they can change color, shape, and labels.",
    );
    gap(Dp(16.0));
    prose(
        "Toggle buttons should use an outlined icon when unselected, and a filled version of the icon when selected. If a filled version doesn’t exist, increase the weight instead.",
    );
    gap(Dp(16.0));
    prose("By default, toggle buttons change from round to square when selected.");
    gap(Dp(24.0));
    toggle_example();
    caption("Use toggle buttons for binary actions");
    gap(Dp(32.0));
    prose(
        "If the label changes on selected or unselected states, be mindful of the character count. Changing the label significantly is disruptive to the user and the page layout.",
    );
    guidance(
        "Do",
        "When using toggleable buttons, keep the label character count a similar length for both states",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "The label length shouldn’t change dramatically to be longer or shorter",
        DONT_COLOR,
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
    anatomy(vec!["Label text", "Container", "Icon (optional)"]);
    gap(Dp(48.0));
    subheading("Label text".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Label text is the most important element of a button. It describes the action that will occur if someone taps a button. It should be very brief, ideally 1–3 words.",
    );
    gap(Dp(16.0));
    prose(
        "Use sentence case, which only capitalizes the first word and proper nouns. This allows the text to distinguish proper nouns, for example: Book with Flights, not BOOK WITH FLIGHTS.",
    );
    gap(Dp(16.0));
    prose("Don’t truncate or wrap label text. It should always be fully visible on a single line.");
    gap(Dp(24.0));
    style_example(ButtonVariant::Filled, "Book with Flights");
    guidance(
        "Do",
        "Use sentence case for button label text, capitalizing the first word and proper nouns",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "Don’t wrap text. For maximum legibility, label text should remain on a single line.",
        DONT_COLOR,
    );
    gap(Dp(24.0));
    prose(
        "Buttons with the outlined and text color style depend on the colors to be recognizable from other text and elements. Use caution when putting these buttons next to visually similar elements, such as chips or large text.",
    );
    guidance(
        "Caution",
        "The outlined button style is very similar to chips. Consider using a filled or tonal button instead.",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("Container".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Button containers hold the label text and optional icon. Buttons with the text color style have a visible container only when hovered, focused, or pressed.",
    );
    gap(Dp(16.0));
    prose("Buttons with a round shape have containers with fully rounded corners.");
    gap(Dp(16.0));
    prose(
        "Buttons with a square shape have containers with more subtle rounding that changes based on button size.",
    );
    gap(Dp(24.0));
    config_figure(vec![0, 5]);
    gap(Dp(16.0));
    legend(vec![
        "Round buttons have containers with fully rounded corners",
        "Square buttons have square containers and change radius as the button size changes",
    ]);
    guidance(
        "Do",
        "A button’s width dynamically adjusts to the label text",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "Avoid setting a fixed width smaller than the label text",
        DONT_COLOR,
    );
    gap(Dp(48.0));
    subheading("Icon (optional)".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Icons visually communicate the button’s action and help draw attention. They should be placed on the leading side of the button, before the label text.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "Send",
                )
                .leading(filled::SEND_SVG);
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                button()
                    .filled()
                    .accessibility_label("Send, right-to-left")
                    .trailing_icon(|| {
                        icon().painter(filled::SEND_SVG).rotation(180.0);
                    })
                    .on_click(|| {})
                    .child(|| {
                        text().content("Send");
                    });
            });
    }));
    guidance(
        "Do",
        "Place the icon to the left of the label in buttons with text in left-to-right languages",
        DO_COLOR,
    );
    guidance(
        "Do",
        "Place the icon to the right of the label in buttons with text in right-to-left languages",
        DO_COLOR,
    );
    guidance(
        "Do",
        "Use icons that clearly communicate their meaning",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "Don’t vertically align an icon and text in the center of a button",
        DONT_COLOR,
    );
    guidance(
        "Don’t",
        "Don’t use two icons in the same button",
        DONT_COLOR,
    );
}

#[tessera]
fn color_styles() {
    subheading("Elevated style".into()).level(4);
    gap(Dp(16.0));
    prose("The elevated button style is the same as the tonal button, but with a shadow.");
    gap(Dp(16.0));
    prose(
        "To avoid overusing shadows, use the elevated style only when absolutely necessary, such as when the button requires visual separation from a visually prominent background.",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Elevated, "Add to cart");
    caption("Elevated buttons provide separation from a visually prominent background");
    gap(Dp(24.0));
    prose(
        "Buttons at higher elevations typically have more emphasis in a design, and should be used sparingly. For high emphasis, consider the filled style instead.",
    );
    guidance(
        "Caution",
        "Higher elevation increases the emphasis of a button",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("Filled style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "The filled button style has the most visual impact after the FAB, and should be used for important, final actions that complete a flow, like Save, Join now, or Confirm.",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Filled, "Join now");
    caption("Filled buttons have high visual impact when used for important actions");
    gap(Dp(24.0));
    prose(
        "Since they have such strong emphasis, the filled style should be used sparingly, ideally for only one action on a page.",
    );
    gap(Dp(16.0));
    prose("In some cases, filled buttons can use tertiary colors.");
    gap(Dp(48.0));
    subheading("Tonal style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "The tonal button style is useful in contexts where a lower-priority button requires slightly more emphasis than an outline would give, such as Next in an onboarding flow. Tonal buttons use the secondary color mapping.",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Tonal, "Next");
    caption("The tonal style has less emphasis than filled or emphasis");
    gap(Dp(48.0));
    subheading("Outlined style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "The outlined style is ideal for medium-emphasis buttons which contain actions that are important, but aren’t the primary action in a product.",
    );
    gap(Dp(16.0));
    prose(
        "Outlined buttons pair well with filled buttons to indicate alternative, secondary actions.",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Outlined, "Share details");
    caption("Outlined buttons contain less important supporting actions");
    gap(Dp(24.0));
    prose(
        "Outlined buttons display a stroke around the button container, and have no fill by default.",
    );
    gap(Dp(16.0));
    prose(
        "They should be placed on simple backgrounds, not visually prominent backgrounds such as images or videos.",
    );
    guidance(
        "Do",
        "Outlined buttons can be used on backgrounds with a color gradient",
        DO_COLOR,
    );
    guidance(
        "Caution",
        "Use caution when placing outlined buttons on top of images. Customizing the button to have a contrasting container fill can help ensure legibility of label text. Or, use a filled button instead.",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("Text style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "The text button style should be used for the lowest priority actions, especially when presenting multiple options.",
    );
    gap(Dp(16.0));
    prose(
        "They should be placed on simple backgrounds, not visually prominent backgrounds such as images or videos. The container isn’t visible until someone interacts with the button.",
    );
    gap(Dp(16.0));
    prose("Don’t underline the text button. Use hyperlinked body text instead to emphasize links.");
    gap(Dp(24.0));
    style_example(ButtonVariant::Text, "Learn more");
    caption("Use text buttons for the lowest priority actions");
    gap(Dp(24.0));
    prose(
        "Text buttons are often placed within components such as cards, dialogs, and snackbars. Since text buttons don’t have a visible container in their default state, they don’t distract from nearby content.",
    );
    gap(Dp(16.0));
    prose(
        "However, since there’s no container, the label text color must always be recognizable from non-button text and elements.",
    );
    gap(Dp(16.0));
    prose("In cards, text buttons help maintain an emphasis on card content.");
    gap(Dp(16.0));
    prose(
        "Dialogs use text buttons because the absence of a container helps unify the action with the dialog text.",
    );
    gap(Dp(16.0));
    prose(
        "Align text buttons to the trailing edge of dialogs, on the right for left-to-right languages and on the left for right-to-left languages.",
    );
}

#[tessera]
fn adaptive() {
    subheading("Resizing".into()).level(4);
    gap(Dp(16.0));
    prose(
        "When scaling layouts for large screen devices, buttons can adapt their visual presentation, size, alignment, and arrangement to fit different contexts and user needs.",
    );
    gap(Dp(16.0));
    prose("Choose the best button position based on screen size.");
    gap(Dp(24.0));
    flight_example(false);
    caption("Filled buttons are end-aligned below flight information in a compact window");
    gap(Dp(24.0));
    flight_example(true);
    caption("Filled buttons are start-aligned beside flight information in a large window");
    gap(Dp(32.0));
    prose(
        "The icon and label text in a button stay centered and grouped as the button's width changes.",
    );
    guidance(
        "Do",
        "Keep the icon and label text grouped and centered",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "Don't ungroup the icon and label text or let them anchor to opposite sides of the button",
        DONT_COLOR,
    );
    gap(Dp(24.0));
    prose(
        "Buttons can be customized to change size and scaling behavior across different breakpoints.",
    );
    gap(Dp(16.0));
    prose(
        "To avoid creating very long buttons in large windows, constrain button width or place buttons beside other elements.",
    );
    guidance(
        "Don’t",
        "Don’t allow the button to stretch in a way that creates long, flat buttons with very little content inside",
        DONT_COLOR,
    );
    gap(Dp(48.0));
    subheading("Presentation".into()).level(4);
    gap(Dp(16.0));
    prose(
        "The size and placement of buttons can change as parent containers, such as cards, adapt for larger screens.",
    );
    gap(Dp(16.0));
    prose(
        "Keep items, including buttons, in the same order between large and small screens to provide a consistent experience for screen readers and keyboard navigation.",
    );
    caption("Buttons can move in the layout, but elements should remain in the same order");
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
    subheading("Color contrast".into()).level(4);
    gap(Dp(16.0));
    prose(
        "Enabled buttons need a 3:1 contrast ratio with the background to meet accessibility best practices.",
    );
    gap(Dp(16.0));
    prose(
        "This is measured from the container for elevated, filled, and tonal button styles, and the label text for outlined and text button styles.",
    );
    gap(Dp(24.0));
    contrast_examples();
    caption("Higher contrast helps differentiate elements");
    gap(Dp(48.0));
    subheading("200% text size".into()).level(4);
    gap(Dp(16.0));
    prose("Avoid excessive text wrapping or truncation by choosing concise strings.");
    gap(Dp(16.0));
    prose(
        "On Android, button labels should be kept concise enough to fit within two lines after the text size is increased to 200%. If a button label exceeds this limit and gets truncated, provide an alternative way to access the full content in a single tap.",
    );
    gap(Dp(24.0));
    text_scale_example();
    guidance(
        "Caution",
        "Avoid excessive text wrapping or truncation by choosing concise strings",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("Rapid clicks".into()).level(4);
    gap(Dp(16.0));
    prose(
        "On the web, you can use a modified motion curve to avoid resonant effects from overlapping animations. This provides a smoother experience for interactions where you anticipate multiple clicks or taps in succession.",
    );
    gap(Dp(24.0));
    rapid_click_example();
    caption("Use the modified motion curve if rapid click or pointer interactions are expected");
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
                    .leading_icon_optional(if with_icon.get() {
                        Some(RenderSlot::new(|| {
                            icon().painter(filled::EDIT_SVG);
                        }))
                    } else {
                        None
                    })
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
