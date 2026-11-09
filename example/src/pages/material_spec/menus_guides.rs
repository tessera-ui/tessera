use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuColorStyle, MenuItemSelectionMode, MenuPosition, menu_group, menu_item},
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use crate::res::I18N;

pub(super) use super::super::article_helpers::caption;

use super::{
    ACCENT, INK,
    annotations::{self, Geometry, dotted, relative, watch},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const STATE_LABELS: [&str; 6] = [
    "ui-enabled",
    "ui-disabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed",
    "ui-active-main-menu-reveals-submenu",
];
const BASELINE_NAMES: [[&str; 4]; 4] = [
    ["b-row-1", "b-lead-1", "b-label-1", "b-trail-1"],
    ["b-row-2", "b-lead-2", "b-label-2", "b-trail-2"],
    ["b-row-3", "b-lead-3", "b-label-3", "b-trail-3"],
    ["b-row-4", "b-lead-4", "b-label-4", "b-trail-4"],
];
const BASELINE_ROLES: [&str; 9] = [
    "On surface variant",
    "On surface",
    "ui-on-surface-opacity-0-08",
    "Surface container",
    "On surface variant",
    "On surface variant",
    "On surface variant",
    "Surface container highest",
    "Outline variant",
];
const PLACEMENTS: [&str; 6] = [
    "ui-below-the-element",
    "ui-below-aligned-to-the-end",
    "ui-above-the-element",
    "ui-to-the-right",
    "ui-to-the-left",
    "ui-in-front-of-the-element",
];
const LEADER: &str = "#49454f";
const MEASURE: &str = "#ee7da4";

fn mid(rect: annotations::Rect) -> f64 {
    rect.y + rect.h / 2.0
}

/// Draws a leader from a 24 x 32dp marker at `(x, y)` to `(tx, ty)`.
fn leader(svg: &mut String, x: f64, y: f64, tx: f64, ty: f64, color: &str) {
    if (tx - x).abs() > (ty - y).abs() {
        let (start, end) = if tx > x { (x + 24.0, tx) } else { (tx, x) };
        dotted(svg, start, ty, end - start, false, color);
    } else {
        let (start, end) = if ty > y { (y + 32.0, ty) } else { (ty, y) };
        dotted(svg, tx, start, end - start, true, color);
    }
}

fn baseline_mark(geometry: Option<State<Geometry>>, slot: Option<usize>, part: usize) -> Modifier {
    match (geometry, slot) {
        (Some(geometry), Some(slot)) => watch(geometry, BASELINE_NAMES[slot][part]),
        _ => Modifier::new(),
    }
}

#[tessera]
pub(super) fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text(format!("•  {}", I18N.text(item)))
                .style(type_style(16.0, 24.0))
                .color(INK);
            gap(Dp(8.0));
        }
    });
}

#[tessera]
pub(super) fn badge(number: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let fill = scheme.on_surface;
    let ink = scheme.surface;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(fill, Shape::Ellipse),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(12.0, 16.0))
                .color(ink);
        });
}

#[tessera]
pub(super) fn legend(items: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, label) in items.clone().into_iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(36.0)))
                    .children(move || {
                        badge(index + 1);
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        text(I18N.text(&(label)))
                            .style(type_style(14.0, 20.0))
                            .color(INK);
                    });
            }
        });
}

#[tessera]
fn verdict(good: bool, #[prop(into)] note: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let tone = scheme.on_surface_variant;
    column().children(move || {
        boxed()
            .alignment(Alignment::Center)
            .modifier(
                Modifier::new()
                    .padding_symmetric(Dp(12.0), Dp(4.0))
                    .background_with_shape(
                        if good { DO_COLOR } else { DONT_COLOR },
                        Shape::rounded_rectangle(Dp(8.0)),
                    ),
            )
            .children(move || {
                text(I18N.text(if good { "ui-do" } else { "ui-dont" }))
                    .style(type_style(14.0, 20.0))
                    .color(Color::WHITE);
            });
        gap(Dp(8.0));
        text(I18N.text(&(note)))
            .style(type_style(14.0, 20.0))
            .color(tone);
    });
}

#[tessera]
fn state_item(
    label: String,
    container: Color,
    content: Color,
    enabled: bool,
    check: bool,
    arrow: bool,
    baseline: bool,
) {
    if baseline {
        let tint = if enabled {
            content
        } else {
            Color::new(content.r, content.g, content.b, 0.38)
        };
        baseline_row(
            label,
            if check { 5 } else { 0 },
            if arrow { 6 } else { 0 },
            tint,
            tint,
            container,
        );
        return;
    }
    let item = menu_item()
        .label(I18N.text(label))
        .container_color(container)
        .label_color(content)
        .enabled(enabled)
        .close_on_click(false)
        .shape(Shape::rounded_rectangle(Dp(12.0)))
        .height(Dp(44.0))
        .on_click(|| {});
    if check {
        item.leading_icon(filled::CHECK_SVG)
            .selection_mode(MenuItemSelectionMode::Radio)
            .selected(true);
    }
    if arrow {
        item.trailing_icon(filled::ARROW_RIGHT_SVG);
    }
}

