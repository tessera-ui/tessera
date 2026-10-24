//! Native light and dark sheet color diagrams.
//!
//! ## Usage
//!
//! Compare sheet color roles with interactive native icon controls.

use std::collections::BTreeMap;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, Modifier, PlacementModifierNode, PxPosition, State, accesskit::Role,
    modifier::ModifierCapabilityExt as _, remember, tessera, use_context,
};

use super::{
    super::{ACCENT, type_style},
    annotations::{self, PreciseGeometry, Rect, dotted, watch_precise},
    sheet_canvas,
};

#[derive(Clone, Copy, PartialEq)]
struct ExactOffset {
    x: Dp,
    y: Dp,
}
impl PlacementModifierNode for ExactOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(self.x.to_pixels_f32(), self.y.to_pixels_f32())
    }
}

fn at(x: f64, y: f64) -> Modifier {
    Modifier::new().push_placement(ExactOffset { x: Dp(x), y: Dp(y) })
}

#[tessera]
pub(super) fn sheet_colors(side: bool, modal: bool) {
    let notice = remember(String::new);
    let half_height: f64 = if side { 440.5 } else { 502.5 };
    sheet_canvas(
        1024.0,
        half_height * 2.0,
        tessera_ui::RenderSlot::new(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().clip_to_bounds())
                .children(move || {
                    for dark in [false, true] {
                        material_theme()
                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                            .child(move || {
                                color_half(side, modal, dark, notice);
                            });
                    }
                });
        }),
    );
    if !notice.with(String::is_empty) {
        super::gap(Dp(12.0));
        super::prose(notice.get());
    }
}

