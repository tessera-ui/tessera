//! Split button specifications with anchored option menus.
//!
//! ## Usage
//!
//! Compare a main action with its related menu actions across sizes and styles.

use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    menus::{MenuController, menu_item, menu_provider},
    modifier::ModifierExt as _,
    spacer::spacer,
    split_buttons::{
        SplitButtonDefaults, SplitButtonSize, SplitButtonVariant, split_button_layout,
        split_leading_button, split_trailing_button,
    },
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, LayoutCoordinates, Modifier, RenderSlot, State, modifier::FocusModifierExt as _, remember,
    tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    article_helpers::{bullets, caption, do_dont, legend, prose},
    gap, panel, spec_table, subheading, type_style,
};

const VARIANTS: [SplitButtonVariant; 4] = [
    SplitButtonVariant::Elevated,
    SplitButtonVariant::Filled,
    SplitButtonVariant::Tonal,
    SplitButtonVariant::Outlined,
];

const SIZES: [SplitButtonSize; 5] = [
    SplitButtonSize::ExtraSmall,
    SplitButtonSize::Small,
    SplitButtonSize::Medium,
    SplitButtonSize::Large,
    SplitButtonSize::ExtraLarge,
];

fn variant_name(variant: SplitButtonVariant) -> &'static str {
    match variant {
        SplitButtonVariant::Elevated => "ui-elevated",
        SplitButtonVariant::Filled => "ui-filled",
        SplitButtonVariant::Tonal => "ui-tonal",
        SplitButtonVariant::Outlined => "ui-outlined",
    }
}

