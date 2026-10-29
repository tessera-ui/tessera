//! Navigation bar specifications for primary destinations.
//!
//! ## Usage
//!
//! Compare destination counts, label behavior, badges and selection feedback.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    badge::{badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::{filled, outlined},
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    navigation_bar::{NavigationBarLabelBehavior, navigation_bar, navigation_bar_item},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const NAMES: [&str; 5] = ["Home", "Inbox", "Saved", "Media", "Settings"];

#[tessera]
pub(super) fn navigation_bar_spec() {
    article(
        "Navigation bar",
        "Navigation bars let people switch between primary destinations",
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
                    title: "Color",
                    content: RenderSlot::new(guide_color),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive_design),
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
                    content: RenderSlot::new(keyboard),
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

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            badge(number);
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
fn bar_demo(
    #[prop(into)] label: String,
    count: usize,
    label_behavior: NavigationBarLabelBehavior,
    badge_count: u32,
) {
    let selected = remember(|| 0_usize);
    let caption_text = label.clone();
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{label} navigation bar example")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            navigation_bar().content(move || {
                for (index, name) in NAMES.into_iter().take(count).enumerate() {
                    if index == 1 {
                        if badge_count > 0 {
                            navigation_bar_item()
                                .label(name)
                                .label_behavior(label_behavior)
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .icon(move || {
                                    let badge_text = badge_count.to_string();
                                    badged_box()
                                        .modifier(Modifier::new().size(Dp(32.0), Dp(32.0)))
                                        .badge(move || {
                                            let badge_text = badge_text.clone();
                                            badge_with_content()
                                                .container_color(
                                                    use_context::<MaterialTheme>()
                                                        .unwrap()
                                                        .get()
                                                        .color_scheme
                                                        .error,
                                                )
                                                .modifier(Modifier::new())
                                                .content(move || {
                                                    text().content(badge_text.clone());
                                                });
                                        })
                                        .content(move || {
                                            destination_icon(index, selected.get() == index);
                                        });
                                });
                        } else {
                            navigation_bar_item()
                                .label(name)
                                .label_behavior(label_behavior)
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .icon(move || {
                                    destination_icon(index, selected.get() == index);
                                });
                        }
                    } else {
                        navigation_bar_item()
                            .label(name)
                            .label_behavior(label_behavior)
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
                .content(format!("{caption_text} destination: {}", NAMES[selected.get()]))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn overview() {
    prose(
        "Navigation bars let people switch between primary destinations in an app. They appear at the bottom of the screen and stay visible across the top-level views.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Use for three to five primary destinations".into(),
        "Keep the bar visible across top-level views".into(),
        "Show the active destination with an indicator and a selected icon".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Overview",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("A navigation bar shows the app's primary destinations and highlights the active one");
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
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Available".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native navigation_bar and navigation_bar_item components with three to five destinations, label behavior and badges.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "The M3 navigation bar keeps the same structure as the baseline: a container with an active indicator behind the selected destination's icon, paired with a label.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Active indicator 56 × 32dp in the native component (M3 reference: 64 × 32dp, 16dp radius)".into(),
        "Selected icon switches from outlined to filled".into(),
        "Labels can always show, or appear only for the selected destination".into(),
        "Badges can mark a destination with unread activity".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Expressive",
            5,
            NavigationBarLabelBehavior::SelectedOnly,
            3,
        );
    }));
    gap(Dp(16.0));
    caption("Five destinations with selected-only labels and a badge on Inbox");
}

#[tessera]
fn differences() {
    bullets(vec![
        "M2: bottom navigation with shifting or fixed items and text-only states".into(),
        "M3: a persistent container, a pill-shaped active indicator behind the icon, and filled selected icons".into(),
        "M3 labels can stay visible or appear only on the selected destination".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Differences",
            3,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("M3 navigation bars highlight the active destination with an indicator and a filled icon");
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        bar_demo(
            "Variant",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("There is one navigation bar variant, holding three to five destinations");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![vec!["Navigation bar".into(), "Available".into(), "Available".into()]],
    );
}

#[tessera]
fn configurations() {
    let count = remember(|| 4_usize);
    let badges = remember(|| true);
    let selected_only = remember(|| false);
    column().children(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                button()
                    .tonal()
                    .accessibility_label("Cycle destination count")
                    .on_click(move || {
                        count.set(if count.get() == 5 { 3 } else { count.get() + 1 });
                    })
                    .child(move || {
                        text().content(format!("Destinations: {}", count.get()));
                    });
                button()
                    .tonal()
                    .selected(selected_only.get())
                    .accessibility_label("Toggle selected-only labels")
                    .on_click(move || selected_only.set(!selected_only.get()))
                    .child(move || {
                        text().content(if selected_only.get() {
                            "Labels: selected only"
                        } else {
                            "Labels: always"
                        });
                    });
                button()
                    .tonal()
                    .selected(badges.get())
                    .accessibility_label("Toggle Inbox badge")
                    .on_click(move || badges.set(!badges.get()))
                    .child(move || {
                        text().content(if badges.get() {
                            "Badge: 3"
                        } else {
                            "Badge: off"
                        });
                    });
            });
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            bar_demo(
                "Configured",
                count.get(),
                if selected_only.get() {
                    NavigationBarLabelBehavior::SelectedOnly
                } else {
                    NavigationBarLabelBehavior::AlwaysShow
                },
                if badges.get() { 3 } else { 0 },
            );
        }));
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Container".into(),
                "md.comp.navigation-bar.container.color".into(),
                "Surface container".into(),
            ],
            vec![
                "Container".into(),
                "md.comp.navigation-bar.container.height".into(),
                "64dp native · 80dp M3 reference".into(),
            ],
            vec![
                "Active indicator".into(),
                "md.comp.navigation-bar.active-indicator.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "Active indicator".into(),
                "md.comp.navigation-bar.active-indicator.shape".into(),
                "56 × 32dp · full round".into(),
            ],
            vec![
                "Active icon".into(),
                "md.comp.navigation-bar.active-icon.color".into(),
                "On secondary container".into(),
            ],
            vec![
                "Active label".into(),
                "md.comp.navigation-bar.active-label-text.color".into(),
                "Secondary".into(),
            ],
            vec![
                "Inactive icon and label".into(),
                "md.comp.navigation-bar.inactive-icon.color".into(),
                "On surface variant".into(),
            ],
            vec![
                "Icon size".into(),
                "md.comp.navigation-bar.icon.size".into(),
                "24dp".into(),
            ],
            vec![
                "Label".into(),
                "md.comp.navigation-bar.label-text".into(),
                "Label medium".into(),
            ],
            vec![
                "State layer".into(),
                "md.comp.navigation-bar.state-layer".into(),
                "Hover 8% · Focus 10% · Press 10%".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        bar_demo(
            "Anatomy",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Container");
        legend(2, "Active indicator");
        legend(3, "Icon");
        legend(4, "Active label");
        legend(5, "Inactive icon");
        legend(6, "Inactive label");
    });
    gap(Dp(16.0));
    caption("The active indicator sits behind the selected icon; labels describe each destination");
}

