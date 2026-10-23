//! Floating primary action specifications and live examples.
//!
//! ## Usage
//!
//! Compare floating actions, expressive sizes and label expansion.

use tessera_components::{
    button::button,
    column::column,
    extended_floating_action_button::{
        ExtendedFloatingActionButtonSize, extended_floating_action_button,
    },
    floating_action_button::{FloatingActionButtonSize, floating_action_button},
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    surface::surface,
    text::text,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

const TONES: [&str; 6] = [
    "Primary container",
    "Secondary container",
    "Tertiary container",
    "Primary",
    "Secondary",
    "Tertiary",
];

fn colors(scheme: &MaterialColorScheme, tone: usize) -> (Color, Color) {
    match tone {
        0 => (scheme.primary_container, scheme.on_primary_container),
        1 => (scheme.secondary_container, scheme.on_secondary_container),
        2 => (scheme.tertiary_container, scheme.on_tertiary_container),
        3 => (scheme.primary, scheme.on_primary),
        4 => (scheme.secondary, scheme.on_secondary),
        _ => (scheme.tertiary, scheme.on_tertiary),
    }
}

#[tessera]
pub(super) fn floating_actions_spec(extended: bool) {
    let mut sections = vec![SpecSection {
        title: "Variants",
        content: RenderSlot::new(move || {
            variants(extended);
        }),
    }];
    if !extended {
        sections.push(SpecSection {
            title: "Configurations",
            content: RenderSlot::new(configurations),
        });
    }
    sections.extend([
        SpecSection {
            title: "Tokens & specs",
            content: RenderSlot::new(move || {
                tokens(extended);
            }),
        },
        SpecSection {
            title: "Anatomy",
            content: RenderSlot::new(move || {
                anatomy(extended);
            }),
        },
        SpecSection {
            title: "Color",
            content: RenderSlot::new(move || {
                color_examples(extended);
            }),
        },
        SpecSection {
            title: "States",
            content: RenderSlot::new(move || {
                states(extended);
            }),
        },
        SpecSection {
            title: "Measurements",
            content: RenderSlot::new(move || {
                measurements(extended);
            }),
        },
    ]);
    action_article(
        if extended {
            Kind::ExtendedFab
        } else {
            Kind::Fab
        },
        sections,
        RenderSlot::new(move || {
            variants(extended);
        }),
        RenderSlot::new(move || {
            anatomy(extended);
        }),
        RenderSlot::new(move || {
            panel(RenderSlot::new(move || {
                sample(extended, 0, 3, "Behavior floating action").collapsible(extended);
            }));
        }),
    );
}

#[tessera]
fn sample(
    extended: bool,
    size: usize,
    tone: usize,
    #[prop(into)] label: String,
    collapsible: Option<bool>,
) {
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    let (container, content) = colors(&theme.color_scheme, tone);
    let count = remember(|| 0_u32);
    let expanded = remember(|| true);
    let count_label = label.clone();
    column().children(move || {
        if collapsible.unwrap_or(false) {
            button()
                .tonal()
                .accessibility_label("Toggle Compose label")
                .selected(expanded.get())
                .on_click(move || expanded.set(!expanded.get()))
                .child(move || {
                    text().content(if expanded.get() {
                        "Collapse label"
                    } else {
                        "Expand label"
                    });
                });
            gap(Dp(24.0));
        }
        if extended {
            extended_floating_action_button("Compose")
                .size(
                    [
                        ExtendedFloatingActionButtonSize::Small,
                        ExtendedFloatingActionButtonSize::Medium,
                        ExtendedFloatingActionButtonSize::Large,
                    ][size],
                )
                .icon(filled::EDIT_SVG)
                .expanded(expanded.get())
                .container_color(container)
                .content_color(content)
                .accessibility_label(label.clone())
                .on_click(move || count.set(count.get() + 1));
        } else {
            floating_action_button()
                .size(
                    [
                        FloatingActionButtonSize::Standard,
                        FloatingActionButtonSize::Medium,
                        FloatingActionButtonSize::Large,
                    ][size],
                )
                .container_color(container)
                .content_color(content)
                .ripple_color(content)
                .accessibility_label(label.clone())
                .on_click(move || count.set(count.get() + 1))
                .content(move || {
                    icon()
                        .painter(filled::ADD_SVG)
                        .size(Dp([24.0, 28.0, 36.0][size]));
                });
        }
        gap(Dp(16.0));
        text()
            .content(format!("{count_label}: {}", count.get()))
            .style(type_style(14.0, 20.0));
    });
}

#[tessera]
fn variants(extended: bool) {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(32.0))
            .children(move || {
                for (size, name) in if extended {
                    ["Small", "Medium", "Large"]
                } else {
                    ["Regular", "Medium", "Large"]
                }
                .into_iter()
                .enumerate()
                {
                    sample(extended, size, 0, format!("{name} floating action"));
                }
            });
    }));
}

