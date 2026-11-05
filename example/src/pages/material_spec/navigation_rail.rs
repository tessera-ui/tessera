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
    material_icons::filled,
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

use crate::res::I18N;

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap,
    navigation::{NAMES, destination_icon, navigation_color_roles},
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
        .accessibility_label(I18N.text("ui-mail-rail-navigation"))
        .on_close_request(move || modal_open.set(false))
        .main_content_shared(main_content)
        .header(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label(I18N.text("ui-close-modal-navigation-rail"))
                .on_click(move || modal_open.set(false));
        })
        .content(move |rail_scope| {
            for (index, name) in NAMES.into_iter().enumerate() {
                navigation_rail_item(rail_scope)
                    .label(I18N.text(name))
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
            "ui-navigation-rail",
            "ui-navigation-rails-let-people-switch-between-ui-views-on-mid-sized-devices",
            vec![
                SpecSection {
                    title: "ui-variants",
                    content: RenderSlot::new(variants),
                },
                SpecSection {
                    title: "ui-configurations",
                    content: RenderSlot::new(configurations),
                },
                SpecSection {
                    title: "ui-tokens-specs",
                    content: RenderSlot::new(tokens),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "ui-color",
                    content: RenderSlot::new(colors),
                },
                SpecSection {
                    title: "ui-states",
                    content: RenderSlot::new(states),
                },
                SpecSection {
                    title: "ui-measurements",
                    content: RenderSlot::new(measurements),
                },
                SpecSection {
                    title: "ui-common-layouts",
                    content: RenderSlot::new(layouts),
                },
                SpecSection {
                    title: "ui-baseline-navigation-rail",
                    content: RenderSlot::new(baseline),
                },
            ],
        )
        .route_optional(route.clone())
        .pages(vec![
            ArticlePage {
                title: "ui-overview",
                intro: Some(RenderSlot::new(overview)),
                sections: vec![
                    SpecSection {
                        title: "ui-availability-resources",
                        content: RenderSlot::new(availability),
                    },
                    SpecSection {
                        title: "ui-m3-expressive-update",
                        content: RenderSlot::new(expressive_update),
                    },
                    SpecSection {
                        title: "ui-differences-from-m2",
                        content: RenderSlot::new(differences),
                    },
                ],
            },
            ArticlePage {
                title: "ui-guidelines",
                intro: None,
                sections: vec![
                    SpecSection {
                        title: "ui-usage",
                        content: RenderSlot::new(usage),
                    },
                    SpecSection {
                        title: "ui-anatomy",
                        content: RenderSlot::new(guide_anatomy),
                    },
                    SpecSection {
                        title: "ui-placement",
                        content: RenderSlot::new(placement),
                    },
                    SpecSection {
                        title: "ui-adaptive-design",
                        content: RenderSlot::new(adaptive_design),
                    },
                    SpecSection {
                        title: "ui-behavior",
                        content: RenderSlot::new(move || {
                            behavior(modal_open, modal_selected).session_optional(session);
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
                        content: RenderSlot::new(use_cases),
                    },
                    SpecSection {
                        title: "ui-interaction-style",
                        content: RenderSlot::new(interaction_style),
                    },
                    SpecSection {
                        title: "ui-text-scaling-and-truncation",
                        content: RenderSlot::new(text_scaling),
                    },
                    SpecSection {
                        title: "ui-initial-focus",
                        content: RenderSlot::new(initial_focus),
                    },
                    SpecSection {
                        title: "ui-visual-indicators",
                        content: RenderSlot::new(visual_indicators),
                    },
                    SpecSection {
                        title: "ui-keyboard-navigation",
                        content: RenderSlot::new(keyboard),
                    },
                    SpecSection {
                        title: "ui-labeling-elements",
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
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
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
            text()
                .content(I18N.text(&(label)))
                .style(type_style(16.0, 24.0));
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

fn format_rail_status(label: &str, selected: usize) -> String {
    let name = NAMES.get(selected).copied().unwrap_or(NAMES[0]);
    I18N.format(
        "ui-value-destination-value",
        [
            ("v0", I18N.text(&label).into()),
            ("v1", I18N.text(&name).into()),
        ],
    )
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
        "ui-use-navigation-rails-in-medium-expanded-large-or-extra-large-window-sizes".into(),
        "ui-can-contain-3-7-destinations-plus-an-optional-fab".into(),
        "ui-always-put-the-rail-in-the-same-place-even-on-different-screens-of-an-app".into(),
    ]);
    gap(Dp(32.0));
    figures::scene(
        "ui-collapsed-and-expanded-navigation-rails-on-a-tablet-and-a-phone",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen(
                "ui-expanded-navigation-rail-on-a-tablet",
                Dp(560.0),
                Dp(480.0),
            )
            .expanded(true)
            .fab(true);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen(
                "ui-collapsed-navigation-rail-on-a-phone",
                Dp(220.0),
                Dp(480.0),
            )
            .fab(true)
            .content_kind(1);
        }),
    );
    gap(Dp(16.0));
    caption(
        "ui-collapsed-and-expanded-navigation-rails-can-transition-between-each-other-on-any-device-in",
    );
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
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-jetpack-compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-jetpack-compose-expressive".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-android-views-mdc-android".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-android-views-mdc-android-expressive".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-web".into(),
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-web-expressive".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-this-page-documents-the-native-navigation-rail-and-navigation-rail-item-components-th",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "ui-may-2025-a-collapsed-and-an-expanded-navigation-rail-replaced-the-baseline-navigation-rail",
    );
    gap(Dp(24.0));
    subheading("ui-variants-and-naming".into());
    bullets(vec![
        "ui-baseline-navigation-rail-is-no-longer-recommended".into(),
        "ui-added-a-collapsed-navigation-rail-that-replaces-the-baseline-rail".into(),
        "ui-added-an-expanded-navigation-rail-that-replaces-the-navigation-drawer".into(),
    ]);
    gap(Dp(24.0));
    subheading("ui-configurations".into());
    bullets(vec![
        "ui-expanded-rail-modality-non-modal-or-modal".into(),
        "ui-expanded-behavior-transition-to-collapsed-rail-or-hide-when-collapsed".into(),
    ]);
    gap(Dp(24.0));
    subheading("ui-color".into());
    bullets(vec![
        "ui-active-label-on-vertical-items-changed-from-on-surface-variant-to-secondary".into(),
    ]);
    gap(Dp(32.0));
    figures::scene(
        "ui-collapsed-and-expanded-navigation-rails-on-timer-screens",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("ui-collapsed-timer-navigation-rail", Dp(300.0), Dp(440.0))
                .fab(true)
                .selected(2);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("ui-expanded-timer-navigation-rail", Dp(400.0), Dp(440.0))
                .expanded(true)
                .fab(true)
                .selected(2);
        }),
    );
    gap(Dp(16.0));
    caption(
        "ui-the-collapsed-and-expanded-navigation-rails-match-visually-and-can-transition-into-each-ot",
    );
}

#[tessera]
fn differences() {
    bullets(vec![
        "ui-behavior-predictive-back-interaction".into(),
        "ui-color-new-color-mappings-and-compatibility-with-dynamic-color".into(),
        "ui-states-the-active-destination-can-be-indicated-with-a-pill-shape-in-a-contrasting-color"
            .into(),
    ]);
    gap(Dp(32.0));
    figures::pair(
        RenderSlot::new(|| {
            figures::example(
                "ui-m2-navigation-rail",
                0,
                "ui-m2-the-navigation-rail-uses-icon-color-weight-and-fill-to-communicate-which-destination-is",
                RenderSlot::new(|| {
                    figures::screen("ui-m2-navigation-rail-with-photos", Dp(340.0), Dp(260.0))
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
                "ui-m3-navigation-rail",
                0,
                "ui-m3-the-navigation-rail-uses-a-pill-shaped-active-indicator-to-communicate-which-destinatio",
                RenderSlot::new(|| {
                    figures::screen("ui-m3-navigation-rail-with-photos", Dp(340.0), Dp(260.0))
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
        "ui-collapsed-and-expanded-navigation-rail-variants",
        Dp(560.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    figures::number(1);
                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                    figures::specimen(
                        "ui-collapsed-navigation-rail-variant",
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
                        "ui-expanded-navigation-rail-variant",
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
        "ui-collapsed-navigation-rail".into(),
        "ui-expanded-navigation-rail".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-baseline-variants".into());
    prose(
        "ui-the-baseline-navigation-rail-is-no-longer-recommended-and-should-be-replaced-by-the-collap",
    );
    gap(Dp(24.0));
    figures::canvas(
        "ui-baseline-navigation-rail",
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
    legend(
        1,
        "ui-the-baseline-navigation-rail-is-no-longer-recommended",
    );
    gap(Dp(32.0));
    spec_table(
        vec!["ui-variant".into(), "M3".into(), "ui-m3-expressive".into()],
        vec![
            vec![
                "ui-collapsed-navigation-rail".into(),
                "—".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-expanded-navigation-rail".into(),
                "—".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-navigation-rail-baseline".into(),
                "ui-available".into(),
                "ui-not-recommended-use-collapsed-navigation-rail".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    prose(
        "ui-in-compact-windows-use-a-navigation-bar-instead-of-a-rail-from-the-medium-breakpoint-up-th",
    );
    gap(Dp(32.0));
    figures::canvas(
        "ui-expanded-standard-and-modal-navigation-rails-in-dark-theme",
        Dp(540.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(|| {
                    figures::number(1);
                    spacer().modifier(Modifier::new().width(Dp(12.0)));
                    figures::screen("ui-expanded-standard-navigation-rail", Dp(440.0), Dp(440.0))
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
                    figures::screen("ui-expanded-modal-navigation-rail", Dp(440.0), Dp(440.0))
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
        "ui-expanded-layout-standard".into(),
        "ui-expanded-layout-modal".into(),
    ]);
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-category".into(),
            "ui-configuration".into(),
            "M3".into(),
            "ui-m3-expressive".into(),
        ],
        vec![
            vec![
                "ui-expanded-layout".into(),
                "ui-standard-default".into(),
                "ui-available-as-navigation-drawer".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-expanded-layout".into(),
                "ui-modal".into(),
                "ui-available-as-modal-navigation-drawer".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-expanded-behavior".into(),
                "ui-hide-when-collapsed".into(),
                "—".into(),
                "ui-available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "ui-browse-the-component-elements-attributes-tokens-and-their-values-color-values-below-are-th",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-container-width-collapsed".into(),
                "md.comp.navigation-rail.container.width".into(),
                "96dp".into(),
            ],
            vec![
                "ui-container-width-expanded".into(),
                "md.comp.navigation-rail-expanded.container.width".into(),
                "220–360dp".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "md.comp.navigation-rail.active-indicator".into(),
                "ui-56-32dp-16dp-radius".into(),
            ],
            vec![
                "ui-active-indicator-color".into(),
                "md.comp.navigation-rail.active-indicator.color".into(),
                "ui-secondary-container-e8def8".into(),
            ],
            vec![
                "ui-active-icon-color".into(),
                "md.comp.navigation-rail.active-icon.color".into(),
                "ui-on-secondary-container-4a4458".into(),
            ],
            vec![
                "ui-active-label-text-color".into(),
                "md.comp.navigation-rail.active-label-text.color".into(),
                "ui-secondary-625b71".into(),
            ],
            vec![
                "ui-inactive-icon-color".into(),
                "md.comp.navigation-rail.inactive-icon.color".into(),
                "ui-on-surface-variant-49454f".into(),
            ],
            vec![
                "ui-inactive-label-text-color".into(),
                "md.comp.navigation-rail.inactive-label-text.color".into(),
                "ui-on-surface-variant-49454f".into(),
            ],
            vec![
                "ui-icon-size".into(),
                "md.comp.navigation-rail.icon.size".into(),
                "24dp".into(),
            ],
            vec![
                "ui-label".into(),
                "md.comp.navigation-rail.label-text".into(),
                "ui-label-medium-vertical-label-large-horizontal".into(),
            ],
            vec![
                "ui-state-layer".into(),
                "md.comp.navigation-rail.state-layer".into(),
                "ui-hover-8-focus-10-press-10".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-published-token-set-groups-container-and-item-values-by-interaction-state-enabled-hove",
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
        "ui-navigation-rail-anatomy",
        Dp(620.0),
        RenderSlot::new(move || {
            let collapsed = collapsed.clone();
            let expanded = expanded.clone();
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    figures::specimen("ui-collapsed-rail-anatomy", false, Dp(200.0), Dp(600.0))
                        .menu(true)
                        .fab(true)
                        .count(count)
                        .badges(true)
                        .badge_start(1)
                        .filled(filled)
                        .callouts(collapsed.clone());
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("ui-expanded-rail-anatomy", true, Dp(324.0), Dp(600.0))
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
    prose("ui-collapsed-and-expanded-navigation-rail-elements");
    gap(Dp(16.0));
    legend_list(vec![
        "ui-container".into(),
        "ui-menu-optional".into(),
        "ui-fab-or-extended-fab-optional".into(),
        "ui-icon".into(),
        "ui-active-indicator".into(),
        "ui-label-text".into(),
        "ui-large-badge-optional".into(),
        "ui-large-badge-label-optional".into(),
        "ui-small-badge-optional".into(),
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
    let mode = if dark { "ui-dark" } else { "ui-light" };
    // The published dark diagram uses the baseline surface pair; light keeps
    // the native Material 2025 surfaces.
    let (background, container) = if dark {
        (Some(srgb(20, 18, 24)), Some(srgb(33, 31, 38)))
    } else {
        (None, None)
    };
    figures::canvas(
        I18N.format(
            "ui-value-navigation-rail-colors",
            [("v0", I18N.text(&mode).into())],
        ),
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
                            I18N.format(
                                "ui-value-value-reference-rail",
                                [
                                    ("v0", I18N.text(&mode).into()),
                                    (
                                        "v1",
                                        I18N.text(
                                            &(if expanded {
                                                "ui-expanded-7100"
                                            } else {
                                                "ui-collapsed-7101"
                                            }),
                                        )
                                        .into(),
                                    ),
                                ],
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
        "ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo",
    );
    gap(Dp(24.0));
    rail_color_pair();
    gap(Dp(16.0));
    caption(
        "ui-the-dark-reference-matches-the-published-baseline-surface-141218-and-container-211f26-the",
    );
    gap(Dp(24.0));
    prose("ui-navigation-rail-color-roles-used-for-light-and-dark-schemes");
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
            "ui-expanded-navigation-rail-states"
        } else {
            "ui-collapsed-navigation-rail-states"
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
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-an-inte",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-navigation-item-s-target-area-always-spans-the-full-width-of-the-nav-rail-even-if-the",
    );
    gap(Dp(32.0));
    state_grid(false);
    gap(Dp(32.0));
    state_grid(true);
    gap(Dp(24.0));
    legend_list(vec![
        "ui-enabled".into(),
        "ui-hovered".into(),
        "ui-focused".into(),
        "ui-pressed".into(),
    ]);
    gap(Dp(24.0));
    prose(
        "ui-the-native-component-does-not-expose-a-per-destination-disabled-state-hide-a-destination-o-4179",
    );
}

#[tessera]
fn measurements() {
    figures::canvas(
        "ui-navigation-rail-padding-and-size-measurements",
        Dp(640.0),
        RenderSlot::new(|| {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(|| {
                    figures::specimen(
                        "ui-collapsed-rail-measurements",
                        false,
                        Dp(240.0),
                        Dp(620.0),
                    )
                    .menu(true)
                    .fab(true)
                    .count(4)
                    .measurements(true)
                    .rail_height(Dp(540.0));
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("ui-expanded-rail-measurements", true, Dp(360.0), Dp(620.0))
                        .menu(true)
                        .fab(true)
                        .count(4)
                        .measurements(true)
                        .rail_height(Dp(540.0));
                });
        }),
    );
    gap(Dp(16.0));
    caption("ui-navigation-rail-padding-and-size-measurements");
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-tessera-native".into(),
            "ui-m3-reference".into(),
        ],
        vec![
            vec![
                "ui-container".into(),
                "ui-collapsed-width".into(),
                "96dp".into(),
                "ui-96dp-baseline-rail-80dp".into(),
            ],
            vec![
                "ui-container".into(),
                "ui-expanded-width".into(),
                "ui-220dp-minimum".into(),
                "220–360dp".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "ui-size".into(),
                "56 × 32dp".into(),
                "56 × 32dp".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "ui-corner-radius".into(),
                "ui-capsule-16dp".into(),
                "16dp".into(),
            ],
            vec![
                "ui-icon".into(),
                "ui-size".into(),
                "24dp".into(),
                "24dp".into(),
            ],
            vec![
                "ui-destination".into(),
                "ui-minimum-height".into(),
                "56dp".into(),
                "56dp".into(),
            ],
            vec![
                "ui-destination".into(),
                "ui-minimum-target".into(),
                "48 × 48dp".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "ui-container".into(),
                "ui-top-padding".into(),
                "36dp".into(),
                "36dp".into(),
            ],
            vec![
                "ui-indicator-to-label".into(),
                "ui-gap".into(),
                "ui-4dp-collapsed-icon-top".into(),
                "4dp".into(),
            ],
            vec![
                "ui-expanded-content".into(),
                "ui-icon-to-label-inside-indicator-distinct-from-collapsed-4dp".into(),
                "8dp".into(),
                "8dp".into(),
            ],
            vec![
                "ui-expanded-content".into(),
                "ui-leading-margin-indicator-padding".into(),
                "20dp · 16dp".into(),
                "20dp · 16dp".into(),
            ],
            vec![
                "ui-destinations".into(),
                "ui-recommended-count".into(),
                "3–7".into(),
                "3–7".into(),
            ],
            vec![
                "ui-window".into(),
                "ui-breakpoint".into(),
                "ui-600dp-and-wider".into(),
                "ui-medium-and-larger".into(),
            ],
        ],
    );
}

#[tessera]
fn layout_figure(expanded: bool) {
    figures::canvas(
        if expanded {
            "ui-common-expanded-navigation-rail-layouts"
        } else {
            "ui-common-collapsed-navigation-rail-layouts"
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
                                    I18N.format(
                                        "ui-common-value-layout-value",
                                        [
                                            (
                                                "v0",
                                                I18N.text(
                                                    &(if expanded {
                                                        "ui-expanded-7100"
                                                    } else {
                                                        "ui-collapsed-7101"
                                                    }),
                                                )
                                                .into(),
                                            ),
                                            ("v1", (index + 1).into()),
                                        ],
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
        "ui-three-navigation-items".into(),
        "ui-three-navigation-items-with-a-menu".into(),
        "ui-three-navigation-items-with-a-fab".into(),
        "ui-three-navigation-items-with-a-menu-and-fab".into(),
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
            "ui-dark-baseline-navigation-rail-states"
        } else {
            "ui-light-baseline-navigation-rail-states"
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
        "ui-the-baseline-navigation-rail-is-no-longer-recommended-use-the-collapsed-navigation-rail-do",
    );
    gap(Dp(24.0));
    baseline_canvas(
        "ui-baseline-navigation-rail-anatomy",
        baseline_callouts(false),
        false,
    );
    gap(Dp(24.0));
    legend_list(vec![
        "ui-container".into(),
        "ui-menu-icon-optional".into(),
        "ui-icon".into(),
        "ui-active-indicator".into(),
        "ui-label-text".into(),
        "ui-large-badge-label-optional".into(),
        "ui-large-badge-optional".into(),
        "ui-badge-optional".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-tokens-specs".into());
    spec_table(
        vec![
            "ui-element".into(),
            "ui-baseline".into(),
            "ui-collapsed-rail-4208".into(),
        ],
        vec![
            vec!["ui-container-width".into(), "80dp".into(), "96dp".into()],
            vec![
                "ui-active-label-color".into(),
                "ui-on-surface".into(),
                "ui-secondary".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "56 × 32dp".into(),
                "56 × 32dp".into(),
            ],
            vec!["ui-icon".into(), "24dp".into(), "24dp".into()],
            vec!["ui-destination-height".into(), "56dp".into(), "56dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-color".into());
    baseline_canvas(
        "ui-light-baseline-navigation-rail-colors",
        baseline_callouts(true),
        true,
    );
    gap(Dp(16.0));
    prose("ui-navigation-rail-color-roles-used-for-light-and-dark-schemes");
    gap(Dp(16.0));
    legend_list(
        navigation_color_roles(true, true)
            .into_iter()
            .map(String::from)
            .collect(),
    );
    gap(Dp(32.0));
    subheading("ui-states".into());
    prose(
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-interac",
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
                ["ui-enabled", "ui-hovered", "ui-focused", "ui-pressed"].map(|state| {
                    I18N.format(
                        "ui-value-on-value-destination",
                        [
                            ("v0", I18N.text(&state).into()),
                            ("v1", I18N.text(&target).into()),
                        ],
                    )
                })
            })
            .collect(),
    );
    gap(Dp(32.0));
    subheading("ui-measurements".into());
    figures::pair(
        RenderSlot::new(|| {
            baseline_measurement("ui-navigation-rail-size-measurements", 2);
        }),
        RenderSlot::new(|| {
            baseline_measurement("ui-navigation-rail-padding-and-margin-measurements", 3);
        }),
    );
    gap(Dp(32.0));
    subheading("ui-configurations".into());
    prose("ui-common-arrangements-of-elements-within-a-navigation-rail");
    gap(Dp(24.0));
    figures::canvas(
        "ui-baseline-navigation-rail-configurations",
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
        "ui-with-a-menu".into(),
        "ui-with-a-fab".into(),
        "ui-with-menu-and-fab-without-labels".into(),
        "ui-all-destinations-with-text-labels".into(),
        "ui-with-menu-fab-and-label-text-for-all-destinations".into(),
    ]);
}

#[tessera]
fn usage() {
    figures::scene(
        "ui-navigation-rail-shown-collapsed-and-expanded",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen("ui-collapsed-rail-on-a-timer-screen", Dp(300.0), Dp(480.0))
                .selected(2)
                .fab(true);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("ui-expanded-rail-on-a-timer-screen", Dp(400.0), Dp(480.0))
                .expanded(true)
                .selected(2)
                .fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("ui-use-the-menu-icon-to-transition-between-collapsed-and-expanded-navigation-rails");
    gap(Dp(32.0));
    prose(
        "ui-the-navigation-rail-can-display-navigation-items-a-menu-and-a-floating-action-button-fab-i",
    );
    gap(Dp(16.0));
    prose(
        "ui-there-are-two-variants-of-navigation-rails-collapsed-and-expanded-which-can-easily-transfo",
    );
    gap(Dp(24.0));
    subheading("ui-collapsed".into());
    prose(
        "ui-the-collapsed-nav-rail-runs-along-the-leading-edge-of-the-window-and-should-contain-3-7-na",
    );
    gap(Dp(16.0));
    prose(
        "ui-it-can-be-used-in-medium-to-extra-large-windows-such-as-tablets-and-desktop-in-medium-wind",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-collapsed-navigation-rail-with-a-timer-fab",
        Dp(540.0),
        RenderSlot::new(|| {
            timer_screen(
                "ui-collapsed-navigation-rail-with-a-fab",
                Dp(560.0),
                Dp(480.0),
            )
            .selected(2)
            .fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("ui-a-navigation-rail-should-be-the-only-visible-navigation-element");
    gap(Dp(32.0));
    subheading("ui-expanded".into());
    prose(
        "ui-the-expanded-navigation-rail-can-be-standard-or-modal-and-should-always-open-from-a-menu-i",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-standard-configuration-is-placed-beside-body-content-it-s-best-for-larger-windows-with",
    );
    gap(Dp(8.0));
    bullets(vec![
        "ui-information-dense-layouts-where-space-is-limited".into(),
        "ui-products-with-many-navigation-items".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "ui-expanded-navigation-rail-by-default-and-over-content",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("ui-expanded-by-default", Dp(440.0), Dp(440.0))
                .expanded(true)
                .selected(2)
                .fab(true);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            figures::screen("ui-expanded-over-content", Dp(440.0), Dp(440.0))
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
        "ui-a-navigation-rail-can-be-expanded-by-default-on-larger-screen-sizes-or-can-be-expanded-ove",
    );
    gap(Dp(32.0));
    prose(
        "ui-in-immersive-experiences-the-expanded-navigation-rail-can-be-hidden-entirely-appearing-onl",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-navigation-rail-and-hidden-navigation-rail",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen(
                "ui-hidden-navigation-rail-with-a-menu-icon",
                Dp(440.0),
                Dp(440.0),
            )
            .hidden_menu(true);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            figures::screen(
                "ui-hidden-navigation-rail-expanded-over-content",
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
        "ui-the-expanded-navigation-rail-can-also-be-hidden-appearing-only-when-the-menu-icon-is-selec",
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
        "ui-container".into(),
        "ui-menu-optional".into(),
        "ui-floating-action-button-fab-optional".into(),
        "ui-icon-active".into(),
        "ui-label-text-active".into(),
        "ui-active-indicator".into(),
        "ui-icon-inactive".into(),
        "ui-large-badge-optional".into(),
        "ui-large-badge-label".into(),
        "ui-small-badge".into(),
        "ui-label-text-inactive".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-container".into());
    prose(
        "ui-the-navigation-rail-should-be-placed-on-the-leading-edge-of-the-window-this-is-the-left-si",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-container-fill-can-be-turned-off-so-the-nav-rail-appears-directly-on-the-surface-when",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-left-to-right-navigation-rail",
                0,
                "ui-left-to-right-layouts-place-the-rail-on-the-left-edge",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-right-to-left-navigation-rail",
                0,
                "ui-right-to-left-layouts-place-the-rail-on-the-right-edge",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            )
            .trailing(true);
        }),
    );
    gap(Dp(16.0));
    caption("ui-the-navigation-rail-should-be-placed-on-the-leading-edge-of-the-window");
    gap(Dp(32.0));
    prose(
        "ui-the-navigation-rail-should-always-run-vertically-along-the-side-of-a-layout-don-t-make-it",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-vertical-navigation-rail",
                1,
                "ui-run-the-navigation-rail-vertically-along-the-side-of-the-layout",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-horizontal-navigation-rail",
                3,
                "ui-don-t-use-the-navigation-rail-horizontally-use-a-navigation-bar-instead",
                RenderSlot::empty(),
            )
            .bar(2);
        }),
    );
    gap(Dp(32.0));
    prose(
        "ui-navigation-rail-items-can-be-aligned-as-a-group-to-the-top-or-center-of-a-layout-on-tablet",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-top-aligned-rail-destinations",
                0,
                "ui-top-aligned-rail-destination-placement",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).menu(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-center-aligned-rail-destinations",
                0,
                "ui-center-aligned-rail-destination-placement",
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
    subheading("ui-menu-optional".into());
    prose(
        "ui-the-menu-button-can-transition-between-the-collapsed-and-expanded-navigation-rails-once-ex",
    );
    gap(Dp(16.0));
    prose(
        "ui-when-the-navigation-rail-is-expanded-the-menu-icon-should-change-to-represent-that-it-can",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-expanded-and-collapsed-navigation-rails-controlled-by-a-menu-icon-button",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen("ui-collapsed-rail-with-a-menu", Dp(440.0), Dp(440.0)).count(4);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            timer_screen(
                "ui-expanded-rail-with-secondary-destinations",
                Dp(440.0),
                Dp(440.0),
            )
            .expanded(true)
            .count(5);
        }),
    );
    gap(Dp(16.0));
    caption("ui-a-navigation-rail-can-expand-to-reveal-more-destinations");
    gap(Dp(32.0));
    subheading("ui-floating-action-button-fab-optional".into());
    prose(
        "ui-the-container-of-the-navigation-rail-is-ideal-for-anchoring-the-fab-to-the-top-of-a-screen",
    );
    gap(Dp(16.0));
    prose(
        "ui-when-nested-within-another-component-such-as-the-navigation-rail-the-fab-s-resting-elevati",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-fab-at-the-top-of-the-rail",
                1,
                "ui-a-top-aligned-fab-in-the-navigation-rail",
                RenderSlot::new(|| {
                    figures::mock_rail().count(3).selected(1).fab(true);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-fab-at-the-bottom-of-the-rail",
                3,
                "ui-avoid-placing-the-fab-below-navigation-items",
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
        "ui-the-top-of-the-rail-can-also-be-used-for-a-logo-however-avoid-using-logos-that-could-be-mi",
    );
    gap(Dp(24.0));
    card(
        "ui-logo-at-the-top-of-the-rail",
        2,
        "ui-use-caution-when-placing-logos-in-the-rail-where-they-might-be-confused-with-an-action-or",
        RenderSlot::new(|| {
            figures::mock_rail().count(3).selected(1).logo(true);
        }),
    );
    gap(Dp(32.0));
    subheading("ui-active-indicator".into());
    prose("ui-the-active-indicator-shows-which-page-is-being-displayed");
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-one-active-indicator",
                1,
                "ui-use-the-active-indicator-only-for-the-current-open-page",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-active-indicators-on-every-item",
                3,
                "ui-don-t-use-the-active-indicator-for-more-than-one-navigation-item-at-a-time",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).all_active(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose(
        "ui-the-active-indicator-hugs-the-label-text-in-the-expanded-nav-rail-to-achieve-a-similar-sty",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-hugging-active-indicator",
                0,
                "ui-the-active-indicator-hugs-contents-in-the-expanded-nav-rail",
                RenderSlot::new(|| {
                    figures::mock_rail().expanded(true).count(4).selected(1);
                }),
            )
            .width(Dp(380.0));
        }),
        RenderSlot::new(|| {
            card(
                "ui-filled-active-indicator",
                0,
                "ui-override-the-indicator-to-fill-the-container-to-more-closely-resemble-the-baseline-navigat",
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
    subheading("ui-icons".into());
    prose("ui-navigation-rail-items-must-use-icons-that-symbolize-the-content-of-their-page");
    gap(Dp(24.0));
    card(
        "ui-icons-for-timer-destinations",
        0,
        "ui-icons-should-symbolize-the-content-of-the-page-they-open",
        RenderSlot::new(|| {
            figures::mock_rail().count(4).selected(2);
        }),
    );
    gap(Dp(32.0));
    prose(
        "ui-when-a-destination-is-selected-the-icon-fills-and-changes-color-an-active-indicator-appear",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-destinations-without-an-active-indicator",
                0,
                "ui-unselected-navigation-items-use-outlined-icons",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-destination-with-an-active-indicator",
                0,
                "ui-selected-navigation-items-have-an-active-indicator-a-filled-icon-and-a-more-prominent-colo",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    subheading("ui-label-text".into());
    prose(
        "ui-the-label-text-should-be-a-short-meaningful-description-of-each-navigation-destination-and",
    );
    gap(Dp(16.0));
    prose(
        "ui-avoid-wrapping-long-labels-when-possible-if-necessary-create-a-line-break-between-words-or",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-clear-navigation-labels",
                1,
                "ui-write-clear-and-concise-labels-that-describe-the-destination-page",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1);
                }),
            )
            .dark(true);
        }),
        RenderSlot::new(|| {
            card(
                "ui-two-line-navigation-labels",
                2,
                "ui-break-up-longer-phrases-into-two-text-lines-if-necessary",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(4);
                }),
            )
            .dark(true);
        }),
    );
    gap(Dp(32.0));
    prose(
        "ui-labels-should-be-short-enough-to-not-be-truncated-don-t-shrink-the-type-scale-to-fit-longe",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-truncated-navigation-labels",
                3,
                "ui-don-t-truncate-or-display-an-ellipsis-in-place-of-label-text",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(2);
                }),
            )
            .dark(true);
        }),
        RenderSlot::new(|| {
            card(
                "ui-small-navigation-labels",
                3,
                "ui-don-t-reduce-the-type-size-to-fit-more-characters-into-a-destination-label",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).label_mode(3);
                }),
            )
            .dark(true);
        }),
    );
    gap(Dp(32.0));
    subheading("ui-badges".into());
    prose(
        "ui-navigation-rail-icons-can-include-badges-to-communicate-dynamic-information-about-the-dest",
    );
    gap(Dp(16.0));
    prose(
        "ui-in-compact-nav-rails-the-badge-is-placed-in-the-upper-right-corner-of-the-icon-in-expanded",
    );
    gap(Dp(24.0));
    figures::canvas(
        "ui-navigation-rail-badges",
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
                    figures::specimen("ui-collapsed-rail-with-badges", false, Dp(220.0), Dp(340.0))
                        .count(4)
                        .badge_series(true)
                        .filled(true)
                        .callouts(callouts.clone())
                        .rail_height(Dp(320.0));
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    figures::specimen("ui-expanded-rail-with-badges", true, Dp(340.0), Dp(340.0))
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
        "ui-small-badge-on-a-rail-destination".into(),
        "ui-large-badge-with-a-number".into(),
        "ui-large-badge-with-a-maximum-character-count".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-divider-optional".into());
    prose(
        "ui-a-vertical-divider-can-help-separate-the-rail-from-app-content-the-divider-should-be-posit",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-navigation-rail-with-a-divider",
        Dp(460.0),
        RenderSlot::new(|| {
            figures::screen(
                "ui-navigation-rail-with-a-divider-beside-a-list",
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
        "ui-a-divider-can-make-the-navigation-rail-container-distinct-from-other-on-screen-content",
    );
}

#[tessera]
fn placement() {
    prose(
        "ui-in-adaptive-layouts-the-navigation-rail-should-be-placed-outside-any-panes-always-along-th",
    );
    gap(Dp(16.0));
    prose(
        "ui-when-the-navigation-rail-is-hidden-the-body-content-can-fill-in-the-remaining-space-as-lon",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-rail-outside-body-panes",
                0,
                "ui-place-the-navigation-rail-outside-panes-along-the-leading-edge-of-the-window",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(1).menu(true);
                }),
            )
            .content_kind(2);
        }),
        RenderSlot::new(|| {
            card(
                "ui-hidden-rail-with-a-menu-icon",
                0,
                "ui-body-content-fills-the-window-while-the-menu-icon-stays-accessible",
                RenderSlot::empty(),
            )
            .content_kind(2)
            .hidden(true);
        }),
    );
    gap(Dp(32.0));
    prose(
        "ui-tabs-can-be-used-alongside-a-navigation-rail-to-create-an-extra-layer-of-visible-navigatio",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-expanded-navigation-rail-opened-from-a-menu-button-on-mobile",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen("ui-modal-expanded-rail-on-a-phone", Dp(300.0), Dp(440.0))
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
    caption("ui-expanded-navigation-rails-can-open-from-menu-buttons-on-mobile");
}

#[tessera]
fn adaptive_design() {
    prose("ui-for-more-see-adaptive-design");
    gap(Dp(24.0));
    subheading("ui-resizing".into());
    prose(
        "ui-when-moving-from-a-large-screen-to-a-small-screen-a-navigation-rail-can-transform-into-a-n",
    );
    gap(Dp(16.0));
    prose(
        "ui-only-use-navigation-rails-for-medium-breakpoints-and-larger-don-t-use-a-navigation-bar-if",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-compact-don-t-use-a-standard-navigation-rail-for-compact-layouts-due-to-space-constraints".into(),
        "ui-medium-use-a-navigation-rail-especially-if-prioritizing-persistent-vertical-navigation-ove".into(),
        "ui-expanded-to-extra-large-use-a-navigation-rail-not-a-navigation-bar-consider-available-hori".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "ui-navigation-bar-on-a-phone-and-navigation-rail-on-a-tablet",
        Dp(500.0),
        RenderSlot::new(|| {
            figures::screen("ui-phone-with-a-navigation-bar", Dp(240.0), Dp(440.0)).bar(1);
            spacer().modifier(Modifier::new().width(Dp(32.0)));
            timer_screen("ui-tablet-with-a-navigation-rail", Dp(600.0), Dp(440.0)).fab(true);
        }),
    );
    gap(Dp(16.0));
    caption("ui-on-smaller-devices-use-a-navigation-bar-on-larger-displays-use-a-navigation-rail");
    gap(Dp(32.0));
    subheading("ui-presentation".into());
    prose(
        "ui-when-the-navigation-rail-transitions-from-collapsed-to-expanded-the-contents-of-the-page-s",
    );
    gap(Dp(16.0));
    prose("ui-extra-destinations-can-be-shown-in-an-expanded-nav-rail");
    gap(Dp(24.0));
    figures::scene(
        "ui-expanded-navigation-rail-with-secondary-destinations",
        Dp(500.0),
        RenderSlot::new(|| {
            timer_screen(
                "ui-standard-expanded-rail-with-extra-destinations",
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
        "ui-use-a-standard-expanded-rail-when-there-are-secondary-destinations-or-actions-that-have-lo",
    );
}

#[tessera]
fn behavior(
    modal_open: State<bool>,
    modal_selected: State<usize>,
    session: Option<LiveRailSession>,
) {
    subheading("ui-modal-expanded-rail".into());
    prose(
        "ui-a-modal-expanded-rail-opens-from-the-menu-and-closes-by-selecting-its-scrim-or-a-destinati",
    );
    gap(Dp(16.0));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            icon_button()
                .standard()
                .icon(filled::MENU_SVG)
                .accessibility_label(I18N.text("ui-open-modal-navigation-rail"))
                .on_click(move || {
                    if session.is_none_or(LiveRailSession::can_open) {
                        modal_open.set(true);
                    }
                });
            text()
                .content(I18N.format(
                    "ui-current-destination-value",
                    [("v0", I18N.text(&(NAMES[modal_selected.get()])).into())],
                ))
                .style(type_style(14.0, 20.0));
        });
    caption(
        "ui-open-from-the-menu-then-choose-a-destination-select-the-scrim-or-press-escape-to-close-and",
    );
    gap(Dp(32.0));
    subheading("ui-scrolling".into());
    prose(
        "ui-destinations-in-the-navigation-rail-should-remain-visible-and-fixed-when-scrolling-vertica",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(scrolling_rail));
    gap(Dp(16.0));
    caption("ui-rail-destinations-remain-fixed-while-on-screen-content-scrolls-vertically");
    gap(Dp(32.0));
    prose(
        "ui-if-a-layout-scrolls-horizontally-the-rail-can-scroll-off-screen-or-remain-fixed-to-disting",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(move || {
            card(
                "ui-rail-with-a-divider-over-horizontal-content",
                0,
                "ui-a-divider-and-color-fill-change-create-visual-distinction-between-the-rail-and-horizontall",
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
                "ui-elevated-rail-over-horizontal-content",
                0,
                "ui-elevating-the-rail-to-level-1-creates-visual-distinction-between-the-rail-and-horizontally",
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
    subheading("ui-selection".into());
    prose(
        "ui-when-a-destination-is-tapped-the-destination-screen-uses-a-top-level-transition-pattern-in",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-menu-control-transitions-the-rail-between-collapsed-and-expanded-selection-is-preserve",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(behavior_rail));
    gap(Dp(16.0));
    caption(
        "ui-select-a-destination-then-use-the-menu-button-to-expand-or-collapse-the-rail-the-selected",
    );
    gap(Dp(32.0));
    subheading("ui-back".into());
    prose(
        "ui-on-android-a-gesture-called-predictive-back-allows-people-to-swipe-left-or-right-on-the-sc",
    );
    gap(Dp(8.0));
    bullets(vec![
        "ui-previous-screen-is-revealed-in-a-preview-to-signal-the-destination".into(),
        "ui-predictive-back-only-applies-to-the-modal-expanded-navigation-rail".into(),
    ]);
    gap(Dp(24.0));
    figures::scene(
        "ui-modal-navigation-rail-during-the-predictive-back-gesture",
        Dp(460.0),
        RenderSlot::new(move || {
            figures::screen(
                "ui-predictive-back-on-a-modal-navigation-rail",
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
    caption("ui-the-nav-rail-pops-off-the-edge-of-the-window-during-the-predictive-back-gesture");
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
                    label: Some(I18N.text("ui-interactive-mail-navigation")),
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
                                .accessibility_label(I18N.text(if expanded.get() {
                                    "ui-collapse-navigation-rail"
                                } else {
                                    "ui-expand-navigation-rail"
                                }))
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
                                            .label(I18N.text(name))
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
                .content(I18N.text(format_rail_status(
                    if expanded.get() {
                        "ui-expanded"
                    } else {
                        "ui-collapsed"
                    },
                    selected.get(),
                )))
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
                        label: Some(I18N.text("ui-fixed-mail-navigation-beside-scrolling-content")),
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
                                    .accessibility_label(I18N.text(if expanded.get() {
                                        "ui-collapse-fixed-rail"
                                    } else {
                                        "ui-expand-fixed-rail"
                                    }))
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
                                                .label(I18N.text(name))
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
                                        .content(I18N.format(
                                            "ui-value-message-value",
                                            [
                                                ("v0", I18N.text(&(NAMES[selected.get()])).into()),
                                                ("v1", (index + 1).into()),
                                            ],
                                        ))
                                        .style(type_style(14.0, 20.0));
                                });
                            gap(Dp(12.0));
                        }
                    });
            });
        gap(Dp(16.0));
        text()
            .content(I18N.text(format_rail_status("ui-fixed-rail", selected.get())))
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
    prose("ui-people-should-be-able-to-do-the-following-using-the-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-between-navigation-destinations".into(),
        "ui-select-a-particular-navigation-destination-from-a-set".into(),
        "ui-get-appropriate-feedback-based-on-input-type".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-when-a-navigation-item-is-tapped-the-active-indicator-appears-providing-the-following-feed",
    );
    gap(Dp(8.0));
    bullets(vec![
        "ui-a-ripple-passes-through-the-indicator".into(),
        "ui-the-icon-switches-from-outlined-to-filled".into(),
        "ui-the-icon-and-text-change-color".into(),
    ]);
    gap(Dp(8.0));
    prose(
        "ui-when-hovered-the-hover-state-appears-providing-a-visual-cue-that-the-destination-is-intera",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-touch-tap-on-a-rail-destination",
                0,
                "ui-touch-tap",
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
                "ui-cursor-hover-on-a-rail-destination",
                0,
                "ui-cursor-hover-click",
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
        "ui-the-target-area-for-expanded-navigation-rails-spans-the-full-width-of-the-container-even-t",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-full-width-target-area-in-an-expanded-rail",
        Dp(420.0),
        RenderSlot::new(|| {
            figures::screen("ui-expanded-rail-target-area", Dp(560.0), Dp(380.0)).rail(|| {
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
    caption("ui-touch-tap");
    gap(Dp(32.0));
    prose(
        "ui-use-a-filled-icon-for-the-active-destination-and-outlined-icons-for-inactive-destinations",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-default-navigation-rail-colors",
                1,
                "ui-use-the-default-color-scheme-to-ensure-proper-contrast-and-emphasis-on-the-active-destinat",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-low-contrast-navigation-rail-colors",
                3,
                "ui-don-t-use-more-than-two-colors-for-destinations-or-low-contrast-colors-in-the-navigation-r",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2).low_contrast(true);
                }),
            );
        }),
    );
    gap(Dp(32.0));
    prose("ui-if-an-icon-doesn-t-have-a-filled-style-use-the-semibold-icon-weight-instead");
    gap(Dp(24.0));
    figures::scene(
        "ui-voicemail-icon-inactive-and-active",
        Dp(220.0),
        RenderSlot::new(voicemail_weights),
    );
    gap(Dp(16.0));
    caption("ui-an-icon-with-no-filled-option-should-use-the-semibold-weight-when-active");
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
                            .content(I18N.text("ui-voicemail"))
                            .style(style.clone())
                            .color(scheme.on_surface);
                    });
            }
        });
}

#[tessera]
fn text_scaling() {
    prose(
        "ui-when-someone-sets-their-device-to-show-a-larger-text-size-the-navigation-rail-items-should",
    );
    gap(Dp(16.0));
    prose(
        "ui-to-remain-accessible-ensure-the-full-label-is-always-visible-on-screen-at-up-to-2x-text-si",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-text-scaled-to-1-5-size",
                0,
                "ui-text-scaled-to-1-5-size",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2).label_scale(1.5);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-text-scaled-to-2x-size",
                0,
                "ui-text-scaled-to-2x-size",
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
        "ui-initial-focus-lands-directly-on-the-first-interactive-item-whether-it-s-the-menu-the-fab-o",
    );
    gap(Dp(16.0));
    prose(
        "ui-from-the-fab-or-menu-tab-brings-the-person-to-the-navigation-items-tab-or-arrows-then-navi",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            figures::example(
                "ui-arrow-keys-move-focus-between-rail-destinations",
                0,
                "ui-use-arrows-to-move-between-navigation-items",
                RenderSlot::new(|| {
                    row().children(|| {
                        focus_example("ui-tab", 0, 0, 2);
                        spacer().modifier(Modifier::new().width(Dp(40.0)));
                        focus_example("ui-arrow", 1, 0, 2);
                    });
                }),
            )
            .height(Dp(380.0));
        }),
        RenderSlot::new(|| {
            figures::example(
                "ui-space-or-enter-activates-the-focused-destination",
                0,
                "ui-use-space-enter-to-activate-the-focused-navigation-item",
                RenderSlot::new(|| {
                    row().children(|| {
                        focus_example("ui-tab", 0, 0, 2);
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
        "ui-icons-give-the-dominant-cue-of-the-navigation-state-use-a-filled-icon-for-the-selected-des",
    );
    gap(Dp(24.0));
    figures::pair(
        RenderSlot::new(|| {
            card(
                "ui-filled-icon-for-the-selected-destination",
                1,
                "ui-use-a-filled-icon-variant-on-the-selected-navigation-item-to-help-differentiate-from-inact",
                RenderSlot::new(|| {
                    figures::mock_rail().count(4).selected(2);
                }),
            );
        }),
        RenderSlot::new(|| {
            card(
                "ui-outlined-icon-for-the-selected-destination",
                3,
                "ui-avoid-using-the-same-unfilled-icon-style-for-both-selected-and-unselected-items-because-it",
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
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-move-focus-into-or-out-of-the-rail".into(),
            ],
            vec![
                "ui-up-down-arrows".into(),
                "ui-move-focus-between-destinations".into(),
            ],
            vec![
                "ui-enter-or-space".into(),
                "ui-activate-the-focused-destination".into(),
            ],
            vec![
                "ui-escape".into(),
                "ui-close-a-modal-expanded-rail-and-restore-focus".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-arrow-keys-move-focus-only-a-destination-changes-only-after-enter-or-space-so-browsing-the",
    );
}

#[tessera]
fn labeling() {
    prose(
        "ui-the-accessibility-label-for-a-navigation-item-is-typically-the-same-as-the-adjacent-text-l",
    );
    gap(Dp(16.0));
    prose(
        "ui-when-the-visible-ui-text-is-ambiguous-accessibility-labels-need-to-be-more-descriptive-for-4411",
    );
    gap(Dp(16.0));
    prose(
        "ui-note-on-android-views-mdc-android-a-more-descriptive-accessibility-label-is-not-available",
    );
    gap(Dp(24.0));
    figures::scene(
        "ui-accessibility-label-for-the-recent-destination",
        Dp(420.0),
        RenderSlot::new(|| {
            figures::label_callout();
            figures::screen(
                "ui-recent-destination-in-a-file-navigation-rail",
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
        "ui-while-the-visible-label-text-reads-recent-the-accessibility-label-for-this-destination-cla",
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
