//! Interactive chip specifications by purpose.
//!
//! ## Usage
//!
//! Compare compact actions, independent filters, input tokens and suggestions.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    chip::{ChipShapeStyle, ChipStyle, ChipVariant, chip},
    column::column,
    flow_row::flow_row,
    material_icons::filled,
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _},
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, provide_context, remember, tessera, use_context,
};

use super::{
    ACCENT,
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

fn name(variant: ChipVariant) -> &'static str {
    match variant {
        ChipVariant::Assist => "Assist",
        ChipVariant::Filter => "Filter",
        ChipVariant::Input => "Input",
        ChipVariant::Suggestion => "Suggestion",
    }
}

#[tessera]
pub(super) fn chips_spec() {
    article(
        "Chips",
        "Enter information, select filters and trigger contextual actions.",
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
    }).pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(chip_variant_set)),sections:vec![
            SpecSection {title:"Updates",content:RenderSlot::new(|| {prose("The updated outline variant stroke creates a subtler hierarchy between chips and buttons.");gap(Dp(24.0));chip_variant_set();})},
            SpecSection {title:"Differences from M2",content:RenderSlot::new(|| {prose("M3 separates actions into assist and suggestion chips, treats choice chips as filter chips, and uses dynamic color mappings with rounded rectangular shapes.");})},
        ]},
        ArticlePage {title:"Guidelines",intro:Some(RenderSlot::new(chip_variant_set)),sections:vec![
            SpecSection {title:"Usage",content:RenderSlot::new(chip_usage)},
            SpecSection {title:"Anatomy",content:RenderSlot::new(chip_guideline_anatomy)},
            SpecSection {title:"Assist chips",content:RenderSlot::new(assist_example)},
            SpecSection {title:"Filter chips",content:RenderSlot::new(filter_example)},
            SpecSection {title:"Input chips",content:RenderSlot::new(input_guidelines)},
            SpecSection {title:"Suggestion chips",content:RenderSlot::new(suggestion_guidelines)},
        ]},
        ArticlePage {title:"Accessibility",intro:None,sections:vec![
            SpecSection {title:"Use cases",content:RenderSlot::new(|| {prose("People must be able to reach a chip, identify its purpose and activate or select it using assistive technology.");gap(Dp(24.0));filter_example();})},
            SpecSection {title:"Interaction & style",content:RenderSlot::new(chip_interaction)},
            SpecSection {title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab".into(),"Reach the next enabled chip or removal action".into()],vec!["Space or Enter".into(),"Activate or toggle the focused action".into()]]);})},
            SpecSection {title:"Labeling elements",content:RenderSlot::new(chip_labels)},
        ]},
    ]);
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
    let selectable = matches!(variant, ChipVariant::Filter | ChipVariant::Input);
    column().children(move || {
        let mut control = chip()
            .variant(variant)
            .style(style)
            .shape_style(ChipShapeStyle::Standard)
            .label(label.clone())
            .accessibility_label(label.clone())
            .enabled(enabled)
            .selected(selected.get())
            .on_click(move || {
                actions.set(actions.get() + 1);
                if selectable {
                    selected.set(!selected.get());
                }
            });
        if with_icon && variant != ChipVariant::Suggestion {
            control = control.leading_icon(if selected.get() && selectable {
                filled::CHECK_SVG
            } else {
                filled::ADD_SVG
            });
        }
        drop(control);
        if !selectable {
            gap(Dp(8.0));
            text()
                .content(format!("Actions: {}", actions.get()))
                .style(type_style(12.0, 16.0));
        }
    });
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
}

