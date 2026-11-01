//! Annotated side sheet figures for the specification article.
//!
//! ## Usage
//!
//! Illustrate side sheet anatomy, color roles, measurements and guidance.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::{ButtonSize, button},
    column::column,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use super::super::{
    INK,
    annotations::{self, SpaceGeometry, watch_in_space},
    gap, srgb, type_style,
};

/// Seed shared by every figure so the illustrations keep the official palette
/// independently of the surrounding article theme.
fn figure_seed() -> Color {
    srgb(103, 80, 164)
}

fn at(x: f64, y: f64, width: f64, height: f64) -> Modifier {
    Modifier::new()
        .offset(Dp(x), Dp(y))
        .required_size(Dp(width.max(0.0)), Dp(height.max(0.0)))
}

fn hex(color: Color) -> String {
    let encode = |linear: f32| {
        let value = if linear <= 0.003_130_8 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        encode(color.r),
        encode(color.g),
        encode(color.b)
    )
}

/// One numbered callout: marker position and the point its leader targets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Callout {
    pub number: usize,
    pub marker: (f64, f64),
    pub target: (f64, f64),
}

const fn callout(number: usize, marker: (f64, f64), target: (f64, f64)) -> Callout {
    Callout {
        number,
        marker,
        target,
    }
}

/// Content shown inside a mock side sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Body {
    #[default]
    Empty,
    Filters,
    Info,
    Settings,
    Scroll,
    Chips,
    Lines,
}

/// Primary content shown beside or beneath a mock side sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum Backdrop {
    #[default]
    Plain,
    Lines,
    Gallery,
    Files,
}

/// Static description of one mock device and its side sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Mock {
    pub modal: bool,
    pub back: bool,
    pub close: bool,
    pub actions: bool,
    pub divider: bool,
    pub start: bool,
    pub focus: bool,
    pub title: &'static str,
    pub body: Body,
    pub backdrop: Backdrop,
    pub sheet_width: f64,
    pub inset: f64,
}

impl Mock {
    pub(super) const fn standard(sheet_width: f64) -> Self {
        Self {
            modal: false,
            back: false,
            close: true,
            actions: false,
            divider: true,
            start: false,
            focus: false,
            title: "Headline",
            body: Body::Empty,
            backdrop: Backdrop::Plain,
            sheet_width,
            inset: 0.0,
        }
    }

    pub(super) const fn modal(sheet_width: f64) -> Self {
        Self {
            modal: true,
            back: true,
            close: true,
            actions: true,
            divider: true,
            start: false,
            focus: false,
            title: "Headline",
            body: Body::Empty,
            backdrop: Backdrop::Plain,
            sheet_width,
            inset: 0.0,
        }
    }

    pub(super) const fn body(mut self, body: Body, title: &'static str) -> Self {
        self.body = body;
        self.title = title;
        self
    }

    pub(super) const fn backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = backdrop;
        self
    }
}

/// Header and actions geometry shared by figures and their callouts.
pub(super) const HEADER_HEIGHT: f64 = 72.0;
pub(super) const ACTIONS_HEIGHT: f64 = 72.0;
const DEVICE_BORDER: f64 = 8.0;
const DEVICE_RADIUS: f64 = 24.0;

/// Stage-space frame of the sheet in a device placed at `x`, `y` with size
/// `width` × `height`.
pub(super) fn sheet_frame(
    mock: Mock,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> (f64, f64, f64, f64) {
    let inner_w = width - DEVICE_BORDER * 2.0;
    let inner_h = height - DEVICE_BORDER * 2.0;
    let sheet_w = mock.sheet_width.min(inner_w - mock.inset * 2.0);
    let sheet_h = inner_h - mock.inset * 2.0;
    let sheet_x = if mock.start {
        mock.inset
    } else {
        inner_w - sheet_w - mock.inset
    };
    (
        x + DEVICE_BORDER + sheet_x,
        y + DEVICE_BORDER + mock.inset,
        sheet_w,
        sheet_h,
    )
}

#[tessera]
pub(super) fn stage(
    width: f64,
    height: f64,
    #[prop(into)] label: String,
    dark: Option<bool>,
    dotted: Option<bool>,
    child: Option<RenderSlot>,
) {
    let dark = dark.unwrap_or(false);
    let dotted = dotted.unwrap_or(false);
    let child = child.unwrap_or_else(RenderSlot::empty);
    material_theme()
        .theme(move || MaterialTheme::from_seed(figure_seed(), dark))
        .child(move || {
            stage_canvas(width, height, label.clone(), dotted).child_shared(child);
        });
}

#[tessera]
fn stage_canvas(width: f64, height: f64, label: String, dotted: bool, child: Option<RenderSlot>) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let background = if dotted {
        scheme.surface
    } else {
        scheme.secondary_container
    };
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.is_alive() && next.0 > 0.0 && measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            let label = label.clone();
            let scale = (measured.get().0 / width).min(1.0) as f32;
            scaled_box(scale).child(move || {
                let label = label.clone();
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(background, Shape::rounded_rectangle(Dp(24.0)))
                            .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Image),
                                label: Some(label),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        if dotted {
                            dots(width, height, scheme.outline_variant);
                        }
                        child.render();
                    });
            });
        });
}

#[tessera]
fn dots(width: f64, height: f64, color: Color) {
    let mut svg = String::new();
    let fill = hex(color);
    let mut y = 10.0;
    while y < height {
        let mut x = 10.0;
        while x < width {
            svg.push_str(&format!(
                "<rect x='{x}' y='{y}' width='1.5' height='1.5' fill='{fill}'/>"
            ));
            x += 20.0;
        }
        y += 20.0;
    }
    annotations::lines(Dp(width), Dp(height), svg);
}

#[tessera]
pub(super) fn marker(number: usize, x: f64, y: f64, tag: Option<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .required_size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.inverse_surface, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!("Side sheet marker {number}")),
                    test_tag: tag.clone(),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.inverse_on_surface);
        });
}

