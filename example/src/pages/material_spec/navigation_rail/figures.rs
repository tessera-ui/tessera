//! Navigation rail reference figures for anatomy, color, states and guidance.
//!
//! ## Usage
//!
//! Illustrate rail specifications and usage guidance beside live native rails.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    badge::{badge, badge_with_content},
    boxed::boxed,
    column::column,
    extended_floating_action_button::extended_floating_action_button,
    floating_action_button::FloatingActionButtonElevation,
    icon::icon,
    material_icons::{Asset, filled, outlined},
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    navigation_rail::{navigation_rail, navigation_rail_item},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, provide_context, remember, tessera, use_context,
};

use super::super::{
    ACCENT, INK,
    annotations::{self, Geometry, Rect, relative, watch},
    gap, srgb, type_style,
};

/// Callout marker: number, geometry key and whether it sits on the trailing
/// lane.
pub(super) type Callout = (usize, &'static str, bool);

/// Geometry sink that static rail illustrations report their parts to.
#[derive(Clone, Copy)]
struct MockGeometry(State<Geometry>);

fn tagged(sink: Option<State<Geometry>>, name: &'static str) -> Modifier {
    match sink {
        Some(geometry) => watch(geometry, name),
        None => Modifier::new(),
    }
}

const ITEM_KEYS: [&str; 6] = ["item0", "item1", "item2", "item3", "item4", "item5"];
const ICON_KEYS: [&str; 6] = ["icon0", "icon1", "icon2", "icon3", "icon4", "icon5"];
const MEASURE: &str = "#e8457a";
// Interaction annotations (touch points, focus boxes and key chips) use the
// reference figures' fixed magenta rather than a theme role.
const INTERACTION: (u8, u8, u8) = (134, 70, 140);
const INTERACTION_DARK: (u8, u8, u8) = (106, 44, 113);
const INTERACTION_FILL: (u8, u8, u8) = (168, 122, 173);

pub(super) const TIMER_LABELS: [&str; 6] = [
    "Alarm",
    "Clock",
    "Timer",
    "Stopwatch",
    "Bedtime",
    "Settings",
];
pub(super) const FILE_LABELS: [&str; 4] = ["All files", "Recent", "Photos", "Library"];

fn glyph(set: usize, index: usize, active: bool) -> Asset {
    match (set, index, active) {
        (1, 0, true) => filled::ALARM_SVG,
        (1, 0, false) => outlined::ALARM_SVG,
        (1, 1, true) => filled::SCHEDULE_SVG,
        (1, 1, false) => outlined::SCHEDULE_SVG,
        (1, 2, true) => filled::HOURGLASS_FULL_SVG,
        (1, 2, false) => outlined::HOURGLASS_EMPTY_SVG,
        (1, 3, true) => filled::TIMER_SVG,
        (1, 3, false) => outlined::TIMER_SVG,
        (1, 4, true) => filled::BEDTIME_SVG,
        (1, 4, false) => outlined::BEDTIME_SVG,
        (1, _, true) => filled::SETTINGS_SVG,
        (1, _, false) => outlined::SETTINGS_SVG,
        (2, 0, true) => filled::FOLDER_SVG,
        (2, 0, false) => outlined::FOLDER_SVG,
        (2, 1, true) => filled::HISTORY_SVG,
        (2, 1, false) => outlined::HISTORY_SVG,
        (2, 2, true) => filled::IMAGE_SVG,
        (2, 2, false) => outlined::IMAGE_SVG,
        (2, _, true) => filled::PHOTO_LIBRARY_SVG,
        (2, _, false) => outlined::PHOTO_LIBRARY_SVG,
        (_, _, true) => filled::STARS_SVG,
        (_, _, false) => outlined::STARS_SVG,
    }
}

fn label_for(set: usize, index: usize) -> String {
    match set {
        1 => TIMER_LABELS[index.min(5)].to_string(),
        2 => FILE_LABELS[index.min(3)].to_string(),
        _ => "Label".to_string(),
    }
}

#[tessera]
fn dot_grid(geometry: State<Geometry>, dark: bool) {
    dot_grid_content(geometry.with(annotations::relative_geometry), dark);
}

#[tessera]
fn dot_grid_content(values: Geometry, dark: bool) {
    let Some(root) = values.get("figure") else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let tone = if dark { "#4a4458" } else { "#e7dcef" };
    let mut path = String::new();
    let mut y = 6.0;
    while y < height.0 {
        let mut x = 6.0;
        while x < width.0 {
            path.push_str(&format!("M{x} {y}h1v1h-1z"));
            x += 12.0;
        }
        y += 12.0;
    }
    if path.is_empty() {
        return;
    }
    annotations::lines(width, height, format!("<path d='{path}' fill='{tone}'/>"));
}

/// Reference canvas with optional dark Material 2025 theme, fill and dot grid.
#[tessera]
pub(super) fn canvas(
    #[prop(into)] label: String,
    height: Dp,
    content: RenderSlot,
    dark: Option<bool>,
    background: Option<Color>,
    dots: Option<bool>,
) {
    let dots = dots.unwrap_or(true);
    if dark.unwrap_or(false) {
        material_theme()
            .theme(|| MaterialTheme::from_seed(ACCENT, true))
            .child(move || {
                canvas_body(label.clone(), height, true, content.clone(), dots)
                    .background_optional(background);
            });
    } else {
        canvas_body(label, height, false, content, dots).background_optional(background);
    }
}

#[tessera]
fn canvas_body(
    label: String,
    height: Dp,
    dark: bool,
    content: RenderSlot,
    dots: bool,
    background: Option<Color>,
) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let background = background.unwrap_or(scheme.surface);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(height)
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .background(background)
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(label),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            if dots {
                dot_grid(geometry, dark);
            }
            content.render();
        });
}

