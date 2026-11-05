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
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    ACCENT,
    action_articles::{Kind, action_article},
    annotations::{self, Geometry, dotted, relative, watch},
    article::SpecSection,
    gap, panel, spec_table, subheading, type_style,
};

#[path = "icon_button_context.rs"]
pub(super) mod context;

#[path = "icon_button_measurements.rs"]
mod measurements;

#[path = "icon_button_baseline.rs"]
mod baseline;

#[path = "icon_button_tokens.rs"]
mod token_sets;

#[path = "icon_button_spec_figures.rs"]
mod figures;

#[path = "icon_button_articles.rs"]
pub(super) mod articles;

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

fn name(style: IconButtonVariant) -> &'static str {
    match style {
        IconButtonVariant::Filled => "ui-filled",
        IconButtonVariant::FilledTonal => "ui-tonal",
        IconButtonVariant::Outlined => "ui-outlined",
        IconButtonVariant::Standard => "ui-standard",
    }
}

#[tessera]
pub(super) fn icon_buttons_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    action_article(
        Kind::Icons,
        vec![
            SpecSection {
                title: "ui-variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "ui-configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "ui-shape-morph",
                content: RenderSlot::new(shape_morph),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements::figures),
            },
            SpecSection {
                title: "ui-baseline-tokens",
                content: RenderSlot::new(baseline::figure),
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
    .route_optional(route.clone())
    .overview(|| {
        articles::overview();
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
        .accessibility_label(I18N.text(label))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
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
                .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                    let next = Dp::from_pixels_f32(size.width.to_f32()).0;
                    if width.is_alive() {
                        width.set_if_changed(next);
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
    text(I18N.text(
        "ui-standard-filled-unselected-filled-selected-filled-tonal-and-outlined-icon-buttons",
    ))
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
            "ui-more-options",
        ),
        1 | 2 => (
            IconButtonVariant::Filled,
            IconButtonSize::Large,
            IconButtonWidth::Uniform,
            "ui-settings",
        ),
        3 => (
            IconButtonVariant::FilledTonal,
            IconButtonSize::Medium,
            IconButtonWidth::Wide,
            "ui-download",
        ),
        _ => (
            IconButtonVariant::Outlined,
            IconButtonSize::Medium,
            IconButtonWidth::Narrow,
            "ui-delete",
        ),
    };
    icon_button()
        .variant(style)
        .size(size)
        .width(width)
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
        .accessibility_label(I18N.format(
            "ui-overview-value-value",
            [("v0", I18N.text(&label).into()), ("v1", (index + 1).into())],
        ))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
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
                tooltip_provider(I18N.text("ui-add-to-favorites"))
                    .controller(controller)
                    .placement(TooltipPlacement::Above)
                    .child(move |_| {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().fill_max_width().height(Dp(180.0)))
                            .children(move || {
                                tooltip_anchor(controller).child(move || {
                                    icon_button()
                                        .standard()
                                        .icon(filled::FAVORITE_SVG)
                                        .accessibility_label(I18N.text("ui-hover-favorite-action"))
                                        .on_click(move || count.set(count.get() + 1));
                                });
                            });
                    });
                text(I18N.format(
                    "ui-hover-example-actions-value",
                    [("v0", (count.get()).into())],
                ))
                .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(64.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                column().children(move || {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Medium,
                        IconButtonWidth::Uniform,
                        IconButtonShape::Round,
                        false,
                        false,
                        true,
                        "ui-default-icon-action",
                    );
                    gap(Dp(16.0));
                    text(I18N.text("ui-1-default-icon-button"));
                });
                column().children(move || {
                    sample(
                        IconButtonVariant::Filled,
                        IconButtonSize::Medium,
                        IconButtonWidth::Uniform,
                        IconButtonShape::Round,
                        true,
                        false,
                        true,
                        "ui-toggle-favorite",
                    );
                    gap(Dp(16.0));
                    text(I18N.text("ui-2-toggle-icon-button"));
                });
            });
    }));
}