/// Draw numbered markers with leader lines on a stage of the given size.
///
/// When `figure` is set, each marker gets the layout tag
/// `side-marker-{figure}-{number}`.
#[tessera]
pub(super) fn callouts(
    width: f64,
    height: f64,
    items: Vec<Callout>,
    #[prop(into)] figure: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let stroke = hex(scheme.on_surface_variant);
    let mut svg = String::new();
    for item in &items {
        let (mx, my) = (item.marker.0 + 12.0, item.marker.1 + 16.0);
        let (tx, ty) = item.target;
        svg.push_str(&format!(
            "<path d='M{mx},{my} L{tx},{ty}' stroke='{stroke}' stroke-width='1' fill='none'/><circle cx='{tx}' cy='{ty}' r='3' fill='none' stroke='{stroke}' stroke-width='1'/>"
        ));
    }
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(at(0.0, 0.0, width, height))
        .children(move || {
            annotations::lines(Dp(width), Dp(height), svg.clone());
            for item in items.clone() {
                let tag = figure
                    .as_ref()
                    .map(|figure| format!("side-marker-{figure}-{}", item.number));
                marker(item.number, item.marker.0, item.marker.1).tag_optional(tag);
            }
        });
}

#[tessera]
pub(super) fn legend(entries: Vec<(usize, String)>) {
    column().children(move || {
        for (number, label) in entries.clone() {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    material_theme()
                        .theme(|| MaterialTheme::from_seed(figure_seed(), false))
                        .child(move || {
                            boxed()
                                .alignment(Alignment::TopStart)
                                .modifier(Modifier::new().required_size(Dp(24.0), Dp(32.0)))
                                .children(move || {
                                    marker(number, 0.0, 0.0);
                                });
                        });
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                    text()
                        .content(label.clone())
                        .style(type_style(16.0, 24.0))
                        .color(INK);
                });
            gap(Dp(12.0));
        }
    });
}

/// Lay out two slots side by side on wide articles and stacked on narrow ones.
#[tessera]
pub(super) fn split(start: Option<RenderSlot>, end: Option<RenderSlot>, start_weight: Option<f32>) {
    let start = start.unwrap_or_else(RenderSlot::empty);
    let end = end.unwrap_or_else(RenderSlot::empty);
    let start_weight = start_weight.unwrap_or(1.0);
    let wide = remember(|| true);
    column()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width).0 >= 720.0;
                if wide.is_alive() && wide.get() != next {
                    wide.set(next);
                }
            },
        ))
        .children(move || {
            if wide.get() {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().weight(start_weight))
                            .children(move || start.render());
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || end.render());
                    });
            } else {
                start.render();
                gap(Dp(24.0));
                end.render();
            }
        });
}

/// Do or Don't card with a figure, a colored status bar and a note.
#[tessera]
pub(super) fn guidance_card(good: bool, #[prop(into)] note: String, figure: Option<RenderSlot>) {
    let figure = figure.unwrap_or_else(RenderSlot::empty);
    let color = if good {
        srgb(56, 106, 32)
    } else {
        srgb(179, 38, 30)
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            figure.render();
            boxed().modifier(
                Modifier::new()
                    .fill_max_width()
                    .height(Dp(8.0))
                    .background(color),
            );
            gap(Dp(12.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    icon()
                        .vector(if good {
                            filled::CHECK_SVG
                        } else {
                            filled::CLOSE_SVG
                        })
                        .size(Dp(24.0))
                        .tint(color);
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    text()
                        .content(if good { "Do" } else { "Don't" })
                        .style(type_style(16.0, 24.0))
                        .color(color);
                });
            gap(Dp(8.0));
            text()
                .content(note.clone())
                .style(type_style(14.0, 20.0))
                .color(INK);
        });
}

/// A device frame containing primary content and one side sheet.
#[tessera]
pub(super) fn device(mock: Mock, x: f64, y: f64, width: f64, height: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let inner_w = width - DEVICE_BORDER * 2.0;
    let inner_h = height - DEVICE_BORDER * 2.0;
    let (sheet_x, sheet_y, sheet_w, sheet_h) = sheet_frame(mock, 0.0, 0.0, width, height);
    let (sheet_x, sheet_y) = (sheet_x - DEVICE_BORDER, sheet_y - DEVICE_BORDER);
    let content_w = if mock.modal {
        inner_w
    } else {
        inner_w - sheet_w
    };
    let content_x = if !mock.modal && mock.start {
        sheet_w
    } else {
        0.0
    };
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(at(x, y, width, height))
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    at(DEVICE_BORDER, DEVICE_BORDER, inner_w, inner_h)
                        .background(scheme.surface)
                        .clip_shape(Shape::rounded_rectangle(Dp(DEVICE_RADIUS - DEVICE_BORDER))),
                )
                .children(move || {
                    backdrop(mock.backdrop, mock.start, content_x, content_w, inner_h);
                    if mock.modal {
                        boxed().modifier(
                            at(0.0, 0.0, inner_w, inner_h)
                                .background(scheme.scrim.with_alpha(0.32)),
                        );
                    }
                    sheet_panel(mock, sheet_x, sheet_y, sheet_w, sheet_h);
                });
            boxed().modifier(at(0.0, 0.0, width, height).border_with_shape(
                Dp(DEVICE_BORDER),
                scheme.outline_variant,
                Shape::rounded_rectangle(Dp(DEVICE_RADIUS)),
            ));
        });
}

