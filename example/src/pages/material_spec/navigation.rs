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
    modifier::ModifierExt as _,
    navigation_bar::{NavigationBarIconPosition, navigation_bar, navigation_bar_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    row::row,
    shape_def::Shape,
    side_sheet::{SideSheetController, SideSheetPosition, modal_side_sheet_provider},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_foundation::modifier::{
    InteractionState, ModifierExt as _, Padding, SelectableArgs, SemanticsArgs,
};
use tessera_ui::{
    Callback, Color, Dp, FocusTraversalPolicy, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    gap, panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "navigation_figures_tests.rs"]
mod tests;

pub(super) const NAMES: [&str; 5] = ["ui-home", "ui-inbox", "ui-saved", "ui-media", "ui-settings"];

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
                label: Some(I18N.format(
                    "ui-value-navigation-preview",
                    [("v0", I18N.text(&label).into())],
                )),
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
                                .accessibility_label(I18N.text("ui-expand-navigation-rail"))
                                .on_click(move || expanded.set(!expanded.get()))
                                .child(move || {
                                    text(I18N.text("ui-expanded"));
                                });
                            button()
                                .tonal()
                                .selected(hidden.get())
                                .accessibility_label(I18N.text("ui-hide-collapsed-rail"))
                                .on_click(move || hidden.set(!hidden.get()))
                                .child(move || {
                                    text(I18N.text("ui-hide-when-collapsed"));
                                });
                        } else {
                            button()
                                .tonal()
                                .accessibility_label(I18N.text("ui-cycle-navigation-item-layout"))
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
                                    text(I18N.format(
                                        "ui-layout-value",
                                        [("v0", format!("{:?}", position.get()).into())],
                                    ));
                                });
                            button()
                                .text()
                                .accessibility_label(
                                    I18N.text("ui-cycle-navigation-destination-count"),
                                )
                                .on_click(move || {
                                    let next = if count.get() == 5 { 3 } else { count.get() + 1 };
                                    count.set(next);
                                    if selected.get() >= next {
                                        selected.set(next - 1);
                                    }
                                })
                                .child(move || {
                                    text(I18N.format(
                                        "ui-destinations-value",
                                        [("v0", (count.get()).into())],
                                    ));
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
                                .content(move |rail_scope| {
                                    for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                        navigation_rail_item(rail_scope)
                                            .label(I18N.text(name))
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
                    .content(move |icon_position| {
                        for (index, name) in NAMES.into_iter().take(count.get()).enumerate() {
                            navigation_bar_item()
                                .icon_position(icon_position)
                                .label(I18N.text(name))
                                .selected(selected.get() == index)
                                .on_click(move || selected.set(index))
                                .icon(move || {
                                    destination_icon(index, selected.get() == index);
                                });
                        }
                    });
            }
            gap(Dp(16.0));
            text(I18N.format(
                "ui-value-destination-value",
                [
                    ("v0", I18N.text(&caption).into()),
                    ("v1", I18N.text(&(NAMES[selected.get()])).into()),
                ],
            ))
            .style(type_style(14.0, 20.0));
            if fab.unwrap_or(false) {
                text(I18N.format(
                    "ui-value-compositions-value",
                    [
                        ("v0", I18N.text(&caption).into()),
                        ("v1", (actions.get()).into()),
                    ],
                ))
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
                .accessibility_label(
                    I18N.format("ui-value-rail-menu", [("v0", I18N.text(&label).into())]),
                )
                .on_click(move || expanded.set(!expanded.get()));
        }
        if fab {
            extended_floating_action_button(I18N.text("ui-compose"))
                .icon(filled::EDIT_SVG)
                .expanded(expanded.get())
                .elevation(FloatingActionButtonElevation::new(
                    Dp::ZERO,
                    Dp::ZERO,
                    Dp::ZERO,
                    Dp::ZERO,
                ))
                .accessibility_label(
                    I18N.format("ui-value-compose", [("v0", I18N.text(&label).into())]),
                )
                .on_click(move || actions.set(actions.get() + 1));
        }
    });
}