/// Dark numbered badge used in figure captions and legends.
#[tessera]
pub(super) fn number(value: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(value.to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

/// Native navigation rail inside a measured frame with optional numbered
/// callouts.
#[tessera]
pub(super) fn specimen(
    #[prop(into)] label: String,
    expanded: bool,
    frame_width: Dp,
    frame_height: Dp,
    menu: Option<bool>,
    fab: Option<bool>,
    count: Option<usize>,
    badges: Option<bool>,
    max_badge: Option<bool>,
    named: Option<bool>,
    callouts: Option<Vec<Callout>>,
    measurements: Option<bool>,
    rail_height: Option<Dp>,
    badge_start: Option<usize>,
    container: Option<Color>,
    filled: Option<bool>,
    badge_series: Option<bool>,
) {
    let geometry = remember(Geometry::new);
    let badge_start = badge_start.unwrap_or(1);
    let selected = remember(|| 0_usize);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let mut rail_theme = theme.clone();
    rail_theme.color_scheme.surface = container.unwrap_or(theme.color_scheme.surface_container);
    let dark = theme.color_scheme.surface.r < 0.2;
    let filled = filled.unwrap_or(false);
    let badge_series = badge_series.unwrap_or(false);
    let menu = menu.unwrap_or(false);
    let fab = fab.unwrap_or(false);
    let count = count.unwrap_or(3);
    let badges = badges.unwrap_or(false);
    let max_badge = max_badge.unwrap_or(false);
    let named = named.unwrap_or(false);
    let callouts = callouts.unwrap_or_default();
    let measurements = measurements.unwrap_or(false);
    let rail_height = rail_height.unwrap_or(Dp(frame_height.0 - 48.0));
    let marks = usize::from(measurements);
    boxed()
        .alignment(Alignment::Center)
        .modifier(watch(geometry, "figure").size(frame_width, frame_height))
        .children(move || {
            let rail_theme = rail_theme.clone();
            let label = label.clone();
            boxed()
                .modifier(
                    watch(geometry, "container")
                        .width(Dp(if expanded { 220.0 } else { 96.0 }))
                        .height(rail_height)
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(label.clone()),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    let rail_theme = rail_theme.clone();
                    let label = label.clone();
                    material_theme()
                        .theme(move || rail_theme.clone())
                        .child(move || {
                            specimen_rail(
                                label.clone(),
                                expanded,
                                menu,
                                fab,
                                count,
                                badges,
                                max_badge,
                                named,
                                badge_start,
                                badge_series,
                                selected,
                                geometry,
                            );
                        });
                });
            let callouts = callouts.clone();
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    callout_overlay(geometry, expanded, callouts.clone(), dark, marks, filled);
                });
        });
}

#[tessera]
fn specimen_rail(
    label: String,
    expanded: bool,
    menu: bool,
    fab: bool,
    count: usize,
    badges: bool,
    max_badge: bool,
    named: bool,
    badge_start: usize,
    badge_series: bool,
    selected: State<usize>,
    geometry: State<Geometry>,
) {
    let header = if menu || fab {
        Some(RenderSlot::new(move || {
            specimen_header(label.clone(), expanded, menu, fab, geometry);
        }))
    } else {
        None
    };
    navigation_rail()
        .expanded(expanded)
        .header_optional(header)
        .content(move || {
            for index in 0..count {
                specimen_item(
                    index,
                    named,
                    badges,
                    max_badge,
                    badge_start,
                    badge_series,
                    selected,
                    geometry,
                );
            }
        });
}

#[tessera]
fn specimen_item(
    index: usize,
    named: bool,
    badges: bool,
    max_badge: bool,
    badge_start: usize,
    badge_series: bool,
    selected: State<usize>,
    geometry: State<Geometry>,
) {
    let name = if named {
        super::super::navigation::NAMES[index.min(4)].to_string()
    } else {
        "Label".to_string()
    };
    boxed()
        .modifier(watch(geometry, ITEM_KEYS[index.min(5)]))
        .children(move || {
            let item = navigation_rail_item()
                .label(name.clone())
                .selected(selected.get() == index)
                .on_click(move || selected.set(index))
                .icon(move || {
                    boxed()
                        .modifier(watch(geometry, ICON_KEYS[index.min(5)]))
                        .children(move || {
                            star(selected.get() == index);
                        });
                });
            if badge_series && index == badge_start + 2 {
                item.accessibility_description("999 or more new items")
                    .badge_shared(RenderSlot::new(move || {
                        boxed()
                            .modifier(watch(geometry, "max-badge"))
                            .children(move || {
                                badge_with_content().content(|| {
                                    text().content("999+");
                                });
                            });
                    }));
            } else if badge_series && index == badge_start {
                item.accessibility_description("New activity")
                    .badge_shared(RenderSlot::new(move || {
                        boxed()
                            .modifier(watch(geometry, "small-badge"))
                            .children(|| {
                                badge();
                            });
                    }));
            } else if badge_series && index == badge_start + 1 {
                item.accessibility_description("3 new items")
                    .badge_shared(RenderSlot::new(move || {
                        boxed()
                            .modifier(watch(geometry, "large-badge"))
                            .children(move || {
                                badge_with_content().content(|| {
                                    text().content("3");
                                });
                            });
                    }));
            } else if badges && index == badge_start {
                item.accessibility_description(if max_badge {
                    "999 or more new items"
                } else {
                    "3 new items"
                })
                .badge_shared(RenderSlot::new(move || {
                    boxed()
                        .modifier(watch(geometry, "large-badge"))
                        .children(move || {
                            badge_with_content().content(move || {
                                text()
                                    .content(if max_badge { "999+" } else { "3" })
                                    .modifier(watch(geometry, "badge-label"));
                            });
                        });
                }));
            } else if badges && index == badge_start + 1 {
                item.accessibility_description("New activity")
                    .badge_shared(RenderSlot::new(move || {
                        boxed()
                            .modifier(watch(geometry, "small-badge"))
                            .children(|| {
                                badge();
                            });
                    }));
            }
        });
}

#[tessera]
fn star(selected: bool) {
    icon()
        .painter(if selected {
            filled::STARS_SVG
        } else {
            outlined::STARS_SVG
        })
        .size(Dp(24.0));
}

#[tessera]
fn specimen_header(
    label: String,
    expanded: bool,
    menu: bool,
    fab: bool,
    geometry: State<Geometry>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        if menu {
            boxed()
                .alignment(if expanded {
                    Alignment::CenterStart
                } else {
                    Alignment::Center
                })
                .modifier(watch(geometry, "menu").size(Dp(56.0), Dp(48.0)).semantics(
                    SemanticsArgs {
                        role: Some(Role::Image),
                        label: Some(format!("{label} menu")),
                        ..Default::default()
                    },
                ))
                .children(move || {
                    icon()
                        .painter(if expanded {
                            filled::MENU_OPEN_SVG
                        } else {
                            filled::MENU_SVG
                        })
                        .size(Dp(24.0))
                        .tint(scheme.on_surface_variant);
                });
        }
        if menu && fab {
            gap(Dp(4.0));
        }
        if fab {
            let label = label.clone();
            boxed().modifier(watch(geometry, "fab")).children(move || {
                extended_floating_action_button("Label")
                    .expanded(expanded)
                    .icon(filled::EDIT_SVG)
                    .elevation(FloatingActionButtonElevation::new(
                        Dp::ZERO,
                        Dp::ZERO,
                        Dp::ZERO,
                        Dp::ZERO,
                    ))
                    .accessibility_label(format!("{label} compose"))
                    .on_click(|| {});
            });
        }
    });
}

