use std::fmt::Write as _;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    flow_row::flow_row,
    icon_button::{
        IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button,
    },
    material_icons::filled,
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, RenderSlot, accesskit::Role, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::{
    super::{annotations, gap, srgb, subheading, type_style},
    SIZES, context,
};

#[path = "icon_button_measurement_table.rs"]
mod radius_table;

#[tessera]
pub(super) fn figures() {
    sizes();
    gap(Dp(24.0));
    caption(vec![
        "ui-a-extra-small",
        "ui-b-small",
        "ui-c-medium",
        "ui-d-large",
        "ui-e-extra-large",
    ]);
    gap(Dp(16.0));
    numbered_legend(vec![
        "ui-icon-size",
        "ui-default-width",
        "ui-narrow-width",
        "ui-wide-width",
    ]);
    gap(Dp(56.0));
    context::context_row(
        RenderSlot::new(move || {
            subheading("ui-target-sizes".into());
            gap(Dp(24.0));
            text(I18N.text("ui-extra-small-and-small-icon-buttons-must-have-a-target-size-of-48x48dp-or-larger-to-be-acce")).style(type_style(16.0,24.0));
        }),
        RenderSlot::new(|| {
            targets();
            gap(Dp(24.0));
            caption(vec![
                "ui-a-extra-small-icon-button-size",
                "ui-b-small-icon-button-size",
            ]);
            gap(Dp(16.0));
            numbered_legend(vec!["ui-narrow-width", "ui-default-width", "ui-wide-width"]);
        }),
    );
    gap(Dp(56.0));
    subheading("ui-button-corner-radius".into());
    gap(Dp(32.0));
    corners();
    gap(Dp(24.0));
    radius_table::figure();
}

#[tessera]
fn caption(items: Vec<&'static str>) {
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(4.0))
        .children(move || {
            for item in &items {
                text(I18N.text(*item)).style(type_style(16.0, 24.0));
            }
        });
}

#[tessera]
fn numbered_legend(items: Vec<&'static str>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    for (index, item) in items.into_iter().enumerate() {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(24.0), Dp(32.0))
                            .background_with_shape(
                                scheme.inverse_surface,
                                Shape::rounded_rectangle(Dp(12.0)),
                            ),
                    )
                    .children(move || {
                        text((index + 1).to_string())
                            .color(scheme.inverse_on_surface)
                            .style(type_style(16.0, 24.0));
                    });
                boxed().modifier(Modifier::new().size(Dp(12.0), Dp(1.0)));
                text(I18N.text(item)).style(type_style(14.0, 20.0));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn stage(width: f64, height: f64, label: String, child: RenderSlot) {
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            let label = label.clone();
            scaled_box((measured.get().0 / width) as f32).child(move || {
                boxed()
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(
                                scheme.surface,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(I18N.text(&(label))),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        grid(width, height);
                        child.render();
                    });
            });
        });
}

#[tessera]
fn grid(width: f64, height: f64) {
    let path = remember(move || {
        let mut path = String::new();
        for y in (14..height as usize).step_by(8) {
            for x in (10..width as usize).step_by(8) {
                write!(&mut path, "M{x} {y}h.8v.8h-.8z").unwrap();
            }
        }
        path
    });
    annotations::lines(
        Dp(width),
        Dp(height),
        format!(
            "<path d='{}' fill='#ada7b0' fill-opacity='.65'/>",
            path.get()
        ),
    );
}

