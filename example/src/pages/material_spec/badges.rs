use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::{badge, badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    navigation_bar::{
        NavigationBarIconPosition, NavigationBarLabelBehavior, navigation_bar, navigation_bar_item,
    },
    navigation_rail::{navigation_rail, navigation_rail_item},
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    annotations::{self, Geometry, relative, watch},
    article::{ArticlePage, SpecSection, article},
    article_helpers::prose,
    gap, panel, scheme_panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn badges_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-badges",
        "ui-badges-show-notifications-counts-or-status-information-on-navigation-items-and-icons",
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-configuration",
                content: RenderSlot::new(configuration_article),
            },
        ],
    ).route_optional(route.clone())
    .pages(vec![
        ArticlePage {title:"ui-overview",intro:Some(RenderSlot::new(overview_article)),sections:vec![]},
        ArticlePage {title:"ui-guidelines",intro:Some(RenderSlot::new(badge_navigation)),sections:vec![
            SpecSection{title:"ui-usage",content:RenderSlot::new(|| {badge_variants();gap(Dp(24.0));subheading("ui-with-other-components".into());prose("ui-anchor-badges-to-navigation-destinations-app-bars-or-tabs-hide-unread-badges-after-their-d");gap(Dp(24.0));spec_table(vec!["Do".into(),"Don't".into()],vec![vec!["ui-use-a-badge-for-new-or-unread-activity".into(),"ui-use-a-badge-as-a-permanent-decoration".into()],vec!["ui-anchor-inside-the-icon-bounding-box-at-its-upper-trailing-edge".into(),"ui-place-the-badge-over-the-icon-or-outside-its-bounds".into()],vec!["ui-clear-the-badge-once-its-destination-is-selected".into(),"ui-keep-a-count-that-no-longer-reflects-unread-items".into()],vec!["ui-limit-large-badge-content-to-four-characters-including-a".into(),"ui-let-a-badge-label-grow-past-four-characters".into()]]);gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"ui-anatomy",content:RenderSlot::new(|| {badge_figure(true,false);gap(Dp(24.0));prose("ui-1-small-badge");gap(Dp(24.0));badge_figure(false,false);gap(Dp(24.0));prose("ui-2-large-badge-container-3-large-badge-label");})},
            SpecSection{title:"ui-container",content:RenderSlot::new(badge_container)},
            SpecSection{title:"ui-placement",content:RenderSlot::new(|| {prose("ui-anchor-at-the-upper-trailing-edge-inside-the-icon-bounds-use-a-dot-when-space-is-tight-and");gap(Dp(24.0));spec_table(vec!["ui-context".into(),"Do".into(),"ui-avoid".into()],vec![vec!["ui-navigation-rail".into(),"ui-large-badge-with-a-count".into(),"ui-clipping-the-badge".into()],vec!["ui-app-bar-or-tabs".into(),"ui-a-small-dot-when-space-is-constrained".into(),"ui-a-wide-label-that-collides-with-actions".into()],vec!["ui-icon-followed-by-text".into(),"ui-anchor-at-the-icon-s-trailing-edge".into(),"ui-placing-the-badge-over-the-trailing-element".into()]]);gap(Dp(24.0));let selected=remember(|| 0_usize);navigation_examples("ui-rail-placement",selected,3,true,false);})},
        ]},
        ArticlePage{title:"ui-accessibility",intro:None,sections:vec![
            SpecSection{title:"ui-use-cases",content:RenderSlot::new(|| {prose("ui-people-must-be-able-to-understand-unread-counts-and-clear-the-notification-by-visiting-the");gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"ui-interaction-style",content:RenderSlot::new(|| {prose("ui-the-navigation-item-owns-the-action-the-badge-itself-adds-information-without-creating-ano");gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"ui-visual-indicators",content:RenderSlot::new(|| {prose("ui-keep-the-default-color-mapping-so-badges-stand-out-custom-badge-colors-need-at-least-3-1-c");gap(Dp(24.0));badge_figure(false,false);})},
            SpecSection{title:"ui-labeling-elements",content:RenderSlot::new(|| {prose("ui-announce-the-destination-followed-by-its-count-or-new-notification-for-a-dot-do-not-announ");gap(Dp(16.0));prose("ui-a-large-badge-contributes-its-count-to-the-destination-s-accessible-name-such-as-inbox-10");gap(Dp(24.0));spec_table(vec!["ui-element".into(),"ui-accessible-information".into()],vec![vec!["ui-navigation-item".into(),"ui-destination-label-and-selected-state".into()],vec!["ui-large-badge".into(),"ui-count-included-in-the-destination-label".into()],vec!["ui-small-badge".into(),"ui-one-new-notification".into()],vec!["ui-decorative-badge-shape".into(),"ui-hidden-from-assistive-technology".into()]]);gap(Dp(24.0));readable_badge_navigation();})},
        ]},
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            let selected = remember(|| 0_usize);
            navigation_examples("ui-preview", selected, 7, false, false);
        }));
    });
}

#[tessera]
fn badged_icon(kind: usize, count: u32) {
    badged_box()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(I18N.text(if count == 0 {
                "ui-no-unread-items".to_owned()
            } else if kind == 0 {
                "ui-new-activity".to_owned()
            } else {
                I18N.format("ui-value-unread-items", [("v0", (count).into())])
            })),
            ..Default::default()
        }))
        .content(move || {
            icon().painter(
                [
                    filled::HOME_SVG,
                    filled::MAIL_SVG,
                    filled::NOTIFICATIONS_SVG,
                ][kind],
            );
        })
        .badge(move || {
            if count > 0 {
                let hidden = Modifier::new().semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                });
                if kind == 0 {
                    badge().modifier(hidden);
                } else {
                    badge_with_content().modifier(hidden).content(move || {
                        text(I18N.text(if count > 999 {
                            "999+".to_owned()
                        } else {
                            count.to_string()
                        }));
                    });
                }
            }
        });
}