/// Static rendering of one interaction state with real menu items.
#[tessera]
fn state_menu(state: usize, baseline: bool, selected: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    if state == 5 && !baseline {
        active_menus();
        return;
    }
    let marked = if selected || baseline {
        2
    } else if state == 2 {
        3
    } else {
        1
    };
    let selected_container = if baseline {
        scheme.surface_container_highest
    } else {
        scheme.tertiary_container
    };
    let selected_content = if baseline {
        scheme.on_surface
    } else {
        scheme.on_tertiary_container
    };
    let on_surface = scheme.on_surface;
    let group_color = scheme.surface_container;
    let mut group =
        menu_group().modifier(Modifier::new().width(if baseline { Dp(152.0) } else { Dp(124.0) }));
    if baseline {
        group = group
            .shape(Shape::rounded_rectangle(Dp(4.0)))
            .container_color(group_color);
    }
    group.children(move || {
        for index in 0..4 {
            let is_marked = index == marked;
            let base = if selected && is_marked {
                selected_container
            } else {
                Color::TRANSPARENT
            };
            let layer = if selected && is_marked {
                selected_content
            } else {
                on_surface
            };
            let container = if !is_marked || state == 1 {
                if state == 1 && selected && is_marked {
                    Color::new(on_surface.r, on_surface.g, on_surface.b, 0.12)
                } else {
                    base
                }
            } else {
                match state {
                    2 => base.blend_over(layer, 0.08),
                    3 | 4 => base.blend_over(layer, 0.10),
                    _ => base,
                }
            };
            let content = if selected && is_marked {
                selected_content
            } else {
                on_surface
            };
            let label = I18N.format("ui-item-value", [("v0", (index + 1).into())]);
            let arrow = index == 2 && !selected;
            let check = selected && is_marked;
            if state == 3 && is_marked {
                let ring = scheme.secondary;
                boxed()
                    .modifier(Modifier::new().fill_max_width().border_with_shape(
                        Dp(3.0),
                        ring,
                        if baseline {
                            Shape::RECTANGLE
                        } else {
                            Shape::rounded_rectangle(Dp(12.0))
                        },
                    ))
                    .children(move || {
                        state_item(
                            label.clone(),
                            container,
                            content,
                            true,
                            check,
                            arrow,
                            baseline,
                        );
                    });
            } else {
                state_item(
                    label,
                    container,
                    content,
                    state != 1,
                    check,
                    arrow,
                    baseline,
                );
            }
        }
    });
}

#[tessera]
fn active_menus() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let on_surface = scheme.on_surface;
    row().children(move || {
        menu_group()
            .shape(Shape::rounded_rectangle(Dp(4.0)))
            .modifier(Modifier::new().width(Dp(112.0)))
            .children(move || {
                for index in 0..3 {
                    state_item(
                        I18N.format("ui-item-value", [("v0", (index + 1).into())]),
                        if index == 1 {
                            Color::TRANSPARENT.blend_over(on_surface, 0.10)
                        } else {
                            Color::TRANSPARENT
                        },
                        on_surface,
                        true,
                        false,
                        index == 1,
                        false,
                    );
                }
            });
        column().children(move || {
            gap(Dp(36.0));
            menu_group()
                .modifier(Modifier::new().width(Dp(112.0)))
                .children(move || {
                    for index in 0..3 {
                        state_item(
                            I18N.format("ui-item-value", [("v0", (index + 1).into())]),
                            if index == 1 {
                                Color::TRANSPARENT.blend_over(on_surface, 0.08)
                            } else {
                                Color::TRANSPARENT
                            },
                            on_surface,
                            true,
                            false,
                            false,
                            false,
                        );
                    }
                });
        });
    });
}

/// Light and dark grids of every interaction state.
#[tessera]
pub(super) fn states_figure(baseline: bool, selected: bool) {
    let count = if baseline { 5 } else { 6 };
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                state_panel(baseline, selected, count);
            });
    }
    gap(Dp(16.0));
    legend(
        STATE_LABELS[..count]
            .iter()
            .map(|label| (*label).to_owned())
            .collect(),
    );
}

#[tessera]
fn state_panel(baseline: bool, selected: bool, count: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(scheme.surface, Shape::RECTANGLE)
                .padding_all(Dp(32.0)),
        )
        .children(move || {
            flow_row()
                .main_axis_alignment(MainAxisAlignment::Center)
                .item_spacing(Dp(48.0))
                .line_spacing(Dp(32.0))
                .children(move || {
                    for state in 0..count {
                        column()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                badge(state + 1);
                                gap(Dp(16.0));
                                state_menu(state, baseline, selected);
                            });
                    }
                });
        });
}

fn watched(geometry: Option<State<Geometry>>, name: &'static str) -> Modifier {
    match geometry {
        Some(geometry) => watch(geometry, name),
        None => Modifier::new(),
    }
}

/// Icon used by the static baseline figures, always drawn at the baseline 24dp
/// size.
#[tessera]
fn baseline_icon(code: u8, tint: Color) {
    let vector = match code {
        1 => filled::VISIBILITY_SVG,
        2 => filled::CONTENT_COPY_SVG,
        3 => filled::EDIT_SVG,
        4 => filled::CLOUD_SVG,
        6 => filled::ARROW_RIGHT_SVG,
        7 => filled::CONTENT_CUT_SVG,
        8 => filled::CONTENT_PASTE_SVG,
        9 => filled::SHARE_SVG,
        10 => filled::LINK_SVG,
        11 => filled::DOWNLOAD_SVG,
        12 => filled::FOLDER_SVG,
        13 => filled::IMAGE_SVG,
        14 => filled::UPLOAD_SVG,
        _ => filled::CHECK_SVG,
    };
    icon().painter(vector).size(Dp(24.0)).tint(tint);
}

/// Static baseline menu row: 48dp tall, 12dp padding and gaps, 24dp icons.
#[tessera]
fn baseline_row(
    label: String,
    leading: u8,
    trailing: u8,
    content: Color,
    muted: Color,
    container: Color,
    #[prop(into)] supporting: Option<String>,
    #[prop(into)] trailing_text: Option<String>,
    geometry: Option<State<Geometry>>,
    slot: Option<usize>,
) {
    let height = if supporting.is_some() {
        Dp(56.0)
    } else {
        Dp(48.0)
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            baseline_mark(geometry, slot, 0)
                .fill_max_width()
                .height(height)
                .background_with_shape(container, Shape::RECTANGLE)
                .padding_symmetric(Dp(12.0), Dp::ZERO),
        )
        .children(move || {
            if leading != 0 {
                boxed()
                    .modifier(baseline_mark(geometry, slot, 1).size(Dp(24.0), Dp(24.0)))
                    .children(move || {
                        baseline_icon(leading, muted);
                    });
                spacer().modifier(Modifier::new().width(Dp(12.0)));
            }
            let label = label.clone();
            let supporting = supporting.clone();
            column()
                .modifier(baseline_mark(geometry, slot, 2).weight(1.0))
                .children(move || {
                    text(I18N.text(&(label)))
                        .style(type_style(14.0, 20.0))
                        .color(content);
                    if let Some(supporting) = supporting.clone() {
                        text(I18N.text(supporting))
                            .style(type_style(12.0, 16.0))
                            .color(muted);
                    }
                });
            if let Some(shortcut) = trailing_text.clone() {
                spacer().modifier(Modifier::new().width(Dp(12.0)));
                text(I18N.text(shortcut))
                    .style(type_style(14.0, 20.0))
                    .color(muted)
                    .modifier(baseline_mark(geometry, slot, 3));
            }
            if trailing != 0 {
                spacer().modifier(Modifier::new().width(Dp(12.0)));
                boxed()
                    .modifier(baseline_mark(geometry, slot, 3).size(Dp(24.0), Dp(24.0)))
                    .children(move || {
                        baseline_icon(trailing, muted);
                    });
            }
        });
}

