//! Navigation bar and rail specification articles.
//!
//! ## Usage
//!
//! Compare primary destination layouts, selection and rail expansion.

#[path = "destination_articles.rs"]
pub(super) mod article_pages;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    badge::{badge, badge_with_content, badged_box},
    boxed::boxed,
    button::button,
    column::column,
    extended_floating_action_button::extended_floating_action_button,
    floating_action_button::FloatingActionButtonElevation,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    material_icons::{filled, outlined},
    modifier::{InteractionState, ModifierExt as _, Padding, SelectableArgs, SemanticsArgs},
    navigation_bar::{NavigationBarIconPosition, navigation_bar, navigation_bar_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    row::row,
    shape_def::Shape,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Color, Dp, FocusTraversalPolicy, Modifier, RenderSlot, RenderSlotWith, State,
    accesskit::Role, modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use self::article_pages::{Figure, Kind, detailed_article};
use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "navigation_figures_tests.rs"]
mod tests;

const NAMES: [&str; 5] = ["Home", "Inbox", "Saved", "Media", "Settings"];

#[tessera]
pub(super) fn navigation_spec(rail: bool) {
    let mut sections = vec![
        SpecSection {
            title: "Variants",
            content: RenderSlot::new(move || {
                variants(rail);
            }),
        },
        SpecSection {
            title: "Configurations",
            content: RenderSlot::new(move || {
                configurations(rail);
            }),
        },
        SpecSection {
            title: "Tokens & specs",
            content: RenderSlot::new(move || {
                tokens(rail);
            }),
        },
        SpecSection {
            title: "Anatomy",
            content: RenderSlot::new(move || {
                anatomy(rail);
            }),
        },
        SpecSection {
            title: "Color",
            content: RenderSlot::new(move || {
                colors(rail);
            }),
        },
        SpecSection {
            title: "States",
            content: RenderSlot::new(move || {
                states(rail);
            }),
        },
        SpecSection {
            title: "Measurements",
            content: RenderSlot::new(move || {
                measurements(rail);
            }),
        },
    ];
    if rail {
        sections.push(SpecSection {
            title: "Common layouts",
            content: RenderSlot::new(layouts),
        });
    }
    sections.push(SpecSection {
        title: if rail {
            "Baseline navigation rail"
        } else {
            "Baseline navigation bar"
        },
        content: RenderSlot::new(move || {
            baseline(rail);
        }),
    });
    detailed_article(
        if rail { Kind::Rail } else { Kind::Bar },
        if rail {
            "Navigation rail"
        } else {
            "Navigation bar"
        },
        if rail {
            "Navigation rails let people switch between UI views on mid-sized devices"
        } else {
            "Navigation bars let people switch between UI views on smaller devices"
        },
        sections,
        RenderSlotWith::new(move |figure| {
            chapter_figure(rail, figure);
        }),
    );
}

