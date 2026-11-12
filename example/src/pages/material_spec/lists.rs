use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    divider::horizontal_divider,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    list_item::{ListItemPosition, ListItemSelectionMode, list_item},
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    article_helpers::prose,
    gap, panel, scheme_panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "lists_figures_tests.rs"]
mod figures_tests;

const ITEMS: [&str; 3] = ["ui-notes", "ui-projects", "ui-archive"];

#[derive(Clone, Copy, PartialEq)]
struct DemoState {
    single: State<usize>,
    checked: State<[bool; 3]>,
    primary: State<u32>,
    secondary: State<u32>,
}

#[tessera]
pub(super) fn lists_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-lists",
        "ui-present-a-continuous-vertical-index-of-related-content-and-actions",
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
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-list-baseline",
                content: RenderSlot::new(baseline),
            },
        ],
    ).route_optional(route.clone()).pages(vec![
        ArticlePage {title:"ui-overview",intro:Some(RenderSlot::new(list_reading_examples)),sections:vec![]},
        ArticlePage {title:"ui-guidelines",intro:None,sections:vec![
            SpecSection {title:"ui-usage",content:RenderSlot::new(|| {prose("ui-order-related-items-logically-and-use-consistent-formatting-so-people-can-find-and-act-on");gap(Dp(24.0));list_reading_examples();})},
            SpecSection {title:"ui-anatomy",content:RenderSlot::new(list_guideline_anatomy)},
            SpecSection {title:"ui-adaptive-design",content:RenderSlot::new(list_adaptive)},
            SpecSection {title:"ui-behavior",content:RenderSlot::new(list_behavior)},
        ]},
        ArticlePage {title:"ui-accessibility",intro:None,sections:vec![
            SpecSection {title:"ui-use-cases",content:RenderSlot::new(|| {prose("ui-people-must-be-able-to-reach-list-items-identify-their-purpose-and-activate-or-select-them");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Action,false,true,true,"ui-accessible-actions");}));})},
            SpecSection {title:"ui-indicate-selection-with-more-than-color",content:RenderSlot::new(|| {prose("ui-add-a-checkmark-or-selection-control-so-selected-items-remain-identifiable-without-relying");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Multiple,false,true,true,"ui-accessible-multiple-selection");}));})},
            SpecSection {title:"ui-interaction-style",content:RenderSlot::new(list_interaction)},
            SpecSection {title:"ui-focus",content:RenderSlot::new(list_focus)},
            SpecSection {title:"ui-keyboard-navigation",content:RenderSlot::new(|| {spec_table(vec!["ui-keys".into(),"ui-action".into()],vec![vec!["ui-tab-shift-tab".into(),"ui-navigate-between-enabled-row-actions".into()],vec!["ui-space-or-enter".into(),"ui-activate-or-select-the-focused-action".into()]]);})},
            SpecSection {title:"ui-labeling-elements",content:RenderSlot::new(|| {prose("ui-the-item-s-visible-label-and-supporting-text-describe-its-purpose-give-secondary-actions-t");gap(Dp(24.0));subheading("ui-platform-specific-labels".into());prose("ui-single-and-multiple-selection-expose-distinct-roles-and-checked-states-action-rows-do-not");gap(Dp(24.0));panel(RenderSlot::new(|| {mode_demo(ListItemSelectionMode::Single,false,true,true,"ui-labeled-single-selection");}));})},
        ]},
    ]);
}

#[tessera]
fn mode_demo(
    mode: ListItemSelectionMode,
    multi_action: bool,
    segmented: bool,
    enabled: bool,
    #[prop(into)] label: String,
) {
    let state = DemoState {
        single: remember(|| 0),
        checked: remember(|| [false; 3]),
        primary: remember(|| 0),
        secondary: remember(|| 0),
    };
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let label = label.clone();
            let caption = label.clone();
            column()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    role: Some(if mode == ListItemSelectionMode::Single {
                        Role::RadioGroup
                    } else {
                        Role::List
                    }),
                    label: Some(I18N.text(&(label))),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .children(move || {
                    for index in 0..ITEMS.len() {
                        item(
                            index,
                            mode,
                            multi_action,
                            segmented,
                            enabled,
                            label.clone(),
                            state,
                        );
                        if index < ITEMS.len() - 1 {
                            gap(Dp(2.0));
                        }
                    }
                });
            if mode == ListItemSelectionMode::Action {
                gap(Dp(16.0));
                text(I18N.format(
                    "ui-value-primary-actions-value",
                    [
                        ("v0", I18N.text(&caption).into()),
                        ("v1", (state.primary.get()).into()),
                    ],
                ))
                .style(type_style(14.0, 20.0));
                if multi_action {
                    text(I18N.format(
                        "ui-value-secondary-actions-value",
                        [
                            ("v0", I18N.text(&caption).into()),
                            ("v1", (state.secondary.get()).into()),
                        ],
                    ))
                    .style(type_style(14.0, 20.0));
                }
            }
        });
}

