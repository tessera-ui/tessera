//! Interactive button group specifications.
//!
//! ## Usage
//!
//! Compare standard and connected groups, selection policies and measurements.

use tessera_components::{
    alignment::CrossAxisAlignment,
    button::{ButtonShape, button},
    button_groups::{
        ButtonGroupsSelectionMode, ButtonGroupsSize, ButtonGroupsState, ButtonGroupsStyle,
        button_groups,
    },
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{CallbackWith, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const SIZES: [ButtonGroupsSize; 5] = [
    ButtonGroupsSize::ExtraSmall,
    ButtonGroupsSize::Small,
    ButtonGroupsSize::Medium,
    ButtonGroupsSize::Large,
    ButtonGroupsSize::ExtraLarge,
];

fn size_name(size: ButtonGroupsSize) -> &'static str {
    match size {
        ButtonGroupsSize::ExtraSmall => "Extra small",
        ButtonGroupsSize::Small => "Small",
        ButtonGroupsSize::Medium => "Medium",
        ButtonGroupsSize::Large => "Large",
        ButtonGroupsSize::ExtraLarge => "Extra large",
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
                    title: "Color",
                    content: RenderSlot::new(color),
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
                                .background_with_shape(
                                    scheme.on_surface_variant,
                                    tessera_components::shape_def::Shape::Ellipse,
                                ),
                        );
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text().content(item.clone()).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// Renders one labelled group example with optional configuration overrides.
#[tessera]
fn sample(
    #[prop(into)] label: String,
    style: ButtonGroupsStyle,
    size: Option<ButtonGroupsSize>,
    shape: Option<ButtonShape>,
    mode: Option<ButtonGroupsSelectionMode>,
    required: Option<bool>,
    enabled: Option<bool>,
    icons: Option<bool>,
    status: Option<bool>,
) {
    let state = remember(ButtonGroupsState::new);
    let size = size.unwrap_or(ButtonGroupsSize::Medium);
    let shape = shape.unwrap_or(ButtonShape::Round);
    let mode = mode.unwrap_or(ButtonGroupsSelectionMode::Single);
    let required = required.unwrap_or(false);
    let enabled = enabled.unwrap_or(true);
    let icons = icons.unwrap_or(false);
    let status = status.unwrap_or(false);
    let status_label = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text()
                .content(label.clone())
                .color(INK)
                .style(type_style(16.0, 24.0));
            gap(Dp(16.0));
            button_groups()
                .style(style)
                .size(size)
                .shape_style(shape)
                .selection_mode(mode)
                .selection_required(required)
                .enabled(enabled)
                .state(state)
                .child_shared(
                    RenderSlot::new(move || {
                        if icons {
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
                        if icons {
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
                        if icons {
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
            if status {
                gap(Dp(16.0));
                text()
                    .content(format!(
                        "{} selection: {}",
                        status_label.clone(),
                        (0..3)
                            .filter(|index| state.with(|state| state.is_selected(*index)))
                            .map(|index| (index + 1).to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn overview() {
    prose(
        "Button groups organize buttons and add interactions between them. Use a group when several related actions belong together and should respond to one another.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Two variants: standard and connected".into(),
        "Standard groups add interaction between adjacent buttons".into(),
        "Connected groups join buttons with a 2dp gap and coordinated corners".into(),
        "Five sizes: extra small, small, medium, large, and extra large".into(),
        "Two shapes: round and square".into(),
        "Single-select, multi-select, and selection-required policies".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Standard button group", ButtonGroupsStyle::Standard)
                    .icons(true)
                    .status(true);
                gap(Dp(40.0));
                sample("Connected button group", ButtonGroupsStyle::Connected)
                    .icons(true)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    prose("1  Standard button group    2  Connected button group");
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
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Available".into(),
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
        "Tessera examples on this page use the native button_groups component. Standard and connected layouts, five sizes, both shape presets, single and multiple selection, selection-required, per-item disabling, and coordinated shape animation are implemented.",
    );
}

#[tessera]
fn expressive() {
    prose(
        "May 2025: button groups are new in M3 Expressive. Before this update, the closest component was the segmented button, which provided joined choices but not the standard layout or the adjacent-button interaction.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Change".into(), "Details".into()],
        vec![
            vec!["Variants".into(), "Standard and connected layouts".into()],
            vec![
                "Interaction".into(),
                "Standard groups coordinate shape and width across adjacent buttons".into(),
            ],
            vec![
                "Sizes".into(),
                "Extra small, small, medium, large, and extra large".into(),
            ],
            vec![
                "Selection".into(),
                "Single-select, multi-select, and selection-required".into(),
            ],
            vec!["Shapes".into(), "Round and square presets".into()],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        sample("Connected choices", ButtonGroupsStyle::Connected)
            .mode(ButtonGroupsSelectionMode::Multiple)
            .icons(true)
            .status(true);
    }));
    caption("Closely related actions work well in a connected button group.");
}

#[tessera]
fn variants() {
    prose(
        "Standard groups apply padding between their buttons. Connected groups place the buttons 2dp apart, share a single container outline, and keep their widths stable when a choice is selected.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Standard button group", ButtonGroupsStyle::Standard)
                    .icons(true)
                    .status(true);
                gap(Dp(40.0));
                sample("Connected button group", ButtonGroupsStyle::Connected)
                    .icons(true)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    caption(
        "Both variants keep selection visible; only the standard variant moves its neighbours.",
    );
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
    prose(
        "Choose a size, shape, and selection policy. Medium is the default size and round is the default shape. All buttons in a group normally share one size and one shape.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("1  Size".into());
                gap(Dp(24.0));
                for size in SIZES {
                    sample(size_name(size), ButtonGroupsStyle::Connected)
                        .size(size)
                        .icons(true);
                    gap(Dp(24.0));
                }
                gap(Dp(16.0));
                subheading("2  Shape".into());
                gap(Dp(24.0));
                sample("Round shape", ButtonGroupsStyle::Connected)
                    .shape(ButtonShape::Round)
                    .icons(true);
                gap(Dp(24.0));
                sample("Square shape", ButtonGroupsStyle::Connected)
                    .shape(ButtonShape::Square)
                    .icons(true);
                gap(Dp(32.0));
                subheading("3  Selection".into());
                gap(Dp(24.0));
                sample("Single-select", ButtonGroupsStyle::Standard)
                    .mode(ButtonGroupsSelectionMode::Single)
                    .status(true);
                gap(Dp(24.0));
                sample("Multi-select", ButtonGroupsStyle::Standard)
                    .mode(ButtonGroupsSelectionMode::Multiple)
                    .status(true);
                gap(Dp(24.0));
                sample("Selection required", ButtonGroupsStyle::Standard)
                    .required(true)
                    .status(true);
            });
    }));
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
    prose(
        "Group metrics follow the button size presets. Standard groups spend their padding on separate containers; connected groups share a 2dp gap and coordinate their corners.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Property".into(), "Value".into()],
        vec![
            vec![
                "Container height · XS / S / M / L / XL".into(),
                "32 / 40 / 56 / 96 / 136dp".into(),
            ],
            vec![
                "Standard inner padding · XS / S / M / L / XL".into(),
                "18 / 12 / 8 / 8 / 8dp".into(),
            ],
            vec!["Connected inner padding".into(), "2dp at every size".into()],
            vec![
                "Connected inner corner · XS / S / M / L / XL".into(),
                "4 / 8 / 8 / 16 / 20dp".into(),
            ],
            vec![
                "Pressed width expansion · standard".into(),
                "15% of the pressed button width".into(),
            ],
            vec!["Minimum interactive target".into(), "48 × 48dp".into()],
            vec![
                "Focus traversal".into(),
                "Horizontal focus group that wraps at the edges".into(),
            ],
            vec![
                "Semantics".into(),
                "Group role, horizontal orientation, merged descendants".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "This build uses 8dp inner corners for extra small connected groups; the specification calls for 4dp. Group heights never fall below the 48dp interactive target.",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "A button group is a single container that arranges its buttons. The container itself is not a focus stop and does not need a label; each button keeps its own visible label.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Connected container", ButtonGroupsStyle::Connected)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    text()
        .content("1  Container")
        .color(INK)
        .style(type_style(16.0, 24.0));
    gap(Dp(32.0));
    subheading("Common layouts".into());
    gap(Dp(8.0));
    prose("Mix and match buttons and icon buttons for different scenarios.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Label buttons", ButtonGroupsStyle::Standard)
                    .icons(false)
                    .status(true);
                gap(Dp(32.0));
                sample("Icon and label buttons", ButtonGroupsStyle::Connected)
                    .icons(true)
                    .status(true);
            });
    }));
}

#[tessera]
fn selection_and_activation() {
    prose(
        "Standard button groups add interaction between adjacent buttons when a button is selected or activated. This interaction changes the width, shape, and padding of the selected or activated button, which adjusts the width of buttons directly next to it.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Standard activation", ButtonGroupsStyle::Standard)
            .mode(ButtonGroupsSelectionMode::Multiple)
            .status(true);
    }));
    gap(Dp(16.0));
    caption(
        "A selected button changes shape, and briefly changes the width of itself and adjacent buttons.",
    );
    gap(Dp(32.0));
    prose(
        "Connected button groups don't add any interaction between buttons when selected or activated. They only affect the shape of the button being selected or activated.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Connected selection", ButtonGroupsStyle::Connected)
            .mode(ButtonGroupsSelectionMode::Multiple)
            .status(true);
    }));
    gap(Dp(16.0));
    caption("A selected button changes shape without affecting adjacent buttons.");
    gap(Dp(32.0));
    spec_table(
        vec!["Policy".into(), "Result".into()],
        vec![
            vec![
                "Single-select".into(),
                "Selecting a choice clears the previous selection".into(),
            ],
            vec![
                "Multi-select".into(),
                "Several choices can stay selected".into(),
            ],
            vec![
                "Selection required".into(),
                "The last selected choice cannot be cleared; an empty selection restores the first enabled choice".into(),
            ],
            vec![
                "Activation".into(),
                "Every click toggles its own choice and reports the new state to the item callback".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "Groups inherit the interaction states of their buttons. Hover, focus, and pressed feedback applies to the individual button under the pointer or keyboard focus, while the container stays visually quiet.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Enabled", ButtonGroupsStyle::Standard)
                    .icons(true)
                    .status(true);
                gap(Dp(32.0));
                sample("Disabled group", ButtonGroupsStyle::Standard)
                    .enabled(false)
                    .icons(true);
                gap(Dp(32.0));
                sample("Selection required", ButtonGroupsStyle::Connected)
                    .required(true)
                    .icons(true)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    prose("1  Enabled    2  Disabled    3  Selection required");
    gap(Dp(32.0));
    spec_table(
        vec!["State".into(), "Behavior".into()],
        vec![
            vec![
                "Enabled".into(),
                "Buttons accept input and show shape feedback".into(),
            ],
            vec![
                "Disabled".into(),
                "Group keeps its selection and stops accepting input".into(),
            ],
            vec![
                "Hovered, focused, pressed".into(),
                "Applied per button inside the group".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    prose(
        "Standard groups apply padding between all buttons. The amount of padding changes based on button size to ensure a minimum accessible target size of 48dp.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for size in SIZES {
                    sample(size_name(size), ButtonGroupsStyle::Standard)
                        .size(size)
                        .icons(true);
                    gap(Dp(24.0));
                }
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec![
            "Size".into(),
            "Visual height".into(),
            "Standard padding".into(),
            "Connected padding".into(),
        ],
        vec![
            vec!["XS".into(), "32dp".into(), "18dp".into(), "2dp".into()],
            vec!["S".into(), "40dp".into(), "12dp".into(), "2dp".into()],
            vec!["M".into(), "56dp".into(), "8dp".into(), "2dp".into()],
            vec!["L".into(), "96dp".into(), "8dp".into(), "2dp".into()],
            vec!["XL".into(), "136dp".into(), "8dp".into(), "2dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Minimum widths".into());
    gap(Dp(8.0));
    prose(
        "Extra small and small connected button groups have 48dp target areas and a minimum width of 48dp. Round connected groups keep their inner corners square.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Extra small", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .icons(true);
                gap(Dp(24.0));
                sample("Small", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .icons(true);
            });
    }));
}

#[tessera]
fn density() {
    prose(
        "Button groups adapt to the density of the buttons inside. Reducing density lowers the visible height of the group, and the padding between buttons scales with the size preset rather than staying fixed.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Default height", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .icons(true);
                gap(Dp(24.0));
                sample("Extra small height", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .icons(true);
            });
    }));
    gap(Dp(16.0));
    caption(
        "Button groups adapt to the height of the buttons inside, including when density is applied.",
    );
    gap(Dp(32.0));
    prose(
        "This build keeps the size-preset heights and never drops below the 48dp interactive target, so extra small and small groups keep a 48dp hit area around a 32dp or 40dp container.",
    );
}

