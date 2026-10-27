//! Interactive Expressive icon button specifications.
//!
//! ## Usage
//!
//! Compare compact actions, toggle selection, proportions and shape feedback.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    icon_button::{
        IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button,
    },
    material_icons::{filled, outlined},
    modifier::{InteractionVisualState, ModifierExt as _},
    shape_def::Shape,
    surface::{RipplePresentation, surface},
    text::text,
    theme::{MaterialTheme, material_theme},
    tooltip::{TooltipController, TooltipPlacement, tooltip_anchor, tooltip_provider},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, modifier::FocusModifierExt as _, remember, tessera,
    use_context,
};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    annotations::{self, Geometry, dotted, relative, watch},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

#[path = "icon_button_context.rs"]
pub(super) mod context;

const STYLES: [IconButtonVariant; 4] = [
    IconButtonVariant::Filled,
    IconButtonVariant::FilledTonal,
    IconButtonVariant::Outlined,
    IconButtonVariant::Standard,
];
const SIZES: [IconButtonSize; 5] = [
    IconButtonSize::ExtraSmall,
    IconButtonSize::Small,
    IconButtonSize::Medium,
    IconButtonSize::Large,
    IconButtonSize::ExtraLarge,
];
const WIDTHS: [IconButtonWidth; 3] = [
    IconButtonWidth::Narrow,
    IconButtonWidth::Uniform,
    IconButtonWidth::Wide,
];

fn name(style: IconButtonVariant) -> &'static str {
    match style {
        IconButtonVariant::Filled => "Filled",
        IconButtonVariant::FilledTonal => "Tonal",
        IconButtonVariant::Outlined => "Outlined",
        IconButtonVariant::Standard => "Standard",
    }
}

#[tessera]
pub(super) fn icon_buttons_spec() {
    action_article(
        Kind::Icons,
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Shape morph",
                content: RenderSlot::new(shape_morph),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
        RenderSlot::new(variants),
        RenderSlot::new(anatomy),
        RenderSlot::new(|| {
            hover_preview();
            gap(Dp(32.0));
            shape_morph();
        }),
    )
    .overview(|| {
        overview_art();
    });
}

#[tessera]
fn sample(
    style: IconButtonVariant,
    size: IconButtonSize,
    width: IconButtonWidth,
    shape: IconButtonShape,
    toggle: bool,
    initial: bool,
    enabled: bool,
    #[prop(into)] label: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    icon_button()
        .variant(style)
        .size(size)
        .width(width)
        .shape_style(shape)
        .selected_optional(if toggle { Some(selected.get()) } else { None })
        .enabled(enabled)
        .icon(if toggle {
            if selected.get() {
                filled::FAVORITE_SVG
            } else {
                filled::FAVORITE_BORDER_SVG
            }
        } else {
            filled::ADD_SVG
        })
        .accessibility_label(label)
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || {
            actions.set(actions.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        });
}

#[tessera]
fn overview_art() {
    let width = remember(|| 1024.0_f64);
    let scale = width.get() / 1024.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(400.0 * scale))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32()).0;
                    if width.is_alive() && width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            for (index, center) in [308.0, 408.0, 528.0, 636.0, 720.0].into_iter().enumerate() {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp((center - 80.0) * scale), Dp(120.0 * scale))
                            .size(Dp(160.0 * scale), Dp(160.0 * scale)),
                    )
                    .children(move || {
                        overview_icon(index);
                    });
            }
        });
    gap(Dp(24.0));
    text()
        .content(
            "Standard, filled unselected, filled selected, filled tonal, and outlined icon buttons",
        )
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn overview_icon(index: usize) {
    let selected = remember(|| index == 2);
    let actions = remember(|| 0_u32);
    let (style, size, width, label) = match index {
        0 => (
            IconButtonVariant::Standard,
            IconButtonSize::Small,
            IconButtonWidth::Uniform,
            "More options",
        ),
        1 | 2 => (
            IconButtonVariant::Filled,
            IconButtonSize::Large,
            IconButtonWidth::Uniform,
            "Settings",
        ),
        3 => (
            IconButtonVariant::FilledTonal,
            IconButtonSize::Medium,
            IconButtonWidth::Wide,
            "Download",
        ),
        _ => (
            IconButtonVariant::Outlined,
            IconButtonSize::Medium,
            IconButtonWidth::Narrow,
            "Delete",
        ),
    };
    icon_button()
        .variant(style)
        .size(size)
        .width(width)
        .shape_style(IconButtonShape::Round)
        .selected_optional(if index == 1 || index == 2 {
            Some(selected.get())
        } else {
            None
        })
        .icon(match index {
            0 => filled::MORE_VERT_SVG,
            1 | 2 => {
                if selected.get() {
                    filled::SETTINGS_SVG
                } else {
                    outlined::SETTINGS_SVG
                }
            }
            3 => filled::DOWNLOAD_SVG,
            _ => outlined::DELETE_SVG,
        })
        .accessibility_label(format!("Overview {label} {}", index + 1))
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || {
            actions.set(actions.get() + 1);
            if index == 1 || index == 2 {
                selected.set(!selected.get());
            }
        });
}