#[tessera]
fn item(
    index: usize,
    mode: ListItemSelectionMode,
    multi_action: bool,
    segmented: bool,
    enabled: bool,
    #[prop(into)] label: String,
    state: DemoState,
) {
    let selected = match mode {
        ListItemSelectionMode::Single => state.single.get() == index,
        ListItemSelectionMode::Multiple => state.checked.get()[index],
        ListItemSelectionMode::Action => false,
    };
    let trailing = if multi_action {
        let label = label.clone();
        Some(RenderSlot::new(move || {
            icon_button()
                .standard()
                .icon(filled::MORE_VERT_SVG)
                .enabled(enabled)
                .accessibility_label(I18N.format(
                    "ui-value-value-more-actions",
                    [
                        ("v0", I18N.text(&label).into()),
                        ("v1", I18N.text(&(ITEMS[index])).into()),
                    ],
                ))
                .on_click(move || state.secondary.set(state.secondary.get() + 1));
        }))
    } else if selected {
        Some(RenderSlot::new(|| {
            icon().painter(filled::CHECK_SVG);
        }))
    } else {
        None
    };
    list_item()
        .headline(I18N.text(ITEMS[index]))
        .supporting_text(I18N.text("ui-folder-and-related-documents"))
        .accessibility_label(I18N.format(
            "ui-value-value-format-1",
            [
                ("v0", I18N.text(&label).into()),
                ("v1", I18N.text(&(ITEMS[index])).into()),
            ],
        ))
        .enabled(enabled)
        .selection_mode(mode)
        .selected_optional((mode != ListItemSelectionMode::Action).then_some(selected))
        .position(if segmented {
            match index {
                0 => ListItemPosition::First,
                2 => ListItemPosition::Last,
                _ => ListItemPosition::Middle,
            }
        } else {
            ListItemPosition::Standalone
        })
        .on_click(move || state.primary.set(state.primary.get() + 1))
        .on_selection_change(move |value| {
            if mode == ListItemSelectionMode::Single {
                state.single.set(index);
            } else {
                state.checked.with_mut(|checked| checked[index] = value);
            }
        })
        .leading(|| {
            icon().painter(filled::FOLDER_SVG);
        })
        .trailing_optional(trailing);
}

#[tessera]
fn variants() {
    text(
        I18N.text("ui-expressive-lists-add-grouped-corners-selection-treatment-and-flexible-slots"),
    )
    .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Multiple,
            false,
            true,
            true,
            "ui-expressive-list",
        );
    }));
}

#[tessera]
fn expansion() {
    let expanded = remember(|| false);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            list_item()
                .headline(I18N.text(if expanded.get() {
                    "ui-hide-related-folders"
                } else {
                    "ui-show-related-folders"
                }))
                .accessibility_label(I18N.text("ui-expand-related-folders"))
                .on_click(move || expanded.set(!expanded.get()))
                .trailing(move || {
                    icon().painter(if expanded.get() {
                        filled::CHECK_SVG
                    } else {
                        filled::ADD_SVG
                    });
                });
            if expanded.get() {
                list_item()
                    .headline(I18N.text("ui-archive"))
                    .supporting_text(I18N.text("ui-completed-projects-and-previous-notes"))
                    .accessibility_label(I18N.text("ui-expanded-archive-folder"))
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    });
            }
        });
}

#[tessera]
fn configurations() {
    subheading("ui-styles".into());
    gap(Dp(24.0));
    for segmented in [false, true] {
        style_configuration(segmented);
    }
    subheading("ui-list-selection".into());
    gap(Dp(24.0));
    for (mode, multi_action, label) in [
        (ListItemSelectionMode::Action, false, "ui-single-action"),
        (ListItemSelectionMode::Action, true, "ui-multi-action"),
        (ListItemSelectionMode::Single, false, "ui-single-select"),
        (ListItemSelectionMode::Multiple, false, "ui-multi-select"),
    ] {
        selection_configuration(mode, multi_action, label.to_owned());
    }
    subheading("ui-expand".into());
    gap(Dp(24.0));
    panel(RenderSlot::new(expansion));
}