fn size_name(size: SplitButtonSize) -> &'static str {
    match size {
        SplitButtonSize::ExtraSmall => "ui-extra-small",
        SplitButtonSize::Small => "ui-small",
        SplitButtonSize::Medium => "ui-medium",
        SplitButtonSize::Large => "ui-large",
        SplitButtonSize::ExtraLarge => "ui-extra-large",
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Menu {
    controller: State<MenuController>,
    anchor: State<LayoutCoordinates>,
    source: State<String>,
    result: State<String>,
    theme: State<MaterialTheme>,
}

#[tessera]
pub(super) fn split_buttons_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    let menu = Menu {
        controller: remember(MenuController::new),
        anchor: remember(LayoutCoordinates::default),
        source: remember(String::new),
        result: remember(|| "ui-no-option-selected".to_owned()),
        theme: remember(MaterialTheme::default),
    };
    boxed()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            let route = route.clone();
            menu_provider()
                .controller(menu.controller)
                .container_color(menu.theme.get().color_scheme.surface_container_low)
                .window_anchor(menu.anchor)
                .main_content(move || {
                    article(
                        "ui-split-button",
                        "ui-split-buttons-pair-a-main-action-with-a-menu-of-related-actions",
                        vec![
                            SpecSection {
                                title: "ui-variants",
                                content: RenderSlot::new(move || {
                                    variants(menu);
                                }),
                            },
                            SpecSection {
                                title: "ui-configurations",
                                content: RenderSlot::new(move || {
                                    configurations(menu);
                                }),
                            },
                            SpecSection {
                                title: "ui-tokens-specs",
                                content: RenderSlot::new(tokens),
                            },
                            SpecSection {
                                title: "ui-anatomy",
                                content: RenderSlot::new(move || {
                                    anatomy(menu);
                                }),
                            },
                            SpecSection {
                                title: "ui-color",
                                content: RenderSlot::new(move || {
                                    colors(menu);
                                }),
                            },
                            SpecSection {
                                title: "ui-states",
                                content: RenderSlot::new(move || {
                                    states(menu);
                                }),
                            },
                            SpecSection {
                                title: "ui-measurements",
                                content: RenderSlot::new(measurements),
                            },
                        ],
                    )
                    .route_optional(route.clone())
                    .pages(vec![
                        ArticlePage {
                            title: "ui-overview",
                            intro: Some(RenderSlot::new(move || {
                                overview(menu);
                            })),
                            sections: vec![],
                        },
                        ArticlePage {
                            title: "ui-guidelines",
                            intro: None,
                            sections: vec![
                                SpecSection {
                                    title: "ui-usage",
                                    content: RenderSlot::new(move || {
                                        usage(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "ui-anatomy",
                                    content: RenderSlot::new(move || {
                                        guide_anatomy(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "ui-color",
                                    content: RenderSlot::new(move || {
                                        guide_color(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "ui-adaptive-design",
                                    content: RenderSlot::new(move || {
                                        adaptive_design(menu);
                                    }),
                                },
                                SpecSection {
                                    title: "ui-behavior",
                                    content: RenderSlot::new(move || {
                                        behavior(menu);
                                    }),
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
                                    title: "ui-initial-focus",
                                    content: RenderSlot::new(initial_focus),
                                },
                                SpecSection {
                                    title: "ui-keyboard-navigation",
                                    content: RenderSlot::new(keyboard),
                                },
                                SpecSection {
                                    title: "ui-labeling-elements",
                                    content: RenderSlot::new(labeling),
                                },
                            ],
                        },
                    ]);
                })
                .menu_content(move |scope| {
                    material_theme()
                        .theme(move || menu.theme.get())
                        .child(move || {
                            menu_item()
                                .menu(scope)
                                .label(I18N.text("ui-schedule-send"))
                                .leading_icon(filled::SCHEDULE_SVG)
                                .on_click(move || {
                                    menu.result.set("ui-schedule-send".into());
                                });
                            menu_item()
                                .menu(scope)
                                .label(I18N.text("ui-save-draft"))
                                .leading_icon(filled::EDIT_SVG)
                                .on_click(move || {
                                    menu.result.set("ui-save-draft".into());
                                });
                        });
                });
        });
}

#[tessera]
fn demo(
    menu: Menu,
    #[prop(into)] label: String,
    variant: SplitButtonVariant,
    size: SplitButtonSize,
    enabled: bool,
) {
    let count = remember(|| 0_u32);
    let bounds = remember(LayoutCoordinates::default);
    let leading_label = label.clone();
    let trailing_label = label.clone();
    let caption_label = label.clone();
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            split_button_layout()
                .leading_button({
                    let label = leading_label.clone();
                    move || {
                        split_leading_button()
                            .size(size)
                            .variant(variant)
                            .enabled(enabled)
                            .accessibility_label(
                                I18N.format(
                                    "ui-value-main-action",
                                    [("v0", I18N.text(&label).into())],
                                ),
                            )
                            .on_click(move || count.set(count.get() + 1))
                            .content(move || {
                                icon().painter(filled::EDIT_SVG);
                                spacer().modifier(
                                    Modifier::new().width(SplitButtonDefaults::icon_spacing(size)),
                                );
                                text(I18N.text("ui-edit"));
                            });
                    }
                })
                .trailing_button({
                    let label = trailing_label.clone();
                    let source = caption_label.clone();
                    move || {
                        let name = label.clone();
                        let selected = source.clone();
                        let theme = use_context::<MaterialTheme>().expect("MaterialTheme");
                        split_trailing_button()
                            .size(size)
                            .variant(variant)
                            .enabled(enabled)
                            .expanded(
                                menu.controller.with(MenuController::is_open)
                                    && menu.source.get() == selected,
                            )
                            .accessibility_label(I18N.format(
                                "ui-value-menu-actions",
                                [("v0", I18N.text(&label).into())],
                            ))
                            .modifier(Modifier::new().on_layout_coordinates(move |coordinates| {
                                bounds.set_if_changed(coordinates);
                                if menu.source.get() == name {
                                    menu.anchor.set_if_changed(coordinates);
                                }
                            }))
                            .on_click(move || {
                                if menu.controller.with(MenuController::is_open)
                                    && menu.source.get() == selected
                                {
                                    menu.controller.with_mut(MenuController::close);
                                } else {
                                    menu.theme.set(theme.get());
                                    menu.source.set(selected.clone());
                                    menu.anchor.set(bounds.get());
                                    menu.controller.with_mut(MenuController::open);
                                }
                            })
                            .content(|| {
                                icon().painter(filled::ARROW_DROP_DOWN_SVG);
                            });
                    }
                });
            gap(Dp(12.0));
            text(I18N.format(
                "ui-main-action-count",
                [
                    ("caption", I18N.text(&caption_label).into()),
                    ("count", count.get().into()),
                ],
            ))
            .style(type_style(14.0, 20.0));
            if menu.source.get() == caption_label {
                text(I18N.format(
                    "ui-menu-choice-value",
                    [("v0", I18N.text(&menu.result.get()).into())],
                ))
                .style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn overview(menu: Menu) {
    bullets(vec![
        "ui-use-to-show-an-action-with-a-menu-of-related-actions".into(),
        "ui-same-size-range-as-buttons-and-icon-buttons-xs-s-m-l-xl".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-overview",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-split-buttons-are-made-of-a-common-button-and-a-menu-icon-button");
}

#[tessera]
fn variants(menu: Menu) {
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-variant",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-a-split-button-pairs-a-leading-action-with-a-trailing-menu-segment");
}

#[tessera]
fn configurations(menu: Menu) {
    subheading("ui-color".into());
    prose("ui-split-buttons-come-in-four-color-configurations");
    gap(Dp(24.0));
    column().children(move || {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(index + 1, variant_name(variant));
            gap(Dp(16.0));
            panel(RenderSlot::new(move || {
                demo(
                    menu,
                    variant_name(variant),
                    variant,
                    SplitButtonSize::Small,
                    true,
                );
            }));
            gap(Dp(24.0));
        }
    });
    gap(Dp(16.0));
    subheading("ui-size".into());
    prose(
        "ui-split-buttons-use-the-same-five-sizes-as-buttons-and-icon-buttons-small-is-the-default",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                demo(
                    menu,
                    "XS",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::ExtraSmall,
                    true,
                );
                demo(
                    menu,
                    "S",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Small,
                    true,
                );
                demo(
                    menu,
                    "M",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Medium,
                    true,
                );
                demo(
                    menu,
                    "L",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::Large,
                    true,
                );
                demo(
                    menu,
                    "XL",
                    SplitButtonVariant::Tonal,
                    SplitButtonSize::ExtraLarge,
                    true,
                );
            });
    }));
}

#[tessera]
fn tokens() {
    prose(
        "ui-split-button-token-sets-are-organized-by-size-each-size-defines-the-container-height-the-i",
    );
    gap(Dp(24.0));
    column().children(|| {
        for size in SIZES {
            size_token_table(size);
        }
    });
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-variant".into(),
            "ui-container".into(),
            "ui-label-and-icon".into(),
        ],
        vec![
            vec!["ui-filled".into(), "Primary".into(), "On primary".into()],
            vec![
                "ui-tonal".into(),
                "Secondary container".into(),
                "On secondary container".into(),
            ],
            vec![
                "ui-elevated".into(),
                "Surface container low".into(),
                "Primary".into(),
            ],
            vec![
                "ui-outlined".into(),
                "ui-transparent".into(),
                "On surface variant".into(),
            ],
        ],
    );
}

#[tessera]
fn size_token_table(size: SplitButtonSize) {
    let leading = SplitButtonDefaults::leading_content_padding(size);
    let trailing = SplitButtonDefaults::trailing_content_padding(size);
    subheading(size_name(size).to_owned());
    gap(Dp(12.0));
    spec_table(
        vec!["ui-size-attribute".into(), "ui-value".into()],
        vec![
            vec![
                "ui-container-height".into(),
                format!("{}dp", SplitButtonDefaults::container_height(size).0),
            ],
            vec![
                "ui-inner-corner".into(),
                format!("{}dp", SplitButtonDefaults::inner_corner_size(size).0),
            ],
            vec!["ui-outer-corner".into(), "ui-50-full-capsule".into()],
            vec![
                "ui-leading-padding".into(),
                format!("{}/{}dp", leading.left.0, leading.right.0),
            ],
            vec![
                "ui-trailing-padding".into(),
                format!("{}/{}dp", trailing.left.0, trailing.right.0),
            ],
            vec![
                "ui-trailing-icon".into(),
                format!("{}dp", SplitButtonDefaults::trailing_icon_size(size).0),
            ],
        ],
    );
    gap(Dp(16.0));
}

#[tessera]
fn anatomy(menu: Menu) {
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-anatomy",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-leading-button");
        legend(2, "ui-icon");
        legend(3, "ui-label-text");
        legend(4, "ui-trailing-button");
    });
    gap(Dp(16.0));
    caption(
        "ui-the-leading-button-holds-the-main-action-the-trailing-button-opens-its-related-actions",
    );
}