#[tessera]
fn sample(
    #[prop(into)] label: String,
    rail: bool,
    initially_expanded: bool,
    controls: Option<bool>,
    menu: Option<bool>,
    fab: Option<bool>,
    compact: Option<bool>,
) {
    let selected = remember(|| 0_usize);
    let expanded = remember(|| initially_expanded);
    let hidden = remember(|| false);
    let count = remember(|| 3_usize);
    let position = remember(|| NavigationBarIconPosition::Auto);
    let actions = remember(|| 0_u32);
    let caption = label.clone();
    let name = remember(|| label.clone());
    column()
        .modifier(
            (if compact.unwrap_or(false) {
                Modifier::new()
            } else {
                Modifier::new().fill_max_width()
            })
            .semantics(SemanticsArgs {
                role: Some(Role::Group),
                label: Some(format!("{label} navigation preview")),
                merge_descendants: true,
                ..Default::default()
            }),
        )
        .children(move || {
            if controls.unwrap_or(false) {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        if rail {
                            button()
                                .tonal()
                                .selected(expanded.get())
                                .accessibility_label("Expand navigation rail")
                                .on_click(move || expanded.set(!expanded.get()))
                                .child(|| {
                                    text().content("Expanded");
                                });
                            button()
                                .tonal()
                                .selected(hidden.get())
                                .accessibility_label("Hide collapsed rail")
                                .on_click(move || hidden.set(!hidden.get()))
                                .child(|| {
                                    text().content("Hide when collapsed");
                                });
                        } else {
                            button()
                                .tonal()
                                .accessibility_label("Cycle navigation item layout")
                                .on_click(move || {
                                    position.set(match position.get() {
                                        NavigationBarIconPosition::Auto => {
                                            NavigationBarIconPosition::Top
                                        }
                                        NavigationBarIconPosition::Top => {
                                            NavigationBarIconPosition::Start
                                        }
                                        NavigationBarIconPosition::Start => {
                                            NavigationBarIconPosition::Auto
                                        }
                                    })
                                })
                                .child(move || {
                                    text().content(format!("Layout: {:?}", position.get()));
                                });
                            button()
                                .text()
                                .accessibility_label("Cycle navigation destination count")
                                .on_click(move || {
                                    let next = if count.get() == 5 { 3 } else { count.get() + 1 };
                                    count.set(next);
                                    if selected.get() >= next {
                                        selected.set(next - 1);
                                    }
                                })
                                .child(move || {
                                    text().content(format!("Destinations: {}", count.get()));
                                });
                        }
                    });
                gap(Dp(24.0));
            }
            if rail {
                if !hidden.get() || expanded.get() {
                    boxed()
                        .modifier(Modifier::new().height(Dp(480.0)))
                        .children(move || {
                            navigation_rail()
                                .expanded(expanded.get())
                                .header_optional(if menu.unwrap_or(false) || fab.unwrap_or(false) {
                                    Some(RenderSlot::new(move || {
                                        rail_header(
                                            name.get(),
                                            expanded,
                                            actions,
                                            menu.unwrap_or(false),
                                            fab.unwrap_or(false),
                                        );
                                    }))
                                } else {
                                    None
                                })
                                .content(move || {
                                    for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                        navigation_rail_item()
                                            .label(name)
                                            .selected(selected.get() == index)
                                            .on_click(move || selected.set(index))
                                            .icon(move || {
                                                destination_icon(index, selected.get() == index);
                                            });
                                    }
                                });
                        });
                }
            } else {
                navigation_bar()
                    .icon_position(position.get())
                    .content(move || {
                        for (index, name) in NAMES.into_iter().take(count.get()).enumerate() {
                            navigation_bar_item()
                                .label(name)
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .icon(move || {
                                    destination_icon(index, selected.get() == index);
                                });
                        }
                    });
            }
            gap(Dp(16.0));
            text()
                .content(format!("{caption} destination: {}", NAMES[selected.get()]))
                .style(type_style(14.0, 20.0));
            if fab.unwrap_or(false) {
                text()
                    .content(format!("{caption} compositions: {}", actions.get()))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn rail_header(
    #[prop(into)] label: String,
    expanded: State<bool>,
    actions: State<u32>,
    menu: bool,
    fab: bool,
) {
    column().children(move || {
        if menu {
            icon_button()
                .standard()
                .icon(if expanded.get() {
                    filled::MENU_OPEN_SVG
                } else {
                    filled::MENU_SVG
                })
                .accessibility_label(format!("{label} rail menu"))
                .on_click(move || expanded.set(!expanded.get()));
        }
        if fab {
            extended_floating_action_button("Compose")
                .icon(filled::EDIT_SVG)
                .expanded(expanded.get())
                .elevation(FloatingActionButtonElevation::new(
                    Dp::ZERO,
                    Dp::ZERO,
                    Dp::ZERO,
                    Dp::ZERO,
                ))
                .accessibility_label(format!("{label} compose"))
                .on_click(move || actions.set(actions.get() + 1));
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

#[tessera]
fn variants(rail: bool) {
    panel(RenderSlot::new(move || {
        if rail {
            flow_row()
                .item_spacing(Dp(32.0))
                .line_spacing(Dp(32.0))
                .children(|| {
                    sample("Collapsed", true, false).compact(true);
                    sample("Expanded", true, true).compact(true);
                });
        } else {
            sample("Flexible", false, false);
        }
    }));
}

#[tessera]
fn configurations(rail: bool) {
    panel(RenderSlot::new(move || {
        sample("Configured", rail, false)
            .controls(true)
            .menu(rail)
            .fab(rail);
    }));
}

#[tessera]
fn tokens(rail: bool) {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec![
                "Container".into(),
                if rail {
                    "Surface (uncontained)".into()
                } else {
                    "Surface container".into()
                },
            ],
            vec!["Selected indicator".into(), "Secondary container".into()],
            vec!["Selected icon".into(), "On secondary container".into()],
            vec![
                "Selected label 路 vertical / horizontal".into(),
                "Secondary / on secondary container".into(),
            ],
            vec![
                "Inactive icon and label".into(),
                "On surface variant".into(),
            ],
            vec![
                "Hover / focus / press state layer".into(),
                "8% / 10% / 10%".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy(rail: bool) {
    panel(RenderSlot::new(move || {
        sample("Anatomy", rail, false).menu(rail).fab(rail);
    }));
    gap(Dp(24.0));
    text().content(if rail {"1  Container    2  Optional menu    3  Optional FAB    4  Icon    5  Active indicator    6  Label"}else{"1  Container    2  Icon    3  Label    4  Active indicator"}).style(type_style(16.0,24.0));
}

#[tessera]
fn colors(rail: bool) {
    for dark in [false, true] {
        color_figure(rail, dark);
    }
    gap(Dp(24.0));
    color_legend(rail, false);
}

#[tessera]
fn color_figure(rail: bool, dark: bool) {
    let mut theme = MaterialTheme::from_seed(ACCENT, dark);
    let backdrop = theme.color_scheme.surface;
    if rail {
        theme.color_scheme.surface = theme.color_scheme.surface_container;
    }
    material_theme()
        .theme(move || theme.clone())
        .child(move || {
            if rail {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        for expanded in [false, true] {
                            boxed()
                                .modifier(Modifier::new().weight(1.0))
                                .children(move || {
                                    color_specimen(true, expanded, dark, backdrop);
                                });
                        }
                    });
            } else {
                color_specimen(false, false, dark, backdrop);
            }
        });
}

#[tessera]
fn color_specimen(rail: bool, initially_expanded: bool, dark: bool, backdrop: Color) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0_usize);
    let expanded = remember(|| initially_expanded);
    let actions = remember(|| 0_usize);
    let context = if rail {
        if expanded.get() {
            "expanded rail"
        } else {
            "collapsed rail"
        }
    } else {
        "bar"
    };
    let mode = if dark { "Dark" } else { "Light" };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(if rail { 601.5 } else { 412.0 }))
                        .background(backdrop)
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            merge_descendants: true,
                            label: Some(format!("{mode} {context} color roles")),
                            ..Default::default()
                        }),
                )
                .children(move || {
                    navigation_grid(geometry, dark);
                    boxed()
                        .modifier(if rail {
                            watch(geometry, "container").height(Dp(512.0))
                        } else {
                            watch(geometry, "container")
                                .width(Dp(412.0))
                                .offset(Dp::ZERO, Dp(20.0))
                        })
                        .children(move || {
                            if rail {
                                navigation_rail()
                                    .expanded(expanded.get())
                                    .header(move || {
                                        column().children(move || {
                                            icon_button()
                                                .standard()
                                                .icon(if expanded.get() {
                                                    filled::MENU_OPEN_SVG
                                                } else {
                                                    filled::MENU_SVG
                                                })
                                                .accessibility_label(format!(
                                                    "{mode} {context} menu"
                                                ))
                                                .on_click(move || expanded.set(!expanded.get()));
                                            gap(Dp(8.0));
                                            extended_floating_action_button("Label")
                                                .expanded(expanded.get())
                                                .icon(filled::EDIT_SVG)
                                                .elevation(FloatingActionButtonElevation::new(
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                ))
                                                .accessibility_label(format!(
                                                    "{mode} {context} compose"
                                                ))
                                                .on_click(move || actions.set(actions.get() + 1));
                                        });
                                    })
                                    .content(move || {
                                        for index in 0..4 {
                                            let mut item = navigation_rail_item()
                                                .label("Label")
                                                .selected(selected.get() == index)
                                                .accessibility_description(format!(
                                                    "{mode} {context} destination {}",
                                                    index + 1
                                                ))
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    boxed()
                                                        .modifier(watch(geometry, ICON_KEYS[index]))
                                                        .children(move || {
                                                            figure_icon(selected.get() == index);
                                                        });
                                                });
                                            if index >= 2 {
                                                item = item.badge(move || {
                                                    navigation_badge(
                                                        if index == 2 { 1 } else { 2 },
                                                        geometry,
                                                    );
                                                });
                                            }
                                            drop(item);
                                        }
                                    });
                            } else {
                                navigation_bar()
                                    .icon_position(NavigationBarIconPosition::Top)
                                    .content(move || {
                                        for index in 0..3 {
                                            navigation_bar_item()
                                                .label("Label")
                                                .selected(selected.get() == index)
                                                .accessibility_description(format!(
                                                    "{mode} bar destination {}",
                                                    index + 1
                                                ))
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    boxed()
                                                        .modifier(watch(geometry, ICON_KEYS[index]))
                                                        .children(move || {
                                                            figure_icon(selected.get() == index);
                                                        });
                                                });
                                        }
                                    });
                            }
                        });
                    boxed()
                        .alignment(Alignment::TopStart)
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            navigation_color_annotations(
                                rail,
                                expanded.get(),
                                false,
                                selected.get(),
                                dark,
                                geometry,
                            );
                        });
                });
            if actions.get() > 0 {
                text()
                    .content(format!("{mode} {context} compositions: {}", actions.get()))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn navigation_grid(geometry: State<Geometry>, dark: bool) {
    navigation_grid_content(geometry.with(annotations::relative_geometry), dark);
}

#[tessera]
fn navigation_grid_content(values: Geometry, dark: bool) {
    let Some(root) = values.get("figure") else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let tone = if dark { "#79747e" } else { "#aea9b4" };
    let mut svg = String::new();
    let mut y = 1.0;
    while y < height.0 {
        let mut x = 1.0;
        while x < width.0 {
            svg.push_str(&format!("<circle cx='{x}' cy='{y}' r='.4' fill='{tone}'/>"));
            x += 8.0;
        }
        y += 8.0;
    }
    annotations::lines(width, height, svg);
}

fn navigation_color_roles(rail: bool, baseline: bool) -> Vec<&'static str> {
    if rail && baseline {
        vec![
            "On secondary container",
            "Secondary container",
            "On surface",
            "On surface variant",
            "On surface variant",
            "Error",
            "On error",
            "Error",
        ]
    } else if rail {
        vec![
            "Surface container (optional)",
            "On secondary container",
            "Secondary container",
            "Secondary (vertical) / On secondary container (horizontal)",
            "On surface variant",
            "On surface variant",
            "Error",
            "On error",
            "Error",
        ]
    } else {
        vec![
            if baseline {
                "Surface"
            } else {
                "Surface container"
            },
            "On secondary container",
            if baseline { "On surface" } else { "Secondary" },
            "Secondary container",
            "On surface variant",
            "On surface variant",
        ]
    }
}

