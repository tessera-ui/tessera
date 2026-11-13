//! Native switch anatomy and icon configuration illustrations.
//!
//! ## Usage
//!
//! Inspect switch parts and independently toggle icon configurations.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    md3::{
        icon::icon,
        material_icons::filled,
        switch::switch,
        text::text,
        theme::{MaterialTheme, TextStyle, material_theme},
    },
    modifier::ModifierExt as _,
    shape_def::Shape,
};
use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

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
                    label: Some(I18N.text(if configuration {
                        "ui-switch-configurations-diagram"
                    } else {
                        "ui-switch-anatomy-diagram"
                    })),
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
            I18N.format(
                "ui-value-value-format-1",
                [
                    (
                        "v0",
                        I18N.text(
                            &([
                                "ui-without-icons",
                                "ui-icon-when-selected",
                                "ui-icons-in-both-states",
                            ][row]),
                        )
                        .into(),
                    ),
                    (
                        "v1",
                        I18N.text(
                            &(if selected {
                                "ui-selected-7075"
                            } else {
                                "ui-unselected-7076"
                            }),
                        )
                        .into(),
                    ),
                ],
            ),
        )
    } else {
        (
            if selected { 592.0 } else { 378.0 },
            126.0,
            usize::from(selected),
            I18N.format(
                "ui-anatomy-value-switch",
                [(
                    "v0",
                    I18N.text(
                        &(if selected {
                            "ui-selected-7075"
                        } else {
                            "ui-unselected-7076"
                        }),
                    )
                    .into(),
                )],
            ),
        )
    };
    switch(checked.get())
        .modifier(Modifier::new().offset(Dp(left), Dp(top)))
        .accessibility_label(I18N.text(label))
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
                    label: Some(
                        I18N.format(
                            "ui-switch-value-marker-value",
                            [
                                (
                                    "v0",
                                    I18N.text(
                                        &(if circular {
                                            "ui-configuration-7109"
                                        } else {
                                            "ui-anatomy-7072"
                                        }),
                                    )
                                    .into(),
                                ),
                                ("v1", (number).into()),
                            ],
                        ),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text((number).to_string())
                .style(TextStyle {
                    font_size: Dp(16.0),
                    line_height: Some(Dp(24.0)),
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
                "ui-without-icons",
                "ui-icon-when-selected",
                "ui-icons-in-both-states",
            ]
        } else {
            ["ui-track", "ui-handle", "ui-icon"]
        }
        .into_iter()
        .enumerate()
        {
            text(I18N.format(
                "ui-value-value",
                [("v0", (index + 1).into()), ("v1", I18N.text(&label).into())],
            ))
            .style(type_style(16.0, 28.0));
        }
    });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "switch_figures_tests.rs"]
mod tests;
