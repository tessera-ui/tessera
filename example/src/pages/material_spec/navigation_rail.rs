//! Navigation rail specifications for primary destinations.
//!
//! ## Usage
//!
//! Compare collapsed and expanded rails, selection and rail composition.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::badge,
    boxed::boxed,
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::{filled, outlined},
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    navigation_rail::{modal_navigation_rail, navigation_rail, navigation_rail_item},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const NAMES: [&str; 5] = ["Home", "Inbox", "Saved", "Media", "Settings"];

#[tessera]
pub(super) fn navigation_rail_spec() {
    let modal_open = remember(|| false);
    let modal_selected = remember(|| 0usize);
    modal_navigation_rail()
        .is_open(modal_open.get())
        .accessibility_label("Mail navigation")
        .on_close_request(move || modal_open.set(false))
        .main_content(move || {
            article(
                "Navigation rail",
                "Navigation rails let people switch between UI views on mid-sized devices",
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
                        title: "Common layouts",
                        content: RenderSlot::new(layouts),
                    },
                    SpecSection {
                        title: "Baseline navigation rail",
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
                            content: RenderSlot::new(availability),
                        },
                        SpecSection {
                            title: "M3 Expressive update",
                            content: RenderSlot::new(expressive_update),
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
                            content: RenderSlot::new(guide_anatomy),
                        },
                        SpecSection {
                            title: "Placement",
                            content: RenderSlot::new(placement),
                        },
                        SpecSection {
                            title: "Adaptive design",
                            content: RenderSlot::new(adaptive_design),
                        },
                        SpecSection {
                            title: "Behavior",
                            content: RenderSlot::new(move || {
                                behavior(modal_open, modal_selected);
                            }),
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
                            content: RenderSlot::new(keyboard),
                        },
                        SpecSection {
                            title: "Labeling elements",
                            content: RenderSlot::new(labeling),
                        },
                    ],
                },
            ]);
        })
        .header(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label("Close modal navigation rail")
                .on_click(move || modal_open.set(false));
        })
        .content(move || {
            for (index, name) in NAMES.into_iter().enumerate() {
                navigation_rail_item()
                    .label(name)
                    .selected(modal_selected.get() == index)
                    .on_click(move || {
                        modal_selected.set(index);
                        modal_open.set(false);
                    })
                    .icon(move || {
                        destination_icon(index, modal_selected.get() == index);
                    });
            }
        });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {item}"))
                .style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn badge_number(number: usize) {
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

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            badge_number(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text().content(label.clone()).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn guidance_line(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
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
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn do_dont(#[prop(into)] good: String, #[prop(into)] bad: String) {
    column().children(move || {
        guidance_line("Do", good.clone(), DO_COLOR);
        gap(Dp(12.0));
        guidance_line("Don't", bad.clone(), DONT_COLOR);
    });
}

#[tessera]
fn destination_icon(index: usize, selected: bool) {
    icon()
        .painter(if selected {
            [
                filled::HOME_SVG,
                filled::MAIL_SVG,
                filled::BOOKMARK_SVG,
                filled::PLAY_ARROW_SVG,
                filled::SETTINGS_SVG,
            ][index]
        } else {
            [
                outlined::HOME_SVG,
                outlined::MAIL_SVG,
                outlined::BOOKMARK_BORDER_SVG,
                outlined::PLAY_ARROW_SVG,
                outlined::SETTINGS_SVG,
            ][index]
        })
        .size(Dp(24.0));
}

#[tessera]
fn rail_view(
    #[prop(into)] label: String,
    expanded: bool,
    count: usize,
    badge_destination: bool,
    with_header: bool,
) {
    let selected = remember(|| 0_usize);
    let caption_text = label.clone();
    let header = if with_header {
        Some(RenderSlot::new(|| {
            text().content("Mail").style(type_style(14.0, 20.0));
        }))
    } else {
        None
    };
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} navigation rail example")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            let header = header.clone();
            navigation_rail()
                .expanded(expanded)
                .header_optional(header)
                .content(move || {
                    for (index, name) in NAMES.into_iter().take(count).enumerate() {
                        if badge_destination && index == 1 {
                            navigation_rail_item()
                                .label(name)
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .badge_optional(Some(RenderSlot::new(|| {
                                    badge();
                                })))
                                .icon(move || {
                                    destination_icon(index, selected.get() == index);
                                });
                        } else {
                            navigation_rail_item()
                                .label(name)
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .icon(move || {
                                    destination_icon(index, selected.get() == index);
                                });
                        }
                    }
                });
            gap(Dp(16.0));
            text()
                .content(format_rail_status(&caption_text, selected.get()))
                .style(type_style(14.0, 20.0));
        });
}

