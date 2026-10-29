//! Interactive chip specifications by purpose.
//!
//! ## Usage
//!
//! Compare compact actions, independent filters, input tokens and suggestions.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    chip::{ChipShapeStyle, ChipStyle, ChipVariant, chip},
    column::column,
    flow_row::flow_row,
    material_icons::filled,
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _, Padding},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, provide_context, remember, tessera, use_context,
};

use super::{
    ACCENT, INK,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const VARIANTS: [ChipVariant; 4] = [
    ChipVariant::Assist,
    ChipVariant::Filter,
    ChipVariant::Input,
    ChipVariant::Suggestion,
];

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

fn name(variant: ChipVariant) -> &'static str {
    match variant {
        ChipVariant::Assist => "Assist",
        ChipVariant::Filter => "Filter",
        ChipVariant::Input => "Input",
        ChipVariant::Suggestion => "Suggestion",
    }
}

fn token_prefix(variant: ChipVariant) -> &'static str {
    match variant {
        ChipVariant::Assist => "md.comp.assist-chip",
        ChipVariant::Filter => "md.comp.filter-chip",
        ChipVariant::Input => "md.comp.input-chip",
        ChipVariant::Suggestion => "md.comp.suggestion-chip",
    }
}

fn selectable(variant: ChipVariant) -> bool {
    matches!(variant, ChipVariant::Filter | ChipVariant::Input)
}

#[tessera]
pub(super) fn chips_spec() {
    article(
        "Chips",
        "Chips help people enter information, make selections, filter content, or trigger actions",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Assist chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Assist);
                }),
            },
            SpecSection {
                title: "Filter chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Filter);
                }),
            },
            SpecSection {
                title: "Input chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Input);
                }),
            },
            SpecSection {
                title: "Suggestion chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Suggestion);
                }),
            },
        ],
    )
    .hero(|| {
        panel(RenderSlot::new(|| {
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    for variant in VARIANTS {
                        sample(
                            variant,
                            ChipStyle::Flat,
                            false,
                            true,
                            true,
                            format!("{} example", name(variant)),
                        );
                    }
                });
        }));
    })
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
                    title: "Updates",
                    content: RenderSlot::new(updates),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(guidelines_intro)),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(chip_usage),
                },
                SpecSection {
                    title: "Variants",
                    content: RenderSlot::new(variants_guideline),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(chip_guideline_anatomy),
                },
                SpecSection {
                    title: "Assist chips",
                    content: RenderSlot::new(assist_guidelines),
                },
                SpecSection {
                    title: "Filter chips",
                    content: RenderSlot::new(filter_guidelines),
                },
                SpecSection {
                    title: "Input chips",
                    content: RenderSlot::new(input_guidelines),
                },
                SpecSection {
                    title: "Suggestion chips",
                    content: RenderSlot::new(suggestion_guidelines),
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
fn overview() {
    bullets(vec![
        "Use chips to show options for a specific context".into(),
        "Four variants: assist, filter, input, and suggestion".into(),
        "Chip elevation defaults to 0 but can be elevated if they need more visual separation"
            .into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for variant in VARIANTS {
                            sample(
                                variant,
                                ChipStyle::Flat,
                                variant == ChipVariant::Filter,
                                true,
                                variant != ChipVariant::Suggestion,
                                format!("{} chip", name(variant)),
                            );
                        }
                    });
            });
    }));
    gap(Dp(16.0));
    column().children(|| {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(index + 1, format!("{} chip", name(variant)));
        }
    });
    gap(Dp(12.0));
    caption("There are four chip variants: assist, filter, input, and suggestion");
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
        "Tessera examples on this page use the native chip component with the flat and elevated styles, standard shape corners, and an independent remove action for input chips.",
    );
}

#[tessera]
fn updates() {
    subheading("Aug 2024".into());
    prose("Updated stroke color from outline to outline variant.");
    gap(Dp(24.0));
    panel(RenderSlot::new(search_stroke));
    gap(Dp(16.0));
    caption("The stroke color was softened to improve visual hierarchy between chips and buttons");
}

#[tessera]
fn search_stroke() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            boxed()
                .modifier(Modifier::new().padding_all(Dp(8.0)))
                .children(|| {
                    chip()
                        .input("Input")
                        .shape_style(ChipShapeStyle::Standard)
                        .on_remove(|| {});
                });
            text()
                .content("→")
                .color(scheme.on_surface_variant)
                .style(type_style(16.0, 24.0));
            boxed()
                .modifier(Modifier::new().padding_all(Dp(8.0)))
                .children(|| {
                    chip()
                        .input("Input")
                        .shape_style(ChipShapeStyle::Standard)
                        .on_remove(|| {});
                });
        });
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color".into(),
        "Shape: Rounded rectangle".into(),
        "Variants: Action chips have been separated into assist chips and suggestion chips. Choice chips are now a subset of filter chips".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column().children(|| {
            flow_row().item_spacing(Dp(8.0)).children(|| {
                chip()
                    .input("Input")
                    .shape_style(ChipShapeStyle::Standard)
                    .on_remove(|| {});
                chip()
                    .filter("Choice")
                    .shape_style(ChipShapeStyle::Standard);
                chip()
                    .filter("Filter")
                    .shape_style(ChipShapeStyle::Standard);
                chip()
                    .assist("Action")
                    .shape_style(ChipShapeStyle::Standard);
            });
        });
    }));
    gap(Dp(12.0));
    caption("M2: Variants of chips are input, choice, filter, and action");
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row().item_spacing(Dp(8.0)).children(|| {
            chip()
                .assist("Assist")
                .shape_style(ChipShapeStyle::Standard)
                .leading_icon(filled::ADD_SVG);
            chip()
                .filter("Filter")
                .shape_style(ChipShapeStyle::Standard)
                .selected(true)
                .leading_icon(filled::CHECK_SVG);
            chip()
                .input("Input")
                .shape_style(ChipShapeStyle::Standard)
                .on_remove(|| {});
            chip()
                .suggestion("Suggestion")
                .shape_style(ChipShapeStyle::Standard);
        });
    }));
    gap(Dp(12.0));
    caption("M3: Variants of chips updated to assist, filter, input, and suggestion");
}