/// Static collapsed rail illustration with callouts or baseline measurement
/// marks.
///
/// `marks` selects no marks (0), baseline size marks (2) or baseline padding
/// marks (3).
#[tessera]
pub(super) fn annotated_mock(
    width: Dp,
    height: Dp,
    content: RenderSlot,
    callouts: Option<Vec<Callout>>,
    marks: Option<usize>,
    filled: Option<bool>,
) {
    let geometry = remember(Geometry::new);
    let callouts = callouts.unwrap_or_default();
    let marks = marks.unwrap_or(0);
    let filled = filled.unwrap_or(false);
    let dark = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .color_scheme
        .surface
        .r
        < 0.2;
    boxed()
        .alignment(Alignment::Center)
        .modifier(watch(geometry, "figure").size(width, height))
        .children(move || {
            let content = content.clone();
            provide_context(move || MockGeometry(geometry), move || content.render());
            let callouts = callouts.clone();
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    callout_overlay(geometry, false, callouts.clone(), dark, marks, filled);
                });
        });
}

#[tessera]
fn callout_overlay(
    geometry: State<Geometry>,
    expanded: bool,
    callouts: Vec<Callout>,
    dark: bool,
    marks: usize,
    filled: bool,
) {
    callout_content(
        geometry.with(annotations::relative_geometry),
        expanded,
        callouts,
        dark,
        marks,
        filled,
    );
}

/// Solid numbered capsule matching the filled callouts of the M3 guidance
/// diagrams.
#[tessera]
fn filled_marker(number: usize, x: Dp, y: Dp, dark: bool) {
    let (fill, ink) = if dark {
        (srgb(230, 224, 233), srgb(29, 27, 32))
    } else {
        (srgb(29, 27, 32), Color::WHITE)
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Anatomy marker {number}")),
                    merge_descendants: true,
                    ..Default::default()
                })
                .background_with_shape(fill, Shape::rounded_rectangle(Dp(16.0))),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(ink);
        });
}

fn target(values: &Geometry, key: &str, expanded: bool) -> Option<Rect> {
    let derived = |prefix: &str| -> Option<Rect> {
        let index: usize = key.strip_prefix(prefix)?.parse().ok()?;
        relative(values, ICON_KEYS[index.min(5)])
    };
    if let Some(icon) = derived("indicator") {
        return Some(if expanded {
            let container = relative(values, "container")?;
            Rect {
                x: icon.x - 16.0,
                y: icon.y - 16.0,
                w: (container.w - 40.0).min(icon.w + 76.0),
                h: 56.0,
            }
        } else {
            Rect {
                x: icon.x - 16.0,
                y: icon.y - 4.0,
                w: 56.0,
                h: 32.0,
            }
        });
    }
    if let Some(icon) = derived("label") {
        return Some(if expanded {
            Rect {
                x: icon.x + 32.0,
                y: icon.y + 2.0,
                w: 36.0,
                h: 20.0,
            }
        } else {
            Rect {
                x: icon.x - 6.0,
                y: icon.y + 32.0,
                w: 36.0,
                h: 16.0,
            }
        });
    }
    relative(values, key)
}

fn segment(svg: &mut String, x0: f64, y0: f64, x1: f64, y1: f64, color: &str) {
    let (x, w) = if x0 <= x1 {
        (x0, x1 - x0)
    } else {
        (x1, x0 - x1)
    };
    let (y, h) = if y0 <= y1 {
        (y0, y1 - y0)
    } else {
        (y1, y0 - y1)
    };
    svg.push_str(&format!(
        "<rect x='{x}' y='{y}' width='{}' height='{}' fill='{color}'/>",
        w.max(1.0),
        h.max(1.0)
    ));
}

#[tessera]
fn callout_content(
    values: Geometry,
    expanded: bool,
    callouts: Vec<Callout>,
    dark: bool,
    marks: usize,
    filled: bool,
) {
    let (Some(root), Some(container)) = (values.get("figure"), relative(&values, "container"))
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let ink = if dark { "#e6e0e9" } else { "#49454f" };
    let mut svg = String::new();
    let mut markers = Vec::new();
    for trailing in [false, true] {
        let mut lane: Vec<(usize, f64, f64, f64)> = callouts
            .iter()
            .filter(|callout| callout.2 == trailing)
            .filter_map(|&(number, key, _)| {
                let rect = target(&values, key, expanded)?;
                let (x, y) = if key == "container" {
                    (
                        if trailing {
                            rect.x + rect.w - 8.0
                        } else {
                            rect.x + 8.0
                        },
                        rect.y + 16.0,
                    )
                } else if trailing {
                    (rect.x + rect.w, rect.y + rect.h * 0.5)
                } else {
                    (rect.x, rect.y + rect.h * 0.5)
                };
                Some((number, x, y, y))
            })
            .collect();
        lane.sort_by(|a, b| a.2.total_cmp(&b.2));
        let mut previous = f64::NEG_INFINITY;
        for entry in &mut lane {
            entry.3 = entry.2.max(previous + 36.0).max(18.0);
            previous = entry.3;
        }
        let mut next = height.0 - 18.0;
        for entry in lane.iter_mut().rev() {
            entry.3 = entry.3.min(next);
            next = entry.3 - 36.0;
        }
        for (number, x, y, marker_y) in lane {
            let marker_x = if trailing {
                container.x + container.w + 28.0
            } else {
                container.x - 28.0
            };
            let elbow = if trailing {
                container.x + container.w + 8.0
            } else {
                container.x - 8.0
            };
            let edge = if trailing {
                marker_x - 12.0
            } else {
                marker_x + 12.0
            };
            segment(&mut svg, edge, marker_y, elbow, marker_y, ink);
            segment(&mut svg, elbow, marker_y, elbow, y, ink);
            segment(&mut svg, elbow, y, x, y, ink);
            svg.push_str(&format!(
                "<circle cx='{x}' cy='{y}' r='2.5' fill='none' stroke='{ink}' stroke-width='1'/>"
            ));
            markers.push((number, marker_x - 12.0, marker_y - 16.0));
        }
    }
    let mut dimensions = Vec::new();
    match marks {
        1 => measurement_marks(&values, expanded, container, &mut svg, &mut dimensions),
        2 | 3 => baseline_marks(&values, container, marks == 3, &mut svg, &mut dimensions),
        _ => {}
    }
    annotations::lines(width, height, svg);
    for (number, x, y) in markers {
        if filled {
            filled_marker(number, Dp(x), Dp(y), dark);
        } else {
            annotations::marker(number, Dp(x), Dp(y));
        }
    }
    for (label, x, y) in dimensions {
        annotations::dimension(label, Dp(x), Dp(y));
    }
}

fn band(svg: &mut String, x: f64, y: f64, w: f64, h: f64) {
    svg.push_str(&format!(
        "<rect x='{x}' y='{y}' width='{}' height='{}' fill='{MEASURE}' fill-opacity='0.28'/>",
        w.max(1.0),
        h.max(1.0)
    ));
}

