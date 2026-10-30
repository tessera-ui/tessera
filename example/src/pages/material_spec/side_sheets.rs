//! Side sheet specifications, tokens and modal demonstrations.
//!
//! ## Usage
//!
//! Compare standard and modal side sheets and review dismissal behavior.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonSize, button},
    checkbox::checkbox,
    column::column,
    divider::horizontal_divider,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider, standard_side_sheet_provider},
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use super::{
    INK,
    annotations::{self, SpaceGeometry, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const SCRIM_COLOR: Color = Color::from_rgb(0.4512, 0.4392, 0.4706);

struct FilterLabels;

impl FilterLabels {
    const ITEMS: [&'static str; 5] = ["Events", "Personal", "Projects", "Reminders", "Family"];

    fn defaults() -> [bool; 5] {
        [false, false, true, true, false]
    }
}

#[tessera]
pub(super) fn side_sheets_spec() {
    let modal_open = remember(|| false);
    let standard_open = remember(|| false);
    let filters = remember(FilterLabels::defaults);
    let notice = remember(|| String::from("No action selected"));
    let main = RenderSlot::new(move || {
        sheet_article(modal_open, standard_open, notice);
    });
    let body = RenderSlot::new(move || {
        sheet_content(modal_open, filters, notice);
    });
    modal_side_sheet_provider()
        .position(SideSheetPosition::End)
        .accessibility_label("Filters")
        .is_open(modal_open.get())
        .on_close_request(move || modal_open.set(false))
        .main_content_shared(main)
        .side_sheet_content_shared(body);
}

#[tessera]
fn sheet_article(modal_open: State<bool>, standard_open: State<bool>, notice: State<String>) {
    article(
        "Side sheets",
        "Side sheets show secondary content anchored to the side of the screen",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Standard side sheet",
                content: RenderSlot::new(standard_specs),
            },
            SpecSection {
                title: "Modal side sheet",
                content: RenderSlot::new(modal_specs),
            },
        ],
    )
    .intro(|| {
        prose("Standard and modal side sheets share the container, headline, close affordance and 16dp corner radius. A modal side sheet adds a scrim, an optional back icon button and optional action buttons at the bottom edge.");
        gap(Dp(32.0));
        anatomy_pair();
        gap(Dp(16.0));
        caption("1  Standard side sheet     2  Modal side sheet");
        gap(Dp(32.0));
        legend(false);
    })
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(move || {
                overview_intro(modal_open);
            })),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(availability),
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
                prose("Two variants of side sheets.");
                gap(Dp(32.0));
                anatomy_pair();
                gap(Dp(16.0));
                caption("1  Standard side sheet     2  Modal side sheet");
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(move || {
                        usage(modal_open, standard_open);
                    }),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(anatomy_guidance),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(|| {
                        adaptive_design();
                    }),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(move || {
                        behavior(modal_open, notice);
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
                    content: RenderSlot::new(move || {
                        use_cases(modal_open);
                    }),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(move || {
                        initial_focus(modal_open);
                    }),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "Labeling",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
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
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
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
                        .padding_symmetric(Dp(12.0), Dp(4.0))
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
fn launch(open: State<bool>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label("Open modal side sheet")
                .on_click(move || open.set(true))
                .child(|| {
                    text().content("Open modal side sheet");
                });
            gap(Dp(16.0));
            caption(
                "Escape, the scrim and the close icon button all dismiss the modal side sheet.",
            );
        });
}

#[tessera]
fn overview_intro(open: State<bool>) {
    prose(
        "Use side sheets to provide optional content and actions without interrupting the main content.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Two variants: standard and modal".into(),
        "People can navigate to another region within the sheet".into(),
        "Side sheets can contain a back icon for navigation".into(),
    ]);
    gap(Dp(24.0));
    anatomy_pair();
    gap(Dp(16.0));
    caption("1  Standard side sheet     2  Modal side sheet");
    gap(Dp(32.0));
    launch(open);
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
                "Unavailable".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Unavailable".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "Right-to-left (RTL) language support with left side sheet".into(),
        "Color: new color mappings and compatibility with dynamic color".into(),
        "Shape: modal side sheets have a 16dp corner radius".into(),
    ]);
    gap(Dp(24.0));
    caption("Side sheets have new color mappings to support dynamic color");
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token".into()],
        vec![
            vec!["Standard container".into(), "Surface".into()],
            vec!["Modal container".into(), "Surface container low".into()],
            vec!["Divider".into(), "Outline variant, 1dp".into()],
            vec!["Headline".into(), "On surface variant".into()],
            vec!["Icon buttons".into(), "On surface variant".into()],
            vec!["Modal scrim".into(), "Scrim, 32%".into()],
            vec!["Corner radius".into(), "16dp".into()],
            vec!["Maximum width".into(), "400dp".into()],
        ],
    );
}