#[tessera]
fn guidelines_intro() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                chip()
                    .assist("Assist")
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::EVENT_SVG);
                chip()
                    .filter("Filter")
                    .shape_style(ChipShapeStyle::Standard)
                    .selected(true)
                    .leading_icon(filled::CHECK_SVG);
                chip()
                    .input("Input")
                    .shape_style(ChipShapeStyle::Standard)
                    .on_remove(|| {});
                chip()
                    .suggestion("Suggestion")
                    .shape_style(ChipShapeStyle::Standard);
            });
    }));
    gap(Dp(16.0));
    caption("There are four variants of chips: assist, filter, input, and suggestion");
}

#[tessera]
fn chip_usage() {
    prose(
        "Chips help people enter information, make selections, filter content, or trigger actions. They're best used to help people accomplish their current task faster and easier.",
    );
    gap(Dp(24.0));
    subheading("Chips aren’t buttons".into());
    prose(
        "Chips and buttons are similar. They both provide visual cues to prompt people to take actions and make selections. While buttons appear consistently and with familiar calls of action, chips are dynamic to the situation, and appear as a group of interactive elements.",
    );
    gap(Dp(16.0));
    prose(
        "Use chips to enhance a person’s current journey and encourage action. Use buttons to progress them through the product and for significant actions.",
    );
    gap(Dp(24.0));
    do_dont(
        "Use chips to present contextual, supplemental options",
        "Avoid replacing major actions with chips. Actions that progress people to the next or previous step should always be displayed as buttons.",
    );
    gap(Dp(32.0));
    prose(
        "Chips should dynamically offer various actions depending on the current task, whereas a button should be a persistent fixture of a layout.",
    );
    gap(Dp(16.0));
    prose(
        "Chips represent forking paths for a current task, while buttons represent linear steps.",
    );
    gap(Dp(24.0));
    assist_example();
    gap(Dp(24.0));
    do_dont(
        "Use buttons for the final step in a task",
        "Avoid using chips to finish or progress a task",
    );
    gap(Dp(32.0));
    prose(
        "Multiple chips should appear together in a set, whereas there should be no more than 3 buttons in a single arrangement.",
    );
    gap(Dp(16.0));
    prose("Chip sets can be scrolled horizontally.");
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(24.0));
    do_dont(
        "Chips can be scrolled horizontally",
        "Don’t display a single chip by itself. Chips should appear in a set.",
    );
}

#[tessera]
fn variants_guideline() {
    prose("Choose the chip variant based on its purpose and author.");
    gap(Dp(16.0));
    prose("Does the chip represent an action (assist chip) or filter results (filter chip)?");
    gap(Dp(16.0));
    prose(
        "Is its content generated by the product (suggestion chip), or by the person using the product (input chip)?",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        chip()
                            .assist("Assist")
                            .shape_style(ChipShapeStyle::Standard)
                            .leading_icon(filled::EVENT_SVG);
                        chip()
                            .filter("Filter")
                            .shape_style(ChipShapeStyle::Standard)
                            .leading_icon(filled::CHECK_SVG);
                        chip()
                            .input("Input")
                            .shape_style(ChipShapeStyle::Standard)
                            .on_remove(|| {});
                        chip()
                            .suggestion("Suggestion")
                            .shape_style(ChipShapeStyle::Standard);
                    });
            });
    }));
    gap(Dp(16.0));
    column().children(|| {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(index + 1, name(variant));
        }
    });
    gap(Dp(32.0));
    spec_table(
        vec![
            "Purpose".into(),
            "Chip variant".into(),
            "Rationale".into(),
            "Example".into(),
        ],
        vec![
            vec![
                "Action".into(),
                "Assist chip".into(),
                "Assist chips represent smart or automated actions that can span multiple apps"
                    .into(),
                "Add to calendar action".into(),
            ],
            vec![
                "Filter".into(),
                "Filter chip".into(),
                "Filter chips represent filters for a collection".into(),
                "Platform selector on material.io/components".into(),
            ],
            vec![
                "Information, user-authored".into(),
                "Input chip".into(),
                "Input chips represent discrete pieces of information entered by a person".into(),
                "Gmail contact in the To field".into(),
            ],
            vec![
                "Information, product-authored".into(),
                "Suggestion chip".into(),
                "Suggestion chips help narrow a person’s intent by presenting dynamically-generated suggestions".into(),
                "Suggested chat response".into(),
            ],
        ],
    );
}

