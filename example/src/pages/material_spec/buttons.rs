//! Interactive Expressive button specifications.
//!
//! ## Usage
//!
//! Compare button selection, sizes, shapes, colors and input feedback.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonShape, ButtonSize, ButtonVariant, button},
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::{Asset, filled},
    modifier::{InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, RenderSlotWith, accesskit::Role, remember, tessera,
    use_context,
};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

const STYLES: [ButtonVariant; 5] = [
    ButtonVariant::Elevated,
    ButtonVariant::Filled,
    ButtonVariant::Tonal,
    ButtonVariant::Outlined,
    ButtonVariant::Text,
];
const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const CAUTION_COLOR: Color = Color::from_rgb(0.705, 0.365, 0.0);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CELL: f64 = 120.0;

const SIZES: [ButtonSize; 5] = [
    ButtonSize::ExtraSmall,
    ButtonSize::Small,
    ButtonSize::Medium,
    ButtonSize::Large,
    ButtonSize::ExtraLarge,
];

fn style_name(style: ButtonVariant) -> &'static str {
    match style {
        ButtonVariant::Elevated => "ui-elevated",
        ButtonVariant::Filled => "ui-filled",
        ButtonVariant::Tonal => "ui-tonal",
        ButtonVariant::Outlined => "ui-outlined",
        ButtonVariant::Text => "ui-text",
    }
}

#[tessera]
pub(super) fn buttons_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-buttons",
        "ui-buttons-prompt-most-actions-in-a-ui",
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
                content: RenderSlot::new(|| {
                    anatomy(vec!["ui-container", "ui-label-text", "ui-icon-optional"]);
                }),
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
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-baseline-tokens",
                content: RenderSlot::new(baseline_tokens),
            },
        ],
    )
    .route_optional(route)
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "ui-m3-expressive-update",
                    content: RenderSlot::new(expressive),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "ui-toggle-buttons",
                    content: RenderSlot::new(toggle_buttons),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "ui-color-styles",
                    content: RenderSlot::new(color_styles),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard_navigation),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
pub(super) fn button_preview(
    style: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    toggle: bool,
    enabled: bool,
    #[prop(into)] label: String,
    initial_selected: Option<bool>,
    leading: Option<Asset>,
    presentation: Option<InteractionVisualState>,
    padding: Option<Padding>,
    modifier: Option<Modifier>,
) {
    let selected = remember(|| initial_selected.unwrap_or(false));
    let activations = remember(|| 0_u32);
    button()
        .variant(style)
        .size(size)
        .shape_style(shape)
        .selected_optional(if toggle && style != ButtonVariant::Text {
            Some(selected.get())
        } else {
            None
        })
        .enabled(enabled)
        .presentation_state_optional(presentation)
        .content_padding_optional(padding)
        .modifier_optional(modifier)
        .accessibility_label(I18N.text(&(label)))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (activations.get()).into())]),
        )
        .leading_icon_optional(if let Some(asset) = leading {
            Some(RenderSlot::new(move || {
                icon().painter(asset);
            }))
        } else {
            None
        })
        .on_click(move || {
            activations.set(activations.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        })
        .child(move || {
            text().content(I18N.text(&(label)));
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
                .height(Dp(300.0 * scale))
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
            for (index, style) in STYLES.into_iter().enumerate() {
                let center = [159.0, 342.0, 515.0, 699.0, 879.0][index];
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp((center - 80.0) * scale), Dp(76.0 * scale))
                            .size(Dp(160.0 * scale), Dp(80.0 * scale)),
                    )
                    .children(move || {
                        overview_button(style, index, scale);
                    });
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp((center - 20.0) * scale), Dp(168.0 * scale))
                            .size(Dp(40.0 * scale), Dp(40.0 * scale))
                            .background_with_shape(scheme.on_surface, Shape::Ellipse)
                            .semantics(SemanticsArgs {
                                role: Some(Role::Note),
                                label: Some(I18N.format(
                                    "ui-value-value-button",
                                    [
                                        ("v0", (index + 1).into()),
                                        ("v1", I18N.text(&style_name(style)).into()),
                                    ],
                                )),
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        text()
                            .content((index + 1).to_string())
                            .style(type_style(16.0 * scale, 24.0 * scale))
                            .color(scheme.surface);
                    });
            }
        });
    gap(Dp(24.0));
    legend(vec![
        "ui-elevated-button",
        "ui-filled-button",
        "ui-filled-tonal-button",
        "ui-outlined-button",
        "ui-text-button",
    ]);
}

#[tessera]
fn overview_button(style: ButtonVariant, index: usize, scale: f64) {
    let actions = remember(|| 0_u32);
    let mut label_style = type_style(14.0 * scale, 20.0 * scale);
    label_style.font_weight = 500;
    button()
        .variant(style)
        .size(ButtonSize::Small)
        .shape_style(ButtonShape::Round)
        .content_padding(Padding::default())
        .modifier(Modifier::new().size(
            Dp([90.0, 68.0, 68.0, 90.0, 64.0][index] * scale),
            Dp(40.0 * scale),
        ))
        .border_width(Dp(if style == ButtonVariant::Outlined {
            scale
        } else {
            0.0
        }))
        .accessibility_label(I18N.format(
            "ui-overview-value-button",
            [("v0", I18N.text(&style_name(style)).into())],
        ))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
        .on_click(move || actions.set(actions.get() + 1))
        .child(move || {
            text()
                .content(I18N.text(style_name(style)))
                .style(label_style);
        });
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(|| {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        button_preview(
                            ButtonVariant::Filled,
                            ButtonSize::Small,
                            ButtonShape::Round,
                            false,
                            true,
                            "ui-label",
                        )
                        .leading(filled::EDIT_SVG);
                    }),
                );
                spacer().modifier(Modifier::new().width(Dp(64.0)));
                marked(2, RenderSlot::new(|| config_row(1)));
            });
    }));
    gap(Dp(16.0));
    legend(vec!["ui-default-button", "ui-toggle-button"]);
    gap(Dp(32.0));
    spec_table(
        vec!["ui-variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec![
                "ui-default".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-toggle-selection".into(),
                "--".into(),
                "ui-available".into(),
            ],
        ],
    );
}

#[tessera]
fn configurations() {
    config_figure(vec![0, 2, 3, 4]);
    gap(Dp(16.0));
    legend(vec![
        "ui-size",
        "ui-shape",
        "ui-color",
        "ui-small-button-padding",
    ]);
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-category".into(),
            "ui-configuration".into(),
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
                "XS, M, L, XL".into(),
                "--".into(),
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
                "--".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-color".into(),
                "ui-elevated-filled-default-tonal-outlined-text".into(),
                "ui-available".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-small-button-padding".into(),
                "24dp".into(),
                "ui-available".into(),
                "ui-not-recommended-use-16dp".into(),
            ],
            vec![
                "ui-small-button-padding".into(),
                "16dp".into(),
                "--".into(),
                "ui-available".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("ui-live-configuration".into());
    gap(Dp(8.0));
    prose(
        "ui-change-the-configuration-and-activate-the-button-focus-it-with-tab-and-activate-it-with-sp",
    );
    gap(Dp(24.0));
    playground();
}

