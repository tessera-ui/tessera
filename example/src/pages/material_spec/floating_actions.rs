//! Floating action button specifications.
//!
//! ## Usage
//!
//! Compare FAB sizes, tones, placement and interaction states.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    floating_action_button::{FloatingActionButtonSize, floating_action_button},
    flow_row::flow_row,
    icon::icon,
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

const SIZES: [FloatingActionButtonSize; 4] = [
    FloatingActionButtonSize::Small,
    FloatingActionButtonSize::Standard,
    FloatingActionButtonSize::Medium,
    FloatingActionButtonSize::Large,
];

const TONES: [&str; 6] = [
    "Primary container",
    "Secondary container",
    "Tertiary container",
    "Primary",
    "Secondary",
    "Tertiary",
];

fn size_name(size: FloatingActionButtonSize) -> &'static str {
    match size {
        FloatingActionButtonSize::Small => "Small",
        FloatingActionButtonSize::Standard => "FAB",
        FloatingActionButtonSize::Medium => "Medium",
        FloatingActionButtonSize::Large => "Large",
    }
}

fn size_height(size: FloatingActionButtonSize) -> &'static str {
    match size {
        FloatingActionButtonSize::Small => "40dp",
        FloatingActionButtonSize::Standard => "56dp",
        FloatingActionButtonSize::Medium => "80dp",
        FloatingActionButtonSize::Large => "96dp",
    }
}