#[tessera]
fn chip_guideline_anatomy() {
    chip_diagram(ChipVariant::Input, false);
    gap(Dp(16.0));
    caption("A chip contains a container, a label, and optional leading and trailing elements");
    gap(Dp(32.0));
    subheading("Container".into());
    prose("All chips are slightly rounded with an 8dp corner.");
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Filter, true);
    gap(Dp(16.0));
    caption("Chips have rounded corners");
    gap(Dp(32.0));
    subheading("Shadows & elevation".into());
    prose("Chip containers can be elevated if placed on top of an image or dynamic background.");
    gap(Dp(24.0));
    elevation_example();
    gap(Dp(16.0));
    caption("When on complicated backgrounds, chip containers can be elevated");
    gap(Dp(16.0));
    do_dont(
        "Use an outline to define the edge of the chip's container on regular backgrounds",
        "Chips shouldn't be elevated when placed directly on the page",
    );
    gap(Dp(16.0));
    do_dont(
        "Chips may use elevation when placed on an image",
        "Avoid using elevation to indicate a chip's pressed state. Instead, use the visual ripple effect.",
    );
    gap(Dp(32.0));
    subheading("Label text".into());
    prose(
        "Chip label text should be 20 characters or fewer, and have the same typography style as buttons.",
    );
    gap(Dp(16.0));
    prose(
        "Chip labels should remain brief for the limited space available. Skip conventional grammar rules, such as articles (take “a” walk), to save space.",
    );
    gap(Dp(24.0));
    label_length_example();
    gap(Dp(16.0));
    do_dont(
        "Keep chip labels short",
        "Avoid chip labels longer than 20 characters",
    );
    gap(Dp(32.0));
    subheading("Leading icon or image (optional)".into());
    prose(
        "Chips can contain a leading icon, logo, or circular image. Use a system icon to help identify a chip’s category.",
    );
    gap(Dp(24.0));
    leading_examples();
    gap(Dp(16.0));
    caption("Chips can contain a logo, icon, or circular image");
    gap(Dp(24.0));
    prose(
        "The leading icon color for unselected chips can be customized through theming. While the default color role is primary, the on surface variant color role is a good alternative when the icon style requires less emphasis.",
    );
    gap(Dp(16.0));
    prose(
        "Leading circular images are sized larger than leading icons to provide more space for detail. Icons are designed to be legible at small sizes. See the Specs tab for precise measurement values.",
    );
    gap(Dp(16.0));
    caption("Leading images, such as avatars, are sized larger than leading icons or logos");
    gap(Dp(32.0));
    subheading("Trailing icon (optional, input and filter chips only)".into());
    prose("The trailing icon is present for input and filter chips.");
    gap(Dp(16.0));
    prose(
        "On input chips, it’s required and must be used to remove the chip. On filter chips, it’s optional, and can be used to open a menu or remove the chip.",
    );
    gap(Dp(16.0));
    prose(
        "Secondary actions (such as a trailing icon button for Remove) must have a 48x48dp interaction target that doesn’t interfere with the chip’s primary action (such as Edit or Drag). To achieve this, apply a minimum width of 88dp to the chip, or 42dp to the label text.",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, true);
    gap(Dp(16.0));
    caption(
        "Interaction targets for actions like Edit or Close have a size of 48x48. This can be achieved by setting the minimum container width to 88dp",
    );
}

#[tessera]
fn elevation_example() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_all(Dp(20.0))
                    .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0))),
            )
            .children(|| {
                flow_row().item_spacing(Dp(8.0)).children(|| {
                    chip()
                        .assist("Assist")
                        .shape_style(ChipShapeStyle::Standard)
                        .leading_icon(filled::EVENT_SVG);
                    chip()
                        .filter("Filter")
                        .shape_style(ChipShapeStyle::Standard);
                    chip()
                        .suggestion("Suggestion")
                        .shape_style(ChipShapeStyle::Standard);
                });
            });
        gap(Dp(16.0));
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_all(Dp(20.0))
                    .background_with_shape(
                        scheme.surface_container_highest,
                        Shape::rounded_rectangle(Dp(16.0)),
                    ),
            )
            .children(|| {
                flow_row().item_spacing(Dp(8.0)).children(|| {
                    chip()
                        .assist("Assist")
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard)
                        .leading_icon(filled::EVENT_SVG);
                    chip()
                        .filter("Filter")
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard);
                    chip()
                        .suggestion("Suggestion")
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard);
                });
            });
    });
    gap(Dp(12.0));
    caption(
        "Flat chips suit regular surfaces; elevated chips separate containers from imagery or dynamic color",
    );
}

#[tessera]
fn label_length_example() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                chip()
                    .assist("Take walk")
                    .shape_style(ChipShapeStyle::Standard);
                chip()
                    .assist("Take a long walk around the park")
                    .shape_style(ChipShapeStyle::Standard);
            });
    }));
}