#[tessera]
fn standard_specs() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            anatomy_diagram(false);
            gap(Dp(24.0));
            legend(false);
            gap(Dp(56.0));
            subheading("Standard side sheet color".into());
            gap(Dp(24.0));
            spec_table(
                vec!["Element".into(), "Color role".into()],
                vec![
                    vec!["1  Divider (optional)".into(), "Outline variant".into()],
                    vec!["2  Headline".into(), "On surface variant".into()],
                    vec!["3  Container".into(), "Surface".into()],
                    vec!["4  Close icon button".into(), "On surface variant".into()],
                ],
            );
            gap(Dp(56.0));
            subheading("Standard side sheet measurements".into());
            gap(Dp(24.0));
            proportions(false);
            gap(Dp(24.0));
            caption("Side sheet padding and size measurements");
            gap(Dp(32.0));
            measurements();
        });
}

#[tessera]
fn modal_specs() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            anatomy_diagram(true);
            gap(Dp(24.0));
            legend(true);
            gap(Dp(56.0));
            subheading("Modal side sheet color".into());
            gap(Dp(24.0));
            spec_table(
                vec!["Element".into(), "Color role".into()],
                vec![
                    vec![
                        "1  Back icon button (optional)".into(),
                        "On surface variant".into(),
                    ],
                    vec!["2  Headline".into(), "On surface variant".into()],
                    vec!["3  Container".into(), "Surface container low".into()],
                    vec!["4  Close icon button".into(), "On surface variant".into()],
                    vec!["5  Divider (optional)".into(), "Outline variant".into()],
                    vec!["6  Scrim".into(), "Scrim, 32%".into()],
                ],
            );
            gap(Dp(56.0));
            subheading("Modal side sheet measurements".into());
            gap(Dp(24.0));
            proportions(true);
            gap(Dp(24.0));
            caption("Side sheet padding and size measurements");
            gap(Dp(32.0));
            measurements();
            gap(Dp(24.0));
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                vec![
                    vec!["Modal corner radius".into(), "16dp".into()],
                    vec!["Modal elevation".into(), "1dp".into()],
                    vec!["Scrim coverage".into(), "Remaining window width".into()],
                ],
            );
        });
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Start/end padding".into(), "24dp".into()],
            vec!["Padding between top elements".into(), "12dp".into()],
            vec!["Bottom actions height".into(), "72dp".into()],
            vec!["Bottom actions top padding".into(), "16dp".into()],
            vec!["Bottom actions bottom padding".into(), "24dp".into()],
            vec![
                "Bottom actions alignment (horizontal)".into(),
                "Left".into(),
            ],
            vec!["Max-width".into(), "400dp".into()],
            vec!["Margins (when detached)".into(), "16dp".into()],
        ],
    );
}

/// Side sheet widths across window size classes.
///
/// The official specification sizes the sheet against the window size class,
/// while this implementation exposes a single maximum width. Both values are
/// listed so the difference stays visible in the article.
#[tessera]
fn width_table() {
    spec_table(
        vec![
            "Window size class".into(),
            "Official width".into(),
            "Native width".into(),
        ],
        vec![
            vec!["Compact".into(), "256dp".into(), "Up to 400dp".into()],
            vec!["Medium".into(), "320dp".into(), "Up to 400dp".into()],
            vec!["Expanded".into(), "400dp".into(), "Up to 400dp".into()],
        ],
    );
}

#[tessera]
fn legend(modal: bool) {
    column().children(move || {
        for (number, label) in if modal {
            vec![
                (1, "Back icon button (optional)"),
                (2, "Headline"),
                (3, "Container"),
                (4, "Close icon button"),
                (5, "Divider (optional)"),
                (6, "Action buttons (optional)"),
                (7, "Scrim"),
            ]
        } else {
            vec![
                (1, "Divider (optional)"),
                (2, "Headline"),
                (3, "Container"),
                (4, "Close icon button"),
            ]
        } {
            prose(format!("{number}  {label}"));
            gap(Dp(12.0));
        }
    });
}