fn size_icon(size: FloatingActionButtonSize) -> f64 {
    match size {
        FloatingActionButtonSize::Small => 24.0,
        FloatingActionButtonSize::Standard => 24.0,
        FloatingActionButtonSize::Medium => 28.0,
        FloatingActionButtonSize::Large => 36.0,
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
pub(super) fn floating_actions_spec() {
    article(
        "Floating action buttons (FABs)",
        "Floating action buttons (FABs) help people take primary actions",
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
                title: "Baseline tokens & specs",
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
                    title: "Actions",
                    content: RenderSlot::new(actions),
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
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "Behaviors",
                    content: RenderSlot::new(behaviors),
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
                    title: "Focus",
                    content: RenderSlot::new(focus),
                },
                SpecSection {
                    title: "Layout & position",
                    content: RenderSlot::new(layout_position),
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

/// Renders one FAB with optional configuration overrides.
#[tessera]
fn fab(
    #[prop(into)] label: String,
    size: Option<FloatingActionButtonSize>,
    tone: Option<usize>,
    enabled: Option<bool>,
    counted: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let size = size.unwrap_or(FloatingActionButtonSize::Standard);
    let (container, content) = tone_colors(&scheme, tone.unwrap_or(0));
    let activations = remember(|| 0_u32);
    let spoken = label.clone();
    let icon_size = size_icon(size);
    column().children(move || {
        floating_action_button()
            .size(size)
            .container_color(container)
            .content_color(content)
            .ripple_color(content)
            .enabled(enabled.unwrap_or(true))
            .accessibility_label(spoken.clone())
            .on_click(move || activations.set(activations.get() + 1))
            .content(move || {
                icon().painter(filled::ADD_SVG).size(Dp(icon_size));
            });
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
        "Floating action buttons help people take a primary action. Use one FAB for the most common or important action on a screen, keep its icon clear, and let it persist while content scrolls.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Use a FAB for the most common or important action on a screen".into(),
        "Make sure the icon in a FAB is clear and understandable".into(),
        "FABs persist on the screen when content is scrolling".into(),
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
    prose("Small · FAB · Medium FAB · Large FAB");
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
        "Tessera examples on this page use the native floating_action_button component. Four sizes, six color tones, elevation and state-layer feedback are implemented, together with an accessibility label that names the action.",
    );
}

#[tessera]
fn expressive() {
    prose(
        "May 2025: the FAB family gained small, medium, and large sizes, and the surface color styles are no longer recommended. Use the primary container tone with a clear icon.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Change".into(), "Details".into()],
        vec![
            vec![
                "Sizes".into(),
                "Small (40dp), FAB (56dp), medium (80dp), and large (96dp)".into(),
            ],
            vec![
                "Color".into(),
                "Primary container is the default; secondary and tertiary tones are available"
                    .into(),
            ],
            vec![
                "Baseline".into(),
                "Surface color styles remain available but are no longer recommended".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        fab("Add").size(FloatingActionButtonSize::Standard);
    }));
    caption("The 56dp FAB with a primary container remains the recommended default.");
}

#[tessera]
fn differences() {
    prose(
        "M2: the FAB used a fixed 56dp container, a single accent color, and elevation 6. M3 introduced color roles, elevation levels tied to interaction, and a state layer that follows the icon color.",
    );
    gap(Dp(16.0));
    prose(
        "M3 Expressive: the family grew to four sizes, the icon scales with the container, and the corner radius follows the size instead of staying fixed.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "M2".into(), "M3 Expressive".into()],
        vec![
            vec!["Size".into(), "56dp".into(), "40 / 56 / 80 / 96dp".into()],
            vec!["Icon".into(), "24dp".into(), "24 / 28 / 36dp".into()],
            vec![
                "Corner radius".into(),
                "28dp".into(),
                "12 / 16 / 20 / 28dp".into(),
            ],
            vec![
                "Color".into(),
                "Accent fill".into(),
                "Primary, secondary, or tertiary container".into(),
            ],
        ],
    );
}

#[tessera]
fn variants() {
    prose(
        "The FAB family has four sizes that share one anatomy. The 56dp FAB is the default; small suits dense layouts and the larger sizes give the action more presence.",
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
    caption("1 Small FAB · 2 FAB · 3 Medium FAB · 4 Large FAB");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["Small FAB".into(), "--".into(), "Available".into()],
            vec!["FAB".into(), "Available".into(), "Available".into()],
            vec!["Medium FAB".into(), "--".into(), "Available".into()],
            vec!["Large FAB".into(), "--".into(), "Available".into()],
            vec![
                "Surface FAB (baseline)".into(),
                "Available".into(),
                "Not recommended".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    prose(
        "Choose a size and a container tone. The icon scales with the size, and the container keeps the same corner treatment across tones.",
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
                                .size(FloatingActionButtonSize::Standard)
                                .tone(tone);
                        }
                    });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec!["Configuration".into(), "Options".into()],
        vec![
            vec!["Size".into(), "Small · FAB · medium · large".into()],
            vec![
                "Container tones".into(),
                "Primary container · secondary container · tertiary container".into(),
            ],
            vec![
                "Accent tones".into(),
                "Primary · secondary · tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "FAB tokens are organized by size and color. Values below follow the native component defaults used on this page.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "md.comp.fab.primary-container.color".into(),
                "Primary container".into(),
            ],
            vec![
                "md.comp.fab.primary-container.icon.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.fab.secondary-container.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "md.comp.fab.tertiary-container.color".into(),
                "Tertiary container".into(),
            ],
            vec![
                "md.comp.fab.container.height".into(),
                "40dp small · 56dp · 80dp medium · 96dp large".into(),
            ],
            vec![
                "md.comp.fab.container.width".into(),
                "Matches the container height".into(),
            ],
            vec![
                "md.comp.fab.icon.size".into(),
                "24dp · 24dp · 28dp · 36dp".into(),
            ],
            vec![
                "md.comp.fab.container.shape".into(),
                "12dp · 16dp · 20dp · 28dp corners".into(),
            ],
            vec![
                "md.comp.fab.container.elevation".into(),
                "Level 3 · level 4 when hovered".into(),
            ],
            vec![
                "md.comp.fab.state-layer".into(),
                "Icon color at 8% hover and 10% focus or press".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Keep the state layer color equal to the icon color when a non-default color mapping is used, so feedback stays visible on every tone.",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "A FAB is a single container with one icon. It has no visible label; the icon carries the meaning and the accessibility label names the action.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column().children(|| {
            fab("Add").size(FloatingActionButtonSize::Standard);
            gap(Dp(32.0));
            for line in ["1  Container", "2  Icon"] {
                text()
                    .content(line)
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption("One container, one icon, one action.");
}

#[tessera]
fn colors() {
    prose(
        "Container tones map to the primary, secondary, and tertiary color roles, either as containers or as solid accents. Each tone pairs with its matching on-color for the icon.",
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
                                            fab(name).size(FloatingActionButtonSize::Standard).tone(tone);
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
    prose(
        "Surface color styles are still available but no longer recommended; prefer a container tone so the action reads as primary.",
    );
}

#[tessera]
fn states() {
    prose(
        "States communicate elevation and interaction. A FAB rests at level 3, rises to level 4 on hover, keeps level 3 while focused or pressed, and sharpens its state layer as interaction increases.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("Enabled").size(FloatingActionButtonSize::Standard);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("Disabled").size(FloatingActionButtonSize::Standard).enabled(false);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("Accent").size(FloatingActionButtonSize::Standard).tone(3);
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
}

#[tessera]
fn measurements() {
    prose(
        "Height, icon size, and corner radius scale together. Reserve 16dp margins around a floating action, and keep a 48dp minimum interactive target for the smaller sizes.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Attribute".into(),
            "Small".into(),
            "FAB".into(),
            "Medium".into(),
            "Large".into(),
        ],
        vec![
            vec![
                "Container".into(),
                "40dp".into(),
                "56dp".into(),
                "80dp".into(),
                "96dp".into(),
            ],
            vec![
                "Icon".into(),
                "24dp".into(),
                "24dp".into(),
                "28dp".into(),
                "36dp".into(),
            ],
            vec![
                "Corner radius".into(),
                "12dp".into(),
                "16dp".into(),
                "20dp".into(),
                "28dp".into(),
            ],
            vec![
                "Padding around icon".into(),
                "8dp".into(),
                "16dp".into(),
                "26dp".into(),
                "30dp".into(),
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
                    fab(format!("{} {}", size_name(size), size_height(size))).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("Container sizes of the small, regular, medium, and large FABs.");
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Outer margin".into(), "16dp".into()],
            vec!["Minimum interactive target".into(), "48 × 48dp".into()],
            vec![
                "Icon to container".into(),
                "Centered in the container".into(),
            ],
        ],
    );
}

#[tessera]
fn baseline_tokens() {
    prose(
        "Baseline FAB token sets are still published, but the surface color styles are no longer recommended in the expressive update. Prefer the primary container mapping shown on this page.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Baseline value".into()],
        vec![
            vec![
                "md.comp.fab.surface-container.color".into(),
                "Surface container high".into(),
            ],
            vec![
                "md.comp.fab.surface-container.icon.color".into(),
                "Primary".into(),
            ],
            vec!["md.comp.fab.container.height".into(), "56dp".into()],
            vec!["md.comp.fab.container.shape".into(), "16dp corners".into()],
            vec!["md.comp.fab.icon.size".into(), "24dp".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Both baseline and expressive mappings keep the same interaction model: elevation level 3 at rest, level 4 on hover, and a state layer that matches the icon color.",
    );
}

#[tessera]
fn usage() {
    prose(
        "Use one FAB for the most common or important action on a screen. Keep the icon clear and understandable, and avoid covering content that people need while the action is visible.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                fab("Add").size(FloatingActionButtonSize::Standard);
                gap(Dp(16.0));
                text()
                    .content("Do: one FAB with a familiar icon for the primary action.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("Add").size(FloatingActionButtonSize::Standard);
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        fab("Edit").size(FloatingActionButtonSize::Standard).tone(2);
                    });
                gap(Dp(16.0));
                text()
                    .content("Don't: two FABs competing for equally important actions.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("A FAB represents one action, not a collection of actions.");
    gap(Dp(32.0));
    bullets(vec![
        "Do not use a FAB for destructive or dismissive actions".into(),
        "Keep the icon understandable without extra explanation".into(),
        "Prefer a FAB menu when several related actions belong together".into(),
    ]);
}

#[tessera]
fn actions() {
    prose(
        "A FAB performs its action immediately. When several related actions belong together, use a FAB menu, and when the action needs a visible label, use an extended FAB instead.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Component".into(), "Use when".into()],
        vec![
            vec![
                "FAB".into(),
                "One primary action with a universally understood icon".into(),
            ],
            vec![
                "Extended FAB".into(),
                "The action needs a visible text label".into(),
            ],
            vec![
                "FAB menu".into(),
                "Two to six related actions belong to one trigger".into(),
            ],
        ],
    );
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "The FAB is a single container with a centered icon. Keep the icon simple and recognizable, and place the accessible label on the container rather than on the icon.",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn guidelines_color() {
    prose(
        "Default to the primary container tone. Secondary and tertiary tones separate a floating action from other primary-colored elements, while solid accent tones raise its emphasis.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for tone in [0_usize, 1, 2, 3] {
                    fab(TONES[tone])
                        .size(FloatingActionButtonSize::Standard)
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
        "Place a FAB at the bottom end of the screen, above the bottom navigation bar or any bottom sheet. Keep a 16dp margin from the screen edges and from other floating elements.",
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
                            floating_action_button()
                                .size(FloatingActionButtonSize::Standard)
                                .container_color(scheme.primary_container)
                                .content_color(scheme.on_primary_container)
                                .ripple_color(scheme.on_primary_container)
                                .accessibility_label("Add")
                                .on_click(|| {})
                                .content(|| {
                                    icon().painter(filled::ADD_SVG).size(Dp(24.0));
                                });
                        });
                });
        });
    gap(Dp(16.0));
    caption("FABs should have margins of 16dp and must not cover the navigation bar.");
}

#[tessera]
fn adaptive() {
    prose(
        "Choose the size by window class. The small and regular FABs suit compact layouts, while the medium and large sizes give the action more presence in expanded layouts.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("Compact".into());
                gap(Dp(16.0));
                fab("Add").size(FloatingActionButtonSize::Standard);
                gap(Dp(32.0));
                subheading("Expanded".into());
                gap(Dp(16.0));
                fab("Add").size(FloatingActionButtonSize::Large);
            });
    }));
    gap(Dp(16.0));
    caption("The same action at two size presets.");
}

#[tessera]
fn behaviors() {
    prose(
        "A FAB can hide while content scrolls down and reappear when scrolling up, so it never blocks the content people are reading. Keep it reachable and avoid animating it on every frame.",
    );
    gap(Dp(24.0));
    let visible = remember(|| true);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .selected(!visible.get())
                    .accessibility_label("Hide or show the floating action")
                    .on_click(move || visible.set(!visible.get()))
                    .child(move || {
                        text().content(if visible.get() {
                            "Hide FAB"
                        } else {
                            "Show FAB"
                        });
                    });
                gap(Dp(32.0));
                if visible.get() {
                    fab("Add")
                        .size(FloatingActionButtonSize::Standard)
                        .counted(true);
                } else {
                    text()
                        .content("FAB hidden while scrolling")
                        .style(type_style(14.0, 20.0));
                }
            });
    }));
    gap(Dp(24.0));
    prose(
        "Hiding is a scroll behavior, not a state change: the action keeps its position and its accessible name when it returns.",
    );
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to and activate the FAB".into(),
        "Identify the action from its accessible label".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Add")
            .size(FloatingActionButtonSize::Standard)
            .counted(true);
    }));
    gap(Dp(16.0));
    caption(
        "The FAB exposes one action named by its accessibility label; activating it increments the counter.",
    );
}

