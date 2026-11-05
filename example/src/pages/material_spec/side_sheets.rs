//! Side sheet specifications, tokens and modal demonstrations.
//!
//! ## Usage
//!
//! Compare standard and modal side sheets and review dismissal behavior.

mod figures;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonSize, button},
    checkbox::checkbox,
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    side_sheet::{
        SideSheetController, SideSheetPosition, modal_side_sheet_provider,
        standard_side_sheet_provider,
    },
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_shard::RouterController;
use tessera_ui::{
    Color, Dp, FocusProperties, FocusRequester, FocusState, Modifier, RenderSlot, State,
    accesskit::Role, modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use self::figures::{guidance_card, legend, split};
use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "side_sheets_tests.rs"]
mod tests;

struct FilterLabels;

impl FilterLabels {
    const ITEMS: [&'static str; 5] = ["Events", "Personal", "Projects", "Reminders", "Family"];

    fn defaults() -> [bool; 5] {
        [false, false, true, true, false]
    }
}

/// Live modal side sheet state owned by the app root while the article is
/// mounted or remounted.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LiveSideSheetSession {
    open: State<bool>,
    filters: State<[bool; 5]>,
    notice: State<String>,
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
}

impl LiveSideSheetSession {
    fn route_active(self) -> bool {
        let selected = self.material.get();
        self.router.with(|router| {
            router
                .current_is::<crate::pages::material_components::MaterialComponentsPageDestination>(
                )
        }) && selected.is_some()
            && selected == super::catalog::component_index(34)
    }

    fn can_open(self) -> bool {
        self.route_active()
            && self
                .directory
                .with(|directory| !directory.is_open() && !directory.is_animating())
    }
}

#[tessera]
pub(crate) fn live_side_sheet_host(
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
    main_content: tessera_ui::RenderSlotWith<LiveSideSheetSession>,
) {
    let session = LiveSideSheetSession {
        open: remember(|| false),
        filters: remember(FilterLabels::defaults),
        notice: remember(|| String::from("No action selected")),
        directory,
        router,
        material,
    };
    if !session.route_active() && session.open.get() {
        session.open.set(false);
    }
    let modal = RenderSlot::new(move || {
        live_modal(
            session.open,
            session.filters,
            session.notice,
            RenderSlot::new(move || {
                main_content.render(session);
            }),
        );
    });
    modal.render();
}

#[tessera]
fn live_modal(
    open: State<bool>,
    filters: State<[bool; 5]>,
    notice: State<String>,
    main_content: RenderSlot,
) {
    modal_side_sheet_provider()
        .position(SideSheetPosition::End)
        .content_padding(Dp::ZERO)
        .accessibility_label("Filters")
        .is_open(open.get())
        .on_close_request(move || open.set(false))
        .main_content_shared(main_content)
        .side_sheet_content(move || {
            sheet_content(open, filters, notice);
        });
}

#[tessera]
pub(super) fn side_sheets_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
    session: Option<LiveSideSheetSession>,
) {
    let root = session;
    let local_open = remember(|| false);
    let local_filters = remember(FilterLabels::defaults);
    let local_notice = remember(|| String::from("No action selected"));
    let standard_open = remember(|| false);
    let (open, filters, notice) = if let Some(session) = root {
        (session.open, session.filters, session.notice)
    } else {
        (local_open, local_filters, local_notice)
    };
    let content = RenderSlot::new(move || {
        sheet_article(open, standard_open, notice)
            .session_optional(session)
            .route_optional(route.clone());
    });
    if root.is_some() {
        content.render();
    } else {
        live_modal(open, filters, notice, content);
    }
}

