//! Interactive button group specifications.
//!
//! ## Usage
//!
//! Compare standard and connected groups, selection policies and measurements.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::{ButtonShape, ButtonVariant, button},
    button_groups::{
        ButtonGroupsController, ButtonGroupsSelectionMode, ButtonGroupsSize, ButtonGroupsStyle,
        button_groups,
    },
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::{Asset, filled},
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_foundation::modifier::{
    InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs,
};
use tessera_ui::{CallbackWith, Color, Dp, Modifier, RenderSlot, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    buttons::{badge, bullets, caption, marked, prose},
    gap, panel, spec_table, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const CAUTION_COLOR: Color = Color::from_rgb(0.705, 0.365, 0.0);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);

const SIZES: [ButtonGroupsSize; 5] = [
    ButtonGroupsSize::ExtraSmall,
    ButtonGroupsSize::Small,
    ButtonGroupsSize::Medium,
    ButtonGroupsSize::Large,
    ButtonGroupsSize::ExtraLarge,
];

const STATES: [&str; 5] = [
    "ui-enabled",
    "ui-disabled",
    "ui-hovered",
    "ui-focused",
    "ui-pressed",
];

fn size_name(size: ButtonGroupsSize) -> &'static str {
    match size {
        ButtonGroupsSize::ExtraSmall => "ui-extra-small",
        ButtonGroupsSize::Small => "ui-small",
        ButtonGroupsSize::Medium => "ui-medium",
        ButtonGroupsSize::Large => "ui-large",
        ButtonGroupsSize::ExtraLarge => "ui-extra-large",
    }
}

fn token_size_name(size: ButtonGroupsSize) -> &'static str {
    match size {
        ButtonGroupsSize::ExtraSmall => "xsmall",
        ButtonGroupsSize::Small => "small",
        ButtonGroupsSize::Medium => "medium",
        ButtonGroupsSize::Large => "large",
        ButtonGroupsSize::ExtraLarge => "xlarge",
    }
}

fn style_name(style: ButtonGroupsStyle) -> &'static str {
    match style {
        ButtonGroupsStyle::Standard => "ui-standard",
        ButtonGroupsStyle::Connected => "ui-connected",
    }
}

fn next_size(size: ButtonGroupsSize) -> ButtonGroupsSize {
    match size {
        ButtonGroupsSize::ExtraSmall => ButtonGroupsSize::Small,
        ButtonGroupsSize::Small => ButtonGroupsSize::Medium,
        ButtonGroupsSize::Medium => ButtonGroupsSize::Large,
        ButtonGroupsSize::Large => ButtonGroupsSize::ExtraLarge,
        ButtonGroupsSize::ExtraLarge => ButtonGroupsSize::ExtraSmall,
    }
}

fn state_presentation(state: usize) -> InteractionVisualState {
    match state {
        2 => InteractionVisualState::Hover,
        3 => InteractionVisualState::Focus,
        4 => InteractionVisualState::Pressed,
        _ => InteractionVisualState::Rest,
    }
}

/// One button inside a demonstration group.
#[derive(Clone, PartialEq)]
struct Item {
    icon: Option<Asset>,
    label: Option<&'static str>,
    description: &'static str,
}

fn labelled(label: &'static str) -> Item {
    Item {
        icon: None,
        label: Some(label),
        description: label,
    }
}

fn icon_label(icon: Asset, label: &'static str) -> Item {
    Item {
        icon: Some(icon),
        label: Some(label),
        description: label,
    }
}

fn icon_only(icon: Asset, description: &'static str) -> Item {
    Item {
        icon: Some(icon),
        label: None,
        description,
    }
}

fn mixed_items() -> Vec<Item> {
    vec![
        icon_only(filled::EDIT_SVG, "ui-edit"),
        icon_label(filled::EDIT_SVG, "ui-label"),
        icon_only(filled::EDIT_SVG, "ui-draw"),
    ]
}

fn label_items() -> Vec<Item> {
    vec![
        icon_label(filled::STAR_SVG, "ui-label"),
        icon_label(filled::STAR_SVG, "ui-label-2"),
        icon_label(filled::STAR_SVG, "ui-label-3"),
    ]
}

#[tessera]
pub(super) fn button_groups_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-button-groups",
        "ui-button-groups-organize-buttons-and-add-interactions-between-them",
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
                title: "ui-selection-activation",
                content: RenderSlot::new(selection_and_activation),
            },
            SpecSection {
                title: "ui-states",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-density",
                content: RenderSlot::new(density),
            },
        ],
    )
    .route_optional(route)
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![],
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
                    title: "ui-anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
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

/// Numbered caption list under a figure, matching the figure markers.
#[tessera]
fn legend(items: Vec<String>) {
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
                        text(I18N.text(&(item))).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// A figure row with a leading numbered marker.

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
                    text(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(note)))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

/// Renders one native button group with explicit items and configuration.
#[tessera]
fn group(
    items: Vec<Item>,
    style: ButtonGroupsStyle,
    size: Option<ButtonGroupsSize>,
    shape: Option<ButtonShape>,
    variant: Option<ButtonVariant>,
    mode: Option<ButtonGroupsSelectionMode>,
    required: Option<bool>,
    enabled: Option<bool>,
    selected: Option<Vec<usize>>,
    presentation: Option<Vec<InteractionVisualState>>,
    #[prop(into)] label: Option<String>,
) {
    let initial = selected.unwrap_or_default();
    let state = remember(move || {
        let mut state = ButtonGroupsController::new();
        for index in initial {
            state.set_selected(index, true);
        }
        state
    });
    let builder = button_groups()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            label: Some(I18N.text(label.as_deref().unwrap_or("ui-button-group"))),
            ..Default::default()
        }))
        .style(style)
        .size(size.unwrap_or(ButtonGroupsSize::Small))
        .shape_style(shape.unwrap_or(ButtonShape::Round))
        .variant(variant.unwrap_or(ButtonVariant::Filled))
        .selection_mode(mode.unwrap_or(ButtonGroupsSelectionMode::Multiple))
        .selection_required(required.unwrap_or(false))
        .enabled(enabled.unwrap_or(true))
        .presentation_states(presentation.unwrap_or_default())
        .controller(state);
    for item in items {
        builder.child_shared(
            RenderSlot::new(move || {
                let item = item.clone();
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        label: Some(I18N.text(item.description)),
                        ..Default::default()
                    }))
                    .children(move || {
                        if let Some(asset) = item.icon {
                            icon().painter(asset);
                        }
                        if item.icon.is_some() && item.label.is_some() {
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                        }
                        if let Some(label) = item.label {
                            text(I18N.text(label));
                        }
                    });
            }),
            CallbackWith::new(|_| {}),
        );
    }
}