#[tessera]
fn navigation_examples(
    #[prop(into)] label: String,
    selected: State<usize>,
    count: u32,
    rail: bool,
    selected_labels: bool,
) {
    let caption = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.format(
                "ui-value-badge-navigation",
                [("v0", I18N.text(&label).into())],
            )),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if rail {
                boxed()
                    .modifier(Modifier::new().height(Dp(320.0)))
                    .children(move || {
                        navigation_rail().content(move |rail_scope| {
                            for (index, name) in ["ui-home", "ui-inbox", "ui-updates"]
                                .into_iter()
                                .enumerate()
                            {
                                navigation_rail_item(rail_scope)
                                    .label(I18N.text(name))
                                    .selected(selected.get() == index)
                                    .on_click(move || selected.set(index))
                                    .icon(move || {
                                        badged_icon(
                                            index,
                                            if index == 2 && count > 0 { 1000 } else { count },
                                        );
                                    });
                            }
                        });
                    });
            } else {
                navigation_bar()
                    .icon_position(NavigationBarIconPosition::Top)
                    .content(move |icon_position| {
                        for (index, name) in ["ui-home", "ui-inbox", "ui-updates"]
                            .into_iter()
                            .enumerate()
                        {
                            navigation_bar_item()
                                .icon_position(icon_position)
                                .label(I18N.text(name))
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .label_behavior(if selected_labels {
                                    NavigationBarLabelBehavior::SelectedOnly
                                } else {
                                    NavigationBarLabelBehavior::AlwaysShow
                                })
                                .icon(move || {
                                    badged_icon(
                                        index,
                                        if index == 2 && count > 0 { 1000 } else { count },
                                    );
                                });
                        }
                    });
            }
            gap(Dp(16.0));
            text(
                I18N.format(
                    "ui-value-destination-value",
                    [
                        ("v0", I18N.text(&caption).into()),
                        (
                            "v1",
                            I18N.text(&(["ui-home", "ui-inbox", "ui-updates"][selected.get()]))
                                .into(),
                        ),
                    ],
                ),
            )
            .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn specimens() {
    flow_row()
        .item_spacing(Dp(40.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            badge().modifier(Modifier::new().semantics(SemanticsArgs {
                role: Some(Role::Image),
                label: Some(I18N.text("ui-small-badge-specimen")),
                ..Default::default()
            }));
            badge_with_content()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(I18N.text("ui-large-badge-specimen")),
                    ..Default::default()
                }))
                .content(move || {
                    text("7");
                });
            badge_with_content()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(I18N.text("ui-maximum-badge-specimen")),
                    ..Default::default()
                }))
                .content(move || {
                    text("999+");
                });
        });
}

