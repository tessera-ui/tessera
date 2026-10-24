//! Interactive Expressive button specifications.
//!
//! ## Usage
//!
//! Compare button selection, sizes, shapes, colors and input feedback.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    shape_def::Shape,
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
    action_articles::{Kind, action_article},
    article::SpecSection,
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
    action_article(
        Kind::Buttons,
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
        ],
        RenderSlot::new(variants),
        RenderSlot::new(anatomy),
        RenderSlot::new(shape_morph),
    )
    .overview(|| {
        overview_art();
    });
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
    let mut control = button()
        .variant(style)
        .size(size)
        .shape_style(shape)
        .selected_optional(if toggle { Some(selected.get()) } else { None })
        .enabled(enabled)
        .accessibility_label(label.clone())
        .accessibility_description(format!("Activations: {}", activations.get()))
        .on_click(move || {
            activations.set(activations.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        });
    if with_icon.unwrap_or(false) {
        control = control.leading_icon(move || {
            icon().painter(filled::EDIT_SVG);
        });
    }
    control.child(move || {
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
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                size_examples();
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
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec!["Category".into(), "Configurations".into()],
        vec![
            vec!["Size".into(), "XS, S, M, L, XL".into()],
            vec!["Shape".into(), "Round, square".into()],
            vec![
                "Color".into(),
                "Elevated, filled, tonal, outlined, text".into(),
            ],
            vec!["Small horizontal padding".into(), "16dp".into()],
        ],
    );
}

#[tessera]
fn tokens() {
    let style = remember(|| ButtonVariant::Filled);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for variant in STYLES {
                button()
                    .text()
                    .accessibility_label(format!("{} color tokens", style_name(variant)))
                    .on_click(move || style.set(variant))
                    .child(move || {
                        text().content(style_name(variant));
                    });
            }
        });
    gap(Dp(24.0));
    let (container, content) = match style.get() {
        ButtonVariant::Elevated => ("Surface container low", "Primary"),
        ButtonVariant::Filled => ("Primary", "On primary"),
        ButtonVariant::Tonal => ("Secondary container", "On secondary container"),
        ButtonVariant::Outlined => ("Transparent · outline variant", "On surface variant"),
        ButtonVariant::Text => ("Transparent", "Primary"),
    };
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), container.into()],
            vec!["Icon and label".into(), content.into()],
            vec!["Disabled label opacity".into(), "38%".into()],
            vec!["Default size".into(), "Small · 40dp".into()],
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
        .selected_optional(if toggle { Some(selected.get()) } else { None })
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
            ["Container height", "32dp", "40dp", "56dp", "96dp", "136dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Icon size", "20dp", "20dp", "24dp", "32dp", "40dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Square corners", "12dp", "12dp", "16dp", "28dp", "28dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Pressed corners", "8dp", "8dp", "12dp", "16dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
    gap(Dp(24.0));
    text()
        .content("Minimum interaction target: 48 × 48dp.")
        .style(type_style(16.0, 24.0))
        .color(INK);
}
