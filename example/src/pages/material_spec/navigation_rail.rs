//! Navigation rail specifications for primary destinations.
//!
//! ## Usage
//!
//! Compare collapsed and expanded rails, selection and rail composition.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::{filled, outlined},
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_rail::{modal_navigation_rail, navigation_rail, navigation_rail_item},
    row::row,
    scrollable::ScrollableController,
    shape_def::Shape,
    side_sheet::SideSheetController,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_shard::RouterController;
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap,
    navigation::{NAMES, navigation_color_roles},
    panel, spec_table, srgb, subheading, type_style,
};

mod figures;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LiveRailSession {
    open: State<bool>,
    selected: State<usize>,
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
}

impl LiveRailSession {
    fn route_active(self) -> bool {
        self.router.with(|router| {
            router
                .current_is::<crate::pages::material_components::MaterialComponentsPageDestination>(
                )
        }) && super::rail_article_selected(self.material.get())
    }

    fn can_open(self) -> bool {
        self.route_active()
            && self
                .directory
                .with(|directory| !directory.is_open() && !directory.is_animating())
    }
}

#[tessera]
pub(crate) fn live_rail_host(
    directory: State<SideSheetController>,
    router: State<RouterController>,
    material: State<Option<usize>>,
    main_content: tessera_ui::RenderSlotWith<LiveRailSession>,
) {
    let open = remember(|| false);
    let selected = remember(|| 0usize);
    let session = LiveRailSession {
        open,
        selected,
        directory,
        router,
        material,
    };
    if !session.route_active() && open.get() {
        open.set(false);
    }
    let modal = RenderSlot::new(move || {
        live_rail_modal(
            open,
            selected,
            RenderSlot::new(move || {
                main_content.render(session);
            }),
        );
    });
    modal.render();
}

#[tessera]
fn live_rail_modal(
    modal_open: State<bool>,
    modal_selected: State<usize>,
    main_content: RenderSlot,
) {
    modal_navigation_rail()
        .is_open(modal_open.get())
        .accessibility_label("Mail rail navigation")
        .on_close_request(move || modal_open.set(false))
        .main_content_shared(main_content)
        .header(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label("Close modal navigation rail")
                .on_click(move || modal_open.set(false));
        })
        .content(move |rail_scope| {
            for (index, name) in NAMES.into_iter().enumerate() {
                navigation_rail_item(rail_scope)
                    .label(name)
                    .selected(modal_selected.get() == index)
                    .on_click(move || {
                        modal_selected.set(index);
                        modal_open.set(false);
                    })
                    .icon(move || {
                        destination_icon(index, modal_selected.get() == index);
                    });
            }
        });
}

#[tessera]
pub(super) fn navigation_rail_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,

    session: Option<LiveRailSession>,
) {
    let root = session;
    let local_open = remember(|| false);
    let local_selected = remember(|| 0usize);
    let (modal_open, modal_selected) = if let Some(session) = root {
        (session.open, session.selected)
    } else {
        (local_open, local_selected)
    };
    let content = RenderSlot::new(move || {
        article(
            "Navigation rail",
            "Navigation rails let people switch between UI views on mid-sized devices",
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
                SpecSection {
                    title: "Common layouts",
                    content: RenderSlot::new(layouts),
                },
                SpecSection {
                    title: "Baseline navigation rail",
                    content: RenderSlot::new(baseline),
                },
            ],
        )
        .route_optional(route.clone())
        .pages(vec![
            ArticlePage {
                title: "Overview",
                intro: Some(RenderSlot::new(overview)),
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
                        title: "Placement",
                        content: RenderSlot::new(placement),
                    },
                    SpecSection {
                        title: "Adaptive design",
                        content: RenderSlot::new(adaptive_design),
                    },
                    SpecSection {
                        title: "Behavior",
                        content: RenderSlot::new(move || {
                            behavior(modal_open, modal_selected).session_optional(session);
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
                        title: "Text scaling and truncation",
                        content: RenderSlot::new(text_scaling),
                    },
                    SpecSection {
                        title: "Initial focus",
                        content: RenderSlot::new(initial_focus),
                    },
                    SpecSection {
                        title: "Visual indicators",
                        content: RenderSlot::new(visual_indicators),
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
    });
    if root.is_some() {
        content.render();
    } else {
        live_rail_modal(modal_open, modal_selected, content);
    }
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
fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            figures::number(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text().content(label.clone()).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn legend_list(items: Vec<String>) {
    column().children(move || {
        for (index, item) in items.iter().enumerate() {
            legend(index + 1, item.clone());
        }
    });
}

#[tessera]
fn destination_icon(index: usize, selected: bool) {
    icon()
        .painter(if selected {
            [
                filled::HOME_SVG,
                filled::MAIL_SVG,
                filled::BOOKMARK_SVG,
                filled::PLAY_ARROW_SVG,
                filled::SETTINGS_SVG,
            ][index]
        } else {
            [
                outlined::HOME_SVG,
                outlined::MAIL_SVG,
                outlined::BOOKMARK_BORDER_SVG,
                outlined::PLAY_ARROW_SVG,
                outlined::SETTINGS_SVG,
            ][index]
        })
        .size(Dp(24.0));
}

fn format_rail_status(label: &str, selected: usize) -> String {
    let name = NAMES.get(selected).copied().unwrap_or(NAMES[0]);
    format!("{label} destination: {name}")
}

#[tessera]
fn timer_screen(
    #[prop(into)] label: String,
    width: Dp,
    height: Dp,
    expanded: Option<bool>,
    count: Option<usize>,
    menu: Option<bool>,
    fab: Option<bool>,
    content_kind: Option<usize>,
    selected: Option<usize>,
) {
    let expanded = expanded.unwrap_or(false);
    let count = count.unwrap_or(4);
    let menu = menu.unwrap_or(true);
    let fab = fab.unwrap_or(false);
    let selected = selected.unwrap_or(1);
    figures::screen(label, width, height)
        .content_kind(content_kind.unwrap_or(0))
        .rail(move || {
            figures::mock_rail()
                .expanded(expanded)
                .count(count)
                .selected(selected)
                .menu(menu)
                .fab(fab);
        });
}

#[tessera]
fn card(
    #[prop(into)] label: String,
    tone: usize,
    #[prop(into)] note: String,
    rail: RenderSlot,
    width: Option<Dp>,
    trailing: Option<bool>,
    content_kind: Option<usize>,
    bar: Option<usize>,
    hidden: Option<bool>,
    dark: Option<bool>,
) {
    let width = width.unwrap_or(Dp(360.0));
    let dark = dark.unwrap_or(false);
    let trailing = trailing.unwrap_or(false);
    let content_kind = content_kind.unwrap_or(0);
    let bar = bar.unwrap_or(0);
    let hidden = hidden.unwrap_or(false);
    let screen_label = label.clone();
    figures::example(
        label,
        tone,
        note,
        RenderSlot::new(move || {
            let screen_label = screen_label.clone();
            let rail = rail.clone();
            let body = RenderSlot::new(move || {
                let screen = figures::screen(screen_label.clone(), width, Dp(340.0))
                    .trailing(trailing)
                    .content_kind(content_kind)
                    .bar(bar)
                    .hidden_menu(hidden);
                if !hidden && bar == 0 {
                    screen.rail_shared(rail.clone());
                }
            });
            if dark {
                // Only this illustration uses the dark scheme; the article
                // keeps the default Material 2025 surfaces.
                material_theme()
                    .theme(|| MaterialTheme::from_seed(ACCENT, true))
                    .child(move || body.render());
            } else {
                body.render();
            }
        }),
    )
    .height(Dp(380.0));
}

#[tessera]
fn overview() {
    bullets(vec![
        "Use navigation rails in medium, expanded, large, or extra-large window sizes".into(),
        "Can contain 3-7 destinations plus an optional FAB".into(),
        "Always put the rail in the same place, even on different screens of an app".into(),
    ]);
    gap(Dp(32.0));
    figures::scene(
        "Collapsed and expanded navigation rails on a tablet and a phone",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen("Expanded navigation rail on a tablet", Dp(560.0), Dp(480.0))
                .expanded(true)
                .fab(true);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("Collapsed navigation rail on a phone", Dp(220.0), Dp(480.0))
                .fab(true)
                .content_kind(1);
        }),
    );
    gap(Dp(16.0));
    caption(
        "Collapsed and expanded navigation rails can transition between each other on any device, including large or medium window size classes like tablets, and compact window size classes like phones in portrait orientation",
    );
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
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "This page documents the native navigation_rail and navigation_rail_item components. The values below were verified against the published navigation rail specification.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "May 2025 — A collapsed and an expanded navigation rail replaced the baseline navigation rail. The expanded rail is meant to replace the navigation drawer; in Expressive interfaces, use an expanded navigation rail instead of a drawer.",
    );
    gap(Dp(24.0));
    subheading("Variants and naming".into());
    bullets(vec![
        "Baseline navigation rail is no longer recommended".into(),
        "Added a collapsed navigation rail that replaces the baseline rail".into(),
        "Added an expanded navigation rail that replaces the navigation drawer".into(),
    ]);
    gap(Dp(24.0));
    subheading("Configurations".into());
    bullets(vec![
        "Expanded rail modality: non-modal or modal".into(),
        "Expanded behavior: transition to collapsed rail, or hide when collapsed".into(),
    ]);
    gap(Dp(24.0));
    subheading("Color".into());
    bullets(vec![
        "Active label on vertical items changed from on surface variant to secondary".into(),
    ]);
    gap(Dp(32.0));
    figures::scene(
        "Collapsed and expanded navigation rails on timer screens",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("Collapsed timer navigation rail", Dp(300.0), Dp(440.0))
                .fab(true)
                .selected(2);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("Expanded timer navigation rail", Dp(400.0), Dp(440.0))
                .expanded(true)
                .fab(true)
                .selected(2);
        }),
    );
    gap(Dp(16.0));
    caption(
        "The collapsed and expanded navigation rails match visually and can transition into each other",
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "Behavior: Predictive back interaction".into(),
        "Color: New color mappings and compatibility with dynamic color".into(),
        "States: The active destination can be indicated with a pill shape in a contrasting color"
            .into(),
    ]);
    gap(Dp(32.0));
    figures::pair(
        RenderSlot::new(|| {
            figures::example(
                "M2 navigation rail",
                0,
                "M2: The navigation rail uses icon color, weight, and fill to communicate which destination is active",
                RenderSlot::new(|| {
                    figures::screen("M2 navigation rail with photos", Dp(340.0), Dp(260.0))
                        .content_kind(2)
                        .rail(|| {
                            figures::mock_rail()
                                .count(3)
                                .selected(1)
                                .m2(true)
                                .width(Dp(80.0));
                        });
                }),
            );
        }),
        RenderSlot::new(|| {
            figures::example(
                "M3 navigation rail",
                0,
                "M3: The navigation rail uses a pill-shaped active indicator to communicate which destination is active",
                RenderSlot::new(|| {
                    figures::screen("M3 navigation rail with photos", Dp(340.0), Dp(260.0))
                        .content_kind(2)
                        .rail(|| {
                            figures::mock_rail().count(3).selected(1);
                        });
                }),
            );
        }),
    );
}