fn measurement_marks(
    values: &Geometry,
    expanded: bool,
    container: Rect,
    svg: &mut String,
    dimensions: &mut Vec<(String, f64, f64)>,
) {
    let (Some(first), Some(second)) = (
        relative(values, ICON_KEYS[0]),
        relative(values, ICON_KEYS[1]),
    ) else {
        return;
    };
    let indicator = target(values, "indicator0", expanded).unwrap_or(first);
    let bottom = container.y + container.h;
    let right = container.x + container.w;
    let icon = relative(values, ICON_KEYS[2]).unwrap_or(second);
    band(svg, icon.x, icon.y, icon.w, icon.h);
    annotations::outline(svg, icon);
    segment(
        svg,
        icon.x,
        icon.y + icon.h * 0.5,
        container.x - 8.0,
        icon.y + icon.h * 0.5,
        MEASURE,
    );
    dimensions.push((
        "24".into(),
        container.x - 30.0,
        icon.y + icon.h * 0.5 - 10.0,
    ));
    if expanded {
        segment(
            svg,
            container.x,
            container.y - 12.0,
            right,
            container.y - 12.0,
            MEASURE,
        );
        dimensions.push((
            "220–360".into(),
            container.x + container.w * 0.5 - 26.0,
            container.y - 34.0,
        ));
        band(
            svg,
            container.x,
            indicator.y,
            first.x - container.x,
            bottom - indicator.y,
        );
        dimensions.push(("36".into(), container.x + 2.0, bottom - 24.0));
        band(
            svg,
            indicator.x,
            indicator.y,
            first.x - indicator.x,
            indicator.h,
        );
        dimensions.push(("16".into(), indicator.x, indicator.y + indicator.h + 2.0));
        annotations::outline(svg, indicator);
        let middle = indicator.y + indicator.h * 0.5;
        segment(
            svg,
            indicator.x + indicator.w,
            middle,
            right + 8.0,
            middle,
            MEASURE,
        );
        dimensions.push((
            format!("{}", indicator.h.round()),
            right + 12.0,
            middle - 10.0,
        ));
        return;
    }
    segment(
        svg,
        container.x,
        bottom + 12.0,
        right,
        bottom + 12.0,
        MEASURE,
    );
    dimensions.push((
        format!("{}", container.w.round()),
        container.x + container.w * 0.5 - 14.0,
        bottom + 16.0,
    ));
    band(
        svg,
        container.x,
        indicator.y + indicator.h,
        container.w,
        4.0,
    );
    dimensions.push(("4".into(), right + 8.0, indicator.y + indicator.h - 8.0));
    // Icon-to-label is measured from the icon itself, so it differs from
    // the 4dp indicator-to-label gap drawn above.
    band(svg, container.x, second.y + second.h, container.w, 8.0);
    dimensions.push(("8".into(), right + 8.0, second.y + second.h - 6.0));
    let (Some(label), Some(next)) = (
        target(values, "label1", false),
        target(values, "indicator2", false),
    ) else {
        return;
    };
    let spacing = next.y - (label.y + label.h);
    if spacing > 0.5 {
        band(svg, container.x, label.y + label.h, container.w, spacing);
        dimensions.push((
            format!("{}", spacing.round()),
            right + 8.0,
            label.y + label.h + spacing * 0.5 - 10.0,
        ));
    }
}

fn baseline_marks(
    values: &Geometry,
    container: Rect,
    padding: bool,
    svg: &mut String,
    dimensions: &mut Vec<(String, f64, f64)>,
) {
    let (Some(indicator), Some(label)) = (
        target(values, "indicator0", false),
        target(values, "label0", false),
    ) else {
        return;
    };
    let right = container.x + container.w;
    if padding {
        band(
            svg,
            container.x,
            indicator.y,
            indicator.x - container.x,
            indicator.h,
        );
        band(
            svg,
            indicator.x + indicator.w,
            indicator.y,
            right - indicator.x - indicator.w,
            indicator.h,
        );
        dimensions.push((
            format!("{}", (indicator.x - container.x).round()),
            container.x - 2.0,
            indicator.y - 22.0,
        ));
        dimensions.push((
            format!("{}", (right - indicator.x - indicator.w).round()),
            indicator.x + indicator.w + 2.0,
            indicator.y - 22.0,
        ));
        let gap = label.y - (indicator.y + indicator.h);
        band(
            svg,
            container.x,
            indicator.y + indicator.h,
            container.w,
            gap,
        );
        dimensions.push((
            format!("{}", gap.round()),
            container.x - 24.0,
            indicator.y + indicator.h + gap * 0.5 - 10.0,
        ));
        if let Some(next) = target(values, "indicator1", false) {
            let spacing = next.y - (label.y + label.h);
            band(svg, container.x, label.y + label.h, container.w, spacing);
            dimensions.push((
                format!("{}", spacing.round()),
                right + 8.0,
                label.y + label.h + spacing * 0.5 - 10.0,
            ));
        }
        return;
    }
    if let Some(menu) = relative(values, "menu") {
        let (x, y) = (
            menu.x + (menu.w - 24.0) * 0.5,
            menu.y + (menu.h - 24.0) * 0.5,
        );
        band(svg, x, y, 24.0, 24.0);
        dimensions.push(("24".into(), x - 30.0, y + 2.0));
    }
    segment(
        svg,
        indicator.x,
        indicator.y - 6.0,
        indicator.x + indicator.w,
        indicator.y - 6.0,
        MEASURE,
    );
    dimensions.push((
        format!("{}", indicator.w.round()),
        indicator.x + indicator.w * 0.5 - 10.0,
        indicator.y - 28.0,
    ));
    segment(
        svg,
        indicator.x - 6.0,
        indicator.y,
        indicator.x - 6.0,
        indicator.y + indicator.h,
        MEASURE,
    );
    dimensions.push((
        format!("{}", indicator.h.round()),
        indicator.x - 32.0,
        indicator.y + indicator.h * 0.5 - 10.0,
    ));
    if let Some(item) = relative(values, ITEM_KEYS[1]) {
        band(svg, container.x, item.y, container.w, item.h);
        segment(
            svg,
            right + 6.0,
            item.y,
            right + 6.0,
            item.y + item.h,
            MEASURE,
        );
        dimensions.push((
            format!("{}", item.h.round()),
            right + 12.0,
            item.y + item.h * 0.5 - 10.0,
        ));
    }
    let bottom = container.y + container.h - 32.0;
    segment(svg, container.x, bottom, right, bottom, MEASURE);
    dimensions.push((
        format!("{}", container.w.round()),
        container.x + container.w * 0.5 - 10.0,
        bottom + 4.0,
    ));
}

