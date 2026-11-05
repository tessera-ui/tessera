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

use crate::res::I18N;

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
    const ITEMS: [&'static str; 5] = [
        "ui-events",
        "ui-personal",
        "ui-projects",
        "ui-reminders",
        "ui-family",
    ];

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
        notice: remember(|| String::from("ui-no-action-selected")),
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
        .accessibility_label(I18N.text("ui-filters"))
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
    let local_notice = remember(|| String::from("ui-no-action-selected"));
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
        "ui-side-sheets",
        "ui-side-sheets-show-secondary-content-anchored-to-the-side-of-the-screen",
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-standard-side-sheet",
                content: RenderSlot::new(move || {
                    variant_specs(false);
                }),
            },
            SpecSection {
                title: "ui-modal-side-sheet",
                content: RenderSlot::new(move || {
                    variant_specs(true);
                }),
            },
        ],
    )
    .route_optional(route)
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(move || {
                overview_intro(modal_open).session_optional(session);
            })),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(move || {
                figures::hero();
                gap(Dp(24.0));
                variants_legend();
            })),
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(move || {
                        usage(modal_open).session_optional(session);
                    }),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(anatomy_guidance),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(move || {
                        adaptive_design(standard_open);
                    }),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(move || {
                        behavior(modal_open, notice).session_optional(session);
                    }),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(move || {
                        use_cases(modal_open).session_optional(session);
                    }),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "ui-initial-focus",
                    content: RenderSlot::new(move || {
                        initial_focus(modal_open).session_optional(session);
                    }),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "ui-labeling",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {}", I18N.text(item)))
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn variants_legend() {
    legend(vec![
        (1, "ui-standard-side-sheet".into()),
        (2, "ui-modal-side-sheet".into()),
    ]);
}

#[tessera]
fn launch(open: State<bool>, session: Option<LiveSideSheetSession>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label(I18N.text("ui-open-modal-side-sheet"))
                .on_click(move || {
                    if session.is_none_or(LiveSideSheetSession::can_open) {
                        open.set(true);
                    }
                })
                .child(move || {
                    text().content(I18N.text("ui-open-modal-side-sheet"));
                });
            gap(Dp(16.0));
            caption(
                "ui-escape-the-scrim-and-the-close-icon-button-all-dismiss-the-modal-side-sheet",
            );
        });
}