#[tessera]
fn variants() {
    figures::canvas(
        "Collapsed and expanded navigation rail variants",
        Dp(560.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    figures::number(1);
                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                    figures::specimen(
                        "Collapsed navigation rail variant",
                        false,
                        Dp(96.0),
                        Dp(512.0),
                    )
                    .menu(true)
                    .fab(true)
                    .badges(true)
                    .rail_height(Dp(512.0));
                    spacer().modifier(Modifier::new().width(Dp(120.0)));
                    figures::number(2);
                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                    figures::specimen(
                        "Expanded navigation rail variant",
                        true,
                        Dp(220.0),
                        Dp(512.0),
                    )
                    .menu(true)
                    .fab(true)
                    .badges(true)
                    .rail_height(Dp(512.0));
                });
        }),
    );
    gap(Dp(24.0));
    legend_list(vec![
        "Collapsed navigation rail".into(),
        "Expanded navigation rail".into(),
    ]);
    gap(Dp(48.0));
    subheading("Baseline variants".into());
    prose(
        "The baseline navigation rail is no longer recommended, and should be replaced by the collapsed navigation rail.",
    );
    gap(Dp(24.0));
    figures::canvas(
        "Baseline navigation rail",
        Dp(480.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    figures::number(1);
                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                    boxed()
                        .modifier(Modifier::new().size(Dp(80.0), Dp(432.0)))
                        .children(|| {
                            figures::mock_rail()
                                .baseline(true)
                                .set(0)
                                .count(3)
                                .selected(0)
                                .menu(true)
                                .fab(true);
                        });
                });
        }),
    );
    gap(Dp(24.0));
    legend(1, "The baseline navigation rail is no longer recommended");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec![
                "Collapsed navigation rail".into(),
                "—".into(),
                "Available".into(),
            ],
            vec![
                "Expanded navigation rail".into(),
                "—".into(),
                "Available".into(),
            ],
            vec![
                "Navigation rail (baseline)".into(),
                "Available".into(),
                "Not recommended. Use collapsed navigation rail.".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    prose(
        "In compact windows, use a navigation bar instead of a rail. From the medium breakpoint up, the rail can be collapsed or expanded; an expanded rail is either standard (placed beside the content) or modal (overlaying the content).",
    );
    gap(Dp(32.0));
    figures::canvas(
        "Expanded standard and modal navigation rails in dark theme",
        Dp(540.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(|| {
                    figures::number(1);
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    figures::screen("Expanded standard navigation rail", Dp(440.0), Dp(440.0))
                        .content_kind(1)
                        .rail(|| {
                            figures::mock_rail()
                                .expanded(true)
                                .set(0)
                                .count(4)
                                .selected(0)
                                .menu(true)
                                .fab(true);
                        });
                    spacer().modifier(Modifier::new().width(Dp(32.0)));
                    figures::number(2);
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    figures::screen("Expanded modal navigation rail", Dp(440.0), Dp(440.0))
                        .content_kind(1)
                        .rail(|| {
                            figures::mock_rail()
                                .set(0)
                                .count(4)
                                .selected(0)
                                .menu(true)
                                .fab(true);
                        })
                        .overlay(|| {
                            figures::modal_overlay().set(0).selected(0);
                        });
                });
        }),
    )
    .dark(true)
    .background(srgb(74, 68, 88))
    .dots(false);
    gap(Dp(24.0));
    legend_list(vec![
        "Expanded layout: standard".into(),
        "Expanded layout: modal".into(),
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
                "Expanded layout".into(),
                "Standard (default)".into(),
                "Available as navigation drawer".into(),
                "Available".into(),
            ],
            vec![
                "Expanded layout".into(),
                "Modal".into(),
                "Available as modal navigation drawer".into(),
                "Available".into(),
            ],
            vec![
                "Expanded behavior".into(),
                "Hide when collapsed".into(),
                "—".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "Browse the component elements, attributes, tokens, and their values. Color values below are the Default light scheme.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Container width (collapsed)".into(),
                "md.comp.navigation-rail.container.width".into(),
                "96dp".into(),
            ],
            vec![
                "Container width (expanded)".into(),
                "md.comp.navigation-rail-expanded.container.width".into(),
                "220–360dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "md.comp.navigation-rail.active-indicator".into(),
                "56 × 32dp, 16dp radius".into(),
            ],
            vec![
                "Active indicator color".into(),
                "md.comp.navigation-rail.active-indicator.color".into(),
                "Secondary container #E8DEF8".into(),
            ],
            vec![
                "Active icon color".into(),
                "md.comp.navigation-rail.active-icon.color".into(),
                "On secondary container #4A4458".into(),
            ],
            vec![
                "Active label text color".into(),
                "md.comp.navigation-rail.active-label-text.color".into(),
                "Secondary #625B71".into(),
            ],
            vec![
                "Inactive icon color".into(),
                "md.comp.navigation-rail.inactive-icon.color".into(),
                "On surface variant #49454F".into(),
            ],
            vec![
                "Inactive label text color".into(),
                "md.comp.navigation-rail.inactive-label-text.color".into(),
                "On surface variant #49454F".into(),
            ],
            vec![
                "Icon size".into(),
                "md.comp.navigation-rail.icon.size".into(),
                "24dp".into(),
            ],
            vec![
                "Label".into(),
                "md.comp.navigation-rail.label-text".into(),
                "Label medium (vertical) · Label large (horizontal)".into(),
            ],
            vec![
                "State layer".into(),
                "md.comp.navigation-rail.state-layer".into(),
                "Hover 8% · Focus 10% · Press 10%".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The published token set groups container and item values by interaction state: Enabled, Hovered, Focused and Pressed.",
    );
}

fn anatomy_callouts() -> Vec<figures::Callout> {
    vec![
        (1, "container", true),
        (2, "menu", false),
        (3, "fab", false),
        (4, "icon0", false),
        (5, "indicator0", true),
        (6, "label0", false),
        (7, "large-badge", false),
        (8, "badge-label", true),
        (9, "small-badge", false),
    ]
}

#[tessera]
fn anatomy_figure(
    collapsed: Vec<figures::Callout>,
    expanded: Vec<figures::Callout>,
    count: usize,
    filled: bool,
) {
    figures::canvas(
        "Navigation rail anatomy",
        Dp(620.0),
        RenderSlot::new(move || {
            let collapsed = collapsed.clone();
            let expanded = expanded.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    figures::specimen("Collapsed rail anatomy", false, Dp(200.0), Dp(600.0))
                        .menu(true)
                        .fab(true)
                        .count(count)
                        .badges(true)
                        .badge_start(1)
                        .filled(filled)
                        .callouts(collapsed.clone());
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("Expanded rail anatomy", true, Dp(324.0), Dp(600.0))
                        .menu(true)
                        .fab(true)
                        .count(count)
                        .badges(true)
                        .badge_start(1)
                        .filled(filled)
                        .callouts(expanded.clone());
                });
        }),
    );
}

#[tessera]
fn anatomy() {
    anatomy_figure(anatomy_callouts(), anatomy_callouts(), 3, false);
    gap(Dp(24.0));
    prose("Collapsed and expanded navigation rail elements:");
    gap(Dp(16.0));
    legend_list(vec![
        "Container".into(),
        "Menu (optional)".into(),
        "FAB or Extended FAB (optional)".into(),
        "Icon".into(),
        "Active indicator".into(),
        "Label text".into(),
        "Large badge (optional)".into(),
        "Large badge label (optional)".into(),
        "Small badge (optional)".into(),
    ]);
}