#[tessera]
fn style_configuration(segmented: bool) {
    subheading(
        if segmented {
            "ui-segmented"
        } else {
            "ui-standard"
        }
        .to_owned(),
    );
    gap(Dp(16.0));
    panel(RenderSlot::new(move || {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            segmented,
            true,
            if segmented {
                "ui-segmented-list"
            } else {
                "ui-standard-list"
            },
        );
    }));
    gap(Dp(24.0));
}

#[tessera]
fn selection_configuration(mode: ListItemSelectionMode, multi_action: bool, label: String) {
    subheading(label.clone());
    gap(Dp(16.0));
    panel(RenderSlot::new(move || {
        mode_demo(mode, multi_action, true, true, label.clone());
    }));
    gap(Dp(24.0));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["ui-element".into(), "ui-token-value".into()],
        vec![
            vec!["ui-container".into(), "Surface".into()],
            vec!["ui-label".into(), "On surface".into()],
            vec![
                "ui-overline-supporting-trailing-content".into(),
                "On surface variant".into(),
            ],
            vec!["ui-divider".into(), "Outline variant".into()],
            vec!["ui-inner-outer-corners".into(), "4 / 16dp".into()],
            vec!["ui-selected-corners".into(), "16dp".into()],
            vec!["ui-leading-icon".into(), "20dp".into()],
        ],
    );
}

#[tessera]
fn anatomy() {
    list_anatomy_diagram();
    gap(Dp(24.0));
    let secondary = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                list_item()
                    .headline(I18N.text("ui-quarterly-summary"))
                    .overline_text(I18N.text("ui-documents"))
                    .supporting_text(I18N.text("ui-review-the-latest-notes-and-decisions"))
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    })
                    .trailing(move || {
                        icon_button()
                            .standard()
                            .icon(filled::MORE_VERT_SVG)
                            .accessibility_label(I18N.text("ui-summary-more-actions"))
                            .on_click(move || secondary.set(secondary.get() + 1));
                    });
                gap(Dp(16.0));
                text(I18N.format(
                    "ui-summary-actions-value",
                    [("v0", (secondary.get()).into())],
                ));
            });
    }));
    gap(Dp(24.0));
    text(I18N.text(
        "ui-1-container-2-overline-3-label-4-trailing-content-5-supporting-text-6-leading-icon",
    ))
    .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    text(I18N.text("ui-content-occupies-the-largest-slot-selection-rows-expose-one-selection-interaction-action-r"))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn scheme(dark: bool) {
    scheme_panel(
        dark,
        RenderSlot::new(move || {
            mode_demo(
                ListItemSelectionMode::Single,
                false,
                true,
                true,
                if dark {
                    "ui-dark-list"
                } else {
                    "ui-light-list"
                },
            );
        }),
    );
}

#[tessera]
fn colors() {
    scheme(false);
    gap(Dp(24.0));
    scheme(true);
}

#[tessera]
fn states() {
    text(
        I18N.text(
            "ui-hover-focus-with-tab-press-and-hold-or-select-a-row-to-inspect-real-feedback",
        ),
    )
    .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    for enabled in [true, false] {
        state_configuration(enabled);
    }
}