#[tessera]
fn token_table(variant: ChipVariant) {
    let selectable = matches!(variant, ChipVariant::Filter | ChipVariant::Input);
    spec_table(
        vec![
            "Element".into(),
            "Unselected / default".into(),
            "Selected".into(),
        ],
        vec![
            vec![
                "Flat container".into(),
                "Transparent".into(),
                if selectable {
                    "Secondary container"
                } else {
                    "Transparent"
                }
                .into(),
            ],
            vec![
                "Elevated container".into(),
                "Surface container low".into(),
                if selectable {
                    "Secondary container"
                } else {
                    "Surface container low"
                }
                .into(),
            ],
            vec![
                "Label".into(),
                if variant == ChipVariant::Assist {
                    "On surface"
                } else {
                    "On surface variant"
                }
                .into(),
                if selectable {
                    "On secondary container"
                } else if variant == ChipVariant::Assist {
                    "On surface"
                } else {
                    "On surface variant"
                }
                .into(),
            ],
            vec![
                "Flat outline".into(),
                if selectable {
                    "Outline variant"
                } else {
                    "Outline"
                }
                .into(),
                if selectable { "None" } else { "Outline" }.into(),
            ],
        ],
    );
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
            if matches!(variant, ChipVariant::Filter | ChipVariant::Input) {
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
    panel(RenderSlot::new(move || {
        examples(variant, true);
    }));
    gap(Dp(24.0));
    text().content(match variant {
        ChipVariant::Assist => "1  Container    2  Label    3  Optional leading icon",
        ChipVariant::Filter => "1  Container    2  Label    3  Optional leading icon    4  Optional trailing icon",
        ChipVariant::Input => "1  Container    2  Label    3  Optional trailing icon    4  Optional leading icon or avatar",
        ChipVariant::Suggestion => "1  Container    2  Label",
    }).style(type_style(16.0, 24.0));
    gap(Dp(48.0));
    subheading(format!("{} chip color", name(variant)));
    gap(Dp(24.0));
    scheme(variant, false);
    gap(Dp(24.0));
    scheme(variant, true);
    gap(Dp(48.0));
    subheading(format!("{} chip states", name(variant)));
    gap(Dp(24.0));
    text()
        .content("Hover, focus with Tab, or press and hold to inspect actual input feedback.")
        .style(type_style(16.0, 24.0));
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
    let mut rows = vec![
        vec!["Container height".into(), "32dp".into()],
        vec!["Corner radius".into(), "8dp".into()],
        vec!["Icon".into(), "18dp".into()],
        vec![
            "Label alignment".into(),
            "Start-aligned · vertically centered".into(),
        ],
        vec!["Icon-to-container edge padding".into(), "8dp".into()],
        vec!["Between elements".into(), "8dp".into()],
    ];
    if variant != ChipVariant::Input {
        rows.push(vec![
            "Horizontal padding without icon".into(),
            "16dp".into(),
        ]);
    }
    if variant == ChipVariant::Input {
        rows.push(vec!["Avatar".into(), "24dp · 12dp radius".into()]);
        rows.push(vec!["Leading avatar padding".into(), "4dp".into()]);
        rows.push(vec!["Close icon target".into(), "At least 48dp".into()]);
    }
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn chip_variant_set() {
    let action = remember(|| false);
    let filter = remember(|| false);
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                chip()
                    .assist(if action.get() {
                        "Added"
                    } else {
                        "Add to calendar"
                    })
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::EVENT_SVG)
                    .on_click(move || action.set(!action.get()));
                chip()
                    .filter("Filter chip")
                    .shape_style(ChipShapeStyle::Standard)
                    .selected(filter.get())
                    .leading_icon(filled::CHECK_SVG)
                    .on_click(move || filter.set(!filter.get()));
                input_recipient("Input chip".into());
                chip()
                    .suggestion("Sounds good")
                    .shape_style(ChipShapeStyle::Standard)
                    .on_click(move || action.set(true));
            });
    }));
}

#[tessera]
fn chip_usage() {
    prose(
        "Use a group of contextual choices to make the current task easier. A chip should supplement the journey, while buttons advance or complete the task.",
    );
    gap(Dp(24.0));
    subheading("Chips aren’t buttons".into());
    prose(
        "Present several chips together. Use persistent buttons for major actions, and dynamic chips for optional directions, filters or suggestions.",
    );
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(24.0));
    subheading("Variants".into());
    spec_table(
        vec!["Purpose".into(), "Variant".into(), "Example".into()],
        vec![
            vec![
                "Contextual action".into(),
                "Assist".into(),
                "Add to calendar".into(),
            ],
            vec![
                "Narrow a collection".into(),
                "Filter".into(),
                "Unread messages".into(),
            ],
            vec![
                "Person-authored information".into(),
                "Input".into(),
                "A recipient".into(),
            ],
            vec![
                "Product-authored information".into(),
                "Suggestion".into(),
                "A quick reply".into(),
            ],
        ],
    );
}