fn format_rail_status(label: &str, selected: usize) -> String {
    let name = NAMES.get(selected).copied().unwrap_or(NAMES[0]);
    format!("{label} destination: {name}")
}

#[tessera]
fn rail_pair(count: usize, badge_destination: bool) {
    row()
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            rail_view("Collapsed", false, count, badge_destination, false);
            spacer().modifier(Modifier::new().width(Dp(40.0)));
            rail_view("Expanded", true, count, badge_destination, true);
        });
}

#[tessera]
fn overview() {
    prose(
        "Navigation rails let people switch between primary destinations on mid-sized, large and extra-large windows. The rail runs vertically along the leading edge, so navigation stays visible beside the content instead of covering it.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Use navigation rails in medium, expanded, large or extra-large windows".into(),
        "Show three to seven destinations plus an optional floating action button".into(),
        "Keep the rail in the same place on every screen of the app".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(16.0));
    caption(
        "The collapsed and expanded navigation rails match visually and can transition into each other",
    );
}

#[tessera]
fn availability() {
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
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "This page documents the native navigation_rail and navigation_rail_item components. The values below were verified against the published navigation rail specification.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "May 2025 — A collapsed and an expanded navigation rail replaced the baseline navigation rail. The expanded rail is meant to replace the navigation drawer; in Expressive interfaces, use an expanded navigation rail instead of a drawer.",
    );
    gap(Dp(24.0));
    subheading("Variants and naming".into());
    bullets(vec![
        "Baseline navigation rail is no longer recommended".into(),
        "Added a collapsed navigation rail that replaces the baseline rail".into(),
        "Added an expanded navigation rail that replaces the navigation drawer".into(),
    ]);
    gap(Dp(24.0));
    subheading("Configurations".into());
    bullets(vec![
        "Expanded rail modality: non-modal or modal".into(),
        "Expanded behavior: transition to collapsed rail, or hide when collapsed".into(),
    ]);
    gap(Dp(24.0));
    subheading("Color".into());
    bullets(vec![
        "Active label on vertical items changed from on surface variant to secondary".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(16.0));
    caption(
        "A collapsed and expanded navigation rail; both share the same colors, indicator and item order",
    );
}

#[tessera]
fn differences() {
    prose("Behavior: Predictive back interaction.");
    gap(Dp(16.0));
    prose("Color: New color mappings and compatibility with dynamic color.");
    gap(Dp(16.0));
    prose(
        "States: The active destination can be indicated with a pill shape in a contrasting color.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("M3", false, 4, false, false);
    }));
    gap(Dp(16.0));
    caption(
        "M2 communicated the active destination with icon color, weight and fill. M3 uses a pill-shaped active indicator.",
    );
}

#[tessera]
fn variants() {
    prose(
        "The navigation rail has two variants. A collapsed rail shows icons with short labels stacked vertically; an expanded rail widens the item container so icons and labels sit side by side.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Collapsed navigation rail");
        legend(2, "Expanded navigation rail");
    });
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec![
                "Collapsed navigation rail".into(),
                "—".into(),
                "Available".into(),
            ],
            vec![
                "Expanded navigation rail".into(),
                "—".into(),
                "Available".into(),
            ],
            vec![
                "Navigation rail (baseline)".into(),
                "Available".into(),
                "Not recommended. Use collapsed navigation rail.".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The baseline navigation rail is no longer recommended, and should be replaced by the collapsed navigation rail.",
    );
}