#[tessera]
fn color_half(side: bool, modal: bool, dark: bool, notice: State<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let height: f64 = if side { 440.5 } else { 502.5 };
    let geometry = remember(PreciseGeometry::new);
    #[cfg(all(test, not(target_family = "wasm")))]
    tests::GEOMETRY.with_borrow_mut(|values| values[dark as usize] = Some(geometry));
    #[cfg(all(test, not(target_family = "wasm")))]
    tests::NOTICE.set(Some(notice));
    boxed()
        .modifier(
            at(0.0, if dark { height } else { 0.0 })
                .then(watch_precise(geometry, "figure"))
                .size(Dp(1024.0), Dp(height.ceil()))
                .clip_to_bounds()
                .background(scheme.surface),
        )
        .children(move || {
            grid(height, dark);
            if side {
                boxed()
                    .modifier(
                        at(83.0, 106.0)
                            .size(Dp(857.0), Dp(700.0))
                            .background_with_shape(
                                scheme.outline_variant,
                                Shape::rounded_rectangle(Dp(28.0)),
                            ),
                    )
                    .children(move || {
                        spacer().modifier(
                            at(8.0, 8.0)
                                .size(Dp(841.0), Dp(684.0))
                                .background_with_shape(
                                    scheme.surface,
                                    Shape::rounded_rectangle(Dp(20.0)),
                                ),
                        );
                    });
                if modal {
                    spacer().modifier(
                        at(91.0, 114.0)
                            .size(Dp(841.0), Dp(684.0))
                            .background_with_shape(
                                scheme.scrim.with_alpha(0.32),
                                Shape::rounded_rectangle(Dp(20.0)),
                            ),
                    );
                }
                boxed()
                    .modifier(
                        at(612.0, 114.0)
                            .then(watch_precise(geometry, "container"))
                            .size(Dp(320.0), Dp(684.0))
                            .background_with_shape(
                                if modal {
                                    scheme.surface_container_low
                                } else {
                                    scheme.surface
                                },
                                if modal {
                                    Shape::rounded_rectangle(Dp(16.0))
                                } else {
                                    Shape::RECTANGLE
                                },
                            ),
                    )
                    .children(move || {
                        if !modal {
                            spacer().modifier(
                                watch_precise(geometry, "divider")
                                    .width(Dp(1.0))
                                    .fill_max_height()
                                    .background(scheme.outline_variant),
                            );
                        }
                        if modal {
                            icon_button()
                                .standard()
                                .icon(filled::ARROW_BACK_SVG)
                                .content_color(scheme.on_surface_variant)
                                .accessibility_label(format!(
                                    "{} sheet color back",
                                    if dark { "Dark" } else { "Light" }
                                ))
                                .modifier(at(4.0, 12.0).then(watch_precise(geometry, "back")))
                                .on_click(move || notice.set("Back action selected".into()));
                        }
                        text()
                            .content("Headline")
                            .style(type_style(22.0, 28.0))
                            .color(scheme.on_surface_variant)
                            .modifier(
                                at(if modal { 52.0 } else { 24.0 }, 22.0)
                                    .then(watch_precise(geometry, "headline")),
                            );
                        icon_button()
                            .standard()
                            .icon(filled::CLOSE_SVG)
                            .content_color(scheme.on_surface_variant)
                            .accessibility_label(format!(
                                "{} sheet color close",
                                if dark { "Dark" } else { "Light" }
                            ))
                            .modifier(at(260.0, 12.0).then(watch_precise(geometry, "close")))
                            .on_click(move || notice.set("Sheet close action selected".into()));
                    });
            } else {
                boxed()
                    .modifier(
                        at(298.0, -480.0)
                            .size(Dp(428.0), Dp(909.0))
                            .background_with_shape(
                                scheme.outline_variant,
                                Shape::rounded_rectangle(Dp(28.0)),
                            ),
                    )
                    .children(move || {
                        spacer().modifier(
                            at(8.0, 8.0)
                                .size(Dp(412.0), Dp(893.0))
                                .background_with_shape(
                                    scheme.surface,
                                    Shape::rounded_rectangle(Dp(20.0)),
                                ),
                        );
                        spacer().modifier(
                            at(8.0, 8.0)
                                .then(watch_precise(geometry, "scrim"))
                                .size(Dp(412.0), Dp(893.0))
                                .background_with_shape(
                                    scheme.scrim.with_alpha(0.32),
                                    Shape::rounded_rectangle(Dp(20.0)),
                                ),
                        );
                    });
                boxed()
                    .modifier(
                        at(306.0, 131.0)
                            .then(watch_precise(geometry, "container"))
                            .size(Dp(412.0), Dp(290.0))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::RoundedRectangle {
                                    top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                                    top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                                    bottom_left: RoundedCorner::manual(Dp(20.0), 3.0),
                                    bottom_right: RoundedCorner::manual(Dp(20.0), 3.0),
                                },
                            ),
                    )
                    .children(move || {
                        spacer().modifier(
                            at(190.0, 16.0)
                                .then(watch_precise(geometry, "handle"))
                                .size(Dp(32.0), Dp(4.0))
                                .background_with_shape(scheme.on_surface_variant, Shape::CAPSULE),
                        );
                    });
            }
            color_annotations(side, modal, dark, geometry);
        });
}

#[tessera]
fn grid(height: f64, dark: bool) {
    let tone = if dark { "#79747e" } else { "#aea9b4" };
    let mut svg = String::new();
    for y in (1..height.ceil() as usize).step_by(8) {
        for x in (1..1024).step_by(8) {
            svg.push_str(&format!("<circle cx='{x}' cy='{y}' r='.4' fill='{tone}'/>"));
        }
    }
    annotations::lines(Dp(1024.0), Dp(height.ceil()), svg);
}

#[tessera]
fn color_annotations(side: bool, modal: bool, dark: bool, geometry: State<PreciseGeometry>) {
    annotation_content(
        side,
        modal,
        dark,
        geometry.with(annotations::relative_precise_geometry),
    );
}