#[tessera]
fn overview() {
    bullets(vec![
        "ui-two-variants-standard-and-connected".into(),
        "ui-applies-shape-morph-when-pressed-and-selected".into(),
        "ui-connected-button-groups-replace-the-segmented-button".into(),
        "ui-works-with-all-button-sizes-xs-s-m-l-and-xl".into(),
        "ui-support-for-single-select-multi-select-and-selection-required".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(
                    vec![
                        icon_only(filled::BLUETOOTH_SVG, "ui-bluetooth"),
                        icon_only(filled::ALARM_SVG, "ui-alarm"),
                        icon_label(filled::MOOD_SVG, "ui-focus"),
                        icon_only(filled::FLASHLIGHT_ON_SVG, "ui-flashlight"),
                        icon_only(filled::WIFI_SVG, "Wi-Fi"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .size(ButtonGroupsSize::Medium)
                .selected(vec![4])
                .label(I18N.text("ui-quick-settings"));
                gap(Dp(24.0));
                group(
                    vec![
                        labelled("8 oz"),
                        labelled("12 oz"),
                        labelled("16 oz"),
                        labelled("20 oz"),
                    ],
                    ButtonGroupsStyle::Connected,
                )
                .mode(ButtonGroupsSelectionMode::Single)
                .required(true)
                .selected(vec![0])
                .label(I18N.text("ui-cup-size"));
            });
    }));
    caption("ui-button-groups-can-contain-buttons-and-icon-buttons");
}