#[tessera]
fn leading_examples() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                chip()
                    .input("Ping Qiang")
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::PERSON_SVG)
                    .on_remove(|| {});
                chip()
                    .filter("Type: Images")
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::IMAGE_SVG);
            });
    }));
}

#[tessera]
fn chip_diagram(variant: ChipVariant, measurements: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| false);
    let removed = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(280.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            provide_context(
                || MinimumInteractiveComponentEnforcement { enabled: false },
                move || {
                    if removed.get() {
                        button()
                            .outlined()
                            .on_click(move || removed.set(false))
                            .child(|| {
                                text().content("Restore input chip");
                            });
                    } else if variant == ChipVariant::Input {
                        chip()
                            .input(format!("{} chip", name(variant)))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()))
                            .leading_icon(filled::IMAGE_SVG)
                            .on_remove(move || removed.set(true));
                    } else if variant == ChipVariant::Filter {
                        chip()
                            .filter(format!("{} chip", name(variant)))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()))
                            .leading_icon(filled::ADD_SVG)
                            .trailing_icon(filled::ARROW_DROP_DOWN_SVG);
                    } else if variant == ChipVariant::Assist {
                        chip()
                            .assist(format!("{} chip", name(variant)))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()))
                            .leading_icon(filled::ADD_SVG);
                    } else {
                        chip()
                            .suggestion(format!("{} chip", name(variant)))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()));
                    }
                },
            );
            if !removed.get() {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        chip_callouts(geometry, variant, measurements);
                    });
            }
        });
}

#[tessera]
fn chip_callouts(geometry: State<Geometry>, variant: ChipVariant, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(chip)) = (values.get("figure"), relative(&values, "chip")) else {
        return;
    };
    let mut svg = String::new();
    let leading = Rect {
        x: chip.x + 8.0,
        y: chip.y + (chip.h - 18.0) / 2.0,
        w: 18.0,
        h: 18.0,
    };
    let trailing = Rect {
        x: chip.x + chip.w - 26.0,
        y: leading.y,
        w: 18.0,
        h: 18.0,
    };
    if measurements {
        let target = Rect {
            x: trailing.x - 15.0,
            y: chip.y + (chip.h - 48.0) / 2.0,
            w: 48.0,
            h: 48.0,
        };
        svg.push_str(&format!(
            "<rect x='{}' y='{}' width='48' height='48' fill='#ee7da4' fill-opacity='.25'/>",
            target.x, target.y
        ));
        annotations::outline(&mut svg, target);
        annotations::dimension("48".into(), Dp(target.x + 12.0), Dp(target.y - 32.0));
        annotations::dimension(
            "48".into(),
            Dp(target.x + target.w + 12.0),
            Dp(target.y + 14.0),
        );
        annotations::dimension("32".into(), Dp(chip.x - 40.0), Dp(chip.y + 6.0));
        annotations::dimension("88dp min".into(), Dp(chip.x), Dp(chip.y + chip.h + 40.0));
    } else {
        for (n, x, y, tx, ty) in [
            (1, chip.x + 16.0, chip.y - 64.0, chip.x + 28.0, chip.y),
            (
                2,
                chip.x + chip.w / 2.0 - 12.0,
                chip.y - 64.0,
                chip.x + chip.w / 2.0,
                chip.y + 16.0,
            ),
            (
                3,
                chip.x + chip.w + 32.0,
                chip.y,
                chip.x + chip.w - 17.0,
                chip.y + 16.0,
            ),
            (
                4,
                chip.x + 4.0,
                chip.y + chip.h + 24.0,
                leading.x + 9.0,
                leading.y + 18.0,
            ),
        ] {
            if n == 3 {
                dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
            } else {
                dotted(
                    &mut svg,
                    tx,
                    if n == 4 { ty } else { y + 32.0 },
                    if n == 4 { y - ty } else { ty - y - 32.0 },
                    true,
                    "#49454f",
                );
            }
            if variant != ChipVariant::Suggestion || n < 3 {
                annotations::marker(n, Dp(x), Dp(y));
            }
        }
        if variant != ChipVariant::Suggestion {
            annotations::outline(&mut svg, leading);
        }
        if matches!(variant, ChipVariant::Input | ChipVariant::Filter) {
            annotations::outline(&mut svg, trailing);
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn assist_example() {
    let saved = remember(|| false);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text()
                .content("Team catch-up")
                .style(type_style(24.0, 32.0));
            prose("Wednesday · 10:00–10:30");
            gap(Dp(16.0));
            flow_row().item_spacing(Dp(8.0)).children(move || {
                chip()
                    .assist(if saved.get() {
                        "Added"
                    } else {
                        "Add to calendar"
                    })
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::EVENT_SVG)
                    .on_click(move || saved.set(!saved.get()));
                chip()
                    .assist("Get directions")
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::DIRECTIONS_SVG)
                    .on_click(move || saved.set(true));
            });
        });
    }));
}