#[tessera]
fn tokens() {
    let style = remember(|| ButtonVariant::Filled);
    prose(
        "ui-select-a-color-style-to-inspect-its-container-content-border-elevation-typography-and-shap",
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for variant in STYLES {
                button()
                    .tonal()
                    .selected(style.get() == variant)
                    .accessibility_label(I18N.format(
                        "ui-value-color-tokens",
                        [("v0", I18N.text(&style_name(variant)).into())],
                    ))
                    .on_click(move || style.set(variant))
                    .child(move || {
                        text().content(I18N.text(style_name(variant)));
                    });
            }
        });
    gap(Dp(24.0));
    let mut rows = match style.get() {
        ButtonVariant::Elevated => vec![
            vec!["ui-container".into(), "Surface container low".into()],
            vec!["ui-icon-and-label".into(), "Primary".into()],
            vec!["ui-border".into(), "ui-none".into()],
            vec!["ui-elevation".into(), "ui-1dp-enabled-0dp-disabled".into()],
        ],
        ButtonVariant::Filled => vec![
            vec!["ui-container".into(), "Primary".into()],
            vec!["ui-icon-and-label".into(), "On primary".into()],
            vec!["ui-border".into(), "ui-none".into()],
            vec!["ui-elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Tonal => vec![
            vec!["ui-container".into(), "Secondary container".into()],
            vec!["ui-icon-and-label".into(), "On secondary container".into()],
            vec!["ui-border".into(), "ui-none".into()],
            vec!["ui-elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Outlined => vec![
            vec!["ui-container".into(), "ui-transparent".into()],
            vec!["ui-icon-and-label".into(), "On surface variant".into()],
            vec!["ui-border".into(), "Outline variant · 1dp".into()],
            vec!["ui-elevation".into(), "0dp".into()],
        ],
        ButtonVariant::Text => vec![
            vec!["ui-container".into(), "ui-transparent".into()],
            vec!["ui-icon-and-label".into(), "Primary".into()],
            vec!["ui-border".into(), "ui-none".into()],
            vec!["ui-elevation".into(), "0dp".into()],
        ],
    };
    rows.push(vec![
        "ui-label-type".into(),
        "Label large · 14sp / 20sp".into(),
    ]);
    rows.push(vec![
        "ui-shape".into(),
        "ui-full-capsule-size-specific-corners-when-square".into(),
    ]);
    rows.push(vec![
        "ui-height".into(),
        "ui-40dp-for-the-default-small-size".into(),
    ]);
    rows.push(vec!["ui-disabled-label-opacity".into(), "38%".into()]);
    spec_table(vec!["ui-element".into(), "ui-token-value".into()], rows);
}

#[tessera]
fn anatomy(labels: Vec<&'static str>) {
    panel(RenderSlot::new(|| {
        button_preview(
            ButtonVariant::Filled,
            ButtonSize::Large,
            ButtonShape::Round,
            false,
            true,
            "ui-label",
        )
        .leading(filled::EDIT_SVG);
    }));
    gap(Dp(16.0));
    legend(labels);
}

#[tessera]
fn color_examples(dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            surface()
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(32.0)))
                        .children(move || {
                            text()
                                .content(I18N.text(if dark {
                                    "ui-dark-scheme"
                                } else {
                                    "ui-light-scheme"
                                }))
                                .style(type_style(24.0, 32.0));
                            gap(Dp(24.0));
                            style_grid(
                                vec![
                                    "ui-1-default",
                                    "ui-2-toggle-unselected",
                                    "ui-3-toggle-selected",
                                ],
                                vec!["A", "B", "C", "D", "E"],
                            );
                        });
                });
        });
}

#[tessera]
fn colors() {
    prose(
        "ui-color-values-are-implemented-through-design-tokens-for-designers-this-means-working-with-c",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-there-are-five-built-in-button-color-styles-elevated-filled-tonal-outlined-and-text"
            .into(),
        "ui-the-default-and-toggle-buttons-use-different-colors".into(),
        "ui-toggle-buttons-don-t-use-the-text-style".into(),
    ]);
    gap(Dp(24.0));
    note(
        "ui-note-these-color-roles-were-chosen-to-create-design-coherence-and-familiarity-other-color",
    );
    gap(Dp(32.0));
    color_examples(false);
    gap(Dp(24.0));
    color_examples(true);
    caption("ui-a-elevated-b-filled-c-tonal-d-outlined-e-text");
    gap(Dp(16.0));
    legend(vec![
        "ui-default",
        "ui-toggle-unselected",
        "ui-toggle-selected",
    ]);
    gap(Dp(32.0));
    let pair = |container: &str, content: &str| format!("{container}\n{content}");
    spec_table(
        vec![
            "".into(),
            "ui-1-default".into(),
            "ui-2-toggle-unselected".into(),
            "ui-3-toggle-selected".into(),
        ],
        vec![
            vec![
                pair("ui-elevated-container", "ui-elevated-icon-label"),
                pair("Surface container low", "Primary"),
                pair("Surface container low", "Primary"),
                pair("Primary", "On primary"),
            ],
            vec![
                pair("ui-filled-container", "ui-filled-icon-label"),
                pair("Primary", "On primary"),
                pair("Surface container", "On surface variant"),
                pair("Primary", "On primary"),
            ],
            vec![
                pair("ui-tonal-container", "ui-tonal-icon-label"),
                pair("Secondary container", "On secondary container"),
                pair("Secondary container", "On secondary container"),
                pair("Secondary", "On secondary"),
            ],
            vec![
                pair("ui-outlined-container", "ui-outlined-icon-label"),
                pair("Outline variant (outline)", "On surface variant"),
                pair("Outline variant (outline)", "On surface variant"),
                pair("Inverse surface", "Inverse on surface"),
            ],
            vec![
                "ui-text-icon-label".into(),
                "Primary".into(),
                "--".into(),
                "--".into(),
            ],
        ],
    );
}

#[tessera]
fn states() {
    prose(
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-interac",
    );
    gap(Dp(40.0));
    for style in STYLES {
        subheading(if style == ButtonVariant::Text {
            "ui-text-button-style-states".into()
        } else {
            I18N.format(
                "ui-value-button-states",
                [("v0", I18N.text(&style_name(style)).into())],
            )
        });
        match style {
            ButtonVariant::Elevated => {
                gap(Dp(16.0));
                prose(
                    "ui-the-elevated-button-style-has-an-elevation-of-1-by-default-and-0-when-disabled",
                );
            }
            ButtonVariant::Outlined => {
                gap(Dp(16.0));
                prose(
                    "ui-the-outlined-button-s-container-fill-is-invisible-at-rest-but-the-opacity-and-state-layers",
                );
            }
            ButtonVariant::Text => {
                gap(Dp(16.0));
                prose(
                    "ui-the-text-button-s-container-is-invisible-at-rest-but-the-opacity-and-state-layers-behave-t",
                );
            }
            _ => {}
        }
        gap(Dp(24.0));
        subheading("ui-default".into()).level(4);
        gap(Dp(24.0));
        button_state_matrix(style, false);
        if style != ButtonVariant::Text {
            gap(Dp(40.0));
            subheading("ui-toggle".into()).level(4);
            gap(Dp(16.0));
            caption("ui-a-unselected-b-selected");
            gap(Dp(16.0));
            button_state_matrix(style, true);
        }
        gap(Dp(64.0));
    }
}