#[tessera]
fn variants() {
    panel(RenderSlot::new(move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        group(
                            vec![
                                icon_only(filled::BLUETOOTH_SVG, "ui-bluetooth"),
                                icon_only(filled::ALARM_SVG, "ui-alarm"),
                                icon_only(filled::LINK_SVG, "ui-link"),
                                icon_only(filled::WIFI_SVG, "Wi-Fi"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![2])
                        .label(I18N.text("ui-standard-icon-buttons"));
                        gap(Dp(12.0));
                        group(
                            vec![
                                labelled("ui-label"),
                                labelled("ui-label"),
                                labelled("ui-label"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![1])
                        .label(I18N.text("ui-standard-label-buttons"));
                        gap(Dp(12.0));
                        group(mixed_items(), ButtonGroupsStyle::Standard)
                            .size(ButtonGroupsSize::ExtraSmall)
                            .label(I18N.text("ui-standard-mixed-buttons"));
                        gap(Dp(16.0));
                        badge(1);
                    });
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        group(
                            vec![
                                icon_only(filled::STAR_SVG, "ui-favorites"),
                                icon_only(filled::HOME_SVG, "ui-home"),
                                icon_only(filled::SEARCH_SVG, "ui-search"),
                                icon_only(filled::SETTINGS_SVG, "ui-settings"),
                            ],
                            ButtonGroupsStyle::Connected,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .mode(ButtonGroupsSelectionMode::Single)
                        .selected(vec![1])
                        .label(I18N.text("ui-connected-icon-buttons"));
                        gap(Dp(12.0));
                        group(label_items(), ButtonGroupsStyle::Connected)
                            .size(ButtonGroupsSize::ExtraSmall)
                            .mode(ButtonGroupsSelectionMode::Single)
                            .selected(vec![0])
                            .label(I18N.text("ui-connected-label-buttons"));
                        gap(Dp(16.0));
                        badge(2);
                    });
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "ui-standard-button-group".into(),
        "ui-connected-button-group".into(),
    ]);
}

#[tessera]
fn configurations() {
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for (index, size) in SIZES.into_iter().enumerate() {
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            group(
                                vec![
                                    icon_only(filled::STAR_SVG, "ui-favorites"),
                                    icon_only(filled::ALARM_SVG, "ui-alarm"),
                                    icon_only(filled::LINK_SVG, "ui-link"),
                                ],
                                ButtonGroupsStyle::Standard,
                            )
                            .size(size)
                            .selected(vec![1])
                            .label(I18N.format(
                                "ui-value-group",
                                [("v0", I18N.text(&size_name(size)).into())],
                            ));
                        }),
                    );
                    gap(Dp(16.0));
                }
                marked(
                    6,
                    RenderSlot::new(move || {
                        column().children(move || {
                            group(label_items(), ButtonGroupsStyle::Connected)
                                .mode(ButtonGroupsSelectionMode::Single)
                                .selected(vec![0])
                                .label(I18N.text("ui-single-select-group"));
                            gap(Dp(8.0));
                            group(label_items(), ButtonGroupsStyle::Connected)
                                .selected(vec![0, 2])
                                .label(I18N.text("ui-multi-select-group"));
                        });
                    }),
                );
                gap(Dp(16.0));
                marked(
                    7,
                    RenderSlot::new(move || {
                        column().children(move || {
                            group(mixed_items(), ButtonGroupsStyle::Standard)
                                .shape(ButtonShape::Round)
                                .label(I18N.text("ui-round-group"));
                            gap(Dp(8.0));
                            group(mixed_items(), ButtonGroupsStyle::Standard)
                                .shape(ButtonShape::Square)
                                .label(I18N.text("ui-square-group"));
                        });
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    prose("ui-configurations-for-both-variants-of-button-groups");
    gap(Dp(8.0));
    legend(vec![
        "ui-extra-small".into(),
        "ui-small".into(),
        "ui-medium".into(),
        "ui-large".into(),
        "ui-extra-large".into(),
        "ui-single-select-and-multi-select".into(),
        "ui-round-and-square".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-live-configuration".into());
    gap(Dp(8.0));
    prose(
        "ui-change-the-configuration-then-click-a-choice-focus-enters-the-group-s-first-button-and-the",
    );
    gap(Dp(24.0));
    playground();
}

#[tessera]
fn playground() {
    let style = remember(|| ButtonGroupsStyle::Standard);
    let size = remember(|| ButtonGroupsSize::Medium);
    let square = remember(|| false);
    let multiple = remember(|| false);
    let required = remember(|| false);
    let enabled = remember(|| true);
    let icons = remember(|| true);
    let state = remember(ButtonGroupsController::new);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(8.0))
                    .line_spacing(Dp(8.0))
                    .children(move || {
                        button()
                            .tonal()
                            .selected(style.get() == ButtonGroupsStyle::Connected)
                            .accessibility_label(I18N.text("ui-connected-layout"))
                            .on_click(move || {
                                style.set(if style.get() == ButtonGroupsStyle::Connected {
                                    ButtonGroupsStyle::Standard
                                } else {
                                    ButtonGroupsStyle::Connected
                                });
                            })
                            .child(move || {
                                text(I18N.text("ui-connected"));
                            });
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-cycle-size"))
                            .on_click(move || size.set(next_size(size.get())))
                            .child(move || {
                                text(I18N.format(
                                    "ui-size-value",
                                    [("v0", I18N.text(&size_name(size.get())).into())],
                                ));
                            });
                        button()
                            .tonal()
                            .selected(square.get())
                            .accessibility_label(I18N.text("ui-square-shape"))
                            .on_click(move || square.set(!square.get()))
                            .child(move || {
                                text(I18N.text("ui-square"));
                            });
                        button()
                            .tonal()
                            .selected(multiple.get())
                            .accessibility_label(I18N.text("ui-multiple-selection"))
                            .on_click(move || multiple.set(!multiple.get()))
                            .child(move || {
                                text(I18N.text("ui-multiple"));
                            });
                        button()
                            .tonal()
                            .selected(required.get())
                            .accessibility_label(I18N.text("ui-selection-required"))
                            .on_click(move || required.set(!required.get()))
                            .child(move || {
                                text(I18N.text("ui-required"));
                            });
                        button()
                            .tonal()
                            .selected(!enabled.get())
                            .accessibility_label(I18N.text("ui-disabled-group"))
                            .on_click(move || enabled.set(!enabled.get()))
                            .child(move || {
                                text(I18N.text("ui-disabled"));
                            });
                        button()
                            .tonal()
                            .selected(icons.get())
                            .accessibility_label(I18N.text("ui-item-icons"))
                            .on_click(move || icons.set(!icons.get()))
                            .child(move || {
                                text(I18N.text("ui-icons"));
                            });
                    });
                gap(Dp(32.0));
                button_groups()
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        label: Some(I18N.text("ui-configured-group")),
                        ..Default::default()
                    }))
                    .style(style.get())
                    .size(size.get())
                    .shape_style(if square.get() {
                        ButtonShape::Square
                    } else {
                        ButtonShape::Round
                    })
                    .selection_mode(if multiple.get() {
                        ButtonGroupsSelectionMode::Multiple
                    } else {
                        ButtonGroupsSelectionMode::Single
                    })
                    .selection_required(required.get())
                    .enabled(enabled.get())
                    .controller(state)
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text(I18N.text("ui-one"));
                                });
                            } else {
                                text(I18N.text("ui-one"));
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text(I18N.text("ui-two"));
                                });
                            } else {
                                text(I18N.text("ui-two"));
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            if icons.get() {
                                row().children(move || {
                                    icon().painter(filled::EDIT_SVG);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    text(I18N.text("ui-three"));
                                });
                            } else {
                                text(I18N.text("ui-three"));
                            }
                        }),
                        CallbackWith::new(|_| {}),
                    );
                gap(Dp(24.0));
                text(
                    I18N.format(
                        "ui-value-value-value-selection-value",
                        [
                            ("v0", I18N.text(&style_name(style.get())).into()),
                            ("v1", I18N.text(&size_name(size.get())).into()),
                            (
                                "v2",
                                I18N.text(
                                    &(if multiple.get() {
                                        "ui-multiple-7083"
                                    } else {
                                        "ui-single-7084"
                                    }),
                                )
                                .into(),
                            ),
                            (
                                "v3",
                                (0..3)
                                    .filter(|index| state.with(|state| state.is_selected(*index)))
                                    .map(|index| (index + 1).to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
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
fn tokens() {
    let connected = remember(|| false);
    let size = remember(|| ButtonGroupsSize::ExtraSmall);
    prose(
        "ui-standard-and-connected-button-group-tokens-are-organized-by-size-select-the-variant-and-si",
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .selected(connected.get())
                .accessibility_label(I18N.text("ui-connected-token-set"))
                .on_click(move || connected.set(!connected.get()))
                .child(move || {
                    text(I18N.text("ui-connected"));
                });
            button()
                .text()
                .accessibility_label(I18N.text("ui-cycle-token-set-size"))
                .on_click(move || size.set(next_size(size.get())))
                .child(move || {
                    text(I18N.format(
                        "ui-size-value",
                        [("v0", I18N.text(&size_name(size.get())).into())],
                    ));
                });
        });
    gap(Dp(16.0));
    let index = SIZES
        .iter()
        .position(|value| *value == size.get())
        .unwrap_or(0);
    let token = token_size_name(size.get());
    let height = ["32dp", "40dp", "56dp", "96dp", "136dp"][index];
    if connected.get() {
        subheading(
            I18N.format(
                "ui-button-group-connected-size-value",
                [(
                    "v0",
                    I18N.text(
                        &({
                            let mut name = token.to_string();
                            name[..1].make_ascii_uppercase();
                            name
                        }),
                    )
                    .into(),
                )],
            ),
        );
        gap(Dp(16.0));
        spec_table(
            vec!["ui-token".into(), "ui-value".into()],
            vec![
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-container-height",
                        [("v0", I18N.text(&token).into())],
                    ),
                    height.into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-space-between-buttons",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "2dp".into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-container-shape",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "ui-full".into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-inner-corner-size",
                        [("v0", I18N.text(&token).into())],
                    ),
                    ["8dp", "8dp", "8dp", "16dp", "20dp"][index].into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-pressed-inner-corner-size",
                        [("v0", I18N.text(&token).into())],
                    ),
                    ["4dp", "4dp", "4dp", "12dp", "16dp"][index].into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-connected-value-selected-inner-corner-size",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "50%".into(),
                ],
            ],
        );
    } else {
        subheading(
            I18N.format(
                "ui-button-group-standard-size-value",
                [(
                    "v0",
                    I18N.text(
                        &({
                            let mut name = token.to_string();
                            name[..1].make_ascii_uppercase();
                            name
                        }),
                    )
                    .into(),
                )],
            ),
        );
        gap(Dp(16.0));
        spec_table(
            vec!["ui-token".into(), "ui-value".into()],
            vec![
                vec![
                    I18N.format(
                        "ui-button-group-value-container-height",
                        [("v0", I18N.text(&token).into())],
                    ),
                    height.into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-value-between-space",
                        [("v0", I18N.text(&token).into())],
                    ),
                    ["18dp", "12dp", "8dp", "8dp", "8dp"][index].into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-value-pressed-motion-spring-dampening",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "0.9".into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-value-pressed-motion-spring-stiffness",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "1400".into(),
                ],
                vec![
                    I18N.format(
                        "ui-button-group-value-pressed-width-multiplier",
                        [("v0", I18N.text(&token).into())],
                    ),
                    "15%".into(),
                ],
            ],
        );
    }
}