#[tessera]
fn configurations() {
    prose(
        "In compact windows, use a navigation bar instead of a rail. From the medium breakpoint up, the rail can be collapsed or expanded; an expanded rail is either standard (placed beside the content) or modal (overlaying the content).",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Collapsed", false, 4, false, false);
    }));
    gap(Dp(16.0));
    caption("Collapsed layout: 96dp wide, icons above labels");
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Expanded", true, 4, false, true);
    }));
    gap(Dp(16.0));
    caption("Expanded layout: 220dp and wider, icons beside labels");
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
                "Expanded layout".into(),
                "Standard (default)".into(),
                "Available as navigation drawer".into(),
                "Available".into(),
            ],
            vec![
                "Expanded layout".into(),
                "Modal".into(),
                "Available as modal navigation drawer".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "Browse the component elements, attributes, tokens, and their values. Color values below are the Default light scheme.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Container width (collapsed)".into(),
                "md.comp.navigation-rail.container.width".into(),
                "96dp".into(),
            ],
            vec![
                "Container width (expanded)".into(),
                "md.comp.navigation-rail-expanded.container.width".into(),
                "220–360dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "md.comp.navigation-rail.active-indicator".into(),
                "56 × 32dp, 16dp radius".into(),
            ],
            vec![
                "Active indicator color".into(),
                "md.comp.navigation-rail.active-indicator.color".into(),
                "Secondary container #E8DEF8".into(),
            ],
            vec![
                "Active icon color".into(),
                "md.comp.navigation-rail.active-icon.color".into(),
                "On secondary container #4A4458".into(),
            ],
            vec![
                "Active label text color".into(),
                "md.comp.navigation-rail.active-label-text.color".into(),
                "Secondary #625B71".into(),
            ],
            vec![
                "Inactive icon color".into(),
                "md.comp.navigation-rail.inactive-icon.color".into(),
                "On surface variant #49454F".into(),
            ],
            vec![
                "Inactive label text color".into(),
                "md.comp.navigation-rail.inactive-label-text.color".into(),
                "On surface variant #49454F".into(),
            ],
            vec![
                "Icon size".into(),
                "md.comp.navigation-rail.icon.size".into(),
                "24dp".into(),
            ],
            vec![
                "Label".into(),
                "md.comp.navigation-rail.label-text".into(),
                "Label medium (vertical) · Label large (horizontal)".into(),
            ],
            vec![
                "State layer".into(),
                "md.comp.navigation-rail.state-layer".into(),
                "Hover 8% · Focus 10% · Press 10%".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The published token set groups container and item values by interaction state: Enabled, Hovered, Focused and Pressed.",
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        rail_pair(4, true);
    }));
    gap(Dp(24.0));
    prose("Collapsed and expanded navigation rail elements:");
    gap(Dp(16.0));
    column().children(|| {
        legend(1, "Container");
        legend(2, "Menu (optional)");
        legend(3, "FAB or Extended FAB (optional)");
        legend(4, "Icon");
        legend(5, "Active indicator");
        legend(6, "Label text");
        legend(7, "Large badge (optional)");
        legend(8, "Large badge label (optional)");
        legend(9, "Small badge (optional)");
    });
    gap(Dp(16.0));
    caption(
        "The active indicator sits behind the selected icon and label; every destination keeps its label in both variants",
    );
}