#[tessera]
fn button_state_matrix(style: ButtonVariant, toggle: bool) {
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
                                        [169.0, 353.0, 537.0, 722.0, 909.0][state]
                                    } else {
                                        [144.0, 328.0, 512.0, 696.0, 880.0][state]
                                    };
                                    button_state_marker(
                                        (state + 1).to_string(),
                                        center,
                                        if toggle {
                                            if dark { 74.0 } else { 79.0 }
                                        } else {
                                            116.0
                                        },
                                        scale,
                                    );
                                    for selected in
                                        [false, true].into_iter().take(if toggle { 2 } else { 1 })
                                    {
                                        let mut center_y = if toggle {
                                            if selected { 208.0 } else { 136.0 }
                                        } else {
                                            172.0
                                        };
                                        if toggle && dark {
                                            center_y -= 4.0;
                                        }
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(
                                                Modifier::new()
                                                    .offset(
                                                        Dp((center - 80.0) * scale),
                                                        Dp((center_y - 40.0) * scale),
                                                    )
                                                    .size(Dp(160.0 * scale), Dp(80.0 * scale)),
                                            )
                                            .children(move || {
                                                button_state_item(
                                                    style,
                                                    toggle,
                                                    selected,
                                                    state,
                                                    scale,
                                                    I18N.format(
                                                        "ui-value-value-value-value-value",
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
                                                            (
                                                                "v1",
                                                                I18N.text(&style_name(style))
                                                                    .into(),
                                                            ),
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
                                    button_state_marker(
                                        "A".into(),
                                        86.0,
                                        if dark { 132.0 } else { 136.0 },
                                        scale,
                                    );
                                    button_state_marker(
                                        "B".into(),
                                        86.0,
                                        if dark { 204.0 } else { 208.0 },
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
            .content(I18N.text("ui-a-unselected-b-selected"))
            .style(type_style(16.0, 24.0));
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
        text()
            .content(I18N.format(
                "ui-value-value",
                [("v0", (index + 1).into()), ("v1", I18N.text(&label).into())],
            ))
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn button_state_marker(label: String, x: f64, y: f64, scale: f64) {
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
                .content(I18N.text(&(label)))
                .style(type_style(14.0 * scale, 20.0 * scale))
                .color(scheme.surface);
        });
}

#[tessera]
fn button_state_item(
    style: ButtonVariant,
    toggle: bool,
    initial: bool,
    state: usize,
    scale: f64,
    name: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    let name = remember(|| name);
    let mut label_style = type_style(14.0 * scale, 20.0 * scale);
    label_style.font_weight = 500;
    button()
        .variant(style)
        .size(ButtonSize::Small)
        .shape_style(ButtonShape::Round)
        .selected_optional(if toggle && style != ButtonVariant::Text {
            Some(selected.get())
        } else {
            None
        })
        .enabled(state != 1)
        .presentation_state(match state {
            2 => InteractionVisualState::Hover,
            3 => InteractionVisualState::Focus,
            4 => InteractionVisualState::Pressed,
            _ => InteractionVisualState::Rest,
        })
        .content_padding(Padding::default())
        .modifier(Modifier::new().size(
            Dp([84.0, 90.0, 86.0, 86.0, 84.0][state] * scale),
            Dp(40.0 * scale),
        ))
        .border_width(Dp(if style == ButtonVariant::Outlined {
            scale
        } else {
            0.0
        }))
        .accessibility_label(I18N.text(name.get()))
        .accessibility_description(
            I18N.format("ui-activations-value", [("v0", (actions.get()).into())]),
        )
        .on_click(move || {
            actions.set(actions.get() + 1);
            if toggle {
                selected.set(!selected.get());
            }
        })
        .child(move || {
            text()
                .content(I18N.text(
                    [
                        "ui-enabled",
                        "ui-disabled",
                        "ui-hovered",
                        "ui-focused",
                        "ui-pressed",
                    ][state],
                ))
                .style(label_style);
        });
}

#[tessera]
fn shape_morph() {
    subheading("ui-pressed-state".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-when-pressed-buttons-can-morph-to-become-more-square-both-round-and-square-buttons-should",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-corner-radius-value-differs-for-each-button-size-see-the-corner-sizes-under-measuremen",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["ui-enabled", "ui-hovered", "ui-pressed"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    [ButtonShape::Round, ButtonShape::Square][row],
                    false,
                    true,
                    "ui-learn-more",
                )
                .presentation(
                    [
                        InteractionVisualState::Rest,
                        InteractionVisualState::Hover,
                        InteractionVisualState::Pressed,
                    ][column],
                );
            }),
        );
    }));
    caption("ui-a-round-button-b-square-button");
    gap(Dp(48.0));
    subheading("ui-when-selected".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-in-addition-to-changing-shape-when-pressed-toggle-buttons-also-change-the-resting-shape-fr",
    );
    gap(Dp(16.0));
    prose("ui-if-the-resting-unselected-shape-is-square-the-selected-shape-should-be-round");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["1", "2", "3", "4"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    [ButtonShape::Round, ButtonShape::Square][row],
                    true,
                    true,
                    "ui-play",
                )
                .initial_selected(column == 3)
                .leading(if column == 3 {
                    filled::PAUSE_SVG
                } else {
                    filled::PLAY_ARROW_SVG
                })
                .presentation(
                    [
                        InteractionVisualState::Rest,
                        InteractionVisualState::Hover,
                        InteractionVisualState::Pressed,
                        InteractionVisualState::Rest,
                    ][column],
                );
            }),
        );
    }));
    caption("ui-a-round-button-b-square-button");
    gap(Dp(16.0));
    legend(vec![
        "ui-enabled",
        "ui-hovered",
        "ui-pressed",
        "ui-selected",
    ]);
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for (index, size) in SIZES.into_iter().enumerate() {
                    if index > 0 {
                        gap(Dp(24.0));
                    }
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    button_preview(
                                        ButtonVariant::Filled,
                                        size,
                                        ButtonShape::Round,
                                        false,
                                        true,
                                        "ui-label",
                                    )
                                    .leading(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(24.0)));
                                    button_preview(
                                        ButtonVariant::Filled,
                                        size,
                                        ButtonShape::Round,
                                        false,
                                        true,
                                        "ui-label",
                                    );
                                });
                        }),
                    );
                }
            });
    }));
    caption("ui-padding-and-size-measurements-of-each-button-size");
    gap(Dp(16.0));
    legend(vec![
        "ui-extra-small",
        "ui-small",
        "ui-medium",
        "ui-large",
        "ui-extra-large",
    ]);
    gap(Dp(32.0));
    spec_table(
        ["ui-attribute", "XS", "S", "M", "L", "XL"]
            .map(str::to_owned)
            .to_vec(),
        vec![
            [
                "ui-container-height",
                "32dp",
                "40dp",
                "56dp",
                "96dp",
                "136dp",
            ]
            .map(str::to_owned)
            .to_vec(),
            [
                "ui-horizontal-padding",
                "12dp",
                "16dp",
                "24dp",
                "48dp",
                "64dp",
            ]
            .map(str::to_owned)
            .to_vec(),
            ["ui-icon-size", "20dp", "20dp", "24dp", "32dp", "40dp"]
                .map(str::to_owned)
                .to_vec(),
            ["ui-icon-label-spacing", "4dp", "8dp", "8dp", "12dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
            ["ui-outline-width", "1dp", "1dp", "1dp", "2dp", "3dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
    gap(Dp(48.0));
    subheading("ui-target-areas".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-extra-small-and-small-icon-buttons-must-have-a-target-size-of-48x48dp-or-larger-to-be-acce",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        figure_grid(
            vec!["ui-round-button", "ui-button-with-icon", "ui-square-button"],
            vec!["A", "B"],
            RenderSlotWith::new(|(row, column): (usize, usize)| {
                let preview = button_preview(
                    ButtonVariant::Filled,
                    [ButtonSize::ExtraSmall, ButtonSize::Small][row],
                    if column == 2 {
                        ButtonShape::Square
                    } else {
                        ButtonShape::Round
                    },
                    false,
                    true,
                    "ui-label",
                );
                if column == 1 {
                    preview.leading(filled::EDIT_SVG);
                }
            }),
        );
    }));
    caption("ui-a-extra-small-b-small");
    gap(Dp(48.0));
    subheading("ui-corner-sizes".into()).level(4);
    gap(Dp(16.0));
    spec_table(
        ["", "XS", "S", "M", "L", "XL"].map(str::to_owned).to_vec(),
        vec![
            [
                "ui-a-round-button",
                "ui-full",
                "ui-full",
                "ui-full",
                "ui-full",
                "ui-full",
            ]
            .map(str::to_owned)
            .to_vec(),
            ["ui-b-square-button", "12dp", "12dp", "16dp", "28dp", "28dp"]
                .map(str::to_owned)
                .to_vec(),
            ["ui-c-pressed-state", "8dp", "8dp", "12dp", "16dp", "16dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
}

fn next_style(style: ButtonVariant) -> ButtonVariant {
    match style {
        ButtonVariant::Elevated => ButtonVariant::Filled,
        ButtonVariant::Filled => ButtonVariant::Tonal,
        ButtonVariant::Tonal => ButtonVariant::Outlined,
        ButtonVariant::Outlined => ButtonVariant::Text,
        ButtonVariant::Text => ButtonVariant::Elevated,
    }
}

fn next_size(size: ButtonSize) -> ButtonSize {
    match size {
        ButtonSize::ExtraSmall => ButtonSize::Small,
        ButtonSize::Small => ButtonSize::Medium,
        ButtonSize::Medium => ButtonSize::Large,
        ButtonSize::Large => ButtonSize::ExtraLarge,
        ButtonSize::ExtraLarge => ButtonSize::ExtraSmall,
    }
}

fn size_name(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::ExtraSmall => "ui-extra-small",
        ButtonSize::Small => "ui-small",
        ButtonSize::Medium => "ui-medium",
        ButtonSize::Large => "ui-large",
        ButtonSize::ExtraLarge => "ui-extra-large",
    }
}

#[tessera]
pub(super) fn prose(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
}

#[tessera]
pub(super) fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text()
        .content(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
pub(super) fn bullets(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children(move || {
                        spacer().modifier(
                            Modifier::new()
                                .size(Dp(8.0), Dp(8.0))
                                .background_with_shape(scheme.on_surface_variant, Shape::Ellipse),
                        );
                        spacer().modifier(Modifier::new().width(Dp(16.0)));
                        text()
                            .content(I18N.text(&(item)))
                            .style(type_style(16.0, 24.0));
                    });
            }
        });
}