#[tessera]
fn anatomy_pair() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                anatomy_diagram(false);
            });
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                anatomy_diagram(true);
            });
        });
}

#[tessera]
fn anatomy_diagram(modal: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(300.0))
                .background(scheme.surface_container)
                .clip_shape(Shape::rounded_rectangle(Dp(20.0))),
        )
        .children(move || {
            if modal {
                boxed()
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .fill_max_height()
                            .background(SCRIM_COLOR),
                    )
                    .children(|| {});
            } else {
                spacer().modifier(Modifier::new().weight(1.0));
            }
            boxed()
                .modifier(Modifier::new().width(Dp(300.0)).fill_max_height())
                .children(move || {
                    sheet_figure(
                        modal,
                        scheme.surface,
                        scheme.surface_container_low,
                        scheme.outline_variant,
                        scheme.on_surface_variant,
                    );
                });
        });
}

#[tessera]
fn sheet_figure(
    modal: bool,
    standard_container: Color,
    modal_container: Color,
    outline_variant: Color,
    on_surface_variant: Color,
    geometry: Option<State<SpaceGeometry>>,
) {
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .fill_max_height()
                .padding_all(Dp(8.0)),
        )
        .children(move || {
            boxed()
                .modifier(
                    measurement_part(geometry, "container")
                        .fill_max_width()
                        .fill_max_height()
                        .background_with_shape(
                            if modal {
                                modal_container
                            } else {
                                standard_container
                            },
                            Shape::rounded_rectangle(Dp(16.0)),
                        ),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .modifier(
                                    measurement_part(geometry, "header")
                                        .fill_max_width()
                                        .padding_symmetric(Dp(24.0), Dp(12.0)),
                                )
                                .children(move || {
                                    if modal {
                                        icon_button()
                                            .standard()
                                            .icon(filled::ARROW_BACK_SVG)
                                            .accessibility_label("Side sheet figure back")
                                            .modifier(measurement_part(geometry, "back"))
                                            .on_click(|| {});
                                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                                    }
                                    text()
                                        .content("Headline")
                                        .style(type_style(18.0, 24.0))
                                        .color(on_surface_variant)
                                        .modifier(
                                            measurement_part(geometry, "headline").weight(1.0),
                                        );
                                    icon_button()
                                        .standard()
                                        .icon(filled::CLOSE_SVG)
                                        .accessibility_label("Side sheet figure close")
                                        .modifier(measurement_part(geometry, "close"))
                                        .on_click(|| {});
                                });
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(1.0))
                                        .background(outline_variant),
                                )
                                .children(|| {});
                            gap(Dp(16.0));
                            for label in ["Labels", "Events", "Personal", "Projects"] {
                                text()
                                    .content(label)
                                    .style(type_style(14.0, 20.0))
                                    .color(on_surface_variant)
                                    .modifier(Modifier::new().padding_symmetric(Dp(24.0), Dp(6.0)));
                            }
                            spacer().modifier(Modifier::new().weight(1.0));
                            if modal {
                                boxed()
                                    .modifier(
                                        Modifier::new()
                                            .fill_max_width()
                                            .height(Dp(1.0))
                                            .background(outline_variant),
                                    )
                                    .children(|| {});
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .modifier(
                                        measurement_part(geometry, "actions")
                                            .fill_max_width()
                                            .height(Dp(72.0))
                                            .padding_symmetric(Dp(24.0), Dp::ZERO),
                                    )
                                    .children(|| {
                                        button()
                                            .text()
                                            .size(ButtonSize::ExtraSmall)
                                            .accessibility_label("Save filters")
                                            .on_click(|| {})
                                            .child(|| {
                                                text().content("Save");
                                            });
                                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                                        button()
                                            .text()
                                            .size(ButtonSize::ExtraSmall)
                                            .accessibility_label("Cancel filters")
                                            .on_click(|| {})
                                            .child(|| {
                                                text().content("Cancel");
                                            });
                                    });
                            }
                        });
                });
        });
}

fn measurement_part(geometry: Option<State<SpaceGeometry>>, name: &'static str) -> Modifier {
    if let Some(geometry) = geometry {
        watch_in_space(geometry, name).semantics(SemanticsArgs {
            test_tag: Some(format!("side-measurement-{name}")),
            merge_descendants: true,
            ..Default::default()
        })
    } else {
        Modifier::new()
    }
}