#[tessera]
fn color_legend(rail: bool, baseline: bool) {
    spec_table(
        vec!["Element".into(), "Color role".into()],
        navigation_color_roles(rail, baseline)
            .into_iter()
            .enumerate()
            .map(|(index, role)| vec![(index + 1).to_string(), role.into()])
            .collect(),
    );
}

#[tessera]
fn navigation_color_annotations(
    rail: bool,
    expanded: bool,
    baseline: bool,
    selected: usize,
    dark: bool,
    geometry: State<Geometry>,
) {
    navigation_color_annotation_content(
        rail,
        expanded,
        baseline,
        selected,
        dark,
        geometry.with(annotations::relative_geometry),
    );
}

#[tessera]
fn navigation_color_annotation_content(
    rail: bool,
    expanded: bool,
    baseline: bool,
    selected: usize,
    dark: bool,
    values: Geometry,
) {
    let (Some(root), Some(container), Some(active), Some(inactive)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, ICON_KEYS[selected]),
        relative(
            &values,
            ICON_KEYS[(selected + 1) % if rail { 4 } else { 3 }],
        ),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let active_label = if rail && expanded {
        (active.x + active.w + 24.0, active.y + active.h * 0.5)
    } else {
        (active.x + active.w * 0.5, active.y + active.h + 12.0)
    };
    let inactive_label = if rail && expanded {
        (
            inactive.x + inactive.w + 24.0,
            inactive.y + inactive.h * 0.5,
        )
    } else {
        (
            inactive.x + inactive.w * 0.5,
            inactive.y + inactive.h + 12.0,
        )
    };
    let indicator = if rail && expanded {
        (active.x + active.w + 28.0, active.y - 8.0)
    } else {
        (
            active.x + active.w + if baseline && !rail { 20.0 } else { 16.0 },
            active.y + active.h * 0.5,
        )
    };
    let mut anchors = if rail {
        let mut points = if baseline {
            vec![
                (active.x + 12.0, active.y + 12.0),
                indicator,
                active_label,
                (inactive.x + 12.0, inactive.y + 12.0),
                inactive_label,
            ]
        } else {
            vec![
                (container.x + container.w - 12.0, container.y + 44.0),
                (active.x + 12.0, active.y + 12.0),
                indicator,
                active_label,
                (inactive.x + 12.0, inactive.y + 12.0),
                inactive_label,
            ]
        };
        for key in ["large-badge", "badge-label", "small-badge"] {
            let Some(rect) = relative(&values, key) else {
                return;
            };
            points.push((rect.x + rect.w * 0.5, rect.y + rect.h * 0.5));
        }
        points
    } else {
        vec![
            (container.x + 4.0, container.y + container.h * 0.5),
            (active.x + 12.0, active.y + 12.0),
            active_label,
            indicator,
            (inactive.x + 12.0, inactive.y + 12.0),
            inactive_label,
        ]
    };
    let roles = navigation_color_roles(rail, baseline);
    let mut svg = String::new();
    for (index, (x, y)) in anchors.drain(..).enumerate() {
        let (mx, my) = if rail {
            let left = if baseline {
                index % 2 == 0
            } else {
                matches!(index, 1 | 3 | 4 | 6)
            };
            (
                if left {
                    container.x - 48.0
                } else {
                    container.x + container.w + 24.0
                },
                y - 16.0,
            )
        } else if index == 0 {
            (container.x - 48.0, y - 16.0)
        } else if matches!(index, 2 | 5) {
            (x - 12.0, container.y + container.h + 24.0)
        } else {
            (x - 12.0, container.y - 56.0)
        };
        if rail || index == 0 {
            dotted(
                &mut svg,
                (mx + 12.0).min(x),
                y,
                (x - mx - 12.0).abs(),
                false,
                if dark { "#cac4d0" } else { "#49454f" },
            );
        } else {
            dotted(
                &mut svg,
                x,
                (my + 16.0).min(y),
                (y - my - 16.0).abs(),
                true,
                if dark { "#cac4d0" } else { "#49454f" },
            );
        }
        navigation_color_marker(index + 1, roles[index].into(), Dp(mx), Dp(my));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn navigation_color_marker(number: usize, role: String, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.on_surface, Shape::rounded_rectangle(Dp(16.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Navigation color role {number}: {role}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn baseline(rail: bool) {
    text().content(if rail { "The baseline rail is retained for comparison. Use a collapsed or expanded navigation rail for new designs." } else { "The baseline bar is retained for comparison. Use the flexible navigation bar for new designs." }).style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    baseline_figure(rail, false);
    gap(Dp(32.0));
    subheading("Tokens & specs".into());
    baseline_measurements(rail);
    gap(Dp(32.0));
    subheading("Color".into());
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                baseline_figure(rail, false)
                    .color_annotations(true)
                    .dark(dark);
            });
    }
    gap(Dp(24.0));
    color_legend(rail, true);
    gap(Dp(32.0));
    subheading("States".into());
    text().content("Select a destination or move keyboard focus to inspect the live selection and input feedback. Focus does not activate a destination.").style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    baseline_figure(rail, false);
    gap(Dp(32.0));
    subheading("Measurements".into());
    baseline_figure(rail, true);
    baseline_measurements(rail);
    gap(Dp(32.0));
    subheading("Configurations".into());
    if rail {
        text().content("Keep destinations in a vertical group. A menu or primary action may precede the destinations.").style(type_style(16.0, 24.0));
        gap(Dp(24.0));
        baseline_figure(true, false);
    } else {
        for count in [3, 4, 5] {
            subheading(format!("{count} destinations"));
            gap(Dp(16.0));
            baseline_figure(false, false).destination_count(count);
            gap(Dp(24.0));
        }
    }
}

#[tessera]
fn baseline_measurements(rail: bool) {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec![
                if rail {
                    "Container width"
                } else {
                    "Container height"
                }
                .into(),
                "80dp".into(),
            ],
            vec![
                "Item height".into(),
                if rail { "56dp" } else { "80dp" }.into(),
            ],
            vec![
                "Active indicator".into(),
                if rail { "56 × 32dp" } else { "64 × 32dp" }.into(),
            ],
            vec!["Icon".into(), "24dp".into()],
            vec!["Icon-to-label gap".into(), "4dp".into()],
            vec![
                if rail {
                    "Item gap"
                } else {
                    "Top / bottom padding"
                }
                .into(),
                if rail { "4dp" } else { "12 / 16dp" }.into(),
            ],
        ],
    );
}