#[tessera]
fn configurations() {
    figures::configs();
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-category".into(),
            "ui-options".into(),
            "M3".into(),
            "M3 Expressive".into(),
        ],
        vec![
            vec![
                "ui-size".into(),
                "ui-small-default".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-size".into(),
                "ui-extra-small-medium-large-extra-large".into(),
                "—".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-shape".into(),
                "ui-round-default".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-shape".into(),
                "ui-square".into(),
                "—".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-color-style".into(),
                "ui-filled-default-tonal-outlined-standard".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-width".into(),
                "ui-default".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-width".into(),
                "ui-narrow-wide".into(),
                "—".into(),
                "ui-available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    token_sets::figure();
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
                            "ui-anatomy-add-action",
                        );
                    });
                    boxed()
                        .modifier(Modifier::new().fill_max_size())
                        .children(move || {
                            icon_anatomy_callouts(geometry);
                        });
                });
            gap(Dp(24.0));
            text(I18N.text("ui-1-icon"));
            text(I18N.text("ui-2-container"));
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
                        (false, false, "ui-default"),
                        (true, false, "ui-toggle-off"),
                        (true, true, "ui-toggle-on"),
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
                                I18N.format(
                                    "ui-value-value-format-2",
                                    [
                                        ("v0", I18N.text(&name(style)).into()),
                                        ("v1", I18N.text(&label).into()),
                                    ],
                                ),
                            );
                            gap(Dp(12.0));
                            text(I18N.text(label)).style(type_style(16.0, 24.0));
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
                        subheading(
                            if dark {
                                "ui-dark-theme"
                            } else {
                                "ui-light-theme"
                            }
                            .to_owned(),
                        );
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
        subheading(I18N.format(
            "ui-value-button-states",
            [("v0", I18N.text(&name(style)).into())],
        ));
        for toggle in [false, true] {
            gap(Dp(24.0));
            subheading(if toggle { "ui-toggle" } else { "ui-default" }.into());
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
                .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                    let next = Dp::from_pixels_f32(size.width.to_f32()).0;
                    if width.is_alive() {
                        width.set_if_changed(next);
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
                                                    I18N.format(
                                                        "ui-value-value-value-value-value-icon",
                                                        [
                                                            (
                                                                "v0",
                                                                I18N.text(
                                                                    &(if dark {
                                                                        "ui-dark"
                                                                    } else {
                                                                        "ui-light"
                                                                    }),
                                                                )
                                                                .into(),
                                                            ),
                                                            ("v1", I18N.text(&name(style)).into()),
                                                            (
                                                                "v2",
                                                                I18N.text(
                                                                    &(if toggle {
                                                                        "ui-toggle-7073"
                                                                    } else {
                                                                        "ui-default-7074"
                                                                    }),
                                                                )
                                                                .into(),
                                                            ),
                                                            (
                                                                "v3",
                                                                I18N.text(
                                                                    &(if selected {
                                                                        "ui-selected-7075"
                                                                    } else {
                                                                        "ui-unselected-7076"
                                                                    }),
                                                                )
                                                                .into(),
                                                            ),
                                                            (
                                                                "v4",
                                                                I18N.text(
                                                                    &([
                                                                        "ui-enabled-7077",
                                                                        "ui-disabled-7078",
                                                                        "ui-hovered-7079",
                                                                        "ui-focused-7080",
                                                                        "ui-pressed-7081",
                                                                    ][state]),
                                                                )
                                                                .into(),
                                                            ),
                                                        ],
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
        text(I18N.text("ui-a-unselected-b-selected-3245")).style(type_style(16.0, 24.0));
    }
    for (index, label) in [
        "ui-enabled",
        "ui-disabled",
        "ui-hovered",
        "ui-focused",
        "ui-pressed",
    ]
    .into_iter()
    .enumerate()
    {
        text(I18N.format(
            "ui-value-value",
            [("v0", (index + 1).into()), ("v1", I18N.text(&label).into())],
        ))
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
            text(I18N.text(&(label)))
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
    let control = icon_button()
        .variant(style)
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
        .accessibility_label(I18N.text(label.get()))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
        .on_click(move || {
            actions.set(actions.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        });
    if state == 4 {
        control.presentation_ripple(RipplePresentation {
            origin: [0.73125, 0.97125],
            radius: 0.67375,
            opacity: 0.12,
        });
    }
}

#[tessera]
fn shape_morph() {
    figures::shape_morph();
}
