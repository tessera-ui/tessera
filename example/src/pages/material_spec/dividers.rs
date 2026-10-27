//! Interactive divider specifications.
//!
//! ## Usage
//!
//! Compare full-width, inset, and vertical dividers in grouped content.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    divider::{horizontal_divider, vertical_divider},
    list_item::list_item,
    modifier::{ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use super::{
    ACCENT,
    annotations::{self, SpaceGeometry, dotted, space_geometry, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, srgb, subheading, type_style,
};

#[tessera]
pub(super) fn divider_spec() {
    article(
        "Divider",
        "Dividers are thin lines that group content in lists or other containers.",
        vec![
            SpecSection {
                title: "Tokens and specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .intro(divider_anatomy)
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
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(|| {
                mail_groups(true);
                caption("Full-width and inset dividers communicate different levels of grouping.");
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(divider_anatomy),
                },
                SpecSection {
                    title: "Full-width dividers",
                    content: RenderSlot::new(full_width_guidelines),
                },
                SpecSection {
                    title: "Inset dividers",
                    content: RenderSlot::new(inset_guidelines),
                },
                SpecSection {
                    title: "Vertical divider",
                    content: RenderSlot::new(|| {
                        prose("Use a vertical divider to arrange content on a larger screen, such as separating paragraph text from related video or imagery.");
                        gap(Dp(24.0));
                        vertical_example();
                        caption("A vertical divider distinguishes media from its description without breaking their shared context.");
                    }),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: Some(RenderSlot::new(|| {
                prose("Dividers are decorative elements, which have no contrast minimums.");
                gap(Dp(24.0));
                decorative_example();
                caption("Decorative elements have no contrast minimums. Text and interactive controls still need to meet their own accessibility requirements.");
                gap(Dp(32.0));
                prose("In this native example, the divider has no label, action, or focus stop. Use Tab to move between the two buttons, then Enter or Space to activate one. The status text identifies the activated action; the line only provides visual grouping.");
            })),
            sections: vec![],
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
fn overview() {
    prose("Make dividers visible but not bold.");
    gap(Dp(16.0));
    prose("Only use dividers if items cannot be grouped with open space.");
    gap(Dp(16.0));
    prose("Use dividers to group things, not separate individual items.");
    gap(Dp(32.0));
    mail_groups(true);
    caption(
        "Dividers group related messages in a list, while the full-width line marks a different section.",
    );
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
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use native horizontal_divider and vertical_divider components. Insets are supplied by the surrounding layout.",
    );
}

#[tessera]
fn differences() {
    prose("Color: Updated color mappings support dynamic color.");
    gap(Dp(16.0));
    prose("Configurations: Dividers can also be vertical.");
    gap(Dp(24.0));
    colors();
    gap(Dp(32.0));
    vertical_example();
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Divider color".into(),
                "md.comp.divider.color".into(),
                "Outline variant".into(),
            ],
            vec![
                "Divider thickness".into(),
                "md.comp.divider.thickness".into(),
                "1dp".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The divider inherits the outline variant color role from the active Material theme. Its length follows the available width, or the available height for a vertical divider.",
    );
}

#[tessera]
fn scheme(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            surface()
                .style(scheme.surface.into())
                .shape(Shape::rounded_rectangle(Dp(24.0)))
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(32.0)))
                        .children(move || {
                            text()
                                .content(if dark { "Dark theme" } else { "Light theme" })
                                .color(scheme.on_surface)
                                .style(type_style(20.0, 28.0));
                            gap(Dp(32.0));
                            text()
                                .content("1  Outline variant")
                                .color(scheme.on_surface_variant)
                                .style(type_style(14.0, 20.0));
                            gap(Dp(24.0));
                            horizontal_divider();
                            gap(Dp(32.0));
                        });
                });
        });
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. The same outline variant role maps to different color values in light and dark schemes.",
    );
    gap(Dp(24.0));
    scheme(false);
    gap(Dp(8.0));
    scheme(true);
    caption("1  Outline variant — divider color in both light and dark themes.");
}

#[tessera]
fn divider_anatomy() {
    let geometry = remember(SpaceGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch_in_space(geometry, "figure")
                .fill_max_width()
                .height(Dp(224.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(48.0), Dp::ZERO),
                )
                .children(move || {
                    horizontal_divider().modifier(watch_in_space(geometry, "line"));
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    divider_callout(geometry);
                });
        });
    caption("1  Divider — a single thin line, with no container, label, or interactive state.");
}