#[tessera]
fn configurations() {
    text().content("Choose a container color or a stronger accent color. Each uses its corresponding on-color for the icon and interaction feedback.").style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    spec_table(
        vec!["Configuration".into(), "Options".into()],
        vec![
            vec![
                "Container colors".into(),
                "Primary container · Secondary container · Tertiary container".into(),
            ],
            vec![
                "Accent colors".into(),
                "Primary · Secondary · Tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens(extended: bool) {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Default container".into(), "Primary container".into()],
            vec![
                if extended {
                    "Icon and label".into()
                } else {
                    "Icon".into()
                },
                "On primary container".into(),
            ],
            vec![
                "Resting / focused / pressed elevation".into(),
                "Level 3 · 6dp".into(),
            ],
            vec!["Hovered elevation".into(), "Level 4 · 8dp".into()],
            vec![
                "Hover / focus / pressed state layer".into(),
                "8% / 10% / 10% · content color".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    measurements(extended);
}

#[tessera]
fn anatomy(extended: bool) {
    panel(RenderSlot::new(move || {
        column().children(move || {
            sample(extended, 0, 0, "Compose action").collapsible(extended);
            gap(Dp(24.0));
            text()
                .content(if extended {
                    "1  Container    2  Label text    3  Icon"
                } else {
                    "1  Container    2  Icon"
                })
                .style(type_style(16.0, 24.0));
        });
    }));
}

#[tessera]
fn color_examples(extended: bool) {
    for dark in [false, true] {
        subheading(if dark {
            "Dark scheme".into()
        } else {
            "Light scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        flow_row()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .item_spacing(Dp(32.0))
                            .line_spacing(Dp(32.0))
                            .children(move || {
                                for (tone, name) in TONES.into_iter().enumerate() {
                                    sample(
                                        extended,
                                        0,
                                        tone,
                                        format!("{} {name}", if dark { "Dark" } else { "Light" }),
                                    );
                                }
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn states(extended: bool) {
    text().content("Hover the action, focus it with Tab, and activate it with Enter, Space or a pointer. Its elevation and state layer respond to the current interaction.").style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        sample(extended, 0, 3, "Interactive floating action");
    }));
}

#[tessera]
fn measurements(extended: bool) {
    let rows = vec![
        vec![
            if extended {
                "Small".into()
            } else {
                "Regular".into()
            },
            "56dp".into(),
            "24dp".into(),
            "16dp".into(),
        ],
        vec!["Medium".into(), "80dp".into(), "28dp".into(), "20dp".into()],
        vec!["Large".into(), "96dp".into(), "36dp".into(), "28dp".into()],
    ];
    spec_table(
        vec![
            "Size".into(),
            "Height".into(),
            "Icon".into(),
            "Corners".into(),
        ],
        rows,
    );
    if extended {
        gap(Dp(24.0));
        spec_table(
            vec![
                "Size".into(),
                "Leading / trailing".into(),
                "Icon to label".into(),
                "Type style".into(),
            ],
            vec![
                vec![
                    "Small".into(),
                    "16dp".into(),
                    "8dp".into(),
                    "Title medium".into(),
                ],
                vec![
                    "Medium".into(),
                    "26dp".into(),
                    "12dp".into(),
                    "Title large".into(),
                ],
                vec![
                    "Large".into(),
                    "28dp".into(),
                    "16dp".into(),
                    "Headline small".into(),
                ],
            ],
        );
        gap(Dp(24.0));
        text().content("Allow 16dp around floating actions. Expanded width follows the label; collapsed width matches the container height.").style(type_style(16.0, 24.0));
    } else {
        gap(Dp(24.0));
        text().content("Container width matches its height. Center the icon with 16dp, 26dp or 30dp of space on each side.").style(type_style(16.0, 24.0));
    }
}