fn color_callouts(expanded: bool) -> Vec<figures::Callout> {
    if expanded {
        vec![
            (1, "container", true),
            (2, "icon0", false),
            (3, "indicator0", true),
            (4, "label0", true),
            (5, "icon1", false),
            (6, "label1", true),
            (7, "large-badge", false),
            (8, "badge-label", true),
            (9, "small-badge", false),
        ]
    } else {
        vec![
            (1, "container", true),
            (2, "icon0", false),
            (3, "indicator0", true),
            (4, "label0", false),
            (5, "icon1", true),
            (6, "label1", false),
            (7, "large-badge", false),
            (8, "badge-label", true),
            (9, "small-badge", true),
        ]
    }
}

#[tessera]
fn rail_color_half(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            rail_color_half_content(dark);
        });
}

#[tessera]
fn rail_color_half_content(dark: bool) {
    let mode = if dark { "Dark" } else { "Light" };
    // The published dark diagram uses the baseline surface pair; light keeps
    // the native Material 2025 surfaces.
    let (background, container) = if dark {
        (Some(srgb(20, 18, 24)), Some(srgb(33, 31, 38)))
    } else {
        (None, None)
    };
    figures::canvas(
        format!("{mode} navigation rail colors"),
        Dp(640.0),
        RenderSlot::new(move || {
            row()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    for expanded in [false, true] {
                        if expanded {
                            spacer().modifier(Modifier::new().width(Dp(48.0)));
                        }
                        let rail = figures::specimen(
                            format!(
                                "{mode} {} reference rail",
                                if expanded { "expanded" } else { "collapsed" }
                            ),
                            expanded,
                            Dp(if expanded { 400.0 } else { 260.0 }),
                            Dp(608.0),
                        )
                        .menu(true)
                        .fab(true)
                        .badges(true)
                        .badge_start(2)
                        .count(4)
                        .filled(true)
                        .callouts(color_callouts(expanded));
                        if let Some(container) = container {
                            rail.container(container);
                        }
                    }
                });
        }),
    )
    .dark(dark)
    .background_optional(background);
}

#[tessera]
fn rail_color_pair() {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            rail_color_half(false);
            gap(Dp(24.0));
            rail_color_half(true);
        });
}

#[tessera]
fn colors() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
    );
    gap(Dp(24.0));
    rail_color_pair();
    gap(Dp(16.0));
    caption(
        "The dark reference matches the published baseline surface (#141218) and container (#211F26); the light reference and all other color roles use the current Material 2025 scheme.",
    );
    gap(Dp(24.0));
    prose("Navigation rail color roles used for light and dark schemes:");
    gap(Dp(16.0));
    legend_list(
        navigation_color_roles(true, false)
            .into_iter()
            .map(|role| role.replace(" / ", ", "))
            .collect(),
    );
}

#[tessera]
fn state_grid(expanded: bool) {
    figures::canvas(
        if expanded {
            "Expanded navigation rail states"
        } else {
            "Collapsed navigation rail states"
        },
        Dp(460.0),
        RenderSlot::new(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(move || {
                    for state in 0..4 {
                        if state > 0 {
                            spacer().modifier(Modifier::new().width(Dp(if expanded {
                                24.0
                            } else {
                                88.0
                            })));
                        }
                        column()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                figures::number(state + 1);
                                gap(Dp(16.0));
                                boxed()
                                    .modifier(
                                        Modifier::new().size(
                                            Dp(if expanded { 184.0 } else { 96.0 }),
                                            Dp(380.0),
                                        ),
                                    )
                                    .children(move || {
                                        let rail = figures::mock_rail().expanded(expanded);
                                        if expanded {
                                            rail.width(Dp(184.0));
                                        }
                                        rail.set(0)
                                            .count(3)
                                            .selected(0)
                                            .menu(true)
                                            .state(state)
                                            .target_count(2)
                                            .cursor(state == 1 || state == 3);
                                    });
                            });
                    }
                });
        }),
    );
}

#[tessera]
fn states() {
    prose(
        "States are visual representations used to communicate the status of a component or an interactive element.",
    );
    gap(Dp(16.0));
    prose(
        "The navigation item's target area always spans the full width of the nav rail, even if the item container hugs its contents.",
    );
    gap(Dp(32.0));
    state_grid(false);
    gap(Dp(32.0));
    state_grid(true);
    gap(Dp(24.0));
    legend_list(vec![
        "Enabled".into(),
        "Hovered".into(),
        "Focused".into(),
        "Pressed".into(),
    ]);
    gap(Dp(24.0));
    prose(
        "The native component does not expose a per-destination disabled state; hide a destination or disable its host when a destination must be unavailable.",
    );
}

#[tessera]
fn measurements() {
    figures::canvas(
        "Navigation rail padding and size measurements",
        Dp(640.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    figures::specimen("Collapsed rail measurements", false, Dp(240.0), Dp(620.0))
                        .menu(true)
                        .fab(true)
                        .count(4)
                        .measurements(true)
                        .rail_height(Dp(540.0));
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("Expanded rail measurements", true, Dp(360.0), Dp(620.0))
                        .menu(true)
                        .fab(true)
                        .count(4)
                        .measurements(true)
                        .rail_height(Dp(540.0));
                });
        }),
    );
    gap(Dp(16.0));
    caption("Navigation rail padding and size measurements");
    gap(Dp(32.0));
    spec_table(
        vec![
            "Element".into(),
            "Attribute".into(),
            "Tessera native".into(),
            "M3 reference".into(),
        ],
        vec![
            vec![
                "Container".into(),
                "Collapsed width".into(),
                "96dp".into(),
                "96dp (baseline rail: 80dp)".into(),
            ],
            vec![
                "Container".into(),
                "Expanded width".into(),
                "220dp minimum".into(),
                "220–360dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "Size".into(),
                "56 × 32dp".into(),
                "56 × 32dp".into(),
            ],
            vec![
                "Active indicator".into(),
                "Corner radius".into(),
                "Capsule (16dp)".into(),
                "16dp".into(),
            ],
            vec!["Icon".into(), "Size".into(), "24dp".into(), "24dp".into()],
            vec![
                "Destination".into(),
                "Minimum height".into(),
                "56dp".into(),
                "56dp".into(),
            ],
            vec![
                "Destination".into(),
                "Minimum target".into(),
                "48 × 48dp".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "Container".into(),
                "Top padding".into(),
                "36dp".into(),
                "36dp".into(),
            ],
            vec![
                "Indicator to label".into(),
                "Gap".into(),
                "4dp (collapsed, icon-top)".into(),
                "4dp".into(),
            ],
            vec![
                "Expanded content".into(),
                "Icon to label inside indicator (distinct from collapsed 4dp)".into(),
                "8dp".into(),
                "8dp".into(),
            ],
            vec![
                "Expanded content".into(),
                "Leading margin · indicator padding".into(),
                "20dp · 16dp".into(),
                "20dp · 16dp".into(),
            ],
            vec![
                "Destinations".into(),
                "Recommended count".into(),
                "3–7".into(),
                "3–7".into(),
            ],
            vec![
                "Window".into(),
                "Breakpoint".into(),
                "600dp and wider".into(),
                "Medium and larger".into(),
            ],
        ],
    );
}

#[tessera]
fn layout_figure(expanded: bool) {
    figures::canvas(
        if expanded {
            "Common expanded navigation rail layouts"
        } else {
            "Common collapsed navigation rail layouts"
        },
        Dp(500.0),
        RenderSlot::new(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(move || {
                    for (index, (menu, fab)) in
                        [(false, false), (true, false), (false, true), (true, true)]
                            .into_iter()
                            .enumerate()
                    {
                        if index > 0 {
                            spacer().modifier(Modifier::new().width(Dp(if expanded {
                                8.0
                            } else {
                                48.0
                            })));
                        }
                        column()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                figures::number(index + 1);
                                gap(Dp(8.0));
                                figures::specimen(
                                    format!(
                                        "Common {} layout {}",
                                        if expanded { "expanded" } else { "collapsed" },
                                        index + 1
                                    ),
                                    expanded,
                                    Dp(if expanded { 228.0 } else { 120.0 }),
                                    Dp(420.0),
                                )
                                .menu(menu)
                                .fab(fab)
                                .rail_height(Dp(420.0));
                            });
                    }
                });
        }),
    );
}

#[tessera]
fn layouts() {
    layout_figure(false);
    gap(Dp(32.0));
    layout_figure(true);
    gap(Dp(24.0));
    legend_list(vec![
        "Three navigation items".into(),
        "Three navigation items with a menu".into(),
        "Three navigation items with a FAB".into(),
        "Three navigation items with a menu and FAB".into(),
    ]);
}

fn baseline_callouts(color: bool) -> Vec<figures::Callout> {
    if color {
        vec![
            (1, "icon0", false),
            (2, "indicator0", true),
            (3, "label0", false),
            (4, "icon1", true),
            (5, "label1", false),
            (6, "large-badge", true),
            (7, "badge-label", false),
            (8, "small-badge", true),
        ]
    } else {
        vec![
            (1, "container", true),
            (2, "menu", false),
            (3, "icon0", false),
            (4, "indicator0", true),
            (5, "label0", false),
            (6, "badge-label", false),
            (7, "large-badge", true),
            (8, "small-badge", true),
        ]
    }
}

