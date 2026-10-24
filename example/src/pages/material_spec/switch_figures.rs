//! Native switch anatomy and icon configuration illustrations.
//!
//! ## Usage
//!
//! Inspect switch parts and independently toggle icon configurations.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    switch::switch,
    text::text,
    theme::{MaterialTheme, TextStyle, material_theme},
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use super::super::{ACCENT, annotations, gap, type_style};

#[tessera]
pub(super) fn anatomy() {
    responsive(false);
    legend(false);
}

#[tessera]
pub(super) fn configurations() {
    responsive(true);
    legend(true);
}

#[tessera]
fn responsive(configuration: bool) {
    let canonical = if configuration { 676.0 } else { 1024.0 };
    let width = remember(move || Dp(canonical));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if next.0 > 0.0 && next != width.get() {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box((width.get().0 / canonical) as f32).child(move || {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, configuration))
                    .child(move || {
                        artboard(configuration);
                    });
            });
        });
}

#[tessera]
fn artboard(configuration: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .required_size(Dp(if configuration { 676.0 } else { 1024.0 }), Dp(300.0))
                .background(scheme.surface)
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(if configuration {
                        "Switch configurations diagram".into()
                    } else {
                        "Switch anatomy diagram".into()
                    }),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            if configuration {
                for row in 0..3 {
                    marker(row + 1, Dp(200.0), Dp(74.0 + row as f64 * 64.0), true);
                    for selected in [false, true] {
                        control(true, row, selected);
                    }
                }
            } else {
                control(false, 0, false);
                control(false, 0, true);
                anatomy_leaders();
                marker(1, Dp(396.0), Dp(72.0), false);
                marker(2, Dp(384.0), Dp(196.0), false);
                marker(3, Dp(616.0), Dp(72.0), false);
            }
        });
}

#[tessera]
fn control(configuration: bool, row: usize, selected: bool) {
    let checked = remember(move || selected);
    let (left, top, icon_mode, label) = if configuration {
        (
            if selected { 424.0 } else { 292.0 },
            62.0 + row as f64 * 64.0,
            row,
            format!(
                "{}: {}",
                [
                    "Without icons",
                    "Icon when selected",
                    "Icons in both states"
                ][row],
                if selected { "selected" } else { "unselected" }
            ),
        )
    } else {
        (
            if selected { 592.0 } else { 378.0 },
            126.0,
            usize::from(selected),
            format!(
                "Anatomy {} switch",
                if selected { "selected" } else { "unselected" }
            ),
        )
    };
    switch()
        .modifier(Modifier::new().offset(Dp(left), Dp(top)))
        .checked(checked.get())
        .accessibility_label(label)
        .on_toggle(move |value| checked.set(value))
        .child_optional(if icon_mode == 2 || (icon_mode == 1 && checked.get()) {
            Some(RenderSlot::new(move || {
                icon()
                    .painter(if checked.get() {
                        filled::CHECK_SVG
                    } else {
                        filled::CLOSE_SVG
                    })
                    .size(Dp(16.0));
            }))
        } else {
            None
        });
}

#[tessera]
fn anatomy_leaders() {
    let mut svg = String::from(
        "<path d='M408 104V138.5M396 161.5V196' fill='none' stroke='#322f35' stroke-width='1'/><circle cx='408' cy='142' r='3.5' fill='none' stroke='#322f35' stroke-width='1'/><circle cx='396' cy='158' r='3.5' fill='none' stroke='#322f35' stroke-width='1'/>",
    );
    annotations::dotted(&mut svg, 627.5, 104.0, 38.0, true, "#322f35");
    for (x, y, length, vertical) in [
        (620.0, 142.0, 16.0, false),
        (620.0, 158.0, 16.0, false),
        (620.0, 142.0, 16.0, true),
        (636.0, 142.0, 16.0, true),
    ] {
        annotations::dotted(&mut svg, x, y, length, vertical, "#322f35");
    }
    boxed()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .children(move || {
            annotations::lines(Dp(1024.0), Dp(300.0), svg.clone());
        });
}

#[tessera]
fn marker(number: usize, x: Dp, y: Dp, circular: bool) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let shape = if circular {
        Shape::Ellipse
    } else {
        Shape::CAPSULE
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(if circular { 24.0 } else { 32.0 }))
                .background_with_shape(scheme.surface_container, shape)
                .border_with_shape(Dp(1.0), scheme.on_surface, shape)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(format!(
                        "Switch {} marker {number}",
                        if circular { "configuration" } else { "anatomy" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(TextStyle {
                    font_size: Dp(16.0),
                    line_height: Some(Dp(24.0)),
                    font_family: Some("Roboto"),
                    ..Default::default()
                })
                .color(scheme.on_surface);
        });
}

#[tessera]
fn legend(configuration: bool) {
    gap(Dp(16.0));
    column().children(move || {
        for (index, label) in if configuration {
            [
                "Without icons",
                "Icon when selected",
                "Icons in both states",
            ]
        } else {
            ["Track", "Handle", "Icon"]
        }
        .into_iter()
        .enumerate()
        {
            text()
                .content(format!("{}  {label}", index + 1))
                .style(type_style(16.0, 28.0));
        }
    });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "switch_figures_tests.rs"]
mod tests;