#[tessera]
fn sheet_article(
    modal_open: State<bool>,
    standard_open: State<bool>,
    notice: State<String>,
    route: Option<crate::pages::material_spec::article::ArticleRoute>,

    session: Option<LiveSideSheetSession>,
) {
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
                content: RenderSlot::new(move || {
                    variant_specs(false);
                }),
            },
            SpecSection {
                title: "Modal side sheet",
                content: RenderSlot::new(move || {
                    variant_specs(true);
                }),
            },
        ],
    )
    .route_optional(route)
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(move || {
                overview_intro(modal_open).session_optional(session);
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
            intro: Some(RenderSlot::new(move || {
                figures::hero();
                gap(Dp(24.0));
                variants_legend();
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(move || {
                        usage(modal_open).session_optional(session);
                    }),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(anatomy_guidance),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(move || {
                        adaptive_design(standard_open);
                    }),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(move || {
                        behavior(modal_open, notice).session_optional(session);
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
                        use_cases(modal_open).session_optional(session);
                    }),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Initial focus",
                    content: RenderSlot::new(move || {
                        initial_focus(modal_open).session_optional(session);
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
fn variants_legend() {
    legend(vec![
        (1, "Standard side sheet".into()),
        (2, "Modal side sheet".into()),
    ]);
}

#[tessera]
fn launch(open: State<bool>, session: Option<LiveSideSheetSession>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label("Open modal side sheet")
                .on_click(move || {
                    if session.is_none_or(LiveSideSheetSession::can_open) {
                        open.set(true);
                    }
                })
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
fn overview_intro(open: State<bool>, session: Option<LiveSideSheetSession>) {
    bullets(vec![
        "Use side sheets to provide optional content and actions without interrupting the main content"
            .into(),
        "Two variants: standard and modal".into(),
        "People can navigate to another region within the sheet".into(),
        "Side sheets can contain a back icon for navigation".into(),
    ]);
    gap(Dp(24.0));
    figures::hero();
    gap(Dp(24.0));
    variants_legend();
    gap(Dp(24.0));
    launch(open).session_optional(session);
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
    split()
        .start_weight(0.5)
        .start(|| {
            bullets(vec![
                "Right-to-left (RTL) language support with left side sheet".into(),
                "Color: new color mappings and compatibility with dynamic color".into(),
                "Shape: modal side sheets have a 16dp corner radius".into(),
            ]);
        })
        .end(|| {
            figures::differences_figure();
            gap(Dp(16.0));
            caption("Side sheets have new color mappings to support dynamic color");
        });
}

struct Token {
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
}

const TOKEN_GROUPS: [&str; 4] = ["Enabled", "Hovered", "Focused", "Pressed (ripple)"];

const TOKENS: [Token; 31] = [
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked container color",
        value: "#FEF7FF",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked modal container color",
        value: "#F7F2FA",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked standard container color",
        value: "#FEF7FF",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked container surface tint layer color",
        value: "#6750A4",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked modal container elevation",
        value: "Level 1",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked standard container elevation",
        value: "Level 0",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked container height",
        value: "100%",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked container shape",
        value: "Square corners",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side detached container shape",
        value: "Rounded corners",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked container width",
        value: "256dp",
    },
    Token {
        group: "Enabled",
        section: "Container",
        name: "Sheet side docked modal container shape",
        value: "Rounded corners",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline color",
        value: "#49454F",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline font",
        value: "Roboto",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline line height",
        value: "28pt",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline size",
        value: "22pt",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline tracking",
        value: "0pt",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline weight",
        value: "400",
    },
    Token {
        group: "Enabled",
        section: "Headline",
        name: "Sheet side docked headline type style",
        value: "Title large",
    },
    Token {
        group: "Enabled",
        section: "Divider",
        name: "Sheet side docked divider color",
        value: "#79747E",
    },
    Token {
        group: "Hovered",
        section: "Label text",
        name: "Sheet side docked action hover label text color",
        value: "#6750A4",
    },
    Token {
        group: "Hovered",
        section: "State layer",
        name: "Sheet side docked action hover state layer color",
        value: "#6750A4",
    },
    Token {
        group: "Hovered",
        section: "State layer",
        name: "Sheet side docked action hover state layer opacity",
        value: "0.08",
    },
    Token {
        group: "Focused",
        section: "Focus indicator",
        name: "Sheet side docked focus indicator color",
        value: "#625B71",
    },
    Token {
        group: "Focused",
        section: "Focus indicator",
        name: "Sheet side docked focus indicator thickness",
        value: "3dp",
    },
    Token {
        group: "Focused",
        section: "Focus indicator",
        name: "Sheet side docked focus indicator offset",
        value: "2dp",
    },
    Token {
        group: "Focused",
        section: "Label text",
        name: "Sheet side docked action focus label text color",
        value: "#6750A4",
    },
    Token {
        group: "Focused",
        section: "State layer",
        name: "Sheet side docked action focus state layer color",
        value: "#6750A4",
    },
    Token {
        group: "Focused",
        section: "State layer",
        name: "Sheet side docked action focus state layer opacity",
        value: "0.1",
    },
    Token {
        group: "Pressed (ripple)",
        section: "Label text",
        name: "Sheet side docked action pressed label text color",
        value: "#6750A4",
    },
    Token {
        group: "Pressed (ripple)",
        section: "State layer",
        name: "Sheet side docked action pressed state layer color",
        value: "#6750A4",
    },
    Token {
        group: "Pressed (ripple)",
        section: "State layer",
        name: "Sheet side docked action pressed state layer opacity",
        value: "0.1",
    },
];

fn parse_hex(value: &str) -> Option<Color> {
    let value = value.strip_prefix('#')?;
    let channel = |index: usize| u8::from_str_radix(value.get(index..index + 2)?, 16).ok();
    Some(srgb(channel(0)?, channel(2)?, channel(4)?))
}

#[tessera]
fn tokens() {
    let expanded = remember(|| [true, false, false, false]);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    prose("Browse the component elements, attributes, tokens, and their values.");
    gap(Dp(24.0));
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .border_with_shape(
                    Dp(1.0),
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(16.0)),
                )
                .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .background(scheme.surface_container_low)
                        .padding_symmetric(Dp(12.0), Dp(8.0)),
                )
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .background_with_shape(
                                    scheme.secondary_container,
                                    Shape::rounded_rectangle(Dp(8.0)),
                                )
                                .padding_symmetric(Dp(16.0), Dp(8.0)),
                        )
                        .children(move || {
                            text()
                                .content("Sheets - Side")
                                .style(type_style(14.0, 20.0))
                                .color(scheme.on_secondary_container);
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    let all_open = expanded.with(|state| state.iter().all(|open| *open));
                    button()
                        .text()
                        .accessibility_label(if all_open {
                            "Collapse all token groups"
                        } else {
                            "Expand all token groups"
                        })
                        .on_click(move || {
                            expanded.with_mut(|state| {
                                let next = !state.iter().all(|open| *open);
                                state.fill(next);
                            });
                        })
                        .child(move || {
                            text().content(if all_open {
                                "Collapse all"
                            } else {
                                "Expand all"
                            });
                        });
                });
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .background(scheme.surface_container_low)
                        .padding_symmetric(Dp(24.0), Dp(12.0)),
                )
                .children(move || {
                    text()
                        .content("Token")
                        .style(type_style(14.0, 20.0))
                        .color(INK)
                        .modifier(Modifier::new().weight(1.0));
                    text()
                        .content("Default, Light")
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                });
            for (index, group) in TOKEN_GROUPS.into_iter().enumerate() {
                let open = expanded.with(|state| state[index]);
                button()
                    .text()
                    .accessibility_label(format!("{group} tokens"))
                    .modifier(Modifier::new().fill_max_width())
                    .on_click(move || expanded.with_mut(|state| state[index] = !state[index]))
                    .child(move || {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(Modifier::new().fill_max_width())
                            .children(move || {
                                icon()
                                    .vector(filled::FOLDER_SVG)
                                    .size(Dp(20.0))
                                    .tint(scheme.on_surface_variant);
                                spacer().modifier(Modifier::new().width(Dp(12.0)));
                                text()
                                    .content(group)
                                    .style(type_style(16.0, 24.0))
                                    .color(INK)
                                    .modifier(Modifier::new().weight(1.0));
                                icon()
                                    .vector(if open {
                                        filled::ARROW_DROP_UP_SVG
                                    } else {
                                        filled::ARROW_DROP_DOWN_SVG
                                    })
                                    .size(Dp(24.0))
                                    .tint(scheme.on_surface_variant);
                            });
                    });
                if open {
                    token_group(group);
                }
            }
        });
}

#[tessera]
fn token_group(#[prop(into)] group: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let mut section = "";
    for token in TOKENS.iter().filter(|token| token.group == group) {
        if token.section != section {
            section = token.section;
            text()
                .content(format!("{group} / {section}"))
                .style(type_style(14.0, 20.0))
                .color(scheme.on_surface_variant)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .background(scheme.surface_container)
                        .padding_symmetric(Dp(32.0), Dp(10.0)),
                );
        }
        let (name, value) = (token.name, token.value);
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_symmetric(Dp(24.0), Dp(12.0)),
            )
            .children(move || {
                text()
                    .content(name)
                    .style(type_style(14.0, 20.0))
                    .color(INK)
                    .modifier(Modifier::new().weight(1.0));
                spacer().modifier(Modifier::new().width(Dp(16.0)));
                let swatch = parse_hex(value);
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size_in(Some(Dp(160.0)), None, Some(Dp(40.0)), None)
                            .background_with_shape(
                                swatch.unwrap_or(scheme.surface_container_low),
                                Shape::rounded_rectangle(Dp(8.0)),
                            )
                            .border_with_shape(
                                Dp(1.0),
                                scheme.outline_variant,
                                Shape::rounded_rectangle(Dp(8.0)),
                            )
                            .padding_symmetric(Dp(12.0), Dp(8.0)),
                    )
                    .children(move || {
                        text().content(value).style(type_style(14.0, 20.0)).color(
                            if swatch.is_some_and(|color| {
                                0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b < 0.25
                            }) {
                                Color::WHITE
                            } else {
                                INK
                            },
                        );
                    });
            });
    }
}