#[tessera]
fn label(value: String, x: f64, y: f64, width: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(width), Dp(24.0)),
        )
        .children(move || {
            text(I18N.text(&(value))).style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn badge(value: String, x: f64, y: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x - 12.0), Dp(y - 12.0))
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
        )
        .children(move || {
            text(I18N.text(&(value)))
                .color(scheme.inverse_on_surface)
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn control(
    size: IconButtonSize,
    width: IconButtonWidth,
    variant: IconButtonVariant,
    shape: IconButtonShape,
    pressed: bool,
    x: f64,
    y: f64,
    name: String,
) {
    let actions = remember(|| 0_u32);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let outline = variant == IconButtonVariant::Outlined;
    let fixed_shape = if pressed {
        Shape::rounded_rectangle(size.pressed_radius())
    } else if shape == IconButtonShape::Square {
        Shape::rounded_rectangle(size.square_radius())
    } else {
        Shape::CAPSULE
    };
    let target_width = size.width(width).0.max(48.0);
    let target_height = size.height().0.max(48.0);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x - target_width * 0.5), Dp(y - target_height * 0.5))
                .size(Dp(target_width), Dp(target_height)),
        )
        .children(move || {
            if outline {
                // Diagram outlines describe geometry with a uniform fine rule;
                // keep them on visual bounds, not the expanded touch target.
                boxed().modifier(
                    Modifier::new()
                        .size(size.width(width), size.height())
                        .border_with_shape(Dp(1.0), scheme.outline_variant, fixed_shape),
                );
            }
            icon_button()
                .size(size)
                .width(width)
                .variant(if outline {
                    IconButtonVariant::Standard
                } else {
                    variant
                })
                .shape_style(shape)
                .shape(fixed_shape)
                .presentation_state(InteractionVisualState::Rest)
                .icon(filled::EDIT_SVG)
                .accessibility_label(I18N.text(&(name)))
                .accessibility_description(
                    I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
                )
                .on_click(move || actions.set(actions.get() + 1));
        });
}

fn horizontal(svg: &mut String, x: f64, y: f64, width: f64) {
    write!(svg,"<path d='M{x} {y}h{width}M{x} {}v6M{} {}v6' fill='none' stroke='#ff6277' stroke-width='.75'/>",y-3.0,x+width,y-3.0).unwrap();
}

fn vertical(svg: &mut String, x: f64, y: f64, height: f64) {
    write!(svg,"<path d='M{x} {y}v{height}M{} {y}h6M{} {}h6' fill='none' stroke='#ff6277' stroke-width='.75'/>",x-3.0,x-3.0,y+height).unwrap();
}

#[tessera]
fn dimensions(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    baseline: f64,
    canvas_width: f64,
    canvas_height: f64,
    guide_x: Option<f64>,
) {
    let guide_x = guide_x.unwrap_or(x + width * 0.5 + 8.0);
    let mut svg = String::new();
    horizontal(&mut svg, x - width * 0.5, baseline, width);
    vertical(&mut svg, guide_x, y - height * 0.5, height);
    annotations::lines(Dp(canvas_width), Dp(canvas_height), svg);
    label(format!("{width:.0}"), x - 24.0, baseline + 4.0, 48.0);
    label(format!("{height:.0}"), guide_x + 3.0, y - 12.0, 36.0);
}

#[tessera]
fn sizes() {
    stage(
        1024.0,
        942.0,
        "ui-icon-button-visual-measurements".into(),
        RenderSlot::new(move || {
            for (index, (x, title)) in [
                (209.0, "ui-icon"),
                (411.0, "ui-default-width"),
                (615.0, "ui-narrow-width"),
                (833.0, "ui-wide-width"),
            ]
            .into_iter()
            .enumerate()
            {
                badge((index + 1).to_string(), x, 57.0);
                label(title.into(), x - 80.0, 91.0, 160.0);
            }
            for (index, (size, y)) in SIZES
                .into_iter()
                .zip([149.0, 287.0, 434.0, 606.0, 818.0])
                .enumerate()
            {
                badge(["A", "B", "C", "D", "E"][index].into(), 79.0, y);
                control(
                    size,
                    IconButtonWidth::Uniform,
                    IconButtonVariant::Outlined,
                    IconButtonShape::Round,
                    false,
                    209.0,
                    y,
                    I18N.format(
                        "ui-measure-value-icon",
                        [("v0", format!("{:?}", size).into())],
                    ),
                );
                let icon = size.icon_size().0;
                let mut svg = String::new();
                for (x, y, length, vertical) in [
                    (209.0 - icon * 0.5, y - icon * 0.5, icon, false),
                    (209.0 - icon * 0.5, y + icon * 0.5, icon, false),
                    (209.0 - icon * 0.5, y - icon * 0.5, icon, true),
                    (209.0 + icon * 0.5, y - icon * 0.5, icon, true),
                ] {
                    annotations::dotted(&mut svg, x, y, length, vertical, "#ff6277");
                }
                annotations::lines(Dp(1024.0), Dp(942.0), svg);
                let baseline = y + size.height().0 * 0.5 + 14.0;
                dimensions(209.0, y, icon, icon, baseline, 1024.0, 942.0)
                    .guide_x(209.0 + (size.height().0 * 0.5).max(22.0) + 8.0);
                for (column, (width, x)) in [
                    (IconButtonWidth::Uniform, 411.0),
                    (IconButtonWidth::Narrow, 615.0),
                    (IconButtonWidth::Wide, 833.0),
                ]
                .into_iter()
                .enumerate()
                {
                    control(
                        size,
                        width,
                        if column == 0 {
                            IconButtonVariant::Filled
                        } else {
                            IconButtonVariant::FilledTonal
                        },
                        IconButtonShape::Round,
                        false,
                        x,
                        y,
                        I18N.format(
                            "ui-measure-value-value",
                            [
                                ("v0", format!("{:?}", size).into()),
                                ("v1", format!("{:?}", width).into()),
                            ],
                        ),
                    );
                    dimensions(
                        x,
                        y,
                        size.width(width).0,
                        size.height().0,
                        baseline,
                        1024.0,
                        942.0,
                    );
                }
            }
        }),
    );
}