#[tessera]
pub(super) fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content((number).to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

/// Numbered caption list under a figure, matching the figure markers.
#[tessera]
fn legend(items: Vec<&'static str>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, item) in items.clone().into_iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
                    .children(move || {
                        badge(index + 1);
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        text()
                            .content(I18N.text(item))
                            .style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// A figure row with a leading numbered marker.
#[tessera]
pub(super) fn marked(number: usize, content: RenderSlot) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            content.render();
        });
}

#[tessera]
fn guidance(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text()
                        .content(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0))
                .modifier(Modifier::new().weight(1.0));
        });
}

/// A lettered and numbered figure grid with one cell per row and column.
#[tessera]
fn figure_grid(
    columns: Vec<&'static str>,
    rows: Vec<&'static str>,
    cell: RenderSlotWith<(usize, usize)>,
) {
    let count = columns.len();
    column().children(move || {
        let columns = columns.clone();
        row().children(move || {
            spacer().modifier(Modifier::new().width(Dp(40.0)));
            for header in columns.clone() {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(Modifier::new().width(Dp(CELL)))
                    .children(move || {
                        text()
                            .content(I18N.text(header))
                            .style(type_style(14.0, 20.0));
                    });
            }
        });
        for (index, label) in rows.clone().into_iter().enumerate() {
            gap(Dp(16.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    boxed()
                        .modifier(Modifier::new().width(Dp(40.0)))
                        .children(move || {
                            text()
                                .content(I18N.text(label))
                                .style(type_style(16.0, 24.0));
                        });
                    for column in 0..count {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().width(Dp(CELL)))
                            .children(move || cell.render((index, column)));
                    }
                });
        }
    });
}

#[tessera]
fn note(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_high,
                    Shape::rounded_rectangle(Dp(16.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            icon().painter(filled::STAR_SVG).size(Dp(24.0));
            spacer().modifier(Modifier::new().width(Dp(16.0)));
            text()
                .content(I18N.text(&(value)))
                .style(type_style(16.0, 24.0))
                .modifier(Modifier::new().weight(1.0));
        });
}

/// One configuration row: sizes, toggle, shapes, colors, padding or square
/// sizes.
#[tessera]
fn config_row(kind: usize) {
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || match kind {
            0 | 5 => {
                for size in SIZES {
                    button_preview(
                        ButtonVariant::Filled,
                        size,
                        if kind == 0 {
                            ButtonShape::Round
                        } else {
                            ButtonShape::Square
                        },
                        false,
                        true,
                        size_name(size),
                    );
                }
            }
            1 => {
                for selected in [false, true] {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Small,
                        ButtonShape::Round,
                        true,
                        true,
                        if selected {
                            "ui-selected"
                        } else {
                            "ui-unselected"
                        },
                    )
                    .initial_selected(selected);
                }
            }
            2 => {
                for (shape, label) in [
                    (ButtonShape::Round, "ui-round"),
                    (ButtonShape::Square, "ui-square"),
                ] {
                    button_preview(
                        ButtonVariant::Filled,
                        ButtonSize::Small,
                        shape,
                        false,
                        true,
                        label,
                    );
                }
            }
            3 => {
                for style in STYLES {
                    button_preview(
                        style,
                        ButtonSize::Small,
                        ButtonShape::Round,
                        false,
                        true,
                        style_name(style),
                    );
                }
            }
            _ => {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-24dp-padding",
                )
                .padding(Padding::symmetric(Dp(24.0), Dp(10.0)));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-16dp-padding",
                );
            }
        });
}

#[tessera]
fn config_figure(rows: Vec<usize>) {
    panel(RenderSlot::new(move || {
        let rows = rows.clone();
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                for (index, kind) in rows.clone().into_iter().enumerate() {
                    if index > 0 {
                        gap(Dp(32.0));
                    }
                    marked(index + 1, RenderSlot::new(move || config_row(kind)));
                }
            });
    }));
}

/// Default and toggle buttons for every color style in a lettered grid.
#[tessera]
fn style_grid(columns: Vec<&'static str>, rows: Vec<&'static str>) {
    figure_grid(
        columns,
        rows,
        RenderSlotWith::new(move |(row, column): (usize, usize)| {
            let style = STYLES[row];
            if column > 0 && style == ButtonVariant::Text {
                text().content("--").style(type_style(14.0, 20.0));
            } else {
                button_preview(
                    style,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    column > 0,
                    true,
                    "ui-label",
                )
                .initial_selected(column == 2)
                .leading(filled::EDIT_SVG);
            }
        }),
    );
}

#[tessera]
fn m2_button(style: ButtonVariant) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (container, content) = match style {
        ButtonVariant::Elevated => (scheme.surface_container_low, scheme.primary),
        ButtonVariant::Filled => (scheme.primary, scheme.on_primary),
        ButtonVariant::Tonal => (scheme.secondary_container, scheme.on_secondary_container),
        ButtonVariant::Outlined | ButtonVariant::Text => (Color::TRANSPARENT, scheme.primary),
    };
    let border = if style == ButtonVariant::Outlined {
        scheme.outline
    } else {
        Color::TRANSPARENT
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .height(Dp(36.0))
                .border_with_shape(Dp(1.0), border, Shape::rounded_rectangle(Dp(4.0)))
                .background_with_shape(container, Shape::rounded_rectangle(Dp(4.0)))
                .padding_symmetric(Dp(16.0), Dp::ZERO),
        )
        .children(move || {
            text()
                .content(I18N.text("ui-button-7082"))
                .color(content)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn style_example(style: ButtonVariant, #[prop(into)] label: String) {
    panel(RenderSlot::new(move || {
        button_preview(
            style,
            ButtonSize::Medium,
            ButtonShape::Round,
            false,
            true,
            label.clone(),
        );
    }));
}

#[tessera]
fn flight_example(large: bool) {
    panel(RenderSlot::new(move || {
        if large {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    flight_info();
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    flight_actions();
                });
        } else {
            column()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(|| {
                    boxed()
                        .modifier(Modifier::new().fill_max_width())
                        .children(flight_info);
                    gap(Dp(24.0));
                    flight_actions();
                });
        }
    }));
}