#[tessera]
fn colors(menu: Menu) {
    prose(
        "ui-split-buttons-use-the-same-color-schemes-as-standard-buttons-unlike-toggle-buttons-the-spl",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                for variant in VARIANTS {
                    demo(
                        menu,
                        I18N.format(
                            "ui-light-value",
                            [("v0", I18N.text(&variant_name(variant)).into())],
                        ),
                        variant,
                        SplitButtonSize::Small,
                        true,
                    );
                }
            });
    }));
    gap(Dp(24.0));
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(move || {
            panel(RenderSlot::new(move || {
                flow_row()
                    .item_spacing(Dp(16.0))
                    .line_spacing(Dp(16.0))
                    .children(move || {
                        for variant in VARIANTS {
                            demo(
                                menu,
                                I18N.format(
                                    "ui-dark-value",
                                    [("v0", I18N.text(&variant_name(variant)).into())],
                                ),
                                variant,
                                SplitButtonSize::Small,
                                true,
                            );
                        }
                    });
            }));
        });
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-variant".into(),
            "ui-container-color".into(),
            "ui-label-color".into(),
        ],
        vec![
            vec!["ui-filled".into(), "Primary".into(), "On primary".into()],
            vec![
                "ui-tonal".into(),
                "Secondary container".into(),
                "On secondary container".into(),
            ],
            vec![
                "ui-elevated".into(),
                "Surface container low".into(),
                "Primary".into(),
            ],
            vec![
                "ui-outlined".into(),
                "ui-transparent".into(),
                "On surface variant".into(),
            ],
        ],
    );
}