#[tessera]
fn assist_guidelines() {
    prose(
        "Assist chips represent smart or automated actions that can span multiple apps, such as opening a calendar event from the home screen. Assist chips function as though the person asked an assistant to complete the action. They should appear dynamically and contextually in a UI.",
    );
    gap(Dp(16.0));
    prose(
        "An alternative to assist chips are buttons, which should appear persistently and consistently.",
    );
    gap(Dp(24.0));
    assist_example();
    gap(Dp(16.0));
    caption("The text in most assist chips begins with a short verb, like Get or Add");
    gap(Dp(24.0));
    prose(
        "Assist chips can trigger an action or show progress and confirmation. Write assist chips like buttons: start with a verb. Adjust text dynamically if the state changes, like Save to Saved.",
    );
    gap(Dp(16.0));
    caption(
        "Tapping an assist chip triggers a contextual action, and the chip can show progress and confirmation feedback",
    );
    gap(Dp(24.0));
    prose(
        "Assist chips are displayed after primary content, such as below a card or persistently at the bottom of a screen.",
    );
}

#[tessera]
fn filter_example() {
    let selection = remember(|| [false; 3]);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text()
                .content("Filter results")
                .style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for index in 0..3 {
                        filter_choice(index, selection);
                    }
                });
            gap(Dp(16.0));
            text().content(format!(
                "{} filters selected",
                selection.get().into_iter().filter(|value| *value).count()
            ));
        });
    }));
}

#[tessera]
fn filter_choice(index: usize, selection: State<[bool; 3]>) {
    let selected = selection.get()[index];
    if selected {
        chip()
            .filter(["Unread", "Favorites", "Attachments"][index])
            .shape_style(ChipShapeStyle::Standard)
            .selected(true)
            .on_click(move || selection.with_mut(|value| value[index] = !value[index]))
            .leading_icon(filled::CHECK_SVG);
    } else {
        chip()
            .filter(["Unread", "Favorites", "Attachments"][index])
            .shape_style(ChipShapeStyle::Standard)
            .selected(false)
            .on_click(move || selection.with_mut(|value| value[index] = !value[index]));
    }
}

#[tessera]
fn filter_guidelines() {
    prose(
        "Filter chips use tags or descriptive words to filter content. They can be a good alternative to segmented buttons or checkboxes when viewing a list or search results.",
    );
    gap(Dp(16.0));
    prose(
        "Tapping on a filter chip activates it and appends a leading checkmark icon to the starting edge of the chip label.",
    );
    gap(Dp(16.0));
    prose(
        "Write filter chips with nouns that describe the category to include in the results. Avoid negative phrases like Exclude images.",
    );
    gap(Dp(24.0));
    filter_example();
    gap(Dp(16.0));
    caption("Filter chips rely on tags or descriptive words to filter content");
    gap(Dp(24.0));
    prose("Tap a chip to select it. Multiple chips can be selected or unselected.");
    gap(Dp(16.0));
    caption("Filter chips in a shopping app and in a real estate app");
    gap(Dp(24.0));
    do_dont(
        "Use nouns that describe the category to include in results",
        "Avoid negative phrases like Exclude images",
    );
}

#[tessera]
fn input_recipient(#[prop(into)] label: String) {
    let visible = remember(|| true);
    let selected = remember(|| false);
    if visible.get() {
        chip()
            .input(label.clone())
            .shape_style(ChipShapeStyle::Standard)
            .leading_icon(filled::PERSON_SVG)
            .selected(selected.get())
            .on_click(move || selected.set(!selected.get()))
            .on_remove(move || visible.set(false));
    }
}

#[tessera]
fn input_example() {
    let visible = remember(|| [true; 3]);
    let selected = remember(|| [false; 3]);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text().content("To").style(type_style(14.0, 20.0));
            gap(Dp(8.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for index in 0..3 {
                        recipient(index, visible, selected);
                    }
                });
            gap(Dp(16.0));
            button()
                .text()
                .accessibility_label("Restore recipients")
                .on_click(move || {
                    visible.set([true; 3]);
                    selected.set([false; 3]);
                })
                .child(|| {
                    text().content("Restore recipients");
                });
        });
    }));
}

#[tessera]
fn recipient(index: usize, visible: State<[bool; 3]>, selected: State<[bool; 3]>) {
    if visible.get()[index] {
        chip()
            .input(["Alex", "Sam", "Taylor"][index])
            .shape_style(ChipShapeStyle::Standard)
            .leading_icon(filled::PERSON_SVG)
            .selected(selected.get()[index])
            .on_click(move || selected.with_mut(|value| value[index] = !value[index]))
            .on_remove(move || visible.with_mut(|value| value[index] = false));
    }
}

#[tessera]
fn input_guidelines() {
    prose(
        "Input chips represent information entered by a person. Their body can select or edit the item; the independent trailing control removes it.",
    );
    gap(Dp(16.0));
    prose(
        "Use input chips for discrete pieces of information, such as recipients, filters, or tags. A leading icon, logo or circular avatar can identify the item, and the removal action must stay independent.",
    );
    gap(Dp(24.0));
    input_example();
    gap(Dp(16.0));
    caption("Input chips represent discrete pieces of information entered by a person");
    gap(Dp(24.0));
    subheading("Placement".into());
    prose(
        "Place input chips inline, in a wrapping set, or in a horizontally scrolling field. Keep all values visible when the task benefits from reviewing the whole set.",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, false);
    gap(Dp(16.0));
    do_dont(
        "Use a leading avatar or icon and a separate trailing remove action",
        "Avoid making the whole chip the remove action; the chip body and its removal stay separate",
    );
}