#[tessera]
fn chip_guideline_anatomy() {
    chip_diagram(ChipVariant::Input, false);
    gap(Dp(24.0));
    subheading("Container".into());
    prose(
        "Standard chip containers have 8dp corners. Use an outline on ordinary surfaces and reserve elevation for imagery or complex backgrounds.",
    );
    gap(Dp(24.0));
    subheading("Label text".into());
    prose(
        "Keep labels brief, ideally no more than twenty characters, using the label typography shared with buttons.",
    );
    gap(Dp(24.0));
    subheading("Leading icon or image (optional)".into());
    prose(
        "An icon, logo or circular avatar can identify the category. Images are larger than icons so their detail remains visible.",
    );
    gap(Dp(24.0));
    subheading("Trailing icon (optional, input and filter chips only)".into());
    prose(
        "Input chips use the trailing action for removal. Separate actions require 48 × 48dp targets and an 88dp minimum chip width, preserving the compact visible height.",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, true);
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
                    } else {
                        let mut control = chip()
                            .variant(variant)
                            .label(format!("{} chip", name(variant)))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()));
                        if variant != ChipVariant::Suggestion {
                            control = control.leading_icon(if variant == ChipVariant::Input {
                                filled::IMAGE_SVG
                            } else {
                                filled::ADD_SVG
                            });
                        }
                        if variant == ChipVariant::Input {
                            control = control.on_remove(move || removed.set(true));
                        } else if variant == ChipVariant::Filter {
                            control = control.trailing_icon(filled::ARROW_DROP_DOWN_SVG);
                        }
                        drop(control);
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
    prose(
        "Assist chips offer contextual actions, often beginning with a verb. Update the label to confirm completion and place the set below primary content.",
    );
    gap(Dp(24.0));
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
fn filter_example() {
    let selection = remember(|| [false; 3]);
    prose(
        "Use descriptive category labels. A leading checkmark reinforces selection. Keep single-select and multi-select behavior consistent within the page.",
    );
    gap(Dp(24.0));
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
    let mut control = chip()
        .filter(["Unread", "Favorites", "Attachments"][index])
        .shape_style(ChipShapeStyle::Standard)
        .selected(selected)
        .on_click(move || selection.with_mut(|value| value[index] = !value[index]));
    if selected {
        control = control.leading_icon(filled::CHECK_SVG);
    }
    drop(control);
}

#[tessera]
fn input_recipient(label: String) {
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
    gap(Dp(24.0));
    input_example();
    gap(Dp(24.0));
    subheading("Placement".into());
    prose(
        "Place input chips inline, in a wrapping set, or in a horizontally scrolling field. Keep all values visible when the task benefits from reviewing the whole set.",
    );
    gap(Dp(24.0));
    subheading("Icons & images".into());
    prose(
        "Use a leading icon, logo or circular avatar. Keep the removal icon at the trailing edge and name it separately from the chip's primary action.",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, false);
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
fn suggestion_choice(label: String, reply: State<String>) {
    chip()
        .suggestion(label.clone())
        .shape_style(ChipShapeStyle::Standard)
        .on_click(move || reply.set(label.clone()));
}

#[tessera]
fn suggestion_guidelines() {
    prose("Offer short, product-generated replies or search suggestions that help narrow intent.");
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(24.0));
    subheading("Placement".into());
    prose(
        "Keep suggestions in an inline set that wraps as needed. Preserve at least 8dp between visual chips and 48dp interaction targets.",
    );
}

#[tessera]
fn chip_interaction() {
    prose(
        "Use sufficient text contrast and communicate chip interactivity with a clear group label, contextual action or icon.",
    );
    gap(Dp(24.0));
    subheading("Horizontal overflow".into());
    prose(
        "Offer a way to reveal every option at once. Reflowing a chip set avoids relying on horizontal scrolling; a menu is another option for chips without a second action.",
    );
    gap(Dp(24.0));
    overflow_example();
    gap(Dp(24.0));
    subheading("Avoid applying density by default".into());
    prose(
        "Keep the default 48 × 48dp interaction targets. A deliberate density setting must remain reversible through adequately sized controls.",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, true);
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
fn chip_labels() {
    prose(
        "Use the visible chip label as its accessible name. Decorative icons are not repeated. Additional actions have their own labels.",
    );
    gap(Dp(24.0));
    subheading("Multi-select".into());
    prose("Space or Enter toggles a focused selectable chip without changing other choices.");
    gap(Dp(24.0));
    filter_example();
    gap(Dp(24.0));
    subheading("Drop-down list".into());
    prose(
        "Menu item labels match their visible text; decorative icons do not add redundant announcements.",
    );
    gap(Dp(24.0));
    subheading("Input chip remove action".into());
    prose(
        "A chip with selection and removal exposes two independent focusable actions. Name the second action Remove followed by the chip label.",
    );
    gap(Dp(24.0));
    input_example();
    gap(Dp(24.0));
    subheading("Showing chip interactivity".into());
    prose(
        "Introduce the chip group with a meaningful instruction, such as Filter results, or use an action-oriented label and leading icon.",
    );
}
