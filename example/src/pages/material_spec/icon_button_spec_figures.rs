//! Native icon-button configuration and shape-morph figures.
//!
//! ## Usage
//!
//! Compare configurations and interact with pressed and selected shapes.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    flow_row::flow_row,
    icon_button::{
        IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button,
    },
    material_icons::{filled, outlined},
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::pages::material_spec::{
    gap, icon_buttons::context, media, srgb, subheading, type_style,
};

#[tessera]
pub(super) fn configs() {
    let seed = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .color_scheme
        .primary;
    material_theme()
        .theme(move || MaterialTheme::from_seed(seed, true))
        .child(|| {
            stage(
                1024.0,
                448.0,
                "Icon button configurations".into(),
                srgb(31, 31, 33),
                RenderSlot::new(|| {
                    for (size, x) in [
                        (IconButtonSize::ExtraSmall, 62.0),
                        (IconButtonSize::Small, 119.0),
                        (IconButtonSize::Medium, 190.0),
                        (IconButtonSize::Large, 284.0),
                        (IconButtonSize::ExtraLarge, 422.0),
                    ] {
                        control(
                            size,
                            IconButtonWidth::Uniform,
                            IconButtonVariant::Filled,
                            IconButtonShape::Round,
                            InteractionVisualState::Rest,
                            false,
                            false,
                            false,
                            0,
                            x,
                            220.0,
                            format!("Play {size:?}"),
                        );
                    }
                    for (shape, y) in [
                        (IconButtonShape::Round, 164.0),
                        (IconButtonShape::Square, 284.0),
                    ] {
                        control(
                            IconButtonSize::Medium,
                            IconButtonWidth::Uniform,
                            IconButtonVariant::Outlined,
                            shape,
                            InteractionVisualState::Rest,
                            false,
                            false,
                            false,
                            1,
                            570.0,
                            y,
                            format!("Share {shape:?}"),
                        );
                    }
                    for (variant, y) in [
                        (IconButtonVariant::Filled, 100.0),
                        (IconButtonVariant::FilledTonal, 180.0),
                        (IconButtonVariant::Outlined, 260.0),
                        (IconButtonVariant::Standard, 340.0),
                    ] {
                        control(
                            IconButtonSize::Medium,
                            IconButtonWidth::Uniform,
                            variant,
                            IconButtonShape::Round,
                            InteractionVisualState::Rest,
                            false,
                            false,
                            false,
                            2,
                            730.0,
                            y,
                            format!("Video {variant:?}"),
                        );
                    }
                    for (width, y) in [
                        (IconButtonWidth::Narrow, 140.0),
                        (IconButtonWidth::Uniform, 240.0),
                        (IconButtonWidth::Wide, 340.0),
                    ] {
                        control(
                            IconButtonSize::Medium,
                            width,
                            IconButtonVariant::FilledTonal,
                            IconButtonShape::Round,
                            InteractionVisualState::Rest,
                            false,
                            false,
                            false,
                            3,
                            908.0,
                            y,
                            format!("More options {width:?}"),
                        );
                    }
                    for (index, x) in [260.0, 570.0, 730.0, 908.0].into_iter().enumerate() {
                        marker((index + 1).to_string(), x, 410.0);
                    }
                }),
            );
        });
    gap(Dp(24.0));
    legend(vec![
        "Five sizes".into(),
        "Two shapes".into(),
        "Four color styles".into(),
        "Three widths".into(),
    ]);
}

#[tessera]
pub(super) fn shape_morph() {
    context::context_row(
        RenderSlot::new(|| {
            subheading("Pressed state".into());
            gap(Dp(24.0));
            paragraph("When pressed, round and square icon buttons morph toward a square shape. Both shapes use the same pressed corner radius for a given size. The corner radius changes with button size.".into());
        }),
        RenderSlot::new(|| {
            morph_grid(false);
        }),
    );
    gap(Dp(56.0));
    context::context_row(
        RenderSlot::new(|| {
            subheading("When selected".into());
            gap(Dp(24.0));
            paragraph("When selected, a round icon button changes to a square shape, and a square icon button changes to a round shape. The change in shape reinforces the selected state.".into());
        }),
        RenderSlot::new(|| {
            morph_grid(true);
        }),
    );
}