#[tessera]
fn states(menu: Menu) {
    let enabled = remember(|| true);
    prose(
        "ui-states-are-visual-representations-used-to-communicate-the-status-of-a-component-or-interac",
    );
    gap(Dp(16.0));
    prose(
        "ui-split-button-states-use-the-same-colors-and-state-layers-as-buttons-and-icon-buttons-the-i",
    );
    gap(Dp(24.0));
    column().children(move || {
        button()
            .tonal()
            .selected(!enabled.get())
            .accessibility_label(I18N.text("ui-disable-split-button"))
            .on_click(move || {
                enabled.set(!enabled.get());
                if !enabled.get() {
                    menu.controller.with_mut(MenuController::close);
                }
            })
            .child(move || {
                text(I18N.text("ui-disabled"));
            });
        gap(Dp(24.0));
        panel(RenderSlot::new(move || {
            demo(
                menu,
                "ui-states",
                SplitButtonVariant::Filled,
                SplitButtonSize::Medium,
                enabled.get(),
            );
        }));
    });
    gap(Dp(16.0));
    caption("ui-enabled-hovered-focused-pressed-and-disabled-states-keep-the-same-color-roles");
}

#[tessera]
fn measurements() {
    prose(
        "ui-text-and-icons-are-optically-centered-when-the-segments-are-asymmetrical-they-re-centered",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-value".into(),
        ],
        vec![
            vec![
                "ui-split-button".into(),
                "ui-container-height-default-s".into(),
                "40dp".into(),
            ],
            vec![
                "ui-each-segment".into(),
                "ui-minimum-target".into(),
                "48 × 48dp".into(),
            ],
            vec![
                "ui-container".into(),
                "ui-outer-corner".into(),
                "ui-50-full-capsule".into(),
            ],
            vec![
                "ui-container".into(),
                "ui-inner-corner".into(),
                "ui-4dp-12dp-active".into(),
            ],
            vec![
                "ui-outlined".into(),
                "ui-divider-outline".into(),
                "1dp".into(),
            ],
            vec![
                "ui-segments".into(),
                "ui-space-between".into(),
                "2dp".into(),
            ],
            vec![
                "ui-trailing-segment".into(),
                "ui-minimum-width".into(),
                "48dp".into(),
            ],
            vec![
                "ui-trailing-segment".into(),
                "ui-icon-size".into(),
                "22dp".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        measurements_figure();
    }));
    gap(Dp(16.0));
    caption("ui-the-trailing-segment-keeps-a-48dp-target-even-when-the-container-is-40dp-tall");
}