#[tessera]
fn variant_specs(modal: bool) {
    let name = if modal { "Modal" } else { "Standard" };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            figures::anatomy(modal, if modal { "specs-modal" } else { "specs-standard" });
            gap(Dp(24.0));
            legend(figures::anatomy_legend(modal));
            gap(Dp(56.0));
            subheading(format!("{name} side sheet color"));
            gap(Dp(16.0));
            prose(
                "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
            );
            gap(Dp(24.0));
            figures::color_roles(modal);
            gap(Dp(8.0));
            caption("Side sheet color roles used for light and dark themes:");
            gap(Dp(16.0));
            legend(figures::color_legend(modal));
            gap(Dp(56.0));
            subheading(format!("{name} side sheet measurements"));
            gap(Dp(24.0));
            figures::measurement_figure(modal);
            gap(Dp(16.0));
            caption(if modal {
                "Modal side sheet padding and size measurements"
            } else {
                "Side sheet padding and size measurements"
            });
            gap(Dp(32.0));
            measurements(modal);
        });
}

#[tessera]
fn measurements(modal: bool) {
    let mut rows: Vec<Vec<String>> = vec![vec!["Start/end padding".into(), "24dp".into()]];
    if modal {
        rows.push(vec!["Start padding with icon".into(), "16dp".into()]);
    }
    rows.extend([
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
    ]);
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[tessera]
fn usage(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    split()
        .start(|| {
            prose(
                "Standard side sheets are supplementary surfaces used mostly in medium to expanded breakpoints, like tablet and desktop. They provide a consistent and predictable surface for contextual actions and information.",
            );
            gap(Dp(16.0));
            prose(
                "Standard side sheets display content that complements the screen's primary content. They remain visible while people interact with primary content.",
            );
            gap(Dp(16.0));
            prose("Common uses include:");
            gap(Dp(8.0));
            bullets(vec![
                "Displaying a list of actions that affect the screen's primary content, such as filters"
                    .into(),
                "Displaying supplemental content and features".into(),
            ]);
        })
        .end(|| {
            figures::usage_figure(false);
            gap(Dp(12.0));
            caption("Information about a photo in a standard side sheet");
        });
    gap(Dp(40.0));
    split()
        .start(move || {
            prose(
                "Modal side sheets are preferred in compact breakpoints, like mobile, due to limited screen size.",
            );
            gap(Dp(16.0));
            prose(
                "They can display the same kinds of content as standard side sheets, but must be dismissed in order to interact with the underlying content.",
            );
            gap(Dp(24.0));
            launch(modal_open).session_optional(session);
        })
        .end(|| {
            figures::usage_figure(true);
            gap(Dp(12.0));
            caption("Modal side sheet with filter controls");
        });
    gap(Dp(40.0));
    prose("Side sheets have a fixed width and typically span the height of the screen.");
    gap(Dp(12.0));
    prose("Their dimensions depend on how the app's layout is subdivided into UI regions.");
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(
                true,
                "Place side sheets along the edge of the screen, usually on the right side to avoid interference with any navigational components on the left edge. They can be slightly inset by 16dp.",
            )
            .figure(|| {
                figures::inset_figure(16.0);
            });
        })
        .end(|| {
            guidance_card(
                false,
                "Don't inset a side sheet from the screen edges far beyond the recommended margin. This makes the sheet's position and scroll behavior unclear, while obscuring primary content.",
            )
            .figure(|| {
                figures::inset_figure(56.0);
            });
        });
}