#[tessera]
fn backdrop(kind: Backdrop, start: bool, x: f64, width: f64, height: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(at(x, 0.0, width, height).clip_to_bounds())
        .children(move || match kind {
            Backdrop::Gallery => {
                let hero_h = (height - 136.0).max(80.0);
                boxed().modifier(at(24.0, 24.0, width - 48.0, hero_h).background_with_shape(
                    scheme.tertiary_container,
                    Shape::rounded_rectangle(Dp(16.0)),
                ));
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(at(24.0, 24.0, width - 48.0, hero_h))
                    .children(move || {
                        icon()
                            .vector(filled::LANDSCAPE_SVG)
                            .size(Dp(64.0))
                            .tint(scheme.on_tertiary_container);
                    });
                let tile = ((width - 48.0 - 36.0) / 4.0).max(0.0);
                for index in 0..4 {
                    boxed().modifier(
                        at(
                            24.0 + index as f64 * (tile + 12.0),
                            hero_h + 40.0,
                            tile,
                            72.0,
                        )
                        .background_with_shape(
                            scheme.secondary_container,
                            Shape::rounded_rectangle(Dp(12.0)),
                        ),
                    );
                }
            }
            Backdrop::Files => {
                boxed().modifier(
                    at(24.0, 20.0, width - 48.0, 48.0)
                        .background_with_shape(scheme.surface_container_high, Shape::CAPSULE),
                );
                for index in 0..8 {
                    let y = 92.0 + index as f64 * 56.0;
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(at(24.0, y, 40.0, 40.0).background_with_shape(
                            scheme.primary_container,
                            Shape::rounded_rectangle(Dp(8.0)),
                        ))
                        .children(move || {
                            icon()
                                .vector(filled::DESCRIPTION_SVG)
                                .size(Dp(20.0))
                                .tint(scheme.on_primary_container);
                        });
                    boxed().modifier(
                        at(80.0, y + 6.0, (width - 160.0).min(220.0), 12.0).background_with_shape(
                            scheme.surface_container_highest,
                            Shape::CAPSULE,
                        ),
                    );
                    boxed().modifier(
                        at(80.0, y + 24.0, (width - 200.0).min(140.0), 10.0)
                            .background_with_shape(scheme.surface_container_high, Shape::CAPSULE),
                    );
                }
            }
            Backdrop::Plain => {}
            Backdrop::Lines => {
                boxed().modifier(at(0.0, 0.0, width, 64.0).background(scheme.surface_container));
                for (index, fraction) in [0.72, 0.9, 0.56, 0.84, 0.64, 0.78].into_iter().enumerate()
                {
                    let bar = (width - 48.0) * fraction;
                    let bar_x = if start { width - 24.0 - bar } else { 24.0 };
                    boxed().modifier(
                        at(bar_x, 96.0 + index as f64 * 32.0, bar, 12.0).background_with_shape(
                            scheme.surface_container_highest,
                            Shape::CAPSULE,
                        ),
                    );
                }
            }
        });
}

fn sheet_shape(mock: Mock) -> Shape {
    let round = RoundedCorner::manual(Dp(16.0), 3.0);
    if mock.inset > 0.0 {
        Shape::rounded_rectangle(Dp(16.0))
    } else if mock.modal {
        if mock.start {
            Shape::RoundedRectangle {
                top_left: RoundedCorner::ZERO,
                top_right: round,
                bottom_right: round,
                bottom_left: RoundedCorner::ZERO,
            }
        } else {
            Shape::RoundedRectangle {
                top_left: round,
                top_right: RoundedCorner::ZERO,
                bottom_right: RoundedCorner::ZERO,
                bottom_left: round,
            }
        }
    } else {
        Shape::RECTANGLE
    }
}

fn mirror(start: bool, width: f64, x: f64, element: f64) -> f64 {
    if start { width - x - element } else { x }
}

/// A static side sheet with the official header, body and action geometry.
#[tessera]
pub(super) fn sheet_panel(mock: Mock, x: f64, y: f64, width: f64, height: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let container = if mock.modal {
        scheme.surface_container_low
    } else {
        scheme.surface
    };
    let body_bottom = if mock.actions {
        height - ACTIONS_HEIGHT
    } else {
        height
    };
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            at(x, y, width, height)
                .background_with_shape(container, sheet_shape(mock))
                .clip_shape(sheet_shape(mock)),
        )
        .children(move || {
            if mock.back {
                glyph(
                    Glyph::Back,
                    mirror(mock.start, width, 16.0, 24.0),
                    24.0,
                    scheme.on_surface_variant,
                );
            }
            let lead = if mock.back { 52.0 } else { 24.0 };
            let trail = if mock.close { 60.0 } else { 24.0 };
            let title_w = (width - lead - trail).max(0.0);
            boxed()
                .alignment(if mock.start {
                    Alignment::CenterEnd
                } else {
                    Alignment::CenterStart
                })
                .modifier(at(
                    mirror(mock.start, width, lead, title_w),
                    22.0,
                    title_w,
                    28.0,
                ))
                .children(move || {
                    text()
                        .content(mock.title)
                        .style(type_style(22.0, 28.0))
                        .color(scheme.on_surface_variant);
                });
            if mock.close {
                glyph(
                    Glyph::Close,
                    mirror(mock.start, width, width - 48.0, 24.0),
                    24.0,
                    scheme.on_surface_variant,
                );
            }
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    at(0.0, HEADER_HEIGHT, width, body_bottom - HEADER_HEIGHT).clip_to_bounds(),
                )
                .children(move || {
                    sheet_body(mock.body, mock.start, width, body_bottom - HEADER_HEIGHT);
                });
            if mock.actions {
                if mock.divider {
                    boxed().modifier(
                        at(0.0, body_bottom, width, 1.0).background(scheme.outline_variant),
                    );
                }
                mock_button(true, "Save", 24.0, body_bottom + 16.0, 72.0);
                mock_button(false, "Cancel", 104.0, body_bottom + 16.0, 88.0);
            }
            if mock.focus {
                let ring = |rx: f64, ry: f64, rw: f64, rh: f64| {
                    at(rx - 3.0, ry - 3.0, rw + 6.0, rh + 6.0).border_with_shape(
                        Dp(3.0),
                        scheme.secondary,
                        Shape::rounded_rectangle(Dp(8.0)),
                    )
                };
                boxed().modifier(ring(lead - 4.0, 18.0, 72.0, 36.0));
                boxed().modifier(ring(width - 56.0, 16.0, 40.0, 40.0));
                if mock.actions {
                    boxed().modifier(ring(24.0, body_bottom + 16.0, 72.0, 32.0));
                    boxed().modifier(ring(104.0, body_bottom + 16.0, 88.0, 32.0));
                }
            }
            if !mock.modal && mock.divider {
                let line_x = if mock.start { width - 1.0 } else { 0.0 };
                boxed().modifier(at(line_x, 0.0, 1.0, height).background(scheme.outline_variant));
            }
        });
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Glyph {
    #[default]
    Back,
    Close,
}