#[tessera]
fn annotation_content(side: bool, modal: bool, dark: bool, values: BTreeMap<&'static str, Rect>) {
    let Some(root) = values.get("figure") else {
        return;
    };
    let Some(container) = values.get("container") else {
        return;
    };
    let stroke = if dark { "#cac4d0" } else { "#49454f" };
    let mut svg = String::new();
    if side {
        let (Some(headline), Some(close)) = (values.get("headline"), values.get("close")) else {
            return;
        };
        let close_icon = Rect {
            x: close.x + 12.0,
            y: close.y + 12.0,
            w: 24.0,
            h: 24.0,
        };
        outline(&mut svg, *headline, stroke);
        outline(&mut svg, close_icon, stroke);
        if modal {
            if let Some(back) = values.get("back") {
                outline(
                    &mut svg,
                    Rect {
                        x: back.x + 12.0,
                        y: back.y + 12.0,
                        w: 24.0,
                        h: 24.0,
                    },
                    stroke,
                );
            }
        }
        for (number, x, y, tx, ty) in [
            (
                1,
                if modal { 629.0 } else { 600.0 },
                64.0,
                if modal { 640.0 } else { container.x },
                if modal { 138.0 } else { 129.5 },
            ),
            (
                2,
                if modal { 673.0 } else { 646.0 },
                64.0,
                headline.x + 26.0,
                headline.y,
            ),
            (3, 760.0, 64.0, 772.0, 129.5),
            (
                4,
                970.0,
                134.0,
                close_icon.x + close_icon.w,
                close_icon.y + 12.0,
            ),
        ] {
            if number == 4 {
                dotted(&mut svg, tx, ty, x - tx, false, stroke);
            } else if number == 1 && !modal || number == 3 {
                svg.push_str(&format!("<path d='M{tx} 96V{}' stroke='{stroke}'/><circle cx='{tx}' cy='{ty}' r='3.5' fill='none' stroke='{stroke}'/>",ty-3.5));
            } else {
                dotted(&mut svg, tx, 96.0, ty - 96.0, true, stroke);
            }
            marker(number, side, dark, x, y);
        }
    } else {
        let Some(handle) = values.get("handle") else {
            return;
        };
        outline(
            &mut svg,
            Rect {
                x: handle.x - 16.0,
                y: handle.y - 10.0,
                w: 64.0,
                h: 24.0,
            },
            stroke,
        );
        dotted(&mut svg, 248.0, 91.0, 107.0, false, stroke);
        dotted(
            &mut svg,
            512.0,
            107.0,
            handle.y - 10.0 - 107.0,
            true,
            stroke,
        );
        let tx = container.x + container.w - 38.0;
        svg.push_str(&format!("<path d='M{} 283H775' stroke='{stroke}'/><circle cx='{tx}' cy='283' r='3.5' fill='none' stroke='{stroke}'/>",tx+3.5));
        marker(1, side, dark, 224.0, 75.0);
        marker(2, side, dark, 500.0, 75.0);
        marker(3, side, dark, 775.0, 267.0);
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

fn outline(svg: &mut String, rect: Rect, stroke: &str) {
    let perimeter = 2.0 * (rect.w + rect.h);
    let point = |distance: f64| {
        if distance <= rect.w {
            (rect.x + distance, rect.y)
        } else if distance <= rect.w + rect.h {
            (rect.x + rect.w, rect.y + distance - rect.w)
        } else if distance <= 2.0 * rect.w + rect.h {
            (rect.x + 2.0 * rect.w + rect.h - distance, rect.y + rect.h)
        } else {
            (rect.x, rect.y + perimeter - distance)
        }
    };
    for dash in 0..(perimeter / 2.0).ceil() as usize {
        let start = dash as f64 * 2.0;
        let end = (start + 1.0).min(perimeter);
        let (x, y) = point(start);
        svg.push_str(&format!("<path d='M{x} {y}"));
        for corner in [rect.w, rect.w + rect.h, 2.0 * rect.w + rect.h] {
            if corner > start && corner < end {
                let (x, y) = point(corner);
                svg.push_str(&format!("L{x} {y}"));
            }
        }
        let (x, y) = point(end);
        svg.push_str(&format!("L{x} {y}' fill='none' stroke='{stroke}'/>"));
    }
}

#[tessera]
fn marker(number: usize, side: bool, dark: bool, x: f64, y: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            at(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.inverse_surface, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!(
                        "{} {} sheet color marker {number}",
                        if dark { "Dark" } else { "Light" },
                        if side { "side" } else { "bottom" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(16.0, 24.0))
                .color(scheme.inverse_on_surface);
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "sheets_color_tests.rs"]
mod tests;