#[tessera]
fn state_configuration(enabled: bool) {
    subheading(if enabled { "ui-enabled" } else { "ui-disabled" }.to_owned());
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        mode_demo(
            ListItemSelectionMode::Single,
            false,
            true,
            enabled,
            if enabled {
                "ui-enabled-list"
            } else {
                "ui-disabled-list"
            },
        );
    }));
    gap(Dp(24.0));
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                list_item()
                    .headline(I18N.text("ui-one-line"))
                    .accessibility_label(I18N.text("ui-one-line-list-measurement"));
                gap(Dp(2.0));
                list_item()
                    .headline(I18N.text("ui-two-lines"))
                    .supporting_text(I18N.text("ui-supporting-text"))
                    .accessibility_label(I18N.text("ui-two-line-list-measurement"));
                gap(Dp(2.0));
                list_item()
                    .headline(I18N.text("ui-three-lines"))
                    .supporting_text(I18N.text("ui-supporting-text-second-supporting-line"))
                    .accessibility_label(I18N.text("ui-three-line-list-measurement"));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec![
                "ui-one-two-three-line-minimum-height".into(),
                "56 / 72 / 88dp".into(),
            ],
            vec!["ui-horizontal-content-padding".into(), "16dp".into()],
            vec!["ui-vertical-content-padding".into(), "10dp".into()],
            vec!["ui-between-segmented-rows".into(), "2dp".into()],
            vec![
                "ui-between-leading-content-trailing-slots".into(),
                "12dp".into(),
            ],
            vec!["ui-leading-icon".into(), "20dp".into()],
            vec![
                "ui-unselected-inner-outer-corners".into(),
                "4 / 16dp".into(),
            ],
            vec!["ui-selected-corner-radius".into(), "16dp".into()],
            vec!["ui-minimum-interactive-target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn baseline() {
    prose(
        "ui-the-baseline-list-keeps-rows-flush-with-their-container-instead-of-using-segmented-shapes",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                list_item()
                    .headline(I18N.text("ui-label-text"))
                    .supporting_text(I18N.text("ui-supporting-text"))
                    .accessibility_label(I18N.text("ui-baseline-two-line-row"))
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    });
                horizontal_divider();
                list_item()
                    .headline(I18N.text("ui-label-text"))
                    .accessibility_label(I18N.text("ui-baseline-one-line-row"))
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    });
                horizontal_divider();
                list_item()
                    .headline(I18N.text("ui-label-text"))
                    .supporting_text(I18N.text("ui-supporting-text-that-wraps-onto-another-line"))
                    .accessibility_label(I18N.text("ui-baseline-three-line-row"))
                    .leading(|| {
                        icon().painter(filled::FOLDER_SVG);
                    });
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-baseline-value".into()],
        vec![
            vec![
                "ui-row-container".into(),
                "ui-surface-without-segmented-corners".into(),
            ],
            vec![
                "ui-between-rows".into(),
                "ui-0dp-with-an-optional-outline-variant-divider".into(),
            ],
            vec![
                "ui-one-two-three-line-height".into(),
                "56 / 72 / 88dp".into(),
            ],
            vec![
                "ui-horizontal-content-padding".into(),
                "ui-16dp-leading-and-trailing".into(),
            ],
        ],
    );
}

#[tessera]
fn list_reading_examples() {
    panel(RenderSlot::new(move || {
        column().modifier(Modifier::new().fill_max_width()).children(move || {
        list_item().headline(I18N.text("ui-label-text")).leading(|| {icon().painter(filled::FOLDER_SVG);});
        gap(Dp(8.0));
        list_item().headline(I18N.text("ui-label-text")).supporting_text(I18N.text("ui-supporting-text")).leading(|| {icon().painter(filled::FOLDER_SVG);});
        gap(Dp(8.0));
        list_item().headline(I18N.text("ui-label-text")).supporting_text(I18N.text("ui-supporting-text-that-wraps-naturally-onto-another-line-when-the-available-width-is-compact"))
            .leading(|| {icon().painter(filled::FOLDER_SVG);});
    });
    }));
}

#[tessera]
fn list_anatomy_diagram() {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(456.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(move || {
                    list_item()
                        .modifier(watch(geometry, "container"))
                        .min_height(Dp(84.0))
                        .leading(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    watch(geometry, "media")
                                        .size(Dp(114.0), Dp(64.0))
                                        .background(scheme.surface_container_highest),
                                )
                                .children(move || {
                                    icon()
                                        .painter(filled::CATEGORY_SVG)
                                        .size(Dp(40.0))
                                        .tint(scheme.outline_variant);
                                });
                        })
                        .content(move || {
                            column().children(move || {
                                text(I18N.text("ui-overline"))
                                    .style(type_style(12.0, 16.0))
                                    .modifier(watch(geometry, "overline"));
                                text(I18N.text("ui-label-text"))
                                    .style(type_style(16.0, 24.0))
                                    .modifier(watch(geometry, "label"));
                            });
                        })
                        .trailing(move || {
                            text("100+")
                                .style(type_style(14.0, 20.0))
                                .modifier(watch(geometry, "trailing-text"));
                        });
                    gap(Dp(16.0));
                    list_item()
                        .headline(I18N.text("ui-label-text"))
                        .min_height(Dp(64.0))
                        .leading(move || {
                            boxed().modifier(watch(geometry, "icon")).children(|| {
                                icon().painter(filled::PERSON_OUTLINE_SVG).size(Dp(20.0));
                            });
                        })
                        .supporting_content(move || {
                            text(I18N.text("ui-supporting-text"))
                                .style(type_style(14.0, 20.0))
                                .modifier(watch(geometry, "support"));
                        });
                    gap(Dp(16.0));
                    list_item()
                        .headline(I18N.text("ui-label-text"))
                        .supporting_text(I18N.text("ui-supporting-text"))
                        .min_height(Dp(64.0))
                        .leading(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    watch(geometry, "avatar")
                                        .size(Dp(40.0), Dp(40.0))
                                        .background_with_shape(
                                            scheme.primary_container,
                                            Shape::Ellipse,
                                        ),
                                )
                                .children(move || {
                                    text("A").color(scheme.on_primary_container);
                                });
                        })
                        .trailing(move || {
                            boxed()
                                .modifier(watch(geometry, "trailing-icon"))
                                .children(|| {
                                    icon().painter(filled::ARROW_RIGHT_SVG).size(Dp(20.0));
                                });
                        });
                    horizontal_divider().modifier(watch(geometry, "divider"));
                });
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    list_callouts(geometry);
                });
        });
}

