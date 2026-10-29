//! Extended floating action button specifications.
//!
//! ## Usage
//!
//! Compare extended FAB sizes, tones, label expansion and interaction states.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    extended_floating_action_button::{
        ExtendedFloatingActionButtonSize, extended_floating_action_button,
    },
    flow_row::flow_row,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const SIZES: [ExtendedFloatingActionButtonSize; 3] = [
    ExtendedFloatingActionButtonSize::Small,
    ExtendedFloatingActionButtonSize::Medium,
    ExtendedFloatingActionButtonSize::Large,
];

const TONES: [&str; 6] = [
    "Primary container",
    "Secondary container",
    "Tertiary container",
    "Primary",
    "Secondary",
    "Tertiary",
];

fn size_name(size: ExtendedFloatingActionButtonSize) -> &'static str {
    match size {
        ExtendedFloatingActionButtonSize::Small => "Small",
        ExtendedFloatingActionButtonSize::Medium => "Medium",
        ExtendedFloatingActionButtonSize::Large => "Large",
    }
}

fn tone_colors(scheme: &MaterialColorScheme, tone: usize) -> (Color, Color) {
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
pub(super) fn extended_fabs_spec() {
    article(
        "Extended FABs",
        "Extended floating action buttons (extended FABs) help people take primary actions",
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
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "Baseline extended FAB",
                content: RenderSlot::new(baseline),
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
                    title: "Anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "Color",
                    content: RenderSlot::new(guidelines_color),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive_layout),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
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
                    title: "Initial focus",
                    content: RenderSlot::new(initial_focus),
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

/// Renders one extended FAB with optional configuration overrides.
#[tessera]
fn fab(
    #[prop(into)] label: String,
    size: Option<ExtendedFloatingActionButtonSize>,
    tone: Option<usize>,
    expanded: Option<bool>,
    enabled: Option<bool>,
    counted: Option<bool>,
    with_icon: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (container, content) = tone_colors(&scheme, tone.unwrap_or(0));
    let activations = remember(|| 0_u32);
    let spoken = label.clone();
    column().children(move || {
        extended_floating_action_button(label.clone())
            .size(size.unwrap_or(ExtendedFloatingActionButtonSize::Small))
            .icon_optional(if with_icon.unwrap_or(true) {
                Some(filled::EDIT_SVG.into())
            } else {
                None
            })
            .expanded(expanded.unwrap_or(true))
            .enabled(enabled.unwrap_or(true))
            .container_color(container)
            .content_color(content)
            .accessibility_label(spoken.clone())
            .on_click(move || activations.set(activations.get() + 1));
        if counted.unwrap_or(false) {
            gap(Dp(16.0));
            text()
                .content(format!("Activations: {}", activations.get()))
                .style(type_style(14.0, 20.0));
        }
    });
}

#[tessera]
fn overview() {
    prose(
        "Extended FABs help people take a primary action. Use them for the most common or important action on a screen, and choose the size that matches the surrounding layout.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Use for the most common or important action on a screen".into(),
        "Three variants: small, medium, and large".into(),
        "Use instead of a FAB when label text is needed to understand the action".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    prose("1  Small extended FAB    2  Medium extended FAB    3  Large extended FAB");
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
        "Tessera examples on this page use the native extended_floating_action_button component. Small, medium, and large sizes, six color tones, optional icons, and label expansion are implemented, together with elevation and state-layer feedback.",
    );
}

#[tessera]
fn expressive() {
    prose(
        "May 2025: the extended FAB gained small, medium, and large sizes, and the baseline extended FAB is no longer recommended. Use the small extended FAB instead.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Change".into(), "Details".into()],
        vec![
            vec![
                "Sizes".into(),
                "Small (56dp), medium (80dp), and large (96dp)".into(),
            ],
            vec![
                "Type style".into(),
                "The small extended FAB uses title medium instead of label large".into(),
            ],
            vec!["Padding".into(), "Inner padding was reduced".into()],
            vec![
                "Baseline".into(),
                "Extended FAB (baseline) is no longer recommended".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
    }));
    caption("The small extended FAB is the recommended size in the expressive update.");
}

#[tessera]
fn differences() {
    prose(
        "M2: extended FABs used a fixed 48dp height, a single primary color, and label large type. M3 introduced container color roles, elevation levels, and state layers that follow the content color.",
    );
    gap(Dp(16.0));
    prose(
        "M3 Expressive: the container grew into three sizes, the label type moved to title medium for the small size, and the icon size follows the container height.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "M2".into(), "M3 Expressive".into()],
        vec![
            vec!["Height".into(), "48dp".into(), "56 / 80 / 96dp".into()],
            vec![
                "Label type".into(),
                "Label large".into(),
                "Title medium / title large / headline small".into(),
            ],
            vec!["Icon size".into(), "24dp".into(), "24 / 28 / 36dp".into()],
            vec![
                "Color".into(),
                "Single accent".into(),
                "Primary, secondary, and tertiary containers or accents".into(),
            ],
        ],
    );
}

#[tessera]
fn variants() {
    prose(
        "Three sizes share the same anatomy and differ in height, icon size, padding, and label type. Small is the recommended default.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("1 Small extended FAB · 2 Medium extended FAB · 3 Large extended FAB");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec![
                "Small extended FAB".into(),
                "Available".into(),
                "Available".into(),
            ],
            vec![
                "Medium extended FAB".into(),
                "--".into(),
                "Available".into(),
            ],
            vec!["Large extended FAB".into(), "--".into(), "Available".into()],
            vec![
                "Extended FAB (baseline)".into(),
                "Available".into(),
                "Not recommended. Use small extended FAB.".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    prose(
        "Choose a size, a container tone, whether the leading icon is present, and whether the label is expanded. All options keep the same anatomy and interaction feedback.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("1  Size".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(40.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for size in SIZES {
                            fab(size_name(size)).size(size);
                        }
                    });
                gap(Dp(40.0));
                subheading("2  Color".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for (tone, name) in TONES.into_iter().enumerate() {
                            fab(name)
                                .size(ExtendedFloatingActionButtonSize::Small)
                                .tone(tone);
                        }
                    });
                gap(Dp(40.0));
                subheading("3  Icon position".into());
                gap(Dp(24.0));
                text()
                    .content("With leading icon")
                    .style(type_style(14.0, 20.0));
                gap(Dp(16.0));
                fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
                gap(Dp(24.0));
                text().content("Without icon").style(type_style(14.0, 20.0));
                gap(Dp(16.0));
                fab("Compose")
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .with_icon(false);
            });
    }));
    gap(Dp(32.0));
    subheading("4  Label expansion".into());
    gap(Dp(8.0));
    prose(
        "An extended FAB can collapse to an icon-only container and expand again when its label is needed. Use the control to compare both states.",
    );
    gap(Dp(24.0));
    collapsible("Compose");
}

