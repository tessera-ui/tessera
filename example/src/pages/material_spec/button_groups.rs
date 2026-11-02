//! Interactive button group specifications.
//!
//! ## Usage
//!
//! Compare standard and connected groups, selection policies and measurements.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonShape, ButtonVariant, button},
    button_groups::{
        ButtonGroupsSelectionMode, ButtonGroupsSize, ButtonGroupsState, ButtonGroupsStyle,
        button_groups,
    },
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::{Asset, filled},
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{CallbackWith, Color, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const CAUTION_COLOR: Color = Color::from_rgb(0.705, 0.365, 0.0);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

const SIZES: [ButtonGroupsSize; 5] = [
    ButtonGroupsSize::ExtraSmall,
    ButtonGroupsSize::Small,
    ButtonGroupsSize::Medium,
    ButtonGroupsSize::Large,
    ButtonGroupsSize::ExtraLarge,
];

const STATES: [&str; 5] = ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"];

fn size_name(size: ButtonGroupsSize) -> &'static str {
    match size {
        ButtonGroupsSize::ExtraSmall => "Extra small",
        ButtonGroupsSize::Small => "Small",
        ButtonGroupsSize::Medium => "Medium",
        ButtonGroupsSize::Large => "Large",
        ButtonGroupsSize::ExtraLarge => "Extra large",
    }
}

fn token_size_name(size: ButtonGroupsSize) -> &'static str {
    match size {
        ButtonGroupsSize::ExtraSmall => "xsmall",
        ButtonGroupsSize::Small => "small",
        ButtonGroupsSize::Medium => "medium",
        ButtonGroupsSize::Large => "large",
        ButtonGroupsSize::ExtraLarge => "xlarge",
    }
}

fn style_name(style: ButtonGroupsStyle) -> &'static str {
    match style {
        ButtonGroupsStyle::Standard => "Standard",
        ButtonGroupsStyle::Connected => "Connected",
    }
}

fn next_size(size: ButtonGroupsSize) -> ButtonGroupsSize {
    match size {
        ButtonGroupsSize::ExtraSmall => ButtonGroupsSize::Small,
        ButtonGroupsSize::Small => ButtonGroupsSize::Medium,
        ButtonGroupsSize::Medium => ButtonGroupsSize::Large,
        ButtonGroupsSize::Large => ButtonGroupsSize::ExtraLarge,
        ButtonGroupsSize::ExtraLarge => ButtonGroupsSize::ExtraSmall,
    }
}

fn state_presentation(state: usize) -> InteractionVisualState {
    match state {
        2 => InteractionVisualState::Hover,
        3 => InteractionVisualState::Focus,
        4 => InteractionVisualState::Pressed,
        _ => InteractionVisualState::Rest,
    }
}

/// One button inside a demonstration group.
#[derive(Clone, PartialEq)]
struct Item {
    icon: Option<Asset>,
    label: Option<&'static str>,
    description: &'static str,
}

fn labelled(label: &'static str) -> Item {
    Item {
        icon: None,
        label: Some(label),
        description: label,
    }
}

fn icon_label(icon: Asset, label: &'static str) -> Item {
    Item {
        icon: Some(icon),
        label: Some(label),
        description: label,
    }
}

fn icon_only(icon: Asset, description: &'static str) -> Item {
    Item {
        icon: Some(icon),
        label: None,
        description,
    }
}

fn mixed_items() -> Vec<Item> {
    vec![
        icon_only(filled::EDIT_SVG, "Edit"),
        icon_label(filled::EDIT_SVG, "Label"),
        icon_only(filled::EDIT_SVG, "Draw"),
    ]
}

fn label_items() -> Vec<Item> {
    vec![
        icon_label(filled::STAR_SVG, "Label"),
        icon_label(filled::STAR_SVG, "Label 2"),
        icon_label(filled::STAR_SVG, "Label 3"),
    ]
}

#[tessera]
pub(super) fn button_groups_spec() {
    article(
        "Button groups",
        "Button groups organize buttons and add interactions between them",
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
                title: "Selection & activation",
                content: RenderSlot::new(selection_and_activation),
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
                title: "Density",
                content: RenderSlot::new(density),
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
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive),
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
fn legend(items: Vec<String>) {
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
                        text().content(item.clone()).style(type_style(16.0, 24.0));
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
                .style(type_style(14.0, 20.0));
        });
}

/// Renders one native button group with explicit items and configuration.
#[tessera]
fn group(
    items: Vec<Item>,
    style: ButtonGroupsStyle,
    size: Option<ButtonGroupsSize>,
    shape: Option<ButtonShape>,
    variant: Option<ButtonVariant>,
    mode: Option<ButtonGroupsSelectionMode>,
    required: Option<bool>,
    enabled: Option<bool>,
    selected: Option<Vec<usize>>,
    presentation: Option<Vec<InteractionVisualState>>,
    #[prop(into)] label: Option<String>,
) {
    let initial = selected.unwrap_or_default();
    let state = remember(move || {
        let mut state = ButtonGroupsState::new();
        for index in initial {
            state.set_selected(index, true);
        }
        state
    });
    let builder = button_groups()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            label: label.or_else(|| Some("Button group".into())),
            ..Default::default()
        }))
        .style(style)
        .size(size.unwrap_or(ButtonGroupsSize::Small))
        .shape_style(shape.unwrap_or(ButtonShape::Round))
        .variant(variant.unwrap_or(ButtonVariant::Filled))
        .selection_mode(mode.unwrap_or(ButtonGroupsSelectionMode::Multiple))
        .selection_required(required.unwrap_or(false))
        .enabled(enabled.unwrap_or(true))
        .presentation_states(presentation.unwrap_or_default())
        .state(state);
    for item in items {
        builder.child_shared(
            RenderSlot::new(move || {
                let item = item.clone();
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        label: Some(item.description.into()),
                        ..Default::default()
                    }))
                    .children(move || {
                        if let Some(asset) = item.icon {
                            icon().painter(asset);
                        }
                        if item.icon.is_some() && item.label.is_some() {
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                        }
                        if let Some(label) = item.label {
                            text().content(label);
                        }
                    });
            }),
            CallbackWith::new(|_| {}),
        );
    }
}