#[tessera]
fn colors() {
    prose(
        "Navigation bar colors come from the surface container role, the secondary container indicator and the on surface variant inactive content.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Light",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            panel(RenderSlot::new(|| {
                bar_demo(
                    "Dark",
                    4,
                    NavigationBarLabelBehavior::AlwaysShow,
                    0,
                );
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Container".into(), "Surface container".into()],
            vec!["Active indicator".into(), "Secondary container".into()],
            vec!["Active icon".into(), "On secondary container".into()],
            vec!["Active label".into(), "Secondary".into()],
            vec!["Inactive icon and label".into(), "On surface variant".into()],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "Destinations show an active or inactive state; the selected destination gains the indicator, a filled icon and its active colors. Hover, focus and press add the standard state layers.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "States",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("Active and inactive destinations; hover, focus and press add state layers");
    gap(Dp(24.0));
    prose(
        "The native component does not expose a per-destination disabled state; hide a destination or use a disabled host when a destination must be unavailable.",
    );
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Element".into(), "Attribute".into(), "Value".into()],
        vec![
            vec![
                "Bar container".into(),
                "Height".into(),
                "64dp native · 80dp M3 reference".into(),
            ],
            vec![
                "Active indicator".into(),
                "Size".into(),
                "56 × 32dp native · 64 × 32dp reference".into(),
            ],
            vec![
                "Active indicator".into(),
                "Corner radius".into(),
                "16dp (full round)".into(),
            ],
            vec!["Icon".into(), "Size".into(), "24dp".into()],
            vec![
                "Label".into(),
                "Typography".into(),
                "Label medium".into(),
            ],
            vec![
                "Destination".into(),
                "Minimum target".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "Indicator to label".into(),
                "Gap".into(),
                "4dp".into(),
            ],
            vec![
                "Destinations".into(),
                "Recommended count".into(),
                "3–5".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Measurements",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("Native values are shown first; the M3 reference values are noted where the component differs");
}

#[tessera]
fn usage() {
    prose(
        "Use a navigation bar for three to five primary destinations that are equal in importance. People can switch between top-level views without losing their place.",
    );
    gap(Dp(24.0));
    do_dont(
        "Use three to five primary destinations",
        "Don’t use a navigation bar for secondary navigation or for actions",
    );
    gap(Dp(24.0));
    prose(
        "Keep the bar visible across the app's top-level views, and keep each destination's label stable so the bar stays predictable.",
    );
    gap(Dp(16.0));
    do_dont(
        "Keep destinations equal and stable",
        "Don’t hide or reorder destinations between screens",
    );
}

#[tessera]
fn guide_anatomy() {
    prose("A navigation bar contains an active indicator, icons and labels for each destination.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Anatomy",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Container");
        legend(2, "Active indicator");
        legend(3, "Icon");
        legend(4, "Active label");
        legend(5, "Inactive icon");
        legend(6, "Inactive label");
    });
    gap(Dp(16.0));
    caption("The indicator highlights the active destination while other destinations stay inactive");
}

#[tessera]
fn guide_color() {
    prose(
        "Use the surface container color for the bar so it separates from the page, and rely on the secondary container indicator plus the filled icon to communicate selection.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Primary",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("The active destination uses the secondary container indicator and on secondary container icon");
}

#[tessera]
fn adaptive_design() {
    prose(
        "On wider windows the icon can move beside the label. The native component switches automatically at 600dp, and the icon position can also be set explicitly.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Adaptive",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("Above 600dp the icon moves before the label and destinations center in the bar");
    gap(Dp(24.0));
    prose(
        "For compact widths keep the icon above the label and let the bar span the window. Avoid increasing the destination count when the bar becomes wider.",
    );
}

#[tessera]
fn behavior() {
    prose(
        "Selecting a destination animates the active indicator to that destination, swaps its icon for the filled variant and updates the label color. The new view replaces the current top-level view.",
    );
    gap(Dp(16.0));
    prose(
        "Keep the bar's selected destination in sync with the visible view. When a destination opens its own stack, restore the previous selection when people return.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Behavior",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption("Tap a destination to move the indicator and swap the icon to its filled variant");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to and activate a destination".into(),
        "Identify the currently selected destination".into(),
        "Understand how many destinations the bar contains".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "Each destination keeps a minimum 48 × 48dp target across the full height of the bar. The active indicator is decorative and does not change the target size.",
    );
    gap(Dp(16.0));
    caption("Every destination keeps a 48dp target even when the label is hidden");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "Targets",
            4,
            NavigationBarLabelBehavior::SelectedOnly,
            0,
        );
    }));
}

#[tessera]
fn initial_focus() {
    prose(
        "When the bar receives focus, focus lands on the selected destination, or on the first destination when none is selected.",
    );
    gap(Dp(16.0));
    prose("Moving focus between destinations does not change the selected view until a destination is activated.");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Move focus between destinations".into()],
            vec![
                "Shift + Tab".into(),
                "Move focus to the previous destination".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Activate the focused destination".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Traversal wraps inside the bar and stays horizontal, so Tab leaves the bar from either end rather than skipping destinations.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "Give every destination a short, visible label. The accessible name matches the label, and the selected destination is exposed as selected together with its position in the set.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Label".into(), "Role".into(), "State".into()],
        vec![
            vec![
                "Destination".into(),
                "Match the visible label".into(),
                "Tab".into(),
                "Selected or not selected".into(),
            ],
            vec![
                "Position".into(),
                "\u{201c}Home, tab, 1 of 4\u{201d}".into(),
                "Tab".into(),
                "Selected".into(),
            ],
            vec![
                "Badge".into(),
                "Append the unread count".into(),
                "Tab".into(),
                "3 unread".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption("Announce each destination with its label, role and position in the bar");
}