#[tessera]
fn collapsible(#[prop(into)] label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let expanded = remember(|| true);
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        let spoken = label.clone();
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .selected(!expanded.get())
                    .accessibility_label("Collapse or expand the extended FAB label")
                    .on_click(move || expanded.set(!expanded.get()))
                    .child(move || {
                        text().content(if expanded.get() {
                            "Collapse label"
                        } else {
                            "Expand label"
                        });
                    });
                gap(Dp(32.0));
                extended_floating_action_button(spoken.clone())
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .expanded(expanded.get())
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .accessibility_label(spoken.clone())
                    .on_click(move || activations.set(activations.get() + 1));
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "{} · activations: {}",
                        if expanded.get() {
                            "Expanded"
                        } else {
                            "Collapsed"
                        },
                        activations.get()
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn tokens() {
    prose(
        "Extended FAB tokens are organized by size and color. Values below follow the native component defaults used on this page.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "md.comp.extended-fab.container.color".into(),
                "Primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.icon.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.label-text.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.container.height".into(),
                "56dp small · 80dp medium · 96dp large".into(),
            ],
            vec![
                "md.comp.extended-fab.container.shape".into(),
                "Full capsule".into(),
            ],
            vec![
                "md.comp.extended-fab.label-text".into(),
                "Title medium · title large · headline small".into(),
            ],
            vec![
                "md.comp.extended-fab.icon.size".into(),
                "24dp · 28dp · 36dp".into(),
            ],
            vec![
                "md.comp.extended-fab.leading-space".into(),
                "16dp · 26dp · 28dp".into(),
            ],
            vec![
                "md.comp.extended-fab.trailing-space".into(),
                "16dp · 20dp · 28dp".into(),
            ],
            vec![
                "md.comp.extended-fab.icon-label-space".into(),
                "8dp · 12dp · 16dp".into(),
            ],
            vec![
                "md.comp.extended-fab.container.elevation".into(),
                "Level 3 · level 4 when hovered".into(),
            ],
            vec![
                "md.comp.extended-fab.state-layer".into(),
                "Content color at 8% hover and 10% focus or press".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "When a non-default color mapping is used, keep the state layer color equal to the icon color. For the primary mapping, the state layer is primary.",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "An extended FAB combines a container, an optional leading icon, and a label. The icon and label share one content color, and the container carries the tone and elevation.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column().children(|| {
            fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
            gap(Dp(32.0));
            for line in ["1  Container", "2  Icon (optional)", "3  Label text"] {
                text()
                    .content(line)
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption(
        "The label states the action; the icon supports recognition without repeating the label.",
    );
}

#[tessera]
fn colors() {
    prose(
        "Container tones map to the primary, secondary, and tertiary color roles, either as containers or as solid accents. Each tone pairs with its matching on-color for the icon and label.",
    );
    gap(Dp(24.0));
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
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                flow_row()
                                    .item_spacing(Dp(24.0))
                                    .line_spacing(Dp(24.0))
                                    .children(move || {
                                        for (tone, name) in TONES.into_iter().enumerate() {
                                            fab(name).size(ExtendedFloatingActionButtonSize::Small)
                                                .tone(tone);
                                        }
                                    });
                                gap(Dp(16.0));
                                text()
                                    .content("1  Primary container    2  Secondary container    3  Tertiary container    4  Primary    5  Secondary    6  Tertiary")
                                    .style(type_style(14.0, 20.0));
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn states() {
    prose(
        "States communicate elevation and interaction. An enabled extended FAB rests at level 3, rises to level 4 on hover, and keeps level 3 while focused or pressed while the state layer darkens. A disabled extended FAB keeps its shape but stops accepting input.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("Enabled").size(ExtendedFloatingActionButtonSize::Small);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("Disabled").size(ExtendedFloatingActionButtonSize::Small).enabled(false);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("Elevated").size(ExtendedFloatingActionButtonSize::Small).tone(3);
                    });
                gap(Dp(32.0));
                text()
                    .content(
                        "Hover, focus with Tab, or press the actions above to see the state layer and elevation respond.",
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["State".into(), "Elevation".into(), "State layer".into()],
        vec![
            vec!["Enabled".into(), "Level 3".into(), "--".into()],
            vec!["Hovered".into(), "Level 4".into(), "8%".into()],
            vec!["Focused".into(), "Level 3".into(), "10%".into()],
            vec!["Pressed".into(), "Level 3".into(), "10%".into()],
            vec!["Disabled".into(), "None".into(), "Faded container".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The expanded and collapsed states use the same elevation. Only the width and label opacity animate when the label expands or collapses.",
    );
}

#[tessera]
fn measurements() {
    prose(
        "Height, icon size, and padding scale with the size variant. Reserve 16dp margins around a floating action, and keep at least 80dp of width available for the label when the action is expanded.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Attribute".into(),
            "Small".into(),
            "Medium".into(),
            "Large".into(),
        ],
        vec![
            vec![
                "Container height".into(),
                "56dp".into(),
                "80dp".into(),
                "96dp".into(),
            ],
            vec![
                "Icon size".into(),
                "24dp".into(),
                "28dp".into(),
                "36dp".into(),
            ],
            vec![
                "Leading space".into(),
                "16dp".into(),
                "26dp".into(),
                "28dp".into(),
            ],
            vec![
                "Icon to label".into(),
                "8dp".into(),
                "12dp".into(),
                "16dp".into(),
            ],
            vec![
                "Trailing space".into(),
                "16dp".into(),
                "20dp".into(),
                "28dp".into(),
            ],
            vec![
                "Label type".into(),
                "Title medium".into(),
                "Title large".into(),
                "Headline small".into(),
            ],
            vec![
                "Corners".into(),
                "Full".into(),
                "Full".into(),
                "Full".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("Size and padding measurements of the small, medium, and large extended FABs.");
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Outer margin".into(), "16dp".into()],
            vec![
                "Collapsed width".into(),
                "Matches the container height".into(),
            ],
            vec![
                "Expanded width".into(),
                "Follows the label, minimum 80dp".into(),
            ],
            vec!["Minimum interactive target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn baseline() {
    prose(
        "The baseline extended FAB is no longer recommended in the M3 expressive update. Use a small extended FAB instead: the type style moved from label large to title medium and the inner padding was reduced.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Label").size(ExtendedFloatingActionButtonSize::Small);
    }));
    gap(Dp(16.0));
    caption("1  Extended FAB (baseline) is superseded by the small extended FAB shown here.");
    gap(Dp(24.0));
    spec_table(
        vec![
            "Attribute".into(),
            "Baseline".into(),
            "Small (recommended)".into(),
        ],
        vec![
            vec![
                "Label type".into(),
                "Label large".into(),
                "Title medium".into(),
            ],
            vec!["Inner padding".into(), "Wider".into(), "Reduced".into()],
            vec![
                "Status".into(),
                "Not recommended".into(),
                "Recommended".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "Use an extended FAB for the most common or important action on a screen, and when the label is needed to understand the action. Choose a size that matches the density of the surrounding layout.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
                gap(Dp(16.0));
                text()
                    .content("Do: keep the label short and specific to the action.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("Create a new message").size(ExtendedFloatingActionButtonSize::Small);
                gap(Dp(16.0));
                text()
                    .content("Don't: repeat the whole task description; the floating action stays concise.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("Label the action, not the object: Compose, not New message composer screen.");
    gap(Dp(32.0));
    bullets(vec![
        "Keep one extended FAB per screen and reserve it for the primary action".into(),
        "Avoid covering content that people need while the action is visible".into(),
        "Prefer a FAB without a label when the icon alone is universally understood".into(),
    ]);
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "An extended FAB is a single container with a leading icon and a label. Keep the icon optional: when the label already names the action, the icon only supports recognition.",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn guidelines_color() {
    prose(
        "Use the default primary container tone for most screens. Secondary and tertiary tones separate a floating action from other primary-colored elements, and solid accent tones raise its emphasis.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for tone in [0_usize, 1, 2, 3] {
                    fab(TONES[tone])
                        .size(ExtendedFloatingActionButtonSize::Small)
                        .tone(tone);
                }
            });
    }));
    gap(Dp(16.0));
    caption("Keep one tone across a screen so the floating action stays identifiable.");
}

#[tessera]
fn placement() {
    prose(
        "Place an extended FAB at the bottom end of the screen, above the bottom navigation bar or any bottom sheet. Keep a 16dp margin from the screen edges and from other floating elements.",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(240.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    text()
                        .content("Content")
                        .color(scheme.on_surface_variant)
                        .style(type_style(14.0, 20.0));
                    spacer().modifier(Modifier::new().weight(1.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            spacer().modifier(Modifier::new().weight(1.0));
                            extended_floating_action_button("Compose")
                                .size(ExtendedFloatingActionButtonSize::Small)
                                .icon(filled::EDIT_SVG)
                                .expanded(true)
                                .container_color(scheme.primary_container)
                                .content_color(scheme.on_primary_container)
                                .accessibility_label("Compose")
                                .on_click(|| {});
                        });
                });
        });
    gap(Dp(16.0));
    caption("Extended FABs should have margins of 16dp.");
}

#[tessera]
fn responsive_layout() {
    prose(
        "Choose the size by window class. The small extended FAB suits compact and medium layouts, while the medium and large sizes give the action more presence in expanded layouts.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("Compact".into());
                gap(Dp(16.0));
                fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
                gap(Dp(32.0));
                subheading("Expanded".into());
                gap(Dp(16.0));
                fab("Compose").size(ExtendedFloatingActionButtonSize::Large);
            });
    }));
    gap(Dp(16.0));
    caption(
        "The same label at two size presets; the large size carries more weight in wide layouts.",
    );
}