#[tessera]
fn interaction_style() {
    prose(
        "The FAB is a large target: the smallest container is 40dp and the interactive target never drops below 48 × 48dp. Keep the icon contrast at 3:1 or higher against the container.",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                floating_action_button()
                    .size(FloatingActionButtonSize::Standard)
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .ripple_color(scheme.on_primary_container)
                    .accessibility_label("Above three to one: add")
                    .on_click(|| {})
                    .content(|| {
                        icon().painter(filled::ADD_SVG).size(Dp(24.0));
                    });
                spacer().modifier(Modifier::new().width(Dp(32.0)));
                floating_action_button()
                    .size(FloatingActionButtonSize::Standard)
                    .container_color(scheme.surface_container)
                    .content_color(scheme.outline)
                    .ripple_color(scheme.outline)
                    .accessibility_label("Below three to one: add")
                    .on_click(|| {})
                    .content(|| {
                        icon().painter(filled::ADD_SVG).size(Dp(24.0));
                    });
            });
    }));
    gap(Dp(16.0));
    caption("Above 3:1 on primary container versus a low-contrast pairing.");
}

#[tessera]
fn focus() {
    prose(
        "A FAB is a single focus stop. Focus is drawn as a ring around the container, and it should follow the reading order of the screen rather than jumping to the corner first.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Add").size(FloatingActionButtonSize::Standard);
    }));
    gap(Dp(16.0));
    caption("Press Tab to reach the action; the container receives focus once.");
}

#[tessera]
fn layout_position() {
    prose(
        "Keep the FAB clear of navigation bars, sheets, and snackbars. It should stay in the same corner across screens, so people can find it without searching.",
    );
    gap(Dp(24.0));
    placement();
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate to the FAB".into()],
            vec!["Space or Enter".into(), "Activate the FAB".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The FAB behaves like a button for keyboard input: one focus stop, activated by Space or Enter, with a focus ring around the container rather than around the icon.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("Add")
            .size(FloatingActionButtonSize::Standard)
            .counted(true);
    }));
}

#[tessera]
fn labeling() {
    prose(
        "A FAB has no visible label, so its accessibility label must name the action. Do not describe the icon; describe what activating it does.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                fab("Add").size(FloatingActionButtonSize::Standard);
                gap(Dp(16.0));
                text()
                    .content("Do: “Add, button” — the label names the action.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("Add").size(FloatingActionButtonSize::Standard).tone(1);
                gap(Dp(16.0));
                text()
                    .content(
                        "Don't: “Plus icon” — describing the glyph does not explain the action.",
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "Keep the same label when the FAB hides and returns, and provide a tooltip only as a visual hint — the accessible name stays on the container.",
    );
}