#[tessera]
fn baseline_figure(
    rail: bool,
    measurements: bool,
    color_annotations: Option<bool>,
    dark: Option<bool>,
    destination_count: Option<usize>,
) {
    let selected = remember(|| 0_usize);
    let geometry = remember(Geometry::new);
    let count = destination_count.unwrap_or(4).clamp(3, 5);
    let color_annotations = color_annotations.unwrap_or(false);
    let dark = dark.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if rail { 480.0 } else { 323.0 }))
                .background(scheme.surface_container_low),
        )
        .children(move || {
            navigation_grid(geometry, dark);
            boxed()
                .modifier(
                    watch(geometry, "container")
                        .size(
                            Dp(if rail { 80.0 } else { 412.0 }),
                            Dp(if rail { 480.0 } else { 80.0 }),
                        )
                        .background(scheme.surface)
                        .semantics(SemanticsArgs {
                            role: Some(Role::TabList),
                            merge_descendants: true,
                            label: Some(
                                if rail {
                                    "Baseline navigation rail"
                                } else {
                                    "Baseline navigation bar"
                                }
                                .into(),
                            ),
                            ..Default::default()
                        })
                        .focus_group()
                        .focus_traversal_policy(if rail {
                            FocusTraversalPolicy::vertical().wrap(true)
                        } else {
                            FocusTraversalPolicy::horizontal().wrap(true)
                        }),
                )
                .children(move || {
                    if rail {
                        column()
                            .modifier(Modifier::new().fill_max_size())
                            .children(move || {
                                boxed()
                                    .alignment(Alignment::TopCenter)
                                    .modifier(Modifier::new().size(Dp(80.0), Dp(120.0)))
                                    .children(|| {
                                        boxed()
                                            .modifier(Modifier::new().offset(Dp::ZERO, Dp(56.0)))
                                            .children(|| {
                                                icon().vector(filled::MENU_SVG).size(Dp(24.0));
                                            });
                                    });
                                for index in 0..count {
                                    if index > 0 {
                                        gap(Dp(4.0));
                                    }
                                    baseline_destination(true, index, count, selected, geometry);
                                }
                            });
                    } else {
                        row()
                            .modifier(Modifier::new().fill_max_size())
                            .children(move || {
                                for index in 0..count {
                                    baseline_destination(false, index, count, selected, geometry);
                                }
                            });
                    }
                });
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    if color_annotations {
                        navigation_color_annotations(
                            rail,
                            false,
                            true,
                            selected.get(),
                            dark,
                            geometry,
                        );
                    } else {
                        figure_annotations(rail, false, measurements, geometry);
                    }
                });
        });
}