#[tessera]
fn divider_callout(geometry: State<SpaceGeometry>) {
    let values = space_geometry(&geometry.get());
    let (Some(root), Some(line)) = (values.get("figure"), values.get("line")) else {
        return;
    };
    let mut svg = String::new();
    dotted(
        &mut svg,
        line.x + line.w / 2.0,
        line.y - 48.0,
        48.0,
        true,
        "#49454f",
    );
    annotations::lines(Dp(root.w), Dp(root.h), svg);
    annotations::marker(1, Dp(line.x + line.w / 2.0 - 12.0), Dp(line.y - 80.0));
}

#[tessera]
fn measurements() {
    prose(
        "The divider is 1dp thick. Full-width, inset, and middle-inset configurations change its length, not its thickness.",
    );
    gap(Dp(24.0));
    measurement_figure("Full width", Dp::ZERO, Dp::ZERO);
    gap(Dp(16.0));
    measurement_figure("Inset", Dp(16.0), Dp::ZERO);
    gap(Dp(16.0));
    measurement_figure("Middle inset", Dp(16.0), Dp(16.0));
    gap(Dp(32.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Divider full-width".into(), "100%".into()],
            vec!["Divider inset left margin".into(), "16dp".into()],
            vec!["Divider inset right margin".into(), "0dp".into()],
            vec!["Divider middle-inset left margin".into(), "16dp".into()],
            vec!["Divider middle-inset right margin".into(), "16dp".into()],
            vec![
                "Space between divider & supporting text".into(),
                "4dp".into(),
            ],
            vec!["Divider right margin".into(), "8dp".into()],
            vec!["Divider bottom margin".into(), "8dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Supporting text and spacing".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text()
                    .content("Supporting text")
                    .style(type_style(14.0, 20.0));
                gap(Dp(4.0));
                boxed()
                    .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                        Dp::ZERO,
                        Dp::ZERO,
                        Dp(8.0),
                        Dp(8.0),
                    )))
                    .children(|| {
                        horizontal_divider();
                    });
                prose("4dp text gap · 8dp end margin · 8dp bottom margin");
            });
    }));
    caption(
        "Spacing belongs to the surrounding content layout; it is not added by the divider itself.",
    );
}

#[tessera]
fn measurement_figure(#[prop(into)] label: String, left: Dp, right: Dp) {
    let geometry = remember(SpaceGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            watch_in_space(geometry, "figure")
                .fill_max_width()
                .height(Dp(176.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            text()
                .content(label.clone())
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().offset(Dp(24.0), Dp(16.0)));
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(40.0), Dp::ZERO)
                        .offset(Dp::ZERO, Dp(88.0)),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            watch_in_space(geometry, "content")
                                .fill_max_width()
                                .padding(Padding::new(left, Dp::ZERO, right, Dp::ZERO)),
                        )
                        .children(move || {
                            horizontal_divider().modifier(watch_in_space(geometry, "line"));
                        });
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    measurement_callouts(geometry, left, right);
                });
        });
}

#[tessera]
fn measurement_callouts(geometry: State<SpaceGeometry>, left: Dp, right: Dp) {
    let values = space_geometry(&geometry.get());
    let (Some(root), Some(content), Some(line)) = (
        values.get("figure"),
        values.get("content"),
        values.get("line"),
    ) else {
        return;
    };
    let mut svg = String::new();
    let y = line.y + 24.0;
    let x1 = line.x;
    let x2 = line.x + line.w;
    svg.push_str(&format!("<path d='M{x1} {y}H{x2} M{x1} {}V{} M{x2} {}V{}' stroke='#b3261e' stroke-width='1' fill='none'/>", y - 4.0, y + 4.0, y - 4.0, y + 4.0));
    dotted(&mut svg, x1, line.y - 16.0, 32.0, true, "#49454f");
    dotted(&mut svg, x2, line.y - 16.0, 32.0, true, "#49454f");
    annotations::lines(Dp(root.w), Dp(root.h), svg);
    annotations::dimension("1dp".into(), Dp(content.x), Dp(line.y - 32.0));
    annotations::dimension(
        if left == Dp::ZERO && right == Dp::ZERO {
            "100% available width".into()
        } else {
            "Available width minus insets".into()
        },
        Dp(content.x + 24.0),
        Dp(y + 8.0),
    );
    if left != Dp::ZERO {
        annotations::dimension("16dp".into(), Dp(content.x), Dp(line.y - 24.0));
    }
    if right != Dp::ZERO {
        annotations::dimension(
            "16dp".into(),
            Dp(content.x + content.w - 36.0),
            Dp(line.y - 24.0),
        );
    }
}