#[tessera]
fn element_section(
    #[prop(into)] title: String,
    #[prop(into)] body: String,
    bullets_after: Option<Vec<String>>,
    #[prop(into)] element: String,
    #[prop(into)] figure_caption: String,
) {
    gap(Dp(48.0));
    split()
        .start(move || {
            subheading(title.clone());
            gap(Dp(12.0));
            prose(body.clone());
            if let Some(items) = bullets_after.clone() {
                gap(Dp(12.0));
                bullets(items);
            }
        })
        .end(move || {
            figures::element_figure(element.clone());
            gap(Dp(12.0));
            caption(figure_caption.clone());
        });
}

#[tessera]
fn anatomy_guidance() {
    figures::anatomy(false, "guidelines-standard");
    gap(Dp(24.0));
    legend(figures::anatomy_legend(false));
    gap(Dp(40.0));
    figures::anatomy(true, "guidelines-modal");
    gap(Dp(24.0));
    legend(figures::anatomy_legend(true));
    element_section(
        "Container",
        "Side sheet containers hold all side sheet elements. Their size is determined by the space those elements occupy. The container is the only required element of a side sheet.",
        "container",
        "Container",
    );
    element_section(
        "Back icon button (optional)",
        "Icon buttons can provide ways to exit a side sheet or move to a different experience. Because the primary content behind or beside a side sheet is always visible, it's important to provide affordances for leaving a side sheet and returning to the primary content.",
        "back",
        "Back icon button",
    );
    element_section(
        "Close icon button (optional)",
        "A close affordance provides a consistent method for dismissing a side sheet. A close icon button is highly recommended, increases accessibility, and makes focused side sheets easier to close.",
        "close",
        "Close icon button",
    );
    element_section(
        "Action buttons (optional)",
        "Buttons represent actions available from a side sheet. Examples: Save, Edit, Download. Use elevation, fill, and tone to call attention to specific actions.",
        "actions",
        "Action buttons",
    );
    element_section(
        "Divider (optional)",
        "Dividers can separate different kinds of content and create distinct regions in a side sheet. Use a divider to separate:",
        "divider",
        "Divider",
    )
    .bullets_after(vec![
        "Action buttons from content".into(),
        "User-generated content from system-generated content".into(),
    ]);
    gap(Dp(48.0));
    subheading("Content (optional)".into());
    gap(Dp(12.0));
    prose(
        "Side sheets can display a wide variety of content and layouts, ranging from a list of actions to supplemental content in a tabular layout.",
    );
    gap(Dp(24.0));
    figures::content_figure();
    gap(Dp(12.0));
    caption("Form controls shown in a side sheet for app settings");
    gap(Dp(40.0));
    figures::transition_figure();
    gap(Dp(12.0));
    caption(
        "Modal side sheets on smaller screens can transition to standard side sheets at larger screen sizes",
    );
}