#[tessera]
fn baseline_destination(
    rail: bool,
    index: usize,
    count: usize,
    selected: State<usize>,
    geometry: State<Geometry>,
) {
    let interaction = remember(InteractionState::new);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let active = selected.get() == index;
    let layer = interaction.with(|value| {
        if value.is_pressed() || value.is_focused() {
            0.10
        } else if value.is_hovered() {
            0.08
        } else {
            0.0
        }
    });
    let mut label_style = theme.typography.label_medium;
    label_style.font_weight = if active { 600 } else { 500 };
    boxed()
        .modifier(
            Modifier::new()
                .size(
                    Dp(if rail { 80.0 } else { 412.0 / count as f64 }),
                    Dp(if rail { 56.0 } else { 80.0 }),
                )
                .selectable_with(SelectableArgs {
                    selected: active,
                    on_click: Callback::new(move || selected.set(index)),
                    role: Some(Role::Tab),
                    label: Some("Label".into()),
                    description: Some(format!(
                        "Baseline navigation {} destination {}",
                        if rail { "rail" } else { "bar" },
                        index + 1
                    )),
                    interaction_state: Some(interaction),
                    ..Default::default()
                }),
        )
        .children(move || {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                    Dp::ZERO,
                    if rail { Dp::ZERO } else { Dp(12.0) },
                    Dp::ZERO,
                    if rail { Dp(4.0) } else { Dp(16.0) },
                )))
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .size(Dp(if rail { 56.0 } else { 64.0 }), Dp(32.0))
                                .background_with_shape(
                                    if active {
                                        scheme.secondary_container
                                    } else {
                                        Color::TRANSPARENT
                                    },
                                    Shape::rounded_rectangle(Dp(16.0)),
                                ),
                        )
                        .children(move || {
                            boxed().modifier(
                                Modifier::new().fill_max_size().background_with_shape(
                                    scheme.on_surface.with_alpha(layer),
                                    Shape::rounded_rectangle(Dp(16.0)),
                                ),
                            );
                            boxed()
                                .modifier(watch(geometry, ICON_KEYS[index]))
                                .children(move || {
                                    badged_box()
                                        .content(move || {
                                            icon()
                                                .vector(
                                                    [
                                                        filled::CIRCLE_SVG,
                                                        filled::CHANGE_HISTORY_SVG,
                                                        filled::CROP_SQUARE_SVG,
                                                        filled::PENTAGON_SVG,
                                                        filled::STARS_SVG,
                                                    ][index],
                                                )
                                                .size(Dp(24.0))
                                                .tint(if active {
                                                    scheme.on_secondary_container
                                                } else {
                                                    scheme.on_surface_variant
                                                });
                                        })
                                        .badge(move || {
                                            if index == 2 {
                                                navigation_badge(1, geometry);
                                            } else if index == 3 {
                                                navigation_badge(2, geometry);
                                            }
                                        });
                                });
                        });
                    gap(Dp(4.0));
                    text().content("Label").style(label_style).color(if active {
                        scheme.on_surface
                    } else {
                        scheme.on_surface_variant
                    });
                });
        });
}

