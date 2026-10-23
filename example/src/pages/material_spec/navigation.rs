//! Navigation bar and rail specification articles.
//!
//! ## Usage
//!
//! Compare primary destination layouts, selection and rail expansion.

#[path = "destination_articles.rs"]
pub(super) mod article_pages;

use tessera_components::{
    alignment::Alignment,
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
    modifier::{ModifierExt as _, SemanticsArgs},
    navigation_bar::{NavigationBarIconPosition, navigation_bar, navigation_bar_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    row::row,
    shape_def::Shape,
    side_sheet::{SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, RenderSlotWith, State, accesskit::Role, remember, tessera,
    use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

use article_pages::{Figure, Kind, detailed_article};

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
                                sample(if dark { "Dark" } else { "Light" }, rail, false);
                            });
                    });
            });
        gap(Dp(32.0));
    }
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
    let values = geometry.get();
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