#[tessera]
fn glyph(kind: Glyph, x: f64, y: f64, tint: Color) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(at(x, y, 24.0, 24.0))
        .children(move || {
            icon()
                .vector(match kind {
                    Glyph::Back => filled::ARROW_BACK_SVG,
                    Glyph::Close => filled::CLOSE_SVG,
                })
                .size(Dp(24.0))
                .tint(tint);
        });
}

#[tessera]
fn mock_button(filled: bool, #[prop(into)] label: String, x: f64, y: f64, width: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let modifier = at(x, y, width, 32.0);
    let modifier = if filled {
        modifier.background_with_shape(scheme.primary, Shape::CAPSULE)
    } else {
        modifier.border_with_shape(Dp(1.0), scheme.outline_variant, Shape::CAPSULE)
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            text()
                .content(label.clone())
                .style(type_style(14.0, 20.0))
                .color(if filled {
                    scheme.on_primary
                } else {
                    scheme.on_surface_variant
                });
        });
}

#[tessera]
fn label(#[prop(into)] value: String, x: f64, y: f64, size: f64, color: Color) {
    text()
        .content(value)
        .style(type_style(size, size + 6.0))
        .color(color)
        .modifier(Modifier::new().offset(Dp(x), Dp(y)));
}

#[tessera]
fn sheet_body(body: Body, start: bool, width: f64, height: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let ink = scheme.on_surface_variant;
    match body {
        Body::Filters => {
            label("Labels", 24.0, 4.0, 14.0, ink);
            for (index, (name, checked)) in [
                ("Events", false),
                ("Personal", false),
                ("Projects", true),
                ("Reminders", true),
                ("Family", false),
            ]
            .into_iter()
            .enumerate()
            {
                let y = 36.0 + index as f64 * 44.0;
                let square = at(28.0, y + 2.0, 18.0, 18.0);
                if checked {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(square.background_with_shape(
                            scheme.primary,
                            Shape::rounded_rectangle(Dp(2.0)),
                        ))
                        .children(move || {
                            icon()
                                .vector(filled::CHECK_SVG)
                                .size(Dp(16.0))
                                .tint(scheme.on_primary);
                        });
                } else {
                    boxed().modifier(square.border_with_shape(
                        Dp(2.0),
                        scheme.on_surface_variant,
                        Shape::rounded_rectangle(Dp(2.0)),
                    ));
                }
                label(name, 64.0, y, 16.0, scheme.on_surface);
            }
            label("Format", 24.0, 264.0, 14.0, ink);
            for (index, name) in ["Any", "Documents", "Images"].into_iter().enumerate() {
                radio(28.0, 296.0 + index as f64 * 44.0, index == 0, name);
            }
        }
        Body::Info => {
            let photo_w = width - 48.0;
            boxed()
                .alignment(Alignment::Center)
                .modifier(at(24.0, 0.0, photo_w, 160.0).background_with_shape(
                    scheme.tertiary_container,
                    Shape::rounded_rectangle(Dp(12.0)),
                ))
                .children(move || {
                    icon()
                        .vector(filled::MUSIC_NOTE_SVG)
                        .size(Dp(40.0))
                        .tint(scheme.on_tertiary_container);
                });
            label("Festival concert", 24.0, 176.0, 16.0, scheme.on_surface);
            label("Albums", 24.0, 212.0, 14.0, ink);
            for (index, (name, items)) in
                [("Home space", "64 items"), ("Favorite sights", "18 items")]
                    .into_iter()
                    .enumerate()
            {
                let y = 244.0 + index as f64 * 64.0;
                boxed().modifier(at(24.0, y, 48.0, 48.0).background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(8.0)),
                ));
                label(name, 88.0, y + 2.0, 16.0, scheme.on_surface);
                label(items, 88.0, y + 26.0, 12.0, ink);
            }
        }
        Body::Settings => {
            label("Density", 24.0, 4.0, 14.0, ink);
            for (index, name) in ["Default", "Comfortable", "Compact"]
                .into_iter()
                .enumerate()
            {
                radio(28.0, 36.0 + index as f64 * 44.0, index == 0, name);
            }
            label("Theme", 24.0, 176.0, 14.0, ink);
            for (index, name) in ["System", "Light", "Dark"].into_iter().enumerate() {
                radio(28.0, 208.0 + index as f64 * 44.0, index == 0, name);
            }
        }
        Body::Scroll => {
            for index in 0..10 {
                let y = index as f64 * 56.0;
                boxed().modifier(at(24.0, y, 40.0, 40.0).background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(8.0)),
                ));
                boxed().modifier(
                    at(80.0, y + 6.0, (width - 128.0).max(0.0), 12.0)
                        .background_with_shape(scheme.surface_container_highest, Shape::CAPSULE),
                );
                boxed().modifier(
                    at(80.0, y + 24.0, (width - 168.0).max(0.0), 10.0)
                        .background_with_shape(scheme.surface_container_high, Shape::CAPSULE),
                );
            }
            boxed().modifier(
                at(width - 10.0, 8.0, 4.0, (height * 0.35).max(24.0))
                    .background_with_shape(scheme.outline, Shape::CAPSULE),
            );
        }
        Body::Chips => {
            label("Tags", 24.0, 4.0, 14.0, ink);
            let mut chip_x = 24.0;
            for (name, chip_w) in [
                ("Travel", 84.0),
                ("Family", 84.0),
                ("Concerts", 100.0),
                ("Archive", 92.0),
            ] {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(at(chip_x, 36.0, chip_w, 32.0).border_with_shape(
                        Dp(1.0),
                        scheme.outline,
                        Shape::rounded_rectangle(Dp(8.0)),
                    ))
                    .children(move || {
                        text()
                            .content(name)
                            .style(type_style(14.0, 20.0))
                            .color(ink);
                    });
                chip_x += chip_w + 8.0;
            }
        }
        Body::Empty => {}
        Body::Lines => {
            for (index, fraction) in [0.8, 0.62, 0.72, 0.5].into_iter().enumerate() {
                let bar = (width - 48.0) * fraction;
                let bar_x = if start { width - 24.0 - bar } else { 24.0 };
                boxed().modifier(
                    at(bar_x, 8.0 + index as f64 * 28.0, bar, 12.0)
                        .background_with_shape(scheme.surface_container_highest, Shape::CAPSULE),
                );
            }
        }
    }
}