#[tessera]
fn states(rail: bool) {
    text().content("Select a destination, move focus with Tab and arrow keys, and activate it with Enter or Space. Hover, focus and press feedback preserve the current destination until activation.").style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        sample("State", rail, false);
    }));
}

#[tessera]
fn measurements(rail: bool) {
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        if rail {
            vec![
                vec![
                    "Collapsed / minimum expanded width".into(),
                    "96 / 220dp".into(),
                ],
                vec!["Collapsed item minimum height".into(), "64dp".into()],
                vec!["Expanded item height".into(), "56dp".into()],
                vec!["Collapsed active indicator".into(), "56 脳 32dp".into()],
                vec!["Icon".into(), "24dp".into()],
            ]
        } else {
            vec![
                vec!["Container height".into(), "64dp".into()],
                vec!["Vertical item active indicator".into(), "56 脳 32dp".into()],
                vec![
                    "Horizontal item active indicator height".into(),
                    "40dp".into(),
                ],
                vec!["Vertical icon-to-label gap".into(), "4dp".into()],
                vec!["Icon".into(), "24dp".into()],
            ]
        },
    );
}

#[tessera]
fn layouts() {
    for (label, menu, fab) in [
        ("Destinations", false, false),
        ("Menu", true, false),
        ("Compose", false, true),
        ("Menu and compose", true, true),
    ] {
        subheading(label.into());
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            sample(label, true, false).menu(menu).fab(fab);
        }));
        gap(Dp(32.0));
    }
}

const ICON_KEYS: [&str; 5] = ["icon0", "icon1", "icon2", "icon3", "icon4"];

#[tessera]
fn chapter_figure(rail: bool, figure: Figure) {
    match figure {
        Figure::None => {}
        Figure::Anatomy => {
            native_figure(rail, false, 1, true);
            gap(Dp(32.0));
            native_figure(rail, true, 1, true);
        }
        Figure::Badges => {
            native_figure(rail, false, 1, true);
            if rail {
                gap(Dp(32.0));
                native_figure(true, true, 1, true);
            }
        }
        Figure::Adaptive => {
            if rail {
                modal_rail();
                gap(Dp(32.0));
            }
            native_figure(rail, false, 0, false);
            gap(Dp(32.0));
            native_figure(rail, true, 0, false);
        }
        Figure::Placement => {
            placement(rail);
        }
        Figure::Scaling => {
            let theme = use_context::<MaterialTheme>().unwrap().get();
            for scale in [1.5, 2.0] {
                subheading(format!("{}× text", scale));
                gap(Dp(16.0));
                let mut scaled = theme.clone();
                scaled.typography.label_medium.font_size =
                    scaled.typography.label_medium.font_size * scale;
                scaled.typography.label_medium.line_height = scaled
                    .typography
                    .label_medium
                    .line_height
                    .map(|height| height * scale);
                scaled.typography.label_large.font_size =
                    scaled.typography.label_large.font_size * scale;
                scaled.typography.label_large.line_height = scaled
                    .typography
                    .label_large
                    .line_height
                    .map(|height| height * scale);
                material_theme()
                    .theme(move || scaled.clone())
                    .child(move || {
                        native_figure(rail, false, 2, false);
                    });
                gap(Dp(24.0));
            }
        }
        Figure::Labels => {
            native_figure(rail, false, 0, false);
            gap(Dp(24.0));
            spec_table(
                vec!["Visible label".into(), "Additional description".into()],
                vec![
                    vec!["Inbox".into(), "3 unread messages".into()],
                    vec!["Saved".into(), "Saved places".into()],
                ],
            );
        }
        Figure::Keyboard | Figure::Selection | Figure::States => {
            native_figure(rail, false, 0, true);
        }
        Figure::Overview | Figure::Fixed | Figure::Scrollable | Figure::Nested => {
            native_figure(rail, false, 0, false);
        }
    }
}