#[tessera]
fn anatomy() {
    prose(
        "ui-button-groups-are-invisible-containers-that-add-padding-between-buttons-and-modify-button",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                marked(
                    1,
                    RenderSlot::new(move || {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .border_with_shape(
                                        Dp(1.0),
                                        INK,
                                        Shape::rounded_rectangle(Dp(4.0)),
                                    )
                                    .padding_all(Dp(4.0)),
                            )
                            .children(move || {
                                group(
                                    vec![
                                        icon_only(filled::EDIT_SVG, "ui-edit"),
                                        icon_only(filled::EDIT_SVG, "ui-draw"),
                                        icon_label(filled::EDIT_SVG, "ui-label"),
                                        icon_only(filled::EDIT_SVG, "ui-sketch"),
                                        icon_only(filled::EDIT_SVG, "ui-write"),
                                    ],
                                    ButtonGroupsStyle::Standard,
                                )
                                .size(ButtonGroupsSize::ExtraSmall)
                                .variant(ButtonVariant::Tonal)
                                .label(I18N.text("ui-standard-container"));
                            });
                    }),
                );
                gap(Dp(24.0));
                marked(
                    1,
                    RenderSlot::new(move || {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .border_with_shape(
                                        Dp(1.0),
                                        INK,
                                        Shape::rounded_rectangle(Dp(4.0)),
                                    )
                                    .padding_all(Dp(4.0)),
                            )
                            .children(move || {
                                group(
                                    vec![
                                        icon_label(filled::STAR_SVG, "ui-label"),
                                        icon_label(filled::STAR_SVG, "ui-label-2"),
                                        icon_label(filled::STAR_SVG, "ui-label-3"),
                                        icon_label(filled::STAR_SVG, "ui-label-4"),
                                    ],
                                    ButtonGroupsStyle::Connected,
                                )
                                .size(ButtonGroupsSize::ExtraSmall)
                                .variant(ButtonVariant::Tonal)
                                .label(I18N.text("ui-connected-container"));
                            });
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    legend(vec!["ui-container".into()]);
    gap(Dp(48.0));
    subheading("ui-common-layouts".into());
    gap(Dp(8.0));
    prose("ui-mix-and-match-buttons-and-icon-buttons-for-different-scenarios");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                marked(
                    1,
                    RenderSlot::new(move || {
                        group(
                            vec![
                                icon_label(filled::STAR_SVG, "ui-start"),
                                icon_label(filled::DIRECTIONS_SVG, "ui-directions"),
                                icon_label(filled::SHARE_SVG, "ui-share"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .variant(ButtonVariant::Tonal)
                        .label(I18N.text("ui-label-buttons"));
                    }),
                );
                gap(Dp(16.0));
                marked(
                    2,
                    RenderSlot::new(move || {
                        group(
                            vec![
                                icon_only(filled::SKIP_PREVIOUS_SVG, "ui-previous"),
                                icon_label(filled::PLAY_ARROW_SVG, "ui-play"),
                                icon_only(filled::SKIP_NEXT_SVG, "ui-next"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .selected(vec![1])
                        .label(I18N.text("ui-label-buttons-and-icon-buttons"));
                    }),
                );
                gap(Dp(16.0));
                marked(
                    3,
                    RenderSlot::new(move || {
                        group(
                            vec![
                                icon_only(filled::FORMAT_BOLD_SVG, "ui-bold"),
                                icon_only(filled::FORMAT_ITALIC_SVG, "ui-italic"),
                                icon_only(filled::FORMAT_UNDERLINED_SVG, "ui-underline"),
                                icon_only(filled::FORMAT_COLOR_TEXT_SVG, "ui-text-color"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::ExtraSmall)
                        .selected(vec![0])
                        .label(I18N.text("ui-extra-small-icon-buttons"));
                    }),
                );
                gap(Dp(16.0));
                marked(
                    4,
                    RenderSlot::new(move || {
                        group(
                            vec![
                                icon_only(filled::ALARM_SVG, "ui-alarm"),
                                icon_only(filled::VOLUME_OFF_SVG, "ui-mute"),
                                icon_only(filled::WIFI_SVG, "Wi-Fi"),
                            ],
                            ButtonGroupsStyle::Standard,
                        )
                        .size(ButtonGroupsSize::Large)
                        .variant(ButtonVariant::Tonal)
                        .label(I18N.text("ui-large-icon-buttons"));
                    }),
                );
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "ui-label-buttons".into(),
        "ui-label-buttons-and-icon-buttons".into(),
        "ui-extra-small-icon-buttons".into(),
        "ui-large-icon-buttons".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-color".into());
    gap(Dp(8.0));
    prose(
        "ui-button-groups-have-no-color-properties-they-can-use-the-default-button-or-toggle-button-co",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for (index, variant) in [
                    ButtonVariant::Filled,
                    ButtonVariant::Tonal,
                    ButtonVariant::Outlined,
                    ButtonVariant::Elevated,
                ]
                .into_iter()
                .enumerate()
                {
                    marked(
                        index + 1,
                        RenderSlot::new(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    if variant == ButtonVariant::Elevated {
                                        text(I18N.text("ui-elevated-style-not-recommended"))
                                            .color(INK)
                                            .style(type_style(14.0, 20.0));
                                    } else {
                                        group(mixed_items(), ButtonGroupsStyle::Standard)
                                            .size(ButtonGroupsSize::ExtraSmall)
                                            .variant(variant)
                                            .selected(vec![1])
                                            .label(I18N.text("ui-standard-color-sample"));
                                    }
                                    spacer().modifier(Modifier::new().width(Dp(40.0)));
                                    group(label_items(), ButtonGroupsStyle::Connected)
                                        .size(ButtonGroupsSize::ExtraSmall)
                                        .variant(variant)
                                        .mode(ButtonGroupsSelectionMode::Single)
                                        .selected(vec![0])
                                        .label(I18N.text("ui-connected-color-sample"));
                                });
                        }),
                    );
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    legend(vec![
        "ui-filled".into(),
        "ui-tonal".into(),
        "ui-outlined".into(),
        "ui-elevated".into(),
    ]);
}

#[tessera]
fn selection_and_activation() {
    prose(
        "ui-standard-button-groups-add-interaction-between-adjacent-buttons-when-a-button-is-selected",
    );
    gap(Dp(8.0));
    prose(
        "ui-this-interaction-changes-the-width-shape-and-padding-of-the-selected-or-activated-button-w",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                icon_only(filled::BLUETOOTH_SVG, "ui-bluetooth"),
                icon_only(filled::ALARM_SVG, "ui-alarm"),
                icon_only(filled::LINK_SVG, "ui-link"),
                icon_only(filled::WIFI_SVG, "Wi-Fi"),
            ],
            ButtonGroupsStyle::Standard,
        )
        .size(ButtonGroupsSize::Large)
        .variant(ButtonVariant::Tonal)
        .selected(vec![2])
        .label(I18N.text("ui-standard-activation"));
    }));
    caption(
        "ui-a-selected-button-changes-shape-and-briefly-changes-the-width-of-itself-and-adjacent-butto",
    );
    gap(Dp(32.0));
    prose(
        "ui-connected-button-groups-don-t-add-any-interaction-between-buttons-when-selected-or-activat",
    );
    gap(Dp(8.0));
    prose("ui-they-only-affect-the-shape-of-the-button-being-selected-or-activated");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                labelled("ui-label"),
                labelled("ui-label"),
                labelled("ui-label"),
            ],
            ButtonGroupsStyle::Connected,
        )
        .variant(ButtonVariant::Tonal)
        .selected(vec![2])
        .label(I18N.text("ui-connected-selection"));
    }));
    caption("ui-a-selected-button-changes-shape-without-affecting-adjacent-buttons");
}

