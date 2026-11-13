//! Navigation bar specifications for primary destinations.
//!
//! ## Usage
//!
//! Compare destination counts, label behavior, badges and selection feedback.

use tessera_components::{
    column::column,
    flow_row::flow_row,
    md3::{
        badge::{badge_with_content, badged_box},
        button::button,
        navigation_bar::{NavigationBarLabelBehavior, navigation_bar, navigation_bar_item},
        text::text,
        theme::{MaterialTheme, material_theme},
    },
};
use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    article_helpers::{bullets, caption, do_dont, legend, prose},
    gap,
    navigation::{NAMES, destination_icon},
    panel, spec_table, type_style,
};

#[tessera]
pub(super) fn navigation_bar_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-navigation-bar",
        "ui-navigation-bars-let-people-switch-between-primary-destinations",
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
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![],
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
                    title: "ui-color",
                    content: RenderSlot::new(guide_color),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive_design),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
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
                    title: "ui-initial-focus",
                    content: RenderSlot::new(initial_focus),
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
}

#[tessera]
fn bar_demo(
    #[prop(into)] label: String,
    count: usize,
    label_behavior: NavigationBarLabelBehavior,
    badge_count: u32,
) {
    let selected = remember(|| 0_usize);
    let caption_text = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.format(
                "ui-value-navigation-bar-example",
                [("v0", I18N.text(&label).into())],
            )),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            navigation_bar().content(move |icon_position| {
                for (index, name) in NAMES.into_iter().take(count).enumerate() {
                    navigation_bar_item()
                        .icon_position(icon_position)
                        .label(I18N.text(name))
                        .label_behavior(label_behavior)
                        .selected(selected.get() == index)
                        .on_click(move || selected.set(index))
                        .icon(move || {
                            if index == 1 && badge_count > 0 {
                                let badge_text = badge_count.to_string();
                                badged_box()
                                    .modifier(Modifier::new().size(Dp(32.0), Dp(32.0)))
                                    .badge(move || {
                                        let badge_text = badge_text.clone();
                                        badge_with_content()
                                            .container_color(
                                                use_context::<MaterialTheme>()
                                                    .unwrap()
                                                    .get()
                                                    .color_scheme
                                                    .error,
                                            )
                                            .modifier(Modifier::new())
                                            .content(move || {
                                                text(I18N.text(&(badge_text)));
                                            });
                                    })
                                    .content(move || {
                                        destination_icon(index, selected.get() == index);
                                    });
                            } else {
                                destination_icon(index, selected.get() == index);
                            }
                        });
                }
            });
            gap(Dp(16.0));
            text(I18N.format(
                "ui-selected-destination",
                [
                    ("caption", I18N.text(&caption_text).into()),
                    ("destination", I18N.text(NAMES[selected.get()]).into()),
                ],
            ))
            .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn overview() {
    prose(
        "ui-navigation-bars-let-people-switch-between-primary-destinations-in-an-app-they-appear-at-th",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-use-for-three-to-five-primary-destinations".into(),
        "ui-keep-the-bar-visible-across-top-level-views".into(),
        "ui-show-the-active-destination-with-an-indicator-and-a-selected-icon".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-overview", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption(
        "ui-a-navigation-bar-shows-the-app-s-primary-destinations-and-highlights-the-active-one",
    );
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        bar_demo("ui-variant", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption("ui-there-is-one-navigation-bar-variant-holding-three-to-five-destinations");
}

#[tessera]
fn configurations() {
    let count = remember(|| 4_usize);
    let badges = remember(|| true);
    let selected_only = remember(|| false);
    column().children(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                button()
                    .tonal()
                    .accessibility_label(I18N.text("ui-cycle-destination-count"))
                    .on_click(move || {
                        count.set(if count.get() == 5 { 3 } else { count.get() + 1 });
                    })
                    .child(move || {
                        text(I18N.format("ui-destinations-value", [("v0", (count.get()).into())]));
                    });
                button()
                    .tonal()
                    .selected(selected_only.get())
                    .accessibility_label(I18N.text("ui-toggle-selected-only-labels"))
                    .on_click(move || selected_only.set(!selected_only.get()))
                    .child(move || {
                        text(I18N.text(if selected_only.get() {
                            "ui-labels-selected-only"
                        } else {
                            "ui-labels-always"
                        }));
                    });
                button()
                    .tonal()
                    .selected(badges.get())
                    .accessibility_label(I18N.text("ui-toggle-inbox-badge"))
                    .on_click(move || badges.set(!badges.get()))
                    .child(move || {
                        text(I18N.text(if badges.get() {
                            "ui-badge-3"
                        } else {
                            "ui-badge-off"
                        }));
                    });
            });
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            bar_demo(
                "ui-configured",
                count.get(),
                if selected_only.get() {
                    NavigationBarLabelBehavior::SelectedOnly
                } else {
                    NavigationBarLabelBehavior::AlwaysShow
                },
                if badges.get() { 3 } else { 0 },
            );
        }));
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-container".into(),
                "md.comp.navigation-bar.container.color".into(),
                "Surface container".into(),
            ],
            vec![
                "ui-container".into(),
                "md.comp.navigation-bar.container.height".into(),
                "ui-64dp-native-80dp-m3-reference".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "md.comp.navigation-bar.active-indicator.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "md.comp.navigation-bar.active-indicator.shape".into(),
                "ui-56-32dp-full-round".into(),
            ],
            vec![
                "ui-active-icon".into(),
                "md.comp.navigation-bar.active-icon.color".into(),
                "On secondary container".into(),
            ],
            vec![
                "ui-active-label".into(),
                "md.comp.navigation-bar.active-label-text.color".into(),
                "Secondary".into(),
            ],
            vec![
                "ui-inactive-icon-and-label".into(),
                "md.comp.navigation-bar.inactive-icon.color".into(),
                "On surface variant".into(),
            ],
            vec![
                "ui-icon-size".into(),
                "md.comp.navigation-bar.icon.size".into(),
                "24dp".into(),
            ],
            vec![
                "ui-label".into(),
                "md.comp.navigation-bar.label-text".into(),
                "Label medium".into(),
            ],
            vec![
                "ui-state-layer".into(),
                "md.comp.navigation-bar.state-layer".into(),
                "ui-hover-8-focus-10-press-10".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    panel(RenderSlot::new(|| {
        bar_demo("ui-anatomy", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-container");
        legend(2, "ui-active-indicator");
        legend(3, "ui-icon");
        legend(4, "ui-active-label");
        legend(5, "ui-inactive-icon");
        legend(6, "ui-inactive-label");
    });
    gap(Dp(16.0));
    caption(
        "ui-the-active-indicator-sits-behind-the-selected-icon-labels-describe-each-destination",
    );
}

#[tessera]
fn colors() {
    prose(
        "ui-navigation-bar-colors-come-from-the-surface-container-role-the-secondary-container-indicat",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-light", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            panel(RenderSlot::new(|| {
                bar_demo("ui-dark", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-color-role".into()],
        vec![
            vec!["ui-container".into(), "Surface container".into()],
            vec!["ui-active-indicator".into(), "Secondary container".into()],
            vec!["ui-active-icon".into(), "On secondary container".into()],
            vec!["ui-active-label".into(), "Secondary".into()],
            vec![
                "ui-inactive-icon-and-label".into(),
                "On surface variant".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "ui-destinations-show-an-active-or-inactive-state-the-selected-destination-gains-the-indicator",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-states", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption("ui-active-and-inactive-destinations-hover-focus-and-press-add-state-layers");
    gap(Dp(24.0));
    prose(
        "ui-the-native-component-does-not-expose-a-per-destination-disabled-state-hide-a-destination-o",
    );
}

#[tessera]
fn measurements() {
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-value".into(),
        ],
        vec![
            vec![
                "ui-bar-container".into(),
                "ui-height".into(),
                "ui-64dp-native-80dp-m3-reference".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "ui-size".into(),
                "ui-56-32dp-native-64-32dp-reference".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "ui-corner-radius".into(),
                "ui-16dp-full-round".into(),
            ],
            vec!["ui-icon".into(), "ui-size".into(), "24dp".into()],
            vec![
                "ui-label".into(),
                "ui-typography".into(),
                "Label medium".into(),
            ],
            vec![
                "ui-destination".into(),
                "ui-minimum-target".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "ui-indicator-to-label".into(),
                "ui-gap".into(),
                "4dp".into(),
            ],
            vec![
                "ui-destinations".into(),
                "ui-recommended-count".into(),
                "3–5".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo(
            "ui-measurements",
            4,
            NavigationBarLabelBehavior::AlwaysShow,
            0,
        );
    }));
    gap(Dp(16.0));
    caption(
        "ui-native-values-are-shown-first-the-m3-reference-values-are-noted-where-the-component-differ",
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-use-a-navigation-bar-for-three-to-five-primary-destinations-that-are-equal-in-importance-p",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-use-three-to-five-primary-destinations",
        "ui-don-t-use-a-navigation-bar-for-secondary-navigation-or-for-actions",
    );
    gap(Dp(24.0));
    prose(
        "ui-keep-the-bar-visible-across-the-app-s-top-level-views-and-keep-each-destination-s-label-st",
    );
    gap(Dp(16.0));
    do_dont(
        "ui-keep-destinations-equal-and-stable",
        "ui-don-t-hide-or-reorder-destinations-between-screens",
    );
}

#[tessera]
fn guide_anatomy() {
    prose("ui-a-navigation-bar-contains-an-active-indicator-icons-and-labels-for-each-destination");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-anatomy", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-container");
        legend(2, "ui-active-indicator");
        legend(3, "ui-icon");
        legend(4, "ui-active-label");
        legend(5, "ui-inactive-icon");
        legend(6, "ui-inactive-label");
    });
    gap(Dp(16.0));
    caption(
        "ui-the-indicator-highlights-the-active-destination-while-other-destinations-stay-inactive",
    );
}

#[tessera]
fn guide_color() {
    prose(
        "ui-use-the-surface-container-color-for-the-bar-so-it-separates-from-the-page-and-rely-on-the",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-primary", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption(
        "ui-the-active-destination-uses-the-secondary-container-indicator-and-on-secondary-container-i",
    );
}

#[tessera]
fn adaptive_design() {
    prose(
        "ui-on-wider-windows-the-icon-can-move-beside-the-label-the-native-component-switches-automati",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-adaptive", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption("ui-above-600dp-the-icon-moves-before-the-label-and-destinations-center-in-the-bar");
    gap(Dp(24.0));
    prose(
        "ui-for-compact-widths-keep-the-icon-above-the-label-and-let-the-bar-span-the-window-avoid-inc",
    );
}

#[tessera]
fn behavior() {
    prose(
        "ui-selecting-a-destination-animates-the-active-indicator-to-that-destination-swaps-its-icon-f",
    );
    gap(Dp(16.0));
    prose(
        "ui-keep-the-bar-s-selected-destination-in-sync-with-the-visible-view-when-a-destination-opens",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-behavior", 4, NavigationBarLabelBehavior::AlwaysShow, 0);
    }));
    gap(Dp(16.0));
    caption("ui-tap-a-destination-to-move-the-indicator-and-swap-the-icon-to-its-filled-variant");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-and-activate-a-destination".into(),
        "ui-identify-the-currently-selected-destination".into(),
        "ui-understand-how-many-destinations-the-bar-contains".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-each-destination-keeps-a-minimum-48-48dp-target-across-the-full-height-of-the-bar-the-acti",
    );
    gap(Dp(16.0));
    caption("ui-every-destination-keeps-a-48dp-target-even-when-the-label-is-hidden");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        bar_demo("ui-targets", 4, NavigationBarLabelBehavior::SelectedOnly, 0);
    }));
}