#[tessera]
fn native_figure(rail: bool, expanded: bool, annotation: usize, badges: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| 0usize);
    let expanded = remember(|| expanded);
    let actions = remember(|| 0u32);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch(geometry, "figure")
                        .fill_max_width()
                        .height(if rail { Dp(512.0) } else { Dp(323.0) })
                        .background_with_shape(
                            scheme.surface_container,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            let modifier = watch(geometry, "container");
                            boxed()
                                .modifier(if rail {
                                    modifier.height(Dp(512.0))
                                } else {
                                    modifier.width(if expanded.get() {
                                        Dp(741.0)
                                    } else {
                                        Dp(412.0)
                                    })
                                })
                                .children(move || {
                                    if rail {
                                        navigation_rail()
                                            .expanded(expanded.get())
                                            .header(move || {
                                                column().children(move || {
                                                    boxed()
                                                        .modifier(watch(geometry, "menu"))
                                                        .children(move || {
                                                            icon_button()
                                                                .standard()
                                                                .icon(if expanded.get() {
                                                                    filled::MENU_OPEN_SVG
                                                                } else {
                                                                    filled::MENU_SVG
                                                                })
                                                                .accessibility_label(
                                                                    "Figure rail menu",
                                                                )
                                                                .on_click(move || {
                                                                    expanded.set(!expanded.get())
                                                                });
                                                        });
                                                    gap(Dp(8.0));
                                                    boxed()
                                                        .modifier(watch(geometry, "fab"))
                                                        .children(move || {
                                                            extended_floating_action_button(
                                                                "Label",
                                                            )
                                                            .expanded(expanded.get())
                                                            .icon(filled::EDIT_SVG)
                                                            .elevation(
                                                                FloatingActionButtonElevation::new(
                                                                    Dp::ZERO,
                                                                    Dp::ZERO,
                                                                    Dp::ZERO,
                                                                    Dp::ZERO,
                                                                ),
                                                            )
                                                            .accessibility_label(
                                                                "Figure rail compose",
                                                            )
                                                            .on_click(move || {
                                                                actions.set(actions.get() + 1)
                                                            });
                                                        });
                                                });
                                            })
                                            .content(move || {
                                                for index in 0..3 {
                                                    let mut item = navigation_rail_item()
                                                        .label("Label")
                                                        .selected(selected.get() == index)
                                                        .on_click(move || selected.set(index))
                                                        .icon(move || {
                                                            boxed()
                                                                .modifier(watch(
                                                                    geometry,
                                                                    ICON_KEYS[index],
                                                                ))
                                                                .children(move || {
                                                                    figure_icon(
                                                                        selected.get() == index,
                                                                    );
                                                                });
                                                        });
                                                    if badges && index > 0 {
                                                        item = item
                                                            .accessibility_description(
                                                                if index == 1 {
                                                                    "3 unread messages"
                                                                } else {
                                                                    "New items"
                                                                },
                                                            )
                                                            .badge(move || {
                                                                navigation_badge(index, geometry);
                                                            });
                                                    }
                                                    drop(item);
                                                }
                                            });
                                    } else {
                                        navigation_bar()
                                            .icon_position(if expanded.get() {
                                                NavigationBarIconPosition::Start
                                            } else {
                                                NavigationBarIconPosition::Top
                                            })
                                            .content(move || {
                                                for index in 0..4 {
                                                    navigation_bar_item()
                                                        .label("Label")
                                                        .selected(selected.get() == index)
                                                        .accessibility_description_optional(
                                                            if badges && index > 1 {
                                                                Some(
                                                                    if index == 3 {
                                                                        "3 unread messages"
                                                                    } else {
                                                                        "New items"
                                                                    }
                                                                    .into(),
                                                                )
                                                            } else {
                                                                None
                                                            },
                                                        )
                                                        .on_click(move || selected.set(index))
                                                        .icon(move || {
                                                            boxed()
                                                                .modifier(watch(
                                                                    geometry,
                                                                    ICON_KEYS[index],
                                                                ))
                                                                .children(move || {
                                                                    if badges && index > 1 {
                                                                        badged_box()
                                                                            .content(move || {
                                                                                figure_icon(
                                                                                    selected.get()
                                                                                        == index,
                                                                                );
                                                                            })
                                                                            .badge(move || {
                                                                                navigation_badge(
                                                                                    if index == 3 {
                                                                                        1
                                                                                    } else {
                                                                                        2
                                                                                    },
                                                                                    geometry,
                                                                                );
                                                                            });
                                                                    } else {
                                                                        figure_icon(
                                                                            selected.get() == index,
                                                                        );
                                                                    }
                                                                });
                                                        });
                                                }
                                            });
                                    }
                                });
                        });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            if annotation > 0 {
                                figure_annotations(rail, expanded.get(), annotation == 2, geometry);
                            }
                        });
                });
            gap(Dp(16.0));
            text()
                .content(format!("Figure destination: {}", selected.get() + 1))
                .style(type_style(14.0, 20.0));
            if actions.get() > 0 {
                text()
                    .content(format!("Figure compositions: {}", actions.get()))
                    .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn figure_icon(selected: bool) {
    icon()
        .painter(if selected {
            filled::STARS_SVG
        } else {
            outlined::STARS_SVG
        })
        .size(Dp(24.0));
}

#[tessera]
fn navigation_badge(index: usize, geometry: State<Geometry>) {
    boxed()
        .modifier(watch(
            geometry,
            if index == 1 {
                "large-badge"
            } else {
                "small-badge"
            },
        ))
        .children(move || {
            if index == 1 {
                badge_with_content().content(move || {
                    text().content("3").modifier(watch(geometry, "badge-label"));
                });
            } else {
                badge();
            }
        });
}

#[tessera]
fn figure_annotations(rail: bool, expanded: bool, measurements: bool, geometry: State<Geometry>) {
    figure_annotation_content(
        rail,
        expanded,
        measurements,
        geometry.with(annotations::relative_geometry),
    );
}

#[tessera]
fn figure_annotation_content(rail: bool, expanded: bool, measurements: bool, values: Geometry) {
    let (Some(root), Some(container), Some(icon)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "icon0"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if measurements {
        annotations::outline(&mut svg, container);
        dotted(
            &mut svg,
            container.x,
            container.y - 16.0,
            container.w,
            false,
            "#ec407a",
        );
        annotations::dimension(
            format!("{}", container.w.round()),
            Dp(container.x + container.w * 0.5 - 12.0),
            Dp(container.y - 40.0),
        );
        annotations::dimension(
            format!("{}", container.h.round()),
            Dp(container.x + container.w + 16.0),
            Dp(container.y + container.h * 0.5 - 10.0),
        );
        annotations::outline(&mut svg, icon);
        annotations::dimension("24".into(), Dp(icon.x), Dp(icon.y - 26.0));
    } else {
        let positions: Vec<(usize, Rect, bool)> = if rail {
            let mut points = vec![(1, container, false), (4, icon, true)];
            if let Some(menu) = relative(&values, "menu") {
                points.push((2, menu, true));
            }
            if let Some(fab) = relative(&values, "fab") {
                points.push((3, fab, true));
            }
            if let Some(badge) = relative(&values, "large-badge") {
                points.push((7, badge, true));
            }
            if let Some(label) = relative(&values, "badge-label") {
                points.push((8, label, false));
            }
            if let Some(badge) = relative(&values, "small-badge") {
                points.push((9, badge, true));
            }
            points
        } else {
            let mut points = vec![(1, container, false), (2, icon, true)];
            if let Some(badge) = relative(&values, "small-badge") {
                points.push((5, badge, true));
            }
            if let Some(badge) = relative(&values, "large-badge") {
                points.push((6, badge, true));
            }
            if let Some(label) = relative(&values, "badge-label") {
                points.push((7, label, false));
            }
            points
        };
        for (number, rect, trailing) in positions {
            let y = rect.y + rect.h * 0.5;
            let x = if trailing {
                rect.x + rect.w + 24.0
            } else {
                rect.x - 40.0
            };
            if trailing {
                dotted(&mut svg, rect.x + rect.w, y, 24.0, false, "#49454f");
            } else {
                dotted(&mut svg, x + 24.0, y, rect.x - x - 24.0, false, "#49454f");
            }
            annotations::marker(number, Dp(x), Dp(y - 16.0));
        }
        let indicator_x = if rail && expanded {
            container.x + 20.0
        } else {
            icon.x - 16.0
        };
        let indicator_y = if rail && expanded {
            icon.y - 16.0
        } else {
            icon.y - 4.0
        };
        let indicator_number = if rail { 5 } else { 4 };
        annotations::marker(
            indicator_number,
            Dp(indicator_x - 60.0),
            Dp(indicator_y - 16.0),
        );
        dotted(
            &mut svg,
            indicator_x - 36.0,
            indicator_y,
            36.0,
            false,
            "#49454f",
        );
        let label_number = if rail { 6 } else { 3 };
        let label_y = if rail && expanded {
            icon.y + 12.0
        } else {
            icon.y + icon.h + 20.0
        };
        annotations::marker(label_number, Dp(icon.x - 12.0), Dp(label_y + 24.0));
        dotted(&mut svg, icon.x, label_y, 24.0, true, "#49454f");
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn placement(rail: bool) {
    let selected = remember(|| 0usize);
    let count = remember(|| 0u32);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(480.0))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
        )
        .children(move || {
            if rail {
                row()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        navigation_rail().content(move || {
                            for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                navigation_rail_item()
                                    .label(name)
                                    .selected(selected.get() == index)
                                    .on_click(move || selected.set(index))
                                    .icon(move || {
                                        destination_icon(index, selected.get() == index);
                                    });
                            }
                        });
                        column()
                            .modifier(Modifier::new().weight(1.0).padding_all(Dp(24.0)))
                            .children(move || {
                                text()
                                    .content(NAMES[selected.get()])
                                    .style(type_style(24.0, 32.0));
                                gap(Dp(16.0));
                                text().content("Page content").style(type_style(16.0, 24.0));
                            });
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        column()
                            .modifier(
                                Modifier::new()
                                    .weight(1.0)
                                    .fill_max_width()
                                    .padding_all(Dp(24.0)),
                            )
                            .children(move || {
                                text()
                                    .content(NAMES[selected.get()])
                                    .style(type_style(24.0, 32.0));
                                spacer().modifier(Modifier::new().weight(1.0));
                                boxed()
                                    .alignment(Alignment::CenterEnd)
                                    .modifier(Modifier::new().fill_max_width())
                                    .children(move || {
                                        extended_floating_action_button("Compose")
                                            .icon(filled::EDIT_SVG)
                                            .on_click(move || count.set(count.get() + 1));
                                    });
                            });
                        navigation_bar().content(move || {
                            for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                navigation_bar_item()
                                    .label(name)
                                    .selected(selected.get() == index)
                                    .on_click(move || selected.set(index))
                                    .icon(move || {
                                        destination_icon(index, selected.get() == index);
                                    });
                            }
                        });
                    });
            }
        });
}