/// One labelled state row; `state` indexes [`STATES`], `number` is its legend
/// marker, and pressed rows show the pressed shape and width.
#[tessera]
fn state_row(style: ButtonGroupsStyle, state: usize, number: usize, selected: bool) {
    marked(
        number,
        RenderSlot::new(move || {
            let mut presentation = vec![InteractionVisualState::Rest; 3];
            presentation[1] = state_presentation(state);
            group(
                vec![
                    icon_label(filled::EDIT_SVG, STATES[state]),
                    icon_label(filled::EDIT_SVG, "ui-label"),
                    icon_label(filled::EDIT_SVG, "ui-label"),
                ],
                style,
            )
            .size(ButtonGroupsSize::Small)
            .variant(ButtonVariant::Tonal)
            .enabled(state != 1)
            .presentation(presentation)
            .selected(if selected { vec![1] } else { Vec::new() })
            .label(I18N.format(
                "ui-value-state",
                [("v0", I18N.text(&(STATES[state])).into())],
            ));
        }),
    );
    gap(Dp(12.0));
}

#[tessera]
fn state_figure(style: ButtonGroupsStyle, selected: bool, states: Vec<usize>) {
    panel(RenderSlot::new(move || {
        let states = states.clone();
        column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move || {
                for (index, state) in states.clone().into_iter().enumerate() {
                    state_row(style, state, index + 1, selected);
                }
            });
    }));
    gap(Dp(16.0));
}

#[tessera]
fn states() {
    subheading("ui-standard-button-group".into());
    gap(Dp(16.0));
    prose(
        "ui-when-a-button-is-pressed-standard-button-groups-modify-the-width-and-shape-of-that-button",
    );
    gap(Dp(24.0));
    state_figure(ButtonGroupsStyle::Standard, false, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(32.0));
    prose(
        "ui-when-a-toggle-button-is-selected-in-a-standard-button-group-its-shape-should-change-betwee",
    );
    gap(Dp(24.0));
    state_figure(ButtonGroupsStyle::Standard, true, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(48.0));
    subheading("ui-connected-button-group".into());
    gap(Dp(16.0));
    prose(
        "ui-connected-button-groups-have-different-shape-changes-than-standard-button-groups-selecting",
    );
    gap(Dp(24.0));
    prose("ui-connected-button-group-unselected-states");
    gap(Dp(16.0));
    state_figure(ButtonGroupsStyle::Connected, false, vec![0, 1, 2, 3, 4]);
    legend(STATES.iter().map(|state| (*state).to_string()).collect());
    gap(Dp(32.0));
    prose("ui-connected-button-group-selected-states");
    gap(Dp(16.0));
    state_figure(ButtonGroupsStyle::Connected, true, vec![0, 2, 3, 4]);
    legend(vec![
        "ui-enabled".into(),
        "ui-hovered".into(),
        "ui-focused".into(),
        "ui-pressed".into(),
    ]);
}