#[tessera]
fn overview() {
    bullets(vec![
        "Two variants: standard and connected".into(),
        "Applies shape morph when pressed and selected".into(),
        "Connected button groups replace the segmented button".into(),
        "Works with all button sizes: XS, S, M, L, and XL".into(),
        "Support for single-select, multi-select, and selection-required".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(
                    vec![
                        icon_only(filled::BLUETOOTH_SVG, "Bluetooth"),
                        icon_only(filled::ALARM_SVG, "Alarm"),
                        icon_label(filled::MOOD_SVG, "Focus"),
                        icon_only(filled::FLASHLIGHT_ON_SVG, "Flashlight"),
                        icon_only(filled::WIFI_SVG, "Wi-Fi"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .size(ButtonGroupsSize::Medium)
                .selected(vec![4])
                .label("Quick settings");
                gap(Dp(24.0));
                group(
                    vec![
                        labelled("8 oz"),
                        labelled("12 oz"),
                        labelled("16 oz"),
                        labelled("20 oz"),
                    ],
                    ButtonGroupsStyle::Connected,
                )
                .mode(ButtonGroupsSelectionMode::Single)
                .required(true)
                .selected(vec![0])
                .label("Cup size");
            });
    }));
    caption("Button groups can contain buttons and icon buttons");
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
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Available".into(),
            ],
            vec!["".into(), "Web: Expressive".into(), "Unavailable".into()],
        ],
    );
}

