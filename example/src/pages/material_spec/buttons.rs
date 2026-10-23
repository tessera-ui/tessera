//! Interactive Expressive button specifications.
//!
//! ## Usage
//!
//! Compare button selection, sizes, shapes, colors and input feedback.

use tessera_components::{
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, remember, tessera};

use super::{
    ACCENT, INK,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, type_style,
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
    );
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
    text()
        .content("Hover, focus with Tab, or hold a button to inspect its live input feedback.")
        .style(type_style(16.0, 24.0))
        .color(INK);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for style in STYLES {
                    text()
                        .content(format!("{} button states", style_name(style)))
                        .style(type_style(24.0, 32.0));
                    gap(Dp(24.0));
                    flow_row()
                        .item_spacing(Dp(16.0))
                        .line_spacing(Dp(16.0))
                        .children(move || {
                            button_preview(
                                style,
                                ButtonSize::Small,
                                ButtonShape::Round,
                                false,
                                true,
                                "Enabled",
                            );
                            button_preview(
                                style,
                                ButtonSize::Small,
                                ButtonShape::Round,
                                false,
                                false,
                                "Disabled",
                            );
                            button_preview(
                                style,
                                ButtonSize::Small,
                                ButtonShape::Round,
                                false,
                                true,
                                "Interact",
                            );
                        });
                    gap(Dp(40.0));
                }
            });
    }));
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