#[tessera]
fn radio(x: f64, y: f64, selected: bool, #[prop(into)] name: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let color = if selected {
        scheme.primary
    } else {
        scheme.on_surface_variant
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(at(x, y + 1.0, 20.0, 20.0).border_with_shape(Dp(2.0), color, Shape::Ellipse))
        .children(move || {
            if selected {
                boxed().modifier(
                    Modifier::new()
                        .size(Dp(10.0), Dp(10.0))
                        .background_with_shape(color, Shape::Ellipse),
                );
            }
        });
    label(name, x + 36.0, y, 16.0, scheme.on_surface);
}

/// Standard (1) and modal (2) side sheets, each cropped from a device.
#[tessera]
pub(super) fn hero() {
    stage(
        1000.0,
        480.0,
        "Side by side comparison of a standard and a modal side sheet",
    )
    .child(|| {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        for (index, modal) in [false, true].into_iter().enumerate() {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(at(index as f64 * 500.0, 0.0, 500.0, 480.0).clip_to_bounds())
                .children(move || {
                    if modal {
                        device(Mock::modal(300.0), -100.0, 100.0, 540.0, 480.0);
                    } else {
                        device(Mock::standard(300.0), -260.0, 100.0, 660.0, 480.0);
                    }
                    marker(index + 1, 238.0, 30.0);
                });
        }
        boxed().modifier(at(499.5, 0.0, 1.0, 480.0).background(scheme.outline));
    });
}

/// Usage examples: a photo with an Info sheet, or files with a Filters sheet.
#[tessera]
pub(super) fn usage_figure(modal: bool) {
    stage(
        640.0,
        440.0,
        if modal {
            "Modal side sheet showing filter controls"
        } else {
            "Standard side sheet showing supplementary information about a photo"
        },
    )
    .child(move || {
        let mock = if modal {
            Mock {
                back: false,
                actions: false,
                ..Mock::modal(240.0)
                    .body(Body::Filters, "Filters")
                    .backdrop(Backdrop::Files)
            }
        } else {
            Mock::standard(240.0)
                .body(Body::Info, "Info")
                .backdrop(Backdrop::Gallery)
        };
        device(mock, 40.0, 40.0, 560.0, 440.0);
    });
}

/// The M3 modal side sheet with its scrim and 16dp leading corners.
#[tessera]
pub(super) fn differences_figure() {
    stage(640.0, 300.0, "Modal side sheet with new color mappings").child(|| {
        device(Mock::modal(300.0), -40.0, 52.0, 593.0, 400.0);
    });
}

fn anatomy_callouts(modal: bool) -> Vec<Callout> {
    if modal {
        let (sx, sy, sw, sh) = sheet_frame(MODAL_ANATOMY, 100.0, 96.0, 800.0, 480.0);
        vec![
            callout(1, (sx + 16.0, 40.0), (sx + 28.0, sy + 36.0)),
            callout(2, (sx + 64.0, 40.0), (sx + 76.0, sy + 34.0)),
            callout(
                3,
                (sx + sw * 0.55, 40.0),
                (sx + sw * 0.55 + 12.0, sy + 120.0),
            ),
            callout(4, (sx + sw + 24.0, sy + 20.0), (sx + sw - 36.0, sy + 36.0)),
            callout(
                5,
                (sx - 72.0, sy + sh - 88.0),
                (sx, sy + sh - ACTIONS_HEIGHT),
            ),
            callout(6, (sx + 48.0, sy + sh + 32.0), (sx + 60.0, sy + sh - 24.0)),
            callout(7, (sx - 260.0, sy + 120.0), (sx - 248.0, sy + 200.0)),
        ]
    } else {
        let (sx, sy, sw, _) = sheet_frame(STANDARD_ANATOMY, 100.0, 96.0, 800.0, 560.0);
        vec![
            callout(1, (sx - 12.0, 40.0), (sx, sy + 24.0)),
            callout(2, (sx + 32.0, 40.0), (sx + 44.0, sy + 24.0)),
            callout(
                3,
                (sx + sw * 0.55, 40.0),
                (sx + sw * 0.55 + 12.0, sy + 120.0),
            ),
            callout(4, (sx + sw + 24.0, sy + 20.0), (sx + sw - 36.0, sy + 36.0)),
        ]
    }
}

const STANDARD_ANATOMY: Mock = Mock::standard(296.0);
const MODAL_ANATOMY: Mock = Mock::modal(296.0);

/// Numbered anatomy of a standard (4 elements) or modal (7 elements) sheet.
#[tessera]
pub(super) fn anatomy(modal: bool, #[prop(into)] figure: String) {
    let height = if modal { 640.0 } else { 600.0 };
    stage(
        1000.0,
        height,
        if modal {
            "7 elements of a modal side sheet"
        } else {
            "4 elements of a standard side sheet"
        },
    )
    .child(move || {
        if modal {
            device(MODAL_ANATOMY, 100.0, 96.0, 800.0, 480.0);
        } else {
            device(STANDARD_ANATOMY, 100.0, 96.0, 800.0, 560.0);
        }
        callouts(1000.0, height, anatomy_callouts(modal)).figure(figure.clone());
    });
}

pub(super) fn anatomy_legend(modal: bool) -> Vec<(usize, String)> {
    let labels: &[&str] = if modal {
        &[
            "Back icon button (optional)",
            "Headline",
            "Container",
            "Close icon button",
            "Divider (optional)",
            "Action buttons (optional)",
            "Scrim",
        ]
    } else {
        &[
            "Divider (optional)",
            "Headline",
            "Container",
            "Close icon button",
        ]
    };
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| (index + 1, (*label).to_string()))
        .collect()
}