#[tessera]
fn flight_info() {
    column().children(move || {
        text()
            .content(I18N.text("ui-sfo-jfk"))
            .style(type_style(24.0, 32.0));
        text()
            .content(I18N.text("ui-depart-8-05-am-nonstop-5h-32m"))
            .style(type_style(14.0, 20.0));
    });
}

#[tessera]
fn flight_actions() {
    row().children(|| {
        button_preview(
            ButtonVariant::Outlined,
            ButtonSize::Small,
            ButtonShape::Round,
            false,
            true,
            "ui-details",
        );
        spacer().modifier(Modifier::new().width(Dp(8.0)));
        button_preview(
            ButtonVariant::Filled,
            ButtonSize::Small,
            ButtonShape::Round,
            false,
            true,
            "ui-book",
        )
        .leading(filled::FLIGHT_SVG);
    });
}

#[tessera]
fn overview() {
    bullets(vec![
        "ui-two-variants-default-and-toggle".into(),
        "ui-can-contain-an-optional-leading-icon".into(),
        "ui-five-color-options-elevated-filled-tonal-outlined-and-text".into(),
        "ui-five-size-recommendations-extra-small-small-medium-large-and-extra-large".into(),
        "ui-two-shape-options-round-and-square".into(),
        "ui-keep-labels-concise-and-use-sentence-case".into(),
    ]);
    gap(Dp(32.0));
    overview_art();
}

#[tessera]
fn resources() {
    spec_table(
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "ui-design-kit-figma".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Flutter".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose: Expressive".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web: Expressive".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
}

#[tessera]
fn expressive() {
    subheading("ui-may-2025".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-buttons-now-have-a-wider-variety-of-shapes-and-sizes-toggle-functionality-and-can-change-s",
    );
    for (title, items) in [
        (
            "ui-variants-and-naming-921",
            vec![
                "ui-default-and-toggle-selection",
                "ui-color-styles-are-now-configurations-elevated-filled-tonal-outlined-text",
            ],
        ),
        (
            "Shapes:",
            vec![
                "ui-round-and-square",
                "ui-shape-morphs-when-pressed",
                "ui-shape-morphs-when-selected",
            ],
        ),
        (
            "Sizes:",
            vec![
                "ui-extra-small",
                "ui-small-existing-default",
                "ui-medium",
                "ui-large",
                "ui-extra-large",
            ],
        ),
        (
            "ui-new-padding-for-small-buttons",
            vec![
                "ui-16dp-recommended-to-match-padding-of-new-sizes",
                "ui-24dp-no-longer-recommended",
            ],
        ),
    ] {
        gap(Dp(16.0));
        prose(title);
        bullets(items.into_iter().map(String::from).collect());
    }
    gap(Dp(32.0));
    config_figure(vec![0, 1, 2, 4]);
    gap(Dp(16.0));
    legend(vec![
        "ui-five-sizes",
        "ui-toggle-selection",
        "ui-two-shapes",
        "ui-two-small-padding-widths",
    ]);
}

#[tessera]
fn differences() {
    bullets(vec![
        "ui-color-new-color-mappings-and-compatibility-with-dynamic-color-icons-and-labels-now-share-t".into(),
        "ui-icons-standard-size-for-leading-and-trailing-icons-is-now-20dp".into(),
        "ui-shape-fully-rounded-corner-radius-and-additional-height-options".into(),
    ]);
    gap(Dp(24.0));
    m2_m3_figure();
    gap(Dp(16.0));
    legend(vec![
        "ui-m2-buttons-have-a-height-of-36dp-and-slightly-rounded-corner-radius",
        "ui-m3-default-buttons-are-taller-at-40dp-and-have-fully-rounded-corners",
    ]);
}

#[tessera]
fn m2_m3_figure() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                marked(
                    1,
                    RenderSlot::new(|| {
                        flow_row()
                            .item_spacing(Dp(16.0))
                            .line_spacing(Dp(16.0))
                            .children(|| {
                                for style in STYLES {
                                    m2_button(style);
                                }
                            });
                    }),
                );
                gap(Dp(32.0));
                marked(2, RenderSlot::new(|| config_row(3)));
            });
    }));
}

#[tessera]
fn usage() {
    prose(
        "ui-buttons-communicate-actions-that-people-can-take-they-are-typically-placed-throughout-the",
    );
    gap(Dp(8.0));
    bullets(vec![
        "ui-dialogs".into(),
        "ui-modal-windows".into(),
        "ui-forms".into(),
        "ui-cards".into(),
        "ui-toolbars".into(),
    ]);
    gap(Dp(8.0));
    prose("ui-they-can-also-be-placed-within-standard-button-groups");
    gap(Dp(24.0));
    usage_example();
    caption("ui-use-visually-prominent-filled-buttons-for-the-most-important-actions");
    gap(Dp(32.0));
    prose(
        "ui-buttons-are-just-one-option-for-representing-actions-in-a-product-and-shouldn-t-be-overuse",
    );
    gap(Dp(16.0));
    prose(
        "ui-consider-placing-additional-actions-in-a-navigation-rail-set-of-chips-text-links-or-icon-b",
    );
    guidance("Do", "ui-use-buttons-for-discrete-actions", DO_COLOR);
    guidance(
        "Don’t",
        "ui-don-t-clutter-your-ui-with-too-many-buttons-consider-presenting-low-priority-actions-in-ov",
        DONT_COLOR,
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-learn-more",
                );
                gap(Dp(24.0));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-learn-more",
                )
                .modifier(Modifier::new().fill_max_width());
                gap(Dp(24.0));
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-learn-more",
                )
                .modifier(Modifier::new().width(Dp(64.0)).clip_to_bounds());
            });
    }));
    guidance(
        "Do",
        "ui-a-button-container-s-width-is-dynamically-set-to-fit-its-label-text",
        DO_COLOR,
    );
    guidance(
        "Do",
        "ui-button-container-width-can-be-responsive-which-allows-it-to-stretch-horizontally",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-a-button-container-s-width-shouldn-t-be-narrower-than-its-label-text",
        DONT_COLOR,
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        style_grid(vec!["A", "B", "C"], vec!["1", "2", "3", "4", "5"]);
    }));
    caption(
        "ui-a-default-button-b-toggle-unselected-c-toggle-selected-for-five-button-styles-in-order-of",
    );
    gap(Dp(16.0));
    legend(vec![
        "ui-elevated-button",
        "ui-filled-button",
        "ui-filled-tonal-button",
        "ui-outlined-button",
        "ui-text-button",
    ]);
    gap(Dp(32.0));
    prose(
        "ui-a-button-group-is-a-collection-of-buttons-that-relate-to-each-other-and-can-respond-to-one",
    );
    gap(Dp(16.0));
    prose(
        "ui-in-some-cases-there-are-primary-and-secondary-actions-within-a-button-group-buttons-with-p",
    );
}

#[tessera]
fn usage_example() {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                text()
                    .content(I18N.text("ui-appointment-thea"))
                    .style(type_style(24.0, 32.0));
                gap(Dp(8.0));
                text()
                    .content(I18N.text("ui-your-host-has-not-yet-joined"))
                    .style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-ask-to-join"))
                            .on_click(|| {})
                            .child(move || {
                                text().content(I18N.text("ui-ask-to-join"));
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-share-details"))
                            .on_click(|| {})
                            .child(move || {
                                text().content(I18N.text("ui-share-details"));
                            });
                    });
            });
    }));
}