#[tessera]
fn baseline_divider(geometry: Option<State<Geometry>>) {
    boxed()
        .modifier(
            watched(geometry, "b-divider")
                .fill_max_width()
                .padding_symmetric(Dp::ZERO, Dp(8.0)),
        )
        .children(|| {
            horizontal_divider();
        });
}

#[tessera]
fn baseline_sample(geometry: Option<State<Geometry>>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let content = scheme.on_surface;
    let muted = scheme.on_surface_variant;
    let hover = Color::TRANSPARENT.blend_over(scheme.on_surface, 0.08);
    let selected = scheme.surface_container_highest;
    let container = scheme.surface_container;
    column()
        .modifier(
            watched(geometry, "b-menu")
                .width(Dp(200.0))
                .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                .padding_symmetric(Dp::ZERO, Dp(8.0)),
        )
        .children(move || {
            baseline_row("ui-item-1".into(), 1, 0, content, muted, hover)
                .geometry_optional(geometry)
                .slot(0);
            baseline_row("ui-item-2".into(), 2, 0, content, muted, Color::TRANSPARENT)
                .trailing_text("⌘C".to_owned())
                .geometry_optional(geometry)
                .slot(1);
            baseline_row("ui-item-3".into(), 3, 5, content, muted, selected)
                .geometry_optional(geometry)
                .slot(2);
            baseline_divider().geometry_optional(geometry);
            baseline_row("ui-item-4".into(), 4, 6, content, muted, Color::TRANSPARENT)
                .geometry_optional(geometry)
                .slot(3);
        });
}

#[tessera]
fn baseline_anatomy() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            baseline_sample().geometry(geometry);
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    baseline_annotations(geometry.with(annotations::relative_geometry));
                });
        });
    gap(Dp(16.0));
    legend(
        [
            "ui-list-item",
            "ui-list-item-leading-icon",
            "ui-list-item-trailing-icon",
            "ui-container",
            "ui-list-item-trailing-text",
            "ui-divider",
        ]
        .map(String::from)
        .to_vec(),
    );
}

#[tessera]
fn baseline_annotations(values: Geometry) {
    let (
        Some(root),
        Some(menu),
        Some(lead),
        Some(second),
        Some(shortcut),
        Some(third),
        Some(check),
        Some(divider),
    ) = (
        values.get("figure"),
        relative(&values, "b-menu"),
        relative(&values, BASELINE_NAMES[0][1]),
        relative(&values, BASELINE_NAMES[1][0]),
        relative(&values, BASELINE_NAMES[1][3]),
        relative(&values, BASELINE_NAMES[2][0]),
        relative(&values, BASELINE_NAMES[2][3]),
        relative(&values, "b-divider"),
    )
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let right = menu.x + menu.w;
    let lead_x = lead.x + lead.w / 2.0;
    let mut svg = String::new();
    for (number, x, y, tx, ty) in [
        (
            1,
            menu.x - 64.0,
            mid(second) - 16.0,
            menu.x + 4.0,
            mid(second),
        ),
        (2, lead_x - 12.0, menu.y - 64.0, lead_x, lead.y),
        (
            3,
            right + 36.0,
            mid(third) - 16.0,
            check.x + check.w,
            mid(third),
        ),
        (4, right + 36.0, menu.y - 16.0, right - 4.0, menu.y),
        (
            5,
            right + 36.0,
            mid(second) - 16.0,
            shortcut.x + shortcut.w,
            mid(second),
        ),
        (
            6,
            menu.x - 64.0,
            mid(divider) - 16.0,
            menu.x + 4.0,
            mid(divider),
        ),
    ] {
        leader(&mut svg, x, y, tx, ty, LEADER);
        annotations::marker(number, Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn baseline_colors() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            for dark in [false, true] {
                boxed()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        material_theme()
                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                            .child(move || {
                                baseline_color_tile();
                            });
                    });
            }
        });
    gap(Dp(16.0));
    caption("ui-baseline-menu-color-roles-used-for-light-and-dark-themes");
    gap(Dp(8.0));
    legend(BASELINE_ROLES.map(String::from).to_vec());
}

#[tessera]
fn baseline_color_tile() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let dark = scheme.is_dark;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(400.0))
                .background_with_shape(scheme.surface, Shape::RECTANGLE),
        )
        .children(move || {
            baseline_sample().geometry(geometry);
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    baseline_color_annotations(geometry.with(annotations::relative_geometry), dark);
                });
        });
}