#[tessera]
fn list_callouts(geometry: State<Geometry>) {
    let values = geometry.get();
    let Some(root) = values.get("figure") else {
        return;
    };
    let Some(container) = relative(&values, "container") else {
        return;
    };
    let mut svg = String::new();
    for (number, name, side) in [
        (1, "container", 0),
        (2, "overline", 0),
        (3, "label", 0),
        (4, "trailing-text", 1),
        (5, "support", 1),
        (6, "trailing-icon", 1),
        (7, "divider", 2),
        (8, "avatar", 2),
        (9, "icon", 3),
        (10, "media", 3),
    ] {
        let Some(target) = relative(&values, name) else {
            continue;
        };
        let (x, y, tx, ty) = match side {
            0 => (
                if number == 1 {
                    container.x + 40.0
                } else {
                    target.x + target.w / 2.0 - 12.0 + if number == 2 { -48.0 } else { 48.0 }
                },
                container.y
                    - 64.0
                    - (if number == 1 {
                        0.0
                    } else if number == 2 {
                        16.0
                    } else {
                        32.0
                    }),
                if number == 1 {
                    container.x + 16.0
                } else {
                    target.x + target.w / 2.0
                },
                target.y + 4.0,
            ),
            1 => (
                container.x + container.w + 40.0,
                target.y + target.h / 2.0 - 16.0,
                target.x + target.w,
                target.y + target.h / 2.0,
            ),
            2 => (
                if number == 7 {
                    target.x + target.w - 80.0
                } else {
                    target.x + target.w / 2.0 - 12.0
                },
                target.y + target.h + 40.0,
                if number == 7 {
                    target.x + target.w - 68.0
                } else {
                    target.x + target.w / 2.0
                },
                target.y + target.h,
            ),
            _ => (
                container.x - 72.0,
                target.y + target.h / 2.0 - 16.0,
                target.x,
                target.y + target.h / 2.0,
            ),
        };
        if side == 0 {
            let elbow = ty - 12.0;
            dotted(
                &mut svg,
                x + 12.0,
                y + 32.0,
                elbow - y - 32.0,
                true,
                "#49454f",
            );
            dotted(&mut svg, x + 12.0, elbow, tx - x - 12.0, false, "#49454f");
            dotted(&mut svg, tx, elbow, ty - elbow, true, "#49454f");
        } else if side == 2 {
            dotted(&mut svg, tx, ty, y - ty, true, "#49454f");
        } else {
            dotted(
                &mut svg,
                if side == 1 { tx } else { x + 24.0 },
                ty,
                if side == 1 { x - tx } else { tx - x - 24.0 },
                false,
                "#49454f",
            );
        }
        if number != 1 && number != 7 {
            annotations::outline(&mut svg, target);
        }
        annotations::marker(number, Dp(x), Dp(y));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn list_guideline_anatomy() {
    prose(
        "ui-the-container-and-label-are-required-supporting-text-leading-media-and-trailing-informatio",
    );
    gap(Dp(24.0));
    list_anatomy_diagram();
    gap(Dp(24.0));
    subheading("ui-container".into());
    prose(
        "ui-the-tallest-contained-element-determines-the-row-s-height-keep-the-content-hierarchy-clear",
    );
    gap(Dp(24.0));
    subheading("ui-label-supporting-text".into());
    prose(
        "ui-keep-the-label-brief-limit-supporting-text-to-a-few-readable-lines-and-reduce-detail-at-co",
    );
    gap(Dp(24.0));
    subheading("ui-icons".into());
    prose(
        "ui-a-leading-icon-identifies-the-subject-a-trailing-icon-can-communicate-status-or-another-ac",
    );
    gap(Dp(24.0));
    subheading("ui-leading-media".into());
    prose(
        "ui-anchor-thumbnails-and-avatars-to-the-leading-edge-for-consistent-scanning-use-circular-ava",
    );
    gap(Dp(24.0));
    subheading("ui-trailing-text".into());
    prose("ui-use-concise-supplemental-details-such-as-a-count-price-or-date-at-the-trailing-edge");
    gap(Dp(24.0));
    subheading("ui-selection-controls".into());
    prose(
        "ui-use-radio-style-semantics-for-one-choice-checkboxes-for-several-choices-and-switches-for-i",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Single,
            false,
            true,
            true,
            "ui-guideline-single-selection",
        );
    }));
    gap(Dp(24.0));
    subheading("ui-gaps-dividers".into());
    prose(
        "ui-use-small-gaps-with-contained-segmented-rows-reserve-dividers-for-uncontained-or-more-comp",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            true,
            true,
            "ui-guideline-grouped-list",
        );
    }));
}