#[tessera]
fn toggle_buttons() {
    prose(
        "ui-toggle-buttons-should-be-used-for-binary-selections-such-as-save-or-favorite-when-toggle-b",
    );
    gap(Dp(16.0));
    prose(
        "ui-toggle-buttons-should-use-an-outlined-icon-when-unselected-and-a-filled-version-of-the-ico",
    );
    gap(Dp(16.0));
    prose("ui-by-default-toggle-buttons-change-from-round-to-square-when-selected");
    gap(Dp(24.0));
    toggle_example();
    caption("ui-use-toggle-buttons-for-binary-actions");
    gap(Dp(32.0));
    prose(
        "ui-if-the-label-changes-on-selected-or-unselected-states-be-mindful-of-the-character-count-ch",
    );
    guidance(
        "Do",
        "ui-when-using-toggleable-buttons-keep-the-label-character-count-a-similar-length-for-both-sta",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-the-label-length-shouldn-t-change-dramatically-to-be-longer-or-shorter",
        DONT_COLOR,
    );
}

#[tessera]
fn toggle_example() {
    let filled = remember(|| false);
    let tonal = remember(|| true);
    let outlined = remember(|| true);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .selected(filled.get())
                            .accessibility_label(I18N.text("ui-filled-toggle"))
                            .on_click(move || filled.set(!filled.get()))
                            .child(move || {
                                text().content(I18N.text("ui-filled"));
                            });
                        button()
                            .tonal()
                            .size(ButtonSize::Medium)
                            .selected(tonal.get())
                            .accessibility_label(I18N.text("ui-tonal-toggle"))
                            .on_click(move || tonal.set(!tonal.get()))
                            .child(move || {
                                text().content(I18N.text("ui-tonal"));
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .selected(outlined.get())
                            .accessibility_label(I18N.text("ui-outlined-toggle"))
                            .on_click(move || outlined.set(!outlined.get()))
                            .child(move || {
                                text().content(I18N.text("ui-outlined"));
                            });
                    });
                gap(Dp(16.0));
                text()
                    .content(
                        I18N.format(
                            "ui-filled-value-tonal-value-outlined-value",
                            [
                                (
                                    "v0",
                                    I18N.text(
                                        &(if filled.get() {
                                            "ui-selected-7075"
                                        } else {
                                            "ui-unselected-7076"
                                        }),
                                    )
                                    .into(),
                                ),
                                (
                                    "v1",
                                    I18N.text(
                                        &(if tonal.get() {
                                            "ui-selected-7075"
                                        } else {
                                            "ui-unselected-7076"
                                        }),
                                    )
                                    .into(),
                                ),
                                (
                                    "v2",
                                    I18N.text(
                                        &(if outlined.get() {
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
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn guidelines_anatomy() {
    anatomy(vec!["ui-label-text", "ui-container", "ui-icon-optional"]);
    gap(Dp(48.0));
    subheading("ui-label-text".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-label-text-is-the-most-important-element-of-a-button-it-describes-the-action-that-will-occ",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-sentence-case-which-only-capitalizes-the-first-word-and-proper-nouns-this-allows-the-t",
    );
    gap(Dp(16.0));
    prose(
        "ui-don-t-truncate-or-wrap-label-text-it-should-always-be-fully-visible-on-a-single-line",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Filled, "ui-book-with-flights");
    guidance(
        "Do",
        "ui-use-sentence-case-for-button-label-text-capitalizing-the-first-word-and-proper-nouns",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-don-t-wrap-text-for-maximum-legibility-label-text-should-remain-on-a-single-line",
        DONT_COLOR,
    );
    gap(Dp(24.0));
    prose(
        "ui-buttons-with-the-outlined-and-text-color-style-depend-on-the-colors-to-be-recognizable-fro",
    );
    guidance(
        "ui-caution",
        "ui-the-outlined-button-style-is-very-similar-to-chips-consider-using-a-filled-or-tonal-button",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-container".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-button-containers-hold-the-label-text-and-optional-icon-buttons-with-the-text-color-style",
    );
    gap(Dp(16.0));
    prose("ui-buttons-with-a-round-shape-have-containers-with-fully-rounded-corners");
    gap(Dp(16.0));
    prose(
        "ui-buttons-with-a-square-shape-have-containers-with-more-subtle-rounding-that-changes-based-o",
    );
    gap(Dp(24.0));
    config_figure(vec![0, 5]);
    gap(Dp(16.0));
    legend(vec![
        "ui-round-buttons-have-containers-with-fully-rounded-corners",
        "ui-square-buttons-have-square-containers-and-change-radius-as-the-button-size-changes",
    ]);
    guidance(
        "Do",
        "ui-a-button-s-width-dynamically-adjusts-to-the-label-text",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-avoid-setting-a-fixed-width-smaller-than-the-label-text",
        DONT_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-icon-optional".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-icons-visually-communicate-the-button-s-action-and-help-draw-attention-they-should-be-plac",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                button_preview(
                    ButtonVariant::Filled,
                    ButtonSize::Small,
                    ButtonShape::Round,
                    false,
                    true,
                    "ui-send",
                )
                .leading(filled::SEND_SVG);
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                button()
                    .filled()
                    .accessibility_label(I18N.text("ui-send-right-to-left"))
                    .trailing_icon(|| {
                        icon().painter(filled::SEND_SVG).rotation(180.0);
                    })
                    .on_click(|| {})
                    .child(move || {
                        text().content(I18N.text("ui-send"));
                    });
            });
    }));
    guidance(
        "Do",
        "ui-place-the-icon-to-the-left-of-the-label-in-buttons-with-text-in-left-to-right-languages",
        DO_COLOR,
    );
    guidance(
        "Do",
        "ui-place-the-icon-to-the-right-of-the-label-in-buttons-with-text-in-right-to-left-languages",
        DO_COLOR,
    );
    guidance(
        "Do",
        "ui-use-icons-that-clearly-communicate-their-meaning",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-don-t-vertically-align-an-icon-and-text-in-the-center-of-a-button",
        DONT_COLOR,
    );
    guidance(
        "Don’t",
        "ui-don-t-use-two-icons-in-the-same-button",
        DONT_COLOR,
    );
}

#[tessera]
fn color_styles() {
    subheading("ui-elevated-style".into()).level(4);
    gap(Dp(16.0));
    prose("ui-the-elevated-button-style-is-the-same-as-the-tonal-button-but-with-a-shadow");
    gap(Dp(16.0));
    prose(
        "ui-to-avoid-overusing-shadows-use-the-elevated-style-only-when-absolutely-necessary-such-as-w",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Elevated, "ui-add-to-cart");
    caption("ui-elevated-buttons-provide-separation-from-a-visually-prominent-background");
    gap(Dp(24.0));
    prose(
        "ui-buttons-at-higher-elevations-typically-have-more-emphasis-in-a-design-and-should-be-used-s",
    );
    guidance(
        "ui-caution",
        "ui-higher-elevation-increases-the-emphasis-of-a-button",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-filled-style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-the-filled-button-style-has-the-most-visual-impact-after-the-fab-and-should-be-used-for-im",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Filled, "ui-join-now");
    caption("ui-filled-buttons-have-high-visual-impact-when-used-for-important-actions");
    gap(Dp(24.0));
    prose(
        "ui-since-they-have-such-strong-emphasis-the-filled-style-should-be-used-sparingly-ideally-for",
    );
    gap(Dp(16.0));
    prose("ui-in-some-cases-filled-buttons-can-use-tertiary-colors");
    gap(Dp(48.0));
    subheading("ui-tonal-style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-the-tonal-button-style-is-useful-in-contexts-where-a-lower-priority-button-requires-slight",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Tonal, "ui-next");
    caption("ui-the-tonal-style-has-less-emphasis-than-filled-or-emphasis");
    gap(Dp(48.0));
    subheading("ui-outlined-style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-the-outlined-style-is-ideal-for-medium-emphasis-buttons-which-contain-actions-that-are-imp",
    );
    gap(Dp(16.0));
    prose(
        "ui-outlined-buttons-pair-well-with-filled-buttons-to-indicate-alternative-secondary-actions",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Outlined, "ui-share-details");
    caption("ui-outlined-buttons-contain-less-important-supporting-actions");
    gap(Dp(24.0));
    prose(
        "ui-outlined-buttons-display-a-stroke-around-the-button-container-and-have-no-fill-by-default",
    );
    gap(Dp(16.0));
    prose(
        "ui-they-should-be-placed-on-simple-backgrounds-not-visually-prominent-backgrounds-such-as-ima",
    );
    guidance(
        "Do",
        "ui-outlined-buttons-can-be-used-on-backgrounds-with-a-color-gradient",
        DO_COLOR,
    );
    guidance(
        "ui-caution",
        "ui-use-caution-when-placing-outlined-buttons-on-top-of-images-customizing-the-button-to-have",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-text-style".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-the-text-button-style-should-be-used-for-the-lowest-priority-actions-especially-when-prese",
    );
    gap(Dp(16.0));
    prose(
        "ui-they-should-be-placed-on-simple-backgrounds-not-visually-prominent-backgrounds-such-as-ima-1016",
    );
    gap(Dp(16.0));
    prose(
        "ui-don-t-underline-the-text-button-use-hyperlinked-body-text-instead-to-emphasize-links",
    );
    gap(Dp(24.0));
    style_example(ButtonVariant::Text, "ui-learn-more");
    caption("ui-use-text-buttons-for-the-lowest-priority-actions");
    gap(Dp(24.0));
    prose(
        "ui-text-buttons-are-often-placed-within-components-such-as-cards-dialogs-and-snackbars-since",
    );
    gap(Dp(16.0));
    prose(
        "ui-however-since-there-s-no-container-the-label-text-color-must-always-be-recognizable-from-n",
    );
    gap(Dp(16.0));
    prose("ui-in-cards-text-buttons-help-maintain-an-emphasis-on-card-content");
    gap(Dp(16.0));
    prose(
        "ui-dialogs-use-text-buttons-because-the-absence-of-a-container-helps-unify-the-action-with-th",
    );
    gap(Dp(16.0));
    prose(
        "ui-align-text-buttons-to-the-trailing-edge-of-dialogs-on-the-right-for-left-to-right-language",
    );
}

#[tessera]
fn adaptive() {
    subheading("ui-resizing".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-when-scaling-layouts-for-large-screen-devices-buttons-can-adapt-their-visual-presentation",
    );
    gap(Dp(16.0));
    prose("ui-choose-the-best-button-position-based-on-screen-size");
    gap(Dp(24.0));
    flight_example(false);
    caption("ui-filled-buttons-are-end-aligned-below-flight-information-in-a-compact-window");
    gap(Dp(24.0));
    flight_example(true);
    caption("ui-filled-buttons-are-start-aligned-beside-flight-information-in-a-large-window");
    gap(Dp(32.0));
    prose(
        "ui-the-icon-and-label-text-in-a-button-stay-centered-and-grouped-as-the-button-s-width-change",
    );
    guidance(
        "Do",
        "ui-keep-the-icon-and-label-text-grouped-and-centered",
        DO_COLOR,
    );
    guidance(
        "Don’t",
        "ui-don-t-ungroup-the-icon-and-label-text-or-let-them-anchor-to-opposite-sides-of-the-button",
        DONT_COLOR,
    );
    gap(Dp(24.0));
    prose(
        "ui-buttons-can-be-customized-to-change-size-and-scaling-behavior-across-different-breakpoints",
    );
    gap(Dp(16.0));
    prose(
        "ui-to-avoid-creating-very-long-buttons-in-large-windows-constrain-button-width-or-place-butto",
    );
    guidance(
        "Don’t",
        "ui-don-t-allow-the-button-to-stretch-in-a-way-that-creates-long-flat-buttons-with-very-little",
        DONT_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-presentation".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-the-size-and-placement-of-buttons-can-change-as-parent-containers-such-as-cards-adapt-for",
    );
    gap(Dp(16.0));
    prose(
        "ui-keep-items-including-buttons-in-the-same-order-between-large-and-small-screens-to-provide",
    );
    caption("ui-buttons-can-move-in-the-layout-but-elements-should-remain-in-the-same-order");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-with-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-use-a-button-to-perform-an-action".into(),
        "ui-navigate-to-and-activate-a-button".into(),
    ]);
    gap(Dp(24.0));
    activation_demo();
    caption(
        "ui-the-native-button-exposes-an-accessible-label-and-a-description-that-reports-its-activatio",
    );
}

#[tessera]
fn activation_demo() {
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label(I18N.text("ui-perform-action"))
                    .accessibility_description(
                        I18N.format("ui-activations-value", [("v0", (activations.get()).into())]),
                    )
                    .on_click(move || activations.set(activations.get() + 1))
                    .child(move || {
                        text().content(I18N.text("ui-perform-action"));
                    });
                gap(Dp(16.0));
                text()
                    .content(
                        I18N.format("ui-activations-value", [("v0", (activations.get()).into())]),
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn interaction_style() {
    subheading("ui-color-contrast".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-enabled-buttons-need-a-3-1-contrast-ratio-with-the-background-to-meet-accessibility-best-p",
    );
    gap(Dp(16.0));
    prose(
        "ui-this-is-measured-from-the-container-for-elevated-filled-and-tonal-button-styles-and-the-la",
    );
    gap(Dp(24.0));
    contrast_examples();
    caption("ui-higher-contrast-helps-differentiate-elements");
    gap(Dp(48.0));
    subheading("ui-200-text-size".into()).level(4);
    gap(Dp(16.0));
    prose("ui-avoid-excessive-text-wrapping-or-truncation-by-choosing-concise-strings");
    gap(Dp(16.0));
    prose(
        "ui-on-android-button-labels-should-be-kept-concise-enough-to-fit-within-two-lines-after-the-t",
    );
    gap(Dp(24.0));
    text_scale_example();
    guidance(
        "ui-caution",
        "ui-avoid-excessive-text-wrapping-or-truncation-by-choosing-concise-strings-1047",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-rapid-clicks".into()).level(4);
    gap(Dp(16.0));
    prose(
        "ui-on-the-web-you-can-use-a-modified-motion-curve-to-avoid-resonant-effects-from-overlapping",
    );
    gap(Dp(24.0));
    rapid_click_example();
    caption("ui-use-the-modified-motion-curve-if-rapid-click-or-pointer-interactions-are-expected");
}

#[tessera]
fn contrast_examples() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(48.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                column().children(move || {
                    button()
                        .filled()
                        .size(ButtonSize::Medium)
                        .accessibility_label(I18N.text("ui-above-three-to-one"))
                        .on_click(|| {})
                        .child(move || {
                            text().content(I18N.text("ui-share"));
                        });
                    gap(Dp(12.0));
                    text()
                        .content(I18N.text("ui-above-3-1-primary-container-on-surface"))
                        .style(type_style(14.0, 20.0));
                });
                column().children(move || {
                    button()
                        .size(ButtonSize::Medium)
                        .color(scheme.surface_container)
                        .content_color(scheme.outline)
                        .accessibility_label(I18N.text("ui-below-three-to-one"))
                        .on_click(|| {})
                        .child(move || {
                            text().content(I18N.text("ui-share"));
                        });
                    gap(Dp(12.0));
                    text()
                        .content(I18N.text("ui-below-3-1-low-contrast-container-and-label"))
                        .style(type_style(14.0, 20.0));
                });
            });
    }));
}

#[tessera]
fn text_scale_example() {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label(I18N.text("ui-save"))
                    .on_click(|| {})
                    .child(move || {
                        text().content(I18N.text("ui-save"));
                    });
                gap(Dp(12.0));
                prose("ui-a-concise-label-keeps-its-single-line-shape-when-text-is-scaled-up");
                gap(Dp(24.0));
                button()
                    .filled()
                    .size(ButtonSize::Medium)
                    .accessibility_label(I18N.text("ui-save-the-current-document-to-your-device"))
                    .on_click(|| {})
                    .child(move || {
                        text().content(I18N.text("ui-save-the-current-document-to-your-device"));
                    });
                gap(Dp(12.0));
                prose("ui-a-long-label-wraps-or-truncates-once-the-text-size-grows-prefer-a-shorter-string-or-a-wide");
            });
    }));
}