#[tessera]
fn usage() {
    prose("There are two variants of button groups: standard and connected.");
    gap(Dp(16.0));
    prose(
        "Standard button groups add interaction between adjacent buttons so they respond to each other. When a button in a standard group is selected, the selected button changes shape and width, a selected toggle button also changes color, and adjacent buttons move and temporarily change width.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Standard group", ButtonGroupsStyle::Standard)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    caption("Standard button groups add interaction between adjacent buttons.");
    gap(Dp(32.0));
    prose(
        "Connected button groups should be used for single or multi-select patterns that use toggle buttons. Avoid using a connected group when none of the buttons can be toggled.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Connected group", ButtonGroupsStyle::Connected)
            .mode(ButtonGroupsSelectionMode::Multiple)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    caption("Use the connected button group with single or multi-select patterns.");
    gap(Dp(32.0));
    prose(
        "By default, all buttons in a standard group should be the same size (XS to XL) and shape (round or square). Only use multiple sizes in a group for hero moments, avoid mixing sizes frequently, and only use a different shape in a group for a key interaction.",
    );
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "A group arranges related buttons in one container. Keep the buttons on a shared baseline, give each one a distinct label, and use the same size and shape across the group so the set reads as one control.",
    );
    gap(Dp(24.0));
    anatomy();
    gap(Dp(24.0));
    prose(
        "Icons are optional. When several buttons share an icon family, pair each icon with a label unless the meaning is unmistakable.",
    );
}