#[tessera]
fn baseline_color_annotations(values: Geometry, dark: bool) {
    let (
        Some(root),
        Some(menu),
        Some(first),
        Some(lead),
        Some(label),
        Some(second),
        Some(shortcut),
        Some(third),
        Some(check),
        Some(divider),
        Some(fourth),
        Some(arrow),
    ) = (
        values.get("figure"),
        relative(&values, "b-menu"),
        relative(&values, BASELINE_NAMES[0][0]),
        relative(&values, BASELINE_NAMES[0][1]),
        relative(&values, BASELINE_NAMES[0][2]),
        relative(&values, BASELINE_NAMES[1][0]),
        relative(&values, BASELINE_NAMES[1][3]),
        relative(&values, BASELINE_NAMES[2][0]),
        relative(&values, BASELINE_NAMES[2][3]),
        relative(&values, "b-divider"),
        relative(&values, BASELINE_NAMES[3][0]),
        relative(&values, BASELINE_NAMES[3][3]),
    )
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let tone = if dark { "#cac4d0" } else { LEADER };
    let right = menu.x + menu.w;
    let top = menu.y - 72.0;
    let lead_x = lead.x + lead.w / 2.0;
    let label_x = label.x + 16.0;
    let state_x = first.x + first.w - 48.0;
    let selected_x = third.x + 112.0;
    let mut svg = String::new();
    for (index, (x, y, tx, ty)) in [
        (lead_x - 12.0, top, lead_x, lead.y),
        (label_x - 12.0, top, label_x, mid(first) - 6.0),
        (state_x - 12.0, top, state_x, first.y + 8.0),
        (right + 36.0, menu.y - 12.0, right - 2.0, menu.y + 4.0),
        (
            right + 36.0,
            mid(second) - 16.0,
            shortcut.x + shortcut.w,
            mid(second),
        ),
        (
            right + 36.0,
            mid(third) - 16.0,
            check.x + check.w,
            mid(third),
        ),
        (
            right + 36.0,
            mid(fourth) - 16.0,
            arrow.x + arrow.w,
            mid(fourth),
        ),
        (
            selected_x - 12.0,
            menu.y + menu.h + 32.0,
            selected_x,
            third.y + third.h - 4.0,
        ),
        (
            menu.x - 64.0,
            mid(divider) - 16.0,
            menu.x + 4.0,
            mid(divider),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        leader(&mut svg, x, y, tx, ty, tone);
        super::menu_color_marker(index + 1, BASELINE_ROLES[index].into(), Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

/// The baseline menu shown in the Specs variants section.
#[tessera]
pub(super) fn baseline_variant() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let content = scheme.on_surface;
    let muted = scheme.on_surface_variant;
    let selected = scheme.surface_container_highest;
    let container = scheme.surface_container;
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                column()
                    .modifier(
                        Modifier::new()
                            .width(Dp(200.0))
                            .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                            .padding_symmetric(Dp::ZERO, Dp(8.0)),
                    )
                    .children(move || {
                        baseline_row(
                            "ui-menu-item".into(),
                            0,
                            6,
                            content,
                            muted,
                            Color::TRANSPARENT,
                        );
                        baseline_row("ui-menu-item".into(), 5, 0, content, muted, selected)
                            .supporting(I18N.text("ui-supporting-text"));
                        baseline_row(
                            "ui-menu-item".into(),
                            0,
                            0,
                            content,
                            muted,
                            Color::TRANSPARENT,
                        );
                        baseline_divider();
                        baseline_row(
                            "ui-menu-item".into(),
                            7,
                            6,
                            content,
                            muted,
                            Color::TRANSPARENT,
                        );
                    });
                gap(Dp(16.0));
                badge(1);
            });
    }));
    gap(Dp(12.0));
    caption(
        "ui-a-baseline-menu-has-square-corners-as-compared-to-a-vertical-menu-s-round-corners-and-expr",
    );
}

#[tessera]
fn trigger_tile(kind: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let accent = scheme.secondary_container;
    let on_accent = scheme.on_secondary_container;
    let highlight = Color::TRANSPARENT.blend_over(scheme.primary, 0.24);
    let tone = scheme.on_surface_variant;
    let field = scheme.surface_container_highest;
    let line = scheme.primary;
    let container = scheme.surface_container;
    let content = scheme.on_surface;
    let faded = Color::new(content.r, content.g, content.b, 0.38);
    column()
        .modifier(Modifier::new().width(Dp(240.0)))
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    badge(kind + 1);
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                    match kind {
                        0 => {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .height(Dp(32.0))
                                        .background_with_shape(
                                            accent,
                                            Shape::rounded_rectangle(Dp(16.0)),
                                        )
                                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                                )
                                .children(move || {
                                    text(I18N.text("ui-activated"))
                                        .style(type_style(14.0, 20.0))
                                        .color(on_accent);
                                });
                        }
                        1 => {
                            column()
                                .modifier(
                                    Modifier::new()
                                        .width(Dp(176.0))
                                        .background_with_shape(
                                            field,
                                            Shape::rounded_rectangle(Dp(4.0)),
                                        )
                                        .border_with_shape(
                                            Dp(1.0),
                                            line,
                                            Shape::rounded_rectangle(Dp(4.0)),
                                        )
                                        .padding_symmetric(Dp(12.0), Dp(6.0)),
                                )
                                .children(move || {
                                    text(I18N.text("ui-label"))
                                        .style(type_style(12.0, 16.0))
                                        .color(line);
                                    text(I18N.text("ui-item-2"))
                                        .style(type_style(16.0, 24.0))
                                        .color(INK);
                                });
                        }
                        2 => {
                            icon()
                                .painter(filled::MORE_VERT_SVG)
                                .size(Dp(24.0))
                                .tint(tone);
                        }
                        _ => {
                            text(I18N.text("ui-although-a-little-too-small"))
                                .style(type_style(14.0, 20.0))
                                .modifier(
                                    Modifier::new()
                                        .background_with_shape(
                                            highlight,
                                            Shape::rounded_rectangle(Dp(4.0)),
                                        )
                                        .padding_symmetric(Dp(4.0), Dp::ZERO),
                                );
                        }
                    }
                });
            gap(Dp(8.0));
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(176.0))
                        .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                        .padding_symmetric(Dp::ZERO, Dp(8.0)),
                )
                .children(move || {
                    let labels: [&str; 2] = match kind {
                        0 | 1 => ["ui-item-1", "ui-item-2"],
                        2 => ["ui-refresh", "ui-settings"],
                        _ => ["ui-undo", "ui-redo"],
                    };
                    for (index, label) in labels.into_iter().enumerate() {
                        let tint = if kind == 3 && index == 1 {
                            faded
                        } else {
                            content
                        };
                        baseline_row(label.into(), 0, 0, tint, tint, Color::TRANSPARENT);
                    }
                });
        });
}