#[tessera]
fn list_adaptive() {
    subheading("ui-line-length".into());
    prose(
        "ui-adjust-margins-or-columns-to-avoid-long-text-lines-preserve-a-comfortable-reading-measure",
    );
    gap(Dp(24.0));
    subheading("ui-adapt-list-elements-layout".into());
    prose(
        "ui-expanded-windows-can-show-additional-context-and-larger-imagery-while-compact-windows-keep",
    );
    gap(Dp(24.0));
    list_reading_examples();
    gap(Dp(24.0));
    subheading("ui-swap-components".into());
    prose(
        "ui-a-card-or-carousel-can-present-the-same-images-text-and-actions-when-more-space-is-availab",
    );
    gap(Dp(24.0));
    subheading("ui-compact-breakpoints".into());
    prose("ui-let-the-list-use-the-available-width-and-open-details-as-another-page");
    gap(Dp(24.0));
    subheading("ui-medium-expanded-breakpoints".into());
    prose("ui-a-list-and-its-selected-detail-can-share-the-same-view");
    gap(Dp(24.0));
    list_detail_example();
}

#[tessera]
fn list_detail_example() {
    let selected = remember(|| 0_usize);
    panel(RenderSlot::new(move || {
        flow_row().item_spacing(Dp(24.0)).line_spacing(Dp(24.0)).children(move || {
        column().modifier(Modifier::new().width(Dp(240.0))).children(move || {for index in 0..3 {detail_row(index,selected);}});
        column().modifier(Modifier::new().width(Dp(240.0)).padding_all(Dp(16.0))).children(move || {text(I18N.text(ITEMS[selected.get()])).style(type_style(24.0,32.0));gap(Dp(16.0));prose("ui-the-selected-item-s-details-remain-next-to-the-list-in-an-expanded-layout");});
    });
    }));
}

#[tessera]
fn detail_row(index: usize, selected: State<usize>) {
    list_item()
        .headline(I18N.text(ITEMS[index]))
        .selection_mode(ListItemSelectionMode::Single)
        .selected(selected.get() == index)
        .on_selection_change(move |value| {
            if value {
                selected.set(index);
            }
        });
}

#[tessera]
fn list_behavior() {
    subheading("ui-list-selection-modes".into());
    prose(
        "ui-single-select-and-multi-select-rows-expose-one-selection-action-for-the-whole-item-action",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Multiple,
            false,
            true,
            true,
            "ui-behavior-multiple-selection",
        );
    }));
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "ui-behavior-multiple-actions",
        );
    }));
    gap(Dp(24.0));
    subheading("ui-list-interactions".into());
    prose(
        "ui-nested-rows-may-expand-and-collapse-if-a-swipe-reveals-actions-provide-a-reachable-seconda",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(expansion));
}