#[tessera]
fn adaptive_design(standard_open: State<bool>) {
    prose(
        "Side sheets have a default width, but can be resized depending on the needs of the layout.",
    );
    gap(Dp(16.0));
    prose(
        "When a standard side sheet opens, the body area shrinks to accommodate the sheet's width while maintaining a margin on the body's trailing edge.",
    );
    gap(Dp(24.0));
    standard_demo(standard_open);
    gap(Dp(12.0));
    caption(
        "Entrance of standard side sheets will cause the body area to adjust and accommodate the new content",
    );
    gap(Dp(48.0));
    split()
        .start(|| {
            subheading("RTL language support".into());
            gap(Dp(12.0));
            prose(
                "In right-to-left (RTL) languages, side sheets should appear on the left edge of the window with all elements reversed.",
            );
        })
        .end(|| {
            figures::rtl_figure();
            gap(Dp(12.0));
            caption("Side sheet elements are reversed in RTL languages");
        });
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
                .main_content(move || {
                    standard_body(open);
                })
                .side_sheet_content(move || {
                    standard_content(open);
                });
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
fn sheet_header(
    #[prop(into)] title: String,
    #[prop(into)] close_label: String,
    on_close: State<bool>,
    announce_open: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let announce_open = announce_open.unwrap_or(false);
    let headline = remember(FocusRequester::new).get();
    let announced = remember(|| false);
    let headline_focused = remember(|| false);
    if announce_open {
        // The headline receives focus once per opening so assistive technology
        // announces the sheet, while Tab traversal still starts at Close.
        if on_close.get() && !announced.get() {
            announced.set(true);
            headline.request_focus();
        } else if !on_close.get() && announced.get() {
            announced.set(false);
        }
    }
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
            Dp(24.0),
            Dp(12.0),
            Dp(12.0),
            Dp(12.0),
        )))
        .children(move || {
            let heading = Modifier::new().semantics(SemanticsArgs {
                role: Some(Role::Heading),
                heading_level: Some(2),
                ..Default::default()
            });
            let heading = if announce_open {
                heading
                    .focus_requester(headline)
                    .focusable()
                    .focus_properties(FocusProperties::new().skip_traversal(true))
                    .on_focus_changed(move |focus: FocusState| {
                        if headline_focused.is_alive()
                            && headline_focused.get() != focus.is_focused()
                        {
                            headline_focused.set(focus.is_focused());
                        }
                    })
            } else {
                heading
            };
            let title = title.clone();
            boxed()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    let ring = if headline_focused.get() {
                        scheme.secondary
                    } else {
                        Color::TRANSPARENT
                    };
                    text()
                        .content(title.clone())
                        .style(type_style(22.0, 28.0))
                        .color(scheme.on_surface_variant)
                        .modifier(heading.clone().border_with_shape(
                            Dp(3.0),
                            ring,
                            Shape::rounded_rectangle(Dp(4.0)),
                        ));
                });
            icon_button()
                .standard()
                .icon(filled::CLOSE_SVG)
                .accessibility_label(close_label.clone())
                .on_click(move || on_close.set(false));
        });
}