#[tessera]
fn behavior() {
    prose(
        "An extended FAB can collapse to an icon-only container while content scrolls, then expand again when people stop or scroll back. Keep the action reachable in both states and never hide it behind scrolling content.",
    );
    gap(Dp(24.0));
    collapsible("Compose");
    gap(Dp(24.0));
    prose(
        "When the label collapses, keep the accessible name unchanged so assistive technology still announces the action. The container keeps its tone and elevation across the transition.",
    );
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to and activate the extended FAB".into(),
        "Identify the action from its label and icon".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Compose")
            .size(ExtendedFloatingActionButtonSize::Small)
            .counted(true);
    }));
    gap(Dp(16.0));
    caption(
        "The extended FAB exposes one action named by its label; activating it increments the counter.",
    );
}

#[tessera]
fn interaction_style() {
    prose(
        "Extended FABs are large targets by construction: the smallest container is 56dp high and the interactive target never drops below 48 × 48dp. Keep the label and icon contrast at 3:1 or higher against the container.",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                extended_floating_action_button("Compose")
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .expanded(true)
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .accessibility_label("Above three to one: Compose")
                    .on_click(|| {});
                spacer().modifier(Modifier::new().width(Dp(32.0)));
                extended_floating_action_button("Compose")
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .expanded(true)
                    .container_color(scheme.surface_container)
                    .content_color(scheme.outline)
                    .accessibility_label("Below three to one: Compose")
                    .on_click(|| {});
            });
    }));
    gap(Dp(16.0));
    caption("Above 3:1 on primary container versus a low-contrast pairing.");
    gap(Dp(24.0));
    prose(
        "The pressed and focused state layers use the content color, so a custom container keeps readable feedback only when its on-color is chosen deliberately.",
    );
}