#[tessera]
fn baseline_rail(callouts: Vec<figures::Callout>, marks: usize, filled: bool) {
    figures::annotated_mock(
        Dp(240.0),
        Dp(420.0),
        tessera_ui::RenderSlotWith::new(move |geometry| {
            boxed()
                .modifier(Modifier::new().size(Dp(80.0), Dp(400.0)))
                .children(move || {
                    figures::mock_rail()
                        .geometry(geometry)
                        .baseline(true)
                        .set(0)
                        .count(4)
                        .selected(0)
                        .menu(true)
                        .badges(true)
                        .badge_start(2);
                });
        }),
    )
    .callouts(callouts)
    .marks(marks)
    .filled(filled);
}

#[tessera]
fn baseline_canvas(#[prop(into)] label: String, callouts: Vec<figures::Callout>, filled: bool) {
    figures::canvas(
        label,
        Dp(460.0),
        RenderSlot::new(move || {
            baseline_rail(callouts.clone(), 0, filled);
        }),
    );
}

#[tessera]
fn baseline_measurement(#[prop(into)] label: String, marks: usize) {
    let note = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            figures::canvas(
                label.clone(),
                Dp(460.0),
                RenderSlot::new(move || {
                    baseline_rail(Vec::new(), marks, false);
                }),
            )
            .background(srgb(232, 222, 248))
            .dots(false);
            gap(Dp(12.0));
            caption(note.clone());
        });
}

#[tessera]
fn baseline_states(dark: bool) {
    figures::canvas(
        if dark {
            "Dark baseline navigation rail states"
        } else {
            "Light baseline navigation rail states"
        },
        Dp(500.0),
        RenderSlot::new(|| {
            column().children(|| {
                for (row_index, target) in [0_usize, 1].into_iter().enumerate() {
                    if row_index > 0 {
                        gap(Dp(24.0));
                    }
                    row().children(move || {
                        for state in 0..4 {
                            if state > 0 {
                                spacer().modifier(Modifier::new().width(Dp(56.0)));
                            }
                            column()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    figures::number(row_index * 4 + state + 1);
                                    gap(Dp(8.0));
                                    boxed()
                                        .modifier(Modifier::new().size(Dp(80.0), Dp(180.0)))
                                        .children(move || {
                                            figures::mock_rail()
                                                .baseline(true)
                                                .set(0)
                                                .count(2)
                                                .selected(0)
                                                .state(state)
                                                .target_index(target)
                                                .cursor(state == 1 || state == 3);
                                        });
                                });
                        }
                    });
                }
            });
        }),
    )
    .dark(dark);
}

#[tessera]
fn baseline() {
    prose(
        "The baseline navigation rail is no longer recommended. Use the collapsed navigation rail documented above for new designs.",
    );
    gap(Dp(24.0));
    baseline_canvas(
        "Baseline navigation rail anatomy",
        baseline_callouts(false),
        false,
    );
    gap(Dp(24.0));
    legend_list(vec![
        "Container".into(),
        "Menu icon (optional)".into(),
        "Icon".into(),
        "Active indicator".into(),
        "Label text".into(),
        "Large badge label (optional)".into(),
        "Large badge (optional)".into(),
        "Badge (optional)".into(),
    ]);
    gap(Dp(32.0));
    subheading("Tokens & specs".into());
    spec_table(
        vec!["Element".into(), "Baseline".into(), "Collapsed rail".into()],
        vec![
            vec!["Container width".into(), "80dp".into(), "96dp".into()],
            vec![
                "Active label color".into(),
                "On surface".into(),
                "Secondary".into(),
            ],
            vec![
                "Active indicator".into(),
                "56 × 32dp".into(),
                "56 × 32dp".into(),
            ],
            vec!["Icon".into(), "24dp".into(), "24dp".into()],
            vec!["Destination height".into(), "56dp".into(), "56dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Color".into());
    baseline_canvas(
        "Light baseline navigation rail colors",
        baseline_callouts(true),
        true,
    );
    gap(Dp(16.0));
    prose("Navigation rail color roles used for light and dark schemes:");
    gap(Dp(16.0));
    legend_list(
        navigation_color_roles(true, true)
            .into_iter()
            .map(String::from)
            .collect(),
    );
    gap(Dp(32.0));
    subheading("States".into());
    prose(
        "States are visual representations used to communicate the status of a component or interactive element.",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        baseline_states(dark);
        gap(Dp(16.0));
    }
    legend_list(
        ["active", "inactive"]
            .into_iter()
            .flat_map(|target| {
                ["Enabled", "Hovered", "Focused", "Pressed"]
                    .map(|state| format!("{state} (on {target} destination)"))
            })
            .collect(),
    );
    gap(Dp(32.0));
    subheading("Measurements".into());
    figures::pair(
        RenderSlot::new(|| {
            baseline_measurement("Navigation rail size measurements", 2);
        }),
        RenderSlot::new(|| {
            baseline_measurement("Navigation rail padding and margin measurements", 3);
        }),
    );
    gap(Dp(32.0));
    subheading("Configurations".into());
    prose("Common arrangements of elements within a navigation rail");
    gap(Dp(24.0));
    figures::canvas(
        "Baseline navigation rail configurations",
        Dp(480.0),
        RenderSlot::new(|| {
            row().children(|| {
                // Labels: 0 none, 1 selected destination only, 2 all
                // destinations.
                for (index, (menu, fab, labels)) in [
                    (true, false, 1),
                    (false, true, 1),
                    (true, true, 0),
                    (false, false, 2),
                    (true, true, 2),
                ]
                .into_iter()
                .enumerate()
                {
                    if index > 0 {
                        spacer().modifier(Modifier::new().width(Dp(20.0)));
                        boxed().modifier(
                            Modifier::new()
                                .width(Dp(1.0))
                                .height(Dp(452.0))
                                .background(srgb(73, 69, 79)),
                        );
                        spacer().modifier(Modifier::new().width(Dp(20.0)));
                    }
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            figures::number(index + 1);
                            gap(Dp(8.0));
                            boxed()
                                .modifier(Modifier::new().size(Dp(80.0), Dp(420.0)))
                                .children(move || {
                                    figures::mock_rail()
                                        .baseline(true)
                                        .set(0)
                                        .count(4)
                                        .selected(0)
                                        .menu(menu)
                                        .fab(fab)
                                        .fab_tertiary(true)
                                        .hide_labels(labels == 0)
                                        .active_label_only(labels == 1)
                                        .header_height(Dp(if labels == 0 { 126.0 } else { 136.0 }));
                                });
                        });
                }
            });
        }),
    )
    .dark(true);
    gap(Dp(24.0));
    legend_list(vec![
        "With a menu".into(),
        "With a FAB".into(),
        "With menu and FAB, without labels".into(),
        "All destinations with text labels".into(),
        "With menu, FAB, and label text for all destinations".into(),
    ]);
}

#[tessera]
fn usage() {
    figures::scene(
        "Navigation rail shown collapsed and expanded",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen("Collapsed rail on a timer screen", Dp(300.0), Dp(480.0))
                .selected(2)
                .fab(true);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("Expanded rail on a timer screen", Dp(400.0), Dp(480.0))
                .expanded(true)
                .selected(2)
                .fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("Use the menu icon to transition between collapsed and expanded navigation rails");
    gap(Dp(32.0));
    prose(
        "The navigation rail can display navigation items, a menu, and a floating action button (FAB) in a vertical orientation.",
    );
    gap(Dp(16.0));
    prose(
        "There are two variants of navigation rails, collapsed and expanded, which can easily transform into each other when the menu button is selected.",
    );
    gap(Dp(24.0));
    subheading("Collapsed".into());
    prose(
        "The collapsed nav rail runs along the leading edge of the window, and should contain 3–7 navigation items. It should not be hidden.",
    );
    gap(Dp(16.0));
    prose(
        "It can be used in medium to extra-large windows, such as tablets and desktop. In medium windows with few destinations, consider using a navigation bar instead. Compact windows should always use a navigation bar.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Collapsed navigation rail with a timer FAB",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen("Collapsed navigation rail with a FAB", Dp(560.0), Dp(480.0))
                .selected(2)
                .fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("A navigation rail should be the only visible navigation element");
    gap(Dp(32.0));
    subheading("Expanded".into());
    prose(
        "The expanded navigation rail can be standard or modal, and should always open from a menu icon. An expanded rail can reveal secondary destinations not visible when collapsed.",
    );
    gap(Dp(16.0));
    prose(
        "The standard configuration is placed beside body content. It's best for larger windows with lots of available space. The modal configuration overlaps the body content, and should be opened from a menu icon. Use the modal configuration for:",
    );
    gap(Dp(8.0));
    bullets(vec![
        "Information dense layouts where space is limited".into(),
        "Products with many navigation items".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "Expanded navigation rail by default and over content",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("Expanded by default", Dp(440.0), Dp(440.0))
                .expanded(true)
                .selected(2)
                .fab(true);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            figures::screen("Expanded over content", Dp(440.0), Dp(440.0))
                .rail(|| {
                    figures::mock_rail()
                        .count(4)
                        .selected(2)
                        .menu(true)
                        .fab(true);
                })
                .overlay(|| {
                    figures::modal_overlay().selected(2);
                });
        }),
    );
    gap(Dp(16.0));
    caption(
        "A navigation rail can be expanded by default on larger screen sizes, or can be expanded over content on smaller screen sizes",
    );
    gap(Dp(32.0));
    prose(
        "In immersive experiences, the expanded navigation rail can be hidden entirely, appearing only when the menu icon is selected. The collapsed navigation rail should not be hidden.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Navigation rail and hidden navigation rail",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen(
                "Hidden navigation rail with a menu icon",
                Dp(440.0),
                Dp(440.0),
            )
            .hidden_menu(true);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            figures::screen(
                "Hidden navigation rail expanded over content",
                Dp(440.0),
                Dp(440.0),
            )
            .overlay(|| {
                figures::modal_overlay().selected(2);
            });
        }),
    );
    gap(Dp(16.0));
    caption(
        "The expanded navigation rail can also be hidden, appearing only when the menu icon is selected",
    );
}