#[tessera]
fn expressive() {
    prose(
        "Button groups apply shape, motion, and width changes to buttons and icon buttons to make them more interactive.",
    );
    gap(Dp(32.0));
    subheading("May 2025".into());
    gap(Dp(8.0));
    prose("New component added to catalog.");
    gap(Dp(16.0));
    prose("Variants and naming:");
    bullets(vec![
        "Added standard button group".into(),
        "Added connected button group".into(),
        "Use instead of segmented button, which is no longer recommended".into(),
    ]);
    gap(Dp(16.0));
    prose("Configurations:");
    bullets(vec![
        "Works with all button sizes: XS, S, M, L, and XL".into(),
        "Applies default shape to all buttons: round or square".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in [
                    ButtonGroupsSize::ExtraSmall,
                    ButtonGroupsSize::Small,
                    ButtonGroupsSize::Medium,
                ] {
                    group(mixed_items(), ButtonGroupsStyle::Standard)
                        .size(size)
                        .selected(vec![1])
                        .label(format!("{} standard group", size_name(size)));
                    gap(Dp(12.0));
                }
                gap(Dp(12.0));
                group(label_items(), ButtonGroupsStyle::Connected)
                    .selected(vec![0])
                    .label("Connected group");
            });
    }));
    caption("Button groups are containers that hold buttons of many shapes and sizes");
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(|| {
                        group(
                            vec![
                                icon_only(filled::BLUETOOTH_SVG, "Bluetooth"),
                                icon_only(filled::ALARM_SVG, "Alarm"),
                                icon_only(filled::LINK_SVG, "Link"),
                                icon_only(filled::WIFI_SVG, "Wi-Fi"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![2])
                        .label("Standard icon buttons");
                        gap(Dp(12.0));
                        group(
                            vec![labelled("Label"), labelled("Label"), labelled("Label")],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![1])
                        .label("Standard label buttons");
                        gap(Dp(12.0));
                        group(mixed_items(), ButtonGroupsStyle::Standard)
                            .size(ButtonGroupsSize::ExtraSmall)
                            .label("Standard mixed buttons");
                        gap(Dp(16.0));
                        badge(1);
                    });
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(|| {
                        group(
                            vec![
                                icon_only(filled::STAR_SVG, "Favorites"),
                                icon_only(filled::HOME_SVG, "Home"),
                                icon_only(filled::SEARCH_SVG, "Search"),
                                icon_only(filled::SETTINGS_SVG, "Settings"),
                            ],
                            ButtonGroupsStyle::Connected,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .mode(ButtonGroupsSelectionMode::Single)
                        .selected(vec![1])
                        .label("Connected icon buttons");
                        gap(Dp(12.0));
                        group(label_items(), ButtonGroupsStyle::Connected)
                            .size(ButtonGroupsSize::ExtraSmall)
                            .mode(ButtonGroupsSelectionMode::Single)
                            .selected(vec![0])
                            .label("Connected label buttons");
                        gap(Dp(16.0));
                        badge(2);
                    });
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "Standard button group".into(),
        "Connected button group".into(),
    ]);
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec![
                "Standard button group".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Connected button group".into(),
                "Available as segmented button".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for (index, size) in SIZES.into_iter().enumerate() {
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            group(
                                vec![
                                    icon_only(filled::STAR_SVG, "Favorites"),
                                    icon_only(filled::ALARM_SVG, "Alarm"),
                                    icon_only(filled::LINK_SVG, "Link"),
                                ],
                                ButtonGroupsStyle::Standard,
                            )
                            .size(size)
                            .selected(vec![1])
                            .label(format!("{} group", size_name(size)));
                        }),
                    );
                    gap(Dp(16.0));
                }
                marked(
                    6,
                    RenderSlot::new(|| {
                        column().children(|| {
                            group(label_items(), ButtonGroupsStyle::Connected)
                                .mode(ButtonGroupsSelectionMode::Single)
                                .selected(vec![0])
                                .label("Single-select group");
                            gap(Dp(8.0));
                            group(label_items(), ButtonGroupsStyle::Connected)
                                .selected(vec![0, 2])
                                .label("Multi-select group");
                        });
                    }),
                );
                gap(Dp(16.0));
                marked(
                    7,
                    RenderSlot::new(|| {
                        column().children(|| {
                            group(mixed_items(), ButtonGroupsStyle::Standard)
                                .shape(ButtonShape::Round)
                                .label("Round group");
                            gap(Dp(8.0));
                            group(mixed_items(), ButtonGroupsStyle::Standard)
                                .shape(ButtonShape::Square)
                                .label("Square group");
                        });
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    prose("Configurations for both variants of button groups:");
    gap(Dp(8.0));
    legend(vec![
        "Extra small".into(),
        "Small".into(),
        "Medium".into(),
        "Large".into(),
        "Extra large".into(),
        "Single-select and multi-select".into(),
        "Round and square".into(),
    ]);
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
                "XS, S, M, L, XL".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Default shape".into(),
                "Round, square".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Selection".into(),
                "Single-select, multi-select, selection-required".into(),
                "Available as segmented button".into(),
                "Available".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("Live configuration".into());
    gap(Dp(8.0));
    prose(
        "Change the configuration, then click a choice. Focus enters the group's first button and the horizontal traversal policy wraps at the edges.",
    );
    gap(Dp(24.0));
    playground();
}

#[tessera]
fn playground() {
    let style = remember(|| ButtonGroupsStyle::Standard);
    let size = remember(|| ButtonGroupsSize::Medium);
    let square = remember(|| false);
    let multiple = remember(|| false);
    let required = remember(|| false);
    let enabled = remember(|| true);
    let icons = remember(|| true);
    let state = remember(ButtonGroupsState::new);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .selected(style.get() == ButtonGroupsStyle::Connected)
                            .accessibility_label("Connected layout")
                            .on_click(move || {
                                style.set(if style.get() == ButtonGroupsStyle::Connected {
                                    ButtonGroupsStyle::Standard
                                } else {
                                    ButtonGroupsStyle::Connected
                                });
                            })
                            .child(|| {
                                text().content("Connected");
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
                            .selected(square.get())
                            .accessibility_label("Square shape")
                            .on_click(move || square.set(!square.get()))
                            .child(|| {
                                text().content("Square");
                            });
                        button()
                            .tonal()
                            .selected(multiple.get())
                            .accessibility_label("Multiple selection")
                            .on_click(move || multiple.set(!multiple.get()))
                            .child(|| {
                                text().content("Multiple");
                            });
                        button()
                            .tonal()
                            .selected(required.get())
                            .accessibility_label("Selection required")
                            .on_click(move || required.set(!required.get()))
                            .child(|| {
                                text().content("Required");
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label("Disabled group")
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(|| {
                                text().content("Disabled");
                            });
                        button()
                            .tonal()
                            .selected(icons.get())
                            .accessibility_label("Item icons")
                            .on_click(move || icons.set(!icons.get()))
                            .child(|| {
                                text().content("Icons");
                            });
                    });
                gap(Dp(32.0));
                button_groups()
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        label: Some("Configured group".into()),
                        ..Default::default()
                    }))
                    .style(style.get())
                    .size(size.get())
                    .shape_style(if square.get() {
                        ButtonShape::Square
                    } else {
                        ButtonShape::Round
                    })
                    .selection_mode(if multiple.get() {
                        ButtonGroupsSelectionMode::Multiple
                    } else {
                        ButtonGroupsSelectionMode::Single
                    })
                    .selection_required(required.get())
                    .enabled(enabled.get())
                    .state(state)
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text().content("One");
                                });
                            } else {
                                text().content("One");
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text().content("Two");
                                });
                            } else {
                                text().content("Two");
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text().content("Three");
                                });
                            } else {
                                text().content("Three");
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    );
                gap(Dp(24.0));
                text()
                    .content(format!(
                        "{} · {} · {} · selection: {}",
                        style_name(style.get()),
                        size_name(size.get()),
                        if multiple.get() { "multiple" } else { "single" },
                        (0..3)
                            .filter(|index| state.with(|state| state.is_selected(*index)))
                            .map(|index| (index + 1).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn tokens() {
    let connected = remember(|| false);
    let size = remember(|| ButtonGroupsSize::ExtraSmall);
    prose(
        "Standard and connected button group tokens are organized by size. Select the variant and size from the token set menu. Go to the button and icon button pages to view their tokens.",
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .selected(connected.get())
                .accessibility_label("Connected token set")
                .on_click(move || connected.set(!connected.get()))
                .child(|| {
                    text().content("Connected");
                });
            button()
                .text()
                .accessibility_label("Cycle token set size")
                .on_click(move || size.set(next_size(size.get())))
                .child(move || {
                    text().content(format!("Size: {}", size_name(size.get())));
                });
        });
    gap(Dp(16.0));
    let index = SIZES
        .iter()
        .position(|value| *value == size.get())
        .unwrap_or(0);
    let token = token_size_name(size.get());
    let height = ["32dp", "40dp", "56dp", "96dp", "136dp"][index];
    if connected.get() {
        subheading(format!("Button group connected - Size - {}", {
            let mut name = token.to_string();
            name[..1].make_ascii_uppercase();
            name
        }));
        gap(Dp(16.0));
        spec_table(
            vec!["Token".into(), "Value".into()],
            vec![
                vec![
                    format!("Button group connected {token} container height"),
                    height.into(),
                ],
                vec![
                    format!("Button group connected {token} space between buttons"),
                    "2dp".into(),
                ],
                vec![
                    format!("Button group connected {token} container shape"),
                    "Full".into(),
                ],
                vec![
                    format!("Button group connected {token} inner corner size"),
                    ["8dp", "8dp", "8dp", "16dp", "20dp"][index].into(),
                ],
                vec![
                    format!("Button group connected {token} pressed inner corner size"),
                    ["4dp", "4dp", "4dp", "12dp", "16dp"][index].into(),
                ],
                vec![
                    format!("Button group connected {token} selected inner corner size"),
                    "50%".into(),
                ],
            ],
        );
    } else {
        subheading(format!("Button group standard - Size - {}", {
            let mut name = token.to_string();
            name[..1].make_ascii_uppercase();
            name
        }));
        gap(Dp(16.0));
        spec_table(
            vec!["Token".into(), "Value".into()],
            vec![
                vec![
                    format!("Button group {token} container height"),
                    height.into(),
                ],
                vec![
                    format!("Button group {token} between space"),
                    ["18dp", "12dp", "8dp", "8dp", "8dp"][index].into(),
                ],
                vec![
                    format!("Button group {token} pressed motion spring dampening"),
                    "0.9".into(),
                ],
                vec![
                    format!("Button group {token} pressed motion spring stiffness"),
                    "1400".into(),
                ],
                vec![
                    format!("Button group {token} pressed width multiplier"),
                    "15%".into(),
                ],
            ],
        );
    }
}

#[tessera]
fn anatomy() {
    prose(
        "Button groups are invisible containers that add padding between buttons and modify button shape. They don't contain any buttons by default.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .border_with_shape(
                                        Dp(1.0),
                                        INK,
                                        Shape::rounded_rectangle(Dp(4.0)),
                                    )
                                    .padding_all(Dp(4.0)),
                            )
                            .children(|| {
                                group(
                                    vec![
                                        icon_only(filled::EDIT_SVG, "Edit"),
                                        icon_only(filled::EDIT_SVG, "Draw"),
                                        icon_label(filled::EDIT_SVG, "Label"),
                                        icon_only(filled::EDIT_SVG, "Sketch"),
                                        icon_only(filled::EDIT_SVG, "Write"),
                                    ],
                                    ButtonGroupsStyle::Standard,
                                )
                                .size(ButtonGroupsSize::ExtraSmall)
                                .variant(ButtonVariant::Tonal)
                                .label("Standard container");
                            });
                    }),
                );
                gap(Dp(24.0));
                marked(
                    1,
                    RenderSlot::new(|| {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .border_with_shape(
                                        Dp(1.0),
                                        INK,
                                        Shape::rounded_rectangle(Dp(4.0)),
                                    )
                                    .padding_all(Dp(4.0)),
                            )
                            .children(|| {
                                group(
                                    vec![
                                        icon_label(filled::STAR_SVG, "Label"),
                                        icon_label(filled::STAR_SVG, "Label 2"),
                                        icon_label(filled::STAR_SVG, "Label 3"),
                                        icon_label(filled::STAR_SVG, "Label 4"),
                                    ],
                                    ButtonGroupsStyle::Connected,
                                )
                                .size(ButtonGroupsSize::ExtraSmall)
                                .variant(ButtonVariant::Tonal)
                                .label("Connected container");
                            });
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    legend(vec!["Container".into()]);
    gap(Dp(48.0));
    subheading("Common layouts".into());
    gap(Dp(8.0));
    prose("Mix and match buttons and icon buttons for different scenarios.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        group(
                            vec![
                                icon_label(filled::STAR_SVG, "Start"),
                                icon_label(filled::DIRECTIONS_SVG, "Directions"),
                                icon_label(filled::SHARE_SVG, "Share"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .variant(ButtonVariant::Tonal)
                        .label("Label buttons");
                    }),
                );
                gap(Dp(16.0));
                marked(
                    2,
                    RenderSlot::new(|| {
                        group(
                            vec![
                                icon_only(filled::SKIP_PREVIOUS_SVG, "Previous"),
                                icon_label(filled::PLAY_ARROW_SVG, "Play"),
                                icon_only(filled::SKIP_NEXT_SVG, "Next"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .selected(vec![1])
                        .label("Label buttons and icon buttons");
                    }),
                );
                gap(Dp(16.0));
                marked(
                    3,
                    RenderSlot::new(|| {
                        group(
                            vec![
                                icon_only(filled::FORMAT_BOLD_SVG, "Bold"),
                                icon_only(filled::FORMAT_ITALIC_SVG, "Italic"),
                                icon_only(filled::FORMAT_UNDERLINED_SVG, "Underline"),
                                icon_only(filled::FORMAT_COLOR_TEXT_SVG, "Text color"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![0])
                        .label("Extra small icon buttons");
                    }),
                );
                gap(Dp(16.0));
                marked(
                    4,
                    RenderSlot::new(|| {
                        group(
                            vec![
                                icon_only(filled::ALARM_SVG, "Alarm"),
                                icon_only(filled::VOLUME_OFF_SVG, "Mute"),
                                icon_only(filled::WIFI_SVG, "Wi-Fi"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::Large)
                        .variant(ButtonVariant::Tonal)
                        .label("Large icon buttons");
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "Label buttons".into(),
        "Label buttons and icon buttons".into(),
        "Extra small icon buttons".into(),
        "Large icon buttons".into(),
    ]);
    gap(Dp(48.0));
    subheading("Color".into());
    gap(Dp(8.0));
    prose(
        "Button groups have no color properties. They can use the default button or toggle button color styles, like filled, tonal, and outlined. Avoid using standard icon buttons or text buttons, as they have no container treatment.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for (index, variant) in [
                    ButtonVariant::Filled,
                    ButtonVariant::Tonal,
                    ButtonVariant::Outlined,
                    ButtonVariant::Elevated,
                ]
                .into_iter()
                .enumerate()
                {
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    if variant == ButtonVariant::Elevated {
                                        text()
                                            .content("Elevated style not recommended")
                                            .color(INK)
                                            .style(type_style(14.0, 20.0));
                                    } else {
                                        group(mixed_items(), ButtonGroupsStyle::Standard)
                                            .size(ButtonGroupsSize::ExtraSmall)
                                            .variant(variant)
                                            .selected(vec![1])
                                            .label("Standard color sample");
                                    }
                                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                                    group(label_items(), ButtonGroupsStyle::Connected)
                                        .size(ButtonGroupsSize::ExtraSmall)
                                        .variant(variant)
                                        .mode(ButtonGroupsSelectionMode::Single)
                                        .selected(vec![0])
                                        .label("Connected color sample");
                                });
                        }),
                    );
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "Filled".into(),
        "Tonal".into(),
        "Outlined".into(),
        "Elevated".into(),
    ]);
}

#[tessera]
fn selection_and_activation() {
    prose(
        "Standard button groups add interaction between adjacent buttons when a button is selected or activated.",
    );
    gap(Dp(8.0));
    prose(
        "This interaction changes the width, shape, and padding of the selected or activated button, which adjusts the width of buttons directly next to it.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![
                icon_only(filled::BLUETOOTH_SVG, "Bluetooth"),
                icon_only(filled::ALARM_SVG, "Alarm"),
                icon_only(filled::LINK_SVG, "Link"),
                icon_only(filled::WIFI_SVG, "Wi-Fi"),
            ],
            ButtonGroupsStyle::Standard,
        )
        .size(ButtonGroupsSize::Large)
        .variant(ButtonVariant::Tonal)
        .selected(vec![2])
        .label("Standard activation");
    }));
    caption(
        "A selected button changes shape, and briefly changes the width of itself and adjacent buttons",
    );
    gap(Dp(32.0));
    prose(
        "Connected button groups don't add any interaction between buttons when selected or activated.",
    );
    gap(Dp(8.0));
    prose("They only affect the shape of the button being selected or activated.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![labelled("Label"), labelled("Label"), labelled("Label")],
            ButtonGroupsStyle::Connected,
        )
        .variant(ButtonVariant::Tonal)
        .selected(vec![2])
        .label("Connected selection");
    }));
    caption("A selected button changes shape without affecting adjacent buttons");
}