#[tessera]
fn usage() {
    prose(
        "Dividers visually group components, create hierarchy, and can imply nested parent/child relationships. Use a full-width divider for a different kind of content, and inset dividers for nested content within a group.",
    );
    gap(Dp(24.0));
    divider_comparison();
    caption(
        "Try the native controls to compare the same content with full-width, leading-inset, middle-inset, or no divider lines.",
    );
}

#[tessera]
fn divider_comparison() {
    let mode = remember(|| 1_usize);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        for index in 0..4 {
                            divider_option(index, mode);
                        }
                    });
                gap(Dp(24.0));
                text()
                    .content(
                        [
                            "Full-width divider",
                            "Inset divider",
                            "Middle-inset divider",
                            "Open space",
                        ][mode.get()],
                    )
                    .style(type_style(20.0, 28.0));
                gap(Dp(16.0));
                for index in 0..3 {
                    mail_row(index);
                    if index < 2 && mode.get() != 3 {
                        let left = if mode.get() == 0 { Dp::ZERO } else { Dp(16.0) };
                        let right = if mode.get() == 2 { Dp(16.0) } else { Dp::ZERO };
                        inset_line(left, right);
                    }
                }
            });
    }));
}

#[tessera]
fn divider_option(index: usize, mode: State<usize>) {
    let label = ["Full width", "Inset", "Middle inset", "No lines"][index];
    button()
        .text()
        .modifier(Modifier::new().weight(1.0))
        .on_click(move || mode.set(index))
        .child(move || {
            text()
                .content(if mode.get() == index {
                    format!("{label} · selected")
                } else {
                    label.to_owned()
                })
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn inset_line(left: Dp, right: Dp) {
    boxed()
        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
            left,
            Dp::ZERO,
            right,
            Dp::ZERO,
        )))
        .children(|| {
            horizontal_divider();
        });
}

#[tessera]
fn full_width_guidelines() {
    prose(
        "Full-width dividers separate content into sections, including content within cards and lists. They can also distinguish interactive areas from non-interactive areas and indicate how elements relate from an interaction perspective.",
    );
    gap(Dp(24.0));
    full_width_card();
    guidance(
        false,
        "Do: Use a full-width line to separate the card's information from its action.",
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                text().content("Your order").style(type_style(24.0, 32.0));
                gap(Dp(16.0));
                horizontal_divider();
                list_item()
                    .headline("Succulents")
                    .supporting_text("Nursery");
                horizontal_divider();
                list_item()
                    .headline("Watering can")
                    .supporting_text("Medium, brass finish");
                horizontal_divider();
                list_item()
                    .headline("Subtotal")
                    .supporting_text("Including tax");
                horizontal_divider();
            });
    }));
    guidance(
        true,
        "Caution: Use full-width dividers sparingly. Too many lines make an interface look cluttered; spacing may already communicate the groups.",
    );
}

#[tessera]
fn guidance(caution: bool, #[prop(into)] value: String) {
    gap(Dp(16.0));
    text()
        .content(value)
        .color(if caution {
            srgb(108, 79, 0)
        } else {
            srgb(31, 97, 52)
        })
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn inset_guidelines() {
    prose(
        "Inset dividers separate related content, such as messages in a list. Use anchoring elements such as icons or avatars, with consistent alignment to the leading edge of the screen.",
    );
    gap(Dp(24.0));
    mail_groups(true);
    caption(
        "Inset dividers in a list of related messages. The avatars provide a stable visual anchor.",
    );
    gap(Dp(32.0));
    prose(
        "A middle-inset divider can separate body text from related selection controls within the same layout.",
    );
    gap(Dp(24.0));
    middle_inset_content();
    caption(
        "The inset line divides descriptive content from related selections, without separating them into unrelated sections.",
    );
    gap(Dp(40.0));
    subheading("Using dividers both ways on the same screen".into());
    gap(Dp(16.0));
    prose(
        "1  Use a full-width divider for a different kind of content.\n2  Use inset dividers for nested content items.",
    );
    gap(Dp(24.0));
    mail_groups(true);
    guidance(
        false,
        "Do: Combine inset and full-width dividers to reflect the hierarchy of information.",
    );
    gap(Dp(32.0));
    prose(
        "List items with repetitive formats may not require an inset divider. Using only the margin between items is acceptable.",
    );
    gap(Dp(24.0));
    mail_groups(false);
    guidance(
        false,
        "Do: Omit divider lines where repeated layouts and spacing already make the relationship clear.",
    );
}

#[tessera]
fn mail_row(index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    list_item()
        .headline(["Alex", "Sam", "Taylor"][index])
        .supporting_text(["Project update", "Weekend plans", "Dinner invitation"][index])
        .leading(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(scheme.secondary_container, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content(["A", "S", "T"][index])
                        .color(scheme.on_secondary_container);
                });
        })
        .trailing(|| {
            text().content("10:30").style(type_style(12.0, 16.0));
        });
}