/// Every baseline menu subsection shown under Specs.
#[tessera]
pub(super) fn baseline_section() {
    super::prose(
        "ui-the-baseline-menu-variant-is-available-and-continues-to-work-in-existing-products-however",
    );
    gap(Dp(32.0));
    subheading("ui-baseline-tokens-specs".into());
    gap(Dp(16.0));
    spec_table(
        vec!["ui-token-group".into(), "ui-value".into()],
        vec![
            vec!["ui-container-shape".into(), "ui-extra-small-4dp".into()],
            vec!["ui-container-color".into(), "Surface container".into()],
            vec!["ui-container-elevation".into(), "ui-level-2-3dp".into()],
            vec!["ui-list-item-height".into(), "48dp".into()],
            vec!["ui-label-text".into(), "Label large".into()],
            vec![
                "ui-leading-trailing-icon".into(),
                "24dp, On surface variant".into(),
            ],
            vec![
                "ui-selected-container".into(),
                "Surface container highest".into(),
            ],
            vec![
                "ui-hover-state-layer".into(),
                "ui-on-surface-opacity-0-08-3932".into(),
            ],
            vec![
                "ui-focus-pressed-state-layer".into(),
                "ui-on-surface-opacity-0-10".into(),
            ],
            vec!["ui-focus-indicator".into(), "Secondary, 3dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-anatomy".into());
    gap(Dp(16.0));
    baseline_anatomy();
    gap(Dp(32.0));
    subheading("ui-color".into());
    gap(Dp(16.0));
    baseline_colors();
    gap(Dp(32.0));
    subheading("ui-states".into());
    gap(Dp(16.0));
    subheading("ui-default-menu-items".into()).level(4);
    gap(Dp(16.0));
    states_figure(true, false);
    gap(Dp(24.0));
    subheading("ui-selected-menu-items".into()).level(4);
    gap(Dp(16.0));
    states_figure(true, true);
    gap(Dp(12.0));
    caption("ui-state-specs-are-in-the-token-module-above");
    gap(Dp(32.0));
    subheading("ui-measurements".into());
    gap(Dp(16.0));
    baseline_measurements();
    gap(Dp(32.0));
    subheading("ui-configurations".into());
    gap(Dp(16.0));
    super::prose(
        "ui-a-baseline-menu-appears-when-a-person-interacts-with-a-button-action-or-other-control",
    );
    gap(Dp(8.0));
    super::prose("ui-a-few-examples");
    gap(Dp(8.0));
    legend(
        [
            "ui-button",
            "ui-text-field",
            "ui-icon-button",
            "ui-selected-text",
        ]
        .map(String::from)
        .to_vec(),
    );
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for kind in 0..4 {
                    trigger_tile(kind);
                }
            });
    }));
}

#[tessera]
fn baseline_measurements() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            baseline_measure_menu(geometry);
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    baseline_measure_annotations(geometry.with(annotations::relative_geometry));
                });
        });
    gap(Dp(12.0));
    caption("ui-baseline-menu-padding-and-size-measurements");
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-container-width".into(), "ui-112dp-min-280dp-max".into()],
            vec!["ui-corner-radius".into(), "4dp".into()],
            vec![
                "ui-vertical-label-text-alignment".into(),
                "ui-center-aligned".into(),
            ],
            vec![
                "ui-horizontal-label-text-alignment".into(),
                "ui-start-aligned".into(),
            ],
            vec!["ui-left-right-padding".into(), "12dp".into()],
            vec!["ui-left-right-padding-with-icon-6959".into(), "12dp".into()],
            vec!["ui-list-item-height".into(), "48dp".into()],
            vec![
                "ui-padding-between-elements-within-a-list-item".into(),
                "12dp".into(),
            ],
            vec!["ui-divider-top-bottom-padding".into(), "8dp".into()],
            vec!["ui-divider-height".into(), "1dp".into()],
            vec!["ui-divider-width".into(), "ui-dynamic".into()],
            vec!["ui-leading-trailing-icon-size".into(), "24dp".into()],
        ],
    );
}

#[tessera]
fn baseline_measure_menu(geometry: State<Geometry>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let content = scheme.on_surface;
    let muted = scheme.on_surface_variant;
    let hover = Color::TRANSPARENT.blend_over(scheme.on_surface, 0.08);
    let container = scheme.surface_container;
    column()
        .modifier(
            watch(geometry, "b-menu")
                .width(Dp(200.0))
                .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                .padding_symmetric(Dp::ZERO, Dp(8.0)),
        )
        .children(move || {
            for (index, (label, code, shortcut)) in [
                ("ui-item-1", 7, "⌘X"),
                ("ui-item-2", 2, "⌘C"),
                ("ui-item-3", 8, "⌘V"),
            ]
            .into_iter()
            .enumerate()
            {
                baseline_row(
                    label.into(),
                    code,
                    0,
                    content,
                    muted,
                    if index == 1 {
                        hover
                    } else {
                        Color::TRANSPARENT
                    },
                )
                .trailing_text(I18N.text(shortcut))
                .geometry(geometry)
                .slot(index);
            }
            baseline_divider().geometry(geometry);
            baseline_row("ui-item-4".into(), 4, 6, content, muted, Color::TRANSPARENT)
                .geometry(geometry)
                .slot(3);
        });
}