#[tessera]
fn initial_focus() {
    prose(
        "Initial focus should land on the extended FAB when it is the only floating action on the screen, and otherwise follow the reading order of the page. The container itself is a single focus stop; its icon and label are not separate targets.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
    }));
    gap(Dp(16.0));
    caption("Press Tab to reach the action; the whole container receives focus once.");
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate to the extended FAB".into()],
            vec!["Space or Enter".into(), "Activate the extended FAB".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The extended FAB behaves like a button for keyboard input: a single focus stop, activated by Space or Enter. Focus is drawn as a ring around the container, not around the icon or label.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Compose")
            .size(ExtendedFloatingActionButtonSize::Small)
            .counted(true);
    }));
}

#[tessera]
fn labeling() {
    prose(
        "The accessible name of an extended FAB is its visible label, so the icon must not repeat it. Do not label the icon: it is decorative whenever the text already names the action.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                fab("Compose").size(ExtendedFloatingActionButtonSize::Small);
                gap(Dp(16.0));
                text()
                    .content("Do: one label, one announcement — “Compose, button”.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("Compose").size(ExtendedFloatingActionButtonSize::Small).tone(1);
                gap(Dp(16.0));
                text()
                    .content("Don't: “Compose, edit pencil icon, button” — the icon description duplicates the action.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "When the label collapses, keep the same accessible name. Provide a visible label on request rather than replacing the name with an icon description.",
    );
}