#[tessera]
fn modal_rail() {
    let open = remember(|| false);
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    modal_side_sheet_provider()
        .position(SideSheetPosition::Start)
        .is_open(open.get())
        .accessibility_label("Expanded navigation rail")
        .on_close_request(move || open.set(false))
        .main_content(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(480.0))
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                        .padding_all(Dp(24.0)),
                )
                .children(move || {
                    icon_button()
                        .standard()
                        .icon(filled::MENU_SVG)
                        .accessibility_label("Open modal expanded rail")
                        .on_click(move || open.set(true));
                    gap(Dp(24.0));
                    text()
                        .content(NAMES[selected.get()])
                        .style(type_style(24.0, 32.0));
                    text()
                        .content("Select the menu to reveal navigation over this page.")
                        .style(type_style(16.0, 24.0));
                });
        })
        .side_sheet_content(move || {
            navigation_rail()
                .expanded(true)
                .header(move || {
                    icon_button()
                        .standard()
                        .icon(filled::MENU_OPEN_SVG)
                        .accessibility_label("Close modal expanded rail")
                        .on_click(move || open.set(false));
                })
                .content(move || {
                    for (index, name) in NAMES.into_iter().enumerate() {
                        navigation_rail_item()
                            .label(name)
                            .selected(selected.get() == index)
                            .on_click(move || {
                                selected.set(index);
                                open.set(false);
                            })
                            .icon(move || {
                                destination_icon(index, selected.get() == index);
                            });
                    }
                });
        });
}
