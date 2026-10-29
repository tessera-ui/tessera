//! Floating action button menu specifications and related actions.
//!
//! ## Usage
//!
//! Explore creation actions, color sets, states and keyboard access.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    floating_action_button_menu::{
        FloatingActionButtonMenuAlignment, FloatingActionButtonMenuColor, FloatingActionButtonMenuItem,
        FloatingActionButtonMenuState, floating_action_button_menu,
    },
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

const ITEMS: [(&str, &str); 3] = [
    ("New document", "Document"),
    ("New folder", "Folder"),
    ("Upload file", "File"),
];

#[tessera]
pub(super) fn fab_menu_spec() {
    article(
        "FAB menu",
        "The FAB menu expands from a floating action button to show related actions",
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
            intro: Some(RenderSlot::new(overview_intro)),
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
                    title: "Adaptive layout",
                    content: RenderSlot::new(adaptive_layout),
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
fn menu_demo(
    #[prop(into)] label: String,
    color: FloatingActionButtonMenuColor,
    mirrored: bool,
    expanded: bool,
) {
    let state = remember(move || {
        let mut state = FloatingActionButtonMenuState::new();
        if expanded {
            state.open();
        }
        state
    });
    let status = remember(|| "No action selected".to_owned());
    let theme = use_context::<MaterialTheme>().expect("MaterialTheme").get();
    let caption_text = label.clone();
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{label} FAB menu example")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            floating_action_button_menu()
                .state(state)
                .color(color)
                .alignment(if mirrored {
                    FloatingActionButtonMenuAlignment::Start
                } else {
                    FloatingActionButtonMenuAlignment::End
                })
                .open_label(format!("Open {label} menu"))
                .close_label(format!("Close {label} menu"))
                .modifier(Modifier::new().fill_max_width().height(Dp(300.0)))
                .items(vec![
                    FloatingActionButtonMenuItem::new(
                        ITEMS[0].0,
                        Some(filled::EDIT_SVG.into()),
                        Callback::new(move || status.set(ITEMS[0].1.into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        ITEMS[1].0,
                        Some(filled::FOLDER_SVG.into()),
                        Callback::new(move || status.set(ITEMS[1].1.into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        ITEMS[2].0,
                        Some(filled::UPLOAD_SVG.into()),
                        Callback::new(move || status.set(ITEMS[2].1.into())),
                    ),
                ])
                .main_content(move || {
                    surface()
                        .style(theme.color_scheme.surface_container_low.into())
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding(Padding::new(
                                    Dp(if mirrored { 220.0 } else { 24.0 }),
                                    Dp(24.0),
                                    Dp(24.0),
                                    Dp(24.0),
                                )))
                                .children(move || {
                                    text()
                                        .content("Create")
                                        .style(theme.typography.title_large);
                                    gap(Dp(8.0));
                                    text()
                                        .content(status.get())
                                        .style(type_style(16.0, 24.0));
                                });
                        });
                });
            gap(Dp(12.0));
            caption(format!("{caption_text} · last action: {}", status.get()));
        });
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "Opens from a FAB to show 2–6 related actions floating on screen".into(),
        "One FAB menu size for all sizes of FABs".into(),
        "Not used with extended FABs".into(),
        "Available in primary, secondary, and tertiary color sets".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Overview",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("The FAB menu comes in three color sets: primary, secondary, tertiary");
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
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Unavailable".into(),
            ],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native floating action button menu with its close button, labelled items and three color sets.",
    );
}