#[tessera]
fn colors() {
    prose(
        "Navigation rail colors come from the container role, the secondary container indicator and the on surface variant inactive content.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, true);
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            panel(RenderSlot::new(|| {
                rail_pair(4, true);
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Container".into(), "Surface container (optional)".into()],
            vec!["Active icon".into(), "On secondary container".into()],
            vec!["Active indicator".into(), "Secondary container".into()],
            vec![
                "Active label".into(),
                "Secondary (vertical) · On secondary container (horizontal)".into(),
            ],
            vec!["Inactive icon".into(), "On surface variant".into()],
            vec!["Inactive label".into(), "On surface variant".into()],
            vec!["Badge container".into(), "Error".into()],
            vec!["Badge label".into(), "On error".into()],
            vec!["Small badge".into(), "Error".into()],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "States are visual representations used to communicate the status of a component or an interactive element. Destinations are either active or inactive; hover, focus and press add the standard state layers on top.",
    );
    gap(Dp(16.0));
    prose(
        "The navigation item's target area always spans the full width of the navigation rail, even if the item container hugs its contents.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(16.0));
    caption(
        "Enabled and selected destinations; hover, focus and press add 8% and 10% state layers",
    );
    gap(Dp(24.0));
    prose(
        "The native component does not expose a per-destination disabled state; hide a destination or disable its host when a destination must be unavailable.",
    );
}

#[tessera]
fn measurements() {
    spec_table(
        vec![
            "Element".into(),
            "Attribute".into(),
            "Tessera native".into(),
            "M3 reference".into(),
        ],
        vec![
            vec![
                "Container".into(),
                "Collapsed width".into(),
                "96dp".into(),
                "96dp (baseline rail: 80dp)".into(),
            ],
            vec![
                "Container".into(),
                "Expanded width".into(),
                "220dp minimum".into(),
                "220–360dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "Size".into(),
                "56 × 32dp".into(),
                "56 × 32dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "Corner radius".into(),
                "Capsule (16dp)".into(),
                "16dp".into(),
            ],
            vec!["Icon".into(), "Size".into(), "24dp".into(), "24dp".into()],
            vec![
                "Destination".into(),
                "Minimum height".into(),
                "64dp icon-top · 56dp icon-start".into(),
                "56dp".into(),
            ],
            vec![
                "Destination".into(),
                "Minimum target".into(),
                "48 × 48dp".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "Container".into(),
                "Top padding".into(),
                "44dp".into(),
                "36dp".into(),
            ],
            vec![
                "Indicator to label".into(),
                "Gap".into(),
                "4dp icon-top · 8dp icon-start".into(),
                "4dp".into(),
            ],
            vec![
                "Destinations".into(),
                "Recommended count".into(),
                "3–7".into(),
                "3–7".into(),
            ],
            vec![
                "Window".into(),
                "Breakpoint".into(),
                "600dp and wider".into(),
                "Medium and larger".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(16.0));
    caption(
        "Native values come from the component's own metrics; the M3 reference column records the published specification where the two differ",
    );
}

#[tessera]
fn layouts() {
    prose(
        "A rail can hold destinations only, a menu plus destinations, a floating action button plus destinations, or all three together. The same arrangement should hold in both the collapsed and expanded variants.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Destinations", false, 3, false, false);
    }));
    gap(Dp(16.0));
    caption("Destinations only");
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("With header", true, 3, false, true);
    }));
    gap(Dp(16.0));
    caption("Menu plus destinations; the header slot holds the menu control");
    gap(Dp(24.0));
    prose(
        "A floating action button can sit above the destinations in either variant. The native component renders it through the rail content, so it keeps the same leading alignment as the items.",
    );
}

#[tessera]
fn baseline() {
    prose(
        "The baseline navigation rail is no longer recommended. It used an 80dp container with a 56 × 32dp indicator and is replaced by the collapsed navigation rail documented above.",
    );
    gap(Dp(16.0));
    prose(
        "Keep using the collapsed layout described on this page; the difference between the baseline and collapsed rail is the container width and the active label color role.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Baseline".into(), "Collapsed rail".into()],
        vec![
            vec!["Container width".into(), "80dp".into(), "96dp".into()],
            vec![
                "Active label color".into(),
                "On surface variant".into(),
                "Secondary".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "The navigation rail can display navigation items, a menu, and a floating action button (FAB) in a vertical orientation.",
    );
    gap(Dp(16.0));
    prose(
        "There are two variants of navigation rails, collapsed and expanded, which can easily transform into each other when the menu button is selected.",
    );
    gap(Dp(24.0));
    subheading("Collapsed".into());
    prose(
        "The collapsed navigation rail runs along one side of the screen and shows an icon above a short label for each destination. It is the default for medium windows where horizontal space is limited.",
    );
    gap(Dp(16.0));
    prose(
        "Avoid using a rail in compact windows; compact windows should always use a navigation bar instead.",
    );
    gap(Dp(24.0));
    subheading("Expanded".into());
    prose(
        "The expanded navigation rail can be standard or modal, and should always open from a menu icon. An expanded rail can reveal secondary destinations that are not visible when collapsed.",
    );
    gap(Dp(16.0));
    prose(
        "The standard configuration is placed beside body content and is best for larger windows with plenty of available space. The modal configuration overlaps the body content and suits information dense layouts where space is limited.",
    );
    gap(Dp(24.0));
    do_dont(
        "Use a rail for three to seven top-level destinations",
        "Don’t use a rail for compact windows or for single tasks",
    );
}

#[tessera]
fn guide_anatomy() {
    prose(
        "A navigation rail contains a container, the destination items, an optional menu control and an optional floating action button.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, true);
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Container");
        legend(2, "Menu (optional)");
        legend(3, "FAB or Extended FAB (optional)");
        legend(4, "Icon");
        legend(5, "Active indicator");
        legend(6, "Label text");
        legend(7, "Large badge (optional)");
        legend(8, "Large badge label (optional)");
        legend(9, "Small badge (optional)");
    });
    gap(Dp(16.0));
    caption("The container holds every element; destinations keep their order in both variants");
}