#[tessera]
fn measurements() {
    subheading("ui-standard-button-group".into());
    gap(Dp(16.0));
    prose(
        "ui-standard-groups-apply-padding-between-all-buttons-the-amount-of-padding-changes-based-on-b",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for size in SIZES {
                    group(mixed_items(), ButtonGroupsStyle::Standard)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .label(I18N.format(
                            "ui-value-padding",
                            [("v0", I18N.text(&size_name(size)).into())],
                        ));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose("ui-standard-button-group-inner-padding");
    bullets(vec![
        "XS: 18dp".into(),
        "S: 12dp".into(),
        "M: 8dp".into(),
        "L: 8dp".into(),
        "XL: 8dp".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-connected-button-group".into());
    gap(Dp(16.0));
    prose(
        "ui-for-all-connected-button-groups-use-2dp-padding-this-provides-visual-consistency-at-scale",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for size in SIZES {
                    group(label_items(), ButtonGroupsStyle::Connected)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .label(I18N.format(
                            "ui-value-round-connected",
                            [("v0", I18N.text(&size_name(size)).into())],
                        ));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose(
        "ui-round-connected-button-group-inner-padding-is-2dp-at-every-size-the-outer-shape-is-fully-r",
    );
    bullets(vec![
        "XS: 4dp".into(),
        "S: 8dp".into(),
        "M: 8dp".into(),
        "L: 16dp".into(),
        "XL: 20dp".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for size in SIZES {
                    group(label_items(), ButtonGroupsStyle::Connected)
                        .size(size)
                        .shape(ButtonShape::Square)
                        .variant(ButtonVariant::Tonal)
                        .label(I18N.format(
                            "ui-value-square-connected",
                            [("v0", I18N.text(&size_name(size)).into())],
                        ));
                    gap(Dp(16.0));
                }
            });
    }));
    gap(Dp(16.0));
    prose(
        "ui-square-connected-button-group-inner-padding-is-2dp-at-every-size-the-outer-shape-has-the-f",
    );
    bullets(vec![
        "XS: 4dp".into(),
        "S: 8dp".into(),
        "M: 8dp".into(),
        "L: 16dp".into(),
        "XL: 20dp".into(),
    ]);
    gap(Dp(48.0));
    subheading("ui-minimum-widths".into());
    gap(Dp(16.0));
    prose(
        "ui-extra-small-and-small-connected-button-groups-have-48dp-target-areas-and-a-minimum-width-o",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for size in [ButtonGroupsSize::ExtraSmall, ButtonGroupsSize::Small] {
                    column()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            group(
                                vec![
                                    icon_only(filled::STAR_SVG, "ui-favorites"),
                                    icon_only(filled::ALARM_SVG, "ui-alarm"),
                                    icon_only(filled::LINK_SVG, "ui-link"),
                                ],
                                ButtonGroupsStyle::Connected,
                            )
                            .size(size)
                            .variant(ButtonVariant::Tonal)
                            .label(I18N.format(
                                "ui-value-minimum-width",
                                [("v0", I18N.text(&size_name(size)).into())],
                            ));
                            gap(Dp(12.0));
                            text(I18N.text(size_name(size)))
                                .color(INK)
                                .style(type_style(14.0, 20.0));
                        });
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                }
            });
    }));
}

#[tessera]
fn density() {
    prose("ui-button-groups-adapt-to-density-of-the-buttons-inside");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(label_items(), ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .variant(ButtonVariant::Tonal)
                    .label(I18N.text("ui-default-density"));
                gap(Dp(16.0));
                group(label_items(), ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .variant(ButtonVariant::Tonal)
                    .label(I18N.text("ui-reduced-height"));
            });
    }));
    caption(
        "ui-button-groups-adapt-to-the-height-of-the-buttons-inside-including-when-density-is-applied",
    );
}

#[tessera]
fn usage() {
    prose("ui-there-are-two-variants-of-button-groups-standard-and-connected");
    gap(Dp(16.0));
    prose(
        "ui-standard-button-groups-add-interaction-between-adjacent-buttons-so-they-respond-to-each-ot",
    );
    bullets(vec![
        "ui-the-selected-button-changes-shape-and-width".into(),
        "ui-a-selected-toggle-button-also-changes-color".into(),
        "ui-adjacent-buttons-move-and-temporarily-change-width".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                icon_only(filled::MIC_SVG, "ui-microphone"),
                icon_only(filled::VIDEOCAM_SVG, "ui-camera"),
                icon_only(filled::MOOD_SVG, "ui-reactions"),
                icon_only(filled::MORE_VERT_SVG, "ui-more"),
            ],
            ButtonGroupsStyle::Standard,
        )
        .size(ButtonGroupsSize::Medium)
        .variant(ButtonVariant::Tonal)
        .selected(vec![1])
        .label(I18N.text("ui-call-controls"));
    }));
    caption("ui-standard-button-groups-add-interaction-between-adjacent-buttons");
    gap(Dp(48.0));
    subheading("ui-button-groups-add-more-expression-to-a-product".into());
    gap(Dp(16.0));
    prose(
        "ui-mix-and-match-the-different-button-variants-widths-and-colors-to-emphasize-what-s-importan",
    );
    gap(Dp(16.0));
    prose(
        "ui-by-default-all-buttons-in-a-standard-group-should-be-the-same-size-xs-to-xl-and-shape-roun",
    );
    bullets(vec![
        "ui-only-use-multiple-sizes-in-a-group-for-hero-moments".into(),
        "ui-avoid-mixing-sizes-frequently".into(),
        "ui-only-use-a-different-shape-in-a-group-when-a-button-is-selected-or-to-add-meaning-or-contr".into(),
    ]);
    guidance(
        "Do",
        "ui-use-the-same-shapes-for-buttons-in-a-group-but-change-other-properties-like-width-and-colo",
        DO_COLOR,
    );
    guidance(
        "ui-caution",
        "ui-reserve-shape-differences-in-button-groups-for-key-interactions",
        CAUTION_COLOR,
    );
    gap(Dp(48.0));
    subheading("ui-connected-button-groups".into());
    gap(Dp(16.0));
    prose(
        "ui-connected-button-groups-help-people-select-options-switch-views-or-sort-elements-in-a-page",
    );
    gap(Dp(8.0));
    prose("ui-they-behave-similarly-to-standard-groups-except-they-don-t-affect-adjacent-buttons");
    gap(Dp(8.0));
    prose(
        "ui-connected-groups-should-replace-the-baseline-segmented-button-which-is-no-longer-recommend",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                labelled("ui-recent"),
                labelled("ui-starred"),
                labelled("ui-shared"),
            ],
            ButtonGroupsStyle::Connected,
        )
        .mode(ButtonGroupsSelectionMode::Single)
        .required(true)
        .selected(vec![0])
        .label(I18N.text("ui-views"));
    }));
    caption("ui-connected-button-groups-can-be-used-to-toggle-between-similar-actions");
    gap(Dp(32.0));
    prose(
        "ui-use-connected-button-groups-when-the-button-content-is-related-and-buttons-can-be-selected",
    );
    caption("ui-closely-related-actions-work-well-in-a-connected-button-group");
    gap(Dp(32.0));
    prose(
        "ui-connected-button-groups-should-be-used-for-single-or-multi-select-patterns-that-use-toggle",
    );
    gap(Dp(8.0));
    prose("ui-avoid-using-a-connected-group-when-none-of-the-buttons-can-be-toggled");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                labelled("Wi-Fi"),
                labelled("ui-cellular"),
                labelled("ui-ethernet"),
            ],
            ButtonGroupsStyle::Connected,
        )
        .selected(vec![0, 2])
        .label(I18N.text("ui-networks"));
    }));
    caption("ui-use-the-connected-button-group-with-single-or-multi-select-patterns");
    gap(Dp(48.0));
    subheading("ui-color".into());
    gap(Dp(16.0));
    prose(
        "ui-avoid-mixing-color-styles-in-connected-button-groups-it-can-make-selection-and-emphasis-un",
    );
    guidance(
        "Don't",
        "ui-don-t-mix-color-styles-in-connected-button-groups",
        DONT_COLOR,
    );
}