#[tessera]
fn proportions(modal: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let geometry = remember(SpaceGeometry::new);
    column().modifier(Modifier::new().fill_max_width()).children(move || {
        boxed()
            .modifier(measurement_part(Some(geometry), "figure").fill_max_width().height(Dp(480.0)))
            .children(move || {
                row()
                    .modifier(Modifier::new().fill_max_size().padding(Padding::new(Dp::ZERO, Dp(32.0), Dp::ZERO, Dp(48.0))).background(scheme.surface_container))
                    .children(move || {
                        if modal {
                            boxed().modifier(Modifier::new().weight(1.0).fill_max_height().background(SCRIM_COLOR));
                        } else {
                            spacer().modifier(Modifier::new().weight(1.0));
                        }
                        boxed()
                            .modifier(Modifier::new().width(Dp(416.0)).fill_max_height())
                            .children(move || {
                                sheet_figure(modal, scheme.surface, scheme.surface_container_low, scheme.outline_variant, scheme.on_surface_variant)
                                    .geometry(geometry);
                            });
                    });
                boxed().modifier(Modifier::new().fill_max_size()).children(move || {
                    measurement_rulers(modal, geometry);
                });
            });
        gap(Dp(16.0));
        caption("24dp start/end padding, measured from the container to the first and last header elements");
        gap(Dp(8.0));
        caption("12dp header top inset; the width ruler follows the rendered container (400dp maximum)");
    });
}

fn dimension_line(svg: &mut String, from: (f64, f64), to: (f64, f64)) {
    let ((x1, y1), (x2, y2)) = (from, to);
    svg.push_str(&format!(
        "<path d='M{x1},{y1} L{x2},{y2}' stroke='#6750a4' stroke-width='1' fill='none'/>",
    ));
    for (x, y) in [from, to] {
        let (dx, dy) = if y1 == y2 { (0.0, 3.0) } else { (3.0, 0.0) };
        svg.push_str(&format!(
            "<path d='M{},{} L{},{}' stroke='#6750a4' stroke-width='1'/>",
            x - dx,
            y - dy,
            x + dx,
            y + dy
        ));
    }
}

#[tessera]
fn measurement_rulers(modal: bool, geometry: State<SpaceGeometry>) {
    let values = geometry.with(annotations::space_geometry);
    let (Some(figure), Some(container), Some(first), Some(close)) = (
        values.get("figure"),
        values.get("container"),
        values.get(if modal { "back" } else { "headline" }),
        values.get("close"),
    ) else {
        return;
    };
    let start = (first.x - container.x).max(0.0);
    let end = (container.x + container.w - close.x - close.w).max(0.0);
    let top = (close.y - container.y).max(0.0);
    let y = container.y - 6.0;
    let bottom = container.y + container.h + 16.0;
    let mut svg = String::new();
    dimension_line(&mut svg, (container.x, y), (first.x, y));
    dimension_line(
        &mut svg,
        (close.x + close.w, y),
        (container.x + container.w, y),
    );
    dimension_line(
        &mut svg,
        (container.x + container.w - 12.0, container.y),
        (container.x + container.w - 12.0, close.y),
    );
    dimension_line(
        &mut svg,
        (container.x, bottom),
        (container.x + container.w, bottom),
    );
    annotations::lines(Dp(figure.w), Dp(figure.h), svg);
    annotations::dimension(format!("{start:.0}"), Dp(container.x), Dp(y - 22.0));
    annotations::dimension(
        format!("{end:.0}"),
        Dp(container.x + container.w - 22.0),
        Dp(y - 22.0),
    );
    annotations::dimension(
        format!("{top:.0}"),
        Dp(container.x + container.w - 22.0),
        Dp(container.y + 2.0),
    );
    annotations::dimension(
        format!("{:.0}dp", container.w),
        Dp(container.x + container.w * 0.5 - 24.0),
        Dp(bottom + 2.0),
    );
}