#[tessera]
fn placement() {
    prose(
        "Place the rail along the leading edge of the window so navigation keeps a stable position while the content changes. Keep it in the same place, even on different screens of an app.",
    );
    gap(Dp(16.0));
    prose(
        "A standard expanded rail sits beside the body content; a modal expanded rail overlays it and needs a visible menu control to open and close.",
    );
    gap(Dp(24.0));
    do_dont(
        "Keep the rail in the same place across screens",
        "Don’t move the rail between the leading and trailing edges",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Expanded", true, 4, false, true);
    }));
    gap(Dp(16.0));
    caption("Use the menu icon to transition between collapsed and expanded navigation rails");
}

#[tessera]
fn adaptive_design() {
    prose(
        "Adaptive design allows an interface to respond or change based on context, such as the user, device and usage.",
    );
    gap(Dp(24.0));
    subheading("Compact".into());
    prose("Use a navigation bar instead of a navigation rail.");
    gap(Dp(24.0));
    subheading("Medium".into());
    prose(
        "Use a collapsed navigation rail, or a navigation bar when horizontal space matters more than vertical space.",
    );
    gap(Dp(24.0));
    subheading("Expanded, large and extra-large".into());
    prose(
        "Use an expanded navigation rail. On the largest windows it can stay expanded by default; on smaller windows it opens from the menu icon over the content.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_pair(4, false);
    }));
    gap(Dp(16.0));
    caption("Collapsed and expanded rails adapt to available window space");
}

#[tessera]
fn behavior(modal_open: State<bool>, modal_selected: State<usize>) {
    prose(
        "The menu control transitions the rail between collapsed and expanded. Selection is preserved across the transition: the same destination keeps its active indicator and filled icon.",
    );
    gap(Dp(16.0));
    prose(
        "Destinations remain visible and fixed while the adjacent page content scrolls vertically.",
    );
    gap(Dp(16.0));
    prose(
        "A modal expanded rail closes by selecting its scrim or a destination, and returns focus to the control that opened it.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(behavior_rail));
    gap(Dp(16.0));
    caption(
        "Use the menu button to expand or collapse the rail; the selected destination is preserved",
    );
    gap(Dp(32.0));
    subheading("Modal expanded rail".into());
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label("Open modal navigation rail")
                .on_click(move || modal_open.set(true));
            text()
                .content(format!(
                    "Current destination: {}",
                    NAMES[modal_selected.get()]
                ))
                .style(type_style(14.0, 20.0));
        });
    caption(
        "Open from the menu, then choose a destination, select the scrim or press Escape to close and return focus",
    );
    gap(Dp(32.0));
    subheading("Scrolling content".into());
    panel(RenderSlot::new(scrolling_rail));
    gap(Dp(16.0));
    caption("Scroll the messages beside the rail; the menu and destinations remain fixed");
}