/// One labelled state row; `state` indexes [`STATES`], `number` is its legend
/// marker, and pressed rows show the pressed shape and width.
#[tessera]
fn state_row(style: ButtonGroupsStyle, state: usize, number: usize, selected: bool) {
    marked(
        number,
        RenderSlot::new(move || {
            let mut presentation = vec![InteractionVisualState::Rest; 3];
            presentation[1] = state_presentation(state);
            group(
                vec![
                    icon_label(filled::EDIT_SVG, STATES[state]),
                    icon_label(filled::EDIT_SVG, "Label"),
                    icon_label(filled::EDIT_SVG, "Label"),
                ],
                style,
            )
            .size(ButtonGroupsSize::Small)
            .variant(ButtonVariant::Tonal)
            .enabled(state != 1)
            .presentation(presentation)
            .selected(if selected { vec![1] } else { Vec::new() })
            .label(format!("{} state", STATES[state]));
        }),
    );
    gap(Dp(12.0));
}

#[tessera]
fn state_figure(style: ButtonGroupsStyle, selected: bool, states: Vec<usize>) {
    panel(RenderSlot::new(move || {
        let states = states.clone();
        column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move || {
                for (index, state) in states.clone().into_iter().enumerate() {
                    state_row(style, state, index + 1, selected);
                }
            });
    }));
    gap(Dp(16.0));
}