fn guide_callouts(expanded: bool) -> Vec<figures::Callout> {
    vec![
        (1, "container", true),
        (2, "menu", false),
        (3, "fab", false),
        (4, "icon0", false),
        (5, "label0", expanded),
        (6, "indicator0", !expanded),
        (7, "icon1", !expanded),
        (8, "large-badge", false),
        (9, "badge-label", true),
        (10, "small-badge", false),
        (11, "label2", true),
    ]
}

#[tessera]
fn guide_anatomy() {
    anatomy_figure(guide_callouts(false), guide_callouts(true), 4, true);
    gap(Dp(24.0));
    legend_list(vec![
        "Container".into(),
        "Menu (optional)".into(),
        "Floating action button (FAB) (optional)".into(),
        "Icon - active".into(),
        "Label text - active".into(),
        "Active indicator".into(),
        "Icon - inactive".into(),
        "Large badge (optional)".into(),
        "Large badge label".into(),
        "Small badge".into(),
        "Label text - inactive".into(),
    ]);
    gap(Dp(32.0));
    subheading("Container".into());
    prose(
        "The navigation rail should be placed on the leading edge of the window. This is the left side for left-to-right languages, and the right side for right-to-left languages.",
    );
    gap(Dp(16.0));
    prose(
        "The container fill can be turned off so the nav rail appears directly on the surface. When doing this, make sure all items have a minimum of 3:1 color contrast.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Left-to-right navigation rail",
                0,
                "Left-to-right layouts place the rail on the left edge",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Right-to-left navigation rail",
                0,
                "Right-to-left layouts place the rail on the right edge",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            )
            .trailing(true);
        }),
    );
    gap(Dp(16.0));
    caption("The navigation rail should be placed on the leading edge of the window");
    gap(Dp(32.0));
    prose(
        "The navigation rail should always run vertically along the side of a layout. Don't make it horizontal. Use a navigation bar for horizontal navigation.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Vertical navigation rail",
                1,
                "Run the navigation rail vertically along the side of the layout",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Horizontal navigation rail",
                3,
                "Don't use the navigation rail horizontally. Use a navigation bar instead.",
                RenderSlot::empty(),
            )
            .bar(2);
        }),
    );
    gap(Dp(32.0));
    prose(
        "Navigation rail items can be aligned as a group to the top or center of a layout. On tablets, use center alignment to make it easier to reach items. The menu icon and FAB should always be top-aligned.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Top-aligned rail destinations",
                0,
                "Top-aligned rail destination placement",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Center-aligned rail destinations",
                0,
                "Center-aligned rail destination placement",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(3)
                        .selected(1)
                        .menu(true)
                        .center(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    subheading("Menu (optional)".into());
    prose(
        "The menu button can transition between the collapsed and expanded navigation rails. Once expanded, the rail can reveal secondary destinations.",
    );
    gap(Dp(16.0));
    prose(
        "When the navigation rail is expanded, the menu icon should change to represent that it can be collapsed.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Expanded and collapsed navigation rails controlled by a menu icon button",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("Collapsed rail with a menu", Dp(440.0), Dp(440.0)).count(4);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            timer_screen(
                "Expanded rail with secondary destinations",
                Dp(440.0),
                Dp(440.0),
            )
            .expanded(true)
            .count(5);
        }),
    );
    gap(Dp(16.0));
    caption("A navigation rail can expand to reveal more destinations");
    gap(Dp(32.0));
    subheading("Floating action button (FAB) (optional)".into());
    prose(
        "The container of the navigation rail is ideal for anchoring the FAB to the top of a screen, placing the app's key action above navigation destinations.",
    );
    gap(Dp(16.0));
    prose(
        "When nested within another component, such as the navigation rail, the FAB's resting elevation should be level 0.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "FAB at the top of the rail",
                1,
                "A top-aligned FAB in the navigation rail",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).fab(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "FAB at the bottom of the rail",
                3,
                "Avoid placing the FAB below navigation items",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(3)
                        .selected(1)
                        .fab(true)
                        .fab_bottom(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose(
        "The top of the rail can also be used for a logo, however avoid using logos that could be mistaken as buttons. Don't use a logo as a menu button to expand the navigation rail.",
    );
    gap(Dp(24.0));
    card(
        "Logo at the top of the rail",
        2,
        "Use caution when placing logos in the rail where they might be confused with an action or destination",
        RenderSlot::new(|| {
            figures::mock_rail().count(3).selected(1).logo(true);
        }),
    );
    gap(Dp(32.0));
    subheading("Active indicator".into());
    prose("The active indicator shows which page is being displayed.");
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "One active indicator",
                1,
                "Use the active indicator only for the current open page",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Active indicators on every item",
                3,
                "Don't use the active indicator for more than one navigation item at a time",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).all_active(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose(
        "The active indicator hugs the label text in the expanded nav rail. To achieve a similar style to the baseline navigation drawer, consider modifying the active indicator to fill the container. The target area should always span the full width.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Hugging active indicator",
                0,
                "The active indicator hugs contents in the expanded nav rail",
                RenderSlot::new(|| {
                    figures::mock_rail().expanded(true).count(4).selected(1);
                }),
            )
            .width(Dp(380.0));
        }),
        RenderSlot::new(|| {
            card(
                "Filled active indicator",
                0,
                "Override the indicator to fill the container to more closely resemble the baseline navigation drawer",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .expanded(true)
                        .count(4)
                        .selected(1)
                        .fill_indicator(true);
                }),
            )
            .width(Dp(380.0));
        }),
    );
    gap(Dp(32.0));
    subheading("Icons".into());
    prose("Navigation rail items must use icons that symbolize the content of their page.");
    gap(Dp(24.0));
    card(
        "Icons for timer destinations",
        0,
        "Icons should symbolize the content of the page they open",
        RenderSlot::new(|| {
            figures::mock_rail().count(4).selected(2);
        }),
    );
    gap(Dp(32.0));
    prose(
        "When a destination is selected, the icon fills and changes color. An active indicator appears behind the icon.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Destinations without an active indicator",
                0,
                "Unselected navigation items use outlined icons",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Destination with an active indicator",
                0,
                "Selected navigation items have an active indicator, a filled icon, and a more prominent color",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    subheading("Label text".into());
    prose(
        "The label text should be a short, meaningful description of each navigation destination and another way for users to understand an icon's meaning. All navigation items require a one word label text.",
    );
    gap(Dp(16.0));
    prose(
        "Avoid wrapping long labels when possible. If necessary, create a line break between words, or hyphenate longer words.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Clear navigation labels",
                1,
                "Write clear and concise labels that describe the destination page",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            )
            .dark(true);
        }),
        RenderSlot::new(|| {
            card(
                "Two-line navigation labels",
                2,
                "Break up longer phrases into two text lines if necessary",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(4);
                }),
            )
            .dark(true);
        }),
    );
    gap(Dp(32.0));
    prose(
        "Labels should be short enough to not be truncated. Don't shrink the type scale to fit longer text labels.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Truncated navigation labels",
                3,
                "Don't truncate or display an ellipsis in place of label text",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(2);
                }),
            )
            .dark(true);
        }),
        RenderSlot::new(|| {
            card(
                "Small navigation labels",
                3,
                "Don't reduce the type size to fit more characters into a destination label",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(3);
                }),
            )
            .dark(true);
        }),
    );
    gap(Dp(32.0));
    subheading("Badges".into());
    prose(
        "Navigation rail icons can include badges to communicate dynamic information about the destination, such as counts or status.",
    );
    gap(Dp(16.0));
    prose(
        "In compact nav rails, the badge is placed in the upper right corner of the icon. In expanded nav rails, the badge should be placed next to the label text.",
    );
    gap(Dp(24.0));
    figures::canvas(
        "Navigation rail badges",
        Dp(380.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    let callouts = vec![
                        (1, "small-badge", true),
                        (2, "large-badge", true),
                        (3, "max-badge", true),
                    ];
                    figures::specimen("Collapsed rail with badges", false, Dp(220.0), Dp(340.0))
                        .count(4)
                        .badge_series(true)
                        .filled(true)
                        .callouts(callouts.clone())
                        .rail_height(Dp(320.0));
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("Expanded rail with badges", true, Dp(340.0), Dp(340.0))
                        .count(4)
                        .badge_series(true)
                        .filled(true)
                        .callouts(callouts)
                        .rail_height(Dp(320.0));
                });
        }),
    );
    gap(Dp(24.0));
    legend_list(vec![
        "Small badge on a rail destination".into(),
        "Large badge with a number".into(),
        "Large badge with a maximum character count".into(),
    ]);
    gap(Dp(32.0));
    subheading("Divider (optional)".into());
    prose(
        "A vertical divider can help separate the rail from app content. The divider should be positioned on the edge of the rail container that's adjacent to the app's content area.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Navigation rail with a divider",
        Dp(460.0),
        RenderSlot::new(|| {
            figures::screen(
                "Navigation rail with a divider beside a list",
                Dp(560.0),
                Dp(420.0),
            )
            .content_kind(1)
            .rail(|| {
                let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                figures::mock_rail()
                    .count(4)
                    .selected(1)
                    .menu(true)
                    .divider(true)
                    .container(scheme.surface);
            });
        }),
    );
    gap(Dp(16.0));
    caption(
        "A divider can make the navigation rail container distinct from other on-screen content",
    );
}