#[tessera]
fn tokens() {
    panel(RenderSlot::new(specimens));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec!["ui-badge-color".into(), "Error".into()],
            vec!["ui-badge-shape".into(), "ui-full".into()],
            vec!["ui-badge-size".into(), "6dp".into()],
            vec!["ui-large-badge-color".into(), "Error".into()],
            vec!["ui-large-badge-shape".into(), "ui-full".into()],
            vec!["ui-large-badge-size".into(), "16dp".into()],
            vec!["ui-large-badge-label-color".into(), "On error".into()],
            vec!["ui-large-badge-label-font".into(), "Roboto".into()],
            vec!["ui-large-badge-label-line-height".into(), "16pt".into()],
            vec!["ui-large-badge-label-size".into(), "11pt".into()],
            vec!["ui-large-badge-label-tracking".into(), "0.5pt".into()],
            vec!["ui-large-badge-label-weight".into(), "500".into()],
        ],
    );
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        scheme_panel(
            dark,
            RenderSlot::new(move || {
                let selected = remember(|| 1_usize);
                navigation_examples(
                    if dark {
                        "ui-dark-navigation-bar"
                    } else {
                        "ui-light-navigation-bar"
                    },
                    selected,
                    7,
                    false,
                    false,
                );
                gap(Dp(24.0));
                navigation_examples(
                    if dark {
                        "ui-dark-navigation-rail"
                    } else {
                        "ui-light-navigation-rail"
                    },
                    selected,
                    3,
                    true,
                    true,
                );
            }),
        );
        gap(Dp(32.0));
    }
    subheading("ui-badge-color-roles".into());
    spec_table(
        vec![
            "ui-placement".into(),
            "ui-container".into(),
            "ui-icon-label".into(),
        ],
        vec![
            vec![
                "ui-navigation-bar".into(),
                "Error".into(),
                "ui-on-error-when-active".into(),
            ],
            vec![
                "ui-navigation-rail".into(),
                "Error".into(),
                "On error".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-small-badge-size-radius".into(), "6dp / 3dp".into()],
            vec!["ui-large-badge-height-radius".into(), "16dp / 8dp".into()],
            vec!["ui-single-digit-width".into(), "16dp".into()],
            vec!["ui-label-horizontal-padding".into(), "4dp".into()],
            vec![
                "ui-large-badge-maximum-label-width-height".into(),
                "34dp / 16dp".into(),
            ],
            vec![
                "ui-small-icon-corner-offset-vertical-horizontal".into(),
                "6 / 6dp".into(),
            ],
            vec![
                "ui-large-icon-corner-offset-vertical-horizontal".into(),
                "14 / 12dp".into(),
            ],
        ],
    );
}

#[tessera]
fn configuration() {
    let selected = remember(|| 0_usize);
    let count = remember(|| 7_u32);
    let rail = remember(|| false);
    let labels = remember(|| false);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .accessibility_label(I18N.text("ui-add-unread-item"))
                .on_click(move || count.set(count.get() + 1))
                .child(move || {
                    text(I18N.text("ui-add-unread"));
                });
            button()
                .text()
                .accessibility_label(I18N.text("ui-clear-unread-items"))
                .on_click(move || count.set(0))
                .child(move || {
                    text(I18N.text("ui-clear"));
                });
            button()
                .tonal()
                .selected(rail.get())
                .accessibility_label(I18N.text("ui-use-badge-navigation-rail"))
                .on_click(move || rail.set(!rail.get()))
                .child(move || {
                    text(I18N.text("ui-rail"));
                });
            button()
                .tonal()
                .selected(labels.get())
                .enabled(!rail.get())
                .accessibility_label(I18N.text("ui-only-selected-bar-label"))
                .on_click(move || labels.set(!labels.get()))
                .child(move || {
                    text(I18N.text("ui-selected-label-only"));
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        navigation_examples(
            "ui-configured",
            selected,
            count.get(),
            rail.get(),
            labels.get(),
        );
    }));
    gap(Dp(24.0));
    text(I18N.format("ui-unread-count-value", [("v0", (count.get()).into())]))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn badge_figure(small: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(240.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            badged_box()
                .content(|| {
                    icon().painter(filled::NOTIFICATIONS_SVG).size(Dp(24.0));
                })
                .badge(move || {
                    if small {
                        badge().modifier(watch(geometry, "badge"));
                    } else {
                        badge_with_content()
                            .modifier(watch(geometry, "badge"))
                            .content(move || {
                                text("3");
                            });
                    }
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    badge_annotations(geometry, small, measurements);
                });
        });
}

