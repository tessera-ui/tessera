//! Split button specifications with anchored option menus.
//!
//! ## Usage
//!
//! Compare a main action with its related menu actions across sizes and styles.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::{ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    split_buttons::{
        SplitButtonDefaults, SplitButtonSize, SplitButtonVariant, split_button_layout,
        split_leading_button, split_trailing_button,
    },
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, PxPosition, PxRect, PxSize, RenderSlot, State,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

const VARIANTS: [SplitButtonVariant; 4] = [
    SplitButtonVariant::Elevated,
    SplitButtonVariant::Filled,
    SplitButtonVariant::Tonal,
    SplitButtonVariant::Outlined,
];

const SIZES: [SplitButtonSize; 5] = [
    SplitButtonSize::ExtraSmall,
    SplitButtonSize::Small,
    SplitButtonSize::Medium,
    SplitButtonSize::Large,
    SplitButtonSize::ExtraLarge,
];

fn variant_name(variant: SplitButtonVariant) -> &'static str {
    match variant {
        SplitButtonVariant::Elevated => "Elevated",
        SplitButtonVariant::Filled => "Filled",
        SplitButtonVariant::Tonal => "Tonal",
        SplitButtonVariant::Outlined => "Outlined",
    }
}

fn size_name(size: SplitButtonSize) -> &'static str {
    match size {
        SplitButtonSize::ExtraSmall => "Extra small",
        SplitButtonSize::Small => "Small",
        SplitButtonSize::Medium => "Medium",
        SplitButtonSize::Large => "Large",
        SplitButtonSize::ExtraLarge => "Extra large",
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Menu {
    controller: State<MenuController>,
    anchor: State<PxRect>,
    origin: State<PxRect>,
    source: State<String>,
    result: State<String>,
    theme: State<MaterialTheme>,
}

#[tessera]
pub(super) fn split_buttons_spec() {
    let menu = Menu {
        controller: remember(MenuController::new),
        anchor: remember(PxRect::default),
        origin: remember(PxRect::default),
        source: remember(String::new),
        result: remember(|| "No option selected".to_owned()),
        theme: remember(MaterialTheme::default),
    };
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .on_globally_positioned(move |rect| {
                    if menu.origin.get() != rect {
                        menu.origin.set(rect);
                    }
                }),
        )
        .children(move || {
            let anchor = menu.anchor.get();
            let origin = menu.origin.get();
            menu_provider()
                .controller(menu.controller)
                .container_color(menu.theme.get().color_scheme.surface_container_low)
                .anchor(MenuAnchor::new(
                    PxPosition::new(anchor.x - origin.x, anchor.y - origin.y),
                    PxSize::new(anchor.width, anchor.height),
                ))
                .main_content(move || {
                    article(
                        "Split button",
                        "Split buttons pair a main action with a menu of related actions",
                        vec![
                            SpecSection {
                                title: "Variants",
                                content: RenderSlot::new(move || {
                                    variants(menu);
                                }),
                            },
                            SpecSection {
                                title: "Configurations",
                                content: RenderSlot::new(move || {
                                    configurations(menu);
                                }),
                            },
                            SpecSection {
                                title: "Tokens & specs",
                                content: RenderSlot::new(tokens),
                            },
                            SpecSection {
                                title: "Anatomy",
                                content: RenderSlot::new(move || {
                                    anatomy(menu);
                                }),
                            },
                            SpecSection {
                                title: "Color",
                                content: RenderSlot::new(move || {
                                    colors(menu);
                                }),
                            },
                            SpecSection {
                                title: "States",
                                content: RenderSlot::new(move || {
                                    states(menu);
                                }),
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
                            intro: Some(RenderSlot::new(move || {
                                overview(menu);
                            })),
                            sections: vec![
                                SpecSection {
                                    title: "Availability & resources",
                                    content: RenderSlot::new(availability),
                                },
                                SpecSection {
                                    title: "M3 Expressive update",
                                    content: RenderSlot::new(move || {
                                        expressive_update(menu);
                                    }),
                                },
                            ],
                        },
                        ArticlePage {
                            title: "Guidelines",
                            intro: None,
                            sections: vec![
                                SpecSection {
                                    title: "Usage",
                                    content: RenderSlot::new(move || {
                                        usage(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "Anatomy",
                                    content: RenderSlot::new(move || {
                                        guide_anatomy(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "Color",
                                    content: RenderSlot::new(move || {
                                        guide_color(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "Adaptive design",
                                    content: RenderSlot::new(move || {
                                        adaptive_design(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "Behavior",
                                    content: RenderSlot::new(move || {
                                        behavior(menu);
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
                })
                .menu_content(move || {
                    material_theme()
                        .theme(move || menu.theme.get())
                        .child(move || {
                            menu_item()
                                .label("Schedule send")
                                .leading_icon(filled::SCHEDULE_SVG)
                                .on_click(move || {
                                    menu.result.set("Schedule send".into());
                                });
                            menu_item()
                                .label("Save draft")
                                .leading_icon(filled::EDIT_SVG)
                                .on_click(move || {
                                    menu.result.set("Save draft".into());
                                });
                        });
                });
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
fn badge(number: usize) {
    boxed()
        .alignment(tessera_components::alignment::Alignment::Center)
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
        .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
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
        .cross_axis_alignment(tessera_components::alignment::CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(tessera_components::alignment::Alignment::Center)
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
fn demo(
    menu: Menu,
    #[prop(into)] label: String,
    variant: SplitButtonVariant,
    size: SplitButtonSize,
    enabled: bool,
) {
    let count = remember(|| 0_u32);
    let bounds = remember(PxRect::default);
    let leading_label = label.clone();
    let trailing_label = label.clone();
    let caption_label = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            split_button_layout()
                .leading_button_shared({
                    let label = leading_label.clone();
                    RenderSlot::new(move || {
                        split_leading_button()
                            .size(size)
                            .variant(variant)
                            .enabled(enabled)
                            .accessibility_label(format!("{label} main action"))
                            .on_click(move || count.set(count.get() + 1))
                            .content(move || {
                                icon().painter(filled::EDIT_SVG);
                                spacer().modifier(
                                    Modifier::new().width(SplitButtonDefaults::icon_spacing(size)),
                                );
                                text().content("Edit");
                            });
                    })
                })
                .trailing_button_shared({
                    let label = trailing_label.clone();
                    let source = caption_label.clone();
                    RenderSlot::new(move || {
                        let name = label.clone();
                        let selected = source.clone();
                        let theme_value =
                            use_context::<MaterialTheme>().expect("MaterialTheme").get();
                        split_trailing_button()
                            .size(size)
                            .variant(variant)
                            .enabled(enabled)
                            .expanded(
                                menu.controller.with(MenuController::is_open)
                                    && menu.source.get() == selected,
                            )
                            .accessibility_label(format!("{label} menu actions"))
                            .modifier(Modifier::new().on_globally_positioned(move |rect| {
                                if bounds.get() != rect {
                                    bounds.set(rect);
                                }
                                if menu.source.get() == name && menu.anchor.get() != rect {
                                    menu.anchor.set(rect);
                                }
                            }))
                            .on_click(move || {
                                if menu.controller.with(MenuController::is_open)
                                    && menu.source.get() == selected
                                {
                                    menu.controller.with_mut(MenuController::close);
                                } else {
                                    menu.theme.set(theme_value.clone());
                                    menu.source.set(selected.clone());
                                    menu.anchor.set(bounds.get());
                                    menu.controller.with_mut(MenuController::open);
                                }
                            })
                            .content(|| {
                                icon().painter(filled::ARROW_DROP_DOWN_SVG);
                            });
                    })
                });
            gap(Dp(12.0));
            text()
                .content(format!("{caption_label}: {} main actions", count.get()))
                .style(type_style(14.0, 20.0));
            if menu.source.get() == caption_label {
                text()
                    .content(format!("Menu choice: {}", menu.result.get()))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn overview(menu: Menu) {
    bullets(vec![
        "Use to show an action with a menu of related actions".into(),
        "Same size range as buttons and icon buttons: XS, S, M, L, XL".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Overview",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Split buttons are made of a common button and a menu icon button");
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
        "Tessera examples on this page use the native split_button_layout with its leading and trailing segments, five sizes and four color variants.",
    );
}

#[tessera]
fn expressive_update(menu: Menu) {
    subheading("May 2025".into());
    prose(
        "The split button has a separate menu button that spins and changes shape when activated. It can be used alongside other buttons of the same size.",
    );
    gap(Dp(24.0));
    prose("New component added to catalog.");
    gap(Dp(16.0));
    prose("Sizes:");
    gap(Dp(8.0));
    bullets(vec![
        "Extra small".into(),
        "Small".into(),
        "Medium".into(),
        "Large".into(),
        "Extra large".into(),
    ]);
    gap(Dp(16.0));
    prose("Color styles:");
    gap(Dp(8.0));
    bullets(vec![
        "Elevated".into(),
        "Filled".into(),
        "Tonal".into(),
        "Outlined".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Expressive",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Split buttons have the same five recommended sizes as label and icon buttons");
}

#[tessera]
fn variants(menu: Menu) {
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Variant",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("A split button pairs a leading action with a trailing menu segment");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![vec!["Split button".into(), "--".into(), "Available".into()]],
    );
}

#[tessera]
fn configurations(menu: Menu) {
    subheading("Color".into());
    prose("Split buttons come in four color configurations.");
    gap(Dp(24.0));
    column().children(move || {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(index + 1, variant_name(variant));
            gap(Dp(16.0));
            panel(RenderSlot::new(move || {
                demo(
                    menu,
                    variant_name(variant),
                    variant,
                    SplitButtonSize::Small,
                    true,
                );
            }));
            gap(Dp(24.0));
        }
    });
    gap(Dp(16.0));
    subheading("Size".into());
    prose(
        "Split buttons use the same five sizes as buttons and icon buttons. Small is the default.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                demo(
                    menu,
                    "XS",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::ExtraSmall,
                    true,
                );
                demo(
                    menu,
                    "S",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Small,
                    true,
                );
                demo(
                    menu,
                    "M",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Medium,
                    true,
                );
                demo(
                    menu,
                    "L",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Large,
                    true,
                );
                demo(
                    menu,
                    "XL",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::ExtraLarge,
                    true,
                );
            });
    }));
    gap(Dp(24.0));
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
                "Color".into(),
                "Elevated, filled, tonal, outlined".into(),
                "--".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "Split button token sets are organized by size. Each size defines the container height, the inner and outer corner, the content padding and the trailing icon size.",
    );
    gap(Dp(24.0));
    column().children(|| {
        for size in SIZES {
            size_token_table(size);
        }
    });
    gap(Dp(32.0));
    spec_table(
        vec![
            "Variant".into(),
            "Container".into(),
            "Label and icon".into(),
        ],
        vec![
            vec!["Filled".into(), "Primary".into(), "On primary".into()],
            vec![
                "Tonal".into(),
                "Secondary container".into(),
                "On secondary container".into(),
            ],
            vec![
                "Elevated".into(),
                "Surface container low".into(),
                "Primary".into(),
            ],
            vec![
                "Outlined".into(),
                "Transparent".into(),
                "On surface variant".into(),
            ],
        ],
    );
}

#[tessera]
fn size_token_table(size: SplitButtonSize) {
    let leading = SplitButtonDefaults::leading_content_padding(size);
    let trailing = SplitButtonDefaults::trailing_content_padding(size);
    subheading(size_name(size).to_owned());
    gap(Dp(12.0));
    spec_table(
        vec!["Size attribute".into(), "Value".into()],
        vec![
            vec![
                "Container height".into(),
                format!("{}dp", SplitButtonDefaults::container_height(size).0),
            ],
            vec![
                "Inner corner".into(),
                format!("{}dp", SplitButtonDefaults::inner_corner_size(size).0),
            ],
            vec!["Outer corner".into(), "50% (full capsule)".into()],
            vec![
                "Leading padding".into(),
                format!("{}/{}dp", leading.left.0, leading.right.0),
            ],
            vec![
                "Trailing padding".into(),
                format!("{}/{}dp", trailing.left.0, trailing.right.0),
            ],
            vec![
                "Trailing icon".into(),
                format!("{}dp", SplitButtonDefaults::trailing_icon_size(size).0),
            ],
        ],
    );
    gap(Dp(16.0));
}

#[tessera]
fn anatomy(menu: Menu) {
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Anatomy",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Leading button");
        legend(2, "Icon");
        legend(3, "Label text");
        legend(4, "Trailing button");
    });
    gap(Dp(16.0));
    caption(
        "The leading button holds the main action; the trailing button opens its related actions",
    );
}

#[tessera]
fn colors(menu: Menu) {
    prose(
        "Split buttons use the same color schemes as standard buttons. Unlike toggle buttons, the split button color doesn’t change when selected — only a state layer is applied.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                for variant in VARIANTS {
                    demo(
                        menu,
                        format!("Light {}", variant_name(variant)),
                        variant,
                        SplitButtonSize::Small,
                        true,
                    );
                }
            });
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(move || {
            panel(RenderSlot::new(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        for variant in VARIANTS {
                            demo(
                                menu,
                                format!("Dark {}", variant_name(variant)),
                                variant,
                                SplitButtonSize::Small,
                                true,
                            );
                        }
                    });
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec![
            "Variant".into(),
            "Container color".into(),
            "Label color".into(),
        ],
        vec![
            vec!["Filled".into(), "Primary".into(), "On primary".into()],
            vec![
                "Tonal".into(),
                "Secondary container".into(),
                "On secondary container".into(),
            ],
            vec![
                "Elevated".into(),
                "Surface container low".into(),
                "Primary".into(),
            ],
            vec![
                "Outlined".into(),
                "Transparent".into(),
                "On surface variant".into(),
            ],
        ],
    );
}

#[tessera]
fn states(menu: Menu) {
    let enabled = remember(|| true);
    prose(
        "States are visual representations used to communicate the status of a component or interactive element.",
    );
    gap(Dp(16.0));
    prose(
        "Split button states use the same colors and state layers as buttons and icon buttons. The inner corners change shape for hovered, focused and pressed states, and the menu icon spins when the menu is expanded.",
    );
    gap(Dp(24.0));
    column().children(move || {
        button()
            .tonal()
            .selected(!enabled.get())
            .accessibility_label("Disable split button")
            .on_click(move || {
                enabled.set(!enabled.get());
                if !enabled.get() {
                    menu.controller.with_mut(MenuController::close);
                }
            })
            .child(|| {
                text().content("Disabled");
            });
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            demo(
                menu,
                "States",
                SplitButtonVariant::Filled,
                SplitButtonSize::Medium,
                enabled.get(),
            );
        }));
    });
    gap(Dp(16.0));
    caption("Enabled, hovered, focused, pressed and disabled states keep the same color roles");
}

#[tessera]
fn measurements() {
    prose(
        "Text and icons are optically centered when the segments are asymmetrical. They’re centered normally when symmetrical.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Attribute".into(), "Value".into()],
        vec![
            vec![
                "Split button".into(),
                "Container height (default S)".into(),
                "40dp".into(),
            ],
            vec![
                "Each segment".into(),
                "Minimum target".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "Container".into(),
                "Outer corner".into(),
                "50% (full capsule)".into(),
            ],
            vec![
                "Container".into(),
                "Inner corner".into(),
                "4dp · 12dp active".into(),
            ],
            vec!["Outlined".into(), "Divider / outline".into(), "1dp".into()],
            vec!["Segments".into(), "Space between".into(), "2dp".into()],
            vec![
                "Trailing segment".into(),
                "Minimum width".into(),
                "48dp".into(),
            ],
            vec!["Trailing segment".into(), "Icon size".into(), "22dp".into()],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        measurements_figure();
    }));
    gap(Dp(16.0));
    caption("The trailing segment keeps a 48dp target even when the container is 40dp tall");
}

#[tessera]
fn measurements_figure() {
    let controller = remember(MenuController::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            split_button_layout()
                .leading_button(move || {
                    split_leading_button()
                        .size(SplitButtonSize::Medium)
                        .variant(SplitButtonVariant::Filled)
                        .accessibility_label("Publish")
                        .on_click(|| {})
                        .content(|| {
                            icon().painter(filled::PUBLISH_SVG);
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            text().content("Publish");
                        });
                })
                .trailing_button(move || {
                    split_trailing_button()
                        .size(SplitButtonSize::Medium)
                        .variant(SplitButtonVariant::Filled)
                        .expanded(controller.with(MenuController::is_open))
                        .accessibility_label("Publish options")
                        .on_click(move || controller.with_mut(MenuController::toggle))
                        .content(|| {
                            icon().painter(filled::ARROW_DROP_DOWN_SVG);
                        });
                });
            gap(Dp(12.0));
            caption("Medium split button: 56dp container, 24/24dp inner padding, 26dp menu icon");
        });
}

#[tessera]
fn usage(menu: Menu) {
    prose(
        "Split buttons are used to add a menu of actions alongside a main action. This reduces visual complexity by hiding extra options. Split buttons work well alone or alongside common buttons and icon buttons.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Usage",
            SplitButtonVariant::Filled,
            SplitButtonSize::Small,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Split buttons on their own can grab attention");
    gap(Dp(24.0));
    prose(
        "Split buttons have five recommended sizes. Scale up the split button in large breakpoints, or to create more emphasis in smaller windows.",
    );
    gap(Dp(24.0));
    do_dont(
        "Open a menu from a split button",
        "Avoid modifying the menu in unusual ways",
    );
    gap(Dp(32.0));
    prose(
        "The main action stays on the leading segment; the trailing segment only opens the menu of related actions, so people always know which part they are about to trigger.",
    );
    gap(Dp(16.0));
    do_dont(
        "Keep the leading segment for the primary action",
        "Don’t put a menu on the leading segment and an action on the trailing segment",
    );
    gap(Dp(32.0));
    prose(
        "Split buttons can be used alongside other buttons and button groups, and can be a different size from them, especially since they take up more space.",
    );
}

#[tessera]
fn guide_anatomy(menu: Menu) {
    prose("The split button has a leading button and a trailing button separated by a small gap.");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Anatomy",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Leading button");
        legend(2, "Icon");
        legend(3, "Label text");
        legend(4, "Trailing button");
    });
    gap(Dp(16.0));
    caption("The trailing button contains the menu icon; the leading button carries the label");
}

#[tessera]
fn guide_color(menu: Menu) {
    prose(
        "Use the split button color that matches the page emphasis: filled for the highest emphasis, tonal for medium emphasis, elevated when the button sits on imagery, and outlined on plain surfaces.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                for variant in VARIANTS {
                    demo(
                        menu,
                        variant_name(variant),
                        variant,
                        SplitButtonSize::Small,
                        true,
                    );
                }
            });
    }));
    gap(Dp(16.0));
    caption("Filled, tonal, elevated and outlined split buttons share the button color roles");
}

#[tessera]
fn adaptive_design(menu: Menu) {
    prose(
        "Split buttons scale with the layout. Keep the leading and trailing segments together, and let the pair wrap below other controls rather than splitting the two segments apart.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            demo(
                menu,
                "Small",
                SplitButtonVariant::Tonal,
                SplitButtonSize::Small,
                true,
            );
            gap(Dp(16.0));
            demo(
                menu,
                "Large",
                SplitButtonVariant::Tonal,
                SplitButtonSize::Large,
                true,
            );
        });
    }));
    gap(Dp(16.0));
    caption("Small and large split buttons keep the same leading/trailing structure");
    gap(Dp(24.0));
    prose(
        "Each segment keeps a minimum 48 × 48dp target, so a 32dp or 40dp container still exposes a full-size target for both the action and the menu.",
    );
}