#[tessera]
fn behavior_rail() {
    let expanded = remember(|| false);
    let selected = remember(|| 0usize);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            boxed()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Navigation),
                    label: Some("Interactive mail navigation".into()),
                    test_tag: Some("behavior-rail".into()),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    navigation_rail()
                        .expanded(expanded.get())
                        .header(move || {
                            icon_button()
                                .standard()
                                .icon(filled::MENU_SVG)
                                .accessibility_label(if expanded.get() {
                                    "Collapse navigation rail"
                                } else {
                                    "Expand navigation rail"
                                })
                                .modifier(Modifier::new().semantics(SemanticsArgs {
                                    test_tag: Some("behavior-rail-menu".into()),
                                    merge_descendants: true,
                                    ..Default::default()
                                }))
                                .on_click(move || {
                                    expanded.with_mut(|expanded| *expanded = !*expanded)
                                });
                        })
                        .content(move || {
                            for (index, name) in NAMES.into_iter().take(4).enumerate() {
                                boxed()
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        test_tag: Some(format!("behavior-rail-item-{index}")),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }))
                                    .children(move || {
                                        navigation_rail_item()
                                            .label(name)
                                            .selected(selected.get() == index)
                                            .on_click(move || selected.set(index))
                                            .icon(move || {
                                                destination_icon(index, selected.get() == index);
                                            });
                                    });
                            }
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format_rail_status(
                    if expanded.get() {
                        "Expanded"
                    } else {
                        "Collapsed"
                    },
                    selected.get(),
                ))
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    test_tag: Some(format!("behavior-rail-destination-{}", selected.get())),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn scrolling_rail() {
    let expanded = remember(|| false);
    let selected = remember(|| 0usize);
    let scroll = remember(ScrollableController::new);
    scrolling_rail_view(expanded, selected, scroll);
}

#[tessera]
fn scrolling_rail_view(
    expanded: State<bool>,
    selected: State<usize>,
    scroll: State<ScrollableController>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        row()
            .modifier(Modifier::new().fill_max_width().height(Dp(480.0)))
            .children(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_height().semantics(SemanticsArgs {
                        role: Some(Role::Navigation),
                        label: Some("Fixed mail navigation beside scrolling content".into()),
                        test_tag: Some("scrolling-rail".into()),
                        merge_descendants: true,
                        ..Default::default()
                    }))
                    .children(move || {
                        navigation_rail()
                            .expanded(expanded.get())
                            .header(move || {
                                icon_button()
                                    .standard()
                                    .icon(filled::MENU_SVG)
                                    .accessibility_label(if expanded.get() {
                                        "Collapse fixed rail"
                                    } else {
                                        "Expand fixed rail"
                                    })
                                    .on_click(move || expanded.with_mut(|value| *value = !*value))
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        test_tag: Some("scrolling-rail-menu".into()),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }));
                            })
                            .content(move || {
                                for (index, name) in NAMES.into_iter().take(4).enumerate() {
                                    boxed()
                                        .modifier(Modifier::new().semantics(SemanticsArgs {
                                            test_tag: Some(format!("scrolling-rail-item-{index}")),
                                            merge_descendants: true,
                                            ..Default::default()
                                        }))
                                        .children(move || {
                                            navigation_rail_item()
                                                .label(name)
                                                .selected(selected.get() == index)
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    destination_icon(
                                                        index,
                                                        selected.get() == index,
                                                    );
                                                });
                                        });
                                }
                            });
                    });
                column()
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .fill_max_size()
                            .vertical_scroll(scroll)
                            .padding_all(Dp(16.0))
                            .semantics(SemanticsArgs {
                                test_tag: Some("scrolling-body".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        for index in 0..12 {
                            boxed()
                                .alignment(Alignment::CenterStart)
                                .modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(96.0))
                                        .padding_all(Dp(12.0))
                                        .background_with_shape(
                                            scheme.surface_container_low,
                                            Shape::rounded_rectangle(Dp(12.0)),
                                        )
                                        .semantics(SemanticsArgs {
                                            test_tag: Some(format!("scrolling-message-{index}")),
                                            merge_descendants: true,
                                            ..Default::default()
                                        }),
                                )
                                .children(move || {
                                    text()
                                        .content(format!(
                                            "{} message {}",
                                            NAMES[selected.get()],
                                            index + 1
                                        ))
                                        .style(type_style(14.0, 20.0));
                                });
                            gap(Dp(12.0));
                        }
                    });
            });
        gap(Dp(16.0));
        text()
            .content(format_rail_status("Fixed rail", selected.get()))
            .style(type_style(14.0, 20.0))
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(format!("scrolling-rail-destination-{}", selected.get())),
                merge_descendants: true,
                ..Default::default()
            }));
    });
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using the assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate between navigation destinations".into(),
        "Select a particular navigation destination from a set".into(),
        "Get appropriate feedback based on input type".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Use cases", false, 4, false, false);
    }));
    gap(Dp(16.0));
    caption("Each destination exposes a label and a selected state");
}