/// Color roles in light and dark themes.
#[tessera]
pub(super) fn color_roles(modal: bool) {
    for dark in [false, true] {
        stage(
            1000.0,
            460.0,
            if dark {
                "Side sheet color roles in the dark theme"
            } else {
                "Side sheet color roles in the light theme"
            },
        )
        .dark(dark)
        .dotted(true)
        .child(move || {
            let mock = Mock {
                actions: false,
                ..if modal {
                    MODAL_ANATOMY
                } else {
                    STANDARD_ANATOMY
                }
            };
            device(mock, 100.0, 96.0, 800.0, 480.0);
            let mut items = anatomy_callouts(modal);
            items.truncate(4);
            callouts(1000.0, 460.0, items);
        });
        gap(Dp(16.0));
    }
}

pub(super) fn color_legend(modal: bool) -> Vec<(usize, String)> {
    let roles: [&str; 4] = if modal {
        [
            "On surface variant",
            "On surface variant",
            "Surface container low",
            "On surface variant",
        ]
    } else {
        [
            "Outline variant",
            "On surface variant",
            "Surface",
            "On surface variant",
        ]
    };
    roles
        .into_iter()
        .enumerate()
        .map(|(index, role)| (index + 1, role.to_string()))
        .collect()
}

/// Layout tag for one measured element of the measurement figure.
pub(super) fn measurement_tag(modal: bool, name: &str) -> String {
    format!(
        "side-measurement-{}-{name}",
        if modal { "modal" } else { "standard" }
    )
}

fn part(geometry: State<SpaceGeometry>, modal: bool, name: &'static str) -> Modifier {
    watch_in_space(geometry, name).semantics(SemanticsArgs {
        test_tag: Some(measurement_tag(modal, name)),
        merge_descendants: true,
        ..Default::default()
    })
}

/// Live padding and size measurements over a real sheet header.
#[tessera]
pub(super) fn measurement_figure(modal: bool) {
    let scheme = MaterialTheme::from_seed(figure_seed(), false).color_scheme;
    let geometry = remember(SpaceGeometry::new);
    material_theme()
        .theme(|| MaterialTheme::from_seed(figure_seed(), false))
        .child(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    part(geometry, modal, "figure")
                        .fill_max_width()
                        .height(Dp(480.0))
                        .background_with_shape(
                            scheme.secondary_container,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    boxed()
                        .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                            Dp(24.0),
                            Dp(56.0),
                            Dp(24.0),
                            Dp(24.0),
                        )))
                        .children(move || {
                            row()
                                .modifier(
                                    Modifier::new()
                                        .fill_max_size()
                                        .border_with_shape(
                                            Dp(DEVICE_BORDER),
                                            scheme.outline_variant,
                                            Shape::rounded_rectangle(Dp(DEVICE_RADIUS)),
                                        )
                                        .padding_all(Dp(DEVICE_BORDER))
                                        .clip_shape(Shape::rounded_rectangle(Dp(
                                            DEVICE_RADIUS - DEVICE_BORDER
                                        ))),
                                )
                                .children(move || {
                                    boxed().modifier(
                                        Modifier::new().weight(1.0).fill_max_height().background(
                                            if modal {
                                                scheme.scrim.with_alpha(0.32)
                                            } else {
                                                scheme.surface
                                            },
                                        ),
                                    );
                                    boxed()
                                        .modifier(
                                            Modifier::new()
                                                .size_in(None, Some(Dp(400.0)), None, None)
                                                .fill_max_width()
                                                .fill_max_height(),
                                        )
                                        .children(move || {
                                            measurement_sheet(modal, geometry);
                                        });
                                });
                        });
                    measurement_rulers(geometry);
                });
        });
}

#[tessera]
fn measurement_sheet(modal: bool, geometry: State<SpaceGeometry>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let shape = if modal {
        sheet_shape(Mock::modal(0.0))
    } else {
        Shape::RECTANGLE
    };
    column()
        .modifier(
            part(geometry, modal, "container")
                .fill_max_size()
                .background_with_shape(
                    if modal {
                        scheme.surface_container_low
                    } else {
                        scheme.surface
                    },
                    shape,
                ),
        )
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                    Dp(if modal { 4.0 } else { 24.0 }),
                    Dp(12.0),
                    Dp(12.0),
                    Dp(12.0),
                )))
                .children(move || {
                    if modal {
                        icon_button()
                            .standard()
                            .icon(filled::ARROW_BACK_SVG)
                            .accessibility_label("Side sheet figure back")
                            .modifier(part(geometry, modal, "back"))
                            .on_click(|| {});
                    }
                    text()
                        .content("Headline")
                        .style(type_style(22.0, 28.0))
                        .color(scheme.on_surface_variant)
                        .modifier(part(geometry, modal, "headline").weight(1.0));
                    icon_button()
                        .standard()
                        .icon(filled::CLOSE_SVG)
                        .accessibility_label("Side sheet figure close")
                        .modifier(part(geometry, modal, "close"))
                        .on_click(|| {});
                });
            for name in ["Labels", "Events", "Personal", "Projects"] {
                text()
                    .content(name)
                    .style(type_style(14.0, 20.0))
                    .color(scheme.on_surface_variant)
                    .modifier(Modifier::new().padding_symmetric(Dp(24.0), Dp(6.0)));
            }
            spacer().modifier(Modifier::new().weight(1.0));
            if modal {
                boxed().modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(1.0))
                        .background(scheme.outline_variant),
                );
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .modifier(
                        part(geometry, modal, "actions")
                            .fill_max_width()
                            .height(Dp(ACTIONS_HEIGHT))
                            .padding(Padding::new(Dp(24.0), Dp(16.0), Dp(24.0), Dp(24.0))),
                    )
                    .children(|| {
                        button()
                            .filled()
                            .size(ButtonSize::ExtraSmall)
                            .accessibility_label("Save filters")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Save");
                            });
                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                        button()
                            .outlined()
                            .size(ButtonSize::ExtraSmall)
                            .accessibility_label("Cancel filters")
                            .on_click(|| {})
                            .child(|| {
                                text().content("Cancel");
                            });
                    });
            }
        });
}