#[tessera]
fn overview_intro(open: State<bool>, session: Option<LiveSideSheetSession>) {
    bullets(vec![
        "ui-use-side-sheets-to-provide-optional-content-and-actions-without-interrupting-the-main-cont"
            .into(),
        "ui-two-variants-standard-and-modal".into(),
        "ui-people-can-navigate-to-another-region-within-the-sheet".into(),
        "ui-side-sheets-can-contain-a-back-icon-for-navigation".into(),
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
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "ui-design-kit-figma".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-flutter".into(),
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-jetpack-compose".into(),
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-android-views-mdc-android".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
}

#[tessera]
fn differences() {
    split()
        .start_weight(0.5)
        .start(|| {
            bullets(vec![
                "ui-right-to-left-rtl-language-support-with-left-side-sheet".into(),
                "ui-color-new-color-mappings-and-compatibility-with-dynamic-color-2123".into(),
                "ui-shape-modal-side-sheets-have-a-16dp-corner-radius".into(),
            ]);
        })
        .end(|| {
            figures::differences_figure();
            gap(Dp(16.0));
            caption("ui-side-sheets-have-new-color-mappings-to-support-dynamic-color");
        });
}

struct Token {
    group: &'static str,
    section: &'static str,
    name: &'static str,
    value: &'static str,
}

const TOKEN_GROUPS: [&str; 4] = [
    "ui-enabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed-ripple",
];

const TOKENS: [Token; 31] = [
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-container-color",
        value: "#FEF7FF",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-modal-container-color",
        value: "#F7F2FA",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-standard-container-color",
        value: "#FEF7FF",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-container-surface-tint-layer-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-modal-container-elevation",
        value: "ui-level-1",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-standard-container-elevation",
        value: "ui-level-0",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-container-height",
        value: "100%",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-container-shape",
        value: "ui-square-corners",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-detached-container-shape",
        value: "ui-rounded-corners",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-container-width",
        value: "256dp",
    },
    Token {
        group: "ui-enabled",
        section: "ui-container",
        name: "ui-sheet-side-docked-modal-container-shape",
        value: "ui-rounded-corners",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-color",
        value: "#49454F",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-font",
        value: "ui-roboto",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-line-height",
        value: "28pt",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-size",
        value: "22pt",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-tracking",
        value: "0pt",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-weight",
        value: "400",
    },
    Token {
        group: "ui-enabled",
        section: "ui-headline",
        name: "ui-sheet-side-docked-headline-type-style",
        value: "ui-title-large",
    },
    Token {
        group: "ui-enabled",
        section: "ui-divider",
        name: "ui-sheet-side-docked-divider-color",
        value: "#79747E",
    },
    Token {
        group: "ui-hovered",
        section: "ui-label-text",
        name: "ui-sheet-side-docked-action-hover-label-text-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-hovered",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-hover-state-layer-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-hovered",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-hover-state-layer-opacity",
        value: "0.08",
    },
    Token {
        group: "ui-focused",
        section: "ui-focus-indicator",
        name: "ui-sheet-side-docked-focus-indicator-color",
        value: "#625B71",
    },
    Token {
        group: "ui-focused",
        section: "ui-focus-indicator",
        name: "ui-sheet-side-docked-focus-indicator-thickness",
        value: "3dp",
    },
    Token {
        group: "ui-focused",
        section: "ui-focus-indicator",
        name: "ui-sheet-side-docked-focus-indicator-offset",
        value: "2dp",
    },
    Token {
        group: "ui-focused",
        section: "ui-label-text",
        name: "ui-sheet-side-docked-action-focus-label-text-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-focused",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-focus-state-layer-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-focused",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-focus-state-layer-opacity",
        value: "0.1",
    },
    Token {
        group: "ui-pressed-ripple",
        section: "ui-label-text",
        name: "ui-sheet-side-docked-action-pressed-label-text-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-pressed-ripple",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-pressed-state-layer-color",
        value: "#6750A4",
    },
    Token {
        group: "ui-pressed-ripple",
        section: "ui-state-layer",
        name: "ui-sheet-side-docked-action-pressed-state-layer-opacity",
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
    prose("ui-browse-the-component-elements-attributes-tokens-and-their-values");
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
                                .content(I18N.text("ui-sheets-side"))
                                .style(type_style(14.0, 20.0))
                                .color(scheme.on_secondary_container);
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    let all_open = expanded.with(|state| state.iter().all(|open| *open));
                    button()
                        .text()
                        .accessibility_label(I18N.text(if all_open {
                            "ui-collapse-all-token-groups"
                        } else {
                            "ui-expand-all-token-groups"
                        }))
                        .on_click(move || {
                            expanded.with_mut(|state| {
                                let next = !state.iter().all(|open| *open);
                                state.fill(next);
                            });
                        })
                        .child(move || {
                            text().content(I18N.text(if all_open {
                                "ui-collapse-all"
                            } else {
                                "ui-expand-all"
                            }));
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
                        .content(I18N.text("ui-token"))
                        .style(type_style(14.0, 20.0))
                        .color(INK)
                        .modifier(Modifier::new().weight(1.0));
                    text()
                        .content(I18N.text("ui-default-light"))
                        .style(type_style(14.0, 20.0))
                        .color(scheme.on_surface_variant);
                });
            for (index, group) in TOKEN_GROUPS.into_iter().enumerate() {
                let open = expanded.with(|state| state[index]);
                button()
                    .text()
                    .accessibility_label(
                        I18N.format("ui-value-tokens", [("v0", I18N.text(&group).into())]),
                    )
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
                                    .content(I18N.text(group))
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
                .content(I18N.format(
                    "ui-value-value-format-31",
                    [
                        ("v0", I18N.text(&group).into()),
                        ("v1", I18N.text(&section).into()),
                    ],
                ))
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
                    .content(I18N.text(name))
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
                        text()
                            .content(I18N.text(value))
                            .style(type_style(14.0, 20.0))
                            .color(
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
    let name = if modal { "ui-modal" } else { "ui-standard" };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            figures::anatomy(modal, if modal { "specs-modal" } else { "specs-standard" });
            gap(Dp(24.0));
            legend(figures::anatomy_legend(modal));
            gap(Dp(56.0));
            subheading(I18N.format("ui-value-side-sheet-color", [("v0", I18N.text(&name).into())]));
            gap(Dp(16.0));
            prose(
                "ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo",
            );
            gap(Dp(24.0));
            figures::color_roles(modal);
            gap(Dp(8.0));
            caption("ui-side-sheet-color-roles-used-for-light-and-dark-themes");
            gap(Dp(16.0));
            legend(figures::color_legend(modal));
            gap(Dp(56.0));
            subheading(I18N.format("ui-value-side-sheet-measurements", [("v0", I18N.text(&name).into())]));
            gap(Dp(24.0));
            figures::measurement_figure(modal);
            gap(Dp(16.0));
            caption(if modal {
                "ui-modal-side-sheet-padding-and-size-measurements"
            } else {
                "ui-side-sheet-padding-and-size-measurements"
            });
            gap(Dp(32.0));
            measurements(modal);
        });
}

#[tessera]
fn measurements(modal: bool) {
    let mut rows: Vec<Vec<String>> = vec![vec!["ui-start-end-padding".into(), "24dp".into()]];
    if modal {
        rows.push(vec!["ui-start-padding-with-icon".into(), "16dp".into()]);
    }
    rows.extend([
        vec!["ui-padding-between-top-elements".into(), "12dp".into()],
        vec!["ui-bottom-actions-height".into(), "72dp".into()],
        vec!["ui-bottom-actions-top-padding".into(), "16dp".into()],
        vec!["ui-bottom-actions-bottom-padding".into(), "24dp".into()],
        vec![
            "ui-bottom-actions-alignment-horizontal".into(),
            "ui-left".into(),
        ],
        vec!["ui-max-width".into(), "400dp".into()],
        vec!["ui-margins-when-detached".into(), "16dp".into()],
    ]);
    spec_table(vec!["ui-attribute".into(), "ui-value".into()], rows);
}

#[tessera]
fn usage(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    split()
        .start(|| {
            prose(
                "ui-standard-side-sheets-are-supplementary-surfaces-used-mostly-in-medium-to-expanded-breakpoi",
            );
            gap(Dp(16.0));
            prose(
                "ui-standard-side-sheets-display-content-that-complements-the-screen-s-primary-content-they-re",
            );
            gap(Dp(16.0));
            prose("ui-common-uses-include");
            gap(Dp(8.0));
            bullets(vec![
                "ui-displaying-a-list-of-actions-that-affect-the-screen-s-primary-content-such-as-filters"
                    .into(),
                "ui-displaying-supplemental-content-and-features".into(),
            ]);
        })
        .end(|| {
            figures::usage_figure(false);
            gap(Dp(12.0));
            caption("ui-information-about-a-photo-in-a-standard-side-sheet");
        });
    gap(Dp(40.0));
    split()
        .start(move || {
            prose(
                "ui-modal-side-sheets-are-preferred-in-compact-breakpoints-like-mobile-due-to-limited-screen-s",
            );
            gap(Dp(16.0));
            prose(
                "ui-they-can-display-the-same-kinds-of-content-as-standard-side-sheets-but-must-be-dismissed-i",
            );
            gap(Dp(24.0));
            launch(modal_open).session_optional(session);
        })
        .end(|| {
            figures::usage_figure(true);
            gap(Dp(12.0));
            caption("ui-modal-side-sheet-with-filter-controls");
        });
    gap(Dp(40.0));
    prose("ui-side-sheets-have-a-fixed-width-and-typically-span-the-height-of-the-screen");
    gap(Dp(12.0));
    prose("ui-their-dimensions-depend-on-how-the-app-s-layout-is-subdivided-into-ui-regions");
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(
                true,
                "ui-place-side-sheets-along-the-edge-of-the-screen-usually-on-the-right-side-to-avoid-interfer",
            )
            .figure(|| {
                figures::inset_figure(16.0);
            });
        })
        .end(|| {
            guidance_card(
                false,
                "ui-don-t-inset-a-side-sheet-from-the-screen-edges-far-beyond-the-recommended-margin-this-make",
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
        "ui-container",
        "ui-side-sheet-containers-hold-all-side-sheet-elements-their-size-is-determined-by-the-space-t",
        "container",
        "ui-container",
    );
    element_section(
        "ui-back-icon-button-optional",
        "ui-icon-buttons-can-provide-ways-to-exit-a-side-sheet-or-move-to-a-different-experience-becau",
        "back",
        "ui-back-icon-button",
    );
    element_section(
        "ui-close-icon-button-optional",
        "ui-a-close-affordance-provides-a-consistent-method-for-dismissing-a-side-sheet-a-close-icon-b",
        "close",
        "ui-close-icon-button",
    );
    element_section(
        "ui-action-buttons-optional",
        "ui-buttons-represent-actions-available-from-a-side-sheet-examples-save-edit-download-use-elev",
        "actions",
        "ui-action-buttons",
    );
    element_section(
        "ui-divider-optional",
        "ui-dividers-can-separate-different-kinds-of-content-and-create-distinct-regions-in-a-side-she",
        "divider",
        "ui-divider",
    )
    .bullets_after(vec![
        "ui-action-buttons-from-content".into(),
        "ui-user-generated-content-from-system-generated-content".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-content-optional".into());
    gap(Dp(12.0));
    prose(
        "ui-side-sheets-can-display-a-wide-variety-of-content-and-layouts-ranging-from-a-list-of-actio",
    );
    gap(Dp(24.0));
    figures::content_figure();
    gap(Dp(12.0));
    caption("ui-form-controls-shown-in-a-side-sheet-for-app-settings");
    gap(Dp(40.0));
    figures::transition_figure();
    gap(Dp(12.0));
    caption(
        "ui-modal-side-sheets-on-smaller-screens-can-transition-to-standard-side-sheets-at-larger-scre",
    );
}

#[tessera]
fn adaptive_design(standard_open: State<bool>) {
    prose(
        "ui-side-sheets-have-a-default-width-but-can-be-resized-depending-on-the-needs-of-the-layout",
    );
    gap(Dp(16.0));
    prose(
        "ui-when-a-standard-side-sheet-opens-the-body-area-shrinks-to-accommodate-the-sheet-s-width-wh",
    );
    gap(Dp(24.0));
    standard_demo(standard_open);
    gap(Dp(12.0));
    caption(
        "ui-entrance-of-standard-side-sheets-will-cause-the-body-area-to-adjust-and-accommodate-the-ne",
    );
    gap(Dp(48.0));
    split()
        .start(|| {
            subheading("ui-rtl-language-support".into());
            gap(Dp(12.0));
            prose(
                "ui-in-right-to-left-rtl-languages-side-sheets-should-appear-on-the-left-edge-of-the-window-wi",
            );
        })
        .end(|| {
            figures::rtl_figure();
            gap(Dp(12.0));
            caption("ui-side-sheet-elements-are-reversed-in-rtl-languages");
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
                .accessibility_label(I18N.text("ui-information"))
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
                        .content(I18N.text("ui-primary-content"))
                        .style(type_style(18.0, 24.0))
                        .color(INK);
                    gap(Dp(12.0));
                    button()
                        .outlined()
                        .accessibility_label(I18N.text("ui-open-standard-side-sheet"))
                        .on_click(move || open.set(true))
                        .child(move || {
                            text().content(I18N.text("ui-open-standard-side-sheet"));
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
                        .content(I18N.text(&(title)))
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
                .accessibility_label(I18N.text(&(close_label)))
                .on_click(move || on_close.set(false));
        });
}

#[tessera]
fn standard_content(open: State<bool>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_size().background(scheme.surface))
        .children(move || {
            sheet_header("ui-info", "ui-close-standard-side-sheet", open);
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                )
                .children(move || {
                    text()
                        .content(I18N.text("ui-festival-concert"))
                        .style(type_style(16.0, 24.0))
                        .color(INK);
                    gap(Dp(8.0));
                    caption("ui-albums");
                    gap(Dp(8.0));
                    text()
                        .content(I18N.text("ui-home-space"))
                        .style(type_style(14.0, 20.0))
                        .color(INK);
                    text()
                        .content(I18N.text("ui-64-items"))
                        .style(type_style(12.0, 16.0))
                        .color(scheme.on_surface_variant);
                });
        });
}

#[tessera]
fn behavior(modal_open: State<bool>, notice: State<String>, session: Option<LiveSideSheetSession>) {
    prose("ui-side-sheets-can-vertically-scroll-independent-of-the-rest-of-the-ui");
    gap(Dp(12.0));
    prose(
        "ui-this-allows-their-scroll-position-and-content-to-persist-while-the-page-is-scrolled-and-vi",
    );
    gap(Dp(12.0));
    prose("ui-side-sheets-cannot-scroll-horizontally");
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(
                true,
                "ui-side-sheets-can-vertically-scroll-internally-when-their-content-exceeds-the-screen-height",
            )
            .figure(|| {
                figures::scroll_figure(true);
            });
        })
        .end(|| {
            guidance_card(
                false,
                "ui-don-t-allow-horizontal-scrolling-or-lay-out-the-side-sheet-in-a-way-that-suggests-horizont",
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
    subheading("ui-predictive-back".into());
    gap(Dp(12.0));
    prose(
        "ui-on-android-a-gesture-called-predictive-back-allows-a-person-to-swipe-left-or-right-on-the",
    );
    gap(Dp(12.0));
    prose("ui-when-predictive-back-is-used");
    gap(Dp(8.0));
    bullets(vec![
        "ui-the-side-sheet-detaches-from-the-top-and-bottom-edges-of-the-screen-to-signal-it-will-clos"
            .into(),
        "ui-the-previous-screen-is-revealed-in-a-preview".into(),
        "ui-the-side-sheet-and-its-content-always-scales-in-the-direction-of-the-gesture".into(),
    ]);
    gap(Dp(24.0));
    figures::predictive_back_figure();
    gap(Dp(12.0));
    caption(
        "ui-preview-of-the-result-of-the-gestures-release-to-commit-fling-to-commit-and-cancel",
    );
}

#[tessera]
fn use_cases(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    prose("ui-people-should-be-able-to-dismiss-the-side-sheet-using-assistive-technology");
    gap(Dp(24.0));
    launch(modal_open).session_optional(session);
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-material-requires-that-a-close-affordance-such-as-a-close-icon-button-is-always-present-wi",
    );
    gap(Dp(24.0));
    split()
        .start(|| {
            guidance_card(true, "ui-a-close-icon-button-makes-the-side-sheet-easy-to-dismiss").figure(
                || {
                    figures::close_figure(true);
                },
            );
        })
        .end(|| {
            guidance_card(
                false,
                "ui-without-a-close-icon-button-people-can-t-predict-the-opening-and-closing-flow-of-side-shee",
            )
            .figure(|| {
                figures::close_figure(false);
            });
        });
}

#[tessera]
fn initial_focus(modal_open: State<bool>, session: Option<LiveSideSheetSession>) {
    prose(
        "ui-actions-within-a-side-sheet-can-be-focused-by-tab-order-using-a-keyboard-or-switch-control",
    );
    gap(Dp(16.0));
    prose(
        "ui-in-the-live-example-opening-the-sheet-moves-focus-to-its-filters-headline-so-the-sheet-is",
    );
    gap(Dp(24.0));
    figures::focus_figure();
    gap(Dp(12.0));
    caption("ui-visible-focus-shown-on-the-available-actions-within-a-side-sheet");
    gap(Dp(16.0));
    legend(vec![
        (1, "ui-headline".into()),
        (2, "ui-close".into()),
        (3, "ui-save".into()),
        (4, "ui-cancel".into()),
    ]);
    gap(Dp(24.0));
    launch(modal_open).session_optional(session);
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-focus-lands-on-non-disabled-icon-button".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activates-the-non-disabled-icon-button".into(),
            ],
        ],
    );
}

#[tessera]
fn labeling() {
    prose("ui-the-accessibility-role-for-a-side-sheet-is-dialog");
    gap(Dp(24.0));
    figures::role_figure();
    gap(Dp(12.0));
    caption("ui-the-role-for-side-sheets-is-dialog");
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
            sheet_header("ui-filters", "ui-close-side-sheet", modal_open).announce_open(true);
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .weight(1.0)
                        .padding_symmetric(Dp(24.0), Dp::ZERO),
                )
                .children(move || {
                    text()
                        .content(I18N.text("ui-labels"))
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
                                    .accessibility_label(I18N.text(label));
                                spacer().modifier(Modifier::new().width(Dp(16.0)));
                                text()
                                    .content(I18N.text(label))
                                    .style(type_style(16.0, 24.0));
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
                        .accessibility_label(I18N.text("ui-save-filters"))
                        .on_click(move || {
                            notice.set(String::from("ui-filters-applied"));
                            modal_open.set(false);
                        })
                        .child(move || {
                            text().content(I18N.text("ui-save"));
                        });
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    button()
                        .outlined()
                        .size(ButtonSize::ExtraSmall)
                        .accessibility_label(I18N.text("ui-cancel-filters"))
                        .on_click(move || modal_open.set(false))
                        .child(move || {
                            text().content(I18N.text("ui-cancel"));
                        });
                });
        });
}