#[tessera]
fn states() {
    subheading("Standard button group".into());
    gap(Dp(16.0));
    prose(
        "When a button is pressed, standard button groups modify the width and shape of that button and adjacent buttons.",
    );
    gap(Dp(24.0));
    state_figure(ButtonGroupsStyle::Standard, false, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(32.0));
    prose(
        "When a toggle button is selected in a standard button group, its shape should change between square and round. The color should change according to the button specs.",
    );
    gap(Dp(24.0));
    state_figure(ButtonGroupsStyle::Standard, true, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(48.0));
    subheading("Connected button group".into());
    gap(Dp(16.0));
    prose(
        "Connected button groups have different shape changes than standard button groups. Selecting a button does not affect adjacent buttons.",
    );
    gap(Dp(24.0));
    prose("Connected button group unselected states:");
    gap(Dp(16.0));
    state_figure(ButtonGroupsStyle::Connected, false, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(32.0));
    prose("Connected button group selected states:");
    gap(Dp(16.0));
    state_figure(ButtonGroupsStyle::Connected, true, vec![0, 2, 3, 4]);
    legend(vec![
        "Enabled".into(),
        "Hovered".into(),
        "Focused".into(),
        "Pressed".into(),
    ]);
}

#[tessera]
fn measurements() {
    subheading("Standard button group".into());
    gap(Dp(16.0));
    prose(
        "Standard groups apply padding between all buttons. The amount of padding changes based on button size to ensure a minimum accessible target size of 48dp.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in SIZES {
                    group(mixed_items(), ButtonGroupsStyle::Standard)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .label(format!("{} padding", size_name(size)));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose("Standard button group inner padding:");
    bullets(vec![
        "XS: 18dp".into(),
        "S: 12dp".into(),
        "M: 8dp".into(),
        "L: 8dp".into(),
        "XL: 8dp".into(),
    ]);
    gap(Dp(48.0));
    subheading("Connected button group".into());
    gap(Dp(16.0));
    prose(
        "For all connected button groups, use 2dp padding. This provides visual consistency at scale.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in SIZES {
                    group(label_items(), ButtonGroupsStyle::Connected)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .label(format!("{} round connected", size_name(size)));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose(
        "Round connected button group inner padding is 2dp at every size. The outer shape is fully round, and the inner shape remains square with the following corner sizes:",
    );
    bullets(vec![
        "XS: 4dp".into(),
        "S: 8dp".into(),
        "M: 8dp".into(),
        "L: 16dp".into(),
        "XL: 20dp".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in SIZES {
                    group(label_items(), ButtonGroupsStyle::Connected)
                        .size(size)
                        .shape(ButtonShape::Square)
                        .variant(ButtonVariant::Tonal)
                        .label(format!("{} square connected", size_name(size)));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose(
        "Square connected button group inner padding is 2dp at every size. The outer shape has the following corner sizes:",
    );
    bullets(vec![
        "XS: 4dp".into(),
        "S: 8dp".into(),
        "M: 8dp".into(),
        "L: 16dp".into(),
        "XL: 20dp".into(),
    ]);
    gap(Dp(48.0));
    subheading("Minimum widths".into());
    gap(Dp(16.0));
    prose(
        "Extra small and small connected button groups have 48dp target areas and a minimum width of 48dp.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in [ButtonGroupsSize::ExtraSmall, ButtonGroupsSize::Small] {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            group(
                                vec![
                                    icon_only(filled::STAR_SVG, "Favorites"),
                                    icon_only(filled::ALARM_SVG, "Alarm"),
                                    icon_only(filled::LINK_SVG, "Link"),
                                ],
                                ButtonGroupsStyle::Connected,
                            )
                            .size(size)
                            .variant(ButtonVariant::Tonal)
                            .label(format!("{} minimum width", size_name(size)));
                            gap(Dp(12.0));
                            text()
                                .content(size_name(size))
                                .color(INK)
                                .style(type_style(14.0, 20.0));
                        });
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                }
            });
    }));
}