#[tessera]
fn usage(modal_open: State<bool>, standard_open: State<bool>) {
    prose(
        "Standard side sheets are supplementary surfaces used mostly in medium to expanded breakpoints, like tablet and desktop. They provide a consistent and predictable surface for contextual actions and information.",
    );
    gap(Dp(16.0));
    prose(
        "Standard side sheets display content that complements the screen's primary content. They remain visible while people interact with primary content.",
    );
    gap(Dp(16.0));
    prose(
        "Modal side sheets are preferred in compact breakpoints, like mobile, due to limited screen size. They can display the same kinds of content as standard side sheets, but must be dismissed in order to interact with the underlying content.",
    );
    gap(Dp(16.0));
    subheading("Common uses include".into());
    gap(Dp(12.0));
    bullets(vec![
        "Displaying a list of actions that affect the screen's primary content, such as filters"
            .into(),
        "Displaying supplemental content and features".into(),
    ]);
    gap(Dp(32.0));
    launch(modal_open);
    gap(Dp(32.0));
    subheading("Standard side sheet example".into());
    gap(Dp(16.0));
    standard_demo(standard_open);
    gap(Dp(16.0));
    caption("The standard side sheet shrinks the body area instead of covering it with a scrim.");
}

#[tessera]
fn standard_demo(open: State<bool>) {
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(360.0))
                .clip_shape(Shape::rounded_rectangle(Dp(20.0))),
        )
        .children(move || {
            standard_side_sheet_provider()
                .position(SideSheetPosition::End)
                .accessibility_label("Information")
                .is_open(open.get())
                .on_close_request(move || open.set(false))
                .main_content_shared(RenderSlot::new(move || {
                    standard_body(open);
                }))
                .side_sheet_content_shared(RenderSlot::new(move || {
                    standard_content(open);
                }));
        });
}

#[tessera]
fn standard_body(open: State<bool>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_size().background(scheme.surface))
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                .children(move || {
                    text()
                        .content("Primary content")
                        .style(type_style(18.0, 24.0))
                        .color(INK);
                    gap(Dp(12.0));
                    button()
                        .outlined()
                        .accessibility_label("Open standard side sheet")
                        .on_click(move || open.set(true))
                        .child(|| {
                            text().content("Open standard side sheet");
                        });
                });
        });
}

#[tessera]
fn standard_content(open: State<bool>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface)
                .padding_symmetric(Dp(24.0), Dp(12.0)),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    text()
                        .content("Info")
                        .style(type_style(18.0, 24.0))
                        .color(scheme.on_surface_variant)
                        .modifier(Modifier::new().weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label("Close standard side sheet")
                        .on_click(move || open.set(false));
                });
            gap(Dp(12.0));
            horizontal_divider();
            gap(Dp(16.0));
            text()
                .content("Festival concert")
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
            caption("Albums");
            gap(Dp(8.0));
            text()
                .content("Home space")
                .style(type_style(14.0, 20.0))
                .color(INK);
            text()
                .content("64 items")
                .style(type_style(12.0, 16.0))
                .color(scheme.on_surface_variant);
        });
}

#[tessera]
fn anatomy_guidance() {
    prose(
        "Side sheets have a fixed width and typically span the height of the screen. Their dimensions depend on how the app's layout is subdivided into UI regions.",
    );
    gap(Dp(24.0));
    anatomy_diagram(false);
    gap(Dp(16.0));
    legend(false);
    gap(Dp(32.0));
    subheading("Container".into());
    gap(Dp(12.0));
    prose(
        "Side sheet containers hold all side sheet elements. Their size is determined by the space those elements occupy. The container is the only required element of a side sheet.",
    );
    gap(Dp(24.0));
    subheading("Back icon button (optional)".into());
    gap(Dp(12.0));
    prose(
        "Icon buttons can provide ways to navigate to another region within the sheet. A back icon button navigates to the previous region or closes the sheet when a close affordance is not present.",
    );
    gap(Dp(24.0));
    subheading("Divider (optional)".into());
    gap(Dp(12.0));
    prose(
        "Dividers can separate different kinds of content and create distinct regions in a side sheet. Use a divider to separate action buttons from content, and user-generated content from system-generated content.",
    );
    gap(Dp(32.0));
    do_dont(
        "Place side sheets along the edge of the screen, usually on the right side to avoid interference with any navigational components on the left edge. They can be slightly inset by 16dp.",
        "Don't inset a side sheet from the screen edges far beyond the recommended margin. This makes the sheet's position and scroll behavior unclear, while obscuring primary content.",
    );
}

#[tessera]
fn adaptive_design() {
    prose(
        "Side sheets have a default width, but can be resized depending on the needs of the layout.",
    );
    gap(Dp(16.0));
    prose(
        "When a standard side sheet opens, the body area shrinks to accommodate the sheet's width while maintaining a margin on the body's trailing edge.",
    );
    gap(Dp(24.0));
    width_table();
    gap(Dp(16.0));
    caption(
        "The official specification sizes the sheet from the window size class; this implementation keeps one 400dp maximum width.",
    );
    gap(Dp(32.0));
    subheading("RTL language support".into());
    gap(Dp(12.0));
    prose(
        "In right-to-left (RTL) languages, side sheets should appear on the left edge of the window with all elements reversed.",
    );
}