#[tessera]
fn behavior(menu: Menu) {
    prose(
        "Activating the leading segment runs the main action immediately and leaves the menu untouched. Activating the trailing segment opens the menu anchored to the trailing edge, and the menu icon spins to indicate the expanded state.",
    );
    gap(Dp(16.0));
    prose(
        "Choosing a menu item closes the menu and reports the choice here. Pressing Escape, activating the trailing segment again, or clicking outside the menu also closes it.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "Behavior",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("Run the main action on the left, then open the menu to choose a related action");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to each button and interact with them".into(),
        "Navigate to any element opened by the trailing button".into(),
        "Understand the current selection state of the button".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "Each button in the split button needs a minimum target area of 48 × 48dp. Extra small and small split buttons are shorter than 48dp, so the target areas around them need to be at least 48dp tall.",
    );
    gap(Dp(16.0));
    caption("Target areas should be at least 48 × 48dp");
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "Extra small");
        legend(2, "Small");
    });
}

#[tessera]
fn initial_focus() {
    prose(
        "Focus should land on the leading button, then move to the trailing button. This can depend on the operating system’s settings.",
    );
    gap(Dp(16.0));
    prose(
        "In right-to-left layouts the order is mirrored, so focus starts on the leading segment at the right edge.",
    );
    gap(Dp(24.0));
    caption("Left to right: leading then trailing. Right to left: mirrored");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Navigate between the two buttons".into()],
            vec![
                "Space or Enter".into(),
                "Activate the focused button".into(),
            ],
            vec![
                "Tab in the menu".into(),
                "Move between the open menu items".into(),
            ],
            vec!["Escape".into(), "Close the menu".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The native sample follows this table: Tab reaches the leading segment and then the trailing segment, Space or Enter activates whichever segment has focus, and Escape dismisses the open menu.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "The accessibility label for the leading button is the same as buttons: it should describe the main action people trigger.",
    );
    gap(Dp(16.0));
    prose(
        "The trailing icon button should have an extra state or similar label indicating that the menu is expanded or collapsed.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Element".into(),
            "Label".into(),
            "Role".into(),
            "State".into(),
        ],
        vec![
            vec![
                "Leading button".into(),
                "Match the main action".into(),
                "Button".into(),
                "—".into(),
            ],
            vec![
                "Trailing button".into(),
                "More {main action} options".into(),
                "Button".into(),
                "Expanded or collapsed".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption("Leading buttons should have the same labels as common buttons");
}