#[tessera]
fn color() {
    prose(
        "Groups use the color styles of their buttons. Filled, tonal, and outlined choices read differently at rest than when selected, and selection is carried by container color as well as shape.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Filled choices", ButtonGroupsStyle::Connected)
                    .icons(true)
                    .status(true);
                gap(Dp(32.0));
                sample("Outline-style choices", ButtonGroupsStyle::Standard)
                    .icons(false)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    caption(
        "Avoid mixing color styles in connected button groups; it can make selection and emphasis unclear.",
    );
    gap(Dp(32.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Unselected container".into(), "Surface container".into()],
            vec!["Unselected content".into(), "On surface variant".into()],
            vec!["Selected container".into(), "Primary".into()],
            vec!["Selected content".into(), "On primary".into()],
            vec!["Pressed state layer".into(), "Content color at 10%".into()],
        ],
    );
}

#[tessera]
fn adaptive() {
    prose(
        "Choose the size that matches the surrounding density. Extra small and small groups suit compact toolbars; large and extra large groups give a primary set of choices more presence in wide layouts.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Extra small", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .icons(true);
                gap(Dp(24.0));
                sample("Large", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Large)
                    .icons(true);
            });
    }));
    gap(Dp(16.0));
    caption("The same three choices at two size presets.");
    gap(Dp(32.0));
    prose(
        "When a group runs out of horizontal space, keep the size and shorten the labels instead of dropping below the 48dp target. Groups wrap only when the layout provides a new line.",
    );
}