#[tessera]
fn baseline_measure_annotations(values: Geometry) {
    let (
        Some(root),
        Some(menu),
        Some(first),
        Some(lead),
        Some(second),
        Some(divider),
        Some(fourth),
        Some(arrow),
    ) = (
        values.get("figure"),
        relative(&values, "b-menu"),
        relative(&values, BASELINE_NAMES[0][0]),
        relative(&values, BASELINE_NAMES[0][1]),
        relative(&values, BASELINE_NAMES[1][0]),
        relative(&values, "b-divider"),
        relative(&values, BASELINE_NAMES[3][0]),
        relative(&values, BASELINE_NAMES[3][3]),
    )
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let right = menu.x + menu.w;
    let bottom = menu.y + menu.h;
    let gap_x = lead.x + lead.w;
    let mut svg = String::new();
    for (x, y, w, h) in [
        (menu.x, menu.y, 12.0, menu.h),
        (right - 12.0, menu.y, 12.0, menu.h),
        (menu.x, menu.y, menu.w, 8.0),
        (menu.x, bottom - 8.0, menu.w, 8.0),
        (gap_x, first.y, 12.0, first.h),
        (menu.x, divider.y, menu.w, divider.h),
    ] {
        svg.push_str(&format!(
            "<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='{MEASURE}' fill-opacity='.3'/>"
        ));
    }
    for (x, y, length) in [
        (right + 6.0, second.y, second.h),
        (menu.x - 6.0, fourth.y, fourth.h),
        (menu.x - 6.0, divider.y, divider.h),
    ] {
        dotted(&mut svg, x, y, length, true, MEASURE);
    }
    dotted(
        &mut svg,
        menu.x - 12.0,
        mid(first),
        lead.x - menu.x + 12.0,
        false,
        MEASURE,
    );
    dotted(&mut svg, right, mid(divider), 12.0, false, MEASURE);
    dotted(
        &mut svg,
        arrow.x + arrow.w,
        mid(fourth),
        right - arrow.x - arrow.w + 12.0,
        false,
        MEASURE,
    );
    for x in [menu.x, gap_x, right - 12.0] {
        dotted(&mut svg, x, bottom + 4.0, 12.0, false, MEASURE);
    }
    for (label, x, y) in [
        ("4", menu.x - 22.0, menu.y - 24.0),
        ("8", right + 12.0, menu.y - 8.0),
        ("24", menu.x - 36.0, mid(first) - 10.0),
        ("48", right + 14.0, mid(second) - 10.0),
        ("16", menu.x - 36.0, mid(divider) - 10.0),
        ("1", right + 16.0, mid(divider) - 10.0),
        ("48", menu.x - 36.0, mid(fourth) - 10.0),
        ("24", right + 16.0, mid(fourth) - 10.0),
        ("8", right + 12.0, bottom - 12.0),
        ("12", menu.x - 2.0, bottom + 8.0),
        ("12", gap_x - 2.0, bottom + 8.0),
        ("12", right - 14.0, bottom + 8.0),
    ] {
        annotations::dimension(label.into(), Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

/// A parent menu with an open submenu whose selected item uses vibrant color.
#[tessera]
pub(super) fn submenu_pair(#[prop(into)] parent: String, options: Vec<String>, selected: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let hover = Color::TRANSPARENT.blend_over(scheme.on_surface, 0.10);
    let vibrant = scheme.tertiary;
    let on_vibrant = scheme.on_tertiary;
    row().children(move || {
        let parent = parent.clone();
        let options = options.clone();
        menu_group()
            .modifier(Modifier::new().width(Dp(176.0)))
            .children(move || {
                menu_item()
                    .label(I18N.text("ui-bold"))
                    .close_on_click(false)
                    .on_click(|| {});
                menu_item()
                    .label(I18N.text(&(parent)))
                    .container_color(hover)
                    .trailing_icon(filled::ARROW_RIGHT_SVG)
                    .close_on_click(false)
                    .on_click(|| {});
                menu_item()
                    .label(I18N.text("ui-italic"))
                    .close_on_click(false)
                    .on_click(|| {});
            });
        spacer().modifier(Modifier::new().width(Dp(4.0)));
        column().children(move || {
            let options = options.clone();
            gap(Dp(46.0));
            menu_group()
                .color_style(MenuColorStyle::Standard)
                .modifier(Modifier::new().width(Dp(176.0)))
                .children(move || {
                    for (index, label) in options.clone().into_iter().enumerate() {
                        let item = menu_item()
                            .label(I18N.text(label))
                            .close_on_click(false)
                            .on_click(|| {});
                        if index == selected {
                            item.leading_icon(filled::CHECK_SVG)
                                .selection_mode(MenuItemSelectionMode::Radio)
                                .selected(true)
                                .container_color(vibrant)
                                .label_color(on_vibrant);
                        }
                    }
                });
        });
    });
}

fn line_spacing() -> Vec<String> {
    ["ui-single", "1.15", "ui-double", "ui-custom-1-2"]
        .map(String::from)
        .to_vec()
}

#[tessera]
pub(super) fn line_spacing_figure() {
    panel(RenderSlot::new(|| {
        submenu_pair("ui-line-spacing", line_spacing(), 3);
    }));
}

#[tessera]
pub(super) fn opening_figure() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let highlight = Color::TRANSPARENT.blend_over(scheme.primary, 0.24);
    panel(RenderSlot::new(move || {
        column().children(move || {
            row().children(move || {
                text(I18N.text("ui-it-showed-a-lady-fitted-out"))
                    .style(type_style(16.0, 24.0))
                    .color(INK);
                text(I18N.text("ui-with-a-fur-hat-and-fur-boa"))
                    .style(type_style(16.0, 24.0))
                    .color(INK)
                    .modifier(
                        Modifier::new()
                            .background_with_shape(highlight, Shape::rounded_rectangle(Dp(4.0))),
                    );
            });
            gap(Dp(8.0));
            edit_menu();
        });
    }));
}

#[tessera]
fn edit_menu() {
    column()
        .modifier(Modifier::new().width(Dp(200.0)))
        .children(move || {
            menu_group()
                .position(MenuPosition::First)
                .children(move || {
                    menu_item()
                        .label(I18N.text("ui-undo"))
                        .leading_icon(filled::UNDO_SVG)
                        .close_on_click(false)
                        .on_click(|| {});
                    menu_item()
                        .label(I18N.text("ui-redo"))
                        .enabled(false)
                        .close_on_click(false)
                        .on_click(|| {});
                });
            menu_group().position(MenuPosition::Last).children(move || {
                menu_item()
                    .label(I18N.text("ui-cut"))
                    .leading_icon(filled::CONTENT_CUT_SVG)
                    .close_on_click(false)
                    .on_click(|| {});
                menu_item()
                    .label(I18N.text("ui-copy"))
                    .leading_icon(filled::CONTENT_COPY_SVG)
                    .close_on_click(false)
                    .on_click(|| {});
                menu_item()
                    .label(I18N.text("ui-paste"))
                    .leading_icon(filled::CONTENT_PASTE_SVG)
                    .close_on_click(false)
                    .on_click(|| {});
            });
        });
}

#[tessera]
pub(super) fn disabled_figure() {
    panel(RenderSlot::new(|| {
        edit_menu();
    }));
}

#[tessera]
pub(super) fn context_figure() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let link = scheme.primary;
    panel(RenderSlot::new(move || {
        column().children(move || {
            text(I18N.text("ui-city-council-approves-new-riverside-park"))
                .style(type_style(16.0, 24.0))
                .color(link);
            gap(Dp(8.0));
            menu_group()
                .modifier(Modifier::new().width(Dp(220.0)))
                .children(move || {
                    for label in [
                        "ui-open-in-new-window",
                        "ui-save-link-as",
                        "ui-copy-address",
                    ] {
                        menu_item()
                            .label(I18N.text(label))
                            .close_on_click(false)
                            .on_click(|| {});
                    }
                    horizontal_divider();
                    menu_item()
                        .label(I18N.text("ui-inspect"))
                        .close_on_click(false)
                        .on_click(|| {});
                });
        });
    }));
}

#[tessera]
pub(super) fn density_figure() {
    panel(RenderSlot::new(move || {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(32.0))
            .line_spacing(Dp(32.0))
            .children(move || {
                for level in 0..4_u8 {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            menu_group()
                                .modifier(Modifier::new().width(Dp(144.0)))
                                .children(move || {
                                    for index in 0..3 {
                                        menu_item()
                                            .label(I18N.format(
                                                "ui-item-value",
                                                [("v0", (index + 1).into())],
                                            ))
                                            .height(Dp(44.0 - 4.0 * f64::from(level)))
                                            .close_on_click(false)
                                            .on_click(|| {});
                                    }
                                });
                            gap(Dp(12.0));
                            caption(if level == 0 {
                                "0".to_owned()
                            } else {
                                format!("-{level}")
                            });
                        });
                }
            });
    }));
}