#[tessera]
fn placement() {
    prose(
        "In adaptive layouts, the navigation rail should be placed outside any panes, always along the leading edge of the window. Don't place it within body content.",
    );
    gap(Dp(16.0));
    prose(
        "When the navigation rail is hidden, the body content can fill in the remaining space as long as the menu icon is still accessible.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Rail outside body panes",
                0,
                "Place the navigation rail outside panes, along the leading edge of the window",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).menu(true);
                }),
            )
            .content_kind(2);
        }),
        RenderSlot::new(|| {
            card(
                "Hidden rail with a menu icon",
                0,
                "Body content fills the window while the menu icon stays accessible",
                RenderSlot::empty(),
            )
            .content_kind(2)
            .hidden(true);
        }),
    );
    gap(Dp(32.0));
    prose(
        "Tabs can be used alongside a navigation rail to create an extra layer of visible navigation.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Expanded navigation rail opened from a menu button on mobile",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen("Modal expanded rail on a phone", Dp(300.0), Dp(440.0))
                .content_kind(1)
                .rail(|| {
                    figures::mock_rail().count(4).selected(1).menu(true);
                })
                .overlay(|| {
                    figures::modal_overlay();
                });
        }),
    );
    gap(Dp(16.0));
    caption("Expanded navigation rails can open from menu buttons on mobile");
}

#[tessera]
fn adaptive_design() {
    prose("For more, see adaptive design.");
    gap(Dp(24.0));
    subheading("Resizing".into());
    prose(
        "When moving from a large screen to a small screen, a navigation rail can transform into a navigation bar, providing the same quick access in a configuration that's easier to use on smaller displays. Never use the navigation rail and navigation bar simultaneously.",
    );
    gap(Dp(16.0));
    prose(
        "Only use navigation rails for medium breakpoints and larger. Don't use a navigation bar. If there are more than five destinations, consider using a modal expanded nav rail instead.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Compact: Don't use a standard navigation rail for compact layouts due to space constraints. Use a navigation bar instead.".into(),
        "Medium: Use a navigation rail, especially if prioritizing persistent vertical navigation over maximizing vertical content space.".into(),
        "Expanded to extra-large: Use a navigation rail, not a navigation bar. Consider available horizontal space and the number of destinations when choosing between standard and modal.".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "Navigation bar on a phone and navigation rail on a tablet",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen("Phone with a navigation bar", Dp(240.0), Dp(440.0)).bar(1);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("Tablet with a navigation rail", Dp(600.0), Dp(440.0)).fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("On smaller devices, use a navigation bar. On larger displays, use a navigation rail.");
    gap(Dp(32.0));
    subheading("Presentation".into());
    prose(
        "When the navigation rail transitions from collapsed to expanded, the contents of the page should automatically adjust to fit. The contents of the navigation rail also expand to fill the space. For example, the FAB should transition into an extended FAB.",
    );
    gap(Dp(16.0));
    prose("Extra destinations can be shown in an expanded nav rail.");
    gap(Dp(24.0));
    figures::scene(
        "Expanded navigation rail with secondary destinations",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen(
                "Standard expanded rail with extra destinations",
                Dp(640.0),
                Dp(460.0),
            )
            .expanded(true)
            .count(5)
            .fab(true);
        }),
    );
    gap(Dp(16.0));
    caption(
        "Use a standard expanded rail when there are secondary destinations or actions that have lower priority than the main navigation items",
    );
}

#[tessera]
fn behavior(
    modal_open: State<bool>,
    modal_selected: State<usize>,
    session: Option<LiveRailSession>,
) {
    subheading("Modal expanded rail".into());
    prose(
        "A modal expanded rail opens from the menu and closes by selecting its scrim or a destination, or by pressing Escape. It returns focus to the control that opened it.",
    );
    gap(Dp(16.0));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label("Open modal navigation rail")
                .on_click(move || {
                    if session.is_none_or(LiveRailSession::can_open) {
                        modal_open.set(true);
                    }
                });
            text()
                .content(format!(
                    "Current destination: {}",
                    NAMES[modal_selected.get()]
                ))
                .style(type_style(14.0, 20.0));
        });
    caption(
        "Open from the menu, then choose a destination, select the scrim or press Escape to close and return focus",
    );
    gap(Dp(32.0));
    subheading("Scrolling".into());
    prose(
        "Destinations in the navigation rail should remain visible and fixed when scrolling vertically.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(scrolling_rail));
    gap(Dp(16.0));
    caption("Rail destinations remain fixed while on-screen content scrolls vertically");
    gap(Dp(32.0));
    prose(
        "If a layout scrolls horizontally, the rail can scroll off-screen or remain fixed. To distinguish that content is scrolling underneath the rail, use a divider or add elevation to the rail.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(move || {
            card(
                "Rail with a divider over horizontal content",
                0,
                "A divider and color fill change create visual distinction between the rail and horizontally scrolling content",
                RenderSlot::new(move || {
                    figures::mock_rail()
                        .count(3)
                        .selected(1)
                        .menu(true)
                        .divider(true);
                }),
            )
            .content_kind(2);
        }),
        RenderSlot::new(move || {
            card(
                "Elevated rail over horizontal content",
                0,
                "Elevating the rail to level 1 creates visual distinction between the rail and horizontally scrolling content",
                RenderSlot::new(move || {
                    figures::mock_rail()
                        .count(3)
                        .selected(1)
                        .menu(true)
                        .elevated(true);
                }),
            )
            .content_kind(2);
        }),
    );
    gap(Dp(32.0));
    subheading("Selection".into());
    prose(
        "When a destination is tapped, the destination screen uses a top level transition pattern. In addition, the icon becomes filled and the active indicator expands from the center of the icon.",
    );
    gap(Dp(16.0));
    prose(
        "The menu control transitions the rail between collapsed and expanded. Selection is preserved across the transition: the same destination keeps its active indicator and filled icon.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(behavior_rail));
    gap(Dp(16.0));
    caption(
        "Select a destination, then use the menu button to expand or collapse the rail; the selected destination is preserved",
    );
    gap(Dp(32.0));
    subheading("Back".into());
    prose(
        "On Android, a gesture called predictive back allows people to swipe left or right on the screen to go back or dismiss modal components.",
    );
    gap(Dp(8.0));
    bullets(vec![
        "Previous screen is revealed in a preview to signal the destination".into(),
        "Predictive back only applies to the modal expanded navigation rail".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "Modal navigation rail during the predictive back gesture",
        Dp(460.0),
        RenderSlot::new(move || {
            figures::screen(
                "Predictive back on a modal navigation rail",
                Dp(560.0),
                Dp(420.0),
            )
            .rail(|| {
                figures::mock_rail().count(3).selected(1).menu(true);
            })
            .overlay(|| {
                figures::modal_overlay().offset(Dp(-48.0)).count(3);
            });
        }),
    );
    gap(Dp(16.0));
    caption("The nav rail pops off the edge of the window during the predictive back gesture");
}

#[tessera]
fn behavior_rail() {
    let expanded = remember(|| false);
    let selected = remember(|| 0usize);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            boxed()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Navigation),
                    label: Some("Interactive mail navigation".into()),
                    test_tag: Some("behavior-rail".into()),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    navigation_rail()
                        .expanded(expanded.get())
                        .header(move || {
                            icon_button()
                                .standard()
                                .icon(filled::MENU_SVG)
                                .accessibility_label(if expanded.get() {
                                    "Collapse navigation rail"
                                } else {
                                    "Expand navigation rail"
                                })
                                .modifier(Modifier::new().semantics(SemanticsArgs {
                                    test_tag: Some("behavior-rail-menu".into()),
                                    merge_descendants: true,
                                    ..Default::default()
                                }))
                                .on_click(move || {
                                    expanded.with_mut(|expanded| *expanded = !*expanded)
                                });
                        })
                        .content(move |rail_scope| {
                            for (index, name) in NAMES.into_iter().take(4).enumerate() {
                                boxed()
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        test_tag: Some(format!("behavior-rail-item-{index}")),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }))
                                    .children(move || {
                                        navigation_rail_item(rail_scope)
                                            .label(name)
                                            .selected(selected.get() == index)
                                            .on_click(move || selected.set(index))
                                            .icon(move || {
                                                destination_icon(index, selected.get() == index);
                                            });
                                    });
                            }
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format_rail_status(
                    if expanded.get() {
                        "Expanded"
                    } else {
                        "Collapsed"
                    },
                    selected.get(),
                ))
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    test_tag: Some(format!("behavior-rail-destination-{}", selected.get())),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn scrolling_rail() {
    let expanded = remember(|| false);
    let selected = remember(|| 0usize);
    let scroll = remember(ScrollableController::new);
    scrolling_rail_view(expanded, selected, scroll);
}