#[tessera]
fn behavior(modal_open: State<bool>, notice: State<String>) {
    prose(
        "Side sheets can vertically scroll independent of the rest of the UI. This allows their scroll position and content to persist while the page is scrolled, and vice versa. Side sheets cannot scroll horizontally.",
    );
    gap(Dp(16.0));
    prose(
        "A modal side sheet is dismissed by the close icon button, the scrim, or the Escape key. It blocks the main content until it is dismissed, and focus returns to the control that opened it.",
    );
    gap(Dp(24.0));
    launch(modal_open);
    gap(Dp(16.0));
    caption(notice.get());
}

#[tessera]
fn use_cases(modal_open: State<bool>) {
    prose("People should be able to dismiss the side sheet using assistive technology.");
    gap(Dp(16.0));
    bullets(vec![
        "Open the sheet from a control with a descriptive label".into(),
        "Move through the sheet's actions in tab order, including the close affordance".into(),
        "Dismiss the sheet without a drag or pointer gesture".into(),
    ]);
    gap(Dp(24.0));
    launch(modal_open);
}

#[tessera]
fn interaction_style() {
    prose(
        "Material requires that a close affordance, such as a close icon button, is always present within a side sheet.",
    );
    gap(Dp(24.0));
    do_dont(
        "A close icon button makes the side sheet easy to dismiss",
        "Without a close icon button, people can't predict the opening and closing flow of side sheets, or know if the sheet is transient or permanent",
    );
    gap(Dp(24.0));
    prose(
        "Provide a visible close action with a minimum 48 × 48dp target, keep text legible at 200% text size, and preserve contrast for the headline, icons and actions.",
    );
}

#[tessera]
fn initial_focus(modal_open: State<bool>) {
    prose(
        "Actions within a side sheet can be focused by tab order using a keyboard or switch control.",
    );
    gap(Dp(16.0));
    prose(
        "The Material focus-order example includes the headline, close icon button and action buttons. In this interactive example, keyboard focus moves through Close, the label checkboxes, Save and Cancel. Focus enters the modal sheet when it opens and stays inside it until the sheet closes.",
    );
    gap(Dp(24.0));
    launch(modal_open);
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab".into(),
                "Focus lands on the next (non-disabled) control in the sheet".into(),
            ],
            vec![
                "Shift + Tab".into(),
                "Focus lands on the previous (non-disabled) control in the sheet".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Activates the (non-disabled) focused control".into(),
            ],
            vec!["Escape".into(), "Dismisses the modal side sheet".into()],
        ],
    );
}

#[tessera]
fn labeling() {
    prose(
        "The accessibility role for a side sheet is Dialog. Give the sheet a name that describes its content or task, name icon controls by the action they perform, and distinguish closing the sheet from navigating within it.",
    );
}