#[tessera]
fn measurements_figure() {
    let controller = remember(MenuController::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            split_button_layout()
                .leading_button(move || {
                    split_leading_button()
                        .size(SplitButtonSize::Medium)
                        .variant(SplitButtonVariant::Filled)
                        .accessibility_label(I18N.text("ui-publish"))
                        .on_click(|| {})
                        .content(move || {
                            icon().painter(filled::PUBLISH_SVG);
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            text(I18N.text("ui-publish"));
                        });
                })
                .trailing_button(move || {
                    split_trailing_button()
                        .size(SplitButtonSize::Medium)
                        .variant(SplitButtonVariant::Filled)
                        .expanded(controller.with(MenuController::is_open))
                        .accessibility_label(I18N.text("ui-publish-options"))
                        .on_click(move || controller.with_mut(MenuController::toggle))
                        .content(|| {
                            icon().painter(filled::ARROW_DROP_DOWN_SVG);
                        });
                });
            gap(Dp(12.0));
            caption("ui-medium-split-button-56dp-container-24-24dp-inner-padding-26dp-menu-icon");
        });
}

#[tessera]
fn usage(menu: Menu) {
    prose(
        "ui-split-buttons-are-used-to-add-a-menu-of-actions-alongside-a-main-action-this-reduces-visua",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-usage",
            SplitButtonVariant::Filled,
            SplitButtonSize::Small,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-split-buttons-on-their-own-can-grab-attention");
    gap(Dp(24.0));
    prose(
        "ui-split-buttons-have-five-recommended-sizes-scale-up-the-split-button-in-large-breakpoints-o",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-open-a-menu-from-a-split-button",
        "ui-avoid-modifying-the-menu-in-unusual-ways",
    );
    gap(Dp(32.0));
    prose(
        "ui-the-main-action-stays-on-the-leading-segment-the-trailing-segment-only-opens-the-menu-of-r",
    );
    gap(Dp(16.0));
    do_dont(
        "ui-keep-the-leading-segment-for-the-primary-action",
        "ui-don-t-put-a-menu-on-the-leading-segment-and-an-action-on-the-trailing-segment",
    );
    gap(Dp(32.0));
    prose(
        "ui-split-buttons-can-be-used-alongside-other-buttons-and-button-groups-and-can-be-a-different",
    );
}

#[tessera]
fn guide_anatomy(menu: Menu) {
    prose(
        "ui-the-split-button-has-a-leading-button-and-a-trailing-button-separated-by-a-small-gap",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-anatomy",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-leading-button");
        legend(2, "ui-icon");
        legend(3, "ui-label-text");
        legend(4, "ui-trailing-button");
    });
    gap(Dp(16.0));
    caption("ui-the-trailing-button-contains-the-menu-icon-the-leading-button-carries-the-label");
}