#[tessera]
fn behavior() {
    prose(
        "Activating a choice toggles its selection and reports the new state to the item callback. Single-select groups clear the previous choice, multi-select groups keep several active, and selection-required groups refuse to clear the final one.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Selection required", ButtonGroupsStyle::Connected)
                    .required(true)
                    .icons(true)
                    .status(true);
                gap(Dp(32.0));
                sample("Multi-select", ButtonGroupsStyle::Connected)
                    .mode(ButtonGroupsSelectionMode::Multiple)
                    .icons(true)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    caption(
        "Activate the same choice twice to compare the single, multiple, and required policies.",
    );
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to and interact with each button in the group".into(),
        "Identify when buttons are selected".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Assistive activation", ButtonGroupsStyle::Connected)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    caption("Each choice keeps its visible label and announces whether it is selected.");
}

#[tessera]
fn interaction_style() {
    subheading("Target size".into());
    gap(Dp(16.0));
    prose(
        "Each button in a group should have a minimum 48 × 48dp target. Extra small and small button groups have larger inner padding to ensure accessible targets; avoid reducing the padding in these sizes.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample("Extra small button group", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .icons(true)
                    .status(true);
                gap(Dp(24.0));
                sample("Small button group", ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .icons(true)
                    .status(true);
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
        sample("Focus order", ButtonGroupsStyle::Standard)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    caption(
        "Press Tab to focus a choice, then use the arrow keys to move inside the group and Space or Enter to toggle the focused choice.",
    );
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
    gap(Dp(24.0));
    prose(
        "The group is a horizontal focus group that wraps at its edges, so focus enters the first button and then visits each button in visual order. The container itself is skipped, which keeps Tab from stopping on the group twice.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        sample("Keyboard group", ButtonGroupsStyle::Connected)
            .mode(ButtonGroupsSelectionMode::Multiple)
            .icons(true)
            .status(true);
    }));
    gap(Dp(16.0));
    caption(
        "Press Tab to focus a choice, then use the arrow keys to move inside the group and Space or Enter to toggle the focused choice.",
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
                sample("Message actions", ButtonGroupsStyle::Connected)
                    .mode(ButtonGroupsSelectionMode::Multiple)
                    .icons(false)
                    .status(true);
            });
    }));
    gap(Dp(16.0));
    caption("Label each button within the button group; the container itself stays unlabeled.");
    gap(Dp(32.0));
    prose(
        "Prefer distinct words over repeated labels. When two buttons would read the same, add the distinguishing detail to the visible label instead of relying on an automated description.",
    );
}