#[tessera]
fn mail_groups(inset: bool) {
    panel(RenderSlot::new(move || {
        surface()
            .modifier(Modifier::new().fill_max_width())
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .child(move || {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        text()
                            .content("Inbox · Today")
                            .style(type_style(24.0, 32.0))
                            .modifier(Modifier::new().padding_all(Dp(16.0)));
                        for index in 0..3 {
                            mail_row(index);
                            if inset && index < 2 {
                                inset_line(Dp(16.0), Dp::ZERO);
                            }
                        }
                        gap(Dp(16.0));
                        horizontal_divider();
                        gap(Dp(16.0));
                        text()
                            .content("Earlier messages")
                            .style(type_style(20.0, 28.0))
                            .modifier(Modifier::new().padding_all(Dp(16.0)));
                        mail_row(0);
                    });
            });
    }));
}

#[tessera]
fn full_width_card() {
    let saved = remember(|| false);
    panel(RenderSlot::new(move || {
        surface()
            .modifier(Modifier::new().fill_max_width())
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .child(move || {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(24.0)))
                            .children(|| {
                                text()
                                    .content("Account preferences")
                                    .style(type_style(24.0, 32.0));
                                gap(Dp(16.0));
                                prose("Manage your profile and notification preferences.");
                            });
                        horizontal_divider();
                        boxed()
                            .modifier(Modifier::new().padding_all(Dp(8.0)))
                            .children(move || {
                                button()
                                    .text()
                                    .on_click(move || saved.set(!saved.get()))
                                    .child(move || {
                                        text().content(if saved.get() {
                                            "Saved · undo"
                                        } else {
                                            "Save changes"
                                        });
                                    });
                            });
                    });
            });
    }));
}

#[tessera]
fn middle_inset_content() {
    let selected = remember(|| false);
    panel(RenderSlot::new(move || {
        surface()
            .modifier(Modifier::new().fill_max_width())
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .child(move || {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(|| {
                                text().content("A little greenery").style(type_style(24.0, 32.0));
                                gap(Dp(16.0));
                                prose("Choose a small desert plant in a locally crafted ceramic pot. Both options share the same care instructions.");
                            });
                        inset_line(Dp(16.0), Dp(16.0));
                        gap(Dp(16.0));
                        text().content("Select type").style(type_style(20.0, 28.0)).modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO));
                        row().children(move || {
                            button().text().on_click(move || selected.set(false)).child(move || { text().content(if selected.get() { "Succulent" } else { "Succulent · selected" }); });
                            button().text().on_click(move || selected.set(true)).child(move || { text().content(if selected.get() { "Cactus · selected" } else { "Cactus" }); });
                        });
                    });
            });
    }));
}

#[tessera]
fn vertical_example() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let playing = remember(|| false);
    panel(RenderSlot::new(move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .height(Dp(200.0))
                            .background_with_shape(
                                scheme.tertiary_container,
                                Shape::rounded_rectangle(Dp(16.0)),
                            ),
                    )
                    .children(move || {
                        button()
                            .text()
                            .on_click(move || playing.set(!playing.get()))
                            .child(move || {
                                text().content(if playing.get() {
                                    "Pause preview"
                                } else {
                                    "Play preview"
                                });
                            });
                    });
                boxed()
                    .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO))
                    .children(|| {
                        vertical_divider().modifier(Modifier::new().height(Dp(200.0)));
                    });
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        text()
                            .content("Explore the collection")
                            .style(type_style(24.0, 32.0));
                        gap(Dp(16.0));
                        prose("Stories, performances, and people behind the music.");
                        gap(Dp(16.0));
                        prose(if playing.get() {
                            "Preview playing"
                        } else {
                            "Preview paused"
                        });
                    });
            });
    }));
}

#[tessera]
fn decorative_example() {
    let action = remember(|| "No action activated");
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                text()
                    .content("Message preferences")
                    .style(type_style(24.0, 32.0));
                gap(Dp(16.0));
                prose("The line is decorative. The content and controls carry their own meaning.");
                gap(Dp(16.0));
                button()
                    .text()
                    .on_click(move || action.set("Profile opened"))
                    .child(|| {
                        text().content("Open profile");
                    });
                gap(Dp(16.0));
                horizontal_divider();
                gap(Dp(16.0));
                button()
                    .text()
                    .on_click(move || action.set("Notifications opened"))
                    .child(|| {
                        text().content("Open notifications");
                    });
                gap(Dp(16.0));
                prose(action.get());
            });
    }));
}