#[tessera]
fn rapid_click_example() {
    let clicks = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .size(ButtonSize::Medium)
                    .accessibility_label(I18N.text("ui-skip-forward"))
                    .accessibility_description(I18N.format("ui-activations-value", [("v0", (clicks.get()).into())]))
                    .on_click(move || clicks.set(clicks.get() + 1))
                    .child(move || {
                        text().content(I18N.text("ui-skip-forward"));
                    });
                gap(Dp(16.0));
                text()
                    .content(I18N.format("ui-press-repeatedly-to-check-the-shape-and-state-animations-activations-value", [("v0", (clicks.get()).into())]))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-navigate-to-a-button".into()],
            vec!["ui-space-or-enter".into(), "ui-activate-a-button".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-in-this-native-example-tab-moves-focus-between-the-two-buttons-and-space-or-enter-activate",
    );
    gap(Dp(24.0));
    keyboard_demo();
}

#[tessera]
fn keyboard_demo() {
    let status = remember(|| "ui-no-activation-yet".to_string());
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-save"))
                            .on_click(move || status.set("ui-save-activated".to_string()))
                            .child(move || {
                                text().content(I18N.text("ui-save"));
                            });
                        button()
                            .outlined()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-discard"))
                            .on_click(move || status.set("ui-discard-activated".to_string()))
                            .child(move || {
                                text().content(I18N.text("ui-discard"));
                            });
                    });
                gap(Dp(16.0));
                text()
                    .content(I18N.text(status.get()))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn labeling() {
    prose(
        "ui-the-accessibility-label-for-a-button-should-match-the-visible-label-text-on-the-button-suc",
    );
    gap(Dp(24.0));
    labeling_example();
    caption("ui-label-next-role-button-the-accessible-label-repeats-the-visible-text");
    gap(Dp(32.0));
    prose(
        "ui-an-icon-inside-a-button-is-decorative-when-the-label-already-names-the-action-provide-the",
    );
}