#[tessera]
fn sheet_content(modal_open: State<bool>, filters: State<[bool; 5]>, notice: State<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface_container_low),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(24.0), Dp(12.0)),
                )
                .children(move || {
                    text()
                        .content("Filters")
                        .style(type_style(18.0, 24.0))
                        .color(scheme.on_surface_variant)
                        .modifier(Modifier::new().weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label("Close side sheet")
                        .on_click(move || modal_open.set(false));
                });
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(1.0))
                        .background(scheme.outline_variant),
                )
                .children(|| {});
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_symmetric(Dp(24.0), Dp(16.0)),
                )
                .children(move || {
                    text()
                        .content("Labels")
                        .style(type_style(12.0, 16.0))
                        .color(scheme.on_surface_variant);
                    gap(Dp(8.0));
                    for (index, label) in FilterLabels::ITEMS.into_iter().enumerate() {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
                            .children(move || {
                                checkbox()
                                    .checked(filters.get()[index])
                                    .on_toggle(move |next| {
                                        filters.with_mut(|values| values[index] = next);
                                    })
                                    .accessibility_label(label);
                                spacer().modifier(Modifier::new().width(Dp(16.0)));
                                text().content(label).style(type_style(14.0, 20.0));
                            });
                    }
                });
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(1.0))
                        .background(scheme.outline_variant),
                )
                .children(|| {});
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(72.0))
                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                )
                .children(move || {
                    button()
                        .text()
                        .accessibility_label("Save filters")
                        .on_click(move || {
                            notice.set(String::from("Filters applied"));
                            modal_open.set(false);
                        })
                        .child(|| {
                            text().content("Save");
                        });
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    button()
                        .text()
                        .size(ButtonSize::ExtraSmall)
                        .accessibility_label("Cancel filters")
                        .on_click(move || modal_open.set(false))
                        .child(|| {
                            text().content("Cancel");
                        });
                });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod measurement_tests {
    use std::cell::Cell;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::Role,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    use super::*;

    thread_local! {
        static MODAL: Cell<bool> = const { Cell::new(false) };
    }

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            column()
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    proportions(MODAL.get());
                });
        });
    }

    #[test]
    fn measurement_figures_build_complete_sheets_and_measure_real_header_insets_at_narrow_widths() {
        for modal in [false, true] {
            MODAL.set(modal);
            for width in [720, 320] {
                let mut session =
                    start_layout_test_session(layout_test(fixture).viewport_px(width, 800));
                pump_layout_test_session(&mut session, 0);
                pump_layout_test_session(&mut session, 16_666_667);
                let snapshot = pump_layout_test_session(&mut session, 33_333_334);
                snapshot
                    .node("side-measurement-figure")
                    .size(width as i32, 480);
                let expected_width = if width == 720 { 400 } else { 304 };
                snapshot
                    .node("side-measurement-container")
                    .size(expected_width, 384);
                let container = snapshot.node("side-measurement-container").bounds();
                let close = snapshot.node("side-measurement-close").bounds();
                let first = snapshot
                    .node(if modal {
                        "side-measurement-back"
                    } else {
                        "side-measurement-headline"
                    })
                    .bounds();
                assert_eq!(first.x.0 - container.x.0, 24);
                assert_eq!(
                    container.x.0 + container.width.0 - close.x.0 - close.width.0,
                    24
                );
                assert_eq!(close.y.0 - container.y.0, 12);
                assert!(first.width.0 > 0 && first.height.0 > 0);
                assert!(close.y.0 + close.height.0 <= container.y.0 + container.height.0);
                if modal {
                    let actions = snapshot.node("side-measurement-actions").bounds();
                    assert_eq!(actions.height.0, 72);
                    assert!(actions.y.0 >= close.y.0 + close.height.0);
                    assert!(actions.y.0 + actions.height.0 <= container.y.0 + container.height.0);
                }
            }
        }
    }

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for complete side-sheet measurement accessibility snapshots"]
    fn measurement_figures_publish_real_headlines_content_and_controls_in_both_variants() {
        for modal in [false, true] {
            MODAL.set(modal);
            for width in [720, 320] {
                let mut renderer = HeadlessRenderer::new(
                    Box::new(fixture),
                    vec![Box::new(Pipelines)],
                    HeadlessConfig {
                        width,
                        height: 800,
                        sample_count: 1,
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(8);
                let snapshot = renderer.accessibility_snapshot().unwrap();
                for label in ["Headline", "Labels", "Events", "Personal", "Projects"] {
                    assert!(
                        snapshot
                            .nodes
                            .iter()
                            .any(|(_, node)| node.label() == Some(label) && !node.is_hidden()),
                        "missing measured sheet content: {label}"
                    );
                }
                let labels: &[&str] = if modal {
                    &[
                        "Side sheet figure back",
                        "Side sheet figure close",
                        "Save filters",
                        "Cancel filters",
                    ]
                } else {
                    &["Side sheet figure close"]
                };
                for label in labels {
                    let node = snapshot
                        .nodes
                        .iter()
                        .find(|(_, node)| {
                            node.role() == Role::Button
                                && node.label() == Some(*label)
                                && !node.is_hidden()
                        })
                        .unwrap_or_else(|| panic!("missing measured sheet button: {label}"));
                    let bounds = node.1.bounds().unwrap();
                    assert!(bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width));
                    assert!(bounds.y0 >= 0.0 && bounds.y1 <= 480.0);
                }
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some("Dimension 24"))
                );
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some("Dimension 12"))
                );
                let width_label = if width == 720 {
                    "Dimension 400dp"
                } else {
                    "Dimension 304dp"
                };
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(width_label))
                );
            }
        }
    }
}