#[tessera]
pub(super) fn destination_icon(index: usize, selected: bool) {
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
                    sample("ui-collapsed", true, false).compact(true);
                    sample("ui-expanded", true, true).compact(true);
                });
        } else {
            sample("ui-flexible", false, false);
        }
    }));
}

#[tessera]
fn configurations(rail: bool) {
    panel(RenderSlot::new(move || {
        sample("ui-configured", rail, false)
            .controls(true)
            .menu(rail)
            .fab(rail);
    }));
}

#[tessera]
fn tokens(rail: bool) {
    spec_table(
        vec!["ui-element".into(), "ui-token-value".into()],
        vec![
            vec![
                "ui-container".into(),
                if rail {
                    "ui-surface-uncontained".into()
                } else {
                    "Surface container".into()
                },
            ],
            vec!["ui-selected-indicator".into(), "Secondary container".into()],
            vec!["ui-selected-icon".into(), "On secondary container".into()],
            vec![
                "ui-selected-label-vertical-horizontal".into(),
                "Secondary / on secondary container".into(),
            ],
            vec![
                "ui-inactive-icon-and-label".into(),
                "On surface variant".into(),
            ],
            vec![
                "ui-hover-focus-press-state-layer".into(),
                "8% / 10% / 10%".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy(rail: bool) {
    native_figure(rail, false, 1, false);
    gap(Dp(24.0));
    text(I18N.text(if rail {
        "ui-1-container-2-optional-menu-3-optional-fab-4-icon-5-active-indicator-6-label"
    } else {
        "ui-1-container-2-icon-3-label-4-active-indicator"
    }))
    .style(type_style(16.0, 24.0));
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
            "ui-expanded-rail"
        } else {
            "ui-collapsed-rail"
        }
    } else {
        "bar"
    };
    let mode = if dark { "ui-dark" } else { "ui-light" };
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
                            label: Some(I18N.format(
                                "ui-value-value-color-roles",
                                [
                                    ("v0", I18N.text(&mode).into()),
                                    ("v1", I18N.text(&context).into()),
                                ],
                            )),
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
                                                .accessibility_label(I18N.format(
                                                    "ui-value-value-menu",
                                                    [
                                                        ("v0", I18N.text(&mode).into()),
                                                        ("v1", I18N.text(&context).into()),
                                                    ],
                                                ))
                                                .on_click(move || expanded.set(!expanded.get()));
                                            gap(Dp(8.0));
                                            extended_floating_action_button(I18N.text("ui-label"))
                                                .expanded(expanded.get())
                                                .icon(filled::EDIT_SVG)
                                                .elevation(FloatingActionButtonElevation::new(
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                    Dp::ZERO,
                                                ))
                                                .accessibility_label(I18N.format(
                                                    "ui-value-value-compose",
                                                    [
                                                        ("v0", I18N.text(&mode).into()),
                                                        ("v1", I18N.text(&context).into()),
                                                    ],
                                                ))
                                                .on_click(move || actions.set(actions.get() + 1));
                                        });
                                    })
                                    .content(move |rail_scope| {
                                        for index in 0..4 {
                                            let item = navigation_rail_item(rail_scope)
                                                .label(I18N.text("ui-label"))
                                                .selected(selected.get() == index)
                                                .accessibility_description(I18N.format(
                                                    "ui-value-value-destination-value",
                                                    [
                                                        ("v0", I18N.text(&mode).into()),
                                                        ("v1", I18N.text(&context).into()),
                                                        ("v2", (index + 1).into()),
                                                    ],
                                                ))
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    boxed()
                                                        .modifier(watch(geometry, NAMES[index]))
                                                        .children(move || {
                                                            figure_icon(selected.get() == index);
                                                        });
                                                });
                                            if index >= 2 {
                                                item.badge(move || {
                                                    navigation_badge(
                                                        if index == 2 { 1 } else { 2 },
                                                        geometry,
                                                    );
                                                });
                                            }
                                        }
                                    });
                            } else {
                                navigation_bar()
                                    .icon_position(NavigationBarIconPosition::Top)
                                    .content(move |icon_position| {
                                        for index in 0..3 {
                                            navigation_bar_item()
                                                .icon_position(icon_position)
                                                .label(I18N.text("ui-label"))
                                                .selected(selected.get() == index)
                                                .accessibility_description(I18N.format(
                                                    "ui-value-bar-destination-value",
                                                    [
                                                        ("v0", I18N.text(&mode).into()),
                                                        ("v1", (index + 1).into()),
                                                    ],
                                                ))
                                                .on_click(move || selected.set(index))
                                                .icon(move || {
                                                    boxed()
                                                        .modifier(watch(geometry, NAMES[index]))
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
                text(I18N.format(
                    "ui-value-value-compositions-value",
                    [
                        ("v0", I18N.text(&mode).into()),
                        ("v1", I18N.text(&context).into()),
                        ("v2", (actions.get()).into()),
                    ],
                ))
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

pub(super) fn navigation_color_roles(rail: bool, baseline: bool) -> Vec<&'static str> {
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
            "ui-surface-container-optional",
            "On secondary container",
            "Secondary container",
            "ui-secondary-vertical-on-secondary-container-horizontal",
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
        vec!["ui-element".into(), "ui-color-role".into()],
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
        relative(&values, NAMES[selected]),
        relative(&values, NAMES[(selected + 1) % if rail { 4 } else { 3 }]),
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
                    label: Some(I18N.format(
                        "ui-navigation-color-role-value-value",
                        [("v0", (number).into()), ("v1", I18N.text(&role).into())],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn baseline(rail: bool) {
    text(I18N.text(if rail { "ui-the-baseline-rail-is-retained-for-comparison-use-a-collapsed-or-expanded-navigation-rail-f" } else { "ui-the-baseline-bar-is-retained-for-comparison-use-the-flexible-navigation-bar-for-new-design" })).style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    baseline_figure(rail, false);
    gap(Dp(32.0));
    subheading("ui-tokens-specs".into());
    baseline_measurements(rail);
    gap(Dp(32.0));
    subheading("ui-color".into());
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
    subheading("ui-states".into());
    text(I18N.text("ui-select-a-destination-or-move-keyboard-focus-to-inspect-the-live-selection-and-input-feedba")).style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    baseline_figure(rail, false);
    gap(Dp(32.0));
    subheading("ui-measurements".into());
    baseline_figure(rail, true);
    baseline_measurements(rail);
    gap(Dp(32.0));
    subheading("ui-configurations".into());
    if rail {
        text(I18N.text("ui-keep-destinations-in-a-vertical-group-a-menu-or-primary-action-may-precede-the-destination")).style(type_style(16.0, 24.0));
        gap(Dp(24.0));
        baseline_figure(true, false);
    } else {
        for count in [3, 4, 5] {
            subheading(I18N.format("ui-value-destinations", [("v0", (count).into())]));
            gap(Dp(16.0));
            baseline_figure(false, false).destination_count(count);
            gap(Dp(24.0));
        }
    }
}

#[tessera]
fn baseline_measurements(rail: bool) {
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec![
                if rail {
                    "ui-container-width"
                } else {
                    "ui-container-height"
                }
                .into(),
                "80dp".into(),
            ],
            vec![
                "ui-item-height".into(),
                if rail { "56dp" } else { "80dp" }.into(),
            ],
            vec![
                "ui-active-indicator".into(),
                if rail { "56 × 32dp" } else { "64 × 32dp" }.into(),
            ],
            vec!["ui-icon".into(), "24dp".into()],
            vec!["ui-icon-to-label-gap-3995".into(), "4dp".into()],
            vec![
                if rail {
                    "ui-item-gap"
                } else {
                    "ui-top-bottom-padding-6966"
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
                            label: Some(I18N.text(if rail {
                                "ui-baseline-navigation-rail"
                            } else {
                                "ui-baseline-navigation-bar"
                            })),
                            ..Default::default()
                        })
                        .focus_group(None)
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
                                                icon().painter(filled::MENU_SVG).size(Dp(24.0));
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
                .selectable(SelectableArgs {
                    selected: active,
                    on_click: Callback::new(move || selected.set(index)),
                    role: Some(Role::Tab),
                    label: Some(I18N.text("ui-label")),
                    description: Some(
                        I18N.format(
                            "ui-baseline-navigation-value-destination-value",
                            [
                                (
                                    "v0",
                                    I18N.text(&(if rail { "ui-rail-7098" } else { "ui-bar" }))
                                        .into(),
                                ),
                                ("v1", (index + 1).into()),
                            ],
                        ),
                    ),
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
                                .modifier(watch(geometry, NAMES[index]))
                                .children(move || {
                                    badged_box()
                                        .content(move || {
                                            icon()
                                                .painter(
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
                    text(I18N.text("ui-label"))
                        .style(label_style)
                        .color(if active {
                            scheme.on_surface
                        } else {
                            scheme.on_surface_variant
                        });
                });
        });
}

#[tessera]
fn states(rail: bool) {
    text(I18N.text("ui-select-a-destination-move-focus-with-tab-and-arrow-keys-and-activate-it-with-enter-or-spac")).style(type_style(16.0,24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        sample("ui-state", rail, false);
    }));
}

#[tessera]
fn measurements(rail: bool) {
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        if rail {
            vec![
                vec![
                    "ui-collapsed-minimum-expanded-width".into(),
                    "96 / 220dp".into(),
                ],
                vec!["ui-collapsed-item-minimum-height".into(), "64dp".into()],
                vec!["ui-expanded-item-height".into(), "56dp".into()],
                vec!["ui-collapsed-active-indicator".into(), "56 × 32dp".into()],
                vec!["ui-icon".into(), "24dp".into()],
            ]
        } else {
            vec![
                vec!["ui-container-height".into(), "64dp".into()],
                vec![
                    "ui-vertical-item-active-indicator".into(),
                    "56 × 32dp".into(),
                ],
                vec![
                    "ui-horizontal-item-active-indicator-height".into(),
                    "40dp".into(),
                ],
                vec!["ui-vertical-icon-to-label-gap".into(), "4dp".into()],
                vec!["ui-icon".into(), "24dp".into()],
            ]
        },
    );
}

#[tessera]
fn layouts() {
    for (label, menu, fab) in [
        ("ui-destinations", false, false),
        ("ui-menu", true, false),
        ("ui-compose", false, true),
        ("ui-menu-and-compose", true, true),
    ] {
        subheading(label.into());
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            sample(label, true, false).menu(menu).fab(fab);
        }));
        gap(Dp(32.0));
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
                                                                    I18N.text(
                                                                        "ui-figure-rail-menu",
                                                                    ),
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
                                                                I18N.text("ui-label"),
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
                                                                I18N.text("ui-figure-rail-compose"),
                                                            )
                                                            .on_click(move || {
                                                                actions.set(actions.get() + 1)
                                                            });
                                                        });
                                                });
                                            })
                                            .content(move |rail_scope| {
                                                for index in 0..3 {
                                                    let item = navigation_rail_item(rail_scope)
                                                        .label(I18N.text("ui-label"))
                                                        .selected(selected.get() == index)
                                                        .on_click(move || selected.set(index))
                                                        .icon(move || {
                                                            boxed()
                                                                .modifier(watch(
                                                                    geometry,
                                                                    NAMES[index],
                                                                ))
                                                                .children(move || {
                                                                    figure_icon(
                                                                        selected.get() == index,
                                                                    );
                                                                });
                                                        });
                                                    if badges && index > 0 {
                                                        item.accessibility_description(I18N.text(
                                                            if index == 1 {
                                                                "ui-3-unread-messages"
                                                            } else {
                                                                "ui-new-items"
                                                            },
                                                        ))
                                                        .badge(move || {
                                                            navigation_badge(index, geometry);
                                                        });
                                                    }
                                                }
                                            });
                                    } else {
                                        navigation_bar()
                                            .icon_position(if expanded.get() {
                                                NavigationBarIconPosition::Start
                                            } else {
                                                NavigationBarIconPosition::Top
                                            })
                                            .content(move |icon_position| {
                                                for index in 0..4 {
                                                    navigation_bar_item()
                                                        .icon_position(icon_position)
                                                        .label(I18N.text("ui-label"))
                                                        .selected(selected.get() == index)
                                                        .accessibility_description_optional(
                                                            if badges && index > 1 {
                                                                Some(
                                                                    if index == 3 {
                                                                        "ui-3-unread-messages"
                                                                    } else {
                                                                        "ui-new-items"
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
                                                                    NAMES[index],
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
            text(I18N.format(
                "ui-figure-destination-value",
                [("v0", (selected.get() + 1).into())],
            ))
            .style(type_style(14.0, 20.0));
            if actions.get() > 0 {
                text(I18N.format(
                    "ui-figure-compositions-value",
                    [("v0", (actions.get()).into())],
                ))
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
                    text("3").modifier(watch(geometry, "badge-label"));
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
        let label = Rect {
            x: if expanded {
                icon.x + icon.w + 8.0
            } else {
                icon.x - 8.0
            },
            y: if expanded {
                icon.y
            } else {
                icon.y + icon.h + 4.0
            },
            w: 40.0,
            h: 20.0,
        };
        let indicator = Rect {
            x: icon.x - 16.0,
            y: icon.y - if rail && expanded { 16.0 } else { 4.0 },
            w: 56.0,
            h: if rail && expanded { 56.0 } else { 32.0 },
        };
        let mut points = if rail {
            vec![
                (1, container, false),
                (4, icon, true),
                (5, indicator, false),
                (6, label, true),
            ]
        } else {
            vec![
                (1, container, false),
                (2, icon, false),
                (3, label, true),
                (4, indicator, false),
            ]
        };
        if rail {
            if let Some(menu) = relative(&values, "menu") {
                points.push((2, menu, false));
            }
            if let Some(fab) = relative(&values, "fab") {
                points.push((3, fab, false));
            }
        }
        for (name, number) in [
            ("large-badge", if rail { 7 } else { 6 }),
            ("badge-label", if rail { 8 } else { 7 }),
            ("small-badge", if rail { 9 } else { 5 }),
        ] {
            if let Some(rect) = relative(&values, name) {
                points.push((number, rect, true));
            }
        }
        for trailing in [false, true] {
            let mut lane: Vec<_> = points.iter().filter(|point| point.2 == trailing).collect();
            lane.sort_by(|a, b| {
                let a = if rail {
                    a.1.y + a.1.h * 0.5
                } else {
                    a.1.x + a.1.w * 0.5
                };
                let b = if rail {
                    b.1.y + b.1.h * 0.5
                } else {
                    b.1.x + b.1.w * 0.5
                };
                a.total_cmp(&b)
            });
            let mut previous = -44.0;
            let mut centers = Vec::with_capacity(lane.len());
            for (_, rect, _) in &lane {
                let target = if rail {
                    rect.y + rect.h * 0.5
                } else {
                    rect.x + rect.w * 0.5
                };
                let center = target.max(previous + 44.0).max(20.0);
                centers.push(center);
                previous = center;
            }
            let limit = if rail { height.0 } else { width.0 } - 20.0;
            for i in (0..centers.len()).rev() {
                let next = centers.get(i + 1).map_or(limit, |next| next - 44.0);
                centers[i] = centers[i].min(next);
            }
            for ((number, rect, _), center) in lane.into_iter().zip(centers) {
                let tx = rect.x + rect.w * 0.5;
                let ty = rect.y + rect.h * 0.5;
                let (x, y) = if rail {
                    (
                        if trailing {
                            container.x + container.w + 52.0
                        } else {
                            container.x - 52.0
                        },
                        center,
                    )
                } else {
                    (
                        center,
                        if trailing {
                            container.y + container.h + 56.0
                        } else {
                            container.y - 56.0
                        },
                    )
                };
                if rail {
                    let edge = x + if trailing { -12.0 } else { 12.0 };
                    let elbow = if trailing {
                        container.x + container.w + 20.0
                    } else {
                        container.x - 20.0
                    };
                    dotted(&mut svg, edge, y, elbow - edge, false, "#49454f");
                    dotted(&mut svg, elbow, y, ty - y, true, "#49454f");
                    dotted(&mut svg, elbow, ty, tx - elbow, false, "#49454f");
                } else {
                    let edge = y + if trailing { -16.0 } else { 16.0 };
                    let elbow = if trailing {
                        container.y + container.h + 20.0
                    } else {
                        container.y - 20.0
                    };
                    dotted(&mut svg, x, edge, elbow - edge, true, "#49454f");
                    dotted(&mut svg, x, elbow, tx - x, false, "#49454f");
                    dotted(&mut svg, tx, elbow, ty - elbow, true, "#49454f");
                }
                annotations::marker(*number, Dp(x - 12.0), Dp(y - 16.0));
            }
        }
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
                        navigation_rail().content(move |rail_scope| {
                            for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                navigation_rail_item(rail_scope)
                                    .label(I18N.text(name))
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
                                text(I18N.text(NAMES[selected.get()]))
                                    .style(type_style(24.0, 32.0));
                                gap(Dp(16.0));
                                text(I18N.text("ui-page-content")).style(type_style(16.0, 24.0));
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
                                text(I18N.text(NAMES[selected.get()]))
                                    .style(type_style(24.0, 32.0));
                                spacer().modifier(Modifier::new().weight(1.0));
                                boxed()
                                    .alignment(Alignment::CenterEnd)
                                    .modifier(Modifier::new().fill_max_width())
                                    .children(move || {
                                        extended_floating_action_button(I18N.text("ui-compose"))
                                            .icon(filled::EDIT_SVG)
                                            .on_click(move || count.set(count.get() + 1));
                                    });
                            });
                        navigation_bar().content(move |icon_position| {
                            for (index, name) in NAMES.into_iter().take(3).enumerate() {
                                navigation_bar_item()
                                    .icon_position(icon_position)
                                    .label(I18N.text(name))
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
    let open = remember(|| SideSheetController::new(false));
    let selected = remember(|| 0usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    modal_side_sheet_provider()
        .position(SideSheetPosition::Start)
        .controller(open)
        .accessibility_label(I18N.text("ui-expanded-navigation-rail"))
        .on_close_request(move || open.with_mut(SideSheetController::close))
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
                        .accessibility_label(I18N.text("ui-open-modal-expanded-rail"))
                        .on_click(move || open.with_mut(SideSheetController::open));
                    gap(Dp(24.0));
                    text(I18N.text(NAMES[selected.get()])).style(type_style(24.0, 32.0));
                    text(I18N.text("ui-select-the-menu-to-reveal-navigation-over-this-page"))
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
                        .accessibility_label(I18N.text("ui-close-modal-expanded-rail"))
                        .on_click(move || open.with_mut(SideSheetController::close));
                })
                .content(move |rail_scope| {
                    for (index, name) in NAMES.into_iter().enumerate() {
                        navigation_rail_item(rail_scope)
                            .label(I18N.text(name))
                            .selected(selected.get() == index)
                            .on_click(move || {
                                selected.set(index);
                                open.with_mut(SideSheetController::close);
                            })
                            .icon(move || {
                                destination_icon(index, selected.get() == index);
                            });
                    }
                });
        });
}
