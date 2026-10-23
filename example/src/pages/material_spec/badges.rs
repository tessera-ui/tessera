//! Badge specifications in real navigation destinations.
//!
//! ## Usage
//!
//! Show new activity and unread counts without adding separate actions.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::{badge, badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_bar::{
        NavigationBarIconPosition, NavigationBarLabelBehavior, navigation_bar, navigation_bar_item,
    },
    navigation_rail::{navigation_rail, navigation_rail_item},
    shape_def::Shape,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn badges_spec() {
    article(
        "Badges",
        "Highlight new activity or unread counts on navigation destinations and icons.",
        vec![
            SpecSection {
                title: "Tokens & specs",
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
            SpecSection {
                title: "Configuration",
                content: RenderSlot::new(configuration),
            },
        ],
    )
    .pages(vec![
        ArticlePage {title:"Overview",intro:Some(RenderSlot::new(badge_variants)),sections:vec![SpecSection{title:"Differences from M2",content:RenderSlot::new(|| {prose("Badges use updated color roles and support dynamic color. Keep the default error and on-error mappings.");gap(Dp(24.0));badge_figure(false,false);})}]},
        ArticlePage {title:"Guidelines",intro:Some(RenderSlot::new(badge_navigation)),sections:vec![
            SpecSection{title:"Usage",content:RenderSlot::new(|| {badge_variants();gap(Dp(24.0));subheading("With other components".into());prose("Anchor badges to navigation destinations, app bars or tabs. Hide unread badges after their destination is selected.");gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"Anatomy",content:RenderSlot::new(|| {badge_figure(true,false);gap(Dp(24.0));badge_figure(false,false);prose("Small badge: container. Large badge: 1 Container, 2 Label text.");})},
            SpecSection{title:"Container",content:RenderSlot::new(badge_container)},
            SpecSection{title:"Placement",content:RenderSlot::new(|| {prose("Anchor at the upper trailing edge inside the icon bounds. Use a dot when space is tight, and a count when it will not collide with adjacent content.");gap(Dp(24.0));let selected=remember(|| 0_usize);navigation_examples("Rail placement",selected,3,true,false);})},
        ]},
        ArticlePage{title:"Accessibility",intro:None,sections:vec![
            SpecSection{title:"Use cases",content:RenderSlot::new(|| {prose("People must be able to understand unread counts and clear the notification by visiting the associated destination.");gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"Interaction & style",content:RenderSlot::new(|| {prose("The navigation item owns the action. The badge itself adds information without creating another focus target. Selecting a destination clears its unread indication.");gap(Dp(24.0));readable_badge_navigation();})},
            SpecSection{title:"Visual indicators",content:RenderSlot::new(|| {prose("Keep the default color mapping so badges stand out. Custom badge colors need at least 3:1 contrast.");gap(Dp(24.0));badge_figure(false,false);})},
            SpecSection{title:"Labeling elements",content:RenderSlot::new(|| {prose("Announce the destination followed by its count, or New notification for a dot. Do not announce the decorative badge twice.");gap(Dp(24.0));readable_badge_navigation();})},
        ]},
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            let selected = remember(|| 0_usize);
            navigation_examples("Preview", selected, 7, false, false);
        }));
    });
}

#[tessera]
fn badged_icon(kind: usize, count: u32) {
    badged_box()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Image),
            label: Some(if count == 0 {
                "No unread items".into()
            } else if kind == 0 {
                "New activity".into()
            } else {
                format!("{count} unread items")
            }),
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
                        text().content(if count > 999 {
                            "999+".into()
                        } else {
                            count.to_string()
                        });
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
            label: Some(format!("{label} badge navigation")),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            if rail {
                boxed()
                    .modifier(Modifier::new().height(Dp(320.0)))
                    .children(move || {
                        navigation_rail().content(move || {
                            for (index, name) in
                                ["Home", "Inbox", "Updates"].into_iter().enumerate()
                            {
                                navigation_rail_item()
                                    .label(name)
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
                    .content(move || {
                        for (index, name) in ["Home", "Inbox", "Updates"].into_iter().enumerate() {
                            navigation_bar_item()
                                .label(name)
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
            text()
                .content(format!(
                    "{caption} destination: {}",
                    ["Home", "Inbox", "Updates"][selected.get()]
                ))
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn specimens() {
    flow_row()
        .item_spacing(Dp(40.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            badge().modifier(Modifier::new().semantics(SemanticsArgs {
                role: Some(Role::Image),
                label: Some("Small badge specimen".into()),
                ..Default::default()
            }));
            badge_with_content()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some("Large badge specimen".into()),
                    ..Default::default()
                }))
                .content(|| {
                    text().content("7");
                });
            badge_with_content()
                .modifier(Modifier::new().semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some("Maximum badge specimen".into()),
                    ..Default::default()
                }))
                .content(|| {
                    text().content("999+");
                });
        });
}

#[tessera]
fn tokens() {
    panel(RenderSlot::new(specimens));
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Error".into()],
            vec!["Label".into(), "On error · label small".into()],
            vec!["Small badge".into(), "6dp circle".into()],
            vec!["Large badge".into(), "16dp high · 8dp corners".into()],
        ],
    );
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        subheading(if dark {
            "Dark scheme".into()
        } else {
            "Light scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                let selected = remember(|| 1_usize);
                                navigation_examples(
                                    if dark { "Dark" } else { "Light" },
                                    selected,
                                    7,
                                    false,
                                    false,
                                );
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn measurements() {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Small badge size / radius".into(), "6dp / 3dp".into()],
            vec!["Large badge height / radius".into(), "16dp / 8dp".into()],
            vec!["Single digit width".into(), "16dp".into()],
            vec!["Label horizontal padding".into(), "4dp".into()],
            vec![
                "Small icon-corner offset · vertical / horizontal".into(),
                "6 / 6dp".into(),
            ],
            vec![
                "Large icon-corner offset · vertical / horizontal".into(),
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
                .accessibility_label("Add unread item")
                .on_click(move || count.set(count.get() + 1))
                .child(|| {
                    text().content("Add unread");
                });
            button()
                .text()
                .accessibility_label("Clear unread items")
                .on_click(move || count.set(0))
                .child(|| {
                    text().content("Clear");
                });
            button()
                .tonal()
                .selected(rail.get())
                .accessibility_label("Use badge navigation rail")
                .on_click(move || rail.set(!rail.get()))
                .child(|| {
                    text().content("Rail");
                });
            button()
                .tonal()
                .selected(labels.get())
                .enabled(!rail.get())
                .accessibility_label("Only selected bar label")
                .on_click(move || labels.set(!labels.get()))
                .child(|| {
                    text().content("Selected label only");
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        navigation_examples(
            "Configured",
            selected,
            count.get(),
            rail.get(),
            labels.get(),
        );
    }));
    gap(Dp(24.0));
    text()
        .content(format!("Unread count: {}", count.get()))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
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
                            .content(|| {
                                text().content("3");
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
        annotations::dotted(&mut svg, badge.x + badge.w, badge.y, 48.0, false, "#49454f");
        if !small {
            annotations::marker(2, Dp(badge.x - 72.0), Dp(badge.y + 24.0));
            annotations::dotted(
                &mut svg,
                badge.x - 48.0,
                badge.y + badge.h,
                48.0,
                false,
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
        "Badges draw attention to new activity. A small dot conveys presence; a large badge shows a short count.",
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
                            text().content(if number == 2 { "1" } else { "999+" });
                        });
                    }
                });
            gap(Dp(12.0));
            text().content("Label").style(type_style(14.0, 20.0));
            gap(Dp(32.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
                )
                .children(move || {
                    text()
                        .content(number.to_string())
                        .style(type_style(14.0, 20.0))
                        .color(scheme.inverse_on_surface);
                });
        });
}

#[tessera]
fn badge_navigation() {
    let selected = remember(|| 1_usize);
    panel(RenderSlot::new(move || {
        navigation_examples("Guideline", selected, 3, false, false);
    }));
}

#[tessera]
fn badge_container() {
    prose(
        "Badge containers expand horizontally as their label grows, retaining the same anchor and height. Do not move the badge over the icon.",
    );
    gap(Dp(24.0));
    badge_figure(true, true);
    gap(Dp(24.0));
    badge_figure(false, true);
    gap(Dp(24.0));
    subheading("Label text".into());
    prose(
        "Limit large badge labels to four characters, including a plus sign. Truncate counts before they collide with nearby content.",
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
        .content(move || {
            for (index, name) in ["Home", "Inbox", "Updates"].into_iter().enumerate() {
                navigation_bar_item()
                    .label(name)
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
    text()
        .content(format!(
            "{} selected; unread badge cleared",
            ["Home", "Inbox", "Updates"][selected.get()]
        ))
        .style(type_style(14.0, 20.0));
}

#[cfg(all(test, not(target_family = "wasm")))]
mod form_article_tests {
    use super::*;
    use std::{cell::Cell, time::Duration};
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::{Action, ActionRequest, NodeId, TreeId},
    };
    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }
    thread_local! {static KIND:Cell<usize>=const {Cell::new(0)};}
    #[tessera]
    fn fixture() {
        material_theme().child(|| match KIND.get() {
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
            .find(|(_, node)| node.label() == Some(label) && node.role() == role)
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
            .0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }));
        renderer.render_frames(4);
        id
    }
    fn contains(renderer: &HeadlessRenderer, label: &str, role: Role) -> bool {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some(label) && node.role() == role)
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
            KIND.set(kind);
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(Pipelines)],
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
                        (node.role() == Role::Label && node.label() == Some("Label"))
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
            for (tab, mut expected) in
                [("Guidelines", guidelines), ("Accessibility", accessibility)]
            {
                activate(&mut renderer, tab, Role::Tab);
                let mut actual = renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .into_iter()
                    .filter_map(|(_, node)| {
                        (node.role() == Role::Heading
                            && node.level()
                                == Some(if kind == 1 && tab == "Accessibility" {
                                    3
                                } else {
                                    2
                                }))
                        .then(|| node.label().unwrap().to_owned())
                    })
                    .collect::<Vec<_>>();
                actual.sort();
                expected.sort();
                assert_eq!(actual, expected, "article {kind} {tab}");
                if kind == 1 && tab == "Accessibility" {
                    let snapshot = renderer.accessibility_snapshot().unwrap();
                    let heading = snapshot
                        .nodes
                        .iter()
                        .find(|(_, node)| {
                            node.role() == Role::Heading && node.label() == Some("Use cases")
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
                            .any(|(_, node)| node.label() == Some("On this page"))
                    );
                    assert!(snapshot.nodes.iter().any(|(_, node)| {
                        node.role() == Role::Heading
                            && node.level() == Some(4)
                            && node.label() == Some("1. Add inline feedback")
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
                            .any(|(id, node)| *id == trigger && node.label() == Some("Saved"))
                    );
                    activate(&mut renderer, "Undo", Role::Button);
                    assert!(
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .any(|(id, node)| *id == trigger && node.label() == Some("Save"))
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