#[tessera]
fn badge_annotations(geometry: State<Geometry>, small: bool, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(badge)) = (values.get("figure"), relative(&values, "badge")) else {
        return;
    };
    let mut svg = String::new();
    annotations::outline(&mut svg, badge);
    if measurements {
        annotations::dimension(
            if small { "6" } else { "16" }.into(),
            Dp(badge.x + badge.w + 24.0),
            Dp(badge.y - 4.0),
        );
        annotations::dimension(
            format!("{:.0}", badge.w),
            Dp(badge.x - 4.0),
            Dp(badge.y + badge.h + 28.0),
        );
        annotations::dotted(
            &mut svg,
            badge.x + badge.w,
            badge.y + badge.h / 2.0,
            24.0,
            false,
            "#49454f",
        );
    } else {
        annotations::marker(1, Dp(badge.x + badge.w + 48.0), Dp(badge.y - 40.0));
        let right = badge.x + badge.w + 60.0;
        annotations::dotted(&mut svg, badge.x + badge.w, badge.y, 60.0, false, "#49454f");
        annotations::dotted(&mut svg, right, badge.y, -8.0, true, "#49454f");
        if !small {
            annotations::marker(2, Dp(badge.x - 72.0), Dp(badge.y + 24.0));
            annotations::dotted(
                &mut svg,
                badge.x - 60.0,
                badge.y + badge.h,
                60.0,
                false,
                "#49454f",
            );
            annotations::dotted(
                &mut svg,
                badge.x - 60.0,
                badge.y + badge.h,
                24.0 - badge.h,
                true,
                "#49454f",
            );
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn badge_variants() {
    prose(
        "ui-badges-draw-attention-to-new-activity-a-small-dot-conveys-presence-a-large-badge-shows-a-s",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(272.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(|| {
            flow_row()
                .main_axis_alignment(MainAxisAlignment::Center)
                .item_spacing(Dp(40.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    for number in 1..=3 {
                        badge_overview_destination(number);
                    }
                });
        });
}

#[tessera]
fn badge_overview_destination(number: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().width(Dp(96.0)))
        .children(move || {
            badged_box()
                .content(|| {
                    icon().painter(filled::CHANGE_HISTORY_SVG).size(Dp(24.0));
                })
                .badge(move || {
                    if number == 1 {
                        badge();
                    } else {
                        badge_with_content().content(move || {
                            text(I18N.text(if number == 2 { "1" } else { "999+" }));
                        });
                    }
                });
            gap(Dp(12.0));
            text(I18N.text("ui-label")).style(type_style(14.0, 20.0));
            gap(Dp(32.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
                )
                .children(move || {
                    text((number).to_string())
                        .style(type_style(14.0, 20.0))
                        .color(scheme.inverse_on_surface);
                });
        });
}

#[tessera]
fn overview_article() {
    badge_variants();
    gap(Dp(16.0));
    prose(
        "ui-1-small-badge-on-a-navigation-item-2-large-badge-on-a-navigation-item-3-large-badge-with-m",
    );
    gap(Dp(32.0));
    subheading("ui-key-points".into());
    for point in [
        "ui-badges-can-contain-labels-or-numbers",
        "ui-there-are-two-variants-a-small-badge-and-a-large-badge",
        "ui-anchor-badges-inside-the-icon-bounding-box-at-the-upper-trailing-edge-of-the-icon",
        "ui-limit-content-to-four-characters-including-a-plus-sign",
        "ui-keep-the-default-color-mapping-so-badges-stand-out",
    ] {
        prose(format!("• {}", I18N.text(point)));
    }
}

#[tessera]
fn anatomy() {
    prose(
        "ui-a-small-badge-is-a-shape-with-no-text-a-large-badge-adds-a-label-so-it-can-carry-a-count-o",
    );
    gap(Dp(24.0));
    badge_figure(true, false);
    gap(Dp(16.0));
    prose("ui-1-small-badge");
    gap(Dp(24.0));
    badge_figure(false, false);
    gap(Dp(16.0));
    prose("ui-2-large-badge-container-3-large-badge-label");
}

#[tessera]
fn configuration_article() {
    anatomy();
    gap(Dp(32.0));
    subheading("ui-elements".into());
    spec_table(
        vec![
            "#".into(),
            "ui-element".into(),
            "ui-navigation-bar".into(),
            "ui-navigation-rail".into(),
        ],
        vec![
            vec![
                "1".into(),
                "ui-small-badge".into(),
                "ui-upper-trailing-icon-corner".into(),
                "ui-upper-trailing-icon-corner".into(),
            ],
            vec![
                "2".into(),
                "ui-large-badge-container".into(),
                "ui-upper-trailing-icon-corner".into(),
                "ui-upper-trailing-icon-corner".into(),
            ],
            vec![
                "3".into(),
                "ui-large-badge-label".into(),
                "ui-centered-in-the-container".into(),
                "ui-centered-in-the-container".into(),
            ],
            vec![
                "4".into(),
                "ui-large-badge-maximum-character-count-container".into(),
                "ui-four-characters-including-a".into(),
                "ui-four-characters-including-a".into(),
            ],
            vec![
                "5".into(),
                "ui-large-badge-maximum-character-count-label".into(),
                "999+".into(),
                "999+".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    configuration_variants();
    gap(Dp(32.0));
    configuration();
}

#[tessera]
fn configuration_variants() {
    subheading("ui-navigation-bar-and-rail-variants".into());
    gap(Dp(16.0));
    spec_table(
        vec!["#".into(), "ui-destination-state".into(), "ui-badge".into()],
        vec![
            vec![
                "1".into(),
                "ui-inactive-bar-with-label".into(),
                "ui-small-badge".into(),
            ],
            vec![
                "2".into(),
                "ui-inactive-bar-with-label".into(),
                "ui-large-number".into(),
            ],
            vec![
                "3".into(),
                "ui-inactive-bar-with-label".into(),
                "ui-maximum-count".into(),
            ],
            vec![
                "4".into(),
                "ui-active-bar-with-label".into(),
                "ui-small-badge".into(),
            ],
            vec![
                "5".into(),
                "ui-active-bar-with-label".into(),
                "ui-large-number".into(),
            ],
            vec![
                "6".into(),
                "ui-active-bar-with-label".into(),
                "ui-maximum-count".into(),
            ],
            vec![
                "7".into(),
                "ui-inactive-bar-no-label".into(),
                "ui-small-badge".into(),
            ],
            vec![
                "8".into(),
                "ui-inactive-bar-no-label".into(),
                "ui-large-number".into(),
            ],
            vec![
                "9".into(),
                "ui-inactive-bar-no-label".into(),
                "ui-maximum-count".into(),
            ],
            vec![
                "10".into(),
                "ui-active-bar-no-label".into(),
                "ui-small-badge".into(),
            ],
            vec![
                "11".into(),
                "ui-active-bar-no-label".into(),
                "ui-large-number".into(),
            ],
            vec![
                "12".into(),
                "ui-active-bar-no-label".into(),
                "ui-maximum-count".into(),
            ],
            vec![
                "13".into(),
                "ui-active-rail-with-label".into(),
                "ui-small-badge".into(),
            ],
            vec![
                "14".into(),
                "ui-active-rail-no-label".into(),
                "ui-large-number".into(),
            ],
            vec![
                "15".into(),
                "ui-active-rail-no-label".into(),
                "ui-maximum-count".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(specimens));
}

#[tessera]
fn badge_navigation() {
    let selected = remember(|| 1_usize);
    panel(RenderSlot::new(move || {
        navigation_examples("ui-guideline", selected, 3, false, false);
    }));
}

#[tessera]
fn badge_container() {
    prose(
        "ui-badge-containers-expand-horizontally-as-their-label-grows-retaining-the-same-anchor-and-he",
    );
    gap(Dp(24.0));
    badge_figure(true, true);
    gap(Dp(24.0));
    badge_figure(false, true);
    gap(Dp(32.0));
    subheading("ui-position-in-right-to-left-layouts".into());
    prose(
        "ui-do-mirror-the-badge-to-the-leading-edge-in-right-to-left-languages-don-t-move-the-badge-ar",
    );
    gap(Dp(24.0));
    subheading("ui-color".into());
    prose(
        "ui-do-use-the-default-badge-color-role-don-t-use-custom-container-or-label-colors-below-3-1-c",
    );
    gap(Dp(24.0));
    subheading("ui-label-text".into());
    prose(
        "ui-limit-large-badge-labels-to-four-characters-including-a-plus-sign-truncate-counts-before-t",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(specimens));
}

#[tessera]
fn readable_badge_navigation() {
    let selected = remember(|| 0_usize);
    let counts = remember(|| [0_u32, 3, 7]);
    navigation_bar()
        .icon_position(NavigationBarIconPosition::Top)
        .content(move |icon_position| {
            for (index, name) in ["ui-home", "ui-inbox", "ui-updates"]
                .into_iter()
                .enumerate()
            {
                navigation_bar_item()
                    .icon_position(icon_position)
                    .label(I18N.text(name))
                    .selected(selected.get() == index)
                    .on_click(move || {
                        selected.set(index);
                        counts.with_mut(|value| value[index] = 0);
                    })
                    .icon(move || {
                        badged_icon(index, counts.with(|value| value[index]));
                    });
            }
        });
    gap(Dp(16.0));
    text(
        I18N.format(
            "ui-value-selected-unread-badge-cleared",
            [(
                "v0",
                I18N.text(&(["ui-home", "ui-inbox", "ui-updates"][selected.get()]))
                    .into(),
            )],
        ),
    )
    .style(type_style(14.0, 20.0));
}

#[cfg(all(test, not(target_family = "wasm")))]
mod form_article_tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        accesskit::{Action, ActionRequest, NodeId, TreeId},
        headless::{HeadlessConfig, HeadlessRenderer},
    };

    use super::*;

    #[tessera]
    fn fixture(kind: usize) {
        material_theme().child(move || match kind {
            0 => {
                badges_spec();
            }
            1 => {
                super::super::snackbars::snackbars_spec();
            }
            _ => {
                super::super::text_fields::text_fields_spec();
            }
        });
    }
    fn activate(renderer: &mut HeadlessRenderer, label: &str, role: Role) -> NodeId {
        let id = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, label) && node.role() == role)
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
            .0;
        for action in [Action::Focus, Action::Click] {
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: id,
                data: None
            }));
            renderer.render_frames(4);
        }
        id
    }
    fn contains(renderer: &HeadlessRenderer, label: &str, role: Role) -> bool {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| crate::has_label(&node, label) && node.role() == role)
    }
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn final_form_articles_publish_official_tabs_and_execute_live_actions() {
        for (kind, guidelines, accessibility) in [
            (
                0,
                vec!["Usage", "Anatomy", "Container", "Placement"],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Visual indicators",
                    "Labeling elements",
                ],
            ),
            (
                1,
                vec![
                    "Usage",
                    "Anatomy",
                    "Placement",
                    "Responsive layout",
                    "Behavior",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Accessibility requirements on web",
                    "Focus",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                2,
                vec![
                    "Usage",
                    "Choosing text fields",
                    "Anatomy",
                    "Adaptive design",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
        ] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || {
                    fixture(kind);
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 900,
                    height: 8000,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            for tab in ["Overview", "Specs", "Guidelines", "Accessibility"] {
                assert!(contains(&renderer, tab, Role::Tab));
            }
            if kind == 0 {
                activate(&mut renderer, "Overview", Role::Tab);
                let mut labels = renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .filter_map(|(_, node)| {
                        (node.role() == Role::Label && crate::has_label(&node, "Label"))
                            .then(|| node.bounds().unwrap())
                    })
                    .collect::<Vec<_>>();
                labels.sort_by(|left, right| left.x0.total_cmp(&right.x0));
                assert_eq!(labels.len(), 3);
                assert_eq!(labels[0].y0, labels[1].y0);
                assert_eq!(labels[1].y0, labels[2].y0);
                assert_eq!(labels[1].x0 - labels[0].x0, 136.0);
                assert_eq!(labels[2].x0 - labels[1].x0, 136.0);
            }
            for (tab, expected) in [("Guidelines", guidelines), ("Accessibility", accessibility)] {
                activate(&mut renderer, tab, Role::Tab);
                let level = if kind == 1 && tab == "Accessibility" {
                    3
                } else {
                    2
                };
                if !(kind == 1 && tab == "Accessibility") {
                    for section in &expected {
                        activate(&mut renderer, tab, Role::Tab);
                        activate(&mut renderer, section, Role::Link);
                        renderer.render_frames(12);
                        let snapshot = renderer.accessibility_snapshot().unwrap();
                        let heading = snapshot
                            .nodes
                            .iter()
                            .find(|(_, node)| {
                                node.role() == Role::Heading
                                    && node.level() == Some(level)
                                    && crate::has_label(&node, *section)
                            })
                            .unwrap_or_else(|| {
                                panic!(
                                    "the real contents action must reveal article {kind} {tab} chapter {section}; published headings: {:?}",
                                    snapshot
                                        .nodes
                                        .iter()
                                        .filter(|(_, node)| node.role() == Role::Heading)
                                        .map(|(id, node)| (id, node.label(), node.level(), node.bounds()))
                                        .collect::<Vec<_>>()
                                )
                            });
                        let bounds = heading.1.bounds().unwrap();
                        assert!(
                            bounds.y0 >= 0.0
                                && bounds.y0 < f64::from(renderer.physical_size().height),
                            "article {kind} {tab} did not reveal {section}: {bounds:?}"
                        );
                    }
                }
                if kind == 1 && tab == "Accessibility" {
                    let snapshot = renderer.accessibility_snapshot().unwrap();
                    let heading = snapshot
                        .nodes
                        .iter()
                        .find(|(_, node)| {
                            node.role() == Role::Heading && crate::has_label(&node, "Use cases")
                        })
                        .unwrap()
                        .1
                        .bounds()
                        .unwrap();
                    assert_eq!(heading.height(), 32.0);
                    assert!(
                        !snapshot
                            .nodes
                            .iter()
                            .any(|(_, node)| crate::has_label(&node, "On this page"))
                    );
                    assert!(snapshot.nodes.iter().any(|(_, node)| {
                        node.role() == Role::Heading
                            && node.level() == Some(4)
                            && crate::has_label(&node, "1. Add inline feedback")
                    }));
                }
            }
            match kind {
                0 => {
                    activate(&mut renderer, "Inbox", Role::Tab);
                    assert!(contains(
                        &renderer,
                        "Inbox selected; unread badge cleared",
                        Role::Label
                    ));
                }
                1 => {
                    let trigger = activate(&mut renderer, "Save", Role::Button);
                    assert!(
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(id, node)| *id == trigger && crate::has_label(&node, "Saved"))
                    );
                    assert!(contains(&renderer, "All changes saved", Role::Status));
                    activate(&mut renderer, "Saved", Role::Button);
                    assert!(
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(id, node)| *id == trigger && crate::has_label(&node, "Save"))
                    );
                }
                _ => {
                    activate(&mut renderer, "Show password", Role::Button);
                    assert!(contains(&renderer, "Hide password", Role::Button));
                    activate(&mut renderer, "Hide password", Role::Button);
                    assert!(!contains(&renderer, "Hide password", Role::Button));
                }
            }
        }
    }
}