#[tessera]
fn hover_preview() {
    let controller = remember(TooltipController::new);
    let count = remember(|| 0_usize);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                tooltip_provider("Add to favorites")
                    .controller(controller)
                    .placement(TooltipPlacement::Above)
                    .child(move || {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().fill_max_width().height(Dp(180.0)))
                            .children(move || {
                                tooltip_anchor().controller(controller).child(move || {
                                    icon_button()
                                        .standard()
                                        .icon(filled::FAVORITE_SVG)
                                        .accessibility_label("Hover favorite action")
                                        .on_click(move || count.set(count.get() + 1));
                                });
                            });
                    });
                text()
                    .content(format!("Hover example actions: {}", count.get()))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                column().children(|| {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Medium,
                        IconButtonWidth::Uniform,
                        IconButtonShape::Round,
                        false,
                        false,
                        true,
                        "Default icon action",
                    );
                    gap(Dp(16.0));
                    text().content("1  Default icon button");
                });
                column().children(|| {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Medium,
                        IconButtonWidth::Uniform,
                        IconButtonShape::Round,
                        true,
                        false,
                        true,
                        "Toggle favorite",
                    );
                    gap(Dp(16.0));
                    text().content("2  Toggle icon button");
                });
            });
    }));
}

#[tessera]
fn sizes() {
    column().children(|| {
        for (index, size) in SIZES.into_iter().enumerate() {
            text()
                .content(["Extra small", "Small", "Medium", "Large", "Extra large"][index])
                .style(type_style(16.0, 24.0));
            gap(Dp(12.0));
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(16.0))
                .children(move || {
                    for width in WIDTHS {
                        sample(
                            IconButtonVariant::Filled,
                            size,
                            width,
                            IconButtonShape::Round,
                            false,
                            false,
                            true,
                            format!("{size:?} {width:?}"),
                        );
                    }
                });
            gap(Dp(32.0));
        }
    });
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(sizes));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(16.0))
            .children(|| {
                for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Medium,
                        IconButtonWidth::Uniform,
                        shape,
                        false,
                        false,
                        true,
                        format!("{shape:?} icon action"),
                    );
                }
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Category".into(), "Configurations".into()],
        vec![
            vec!["Size".into(), "XS, S (default), M, L, XL".into()],
            vec!["Shape".into(), "Round (default), square".into()],
            vec![
                "Color style".into(),
                "Filled, tonal, outlined, standard".into(),
            ],
            vec!["Width".into(), "Narrow, uniform (default), wide".into()],
        ],
    );
}

#[tessera]
fn tokens() {
    let style = remember(|| IconButtonVariant::Filled);
    flow_row().item_spacing(Dp(8.0)).children(move || {
        for variant in STYLES {
            tessera_components::button::button()
                .text()
                .accessibility_label(format!("{} icon tokens", name(variant)))
                .on_click(move || style.set(variant))
                .child(move || {
                    text().content(name(variant));
                });
        }
    });
    gap(Dp(24.0));
    let (
        default_container,
        default_icon,
        off_container,
        off_icon,
        selected_container,
        selected_icon,
    ) = match style.get() {
        IconButtonVariant::Filled => (
            "Primary",
            "On primary",
            "Surface container",
            "On surface variant",
            "Primary",
            "On primary",
        ),
        IconButtonVariant::FilledTonal => (
            "Secondary container",
            "On secondary container",
            "Secondary container",
            "On secondary container",
            "Secondary",
            "On secondary",
        ),
        IconButtonVariant::Outlined => (
            "Transparent · outline variant",
            "On surface variant",
            "Transparent · outline variant",
            "On surface variant",
            "Inverse surface",
            "Inverse on surface",
        ),
        IconButtonVariant::Standard => (
            "Transparent",
            "On surface variant",
            "Transparent",
            "On surface variant",
            "Transparent",
            "Primary",
        ),
    };
    spec_table(
        vec![
            "Element".into(),
            "Default".into(),
            "Toggle off".into(),
            "Toggle on".into(),
        ],
        vec![
            vec![
                "Container".into(),
                default_container.into(),
                off_container.into(),
                selected_container.into(),
            ],
            vec![
                "Icon".into(),
                default_icon.into(),
                off_icon.into(),
                selected_icon.into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    let geometry = remember(Geometry::new);
    panel(RenderSlot::new(move || {
        column().children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(watch(geometry, "figure").fill_max_width().height(Dp(224.0)))
                .children(move || {
                    boxed().modifier(watch(geometry, "container")).children(|| {
                        sample(
                            IconButtonVariant::Filled,
                            IconButtonSize::Large,
                            IconButtonWidth::Uniform,
                            IconButtonShape::Round,
                            false,
                            false,
                            true,
                            "Anatomy add action",
                        );
                    });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            icon_anatomy_callouts(geometry);
                        });
                });
            gap(Dp(24.0));
            text().content("1  Icon");
            text().content("2  Container");
        });
    }));
}