#[tessera]
fn targets() {
    stage(
        676.0,
        300.0,
        "ui-icon-button-touch-targets".into(),
        RenderSlot::new(move || {
            for (row, (size, y)) in [
                (IconButtonSize::ExtraSmall, 53.0),
                (IconButtonSize::Small, 161.0),
            ]
            .into_iter()
            .enumerate()
            {
                badge(["A", "B"][row].into(), 122.0, y);
                for (column, (width, x)) in [
                    (IconButtonWidth::Narrow, 188.0),
                    (IconButtonWidth::Uniform, 338.0),
                    (IconButtonWidth::Wide, 488.0),
                ]
                .into_iter()
                .enumerate()
                {
                    let target = size.width(width).0.max(48.0);
                    boxed().modifier(
                        Modifier::new()
                            .offset(Dp(x - target * 0.5), Dp(y - 24.0))
                            .size(Dp(target), Dp(48.0))
                            .background(srgb(254, 209, 228)),
                    );
                    control(
                        size,
                        width,
                        IconButtonVariant::Filled,
                        IconButtonShape::Round,
                        false,
                        x,
                        y,
                        I18N.format(
                            "ui-target-value-value",
                            [
                                ("v0", format!("{:?}", size).into()),
                                ("v1", format!("{:?}", width).into()),
                            ],
                        ),
                    );
                    dimensions(x, y, target, 48.0, y + 32.0, 676.0, 300.0);
                    if row == 1 {
                        badge((column + 1).to_string(), x, 246.0);
                    }
                }
            }
        }),
    );
}

#[tessera]
fn corners() {
    stage(
        1024.0,
        626.0,
        "ui-icon-button-corner-radii".into(),
        RenderSlot::new(move || {
            for (row, y) in [107.0, 303.0, 498.0].into_iter().enumerate() {
                badge(["A", "B", "C"][row].into(), 124.0, y);
                for (size, x) in SIZES.into_iter().zip([242.0, 360.0, 492.0, 648.0, 844.0]) {
                    control(
                        size,
                        IconButtonWidth::Uniform,
                        IconButtonVariant::Outlined,
                        if row == 1 {
                            IconButtonShape::Square
                        } else {
                            IconButtonShape::Round
                        },
                        row == 2,
                        x,
                        y,
                        I18N.format(
                            "ui-corner-row-value-value",
                            [("v0", (row).into()), ("v1", format!("{:?}", size).into())],
                        ),
                    );
                    if row == 0 {
                        continue;
                    }
                    let radius = if row == 1 {
                        size.square_radius().0
                    } else {
                        size.pressed_radius().0
                    };
                    let left = x - size.height().0 * 0.5;
                    let top = y - size.height().0 * 0.5;
                    let end = left + radius;
                    let baseline = top - 36.0;
                    let mut svg = format!(
                        "<path d='M{end} {top}v{radius}h-{radius}A{radius} {radius} 0 0 1 {end} {top}Z' fill='#fed1e4'/>"
                    );
                    annotations::dotted(
                        &mut svg,
                        left + radius * 0.25,
                        baseline + 20.0,
                        top + radius * 0.25 - baseline - 20.0,
                        true,
                        "#ff6277",
                    );
                    annotations::lines(Dp(1024.0), Dp(626.0), svg);
                    label(
                        format!("{radius:.0}"),
                        left + radius * 0.25 - 24.0,
                        baseline,
                        48.0,
                    );
                }
            }
        }),
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_measurements_tests.rs"]
mod tests;