#[tessera]
fn density() {
    prose("Button groups adapt to density of the buttons inside.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(label_items(), ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .variant(ButtonVariant::Tonal)
                    .label("Default density");
                gap(Dp(16.0));
                group(label_items(), ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .variant(ButtonVariant::Tonal)
                    .label("Reduced height");
            });
    }));
    caption(
        "Button groups adapt to the height of the buttons inside, including when density is applied",
    );
}

#[tessera]
fn usage() {
    prose("There are two variants of button groups: standard and connected.");
    gap(Dp(16.0));
    prose(
        "Standard button groups add interaction between adjacent buttons so they respond to each other. When a button in a standard group is selected:",
    );
    bullets(vec![
        "The selected button changes shape and width".into(),
        "A selected toggle button also changes color".into(),
        "Adjacent buttons move and temporarily change width".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![
                icon_only(filled::MIC_SVG, "Microphone"),
                icon_only(filled::VIDEOCAM_SVG, "Camera"),
                icon_only(filled::MOOD_SVG, "Reactions"),
                icon_only(filled::MORE_VERT_SVG, "More"),
            ],
            ButtonGroupsStyle::Standard,
        )
        .size(ButtonGroupsSize::Medium)
        .variant(ButtonVariant::Tonal)
        .selected(vec![1])
        .label("Call controls");
    }));
    caption("Standard button groups add interaction between adjacent buttons");
    gap(Dp(48.0));
    subheading("Button groups add more expression to a product".into());
    gap(Dp(16.0));
    prose(
        "Mix and match the different button variants, widths, and colors to emphasize what's important, and to visually group related buttons.",
    );
    gap(Dp(16.0));
    prose(
        "By default, all buttons in a standard group should be the same size (XS to XL) and shape (round or square).",
    );
    bullets(vec![
        "Only use multiple sizes in a group for hero moments".into(),
        "Avoid mixing sizes frequently".into(),
        "Only use a different shape in a group when a button is selected, or to add meaning or contrast".into(),
    ]);
    guidance(
        "Do",
        "Use the same shapes for buttons in a group, but change other properties like width and color",
        DO_COLOR,
    );
    guidance(
        "Caution",
        "Reserve shape differences in button groups for key interactions",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("Connected button groups".into());
    gap(Dp(16.0));
    prose(
        "Connected button groups help people select options, switch views, or sort elements in a page.",
    );
    gap(Dp(8.0));
    prose("They behave similarly to standard groups, except they don't affect adjacent buttons.");
    gap(Dp(8.0));
    prose(
        "Connected groups should replace the baseline segmented button, which is no longer recommended.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![labelled("Recent"), labelled("Starred"), labelled("Shared")],
            ButtonGroupsStyle::Connected,
        )
        .mode(ButtonGroupsSelectionMode::Single)
        .required(true)
        .selected(vec![0])
        .label("Views");
    }));
    caption("Connected button groups can be used to toggle between similar actions");
    gap(Dp(32.0));
    prose(
        "Use connected button groups when the button content is related, and buttons can be selected.",
    );
    caption("Closely related actions work well in a connected button group");
    gap(Dp(32.0));
    prose(
        "Connected button groups should be used for single or multi-select patterns that use toggle buttons.",
    );
    gap(Dp(8.0));
    prose("Avoid using a connected group when none of the buttons can be toggled.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![
                labelled("Wi-Fi"),
                labelled("Cellular"),
                labelled("Ethernet"),
            ],
            ButtonGroupsStyle::Connected,
        )
        .selected(vec![0, 2])
        .label("Networks");
    }));
    caption("Use the connected button group with single or multi-select patterns");
    gap(Dp(48.0));
    subheading("Color".into());
    gap(Dp(16.0));
    prose(
        "Avoid mixing color styles in connected button groups; it can make selection and emphasis unclear.",
    );
    guidance(
        "Don't",
        "Don't mix color styles in connected button groups",
        DONT_COLOR,
    );
}