/// Static navigation rail illustration for guidance figures.
#[tessera]
pub(super) fn mock_rail(
    expanded: Option<bool>,
    count: Option<usize>,
    set: Option<usize>,
    selected: Option<usize>,
    all_active: Option<bool>,
    state: Option<usize>,
    state_all: Option<bool>,
    target_index: Option<usize>,
    focus_ring: Option<bool>,
    target_outline: Option<bool>,
    touch: Option<bool>,
    cursor: Option<bool>,
    menu: Option<bool>,
    fab: Option<bool>,
    fab_bottom: Option<bool>,
    logo: Option<bool>,
    center: Option<bool>,
    divider: Option<bool>,
    outlined_active: Option<bool>,
    fill_indicator: Option<bool>,
    hide_labels: Option<bool>,
    baseline: Option<bool>,
    m2: Option<bool>,
    badges: Option<bool>,
    label_mode: Option<usize>,
    label_scale: Option<f64>,
    width: Option<Dp>,
    elevated: Option<bool>,
    container: Option<Color>,
    badge_start: Option<usize>,
    target_count: Option<usize>,
    active_label_only: Option<bool>,
    fab_tertiary: Option<bool>,
    header_height: Option<Dp>,
    low_contrast: Option<bool>,
    focus_box: Option<bool>,
    focus_badge: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let sink = use_context::<MockGeometry>().map(|context| context.get().0);
    let low_contrast = low_contrast.unwrap_or(false);
    let focus_box = focus_box.unwrap_or(false);
    let focus_badge = focus_badge.unwrap_or(true);
    let badge_start = badge_start.unwrap_or(1);
    let target_count = target_count.unwrap_or(0);
    let active_label_only = active_label_only.unwrap_or(false);
    let fab_tertiary = fab_tertiary.unwrap_or(false);
    let expanded = expanded.unwrap_or(false);
    let count = count.unwrap_or(4);
    let set = set.unwrap_or(1);
    let selected = selected.unwrap_or(usize::MAX);
    let all_active = all_active.unwrap_or(false);
    let state = state.unwrap_or(0);
    let state_all = state_all.unwrap_or(false);
    let target_index = target_index.unwrap_or(usize::MAX);
    let focus_ring = focus_ring.unwrap_or(state == 2 && !focus_box);
    let target_outline = target_outline.unwrap_or(false);
    let touch = touch.unwrap_or(false);
    let cursor = cursor.unwrap_or(false);
    let menu = menu.unwrap_or(false);
    let fab = fab.unwrap_or(false);
    let fab_bottom = fab_bottom.unwrap_or(false);
    let logo = logo.unwrap_or(false);
    let center = center.unwrap_or(false);
    let divider = divider.unwrap_or(false);
    let outlined_active = outlined_active.unwrap_or(false);
    let fill_indicator = fill_indicator.unwrap_or(false);
    let hide_labels = hide_labels.unwrap_or(false);
    let baseline = baseline.unwrap_or(false);
    let m2 = m2.unwrap_or(false);
    let badges = badges.unwrap_or(false);
    let label_mode = label_mode.unwrap_or(0);
    let label_scale = label_scale.unwrap_or(1.0);
    let width = width.unwrap_or(Dp(if expanded {
        220.0
    } else if baseline {
        80.0
    } else {
        96.0
    }));
    let elevated = elevated.unwrap_or(false);
    let container = container.unwrap_or(if baseline {
        scheme.surface_container_low
    } else {
        scheme.surface_container
    });
    let rail = RenderSlot::new(move || {
        column()
            .cross_axis_alignment(if expanded {
                CrossAxisAlignment::Start
            } else {
                CrossAxisAlignment::Center
            })
            .modifier(
                tagged(sink, "container")
                    .width(width)
                    .fill_max_height()
                    .background(container)
                    .padding(Padding::new(
                        if expanded { Dp(20.0) } else { Dp::ZERO },
                        Dp(if baseline { 44.0 } else { 36.0 }),
                        if expanded { Dp(20.0) } else { Dp::ZERO },
                        Dp(16.0),
                    )),
            )
            .children(move || {
                let has_header = logo || menu || (fab && !fab_bottom);
                if let Some(height) = header_height {
                    // A fixed header slot keeps the first destination at the
                    // same height across side-by-side
                    // configurations.
                    boxed()
                        .alignment(Alignment::TopCenter)
                        .modifier(Modifier::new().width(width).height(height))
                        .children(move || {
                            if has_header {
                                mock_header(
                                    expanded,
                                    logo,
                                    menu,
                                    fab && !fab_bottom,
                                    set,
                                    fab_tertiary,
                                );
                            }
                        });
                } else if has_header {
                    mock_header(expanded, logo, menu, fab && !fab_bottom, set, fab_tertiary);
                    gap(Dp(if baseline { 24.0 } else { 40.0 }));
                }
                if center {
                    spacer().modifier(Modifier::new().weight(1.0));
                }
                for index in 0..count {
                    if index > 0 && !expanded && !baseline {
                        gap(Dp(4.0));
                    }
                    let pointed = index == target_index || index < target_count;
                    let targeted = state_all || pointed;
                    let active = all_active || selected == index;
                    mock_item(
                        set,
                        index,
                        expanded,
                        active,
                        if targeted { state } else { 0 },
                        targeted && focus_ring,
                        targeted && focus_box,
                        targeted && focus_box && focus_badge,
                        low_contrast && index == 3,
                        index == target_index && target_outline,
                        index == target_index && touch,
                        pointed && cursor,
                        outlined_active,
                        fill_indicator,
                        hide_labels || (active_label_only && !active),
                        m2,
                        badges,
                        label_mode,
                        label_scale,
                        baseline,
                        badge_start,
                    );
                }
                if center || fab_bottom {
                    spacer().modifier(Modifier::new().weight(1.0));
                }
                if fab && fab_bottom {
                    mock_header(expanded, false, false, true, set, fab_tertiary);
                }
            });
    });
    row()
        .modifier(Modifier::new().fill_max_height())
        .children(move || {
            if elevated {
                let rail = rail.clone();
                surface()
                    .style(container.into())
                    .elevation(Dp(1.0))
                    .modifier(Modifier::new().fill_max_height())
                    .child(move || rail.render());
            } else {
                rail.render();
            }
            if divider {
                boxed().modifier(
                    Modifier::new()
                        .width(Dp(1.0))
                        .fill_max_height()
                        .background(scheme.outline_variant),
                );
            }
        });
}