#[tessera]
fn icon_anatomy_callouts(geometry: State<Geometry>) {
    let values = geometry.get();
    let (Some(root), Some(container)) = (values.get("figure"), relative(&values, "container"))
    else {
        return;
    };
    let mut svg = String::new();
    let y = container.y + container.h * 0.5;
    let left = container.x - 64.0;
    let right = container.x + container.w + 40.0;
    dotted(
        &mut svg,
        left + 24.0,
        y,
        container.x + container.w * 0.5 - left - 24.0,
        false,
        "#49454f",
    );
    dotted(
        &mut svg,
        container.x + container.w,
        y,
        right - container.x - container.w,
        false,
        "#49454f",
    );
    annotations::marker(1, Dp(left), Dp(y - 16.0));
    annotations::marker(2, Dp(right), Dp(y - 16.0));
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn style_examples(enabled: bool) {
    column().children(move || {
        for style in STYLES {
            subheading(name(style).to_owned());
            gap(Dp(16.0));
            flow_row()
                .item_spacing(Dp(32.0))
                .line_spacing(Dp(16.0))
                .children(move || {
                    for (toggle, initial, label) in [
                        (false, false, "Default"),
                        (true, false, "Toggle off"),
                        (true, true, "Toggle on"),
                    ] {
                        column().children(move || {
                            sample(
                                style,
                                IconButtonSize::Small,
                                IconButtonWidth::Uniform,
                                IconButtonShape::Round,
                                toggle,
                                initial,
                                enabled,
                                format!("{} {label}", name(style)),
                            );
                            gap(Dp(12.0));
                            text().content(label).style(type_style(16.0, 24.0));
                        });
                    }
                });
            gap(Dp(32.0));
        }
    });
}

#[tessera]
fn scheme(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                .child(move || {
                    column().children(move || {
                        subheading(if dark { "Dark theme" } else { "Light theme" }.to_owned());
                        gap(Dp(24.0));
                        style_examples(true);
                    });
                });
        });
}

#[tessera]
fn colors() {
    scheme(false);
    gap(Dp(24.0));
    scheme(true);
}

#[tessera]
fn states() {
    for style in STYLES {
        subheading(format!("{} button states", name(style)));
        for toggle in [false, true] {
            gap(Dp(24.0));
            subheading(if toggle { "Toggle" } else { "Default" }.into());
            gap(Dp(24.0));
            icon_state_matrix(style, toggle);
        }
        gap(Dp(64.0));
    }
}