#[tessera]
fn suggestion_example() {
    let reply = remember(String::new);
    panel(RenderSlot::new(move || {
        column().children(move || {
            prose("Would you like to meet tomorrow?");
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for label in ["Sounds good", "Maybe later", "What time?"] {
                        suggestion_choice(label.to_owned(), reply);
                    }
                });
            if !reply.with(String::is_empty) {
                gap(Dp(16.0));
                prose(format!("Reply: {}", reply.get()));
            }
        });
    }));
}

#[tessera]
fn suggestion_choice(#[prop(into)] label: String, reply: State<String>) {
    chip()
        .suggestion(label.clone())
        .shape_style(ChipShapeStyle::Standard)
        .on_click(move || reply.set(label.clone()));
}

#[tessera]
fn suggestion_guidelines() {
    prose(
        "Suggestion chips help narrow a person’s intent by presenting dynamically-generated suggestions. They are product-authored, so their labels come from the app rather than the person.",
    );
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(16.0));
    caption("Suggestion chips offer short, product-generated replies or search suggestions");
    gap(Dp(24.0));
    subheading("Placement".into());
    prose(
        "Keep suggestions in an inline set that wraps as needed. Preserve at least 8dp between visual chips and 48dp interaction targets.",
    );
    gap(Dp(24.0));
    do_dont(
        "Offer a short set of suggestions next to the content they answer",
        "Avoid presenting a single suggestion chip on its own",
    );
}

#[tessera]
fn tokens() {
    let selected = remember(|| ChipVariant::Assist);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for variant in VARIANTS {
                button()
                    .text()
                    .accessibility_label(format!("{} chip tokens", name(variant)))
                    .on_click(move || selected.set(variant))
                    .child(move || {
                        text().content(name(variant));
                    });
            }
        });
    gap(Dp(24.0));
    token_table(selected.get());
    gap(Dp(32.0));
    value_table(selected.get());
}

#[tessera]
fn token_table(variant: ChipVariant) {
    let prefix = token_prefix(variant);
    let selectable = selectable(variant);
    let mut rows = vec![
        vec![
            "Container color".into(),
            format!("{prefix}.container.color"),
            if selectable { "Transparent" } else { "Surface" }.into(),
        ],
        vec![
            "Container elevation".into(),
            format!("{prefix}.container.elevation"),
            "0dp · elevated 1dp".into(),
        ],
        vec![
            "Label color".into(),
            format!("{prefix}.label.text.color"),
            if variant == ChipVariant::Assist {
                "On surface"
            } else {
                "On surface variant"
            }
            .into(),
        ],
        vec![
            "Outline color".into(),
            format!("{prefix}.outline.color"),
            if selectable {
                "Outline variant"
            } else {
                "Outline"
            }
            .into(),
        ],
        vec![
            "Shape".into(),
            format!("{prefix}.container.shape"),
            "8dp corner".into(),
        ],
    ];
    if variant != ChipVariant::Suggestion {
        rows.push(vec![
            "Leading icon color".into(),
            format!("{prefix}.leading-icon.color"),
            if variant == ChipVariant::Assist {
                "Primary"
            } else {
                "On surface variant"
            }
            .into(),
        ]);
    }
    if matches!(variant, ChipVariant::Filter | ChipVariant::Input) {
        rows.push(vec![
            "Trailing icon color".into(),
            format!("{prefix}.trailing-icon.color"),
            "On surface variant".into(),
        ]);
    }
    if selectable {
        rows.push(vec![
            "Selected container".into(),
            format!("{prefix}.container.color"),
            "Secondary container".into(),
        ]);
        rows.push(vec![
            "Selected label".into(),
            format!("{prefix}.label.text.color"),
            "On secondary container".into(),
        ]);
        rows.push(vec![
            "Selected icon".into(),
            format!("{prefix}.leading-icon.color"),
            "On secondary container".into(),
        ]);
    }
    spec_table(vec!["Element".into(), "Token".into(), "Value".into()], rows);
}