#[tessera]
fn expressive_update() {
    subheading("May 2025".into());
    prose(
        "The FAB menu adds more options to the FAB. It should replace the speed dial and any usage of stacked small FABs.",
    );
    gap(Dp(24.0));
    prose("New component added to catalog:");
    gap(Dp(8.0));
    bullets(vec![
        "One menu size that pairs with any FAB".into(),
        "Replaces any usage of stacked small FABs".into(),
    ]);
    gap(Dp(24.0));
    prose("Color:");
    gap(Dp(8.0));
    bullets(vec![
        "Contrasting close button and item colors".into(),
        "Supports dynamic color".into(),
        "Compatible with any FAB color style".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Expressive",
            FloatingActionButtonMenuColor::Tertiary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption(
        "The FAB menu uses contrasting color and large items to focus attention. It can open from any size FAB",
    );
}

#[tessera]
fn differences() {
    prose(
        "M2 speed dials stacked several small FABs and hid their labels. The M3 FAB menu uses one size for every FAB and labels each action.",
    );
    gap(Dp(24.0));
    bullets(vec![
        "M2: speed dial with stacked small icon-only FABs".into(),
        "M3: a single FAB menu whose labelled items expand from one FAB".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Differences",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("The M3 FAB menu replaces stacked small FABs with labelled, expressive items");
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        menu_demo(
            "Variant",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("There’s one variant of FAB menu");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![vec!["FAB menu".into(), "--".into(), "Available".into()]],
    );
}

#[tessera]
fn configurations() {
    subheading("Color set".into());
    prose("FAB menus come in three color sets. Each set colors the close button and the items together.");
    gap(Dp(24.0));
    for (number, color) in [
        FloatingActionButtonMenuColor::Primary,
        FloatingActionButtonMenuColor::Secondary,
        FloatingActionButtonMenuColor::Tertiary,
    ]
    .into_iter()
    .enumerate()
    {
        legend(number + 1, format!("{:?} set", color));
        gap(Dp(16.0));
        panel(RenderSlot::new(move || {
            menu_demo(format!("{:?}", color), color, false, true);
        }));
        gap(Dp(24.0));
    }
    spec_table(
        vec![
            "Category".into(),
            "Configuration".into(),
            "M3".into(),
            "M3 Expressive".into(),
        ],
        vec![vec![
            "Color".into(),
            "Primary set, secondary set, tertiary set".into(),
            "--".into(),
            "Available".into(),
        ]],
    );
    gap(Dp(32.0));
    subheading("Alignment".into());
    prose(
        "The FAB menu is aligned to the trailing edge of the window. In right-to-left languages it is aligned to the left edge and the layout of elements is mirrored.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Trailing",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Leading",
            FloatingActionButtonMenuColor::Primary,
            true,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Trailing alignment by default; leading alignment mirrors the layout for RTL");
}

#[tessera]
fn tokens() {
    prose(
        "The FAB menu has a common token set and six color sets, three for each element (close button and menu item). Values below come from the native implementation.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Close button container".into(),
                "md.comp.fab-menu.close-button.container.size".into(),
                "56 × 56dp · full round".into(),
            ],
            vec![
                "Close button color".into(),
                "md.comp.fab-menu.close-button.container.color".into(),
                "Primary · secondary · tertiary".into(),
            ],
            vec![
                "Close button icon".into(),
                "md.comp.fab-menu.close-button.icon.color".into(),
                "On primary · on secondary · on tertiary".into(),
            ],
            vec![
                "Item container".into(),
                "md.comp.fab-menu.menu-item.container.color".into(),
                "Primary container · secondary container · tertiary container".into(),
            ],
            vec![
                "Item label".into(),
                "md.comp.fab-menu.menu-item.label.text.color".into(),
                "On primary container · on secondary container · on tertiary container".into(),
            ],
            vec![
                "Item shape".into(),
                "md.comp.fab-menu.menu-item.container.shape".into(),
                "Full capsule".into(),
            ],
            vec![
                "Item height".into(),
                "md.comp.fab-menu.menu-item.container.height".into(),
                "56dp".into(),
            ],
            vec![
                "Item spacing".into(),
                "md.comp.fab-menu.menu-item.container.spacing".into(),
                "4dp".into(),
            ],
            vec![
                "Menu margins".into(),
                "md.comp.fab-menu.menu.margin".into(),
                "16dp".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        menu_demo(
            "Anatomy",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Close button");
        legend(2, "Menu item");
    });
    gap(Dp(16.0));
    caption("The close button replaces the FAB; each menu item pairs an icon with a label");
    gap(Dp(32.0));
    panel(RenderSlot::new(six_items));
    gap(Dp(16.0));
    caption("The FAB menu can have up to six items");
}

#[tessera]
fn six_items() {
    let state = remember(|| {
        let mut state = FloatingActionButtonMenuState::new();
        state.open();
        state
    });
    let theme = use_context::<MaterialTheme>().expect("MaterialTheme").get();
    let status = remember(|| "Six actions".to_owned());
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            floating_action_button_menu()
                .state(state)
                .color(FloatingActionButtonMenuColor::Primary)
                .open_label("Open six action menu")
                .close_label("Close six action menu")
                .modifier(Modifier::new().fill_max_width().height(Dp(480.0)))
                .items(vec![
                    FloatingActionButtonMenuItem::new(
                        "First",
                        Some(filled::EDIT_SVG.into()),
                        Callback::new(move || status.set("First".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Second",
                        Some(filled::FOLDER_SVG.into()),
                        Callback::new(move || status.set("Second".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Third",
                        Some(filled::UPLOAD_SVG.into()),
                        Callback::new(move || status.set("Third".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Fourth",
                        Some(filled::IMAGE_SVG.into()),
                        Callback::new(move || status.set("Fourth".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Fifth",
                        Some(filled::EVENT_SVG.into()),
                        Callback::new(move || status.set("Fifth".into())),
                    ),
                    FloatingActionButtonMenuItem::new(
                        "Sixth",
                        Some(filled::CHECK_SVG.into()),
                        Callback::new(move || status.set("Sixth".into())),
                    ),
                ])
                .main_content(move || {
                    surface()
                        .style(theme.color_scheme.surface_container_low.into())
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(24.0)))
                                .children(move || {
                                    text()
                                        .content("Up to six actions")
                                        .style(theme.typography.title_large);
                                    gap(Dp(8.0));
                                    text().content(status.get());
                                });
                        });
                });
        });
}

#[tessera]
fn colors() {
    prose(
        "FAB menu colors come from the active color set: primary, secondary, or tertiary. The close button uses the set's fill color while items use its container color.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Primary",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            panel(RenderSlot::new(|| {
                menu_demo(
                    "Dark",
                    FloatingActionButtonMenuColor::Primary,
                    false,
                    true,
                );
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Set".into(), "Element".into(), "Color role".into()],
        vec![
            vec![
                "Primary".into(),
                "Item container".into(),
                "Primary container".into(),
            ],
            vec![
                "Primary".into(),
                "Item label and icon".into(),
                "On primary container".into(),
            ],
            vec![
                "Primary".into(),
                "Close button".into(),
                "Primary".into(),
            ],
            vec![
                "Primary".into(),
                "Close icon".into(),
                "On primary".into(),
            ],
            vec![
                "Secondary".into(),
                "Item container".into(),
                "Secondary container".into(),
            ],
            vec![
                "Secondary".into(),
                "Item label and icon".into(),
                "On secondary container".into(),
            ],
            vec![
                "Secondary".into(),
                "Close button".into(),
                "Secondary".into(),
            ],
            vec![
                "Secondary".into(),
                "Close icon".into(),
                "On secondary".into(),
            ],
            vec![
                "Tertiary".into(),
                "Item container".into(),
                "Tertiary container".into(),
            ],
            vec![
                "Tertiary".into(),
                "Item label and icon".into(),
                "On tertiary container".into(),
            ],
            vec![
                "Tertiary".into(),
                "Close button".into(),
                "Tertiary".into(),
            ],
            vec![
                "Tertiary".into(),
                "Close icon".into(),
                "On tertiary".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "States are visual representations used to communicate the status of a component or interactive element.",
    );
    gap(Dp(32.0));
    subheading("Close button".into());
    prose("Enabled, hovered, focused and pressed states share the same close button colors. Hover, focus with Tab, or press and hold in the live sample below.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Close states",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Close button states: enabled, hovered, focused, pressed");
    gap(Dp(32.0));
    subheading("Menu item".into());
    prose("Menu items keep their own hover, focus and pressed state layers while the menu stays open.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Item states",
            FloatingActionButtonMenuColor::Secondary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Menu item states: enabled, hovered, focused, pressed");
}

#[tessera]
fn measurements() {
    prose(
        "FAB menu items share the same measurements as the medium button specs. The close button should always be 56dp.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Attribute".into(), "Value".into()],
        vec![
            vec![
                "Menu item".into(),
                "Container height".into(),
                "56dp".into(),
            ],
            vec![
                "Menu item".into(),
                "Horizontal padding".into(),
                "24dp".into(),
            ],
            vec![
                "Menu item".into(),
                "Icon size".into(),
                "24dp".into(),
            ],
            vec![
                "Menu item".into(),
                "Icon to label gap".into(),
                "8dp".into(),
            ],
            vec![
                "Menu items".into(),
                "Spacing between items".into(),
                "4dp".into(),
            ],
            vec![
                "Close button".into(),
                "Container size".into(),
                "56 × 56dp".into(),
            ],
            vec![
                "Menu".into(),
                "Window margin".into(),
                "16dp".into(),
            ],
            vec![
                "Menu items".into(),
                "Minimum target size".into(),
                "48dp".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Measurements",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("FAB menu size measurements");
    gap(Dp(24.0));
    bullets(vec![
        "The FAB menu animates from the top trailing edge of the FAB to ensure a smooth animation".into(),
        "The FAB should always have a 16dp margin from the window edge".into(),
        "The close button and FAB share the same trailing edge".into(),
    ]);
}

#[tessera]
fn usage() {
    prose(
        "A FAB menu opens from a FAB to show multiple related actions. It should always appear in the same place as the FAB that opened it.",
    );
    gap(Dp(16.0));
    prose(
        "This makes actions immediately accessible, and keeps the UI clean by concealing actions when they’re not needed.",
    );
    gap(Dp(24.0));
    do_dont(
        "Open the FAB menu from a FAB",
        "Don’t open a FAB menu from an extended FAB or any other extended component",
    );
    gap(Dp(32.0));
    prose(
        "The FAB menu should be aligned to the trailing edge of the window. In right-to-left (RTL) languages, this means the FAB and FAB menu should be aligned to the left edge, and the layout of elements should be mirrored.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Mirrored",
            FloatingActionButtonMenuColor::Primary,
            true,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("In RTL languages, the FAB menu is left-aligned and the icon and text placement is mirrored");
    gap(Dp(32.0));
    prose(
        "FAB menus can contain 2–6 items. These should be closely related under a single action, like Share.",
    );
    gap(Dp(16.0));
    prose("Avoid grouping unrelated actions in the same FAB menu.");
    gap(Dp(24.0));
    do_dont(
        "FAB menus can have 2–6 items",
        "Don’t use a FAB menu with one item",
    );
    gap(Dp(32.0));
    prose(
        "When a FAB is paired with other components, like the floating toolbar or navigation rail, don’t use the FAB menu. This prevents cognitive overload and interface clutter.",
    );
    gap(Dp(24.0));
    do_dont(
        "FABs can be placed next to toolbars and other components",
        "Don’t use a FAB menu with a toolbar or navigation rail",
    );
}

#[tessera]
fn guide_anatomy() {
    prose("The FAB menu has two elements: the close button and the menu items.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Anatomy",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Close button");
        legend(2, "Menu item");
    });
    gap(Dp(16.0));
    caption("The close button takes the place of the FAB and each item pairs an icon with a label");
}

#[tessera]
fn guide_color() {
    prose(
        "FAB menus have three color sets: primary, secondary, and tertiary. Use the color set that best matches the FAB color style.",
    );
    gap(Dp(16.0));
    prose(
        "Use the primary FAB menu color set with the primary or primary container FAB color styles.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Primary pair",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("A primary FAB is paired with a primary FAB menu");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Tertiary pair",
            FloatingActionButtonMenuColor::Tertiary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("A tertiary FAB is paired with a tertiary FAB menu");
}

#[tessera]
fn adaptive_layout() {
    prose(
        "The FAB menu can scroll when there are more actions than fit on screen. Make sure the items scroll behind the close button and keep the close button easy to access and unobstructed.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Adaptive",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(24.0));
    do_dont(
        "Allow the menu items to scroll behind the close button",
        "Don’t obstruct the close button in short screens like horizontal orientation",
    );
    gap(Dp(32.0));
    prose(
        "Each item keeps a minimum 48dp target, so density changes must preserve the target size of both the close button and the items.",
    );
}

#[tessera]
fn behavior() {
    prose(
        "The FAB menu animates from the top trailing edge of the FAB. Items expand upward in order, and the close button replaces the FAB in place.",
    );
    gap(Dp(16.0));
    prose(
        "Selecting an item closes the menu and runs that action. Pressing Escape, or activating the close button, dismisses the menu and returns focus to the FAB; tapping outside the menu also closes it.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Behavior",
            FloatingActionButtonMenuColor::Primary,
            false,
            false,
        );
    }));
    gap(Dp(16.0));
    caption("Open the menu with the FAB, choose an action, then reopen it to confirm the reported action");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate and interact with the FAB menu".into(),
        "Ensure focus is correct when navigating through the menu".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Use cases",
            FloatingActionButtonMenuColor::Primary,
            false,
            false,
        );
    }));
}

#[tessera]
fn interaction_style() {
    prose("FAB menu elements meet the minimum target size of 48dp.");
    gap(Dp(16.0));
    caption("FAB menus have 48x48dp minimum width and sufficient spacing by default");
    gap(Dp(24.0));
    prose(
        "When the FAB menu can scroll, make sure the items scroll behind the close button. The close button should always be easy to access and unobstructed.",
    );
    gap(Dp(24.0));
    do_dont(
        "Allow the menu items to scroll behind the close button",
        "Don’t obstruct the close button in short screens like horizontal orientation",
    );
}

#[tessera]
fn initial_focus() {
    prose(
        "When the FAB is selected, the FAB menu opens, and initial focus remains on the close button, which takes the place of the original FAB.",
    );
    gap(Dp(16.0));
    prose("Then the focus moves from the top menu item to the bottom.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Initial focus",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Focus lands on the close button. People can then navigate through all the items");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate to the next interactive element".into()],
            vec!["Space or Enter".into(), "Activate the focused button or item".into()],
            vec!["Escape".into(), "Close the menu and return focus to the FAB".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The native sample follows this table: Tab reaches the close button and each item, Space or Enter activates the focused element, and Escape dismisses the menu.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Keyboard",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
}

#[tessera]
fn labeling() {
    prose("On Android, a FAB menu’s close button should include a state to tell screen readers what action will occur when it’s toggled. The close button should be labeled:");
    gap(Dp(16.0));
    spec_table(
        vec!["Element".into(), "Label".into(), "Role".into(), "State".into()],
        vec![
            vec![
                "Close button".into(),
                "Toggle menu".into(),
                "Button".into(),
                "Expanded or collapsed".into(),
            ],
            vec![
                "Menu item".into(),
                "Match the item’s UI text".into(),
                "Menu item".into(),
                "—".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption("On Android, the close button accessibility labels should include a toggle menu label, button role, and an expanded or collapsed state");
    gap(Dp(24.0));
    prose(
        "FAB menu items should be labeled with the same text people see on screen, so the announced action matches the visible label.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        menu_demo(
            "Labeling",
            FloatingActionButtonMenuColor::Primary,
            false,
            true,
        );
    }));
}