#[tessera]
fn standard_content(open: State<bool>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_size().background(scheme.surface))
        .children(move || {
            sheet_header("Info", "Close standard side sheet", open);
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                )
                .children(move || {
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
        });
}

#[tessera]
fn behavior(modal_open: State<bool>, notice: State<String>, session: Option<LiveSideSheetSession>) {
    prose("Side sheets can vertically scroll independent of the rest of the UI.");
    gap(Dp(12.0));
    prose(
        "This allows their scroll position and content to persist while the page is scrolled, and vice versa.",
    );
    gap(Dp(12.0));
    prose("Side sheets cannot scroll horizontally.");
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(
                true,
                "Side sheets can vertically scroll internally when their content exceeds the screen height",
            )
            .figure(|| {
                figures::scroll_figure(true);
            });
        })
        .end(|| {
            guidance_card(
                false,
                "Don't allow horizontal scrolling or lay out the side sheet in a way that suggests horizontal scrolling. A side sheet's narrow width leaves limited space to fully view items.",
            )
            .figure(|| {
                figures::scroll_figure(false);
            });
        });
    gap(Dp(32.0));
    launch(modal_open).session_optional(session);
    gap(Dp(16.0));
    caption(notice.get());
    gap(Dp(48.0));
    subheading("Predictive back".into());
    gap(Dp(12.0));
    prose(
        "On Android, a gesture called predictive back allows a person to swipe left or right on the side sheet.",
    );
    gap(Dp(12.0));
    prose("When predictive back is used:");
    gap(Dp(8.0));
    bullets(vec![
        "The side sheet detaches from the top and bottom edges of the screen to signal it will close"
            .into(),
        "The previous screen is revealed in a preview".into(),
        "The side sheet and its content always scales in the direction of the gesture".into(),
    ]);
    gap(Dp(24.0));
    figures::predictive_back_figure();
    gap(Dp(12.0));
    caption(
        "Preview of the result of the gestures: release to commit, fling to commit, and cancel",
    );
}