#[tessera]
fn scrolling_rail_view(
    expanded: State<bool>,
    selected: State<usize>,
    scroll: State<ScrollableController>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        row()
            .modifier(Modifier::new().fill_max_width().height(Dp(480.0)))
            .children(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_height().semantics(SemanticsArgs {
                        role: Some(Role::Navigation),
                        label: Some("Fixed mail navigation beside scrolling content".into()),
                        test_tag: Some("scrolling-rail".into()),
                        merge_descendants: true,
                        ..Default::default()
                    }))
                    .children(move || {
                        navigation_rail()
                            .expanded(expanded.get())
                            .header(move || {
                                icon_button()
                                    .standard()
                                    .icon(filled::MENU_SVG)
                                    .accessibility_label(if expanded.get() {
                                        "Collapse fixed rail"
                                    } else {
                                        "Expand fixed rail"
                                    })
                                    .on_click(move || expanded.with_mut(|value| *value = !*value))
                                    .modifier(Modifier::new().semantics(SemanticsArgs {
                                        test_tag: Some("scrolling-rail-menu".into()),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }));
                            })
                            .content(move |rail_scope| {
                                for (index, name) in NAMES.into_iter().take(4).enumerate() {
                                    boxed()
                                        .modifier(Modifier::new().semantics(SemanticsArgs {
                                            test_tag: Some(format!("scrolling-rail-item-{index}")),
                                            merge_descendants: true,
                                            ..Default::default()
                                        }))
                                        .children(move || {
                                            navigation_rail_item(rail_scope)
                                                .label(name)
                                                .selected(selected.get() == index)
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    destination_icon(
                                                        index,
                                                        selected.get() == index,
                                                    );
                                                });
                                        });
                                }
                            });
                    });
                column()
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .fill_max_size()
                            .vertical_scroll(scroll)
                            .padding_all(Dp(16.0))
                            .semantics(SemanticsArgs {
                                test_tag: Some("scrolling-body".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        for index in 0..12 {
                            boxed()
                                .alignment(Alignment::CenterStart)
                                .modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(96.0))
                                        .padding_all(Dp(12.0))
                                        .background_with_shape(
                                            scheme.surface_container_low,
                                            Shape::rounded_rectangle(Dp(12.0)),
                                        )
                                        .semantics(SemanticsArgs {
                                            test_tag: Some(format!("scrolling-message-{index}")),
                                            merge_descendants: true,
                                            ..Default::default()
                                        }),
                                )
                                .children(move || {
                                    text()
                                        .content(format!(
                                            "{} message {}",
                                            NAMES[selected.get()],
                                            index + 1
                                        ))
                                        .style(type_style(14.0, 20.0));
                                });
                            gap(Dp(12.0));
                        }
                    });
            });
        gap(Dp(16.0));
        text()
            .content(format_rail_status("Fixed rail", selected.get()))
            .style(type_style(14.0, 20.0))
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some(format!("scrolling-rail-destination-{}", selected.get())),
                merge_descendants: true,
                ..Default::default()
            }));
    });
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following using the assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate between navigation destinations".into(),
        "Select a particular navigation destination from a set".into(),
        "Get appropriate feedback based on input type".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "When a navigation item is tapped, the active indicator appears, providing the following feedback to the user that it is selected:",
    );
    gap(Dp(8.0));
    bullets(vec![
        "A ripple passes through the indicator".into(),
        "The icon switches from outlined to filled".into(),
        "The icon and text change color".into(),
    ]);
    gap(Dp(8.0));
    prose(
        "When hovered, the hover state appears, providing a visual cue that the destination is interactive.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Touch tap on a rail destination",
                0,
                "Touch: Tap",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(4)
                        .selected(1)
                        .state(3)
                        .target_index(0)
                        .touch(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Cursor hover on a rail destination",
                0,
                "Cursor: Hover, Click",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(4)
                        .selected(1)
                        .state(1)
                        .target_index(0)
                        .cursor(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose(
        "The target area for expanded navigation rails spans the full width of the container, even though the active indicator visually hugs the content.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Full-width target area in an expanded rail",
        Dp(420.0),
        RenderSlot::new(|| {
            figures::screen("Expanded rail target area", Dp(560.0), Dp(380.0)).rail(|| {
                figures::mock_rail()
                    .expanded(true)
                    .count(4)
                    .selected(2)
                    .state(3)
                    .target_index(1)
                    .target_outline(true)
                    .touch(true);
            });
        }),
    );
    gap(Dp(16.0));
    caption("Touch: Tap");
    gap(Dp(32.0));
    prose(
        "Use a filled icon for the active destination and outlined icons for inactive destinations. Active and inactive icons need sufficient contrast against the container.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Default navigation rail colors",
                1,
                "Use the default color scheme to ensure proper contrast and emphasis on the active destination",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Low-contrast navigation rail colors",
                3,
                "Don't use more than two colors for destinations or low-contrast colors in the navigation rail. This will make distinguishing active items difficult.",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2).low_contrast(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose("If an icon doesn't have a filled style, use the semibold icon weight instead.");
    gap(Dp(24.0));
    figures::scene(
        "Voicemail icon inactive and active",
        Dp(220.0),
        RenderSlot::new(voicemail_weights),
    );
    gap(Dp(16.0));
    caption("An icon with no filled option should use the semibold weight when active");
}

#[tessera]
fn voicemail_weights() {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let style = theme.typography.headline_small;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            for active in [false, true] {
                if active {
                    spacer().modifier(Modifier::new().width(Dp(64.0)));
                }
                let mut style = style.clone();
                style.font_weight = if active { 600 } else { 400 };
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(112.0), Dp(64.0))
                                    .background_with_shape(
                                        if active {
                                            scheme.secondary_container
                                        } else {
                                            scheme.surface_container_high
                                        },
                                        Shape::CAPSULE,
                                    ),
                            )
                            .children(move || {
                                icon().painter(filled::VOICEMAIL_SVG).size(Dp(40.0)).tint(
                                    if active {
                                        scheme.on_secondary_container
                                    } else {
                                        scheme.on_surface_variant
                                    },
                                );
                            });
                        gap(Dp(8.0));
                        text()
                            .content("Voicemail")
                            .style(style.clone())
                            .color(scheme.on_surface);
                    });
            }
        });
}

#[tessera]
fn text_scaling() {
    prose(
        "When someone sets their device to show a larger text size, the navigation rail items should grow vertically to accommodate larger labels while retaining the default padding. It's okay for scaled text to wrap in navigation items.",
    );
    gap(Dp(16.0));
    prose(
        "To remain accessible, ensure the full label is always visible on-screen at up to 2x text sizing. Beyond this size, text can truncate.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Text scaled to 1.5 size",
                0,
                "Text scaled to 1.5 size",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2).label_scale(1.5);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Text scaled to 2x size",
                0,
                "Text scaled to 2x size",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(4)
                        .selected(2)
                        .label_scale(2.0)
                        .label_mode(5);
                }),
            );
        }),
    );
}

#[tessera]
fn focus_example(#[prop(into)] key: String, target: usize, selected: usize, state: usize) {
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            figures::key_chip(key.clone());
            gap(Dp(16.0));
            boxed()
                .modifier(Modifier::new().size(Dp(96.0), Dp(300.0)))
                .children(move || {
                    figures::mock_rail()
                        .count(4)
                        .selected(selected)
                        .state(state)
                        .focus_box(true)
                        .target_index(target);
                });
        });
}

#[tessera]
fn initial_focus() {
    prose(
        "Initial focus lands directly on the first interactive item, whether it's the menu, the FAB, or the first navigation item.",
    );
    gap(Dp(16.0));
    prose(
        "From the FAB or menu, Tab brings the person to the navigation items. Tab or Arrows then navigate between items.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            figures::example(
                "Arrow keys move focus between rail destinations",
                0,
                "Use arrows to move between navigation items",
                RenderSlot::new(|| {
                    row().children(|| {
                        focus_example("Tab", 0, 0, 2);
                        spacer().modifier(Modifier::new().width(Dp(40.0)));
                        focus_example("Arrow", 1, 0, 2);
                    });
                }),
            )
            .height(Dp(380.0));
        }),
        RenderSlot::new(|| {
            figures::example(
                "Space or Enter activates the focused destination",
                0,
                "Use space/enter to activate the focused navigation item",
                RenderSlot::new(|| {
                    row().children(|| {
                        focus_example("Tab", 0, 0, 2);
                        spacer().modifier(Modifier::new().width(Dp(40.0)));
                        focus_example("Space/Enter", 0, 0, 3);
                    });
                }),
            )
            .height(Dp(380.0));
        }),
    );
}