#[tessera]
fn value_table(variant: ChipVariant) {
    if variant == ChipVariant::Suggestion {
        spec_table(
            vec!["Attribute".into(), "Value".into()],
            vec![
                vec!["Container height".into(), "32dp".into()],
                vec!["Container shape".into(), "8dp corner radius".into()],
                vec![
                    "Label alignment".into(),
                    "Center-aligned · start-aligned".into(),
                ],
                vec!["Left/right padding".into(), "16dp".into()],
                vec!["Padding between elements".into(), "8dp".into()],
            ],
        );
        return;
    }
    let mut rows = vec![
        vec!["Container height".into(), "32dp".into()],
        vec!["Container shape".into(), "8dp corner radius".into()],
        vec!["Icon size".into(), "18dp".into()],
        vec![
            "Vertical label text alignment".into(),
            "Center-aligned".into(),
        ],
        vec![
            "Horizontal label text alignment".into(),
            "Start-aligned".into(),
        ],
        vec!["Left/right padding without icon".into(), "16dp".into()],
        vec!["Left/right padding with icon".into(), "8dp".into()],
        vec!["Padding between elements".into(), "8dp".into()],
    ];
    if variant == ChipVariant::Input {
        rows.push(vec!["Avatar shape".into(), "12dp corner radius".into()]);
        rows.push(vec!["Avatar size".into(), "24dp".into()]);
        rows.push(vec!["Left padding for avatar".into(), "4dp".into()]);
        rows.push(vec!["Right padding for avatar".into(), "8dp".into()]);
        rows.push(vec!["Target size for close icon".into(), "Min 48dp".into()]);
    }
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[tessera]
fn examples(variant: ChipVariant, enabled: bool) {
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            for style in [ChipStyle::Flat, ChipStyle::Elevated] {
                sample(
                    variant,
                    style,
                    false,
                    enabled,
                    true,
                    format!("{} {style:?}", name(variant)),
                );
            }
            if selectable(variant) {
                sample(
                    variant,
                    ChipStyle::Flat,
                    true,
                    enabled,
                    true,
                    format!("{} selected", name(variant)),
                );
            }
        });
}

#[tessera]
fn sample(
    variant: ChipVariant,
    style: ChipStyle,
    initial: bool,
    enabled: bool,
    with_icon: bool,
    #[prop(into)] label: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    let toggles = selectable(variant);
    column().children(move || {
        if with_icon && variant != ChipVariant::Suggestion {
            chip()
                .variant(variant)
                .style(style)
                .shape_style(ChipShapeStyle::Standard)
                .label(label.clone())
                .accessibility_label(label.clone())
                .enabled(enabled)
                .selected(selected.get())
                .on_click(move || {
                    actions.set(actions.get() + 1);
                    if toggles {
                        selected.set(!selected.get());
                    }
                })
                .leading_icon(if selected.get() && toggles {
                    filled::CHECK_SVG
                } else {
                    filled::ADD_SVG
                });
        } else {
            chip()
                .variant(variant)
                .style(style)
                .shape_style(ChipShapeStyle::Standard)
                .label(label.clone())
                .accessibility_label(label.clone())
                .enabled(enabled)
                .selected(selected.get())
                .on_click(move || {
                    actions.set(actions.get() + 1);
                    if toggles {
                        selected.set(!selected.get());
                    }
                });
        }
        if !toggles {
            gap(Dp(8.0));
            text()
                .content(format!("Actions: {}", actions.get()))
                .style(type_style(12.0, 16.0));
        }
    });
}

#[tessera]
fn scheme(variant: ChipVariant, dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            subheading(if dark { "Dark theme" } else { "Light theme" }.to_owned());
                            gap(Dp(24.0));
                            examples(variant, true);
                        });
                });
        });
}

#[tessera]
fn detail(variant: ChipVariant) {
    chip_diagram(variant, false);
    gap(Dp(16.0));
    let labels = match variant {
        ChipVariant::Assist => vec![
            (1_usize, "Container"),
            (2, "Label text"),
            (3, "Leading icon"),
        ],
        ChipVariant::Filter => vec![
            (1_usize, "Container"),
            (2, "Label text"),
            (3, "Leading icon"),
            (4, "Trailing icon"),
        ],
        ChipVariant::Input => vec![
            (1_usize, "Container"),
            (2, "Label text"),
            (3, "Trailing icon"),
            (4, "Leading icon"),
        ],
        ChipVariant::Suggestion => vec![(1_usize, "Container"), (2, "Label text")],
    };
    column().children(move || {
        for (number, label) in labels.clone() {
            legend(number, label.to_owned());
        }
    });
    gap(Dp(32.0));
    subheading(format!("{} chip color", name(variant)));
    gap(Dp(16.0));
    prose(format!(
        "{} chip color roles used for light and dark themes: {}.",
        name(variant),
        match variant {
            ChipVariant::Assist => "Surface container low (optional), on surface, outline, primary",
            ChipVariant::Filter =>
                "On surface variant, on secondary container, secondary container, outline variant, surface container low (optional)",
            ChipVariant::Input =>
                "On surface variant, surface container low (optional), outline variant, primary, secondary container, on secondary container",
            ChipVariant::Suggestion =>
                "Outline, surface container low (optional), on surface variant",
        }
    ));
    gap(Dp(24.0));
    scheme(variant, false);
    gap(Dp(24.0));
    scheme(variant, true);
    gap(Dp(48.0));
    subheading(format!("{} chip states", name(variant)));
    gap(Dp(16.0));
    prose(
        "Enabled, disabled, hovered, focused, pressed and dragged states use the same container, label and icon color roles. Hover, focus with Tab, or press and hold to inspect actual input feedback.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            subheading("Enabled".into());
            gap(Dp(16.0));
            examples(variant, true);
            gap(Dp(24.0));
            subheading("Disabled".into());
            gap(Dp(16.0));
            examples(variant, false);
        });
    }));
    gap(Dp(48.0));
    subheading(format!("{} chip measurements", name(variant)));
    gap(Dp(24.0));
    value_table(variant);
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Use a chip to perform an action".into(),
        "Navigate to a chip".into(),
        "Activate a chip".into(),
    ]);
    gap(Dp(24.0));
    filter_example();
}