#[tessera]
fn guidelines_anatomy() {
    subheading("Container".into());
    gap(Dp(16.0));
    prose(
        "The standard button group container has padding between buttons so they can animate width and shape without disrupting the product layout.",
    );
    gap(Dp(8.0));
    prose("The standard button group hugs the width of the buttons inside.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(
            vec![labelled("Button"), labelled("Button"), labelled("Button")],
            ButtonGroupsStyle::Standard,
        )
        .variant(ButtonVariant::Tonal)
        .label("Hugging group");
    }));
    caption("Button groups can animate without affecting their surroundings");
    gap(Dp(32.0));
    prose(
        "The connected button group should span the width of the page or surface it's placed on, increasing the button widths inside.",
    );
    gap(Dp(8.0));
    prose(
        "In larger windows, consider adding a maximum width to the connected group to avoid it growing too wide.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        boxed()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                button_groups()
                    .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                        label: Some("Full width group".into()),
                        ..Default::default()
                    }))
                    .style(ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .selection_mode(ButtonGroupsSelectionMode::Single)
                    .child_shared(
                        RenderSlot::new(|| {
                            text().content("Day");
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(|| {
                            text().content("Week");
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(|| {
                            text().content("Month");
                        }),
                        CallbackWith::new(|_| {}),
                    );
            });
    }));
    caption(
        "Connected button groups increase the widths of each button inside and expand to their container width",
    );
}

#[tessera]
fn adaptive() {
    prose(
        "Adaptive design allows an interface to respond or change based on context, such as the user, device, and usage.",
    );
    gap(Dp(32.0));
    subheading("Resizing".into());
    gap(Dp(16.0));
    prose(
        "Button groups should move through layouts together in a single line. They shouldn't wrap to a second line. Multiple button groups can be stacked to keep items close together. However, button groups don't interact vertically.",
    );
    gap(Dp(16.0));
    prose("Button groups and individual buttons can be set to fixed or flexible resizing:");
    bullets(vec![
        "Fixed: Manually define the button width (narrow to wide), size (XS to XL), or padding at each breakpoint.".into(),
        "Flexible: Automatically increase or decrease the width of buttons and the button group. Button groups grow until all flexible buttons are at their largest width.".into(),
    ]);
    gap(Dp(16.0));
    prose(
        "If adjusting button width manually, avoid stretching icon buttons beyond the wide setting.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                for size in [
                    ButtonGroupsSize::ExtraSmall,
                    ButtonGroupsSize::Small,
                    ButtonGroupsSize::Medium,
                ] {
                    group(mixed_items(), ButtonGroupsStyle::Standard)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .selected(vec![1])
                        .label(format!("{} breakpoint", size_name(size)));
                    gap(Dp(16.0));
                }
            });
    }));
    caption(
        "Buttons can have width, size, and padding manually adjusted to fit different breakpoints",
    );
    gap(Dp(32.0));
    prose(
        "In compact windows, consider using smaller, narrower buttons so all buttons in the button group can fit. In large and extra large windows, consider using larger, wider buttons to better fill in the available space.",
    );
    gap(Dp(8.0));
    prose("Flexible buttons or button groups will automatically adjust width.");
    gap(Dp(32.0));
    prose(
        "When scaling to larger breakpoints, make sure that the visual hierarchy of each button is preserved using qualities like color and size.",
    );
    gap(Dp(8.0));
    prose(
        "For example, the primary action should remain the largest, widest, or most visually prominent button at all breakpoints.",
    );
    gap(Dp(48.0));
    subheading("Presentation".into());
    gap(Dp(16.0));
    prose(
        "Buttons at the trailing edge of the button group can be customized to collapse into an overflow menu at smaller breakpoints, and become visible again at larger sizes. Place the overflow menu at the trailing end of the group.",
    );
    gap(Dp(8.0));
    prose("Buttons outside the group aren't affected by button group behavior.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(
                    vec![
                        icon_only(filled::MIC_SVG, "Microphone"),
                        icon_only(filled::VIDEOCAM_SVG, "Camera"),
                        icon_only(filled::MOOD_SVG, "Reactions"),
                        icon_only(filled::PRESENT_TO_ALL_SVG, "Present"),
                        icon_only(filled::MORE_VERT_SVG, "More"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label("Expanded call controls");
                gap(Dp(16.0));
                group(
                    vec![
                        icon_only(filled::MIC_SVG, "Microphone"),
                        icon_only(filled::VIDEOCAM_SVG, "Camera"),
                        icon_only(filled::MORE_VERT_SVG, "More"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label("Collapsed call controls");
            });
    }));
    caption(
        "Buttons should become hidden in an overflow menu or visible again, depending on screen size. Buttons outside the button group, like the end call button, will not be affected.",
    );
}