#[tessera]
fn morph_grid(selection: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    stage(
        676.0,
        296.0,
        if selection {
            "Selected icon button shapes".into()
        } else {
            "Pressed icon button shapes".into()
        },
        scheme.surface_container_low,
        RenderSlot::new(move || {
            let columns = if selection { 4 } else { 3 };
            let step = if selection { 140.0 } else { 176.0 };
            let first = if selection { 180.0 } else { 210.0 };
            for column in 0..columns {
                marker((column + 1).to_string(), first + column as f64 * step, 34.0);
            }
            for (row, shape) in [IconButtonShape::Round, IconButtonShape::Square]
                .into_iter()
                .enumerate()
            {
                let y = 108.0 + row as f64 * 116.0;
                marker(if row == 0 { "A".into() } else { "B".into() }, 54.0, y);
                for column in 0..columns {
                    control(
                        IconButtonSize::Small,
                        IconButtonWidth::Uniform,
                        IconButtonVariant::Outlined,
                        shape,
                        if column == 1 {
                            InteractionVisualState::Hover
                        } else {
                            InteractionVisualState::Rest
                        },
                        column == 2,
                        selection && column == 3,
                        selection && column == 3,
                        4,
                        first + column as f64 * step,
                        y,
                        format!(
                            "{} {shape:?} {}",
                            if selection {
                                "Selected shapes"
                            } else {
                                "Pressed shapes"
                            },
                            ["Enabled", "Hovered", "Pressed", "Selected"][column]
                        ),
                    );
                }
            }
        }),
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(8.0))
        .children(|| {
            paragraph("A. Round button".into());
            paragraph("B. Square button".into());
        });
    gap(Dp(16.0));
    let mut items = vec!["Enabled".into(), "Hovered".into(), "Pressed".into()];
    if selection {
        items.push("Selected".into());
    }
    legend(items);
}

#[tessera]
fn stage(width: f64, height: f64, label: String, background: Color, child: RenderSlot) {
    media::load_fonts();
    let measured = remember(move || Dp(width));
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
                            .background_with_shape(background, Shape::rounded_rectangle(Dp(32.0)))
                            .clip_shape(Shape::rounded_rectangle(Dp(32.0)))
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(label.clone()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children_shared(child);
            });
        });
}

#[tessera]
fn control(
    size: IconButtonSize,
    width: IconButtonWidth,
    variant: IconButtonVariant,
    shape: IconButtonShape,
    visual: InteractionVisualState,
    pressed: bool,
    initially_selected: bool,
    toggle: bool,
    icon_kind: usize,
    x: f64,
    y: f64,
    label: String,
) {
    let actions = remember(|| 0_u32);
    let selected = remember(move || initially_selected);
    let is_selected = toggle && selected.get();
    let fixed_shape = if pressed {
        Shape::rounded_rectangle(size.pressed_radius())
    } else if (shape == IconButtonShape::Round) != is_selected {
        Shape::CAPSULE
    } else {
        Shape::rounded_rectangle(size.square_radius())
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
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
            let button = icon_button()
                .size(size)
                .width(width)
                .variant(if is_selected {
                    IconButtonVariant::Filled
                } else {
                    variant
                })
                .shape_style(shape)
                .shape(fixed_shape)
                .presentation_state(visual)
                .selected_optional(if toggle { Some(is_selected) } else { None })
                .icon(match icon_kind {
                    0 => filled::PLAY_ARROW_SVG,
                    1 => filled::SHARE_SVG,
                    2 => filled::VIDEOCAM_SVG,
                    3 => filled::MORE_VERT_SVG,
                    _ => {
                        if is_selected {
                            filled::EDIT_SVG
                        } else {
                            outlined::EDIT_SVG
                        }
                    }
                })
                .accessibility_label(label.clone())
                .accessibility_description(format!("Activations: {}", actions.get()))
                .on_click(move || {
                    actions.set(actions.get() + 1);
                    if toggle {
                        selected.set(!selected.get());
                    }
                });
            let button = if is_selected {
                button
                    .color(scheme.inverse_surface)
                    .content_color(scheme.inverse_on_surface)
            } else {
                button
            };
            drop(button);
        });
}

#[tessera]
fn marker(value: String, x: f64, y: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x - 12.0), Dp(y - 12.0))
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(scheme.inverse_surface, Shape::CAPSULE),
        )
        .children(move || {
            text()
                .content(value.clone())
                .color(scheme.inverse_on_surface)
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn paragraph(value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn legend(items: Vec<String>) {
    for (index, value) in items.iter().enumerate() {
        let value = value.clone();
        row().children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .size(Dp(24.0), Dp(24.0))
                        .background_with_shape(
                            use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .color_scheme
                                .inverse_surface,
                            Shape::CAPSULE,
                        ),
                )
                .children(move || {
                    text()
                        .content((index + 1).to_string())
                        .color(
                            use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .color_scheme
                                .inverse_on_surface,
                        )
                        .style(type_style(16.0, 24.0));
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text().content(value.clone()).style(type_style(16.0, 24.0));
        });
        gap(Dp(8.0));
    }
}