/// Glyph rectangle centered in a measured icon button.
pub(super) fn glyph_rect(button: &annotations::Rect) -> annotations::Rect {
    annotations::Rect {
        x: button.x + button.w * 0.5 - 12.0,
        y: button.y + button.h * 0.5 - 12.0,
        w: 24.0,
        h: 24.0,
    }
}

#[tessera]
fn measurement_rulers(geometry: State<SpaceGeometry>) {
    let values = geometry.with(annotations::space_geometry);
    let (Some(figure), Some(container), Some(headline), Some(close)) = (
        values.get("figure"),
        values.get("container"),
        values.get("headline"),
        values.get("close"),
    ) else {
        return;
    };
    let close = glyph_rect(close);
    let back = values.get("back").map(glyph_rect);
    let band = "#f2b8c6";
    let tick = "#b3261e";
    let top = container.y;
    let mut spans = Vec::new();
    let mut svg = String::new();
    let mut horizontal = |svg: &mut String, from: f64, to: f64| {
        svg.push_str(&format!(
            "<rect x='{from}' y='{top}' width='{}' height='{}' fill='{band}' fill-opacity='0.8'/>",
            (to - from).max(0.0),
            HEADER_HEIGHT - 16.0
        ));
        let y = top - 10.0;
        svg.push_str(&format!(
            "<path d='M{from},{y} L{to},{y} M{from},{} L{from},{} M{to},{} L{to},{}' stroke='{tick}' stroke-width='1' fill='none'/>",
            y - 3.0,
            y + 3.0,
            y - 3.0,
            y + 3.0
        ));
        spans.push((from, to));
    };
    if let Some(back) = back {
        horizontal(&mut svg, container.x, back.x);
        horizontal(&mut svg, back.x + back.w, headline.x);
    } else {
        horizontal(&mut svg, container.x, headline.x);
    }
    horizontal(&mut svg, close.x - 12.0, close.x);
    horizontal(&mut svg, close.x + close.w, container.x + container.w);
    svg.push_str(&format!(
        "<rect x='{}' y='{top}' width='{}' height='{}' fill='{band}' fill-opacity='0.8'/>",
        container.x,
        container.w,
        close.y - top
    ));
    let rx = container.x + container.w + 14.0;
    svg.push_str(&format!(
        "<path d='M{rx},{top} L{rx},{} M{},{top} L{},{top} M{},{} L{},{}' stroke='{tick}' stroke-width='1' fill='none'/>",
        close.y,
        rx - 3.0,
        rx + 3.0,
        rx - 3.0,
        close.y,
        rx + 3.0,
        close.y
    ));
    annotations::lines(Dp(figure.w), Dp(figure.h), svg);
    for (from, to) in spans {
        annotations::dimension(
            format!("{:.0}", to - from),
            Dp((from + to) * 0.5 - 8.0),
            Dp(top - 34.0),
        );
    }
    annotations::dimension(
        format!("{:.0}", close.y - top),
        Dp((rx + 4.0).min(figure.w - 20.0)),
        Dp(top + (close.y - top) * 0.5 - 10.0),
    );
}

/// Inset Do/Don't figure for sheet placement.
#[tessera]
pub(super) fn inset_figure(inset: f64) {
    stage(
        480.0,
        320.0,
        if inset > 16.0 {
            "A modal side sheet at the right of the screen with the wrong inset"
        } else {
            "A modal sheet at the right of a screen, with the correct inset"
        },
    )
    .child(move || {
        let mock = Mock {
            back: false,
            actions: false,
            inset,
            ..Mock::modal(176.0)
                .body(Body::Filters, "Filters")
                .backdrop(Backdrop::Files)
        };
        device(mock, 24.0, 24.0, 432.0, 360.0);
    });
}

const ELEMENT_MOCK: Mock = Mock::modal(296.0);

/// Cropped modal sheet with a single marker on one element.
#[tessera]
pub(super) fn element_figure(#[prop(into)] element: String) {
    let (sx, sy, sw, sh) = sheet_frame(ELEMENT_MOCK, 80.0, 72.0, 640.0, 528.0);
    let (title, item) = match element.as_str() {
        "back" => (
            "Back icon button on the upper left of a modal side sheet",
            callout(1, (sx - 56.0, 32.0), (sx + 28.0, sy + 36.0)),
        ),
        "close" => (
            "Close icon button on the upper right of a modal side sheet",
            callout(4, (sx + sw + 8.0, 32.0), (sx + sw - 36.0, sy + 36.0)),
        ),
        "actions" => (
            "Save and cancel buttons at the bottom of a modal side sheet",
            callout(6, (sx - 72.0, sy + sh - 56.0), (sx + 24.0, sy + sh - 40.0)),
        ),
        "divider" => (
            "Horizontal divider on a modal side sheet",
            callout(
                5,
                (sx - 72.0, sy + sh - 104.0),
                (sx, sy + sh - ACTIONS_HEIGHT),
            ),
        ),
        _ => (
            "A modal side sheet's container",
            callout(3, (sx - 72.0, sy + 180.0), (sx + 40.0, sy + 200.0)),
        ),
    };
    stage(800.0, 600.0, title).child(move || {
        device(ELEMENT_MOCK, 80.0, 72.0, 640.0, 528.0);
        callouts(800.0, 600.0, vec![item]).figure(format!("element-{element}"));
    });
}

/// Two side sheets showing different content and layouts.
#[tessera]
pub(super) fn content_figure() {
    stage(
        800.0,
        520.0,
        "2 side sheets with different content displayed side by side",
    )
    .child(|| {
        let settings = Mock {
            close: true,
            ..Mock::standard(320.0).body(Body::Settings, "Settings")
        };
        let info = Mock::standard(320.0).body(Body::Info, "Info");
        sheet_panel(
            Mock {
                divider: false,
                ..settings
            },
            48.0,
            48.0,
            320.0,
            424.0,
        );
        sheet_panel(
            Mock {
                divider: false,
                ..info
            },
            432.0,
            48.0,
            320.0,
            424.0,
        );
    });
}