#[tessera]
fn guide_color(menu: Menu) {
    prose(
        "ui-use-the-split-button-color-that-matches-the-page-emphasis-filled-for-the-highest-emphasis",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(16.0))
            .line_spacing(Dp(16.0))
            .children(move || {
                for variant in VARIANTS {
                    demo(
                        menu,
                        variant_name(variant),
                        variant,
                        SplitButtonSize::Small,
                        true,
                    );
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-filled-tonal-elevated-and-outlined-split-buttons-share-the-button-color-roles");
}

#[tessera]
fn adaptive_design(menu: Menu) {
    prose(
        "ui-split-buttons-scale-with-the-layout-keep-the-leading-and-trailing-segments-together-and-le",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            demo(
                menu,
                "ui-small",
                SplitButtonVariant::Tonal,
                SplitButtonSize::Small,
                true,
            );
            gap(Dp(16.0));
            demo(
                menu,
                "ui-large",
                SplitButtonVariant::Tonal,
                SplitButtonSize::Large,
                true,
            );
        });
    }));
    gap(Dp(16.0));
    caption("ui-small-and-large-split-buttons-keep-the-same-leading-trailing-structure");
    gap(Dp(24.0));
    prose(
        "ui-each-segment-keeps-a-minimum-48-48dp-target-so-a-32dp-or-40dp-container-still-exposes-a-fu",
    );
}

#[tessera]
fn behavior(menu: Menu) {
    prose(
        "ui-activating-the-leading-segment-runs-the-main-action-immediately-and-leaves-the-menu-untouc",
    );
    gap(Dp(16.0));
    prose(
        "ui-choosing-a-menu-item-closes-the-menu-and-reports-the-choice-here-pressing-escape-activatin",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        demo(
            menu,
            "ui-behavior",
            SplitButtonVariant::Filled,
            SplitButtonSize::Medium,
            true,
        );
    }));
    gap(Dp(16.0));
    caption("ui-run-the-main-action-on-the-left-then-open-the-menu-to-choose-a-related-action");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-each-button-and-interact-with-them".into(),
        "ui-navigate-to-any-element-opened-by-the-trailing-button".into(),
        "ui-understand-the-current-selection-state-of-the-button".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-each-button-in-the-split-button-needs-a-minimum-target-area-of-48-48dp-extra-small-and-sma",
    );
    gap(Dp(16.0));
    caption("ui-target-areas-should-be-at-least-48-48dp");
    gap(Dp(24.0));
    column().children(|| {
        legend(1, "ui-extra-small");
        legend(2, "ui-small");
    });
}

#[tessera]
fn initial_focus() {
    prose(
        "ui-focus-should-land-on-the-leading-button-then-move-to-the-trailing-button-this-can-depend-o",
    );
    gap(Dp(16.0));
    prose(
        "ui-in-right-to-left-layouts-the-order-is-mirrored-so-focus-starts-on-the-leading-segment-at-t",
    );
    gap(Dp(24.0));
    caption("ui-left-to-right-leading-then-trailing-right-to-left-mirrored");
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-navigate-between-the-two-buttons".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-button".into(),
            ],
            vec![
                "ui-tab-in-the-menu".into(),
                "ui-move-between-the-open-menu-items".into(),
            ],
            vec!["ui-escape".into(), "ui-close-the-menu".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-native-sample-follows-this-table-tab-reaches-the-leading-segment-and-then-the-trailing",
    );
}

#[tessera]
fn labeling() {
    prose(
        "ui-the-accessibility-label-for-the-leading-button-is-the-same-as-buttons-it-should-describe-t",
    );
    gap(Dp(16.0));
    prose(
        "ui-the-trailing-icon-button-should-have-an-extra-state-or-similar-label-indicating-that-the-m",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-label".into(),
            "ui-role".into(),
            "ui-state".into(),
        ],
        vec![
            vec![
                "ui-leading-button".into(),
                "ui-match-the-main-action".into(),
                "ui-button".into(),
                "—".into(),
            ],
            vec![
                "ui-trailing-button".into(),
                "ui-more-main-action-options".into(),
                "ui-button".into(),
                "ui-expanded-or-collapsed".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    caption("ui-leading-buttons-should-have-the-same-labels-as-common-buttons");
}