#[tessera]
fn mock_header(expanded: bool, logo: bool, menu: bool, fab: bool, set: usize, tertiary: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let sink = use_context::<MockGeometry>().map(|context| context.get().0);
    column()
        .cross_axis_alignment(if expanded {
            CrossAxisAlignment::Start
        } else {
            CrossAxisAlignment::Center
        })
        .children(move || {
            if logo {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(40.0), Dp(40.0))
                            .background_with_shape(scheme.tertiary, Shape::Ellipse),
                    )
                    .children(move || {
                        text()
                            .content("M")
                            .style(type_style(16.0, 24.0))
                            .color(scheme.on_tertiary);
                    });
                gap(Dp(16.0));
            }
            if menu {
                boxed()
                    .alignment(if expanded {
                        Alignment::CenterStart
                    } else {
                        Alignment::Center
                    })
                    .modifier(tagged(sink, "menu").size(Dp(56.0), Dp(48.0)))
                    .children(move || {
                        icon()
                            .painter(if expanded {
                                filled::MENU_OPEN_SVG
                            } else {
                                filled::MENU_SVG
                            })
                            .size(Dp(24.0))
                            .tint(scheme.on_surface_variant);
                    });
            }
            if menu && fab {
                gap(Dp(4.0));
            }
            if fab {
                mock_fab(expanded, set, tertiary);
            }
        });
}

#[tessera]
fn mock_fab(expanded: bool, set: usize, tertiary: bool) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let (fill, ink) = if tertiary {
        (scheme.tertiary_container, scheme.on_tertiary_container)
    } else {
        (scheme.primary_container, scheme.on_primary_container)
    };
    let style = theme.typography.title_medium;
    let sink = use_context::<MockGeometry>().map(|context| context.get().0);
    let (symbol, word) = if set == 1 {
        (filled::HOURGLASS_EMPTY_SVG, "Add timer")
    } else {
        (filled::EDIT_SVG, "Label")
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            tagged(sink, "fab")
                .height(Dp(56.0))
                .background_with_shape(fill, Shape::rounded_rectangle(Dp(16.0)))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            icon().painter(symbol).size(Dp(24.0)).tint(ink);
            if expanded {
                spacer().modifier(Modifier::new().width(Dp(12.0)));
                text().content(word).style(style.clone()).color(ink);
            }
        });
}

fn mock_label(set: usize, index: usize, mode: usize) -> String {
    match (mode, index) {
        (1, 3) => "Stop-\nwatch".into(),
        (1, 1) => "World\nclock".into(),
        (2, 1) => "World time…".into(),
        (3, 1) => "World time zones".into(),
        (4, 1) => "World time\nzones".into(),
        (5, 3) => "Stop-\nwatch".into(),
        _ => label_for(set, index),
    }
}

#[tessera]
fn mock_item(
    set: usize,
    index: usize,
    expanded: bool,
    active: bool,
    state: usize,
    focus_ring: bool,
    focus_box: bool,
    focus_badge: bool,
    faded: bool,
    target_outline: bool,
    touch: bool,
    cursor: bool,
    outlined_active: bool,
    fill_indicator: bool,
    hide_labels: bool,
    m2: bool,
    badges: bool,
    label_mode: usize,
    label_scale: f64,
    baseline: bool,
    badge_start: usize,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let sink = use_context::<MockGeometry>().map(|context| context.get().0);
    let layer = if active {
        scheme.on_secondary_container
    } else {
        scheme.on_surface
    };
    let layer_alpha = match state {
        1 => 0.08,
        2 | 3 => 0.10,
        _ => 0.0,
    };
    let indicator = if active && !m2 {
        scheme.secondary_container
    } else {
        Color::TRANSPARENT
    };
    let icon_color = if m2 {
        if active {
            scheme.primary
        } else {
            scheme.on_surface_variant
        }
    } else if active {
        scheme.on_secondary_container
    } else if faded {
        scheme.inverse_primary
    } else {
        scheme.on_surface_variant
    };
    let label_color = if faded {
        scheme.inverse_primary
    } else if m2 && active {
        scheme.primary
    } else if active && !expanded {
        scheme.secondary
    } else if active {
        scheme.on_secondary_container
    } else {
        scheme.on_surface_variant
    };
    let mut label_style = if expanded {
        theme.typography.label_large
    } else {
        theme.typography.label_medium
    };
    let scale = if label_mode == 3 { 0.7 } else { label_scale };
    label_style.font_size = Dp(label_style.font_size.0 * scale);
    label_style.line_height = label_style.line_height.map(|height| Dp(height.0 * scale));
    let label = mock_label(set, index, label_mode);
    let show_label = !hide_labels && (!m2 || active);
    // Baseline rails without labels use a 56dp circular indicator; with
    // selected-only labels, unlabeled destinations keep the 56dp slot.
    let compact = baseline && !show_label;
    let round = compact && active;
    let glyph = glyph(set, index, active && !outlined_active);
    let pill = move |modifier: Modifier| {
        let mut modifier = modifier.background_with_shape(indicator, Shape::CAPSULE);
        if layer_alpha > 0.0 {
            modifier =
                modifier.background_with_shape(layer.with_alpha(layer_alpha), Shape::CAPSULE);
        }
        if focus_ring {
            modifier = modifier.border_with_shape(Dp(3.0), scheme.secondary, Shape::CAPSULE);
        }
        modifier
    };
    let marks = RenderSlot::new(move || {
        if touch {
            let (x, y) = if expanded {
                (Dp(136.0), Dp(8.0))
            } else {
                (Dp(24.0), Dp(-8.0))
            };
            let (r, g, b) = INTERACTION_FILL;
            let (dr, dg, db) = INTERACTION_DARK;
            boxed().modifier(
                Modifier::new()
                    .offset(x, y)
                    .size(Dp(40.0), Dp(40.0))
                    .background_with_shape(srgb(r, g, b), Shape::Ellipse)
                    .border_with_shape(Dp(3.0), srgb(dr, dg, db), Shape::Ellipse),
            );
        }
        if cursor {
            boxed()
                .modifier(Modifier::new().offset(Dp(34.0), Dp(10.0)))
                .children(move || {
                    icon()
                        .painter(filled::PAN_TOOL_ALT_SVG)
                        .size(Dp(20.0))
                        .tint(scheme.on_surface);
                });
        }
    });
    let badge_slot = RenderSlot::new(move || {
        if badges && index == badge_start {
            boxed()
                .modifier(Modifier::new().offset(Dp(14.0), Dp(-4.0)))
                .children(move || {
                    boxed()
                        .modifier(tagged(sink, "large-badge"))
                        .children(move || {
                            badge_with_content().content(move || {
                                text().content("3").modifier(tagged(sink, "badge-label"));
                            });
                        });
                });
        } else if badges && index == badge_start + 1 {
            boxed()
                .modifier(Modifier::new().offset(Dp(18.0), Dp::ZERO))
                .children(move || {
                    boxed().modifier(tagged(sink, "small-badge")).children(|| {
                        badge();
                    });
                });
        }
    });
    if expanded {
        boxed()
            .alignment(Alignment::CenterStart)
            .modifier(if target_outline {
                Modifier::new().width(Dp(180.0)).height(Dp(56.0))
            } else {
                Modifier::new().height(Dp(56.0))
            })
            .children(move || {
                if target_outline {
                    let mut svg = String::new();
                    let ink = "#49454f";
                    annotations::dotted(&mut svg, 0.0, 0.0, 180.0, false, ink);
                    annotations::dotted(&mut svg, 0.0, 55.3, 180.0, false, ink);
                    annotations::dotted(&mut svg, 0.0, 0.0, 56.0, true, ink);
                    annotations::dotted(&mut svg, 179.3, 0.0, 56.0, true, ink);
                    annotations::lines(Dp(180.0), Dp(56.0), svg);
                }
                let label = label.clone();
                let label_style = label_style.clone();
                let badge_slot = badge_slot.clone();
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        pill(if fill_indicator {
                            Modifier::new().width(Dp(180.0)).height(Dp(56.0))
                        } else {
                            Modifier::new().height(Dp(56.0))
                        })
                        .padding_symmetric(Dp(16.0), Dp::ZERO),
                    )
                    .children(move || {
                        let badge_slot = badge_slot.clone();
                        boxed().children(move || {
                            icon().painter(glyph).size(Dp(24.0)).tint(icon_color);
                            badge_slot.render();
                        });
                        if show_label {
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            text()
                                .content(label.clone())
                                .style(label_style.clone())
                                .color(label_color);
                        }
                    });
                marks.render();
            });
    } else {
        let (r, g, b) = INTERACTION;
        let focus_color = srgb(r, g, b);
        let item = RenderSlot::new(move || {
            let badge_slot = badge_slot.clone();
            let marks = marks.clone();
            let label = label.clone();
            let label_style = label_style.clone();
            let base = tagged(sink, ITEM_KEYS[index.min(5)]);
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(if compact {
                    base.width(Dp(80.0)).height(Dp(56.0))
                } else if focus_box {
                    // Accessibility figures frame the whole focused item
                    // (indicator and label) instead of drawing a ring.
                    base.width(Dp(64.0))
                        .border_with_shape(Dp(3.0), focus_color, Shape::rounded_rectangle(Dp(4.0)))
                        .padding_symmetric(Dp::ZERO, Dp(6.0))
                } else {
                    // Scaled labels may use the full 96dp rail width before
                    // wrapping, as in the text-scaling reference.
                    base.width(Dp(if label_scale > 1.0 { 96.0 } else { 80.0 }))
                        .padding_symmetric(Dp::ZERO, Dp(if baseline { 2.0 } else { 4.0 }))
                })
                .main_axis_alignment(MainAxisAlignment::Center)
                .children(move || {
                    let badge_slot = badge_slot.clone();
                    let marks = marks.clone();
                    boxed().children(move || {
                        let badge_slot = badge_slot.clone();
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(pill(
                                Modifier::new().size(Dp(56.0), Dp(if round { 56.0 } else { 32.0 })),
                            ))
                            .children(move || {
                                boxed()
                                    .modifier(tagged(sink, ICON_KEYS[index.min(5)]))
                                    .children(move || {
                                        icon().painter(glyph).size(Dp(24.0)).tint(icon_color);
                                    });
                                badge_slot.render();
                            });
                        marks.render();
                    });
                    if show_label {
                        gap(Dp(4.0));
                        text()
                            .content(label.clone())
                            .style(label_style.clone())
                            .color(label_color);
                    }
                });
        });
        if focus_box {
            boxed().alignment(Alignment::TopStart).children(move || {
                item.render();
                if !focus_badge {
                    return;
                }
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(-10.0), Dp(-10.0))
                            .size(Dp(20.0), Dp(20.0))
                            .background_with_shape(srgb(29, 27, 32), Shape::Ellipse),
                    )
                    .children(|| {
                        text()
                            .content("1")
                            .style(type_style(11.0, 16.0))
                            .color(Color::WHITE);
                    });
            });
        } else {
            item.render();
        }
    }
}