#[tessera]
fn icon_state_matrix(style: IconButtonVariant, toggle: bool) {
    let width = remember(|| 1024.0_f64);
    let scale = width.get() / 1024.0;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(600.0 * scale))
                .clip_to_bounds()
                .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(bounds.width.to_f32()).0;
                    if width.is_alive() && width.get() != next {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(move || {
                        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp::ZERO, Dp(if dark { 300.0 } else { 0.0 } * scale))
                                    .size(Dp(1024.0 * scale), Dp(300.0 * scale))
                                    .background(scheme.surface_container_low),
                            )
                            .children(move || {
                                for state in 0..5 {
                                    let center = if toggle {
                                        236.0 + state as f64 * 160.0
                                    } else {
                                        192.0 + state as f64 * 160.0
                                    };
                                    icon_state_marker(
                                        (state + 1).to_string(),
                                        center,
                                        if toggle {
                                            if dark { 58.0 } else { 74.0 }
                                        } else if dark {
                                            102.0
                                        } else {
                                            108.0
                                        },
                                        scale,
                                    );
                                    for selected in
                                        [false, true].into_iter().take(if toggle { 2 } else { 1 })
                                    {
                                        let mut center_y = if toggle {
                                            if selected { 208.0 } else { 140.0 }
                                        } else {
                                            174.0
                                        };
                                        if dark {
                                            center_y -= if toggle { 16.0 } else { 6.0 };
                                        }
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(
                                                Modifier::new()
                                                    .offset(
                                                        Dp((center - 40.0) * scale),
                                                        Dp((center_y - 40.0) * scale),
                                                    )
                                                    .size(Dp(80.0 * scale), Dp(80.0 * scale)),
                                            )
                                            .children(move || {
                                                icon_state_item(
                                                    style,
                                                    toggle,
                                                    selected,
                                                    state,
                                                    scale,
                                                    format!(
                                                        "{} {} {} {} {} icon",
                                                        if dark { "Dark" } else { "Light" },
                                                        name(style),
                                                        if toggle { "toggle" } else { "default" },
                                                        if selected {
                                                            "selected"
                                                        } else {
                                                            "unselected"
                                                        },
                                                        [
                                                            "enabled", "disabled", "hovered",
                                                            "focused", "pressed"
                                                        ][state]
                                                    ),
                                                );
                                            });
                                    }
                                }
                                if toggle {
                                    icon_state_marker(
                                        "A".into(),
                                        140.0,
                                        if dark { 124.0 } else { 140.0 },
                                        scale,
                                    );
                                    icon_state_marker(
                                        "B".into(),
                                        140.0,
                                        if dark { 192.0 } else { 208.0 },
                                        scale,
                                    );
                                }
                            });
                    });
            }
        });
    gap(Dp(24.0));
    if toggle {
        text()
            .content("A: Unselected. B: Selected")
            .style(type_style(16.0, 24.0));
    }
    for (index, label) in ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"]
        .into_iter()
        .enumerate()
    {
        text()
            .content(format!("{}  {label}", index + 1))
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn icon_state_marker(label: String, x: f64, y: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((x - 12.0) * scale), Dp((y - 12.0) * scale))
                .size(Dp(24.0 * scale), Dp(24.0 * scale))
                .background_with_shape(scheme.on_surface, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(label.clone())
                .style(type_style(14.0 * scale, 20.0 * scale))
                .color(scheme.surface);
        });
}

#[tessera]
fn icon_state_item(
    style: IconButtonVariant,
    toggle: bool,
    initial: bool,
    state: usize,
    scale: f64,
    label: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    let label = remember(|| label);
    let mut control = icon_button()
        .variant(style)
        .size(IconButtonSize::Small)
        .width(IconButtonWidth::Uniform)
        .shape_style(IconButtonShape::Round)
        .modifier(Modifier::new().size(Dp(40.0 * scale), Dp(40.0 * scale)))
        .selected_optional(if toggle { Some(selected.get()) } else { None })
        .enabled(state != 1)
        .presentation_state(match state {
            2 => InteractionVisualState::Hover,
            3 => InteractionVisualState::Focus,
            4 => InteractionVisualState::Pressed,
            _ => InteractionVisualState::Rest,
        })
        .icon(if toggle && !selected.get() {
            outlined::SETTINGS_SVG
        } else {
            filled::SETTINGS_SVG
        })
        .accessibility_label(label.get())
        .accessibility_description(format!("Activations: {}", actions.get()))
        .on_click(move || {
            actions.set(actions.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        });
    if state == 4 {
        control = control.presentation_ripple(RipplePresentation {
            origin: [0.73125, 0.97125],
            radius: 0.67375,
            opacity: 0.12,
        });
    }
    drop(control);
}

#[tessera]
fn shape_morph() {
    text()
        .content(
            "Press either shape. Toggle selection changes round to square, and square to round.",
        )
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for shape in [IconButtonShape::Round, IconButtonShape::Square] {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Large,
                        IconButtonWidth::Uniform,
                        shape,
                        true,
                        false,
                        true,
                        format!("Morph {shape:?}"),
                    );
                }
            });
    }));
}

#[tessera]
fn measurements() {
    spec_table(
        vec![
            "Size".into(),
            "Height / icon".into(),
            "Narrow / uniform / wide".into(),
        ],
        vec![
            vec!["XS".into(), "32 / 20dp".into(), "28 / 32 / 40dp".into()],
            vec!["S".into(), "40 / 24dp".into(), "32 / 40 / 52dp".into()],
            vec!["M".into(), "56 / 24dp".into(), "48 / 56 / 72dp".into()],
            vec!["L".into(), "96 / 32dp".into(), "64 / 96 / 128dp".into()],
            vec!["XL".into(), "136 / 40dp".into(), "104 / 136 / 184dp".into()],
        ],
    );
    gap(Dp(24.0));
    text()
        .content("Minimum target: 48 × 48dp. Round corners are fully rounded at rest.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    spec_table(
        vec![
            "Size".into(),
            "Square radius".into(),
            "Pressed radius".into(),
        ],
        vec![
            vec!["XS / S".into(), "12dp".into(), "8dp".into()],
            vec!["M".into(), "16dp".into(), "12dp".into()],
            vec!["L / XL".into(), "28dp".into(), "16dp".into()],
        ],
    );
}