#[tessera]
pub(super) fn focus_examples() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let ring = scheme.secondary;
    let surface = scheme.surface_container_low;
    flow_row()
        .item_spacing(Dp(32.0))
        .line_spacing(Dp(32.0))
        .children(move || {
            for good in [true, false] {
                column()
                    .modifier(Modifier::new().width(Dp(320.0)))
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .height(Dp(240.0))
                                    .background_with_shape(
                                        surface,
                                        Shape::rounded_rectangle(Dp(24.0)),
                                    ),
                            )
                            .children(move || {
                                menu_group()
                                    .modifier(Modifier::new().width(Dp(176.0)))
                                    .children(move || {
                                        menu_item()
                                            .label(I18N.text("ui-edit-name"))
                                            .close_on_click(false)
                                            .on_click(|| {});
                                        if good {
                                            boxed()
                                                .modifier(
                                                    Modifier::new()
                                                        .fill_max_width()
                                                        .border_with_shape(
                                                            Dp(3.0),
                                                            ring,
                                                            Shape::rounded_rectangle(Dp(12.0)),
                                                        ),
                                                )
                                                .children(move || {
                                                    menu_item()
                                                        .label(I18N.text("ui-share"))
                                                        .enabled(false)
                                                        .on_click(|| {});
                                                });
                                        } else {
                                            boxed()
                                                .modifier(
                                                    Modifier::new()
                                                        .fill_max_width()
                                                        .padding_symmetric(Dp(4.0), Dp(6.0))
                                                        .border_with_shape(
                                                            Dp(3.0),
                                                            ring,
                                                            Shape::rounded_rectangle(Dp(4.0)),
                                                        )
                                                        .padding_symmetric(Dp(4.0), Dp(6.0)),
                                                )
                                                .children(|| {
                                                    horizontal_divider();
                                                });
                                        }
                                        menu_item()
                                            .label(I18N.text("ui-get-link"))
                                            .close_on_click(false)
                                            .on_click(|| {});
                                    });
                            });
                        gap(Dp(12.0));
                        verdict(
                            good,
                            if good {
                                "ui-disabled-menu-items-can-receive-focus"
                            } else {
                                "ui-a-divider-or-gap-can-t-receive-focus"
                            },
                        );
                    });
            }
        });
}

/// Static placement diagram showing where a menu can appear around its trigger.
#[tessera]
pub(super) fn placement_figure() {
    panel(RenderSlot::new(|| {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for kind in 0..PLACEMENTS.len() {
                    placement_tile(kind);
                }
            });
    }));
    gap(Dp(16.0));
    legend(PLACEMENTS.map(String::from).to_vec());
}

#[tessera]
fn placement_tile(kind: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let frame = scheme.surface;
    let edge = scheme.outline_variant;
    let trigger = scheme.primary;
    let menu = scheme.surface_container_highest;
    let line = scheme.outline_variant;
    let (tx, ty, mx, my) = match kind {
        0 => (72.0, 24.0, 72.0, 56.0),
        1 => (112.0, 24.0, 96.0, 56.0),
        2 => (72.0, 132.0, 72.0, 48.0),
        3 => (24.0, 40.0, 84.0, 40.0),
        4 => (120.0, 40.0, 44.0, 40.0),
        _ => (72.0, 52.0, 64.0, 48.0),
    };
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    Modifier::new()
                        .size(Dp(200.0), Dp(184.0))
                        .background_with_shape(frame, Shape::rounded_rectangle(Dp(16.0)))
                        .border_with_shape(Dp(1.0), edge, Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(move || {
                    boxed().modifier(
                        Modifier::new()
                            .offset(Dp(tx), Dp(ty))
                            .size(Dp(56.0), Dp(28.0))
                            .background_with_shape(trigger, Shape::rounded_rectangle(Dp(14.0))),
                    );
                    column()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(mx), Dp(my))
                                .size(Dp(72.0), Dp(80.0))
                                .background_with_shape(menu, Shape::rounded_rectangle(Dp(8.0)))
                                .padding_symmetric(Dp(10.0), Dp(12.0)),
                        )
                        .children(move || {
                            for index in 0..3 {
                                if index > 0 {
                                    gap(Dp(12.0));
                                }
                                boxed().modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(8.0))
                                        .background_with_shape(
                                            line,
                                            Shape::rounded_rectangle(Dp(4.0)),
                                        ),
                                );
                            }
                        });
                });
            gap(Dp(8.0));
            badge(kind + 1);
        });
}