#[tessera]
fn guidelines_anatomy() {
    subheading("ui-container".into());
    gap(Dp(16.0));
    prose(
        "ui-the-standard-button-group-container-has-padding-between-buttons-so-they-can-animate-width",
    );
    gap(Dp(8.0));
    prose("ui-the-standard-button-group-hugs-the-width-of-the-buttons-inside");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(
            vec![
                labelled("ui-button"),
                labelled("ui-button"),
                labelled("ui-button"),
            ],
            ButtonGroupsStyle::Standard,
        )
        .variant(ButtonVariant::Tonal)
        .label(I18N.text("ui-hugging-group"));
    }));
    caption("ui-button-groups-can-animate-without-affecting-their-surroundings");
    gap(Dp(32.0));
    prose(
        "ui-the-connected-button-group-should-span-the-width-of-the-page-or-surface-it-s-placed-on-inc",
    );
    gap(Dp(8.0));
    prose(
        "ui-in-larger-windows-consider-adding-a-maximum-width-to-the-connected-group-to-avoid-it-growi",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        boxed()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button_groups()
                    .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                        label: Some(I18N.text("ui-full-width-group")),
                        ..Default::default()
                    }))
                    .style(ButtonGroupsStyle::Connected)
                    .size(ButtonGroupsSize::Small)
                    .selection_mode(ButtonGroupsSelectionMode::Single)
                    .child_shared(
                        RenderSlot::new(move || {
                            text(I18N.text("ui-day"));
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            text(I18N.text("ui-week"));
                        }),
                        CallbackWith::new(|_| {}),
                    )
                    .child_shared(
                        RenderSlot::new(move || {
                            text(I18N.text("ui-month"));
                        }),
                        CallbackWith::new(|_| {}),
                    );
            });
    }));
    caption(
        "ui-connected-button-groups-increase-the-widths-of-each-button-inside-and-expand-to-their-cont",
    );
}

#[tessera]
fn adaptive() {
    prose(
        "ui-adaptive-design-allows-an-interface-to-respond-or-change-based-on-context-such-as-the-user",
    );
    gap(Dp(32.0));
    subheading("ui-resizing".into());
    gap(Dp(16.0));
    prose(
        "ui-button-groups-should-move-through-layouts-together-in-a-single-line-they-shouldn-t-wrap-to",
    );
    gap(Dp(16.0));
    prose("ui-button-groups-and-individual-buttons-can-be-set-to-fixed-or-flexible-resizing");
    bullets(vec![
        "ui-fixed-manually-define-the-button-width-narrow-to-wide-size-xs-to-xl-or-padding-at-each-bre".into(),
        "ui-flexible-automatically-increase-or-decrease-the-width-of-buttons-and-the-button-group-butt".into(),
    ]);
    gap(Dp(16.0));
    prose(
        "ui-if-adjusting-button-width-manually-avoid-stretching-icon-buttons-beyond-the-wide-setting",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                for size in [
                    ButtonGroupsSize::ExtraSmall,
                    ButtonGroupsSize::Small,
                    ButtonGroupsSize::Medium,
                ] {
                    group(mixed_items(), ButtonGroupsStyle::Standard)
                        .size(size)
                        .variant(ButtonVariant::Tonal)
                        .selected(vec![1])
                        .label(I18N.format(
                            "ui-value-breakpoint",
                            [("v0", I18N.text(&size_name(size)).into())],
                        ));
                    gap(Dp(16.0));
                }
            });
    }));
    caption(
        "ui-buttons-can-have-width-size-and-padding-manually-adjusted-to-fit-different-breakpoints",
    );
    gap(Dp(32.0));
    prose(
        "ui-in-compact-windows-consider-using-smaller-narrower-buttons-so-all-buttons-in-the-button-gr",
    );
    gap(Dp(8.0));
    prose("ui-flexible-buttons-or-button-groups-will-automatically-adjust-width");
    gap(Dp(32.0));
    prose(
        "ui-when-scaling-to-larger-breakpoints-make-sure-that-the-visual-hierarchy-of-each-button-is-p",
    );
    gap(Dp(8.0));
    prose(
        "ui-for-example-the-primary-action-should-remain-the-largest-widest-or-most-visually-prominent",
    );
    gap(Dp(48.0));
    subheading("ui-presentation".into());
    gap(Dp(16.0));
    prose(
        "ui-buttons-at-the-trailing-edge-of-the-button-group-can-be-customized-to-collapse-into-an-ove",
    );
    gap(Dp(8.0));
    prose("ui-buttons-outside-the-group-aren-t-affected-by-button-group-behavior");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(
                    vec![
                        icon_only(filled::MIC_SVG, "ui-microphone"),
                        icon_only(filled::VIDEOCAM_SVG, "ui-camera"),
                        icon_only(filled::MOOD_SVG, "ui-reactions"),
                        icon_only(filled::PRESENT_TO_ALL_SVG, "ui-present"),
                        icon_only(filled::MORE_VERT_SVG, "ui-more"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label(I18N.text("ui-expanded-call-controls"));
                gap(Dp(16.0));
                group(
                    vec![
                        icon_only(filled::MIC_SVG, "ui-microphone"),
                        icon_only(filled::VIDEOCAM_SVG, "ui-camera"),
                        icon_only(filled::MORE_VERT_SVG, "ui-more"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label(I18N.text("ui-collapsed-call-controls"));
            });
    }));
    caption(
        "ui-buttons-should-become-hidden-in-an-overflow-menu-or-visible-again-depending-on-screen-size",
    );
}