#[tessera]
fn initial_focus() {
    prose(
        "ui-when-the-bar-receives-focus-focus-lands-on-the-selected-destination-or-on-the-first-destin",
    );
    gap(Dp(16.0));
    prose(
        "ui-moving-focus-between-destinations-does-not-change-the-selected-view-until-a-destination-is",
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-move-focus-between-destinations".into()],
            vec![
                "ui-shift-tab".into(),
                "ui-move-focus-to-the-previous-destination".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-destination".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-traversal-wraps-inside-the-bar-and-stays-horizontal-so-tab-leaves-the-bar-from-either-end",
    );
}

#[tessera]
fn labeling() {
    prose(
        "ui-give-every-destination-a-short-visible-label-the-accessible-name-matches-the-label-and-the",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-label".into(),
            "ui-role".into(),
            "ui-state".into(),
        ],
        vec![
            vec![
                "ui-destination".into(),
                "ui-match-the-visible-label".into(),
                "ui-tab".into(),
                "ui-selected-or-not-selected".into(),
            ],
            vec![
                "ui-position".into(),
                "\u{201c}Home, tab, 1 of 4\u{201d}".into(),
                "ui-tab".into(),
                "ui-selected".into(),
            ],
            vec![
                "ui-badge".into(),
                "ui-append-the-unread-count".into(),
                "ui-tab".into(),
                "ui-3-unread".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption("ui-announce-each-destination-with-its-label-role-and-position-in-the-bar");
}