#[tessera]
fn list_interaction() {
    subheading("ui-touch".into());
    prose("ui-a-ripple-confirms-activation-or-selection");
    gap(Dp(24.0));
    subheading("ui-cursor".into());
    prose("ui-hover-feedback-distinguishes-actionable-rows-from-passive-information");
    gap(Dp(24.0));
    subheading("ui-keyboard-switch".into());
    prose(
        "ui-focus-identifies-the-action-reached-by-traversal-space-or-enter-activates-the-focused-item",
    );
    gap(Dp(24.0));
    subheading("ui-swipe".into());
    prose("ui-keep-hidden-swipe-actions-reachable-through-a-visible-secondary-control");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "ui-accessible-multiple-actions",
        );
    }));
}

#[tessera]
fn list_focus() {
    subheading("ui-single-action-lists".into());
    prose("ui-each-row-exposes-one-action-and-an-accessible-name-matching-its-content");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            false,
            true,
            true,
            "ui-focus-single-actions",
        );
    }));
    gap(Dp(24.0));
    subheading("ui-multi-action-lists".into());
    prose(
        "ui-the-primary-action-and-supplementary-controls-must-each-be-reachable-activating-a-secondar",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        mode_demo(
            ListItemSelectionMode::Action,
            true,
            true,
            true,
            "ui-focus-multiple-actions",
        );
    }));
}