/// Device-like window combining a rail illustration with simple app content.
#[tessera]
pub(super) fn screen(
    #[prop(into)] label: String,
    width: Dp,
    height: Dp,
    rail: Option<RenderSlot>,
    overlay: Option<RenderSlot>,
    trailing: Option<bool>,
    content_kind: Option<usize>,
    bar: Option<usize>,
    hidden_menu: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let trailing = trailing.unwrap_or(false);
    let content_kind = content_kind.unwrap_or(0);
    let bar = bar.unwrap_or(0);
    let hidden_menu = hidden_menu.unwrap_or(false);
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .size(width, height)
                .clip_shape(Shape::rounded_rectangle(Dp(20.0)))
                .background(srgb(120, 117, 128))
                .padding_all(Dp(4.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(label),
                    ..Default::default()
                }),
        )
        .children(move || {
            let rail = rail.clone();
            let overlay = overlay.clone();
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                        .background(scheme.surface),
                )
                .children(move || {
                    let rail = rail.clone();
                    column()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            let rail = rail.clone();
                            if bar == 2 {
                                mock_bar();
                            }
                            row()
                                .modifier(Modifier::new().fill_max_width().weight(1.0))
                                .children(move || {
                                    if !trailing && let Some(rail) = rail.as_ref() {
                                        rail.render();
                                    }
                                    boxed()
                                        .modifier(Modifier::new().weight(1.0).fill_max_height())
                                        .children(move || {
                                            app_content(content_kind);
                                        });
                                    if trailing && let Some(rail) = rail.as_ref() {
                                        rail.render();
                                    }
                                });
                            if bar == 1 {
                                mock_bar();
                            }
                        });
                    if hidden_menu {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(8.0), Dp(12.0))
                                    .size(Dp(48.0), Dp(48.0)),
                            )
                            .children(move || {
                                icon()
                                    .painter(filled::MENU_SVG)
                                    .size(Dp(24.0))
                                    .tint(scheme.on_surface_variant);
                            });
                    }
                    if let Some(overlay) = overlay.as_ref() {
                        overlay.render();
                    }
                });
        });
}

/// Scrim and expanded rail drawn over a screen.
#[tessera]
pub(super) fn modal_overlay(
    offset: Option<Dp>,
    count: Option<usize>,
    set: Option<usize>,
    selected: Option<usize>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let offset = offset.unwrap_or(Dp::ZERO);
    let count = count.unwrap_or(4);
    let set = set.unwrap_or(1);
    let selected = selected.unwrap_or(1);
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .fill_max_size()
                .background(scheme.scrim.with_alpha(0.32)),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(offset, Dp::ZERO)
                        .fill_max_height()
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                )
                .children(move || {
                    mock_rail()
                        .expanded(true)
                        .set(set)
                        .count(count)
                        .selected(selected)
                        .menu(true)
                        .fab(true)
                        .container(scheme.surface_container_low);
                });
        });
}