#[tessera]
fn labeling_example() {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                text()
                    .content(I18N.text("ui-maze-of-mycelium"))
                    .style(type_style(20.0, 28.0));
                gap(Dp(8.0));
                text()
                    .content(I18N.text("ui-from-truffle-hunting-in-the-forests-of-france-to-digging-deep-underground-to-find-vast-myc"))
                    .style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        button()
                            .text()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-back"))
                            .on_click(|| {})
                            .child(move || {
                                text().content(I18N.text("ui-back"));
                            });
                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                        button()
                            .filled()
                            .size(ButtonSize::Medium)
                            .accessibility_label(I18N.text("ui-next"))
                            .on_click(|| {})
                            .child(move || {
                                text().content(I18N.text("ui-next"));
                            });
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        column().children(move || {
                            text()
                                .content(I18N.text("ui-label-next"))
                                .style(type_style(14.0, 20.0));
                            text()
                                .content(I18N.text("ui-role-button"))
                                .style(type_style(14.0, 20.0));
                        });
                    });
            });
    }));
}

#[tessera]
fn baseline_tokens() {
    prose(
        "ui-baseline-token-sets-describe-the-reference-color-typography-and-size-roles-for-a-small-fil",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "md.comp.filled-button.container.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.filled-button.label-text.color".into(),
                "On primary".into(),
            ],
            vec![
                "md.comp.filled-button.container.height".into(),
                "40dp".into(),
            ],
            vec![
                "md.comp.filled-button.container.shape".into(),
                "ui-full-capsule".into(),
            ],
            vec![
                "md.comp.filled-button.label-text".into(),
                "Label large · 14sp / 20sp".into(),
            ],
            vec!["md.comp.filled-button.icon.size".into(), "20dp".into()],
            vec!["md.comp.filled-button.leading-space".into(), "16dp".into()],
            vec!["md.comp.filled-button.trailing-space".into(), "16dp".into()],
            vec![
                "md.comp.filled-button.icon-label-space".into(),
                "8dp".into(),
            ],
            vec![
                "md.comp.filled-button.pressed.state-layer.opacity".into(),
                "10%".into(),
            ],
        ],
    );
}

#[tessera]
fn playground() {
    let style = remember(|| ButtonVariant::Filled);
    let size = remember(|| ButtonSize::Small);
    let shape = remember(|| ButtonShape::Round);
    let toggle = remember(|| false);
    let selected = remember(|| false);
    let enabled = remember(|| true);
    let with_icon = remember(|| false);
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-cycle-color-style"))
                            .on_click(move || style.set(next_style(style.get())))
                            .child(move || {
                                text().content(I18N.format(
                                    "ui-style-value",
                                    [("v0", I18N.text(&style_name(style.get())).into())],
                                ));
                            });
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-cycle-size"))
                            .on_click(move || size.set(next_size(size.get())))
                            .child(move || {
                                text().content(I18N.format(
                                    "ui-size-value",
                                    [("v0", I18N.text(&size_name(size.get())).into())],
                                ));
                            });
                        button()
                            .tonal()
                            .selected(shape.get() == ButtonShape::Square)
                            .accessibility_label(I18N.text("ui-square-shape"))
                            .on_click(move || {
                                shape.set(if shape.get() == ButtonShape::Square {
                                    ButtonShape::Round
                                } else {
                                    ButtonShape::Square
                                });
                            })
                            .child(move || {
                                text().content(I18N.text("ui-square"));
                            });
                        button()
                            .tonal()
                            .selected(toggle.get())
                            .accessibility_label(I18N.text("ui-toggle-variant"))
                            .on_click(move || {
                                let next = !toggle.get();
                                toggle.set(next);
                                if !next {
                                    selected.set(false);
                                }
                            })
                            .child(move || {
                                text().content(I18N.text("ui-toggle"));
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label(I18N.text("ui-disabled"))
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(move || {
                                text().content(I18N.text("ui-disabled"));
                            });
                        button()
                            .tonal()
                            .selected(with_icon.get())
                            .accessibility_label(I18N.text("ui-leading-icon"))
                            .on_click(move || with_icon.set(!with_icon.get()))
                            .child(move || {
                                text().content(I18N.text("ui-icon"));
                            });
                    });
                gap(Dp(32.0));
                button()
                    .variant(style.get())
                    .size(size.get())
                    .shape_style(shape.get())
                    .selected_optional(if toggle.get() && style.get() != ButtonVariant::Text {
                        Some(selected.get())
                    } else {
                        None
                    })
                    .enabled(enabled.get())
                    .accessibility_label(I18N.text("ui-live-button"))
                    .accessibility_description(
                        I18N.format("ui-activations-value", [("v0", (activations.get()).into())]),
                    )
                    .leading_icon_optional(if with_icon.get() {
                        Some(RenderSlot::new(|| {
                            icon().painter(filled::EDIT_SVG);
                        }))
                    } else {
                        None
                    })
                    .on_click(move || {
                        activations.set(activations.get() + 1);
                        if toggle.get() && style.get() != ButtonVariant::Text {
                            selected.set(!selected.get());
                        }
                    })
                    .child(move || {
                        text().content(I18N.text("ui-enable"));
                    });
                gap(Dp(24.0));
                text()
                    .content(
                        I18N.format(
                            "ui-activations-value-value",
                            [
                                ("v0", (activations.get()).into()),
                                (
                                    "v1",
                                    I18N.text(
                                        &(if toggle.get() && style.get() != ButtonVariant::Text {
                                            if selected.get() {
                                                "ui-selected"
                                            } else {
                                                "ui-unselected"
                                            }
                                        } else {
                                            "ui-default-action"
                                        }),
                                    )
                                    .into(),
                                ),
                            ],
                        ),
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
}