#[tessera]
fn interaction_style() {
    prose(
        "Expose selection separately from focus. The active indicator, the active foreground color and the filled icon communicate the selected destination; hover, focus and press feedback communicate interaction.",
    );
    gap(Dp(16.0));
    prose(
        "Because the item target area spans the full rail width, the pointer and focus targets stay comfortable even though the icon and indicator hug the center.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Interaction", false, 4, true, false);
    }));
    gap(Dp(16.0));
    caption("Badges report counts without replacing the destination label");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Move focus into or out of the rail".into()],
            vec![
                "Up / Down arrows".into(),
                "Move focus between destinations".into(),
            ],
            vec![
                "Enter or Space".into(),
                "Activate the focused destination".into(),
            ],
            vec![
                "Escape".into(),
                "Close a modal expanded rail and restore focus".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Arrow keys move focus only; a destination changes only after Enter or Space, so browsing the list never navigates unexpectedly.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "Every destination needs a short label of one or two words. The label is announced together with the selected state, so the icon alone never carries the meaning.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Name the rail region so the destination list has context".into(),
        "Keep labels stable between the collapsed and expanded variants".into(),
        "Describe a badge in the destination's accessibility description when its count matters"
            .into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        rail_view("Labeling", false, 4, true, false);
    }));
    gap(Dp(16.0));
    caption("Short labels stay legible in both variants");
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition,
        testing::{
            LayoutSnapshot, LayoutTestSession, layout_test, pump_layout_test_session,
            pump_layout_test_session_with_input, start_layout_test_session,
        },
    };

    use super::*;

    fn click(
        session: &mut LayoutTestSession<impl Fn()>,
        snapshot: &LayoutSnapshot,
        tag: &str,
        frame: &mut u64,
    ) {
        let bounds = snapshot.node(tag).bounds();
        let cursor = Some(PxPosition::from_f32_xy(
            bounds.x.to_f32() + bounds.width.to_f32() * 0.5,
            bounds.y.to_f32() + bounds.height.to_f32() * 0.5,
        ));
        for content in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ] {
            *frame += 16_666_667;
            pump_layout_test_session_with_input(
                session,
                *frame,
                vec![PointerChange::new(
                    tessera_ui::time::Instant::now(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                cursor,
            );
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(behavior_rail);
    }

    #[test]
    fn menu_toggles_the_rail_without_resetting_the_selected_destination() {
        let mut session = start_layout_test_session(layout_test(fixture).viewport_px(400, 700));
        let mut frame = 0;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(96);
        click(&mut session, &snapshot, "behavior-rail-item-1", &mut frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail-destination-1").exists();
        click(&mut session, &snapshot, "behavior-rail-menu", &mut frame);
        frame += 400_000_000;
        pump_layout_test_session(&mut session, frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(220);
        snapshot.node("behavior-rail-destination-1").exists();
        click(&mut session, &snapshot, "behavior-rail-menu", &mut frame);
        frame += 400_000_000;
        pump_layout_test_session(&mut session, frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(96);
        snapshot.node("behavior-rail-destination-1").exists();
    }

    thread_local! {
        static SCROLL: std::cell::Cell<Option<State<ScrollableController>>> = const { std::cell::Cell::new(None) };
        static EXPANDED: std::cell::Cell<Option<State<bool>>> = const { std::cell::Cell::new(None) };
    }

    #[tessera]
    fn scrolling_fixture() {
        let expanded = remember(|| false);
        let selected = remember(|| 0usize);
        let scroll = remember(ScrollableController::new);
        SCROLL.set(Some(scroll));
        EXPANDED.set(Some(expanded));
        material_theme().child(move || {
            scrolling_rail_view(expanded, selected, scroll);
        });
    }

    #[test]
    fn scrolling_body_keeps_menu_and_destinations_fixed_and_selectable_in_both_variants() {
        let mut session =
            start_layout_test_session(layout_test(scrolling_fixture).viewport_px(600, 600));
        let mut frame = 0;
        pump_layout_test_session(&mut session, frame);
        for expanded in [false, true] {
            EXPANDED.get().unwrap().set(expanded);
            SCROLL
                .get()
                .unwrap()
                .with_mut(|scroll| scroll.set_scroll_position(PxPosition::ZERO));
            frame += 16_666_667;
            pump_layout_test_session(&mut session, frame);
            frame += 400_000_000;
            let before = pump_layout_test_session(&mut session, frame);
            before
                .node("scrolling-rail")
                .size(if expanded { 220 } else { 96 }, 480);
            let menu = before.node("scrolling-rail-menu").bounds();
            let destination = before.node("scrolling-rail-item-3").bounds();
            let message = before.node("scrolling-message-0").bounds();
            assert!(destination.y.0 + destination.height.0 <= 480);
            SCROLL.get().unwrap().with_mut(|scroll| {
                scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -200.0));
            });
            frame += 16_666_667;
            let scrolled = pump_layout_test_session(&mut session, frame);
            assert_eq!(scrolled.node("scrolling-rail-menu").bounds(), menu);
            assert_eq!(scrolled.node("scrolling-rail-item-3").bounds(), destination);
            assert_eq!(
                scrolled.node("scrolling-message-0").bounds().y.0,
                message.y.0 - 200
            );
            click(&mut session, &scrolled, "scrolling-rail-item-3", &mut frame);
            frame += 16_666_667;
            pump_layout_test_session(&mut session, frame)
                .node("scrolling-rail-destination-3")
                .exists();
        }
    }

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn gallery_fixture() {
        material_theme().child(|| {
            rail_view("Accessible figure", false, 4, false, false);
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for gallery semantics and accessibility focus"]
    fn gallery_and_behavior_wrappers_keep_destinations_visible_and_focus_separate_from_selection() {
        use tessera_ui::{
            HeadlessConfig, HeadlessRenderer,
            accesskit::{Action, ActionRequest, TreeId},
        };

        for (component, menu) in [
            (gallery_fixture as fn(), None),
            (fixture as fn(), Some("Expand navigation rail")),
            (scrolling_fixture as fn(), Some("Expand fixed rail")),
        ] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(component),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 600,
                    height: 600,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(6);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            let destinations: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab && !node.is_hidden())
                .collect();
            assert_eq!(
                destinations.len(),
                4,
                "every destination must survive wrapper semantics"
            );
            for label in NAMES.into_iter().take(4) {
                assert!(
                    destinations
                        .iter()
                        .any(|(_, node)| node.label() == Some(label))
                );
            }
            if let Some(menu) = menu {
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.role() == Role::Button
                            && node.label() == Some(menu)
                            && !node.is_hidden())
                );
            }
            let saved = destinations
                .iter()
                .find(|(_, node)| node.label() == Some("Saved"))
                .unwrap()
                .0;
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node: saved,
                data: None,
            }));
            renderer.render_frames(4);
            let focused = renderer.accessibility_snapshot().unwrap();
            assert_eq!(focused.focus, saved);
            let tabs: Vec<_> = focused
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab)
                .collect();
            assert_eq!(
                tabs.iter()
                    .find(|(_, node)| node.label() == Some("Home"))
                    .unwrap()
                    .1
                    .is_selected(),
                Some(true)
            );
            assert_eq!(
                tabs.iter()
                    .find(|(_, node)| node.label() == Some("Saved"))
                    .unwrap()
                    .1
                    .is_selected(),
                Some(false)
            );
        }
    }
}