#[cfg(all(test, not(target_family = "wasm")))]
mod collection_article_tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::Node,
        headless::{HeadlessConfig, HeadlessRenderer},
    };

    use super::*;

    #[tessera]
    fn fixture(kind: usize) {
        material_theme().child(move || match kind {
            0 => {
                super::super::cards::cards_spec();
            }
            1 => {
                super::super::chips::chips_spec();
            }
            2 => {
                lists_spec();
            }
            _ => {
                super::super::dividers::divider_spec();
            }
        });
    }

    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> Node {
        let snapshot = renderer
            .inspect_accessibility()
            .expect("accessibility tree");
        snapshot
            .find(&crate::res::I18N.text(label), role)
            .unwrap_or_else(|| panic!("missing {label} {role:?}"))
            .1
            .clone()
    }

    fn click(renderer: &mut HeadlessRenderer, node: Node) {
        let target = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, current)| {
                current.role() == node.role()
                    && current.label() == node.label()
                    && current.bounds() == node.bounds()
            })
            .expect("published pointer target")
            .0;
        let mut bounds = node.bounds().unwrap();
        let top = if node.role() == Role::Tab { 0.0 } else { 100.0 };
        for _ in 0..48 {
            let center = (bounds.y0 + bounds.y1) * 0.5;
            let height = f64::from(renderer.physical_size().height);
            if center >= top && center < height - 24.0 {
                break;
            }
            renderer.pointer_scroll(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(
                    renderer.physical_size().width as f32 - 30.0,
                    renderer.physical_size().height as f32 * 0.5,
                ),
                0.0,
                (height * 0.5 - center).clamp(-2000.0, 2000.0) as f32,
            );
            renderer.render_frames(12);
            bounds = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(id, _)| *id == target)
                .expect("retained pointer target after scrolling")
                .1
                .bounds()
                .unwrap();
        }
        let center = (bounds.y0 + bounds.y1) * 0.5;
        assert!(
            center >= top && center < f64::from(renderer.physical_size().height) - 24.0,
            "scrolling must reveal the actual pointer target: {bounds:?}"
        );
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(4);
    }

    fn reveal_section(renderer: &mut HeadlessRenderer, tab: &str, section: &str) {
        let target = node(renderer, tab, Role::Tab);
        click(renderer, target);
        let contents = node(renderer, section, Role::Link);
        click(renderer, contents);
        renderer.render_frames(12);
        let bounds = node(renderer, section, Role::Heading).bounds().unwrap();
        assert!(
            bounds.y0 >= 0.0 && bounds.y0 < f64::from(renderer.physical_size().height),
            "the contents action must reveal {section}: {bounds:?}"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn collection_articles_keep_official_chapters_and_independent_live_actions() {
        for (kind, guidelines, accessibility) in [
            (
                0,
                vec![
                    "Usage",
                    "Anatomy",
                    "Cards in a collection",
                    "Adaptive design",
                    "Behavior",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Focus",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                1,
                vec![
                    "Usage",
                    "Variants",
                    "Anatomy",
                    "Assist chips",
                    "Filter chips",
                    "Input chips",
                    "Suggestion chips",
                ],
                vec![
                    "Use cases",
                    "Interaction & style",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                2,
                vec!["Usage", "Anatomy", "Adaptive design", "Behavior"],
                vec![
                    "Use cases",
                    "Indicate selection with more than color",
                    "Interaction & style",
                    "Focus",
                    "Keyboard navigation",
                    "Labeling elements",
                ],
            ),
            (
                3,
                vec![
                    "Usage",
                    "Anatomy",
                    "Full-width dividers",
                    "Inset dividers",
                    "Vertical divider",
                ],
                vec![],
            ),
        ] {
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || {
                    fixture(kind);
                }),
                vec![Box::new(crate::TestPipelines)],
                HeadlessConfig {
                    width: 900,
                    height: 8000,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            for (tab, sections) in [("Guidelines", guidelines), ("Accessibility", accessibility)] {
                for section in sections {
                    reveal_section(&mut renderer, tab, section);
                }
            }
            if kind == 1 {
                reveal_section(&mut renderer, "Accessibility", "Labeling elements");
                let before = renderer.accessibility_snapshot().unwrap();
                let alex_count = before
                    .nodes
                    .iter()
                    .filter(|(_, node)| {
                        crate::has_label(&node, "Alex") && node.role() == Role::CheckBox
                    })
                    .count();
                assert!(
                    alex_count > 0,
                    "the live recipient examples must be composed"
                );
                let sam_ids = before
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        (crate::has_label(&node, "Sam") && node.role() == Role::CheckBox)
                            .then_some(*id)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(sam_ids.len(), alex_count);
                let remove_ids = before
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        (node.role() == Role::Button && crate::has_label(&node, "Remove Alex"))
                            .then_some(*id)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(remove_ids.len(), alex_count);
                for (index, id) in remove_ids.into_iter().enumerate() {
                    let snapshot = renderer.inspect_accessibility().unwrap();
                    let remove = snapshot
                        .get(id)
                        .expect("retained recipient remove action")
                        .clone();
                    click(&mut renderer, remove);
                    let snapshot = renderer.accessibility_snapshot().unwrap();
                    assert_eq!(
                        snapshot
                            .nodes
                            .iter()
                            .filter(|(_, node)| {
                                crate::has_label(&node, "Alex") && node.role() == Role::CheckBox
                            })
                            .count(),
                        alex_count - index - 1,
                        "a remove action must affect only its own recipient example"
                    );
                    for id in &sam_ids {
                        assert!(snapshot.nodes.iter().any(|(current, node)| {
                            current == id
                                && crate::has_label(&node, "Sam")
                                && node.role() == Role::CheckBox
                        }));
                    }
                }
                let snapshot = renderer.accessibility_snapshot().unwrap();
                assert!(
                    !snapshot
                        .nodes
                        .iter()
                        .any(|(_, node)| crate::has_label(&node, "Alex")
                            && node.role() == Role::CheckBox)
                );
                let restores = snapshot
                    .nodes
                    .iter()
                    .filter_map(|(id, node)| {
                        (crate::has_label(&node, "Restore recipients")
                            && node.role() == Role::Button)
                            .then_some(*id)
                    })
                    .collect::<Vec<_>>();
                assert_eq!(restores.len(), alex_count);
                for (index, id) in restores.into_iter().enumerate() {
                    let restore = renderer
                        .accessibility_snapshot()
                        .unwrap()
                        .nodes
                        .into_iter()
                        .find(|(current, _)| *current == id)
                        .expect("retained restore action")
                        .1;
                    click(&mut renderer, restore);
                    assert_eq!(
                        renderer
                            .accessibility_snapshot()
                            .unwrap()
                            .nodes
                            .iter()
                            .filter(|(_, node)| {
                                crate::has_label(&node, "Alex") && node.role() == Role::CheckBox
                            })
                            .count(),
                        index + 1,
                        "restore must rebuild only its own removed recipient"
                    );
                }
            } else if kind == 2 {
                reveal_section(&mut renderer, "Accessibility", "Interaction & style");
                let secondary = node(
                    &renderer,
                    "Accessible multiple actions: Notes more actions",
                    Role::Button,
                );
                click(&mut renderer, secondary);
                node(
                    &renderer,
                    "Accessible multiple actions secondary actions: 1",
                    Role::Label,
                );
                node(
                    &renderer,
                    "Accessible multiple actions primary actions: 0",
                    Role::Label,
                );
            }
        }
    }
}