#[tessera]
fn interaction_style() {
    prose("The chip label needs at least 3:1 contrast with the background.");
    gap(Dp(16.0));
    prose(
        "A chip that performs an action should present the same semantics as a button to a platform’s accessibility API.",
    );
    gap(Dp(24.0));
    prose(
        "This native sample exposes assist chips as buttons, filter and input chips as toggleable controls, and the input chip removal as a separate action.",
    );
    gap(Dp(24.0));
    filter_example();
    gap(Dp(16.0));
    caption("High contrast helps differentiate chips clustered together");
    gap(Dp(32.0));
    subheading("Horizontal overflow".into());
    prose(
        "When there are too many chips to fit on one row, provide a way to display them all at once and avoid scrolling.",
    );
    gap(Dp(16.0));
    prose(
        "Reflow method: Use a filter chip as a leading element to reflow the horizontal list. This should shift down the content below and make room for all chips to show.",
    );
    gap(Dp(24.0));
    overflow_example();
    gap(Dp(16.0));
    caption(
        "The Show all filter chip is used to reflow the list, displaying all chips at once and pushing down the content below",
    );
    gap(Dp(32.0));
    prose(
        "Menu method: Create a leading button to display all chip options in a menu. Use this option to avoid shifting the position of the content below.",
    );
    gap(Dp(16.0));
    prose("Don’t use the menu method on chips with a second action, like a remove icon.");
    gap(Dp(32.0));
    subheading("Avoid applying density by default".into());
    prose(
        "Don’t apply density to chips by default — this lowers their targets below our best practice of 48x48 CSS pixels. Instead, give people a way to choose a higher density, like selecting a denser layout or changing the theme.",
    );
    gap(Dp(16.0));
    prose(
        "To ensure that this density setting can be easily reverted when it’s active, keep all the targets to change it at minimum 48x48 CSS pixels each.",
    );
}

#[tessera]
fn overflow_example() {
    let all = remember(|| false);
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                chip()
                    .filter(if all.get() { "Show less" } else { "Show all" })
                    .shape_style(ChipShapeStyle::Standard)
                    .selected(all.get())
                    .on_click(move || all.set(!all.get()));
                for index in 0..6 {
                    overflow_choice(index, all.get());
                }
            });
    }));
}

#[tessera]
fn overflow_choice(index: usize, all: bool) {
    let selected = remember(|| false);
    if all || index < 2 {
        chip()
            .filter(["Music", "Movies", "Books", "Travel", "Food", "Sports"][index])
            .shape_style(ChipShapeStyle::Standard)
            .selected(selected.get())
            .on_click(move || selected.set(!selected.get()));
    }
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab".into(),
                "Moves focus to enabled chip or chip group".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Activates, selects, or deselects the focused chip".into(),
            ],
            vec![
                "Backspace or Delete".into(),
                "Removes currently focused input chip".into(),
            ],
            vec!["Arrows".into(), "Moves focus between chips".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "This native sample follows the table: Tab reaches each chip, Space or Enter activates the focused chip, and the remove action of an input chip is a separate focus stop.",
    );
    gap(Dp(24.0));
    input_example();
}

#[tessera]
fn labeling() {
    prose(
        "Use the visible chip label as its accessible name. Decorative icons are not repeated. Additional actions have their own labels.",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "Element".into(),
            "A11y label".into(),
            "Role (Web)".into(),
            "Role (Android Views)".into(),
            "Role (Jetpack Compose)".into(),
        ],
        vec![
            vec![
                "Image / Icon within chip".into(),
                "Hide image".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            ],
            vec![
                "Basic chip (one action)".into(),
                "{chip content}".into(),
                "gridcell".into(),
                "button".into(),
                "button".into(),
            ],
            vec![
                "Selectable chip".into(),
                "{chip content}".into(),
                "gridcell".into(),
                "radio button".into(),
                "checkbox".into(),
            ],
            vec![
                "Remove icon (no other action)".into(),
                "Remove {chip content}".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            ],
            vec![
                "Two actions (e.g., select + remove)".into(),
                "{chip content}. Then “Remove {chip content}”".into(),
                "button or checkbox".into(),
                "button or checkbox".into(),
                "button or checkbox".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("Multi-select".into());
    prose("Space or Enter toggles a focused selectable chip without changing other choices.");
    gap(Dp(24.0));
    filter_example();
    gap(Dp(32.0));
    subheading("Drop-down list".into());
    prose(
        "Menu item labels match their visible text; decorative icons do not add redundant announcements.",
    );
    gap(Dp(32.0));
    subheading("Input chip remove action".into());
    prose(
        "A chip with selection and removal exposes two independent focusable actions. Name the second action Remove followed by the chip label.",
    );
    gap(Dp(24.0));
    input_example();
    gap(Dp(32.0));
    subheading("Showing chip interactivity".into());
    prose(
        "Introduce the chip group with a meaningful instruction, such as Filter results, or use an action-oriented label and leading icon.",
    );
}