#[tessera]
fn mock_bar() {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let style = theme.typography.label_medium;
    row()
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(72.0))
                .background(scheme.surface_container),
        )
        .children(move || {
            for index in 0..4 {
                let style = style.clone();
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().padding_symmetric(Dp::ZERO, Dp(8.0)))
                    .children(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .size(Dp(56.0), Dp(32.0))
                                    .background_with_shape(
                                        if index == 1 {
                                            scheme.secondary_container
                                        } else {
                                            Color::TRANSPARENT
                                        },
                                        Shape::CAPSULE,
                                    ),
                            )
                            .children(move || {
                                icon()
                                    .painter(glyph(1, index, index == 1))
                                    .size(Dp(24.0))
                                    .tint(if index == 1 {
                                        scheme.on_secondary_container
                                    } else {
                                        scheme.on_surface_variant
                                    });
                            });
                        gap(Dp(4.0));
                        text()
                            .content(label_for(1, index))
                            .style(style.clone())
                            .color(scheme.on_surface_variant);
                    });
            }
        });
}

#[tessera]
fn app_content(kind: usize) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    match kind {
        1 => {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0)))
                .children(move || {
                    for index in 0..6 {
                        if index > 0 {
                            gap(Dp(8.0));
                        }
                        boxed().modifier(
                            Modifier::new()
                                .fill_max_width()
                                .height(Dp(48.0))
                                .background_with_shape(
                                    scheme.surface_container_high,
                                    Shape::rounded_rectangle(Dp(12.0)),
                                ),
                        );
                    }
                });
        }
        2 => {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(12.0)))
                .children(move || {
                    for row_index in 0..3 {
                        if row_index > 0 {
                            gap(Dp(8.0));
                        }
                        row()
                            .modifier(Modifier::new().fill_max_width().weight(1.0))
                            .children(move || {
                                for column_index in 0..3 {
                                    if column_index > 0 {
                                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    }
                                    boxed().modifier(
                                        Modifier::new()
                                            .weight(1.0)
                                            .fill_max_height()
                                            .background_with_shape(
                                                match (row_index + column_index) % 3 {
                                                    0 => scheme.tertiary_container,
                                                    1 => scheme.primary_container,
                                                    _ => scheme.secondary_container,
                                                },
                                                Shape::rounded_rectangle(Dp(12.0)),
                                            ),
                                    );
                                }
                            });
                    }
                });
        }
        _ => {
            let mut style = theme.typography.display_large;
            style.font_size = Dp(56.0);
            style.line_height = Some(Dp(64.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background(scheme.primary_container),
                )
                .children(move || {
                    text()
                        .content("8:30")
                        .style(style.clone())
                        .color(scheme.on_primary_container);
                });
        }
    }
}

/// Tinted guidance scene holding one or more screens.
#[tessera]
pub(super) fn scene(#[prop(into)] label: String, height: Dp, content: RenderSlot) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(height)
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .background(srgb(232, 222, 248))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(label),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .main_axis_alignment(MainAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children_shared(content.clone());
        });
}

/// Guidance card with a scene, an optional Do/Caution/Don't strip and a
/// caption.
#[tessera]
pub(super) fn example(
    #[prop(into)] label: String,
    tone: usize,
    #[prop(into)] note: String,
    content: RenderSlot,
    height: Option<Dp>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let height = height.unwrap_or(Dp(300.0));
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(height)
                        .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                        .background(srgb(232, 222, 248))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(label.clone()),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children_shared(content.clone());
            if tone > 0 {
                let (background, ink, symbol, word) = match tone {
                    1 => (
                        srgb(196, 238, 214),
                        srgb(0, 83, 46),
                        filled::CHECK_CIRCLE_SVG,
                        "Do",
                    ),
                    2 => (
                        srgb(255, 235, 156),
                        srgb(91, 67, 0),
                        filled::WARNING_SVG,
                        "Caution",
                    ),
                    _ => (
                        srgb(255, 218, 214),
                        srgb(147, 0, 10),
                        filled::CANCEL_SVG,
                        "Don't",
                    ),
                };
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .background(background)
                            .padding_symmetric(Dp(16.0), Dp(8.0)),
                    )
                    .children(move || {
                        icon().painter(symbol).size(Dp(20.0)).tint(ink);
                        text()
                            .content(word)
                            .style(type_style(16.0, 24.0))
                            .color(ink)
                            .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
                    });
            }
            gap(Dp(12.0));
            text()
                .content(note.clone())
                .style(type_style(14.0, 20.0))
                .color(scheme.on_surface_variant);
        });
}

/// Two guidance cards in equal columns, stacking them on narrow windows.
#[tessera]
pub(super) fn pair(first: RenderSlot, second: RenderSlot) {
    let width = remember(|| Dp(760.0));
    let wide = width.get().0 >= 640.0;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            let first = first.clone();
            let second = second.clone();
            if wide {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        let first = first.clone();
                        let second = second.clone();
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || first.render());
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || second.render());
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        first.render();
                        gap(Dp(24.0));
                        second.render();
                    });
            }
        });
}

/// Keyboard key chip shown above focus illustrations.
#[tessera]
pub(super) fn key_chip(#[prop(into)] key: String) {
    let (r, g, b) = INTERACTION;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(32.0))
                .background_with_shape(srgb(r, g, b), Shape::rounded_rectangle(Dp(4.0)))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            text()
                .content(key.clone())
                .style(type_style(14.0, 20.0))
                .color(Color::WHITE);
        });
}

/// Accessibility label call-out beside a rail destination.
#[tessera]
pub(super) fn label_callout() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (r, g, b) = INTERACTION;
    let accent = srgb(r, g, b);
    // The scene centers its children; lift the call-out so the connector
    // meets the Recent destination, the second item of the rail.
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().offset(Dp::ZERO, Dp(-60.0)))
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(200.0))
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(8.0)))
                        .border_with_shape(Dp(3.0), accent, Shape::rounded_rectangle(Dp(8.0)))
                        .padding_symmetric(Dp(16.0), Dp(12.0)),
                )
                .children(move || {
                    for (index, (key, value)) in [("Label", "Recent images"), ("Role", "Tab")]
                        .into_iter()
                        .enumerate()
                    {
                        if index > 0 {
                            gap(Dp(8.0));
                        }
                        text()
                            .content(key)
                            .style(type_style(14.0, 20.0))
                            .color(scheme.on_surface_variant);
                        let mut style = type_style(16.0, 24.0);
                        style.font_weight = 600;
                        text().content(value).style(style).color(accent);
                    }
                });
            boxed().modifier(
                Modifier::new()
                    .width(Dp(32.0))
                    .height(Dp(3.0))
                    .background(accent),
            );
        });
}