#[tessera]
fn visual_indicators() {
    prose(
        "Icons give the dominant cue of the navigation state. Use a filled icon for the selected destination to contrast with outlined icons for the non-selected destinations.",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "Filled icon for the selected destination",
                1,
                "Use a filled icon variant on the selected navigation item to help differentiate from inactive navigation items",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "Outlined icon for the selected destination",
                3,
                "Avoid using the same unfilled icon style for both selected and unselected items because it lacks important visual feedback",
                RenderSlot::new(|| {
                    figures::mock_rail()
                        .count(4)
                        .selected(2)
                        .outlined_active(true);
                }),
            );
        }),
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec!["Tab".into(), "Move focus into or out of the rail".into()],
            vec![
                "Up / Down arrows".into(),
                "Move focus between destinations".into(),
            ],
            vec![
                "Enter or Space".into(),
                "Activate the focused destination".into(),
            ],
            vec![
                "Escape".into(),
                "Close a modal expanded rail and restore focus".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Arrow keys move focus only; a destination changes only after Enter or Space, so browsing the list never navigates unexpectedly.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "The accessibility label for a navigation item is typically the same as the adjacent text label.",
    );
    gap(Dp(16.0));
    prose(
        "When the visible UI text is ambiguous, accessibility labels need to be more descriptive. For example, a navigation item visibly labeled Recent would benefit from additional information in its accessibility label to clarify the destination's intent.",
    );
    gap(Dp(16.0));
    prose(
        "Note: On Android Views (MDC-Android), a more descriptive accessibility label is not available and the role is not announced.",
    );
    gap(Dp(24.0));
    figures::scene(
        "Accessibility label for the Recent destination",
        Dp(420.0),
        RenderSlot::new(|| {
            figures::label_callout();
            figures::screen(
                "Recent destination in a file navigation rail",
                Dp(520.0),
                Dp(380.0),
            )
            .content_kind(2)
            .rail(|| {
                figures::mock_rail()
                    .set(2)
                    .count(4)
                    .selected(1)
                    .state(2)
                    .focus_box(true)
                    .focus_badge(false)
                    .target_index(1);
            });
        }),
    );
    gap(Dp(16.0));
    caption(
        "While the visible label text reads Recent, the accessibility label for this destination clarifies its function: Recent images",
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "navigation_rail_tests.rs"]
mod reference_tests;

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_ui::{
        CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition,
        testing::{
            LayoutSnapshot, LayoutTestSession, layout_test, pump_layout_test_session,
            pump_layout_test_session_with_input, start_layout_test_session,
        },
    };

    use super::*;

    fn click(
        session: &mut LayoutTestSession<impl Fn()>,
        snapshot: &LayoutSnapshot,
        tag: &str,
        frame: &mut u64,
    ) {
        let bounds = snapshot.node(tag).bounds();
        let cursor = Some(PxPosition::from_f32_xy(
            bounds.x.to_f32() + bounds.width.to_f32() * 0.5,
            bounds.y.to_f32() + bounds.height.to_f32() * 0.5,
        ));
        for content in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ] {
            *frame += 16_666_667;
            pump_layout_test_session_with_input(
                session,
                *frame,
                vec![PointerChange::new(
                    tessera_ui::time::Instant::now(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                cursor,
            );
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(behavior_rail);
    }

    #[test]
    fn menu_toggles_the_rail_without_resetting_the_selected_destination() {
        let mut session = start_layout_test_session(layout_test(fixture).viewport_px(400, 700));
        let mut frame = 0;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(96);
        click(&mut session, &snapshot, "behavior-rail-item-1", &mut frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail-destination-1").exists();
        click(&mut session, &snapshot, "behavior-rail-menu", &mut frame);
        frame += 400_000_000;
        pump_layout_test_session(&mut session, frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(220);
        snapshot.node("behavior-rail-destination-1").exists();
        click(&mut session, &snapshot, "behavior-rail-menu", &mut frame);
        frame += 400_000_000;
        pump_layout_test_session(&mut session, frame);
        frame += 400_000_000;
        let snapshot = pump_layout_test_session(&mut session, frame);
        snapshot.node("behavior-rail").width(96);
        snapshot.node("behavior-rail-destination-1").exists();
    }

    #[tessera]
    fn scrolling_fixture(
        output: tessera_ui::testing::TestProbe<Option<(State<ScrollableController>, State<bool>)>>,
    ) {
        let expanded = remember(|| false);
        let selected = remember(|| 0usize);
        let scroll = remember(ScrollableController::new);
        output.set(Some((scroll, expanded)));
        material_theme().child(move || {
            scrolling_rail_view(expanded, selected, scroll);
        });
    }

    #[test]
    fn scrolling_body_keeps_menu_and_destinations_fixed_and_selectable_in_both_variants() {
        let output = tessera_ui::testing::TestProbe::default();
        let probe = output.clone();
        let mut session = start_layout_test_session(
            layout_test(move || scrolling_fixture(probe.clone())).viewport_px(600, 600),
        );
        let mut frame = 0;
        pump_layout_test_session(&mut session, frame);
        let (scroll, expanded_state) = output.get().unwrap();
        for expanded in [false, true] {
            expanded_state.set(expanded);
            scroll.with_mut(|scroll| scroll.set_scroll_position(PxPosition::ZERO));
            frame += 16_666_667;
            pump_layout_test_session(&mut session, frame);
            frame += 400_000_000;
            let before = pump_layout_test_session(&mut session, frame);
            before
                .node("scrolling-rail")
                .size(if expanded { 220 } else { 96 }, 480);
            let menu = before.node("scrolling-rail-menu").bounds();
            let destination = before.node("scrolling-rail-item-3").bounds();
            let message = before.node("scrolling-message-0").bounds();
            assert!(destination.y.0 + destination.height.0 <= 480);
            scroll.with_mut(|scroll| {
                scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -200.0));
            });
            frame += 16_666_667;
            let scrolled = pump_layout_test_session(&mut session, frame);
            assert_eq!(scrolled.node("scrolling-rail-menu").bounds(), menu);
            assert_eq!(scrolled.node("scrolling-rail-item-3").bounds(), destination);
            assert_eq!(
                scrolled.node("scrolling-message-0").bounds().y.0,
                message.y.0 - 200
            );
            click(&mut session, &scrolled, "scrolling-rail-item-3", &mut frame);
            frame += 16_666_667;
            pump_layout_test_session(&mut session, frame)
                .node("scrolling-rail-destination-3")
                .exists();
        }
    }

    #[tessera]
    fn gallery_fixture() {
        material_theme().child(|| {
            figures::canvas(
                "Accessible figure",
                Dp(560.0),
                RenderSlot::new(|| {
                    figures::specimen("Accessible figure rail", false, Dp(200.0), Dp(540.0))
                        .named(true)
                        .count(4)
                        .callouts(anatomy_callouts());
                }),
            );
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for gallery semantics and accessibility focus"]
    fn gallery_and_behavior_wrappers_keep_destinations_visible_and_focus_separate_from_selection() {
        use tessera_ui::{
            HeadlessConfig, HeadlessRenderer,
            accesskit::{Action, ActionRequest, TreeId},
        };

        for (variant, menu) in [
            None,
            Some("Expand navigation rail"),
            Some("Expand fixed rail"),
        ]
        .into_iter()
        .enumerate()
        {
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || match variant {
                    0 => {
                        gallery_fixture();
                    }
                    1 => {
                        fixture();
                    }
                    _ => {
                        scrolling_fixture(tessera_ui::testing::TestProbe::default());
                    }
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 600,
                    height: 600,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(6);
            let snapshot = renderer.accessibility_snapshot().unwrap();
            let destinations: Vec<_> = snapshot
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab && !node.is_hidden())
                .collect();
            assert_eq!(
                destinations.len(),
                4,
                "every destination must survive wrapper semantics"
            );
            for label in NAMES.into_iter().take(4) {
                assert!(
                    destinations
                        .iter()
                        .any(|(_, node)| node.label() == Some(label))
                );
            }
            if let Some(menu) = menu {
                assert!(
                    snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| node.role() == Role::Button
                            && node.label() == Some(menu)
                            && !node.is_hidden())
                );
            }
            let saved = destinations
                .iter()
                .find(|(_, node)| node.label() == Some("Saved"))
                .unwrap()
                .0;
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action: Action::Focus,
                target_tree: TreeId::ROOT,
                target_node: saved,
                data: None,
            }));
            renderer.render_frames(4);
            let focused = renderer.accessibility_snapshot().unwrap();
            assert_eq!(focused.focus, saved);
            let tabs: Vec<_> = focused
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab)
                .collect();
            assert_eq!(
                tabs.iter()
                    .find(|(_, node)| node.label() == Some("Home"))
                    .unwrap()
                    .1
                    .is_selected(),
                Some(true)
            );
            assert_eq!(
                tabs.iter()
                    .find(|(_, node)| node.label() == Some("Saved"))
                    .unwrap()
                    .1
                    .is_selected(),
                Some(false)
            );
        }
    }
}