/// Static comparison of a compact bottom sheet and an expanded in-context menu.
#[tessera]
pub(super) fn adaptive_figure(compact: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let frame = scheme.surface;
    let edge = scheme.outline_variant;
    let block = scheme.surface_container_highest;
    let sheet = scheme.surface_container_low;
    let ink = scheme.on_surface;
    let muted = scheme.on_surface_variant;
    panel(RenderSlot::new(move || {
        if compact {
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(280.0))
                        .background_with_shape(frame, Shape::rounded_rectangle(Dp(28.0)))
                        .border_with_shape(Dp(1.0), edge, Shape::rounded_rectangle(Dp(28.0)))
                        .padding_all(Dp(12.0)),
                )
                .children(move || {
                    for _ in 0..2 {
                        boxed().modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(56.0))
                                .background_with_shape(block, Shape::rounded_rectangle(Dp(12.0))),
                        );
                        gap(Dp(12.0));
                    }
                    gap(Dp(16.0));
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .background_with_shape(sheet, Shape::rounded_rectangle(Dp(20.0)))
                                .padding_symmetric(Dp(16.0), Dp(12.0)),
                        )
                        .children(move || {
                            boxed().modifier(
                                Modifier::new()
                                    .size(Dp(32.0), Dp(4.0))
                                    .background_with_shape(
                                        muted,
                                        Shape::rounded_rectangle(Dp(2.0)),
                                    ),
                            );
                            gap(Dp(12.0));
                            for (code, label) in [
                                (9, "ui-share-with-a-contact"),
                                (10, "ui-copy-link-to-photo"),
                                (11, "ui-download-original"),
                                (12, "ui-move-to-album"),
                            ] {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
                                    .children(move || {
                                        baseline_icon(code, muted);
                                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                                        text(I18N.text(label))
                                            .style(type_style(14.0, 20.0))
                                            .color(ink);
                                    });
                            }
                        });
                });
        } else {
            row()
                .modifier(
                    Modifier::new()
                        .size(Dp(480.0), Dp(320.0))
                        .background_with_shape(frame, Shape::rounded_rectangle(Dp(24.0)))
                        .border_with_shape(Dp(1.0), edge, Shape::rounded_rectangle(Dp(24.0)))
                        .padding_all(Dp(16.0)),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(160.0)))
                        .children(move || {
                            for _ in 0..4 {
                                boxed().modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .height(Dp(48.0))
                                        .background_with_shape(
                                            block,
                                            Shape::rounded_rectangle(Dp(12.0)),
                                        ),
                                );
                                gap(Dp(12.0));
                            }
                        });
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                    boxed()
                        .alignment(Alignment::TopStart)
                        .modifier(
                            Modifier::new()
                                .weight(1.0)
                                .height(Dp(288.0))
                                .background_with_shape(block, Shape::rounded_rectangle(Dp(16.0))),
                        )
                        .children(move || {
                            menu_group()
                                .color_style(MenuColorStyle::Vibrant)
                                .modifier(
                                    Modifier::new().offset(Dp(56.0), Dp(48.0)).width(Dp(200.0)),
                                )
                                .children(move || {
                                    for label in [
                                        "ui-share",
                                        "ui-copy-link",
                                        "ui-download",
                                        "ui-move-to-album",
                                    ] {
                                        menu_item()
                                            .label(I18N.text(label))
                                            .close_on_click(false)
                                            .on_click(|| {});
                                    }
                                });
                        });
                });
        }
    }));
}

/// Static structure diagram of menu item slots and simple slot content.
#[tessera]
pub(super) fn slots_figure() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let outline = scheme.outline;
    let menu = scheme.surface_container;
    let ink = scheme.on_surface;
    let muted = scheme.on_surface_variant;
    let image = scheme.tertiary_container;
    let on_image = scheme.on_tertiary_container;
    let track = scheme.secondary_container;
    let bar = scheme.primary;
    let swatches = [
        scheme.primary,
        scheme.secondary,
        scheme.tertiary,
        scheme.error,
    ];
    panel(RenderSlot::new(move || {
        flow_row()
            .main_axis_alignment(MainAxisAlignment::Center)
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(32.0))
            .children(move || {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .modifier(
                                Modifier::new()
                                    .width(Dp(240.0))
                                    .height(Dp(56.0))
                                    .background_with_shape(menu, Shape::rounded_rectangle(Dp(8.0)))
                                    .padding_symmetric(Dp(12.0), Dp(8.0)),
                            )
                            .children(move || {
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(
                                        Modifier::new()
                                            .weight(1.0)
                                            .height(Dp(40.0))
                                            .border_with_shape(
                                                Dp(1.0),
                                                outline,
                                                Shape::rounded_rectangle(Dp(4.0)),
                                            ),
                                    )
                                    .children(move || {
                                        text(I18N.text("ui-slot"))
                                            .style(type_style(14.0, 20.0))
                                            .color(muted);
                                    });
                            });
                        gap(Dp(12.0));
                        badge(1);
                    });
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        column()
                            .modifier(
                                Modifier::new()
                                    .width(Dp(240.0))
                                    .background_with_shape(menu, Shape::rounded_rectangle(Dp(16.0)))
                                    .padding_symmetric(Dp::ZERO, Dp(8.0)),
                            )
                            .children(move || {
                                slot_image_row(image, on_image, ink);
                                slot_progress_row(muted, ink, track, bar);
                                slot_swatch_row(ink, swatches.to_vec());
                            });
                        gap(Dp(12.0));
                        badge(2);
                    });
            });
    }));
    gap(Dp(16.0));
    legend(
        [
            "ui-menu-item-container-with-a-swappable-slot",
            "ui-image-progress-indicator-and-color-swatch-slots",
        ]
        .map(String::from)
        .to_vec(),
    );
}

#[tessera]
fn slot_image_row(image: Color, on_image: Color, ink: Color) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(12.0), Dp::ZERO),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(image, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    baseline_icon(13, on_image);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text("ui-album-cover"))
                .style(type_style(14.0, 20.0))
                .color(ink);
        });
}

#[tessera]
fn slot_progress_row(muted: Color, ink: Color, track: Color, bar: Color) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .padding_symmetric(Dp(12.0), Dp::ZERO),
        )
        .children(move || {
            baseline_icon(14, muted);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    text(I18N.text("ui-uploading-2-of-5"))
                        .style(type_style(14.0, 20.0))
                        .color(ink);
                    gap(Dp(6.0));
                    boxed()
                        .alignment(Alignment::CenterStart)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(4.0))
                                .background_with_shape(track, Shape::rounded_rectangle(Dp(2.0))),
                        )
                        .children(move || {
                            boxed().modifier(
                                Modifier::new()
                                    .size(Dp(64.0), Dp(4.0))
                                    .background_with_shape(bar, Shape::rounded_rectangle(Dp(2.0))),
                            );
                        });
                });
        });
}

#[tessera]
fn slot_swatch_row(ink: Color, swatches: Vec<Color>) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(48.0))
                .padding_symmetric(Dp(12.0), Dp::ZERO),
        )
        .children(move || {
            text(I18N.text("ui-theme"))
                .style(type_style(14.0, 20.0))
                .color(ink);
            spacer().modifier(Modifier::new().weight(1.0));
            for (index, swatch) in swatches.clone().into_iter().enumerate() {
                if index > 0 {
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                }
                boxed().modifier(
                    Modifier::new()
                        .size(Dp(20.0), Dp(20.0))
                        .background_with_shape(swatch, Shape::Ellipse),
                );
            }
        });
}