#[tessera]
fn behavior() {
    subheading("ui-pressed".into());
    gap(Dp(16.0));
    prose("ui-when-a-button-is-pressed-it-changes-width-and-shape");
    gap(Dp(8.0));
    prose(
        "ui-in-a-standard-button-group-pressing-a-button-also-affects-the-width-of-adjacent-buttons",
    );
    gap(Dp(8.0));
    prose("ui-in-a-connected-button-group-only-the-shape-of-the-pressed-button-changes");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(
                    vec![
                        labelled("AC"),
                        labelled("( )"),
                        labelled("%"),
                        labelled("÷"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .size(ButtonGroupsSize::Medium)
                .variant(ButtonVariant::Tonal)
                .presentation(vec![
                    InteractionVisualState::Rest,
                    InteractionVisualState::Pressed,
                    InteractionVisualState::Rest,
                    InteractionVisualState::Rest,
                ])
                .label(I18N.text("ui-pressed-standard-group"));
                gap(Dp(16.0));
                group(
                    vec![labelled("7"), labelled("8"), labelled("9")],
                    ButtonGroupsStyle::Connected,
                )
                .size(ButtonGroupsSize::Medium)
                .variant(ButtonVariant::Tonal)
                .presentation(vec![
                    InteractionVisualState::Rest,
                    InteractionVisualState::Pressed,
                    InteractionVisualState::Rest,
                ])
                .label(I18N.text("ui-pressed-connected-group"));
            });
    }));
    caption("ui-pressing-buttons-in-a-standard-group-changes-the-width-of-adjacent-buttons");
    gap(Dp(48.0));
    subheading("ui-selected".into());
    gap(Dp(16.0));
    prose("ui-a-selected-button-should-change-shape-from-round-to-square-or-square-to-round");
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Medium)
                    .selected(vec![1])
                    .label(I18N.text("ui-round-group-with-selection"));
                gap(Dp(16.0));
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Medium)
                    .shape(ButtonShape::Square)
                    .selected(vec![1])
                    .label(I18N.text("ui-square-group-with-selection"));
            });
    }));
    caption("ui-selected-buttons-should-change-shape");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-with-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-and-interact-with-each-button-in-the-group".into(),
        "ui-identify-when-buttons-are-selected".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose("ui-each-button-in-a-group-should-have-a-minimum-48x48dp-target");
    gap(Dp(8.0));
    prose(
        "ui-extra-small-and-small-button-groups-have-larger-inner-padding-to-ensure-accessible-targets",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::ExtraSmall)
                    .variant(ButtonVariant::Tonal)
                    .label(I18N.text("ui-extra-small-button-group"));
                gap(Dp(8.0));
                text(I18N.text("ui-extra-small-button-group")).style(type_style(14.0, 20.0));
                gap(Dp(24.0));
                group(mixed_items(), ButtonGroupsStyle::Standard)
                    .size(ButtonGroupsSize::Small)
                    .variant(ButtonVariant::Tonal)
                    .label(I18N.text("ui-small-button-group"));
                gap(Dp(8.0));
                text(I18N.text("ui-small-button-group")).style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(48.0));
    subheading("ui-initial-focus".into());
    gap(Dp(16.0));
    prose(
        "ui-the-button-group-container-is-not-a-focusable-element-initial-focus-should-land-on-the-fir",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        group(mixed_items(), ButtonGroupsStyle::Standard)
            .size(ButtonGroupsSize::Medium)
            .presentation(vec![InteractionVisualState::Focus])
            .label(I18N.text("ui-focus-order"));
    }));
    caption("ui-initial-focus-should-land-on-the-first-button-not-on-the-container");
    gap(Dp(32.0));
    prose(
        "ui-use-tab-to-navigate-to-a-button-group-the-arrow-keys-to-navigate-between-each-item-in-the",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-tab-initial-focus-lands-on-the-first-button".into(),
        "ui-arrow-keys-navigate-between-items".into(),
        "ui-space-or-enter-selects-a-focused-item".into(),
    ]);
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-navigate-between-buttons".into()],
            vec![
                "ui-arrow-keys".into(),
                "ui-navigate-inside-the-component".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-a-focused-button".into(),
            ],
        ],
    );
}

#[tessera]
fn labeling() {
    prose(
        "ui-the-button-group-container-doesn-t-need-to-be-labeled-label-each-button-according-to-the-b",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                text(I18N.text("ui-dinner-club"))
                    .color(INK)
                    .style(type_style(20.0, 28.0));
                gap(Dp(8.0));
                text(I18N.text("ui-3-messages")).style(type_style(16.0, 24.0));
                gap(Dp(24.0));
                group(
                    vec![
                        icon_only(filled::REPLY_SVG, "ui-reply"),
                        icon_only(filled::ARCHIVE_SVG, "ui-archive"),
                        icon_only(filled::DELETE_SVG, "ui-delete"),
                    ],
                    ButtonGroupsStyle::Standard,
                )
                .variant(ButtonVariant::Tonal)
                .label(I18N.text("ui-message-actions"));
            });
    }));
    caption("ui-label-each-button-within-the-button-group");
}