#[tessera]
fn use_cases(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    prose("People should be able to dismiss the side sheet using assistive technology.");
    gap(Dp(24.0));
    launch(modal_open).session_optional(session);
}

#[tessera]
fn interaction_style() {
    prose(
        "Material requires that a close affordance, such as a close icon button, is always present within a side sheet.",
    );
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(true, "A close icon button makes the side sheet easy to dismiss").figure(
                || {
                    figures::close_figure(true);
                },
            );
        })
        .end(|| {
            guidance_card(
                false,
                "Without a close icon button, people can't predict the opening and closing flow of side sheets, or know if the sheet is transient or permanent",
            )
            .figure(|| {
                figures::close_figure(false);
            });
        });
}

#[tessera]
fn initial_focus(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    prose(
        "Actions within a side sheet can be focused by tab order using a keyboard or switch control.",
    );
    gap(Dp(16.0));
    prose(
        "In the live example, opening the sheet moves focus to its Filters headline so the sheet is announced. Tab then reaches Close, the label checkboxes, Save and Cancel in reading order; the headline is not a separate Tab stop.",
    );
    gap(Dp(24.0));
    figures::focus_figure();
    gap(Dp(12.0));
    caption("Visible focus shown on the available actions within a side sheet:");
    gap(Dp(16.0));
    legend(vec![
        (1, "Headline".into()),
        (2, "Close".into()),
        (3, "Save".into()),
        (4, "Cancel".into()),
    ]);
    gap(Dp(24.0));
    launch(modal_open).session_optional(session);
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab".into(),
                "Focus lands on (non-disabled) icon button".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Activates the (non-disabled) icon button".into(),
            ],
        ],
    );
}

#[tessera]
fn labeling() {
    prose("The accessibility role for a side sheet is Dialog.");
    gap(Dp(24.0));
    figures::role_figure();
    gap(Dp(12.0));
    caption("The role for side sheets is Dialog");
}

#[tessera]
fn sheet_content(modal_open: State<bool>, filters: State<[bool; 5]>, notice: State<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.surface_container_low)
                .semantics(SemanticsArgs {
                    test_tag: Some("side-sheet-live-content".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            sheet_header("Filters", "Close side sheet", modal_open).announce_open(true);
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                )
                .children(move || {
                    text()
                        .content("Labels")
                        .style(type_style(14.0, 20.0))
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
                                text().content(label).style(type_style(16.0, 24.0));
                            });
                    }
                });
            boxed().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(1.0))
                    .background(scheme.outline_variant),
            );
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(72.0))
                        .padding(Padding::new(Dp(24.0), Dp(16.0), Dp(24.0), Dp(24.0))),
                )
                .children(move || {
                    button()
                        .filled()
                        .size(ButtonSize::ExtraSmall)
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
                        .outlined()
                        .size(ButtonSize::ExtraSmall)
                        .accessibility_label("Cancel filters")
                        .on_click(move || modal_open.set(false))
                        .child(|| {
                            text().content("Cancel");
                        });
                });
        });
}