/// A modal sheet on a phone becoming a standard sheet on a tablet.
#[tessera]
pub(super) fn transition_figure() {
    stage(1000.0, 520.0, "Modal side sheets on smaller screens can transition to standard side sheets at larger screen sizes").child(|| {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        let phone = Mock {
            back: false,
            ..Mock::modal(208.0).body(Body::Filters, "Filters").backdrop(Backdrop::Files)
        };
        let tablet = Mock::standard(256.0).body(Body::Filters, "Filters").backdrop(Backdrop::Files);
        device(phone, 48.0, 48.0, 288.0, 424.0);
        boxed()
            .alignment(Alignment::Center)
            .modifier(at(352.0, 236.0, 48.0, 48.0))
            .children(move || {
                icon().vector(filled::ARROW_FORWARD_SVG).size(Dp(32.0)).tint(scheme.on_secondary_container);
            });
        device(tablet, 416.0, 48.0, 536.0, 424.0);
    });
}

/// A side sheet on the left edge with mirrored elements.
#[tessera]
pub(super) fn rtl_figure() {
    stage(
        800.0,
        520.0,
        "Side sheet along the left edge of a screen. All buttons and icons are reversed",
    )
    .child(|| {
        let mock = Mock {
            start: true,
            back: true,
            actions: false,
            body: Body::Lines,
            backdrop: Backdrop::Lines,
            ..Mock::modal(296.0)
        };
        device(mock, 80.0, 48.0, 640.0, 520.0);
    });
}

/// Vertical scrolling Do and horizontal scrolling Don't figures.
#[tessera]
pub(super) fn scroll_figure(good: bool) {
    stage(
        480.0,
        360.0,
        if good {
            "A side sheet scrolling vertically"
        } else {
            "A side sheet appears to scroll horizontally"
        },
    )
    .child(move || {
        let mock = Mock::standard(208.0).body(
            if good { Body::Scroll } else { Body::Chips },
            if good { "Library" } else { "Tags" },
        );
        device(mock, 24.0, 24.0, 432.0, 400.0);
    });
}

/// A detached sheet scaling with the predictive back gesture.
#[tessera]
pub(super) fn predictive_back_figure() {
    stage(800.0, 480.0, "Preview of the predictive back gesture on a side sheet").child(|| {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        let mock = Mock {
            inset: 24.0,
            actions: false,
            ..Mock::modal(256.0).body(Body::Filters, "Filters").backdrop(Backdrop::Files)
        };
        device(mock, 80.0, 48.0, 640.0, 480.0);
        let mut svg = String::new();
        let stroke = hex(scheme.on_secondary_container);
        svg.push_str(&format!(
            "<path d='M560,300 L440,300 M452,290 L440,300 L452,310' stroke='{stroke}' stroke-width='3' fill='none'/>"
        ));
        boxed().modifier(at(0.0, 0.0, 800.0, 480.0)).children(move || {
            annotations::lines(Dp(800.0), Dp(480.0), svg.clone());
        });
    });
}

/// Close-affordance Do and Don't figures.
#[tessera]
pub(super) fn close_figure(good: bool) {
    stage(
        480.0,
        360.0,
        if good {
            "Side sheet correctly designed with close icon in upper right corner"
        } else {
            "Side sheet incorrectly designed with no close icon button"
        },
    )
    .child(move || {
        let mock = Mock {
            back: false,
            actions: false,
            close: good,
            ..Mock::modal(208.0)
                .body(Body::Filters, "Filters")
                .backdrop(Backdrop::Files)
        };
        device(mock, 24.0, 24.0, 432.0, 400.0);
    });
}

const FOCUS_MOCK: Mock = Mock {
    back: false,
    focus: true,
    ..Mock::modal(296.0).body(Body::Filters, "Filters")
};

/// Visible focus on the headline, close and action buttons.
#[tessera]
pub(super) fn focus_figure() {
    let (sx, sy, sw, sh) = sheet_frame(FOCUS_MOCK, 100.0, 48.0, 800.0, 528.0);
    stage(
        1000.0,
        600.0,
        "Side sheet diagram showing the focus order of headline, close, save, cancel",
    )
    .child(move || {
        device(FOCUS_MOCK, 100.0, 48.0, 800.0, 528.0);
        callouts(
            1000.0,
            600.0,
            vec![
                callout(1, (sx - 64.0, sy + 20.0), (sx + 20.0, sy + 36.0)),
                callout(2, (sx + sw + 24.0, sy + 20.0), (sx + sw - 16.0, sy + 36.0)),
                callout(4, (sx + 136.0, sy + sh + 8.0), (sx + 148.0, sy + sh - 20.0)),
                callout(3, (sx - 64.0, sy + sh - 56.0), (sx + 20.0, sy + sh - 40.0)),
            ],
        )
        .figure("focus");
    });
}

/// The accessibility role of a side sheet.
#[tessera]
pub(super) fn role_figure() {
    let mock = Mock {
        back: false,
        ..Mock::modal(296.0)
            .body(Body::Filters, "Filters")
            .backdrop(Backdrop::Files)
    };
    let (sx, sy, _, _) = sheet_frame(mock, 100.0, 48.0, 800.0, 528.0);
    stage(1000.0, 600.0, "Side sheet showing the accessibility role as dialog").child(move || {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        device(mock, 100.0, 48.0, 800.0, 528.0);
        let stroke = hex(scheme.on_surface_variant);
        let svg = format!(
            "<path d='M{},{} L{},{}' stroke='{stroke}' stroke-width='1' fill='none'/><circle cx='{}' cy='{}' r='3' fill='none' stroke='{stroke}'/>",
            sx - 120.0,
            sy + 160.0,
            sx,
            sy + 160.0,
            sx,
            sy + 160.0
        );
        boxed().modifier(at(0.0, 0.0, 1000.0, 600.0)).children(move || {
            annotations::lines(Dp(1000.0), Dp(600.0), svg.clone());
        });
        boxed()
            .alignment(Alignment::Center)
            .modifier(
                at(sx - 240.0, sy + 140.0, 120.0, 40.0)
                    .background_with_shape(scheme.inverse_surface, Shape::rounded_rectangle(Dp(8.0))),
            )
            .children(move || {
                text()
                    .content("Role: Dialog")
                    .style(type_style(14.0, 20.0))
                    .color(scheme.inverse_on_surface);
            });
    });
}
