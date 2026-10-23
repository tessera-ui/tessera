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
    material_icons::filled,
    modifier::ModifierExt as _,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    tooltip::{TooltipController, TooltipPlacement, tooltip_anchor, tooltip_provider},
};
use tessera_ui::{Dp, Modifier, RenderSlot, remember, tessera, use_context};

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

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
    );
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
    panel(RenderSlot::new(|| {
        column().children(|| {
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
            gap(Dp(24.0));
            text().content("1  Icon");
            text().content("2  Container");
        });
    }));
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
    text()
        .content("Hover, focus with Tab, or press and hold to inspect the real state layers.")
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    subheading("Enabled".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        style_examples(true);
    }));
    gap(Dp(24.0));
    subheading("Disabled".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        style_examples(false);
    }));
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