#[tessera]
fn behavior() {
    subheading("Pressed".into());
    gap(Dp(16.0));
    prose("When a button is pressed, it changes width and shape.");
    gap(Dp(8.0));
    prose(
        "In a standard button group, pressing a button also affects the width of adjacent buttons.",
    );
    gap(Dp(8.0));
    prose("In a connected button group, only the shape of the pressed button changes.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(
                    vec![
                        labelled("AC"),
                        labelled("( )"),
                        labelled("%"),
                        labelled("÷"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .size(ButtonGroupsSize::Medium)
                .variant(ButtonVariant::Tonal)
                .presentation(vec![
                    InteractionVisualState::Rest,
                    InteractionVisualState::Pressed,
                    InteractionVisualState::Rest,
                    InteractionVisualState::Rest,
                ])
                .label("Pressed standard group");
                gap(Dp(16.0));
                group(
                    vec![labelled("7"), labelled("8"), labelled("9")],
                    ButtonGroupsStyle::Connected,
                )
                .size(ButtonGroupsSize::Medium)
                .variant(ButtonVariant::Tonal)
                .presentation(vec![
                    InteractionVisualState::Rest,
                    InteractionVisualState::Pressed,
                    InteractionVisualState::Rest,
                ])
                .label("Pressed connected group");
            });
    }));
    caption("Pressing buttons in a standard group changes the width of adjacent buttons");
    gap(Dp(48.0));
    subheading("Selected".into());
    gap(Dp(16.0));
    prose("A selected button should change shape from round to square, or square to round.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Medium)
                    .selected(vec![1])
                    .label("Round group with selection");
                gap(Dp(16.0));
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Medium)
                    .shape(ButtonShape::Square)
                    .selected(vec![1])
                    .label("Square group with selection");
            });
    }));
    caption("Selected buttons should change shape");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to and interact with each button in the group".into(),
        "Identify when buttons are selected".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose("Each button in a group should have a minimum 48x48dp target.");
    gap(Dp(8.0));
    prose(
        "Extra small and small button groups have larger inner padding to ensure accessible targets. Avoid reducing the padding in these sizes.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .variant(ButtonVariant::Tonal)
                    .label("Extra small button group");
                gap(Dp(8.0));
                text()
                    .content("Extra small button group")
                    .style(type_style(14.0, 20.0));
                gap(Dp(24.0));
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Small)
                    .variant(ButtonVariant::Tonal)
                    .label("Small button group");
                gap(Dp(8.0));
                text()
                    .content("Small button group")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(48.0));
    subheading("Initial focus".into());
    gap(Dp(16.0));
    prose(
        "The button group container is not a focusable element. Initial focus should land on the first button in the group and then move to each button.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        group(mixed_items(), ButtonGroupsStyle::Standard)
            .size(ButtonGroupsSize::Medium)
            .presentation(vec![InteractionVisualState::Focus])
            .label("Focus order");
    }));
    caption("Initial focus should land on the first button, not on the container");
    gap(Dp(32.0));
    prose(
        "Use Tab to navigate to a button group, the arrow keys to navigate between each item in the group, and Space or Enter to select a focused button.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Tab: Initial focus lands on the first button".into(),
        "Arrow keys: Navigate between items".into(),
        "Space or Enter: Selects a focused item".into(),
    ]);
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate between buttons".into()],
            vec!["Arrow keys".into(), "Navigate inside the component".into()],
            vec!["Space or Enter".into(), "Activate a focused button".into()],
        ],
    );
}

#[tessera]
fn labeling() {
    prose(
        "The button group container doesn't need to be labeled. Label each button according to the button and icon button accessibility guidance.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Dinner Club")
                    .color(INK)
                    .style(type_style(20.0, 28.0));
                gap(Dp(8.0));
                text().content("3 Messages").style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                group(
                    vec![
                        icon_only(filled::REPLY_SVG, "Reply"),
                        icon_only(filled::ARCHIVE_SVG, "Archive"),
                        icon_only(filled::DELETE_SVG, "Delete"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label("Message actions");
            });
    }));
    caption("Label each button within the button group");
}
