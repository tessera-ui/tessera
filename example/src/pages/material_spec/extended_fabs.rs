use tessera_components::{
    boxed::boxed,
    button::button,
    column::column,
    extended_floating_action_button::{
        ExtendedFloatingActionButtonSize, extended_floating_action_button,
    },
    flow_row::flow_row,
    material_icons::filled,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, RenderSlot, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    INK,
    article::{ArticlePage, SpecSection, article},
    floating_actions::{TONES, tone_colors},
    gap, panel, scheme_panel, spec_table, subheading, type_style,
};

const SIZES: [ExtendedFloatingActionButtonSize; 3] = [
    ExtendedFloatingActionButtonSize::Small,
    ExtendedFloatingActionButtonSize::Medium,
    ExtendedFloatingActionButtonSize::Large,
];

fn size_name(size: ExtendedFloatingActionButtonSize) -> &'static str {
    match size {
        ExtendedFloatingActionButtonSize::Small => "ui-small",
        ExtendedFloatingActionButtonSize::Medium => "ui-medium",
        ExtendedFloatingActionButtonSize::Large => "ui-large",
    }
}

#[tessera]
pub(super) fn extended_fabs_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-extended-fabs",
        "ui-extended-floating-action-buttons-extended-fabs-help-people-take-primary-actions",
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
                title: "ui-baseline-extended-fab",
                content: RenderSlot::new(baseline),
            },
        ],
    )
    .route_optional(route.clone())
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
                    title: "ui-color",
                    content: RenderSlot::new(guidelines_color),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-responsive-layout",
                    content: RenderSlot::new(responsive_layout),
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
                    title: "ui-initial-focus",
                    content: RenderSlot::new(initial_focus),
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
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                row()
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
                        text(I18N.text(&(item))).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// Renders one extended FAB with optional configuration overrides.
#[tessera]
fn fab(
    #[prop(into)] label: String,
    size: Option<ExtendedFloatingActionButtonSize>,
    tone: Option<usize>,
    enabled: Option<bool>,
    counted: Option<bool>,
    with_icon: Option<bool>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (container, content) = tone_colors(&scheme, tone.unwrap_or(0));
    let activations = remember(|| 0_u32);
    column().children(move || {
        extended_floating_action_button(I18N.text(label.clone()))
            .size(size.unwrap_or(ExtendedFloatingActionButtonSize::Small))
            .icon_optional(if with_icon.unwrap_or(true) {
                Some(filled::EDIT_SVG.into())
            } else {
                None
            })
            .enabled(enabled.unwrap_or(true))
            .container_color(container)
            .content_color(content)
            .on_click(move || activations.set(activations.get() + 1));
        if counted.unwrap_or(false) {
            gap(Dp(16.0));
            text(I18N.format("ui-activations-value", [("v0", (activations.get()).into())]))
                .style(type_style(14.0, 20.0));
        }
    });
}

#[tessera]
fn overview() {
    prose(
        "ui-extended-fabs-help-people-take-a-primary-action-use-them-for-the-most-common-or-important",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-use-for-the-most-common-or-important-action-on-a-screen".into(),
        "ui-three-variants-small-medium-and-large".into(),
        "ui-use-instead-of-a-fab-when-label-text-is-needed-to-understand-the-action".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    prose("ui-1-small-extended-fab-2-medium-extended-fab-3-large-extended-fab");
}

#[tessera]
fn variants() {
    prose(
        "ui-three-sizes-share-the-same-anatomy-and-differ-in-height-icon-size-padding-and-label-type-s",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(32.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-1-small-extended-fab-2-medium-extended-fab-3-large-extended-fab-2899");
}

#[tessera]
fn configurations() {
    prose(
        "ui-choose-a-size-a-container-tone-whether-the-leading-icon-is-present-and-whether-the-label-i",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                subheading("ui-1-size".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(40.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for size in SIZES {
                            fab(size_name(size)).size(size);
                        }
                    });
                gap(Dp(40.0));
                subheading("ui-2-color".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        for (tone, name) in TONES.into_iter().enumerate() {
                            fab(name).tone(tone);
                        }
                    });
                gap(Dp(40.0));
                subheading("ui-3-icon-position".into());
                gap(Dp(24.0));
                text(I18N.text("ui-with-leading-icon")).style(type_style(14.0, 20.0));
                gap(Dp(16.0));
                fab("ui-compose");
                gap(Dp(24.0));
                text(I18N.text("ui-without-icon")).style(type_style(14.0, 20.0));
                gap(Dp(16.0));
                fab("ui-compose").with_icon(false);
            });
    }));
    gap(Dp(32.0));
    subheading("ui-4-label-expansion".into());
    gap(Dp(8.0));
    prose(
        "ui-an-extended-fab-can-collapse-to-an-icon-only-container-and-expand-again-when-its-label-is",
    );
    gap(Dp(24.0));
    collapsible("ui-compose");
}

#[tessera]
fn collapsible(#[prop(into)] label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let expanded = remember(|| true);
    let activations = remember(|| 0_u32);
    panel(RenderSlot::new(move || {
        let label = label.clone();
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                button()
                    .tonal()
                    .selected(!expanded.get())
                    .accessibility_label(I18N.text("ui-collapse-or-expand-the-extended-fab-label"))
                    .on_click(move || expanded.set(!expanded.get()))
                    .child(move || {
                        text(I18N.text(if expanded.get() {
                            "ui-collapse-label"
                        } else {
                            "ui-expand-label"
                        }));
                    });
                gap(Dp(32.0));
                extended_floating_action_button(I18N.text(label.clone()))
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .expanded(expanded.get())
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .on_click(move || activations.set(activations.get() + 1));
                gap(Dp(16.0));
                text(
                    I18N.format(
                        "ui-value-activations-value-format-161",
                        [
                            (
                                "v0",
                                I18N.text(
                                    &(if expanded.get() {
                                        "ui-expanded"
                                    } else {
                                        "ui-collapsed"
                                    }),
                                )
                                .into(),
                            ),
                            ("v1", (activations.get()).into()),
                        ],
                    ),
                )
                .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn tokens() {
    prose(
        "ui-extended-fab-tokens-are-organized-by-size-and-color-values-below-follow-the-native-compone",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "md.comp.extended-fab.container.color".into(),
                "Primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.icon.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.label-text.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.extended-fab.container.height".into(),
                "ui-56dp-small-80dp-medium-96dp-large".into(),
            ],
            vec![
                "md.comp.extended-fab.container.shape".into(),
                "ui-full-capsule".into(),
            ],
            vec![
                "md.comp.extended-fab.label-text".into(),
                "Title medium · title large · headline small".into(),
            ],
            vec![
                "md.comp.extended-fab.icon.size".into(),
                "24dp · 28dp · 36dp".into(),
            ],
            vec![
                "md.comp.extended-fab.leading-space".into(),
                "16dp · 26dp · 28dp".into(),
            ],
            vec![
                "md.comp.extended-fab.trailing-space".into(),
                "16dp · 20dp · 28dp".into(),
            ],
            vec![
                "md.comp.extended-fab.icon-label-space".into(),
                "8dp · 12dp · 16dp".into(),
            ],
            vec![
                "md.comp.extended-fab.container.elevation".into(),
                "ui-level-3-level-4-when-hovered".into(),
            ],
            vec![
                "md.comp.extended-fab.state-layer".into(),
                "ui-content-color-at-8-hover-and-10-focus-or-press".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-when-a-non-default-color-mapping-is-used-keep-the-state-layer-color-equal-to-the-icon-colo",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "ui-an-extended-fab-combines-a-container-an-optional-leading-icon-and-a-label-the-icon-and-lab",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            fab("ui-compose");
            gap(Dp(32.0));
            for line in ["ui-1-container", "ui-2-icon-optional", "ui-3-label-text"] {
                text(I18N.text(line))
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption(
        "ui-the-label-states-the-action-the-icon-supports-recognition-without-repeating-the-label",
    );
}

#[tessera]
fn colors() {
    prose(
        "ui-container-tones-map-to-the-primary-secondary-and-tertiary-color-roles-either-as-containers",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        scheme_panel(
            dark,
            RenderSlot::new(move || {
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(move || {
                        for (tone, name) in TONES.into_iter().enumerate() {
                            fab(name).tone(tone);
                        }
                    });
                gap(Dp(16.0));
                text("1  Primary container    2  Secondary container    3  Tertiary container    4  Primary    5  Secondary    6  Tertiary")
                    .style(type_style(14.0, 20.0));
            }),
        );
        gap(Dp(32.0));
    }
}

#[tessera]
fn states() {
    prose(
        "ui-states-communicate-elevation-and-interaction-an-enabled-extended-fab-rests-at-level-3-rise",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(|| {
                        fab("ui-enabled");
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("ui-disabled").enabled(false);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        fab("ui-elevated").tone(3);
                    });
                gap(Dp(32.0));
                text(I18N.text("ui-hover-focus-with-tab-or-press-the-actions-above-to-see-the-state-layer-and-elevation-respo"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-state".into(),
            "ui-elevation".into(),
            "ui-state-layer".into(),
        ],
        vec![
            vec!["ui-enabled".into(), "ui-level-3".into(), "--".into()],
            vec!["ui-hovered".into(), "ui-level-4".into(), "8%".into()],
            vec!["ui-focused".into(), "ui-level-3".into(), "10%".into()],
            vec!["ui-pressed".into(), "ui-level-3".into(), "10%".into()],
            vec![
                "ui-disabled".into(),
                "ui-none".into(),
                "ui-faded-container".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-expanded-and-collapsed-states-use-the-same-elevation-only-the-width-and-label-opacity",
    );
}

#[tessera]
fn measurements() {
    prose(
        "ui-height-icon-size-and-padding-scale-with-the-size-variant-reserve-16dp-margins-around-a-flo",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-attribute".into(),
            "ui-small".into(),
            "ui-medium".into(),
            "ui-large".into(),
        ],
        vec![
            vec![
                "ui-container-height".into(),
                "56dp".into(),
                "80dp".into(),
                "96dp".into(),
            ],
            vec![
                "ui-icon-size".into(),
                "24dp".into(),
                "28dp".into(),
                "36dp".into(),
            ],
            vec![
                "ui-leading-space".into(),
                "16dp".into(),
                "26dp".into(),
                "28dp".into(),
            ],
            vec![
                "ui-icon-to-label".into(),
                "8dp".into(),
                "12dp".into(),
                "16dp".into(),
            ],
            vec![
                "ui-trailing-space".into(),
                "16dp".into(),
                "20dp".into(),
                "28dp".into(),
            ],
            vec![
                "ui-label-type".into(),
                "Title medium".into(),
                "Title large".into(),
                "Headline small".into(),
            ],
            vec![
                "ui-corners".into(),
                "ui-full".into(),
                "ui-full".into(),
                "ui-full".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(40.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for size in SIZES {
                    fab(size_name(size)).size(size);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-size-and-padding-measurements-of-the-small-medium-and-large-extended-fabs");
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-outer-margin".into(), "16dp".into()],
            vec![
                "ui-collapsed-width".into(),
                "ui-matches-the-container-height".into(),
            ],
            vec![
                "ui-expanded-width".into(),
                "ui-follows-the-label-minimum-80dp".into(),
            ],
            vec!["ui-minimum-interactive-target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn baseline() {
    prose(
        "ui-the-baseline-extended-fab-is-no-longer-recommended-in-the-m3-expressive-update-use-a-small",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-label");
    }));
    gap(Dp(16.0));
    caption("ui-1-extended-fab-baseline-is-superseded-by-the-small-extended-fab-shown-here");
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-attribute".into(),
            "ui-baseline".into(),
            "ui-small-recommended".into(),
        ],
        vec![
            vec![
                "ui-label-type".into(),
                "Label large".into(),
                "Title medium".into(),
            ],
            vec![
                "ui-inner-padding".into(),
                "ui-wider".into(),
                "ui-reduced".into(),
            ],
            vec![
                "ui-status".into(),
                "ui-not-recommended".into(),
                "ui-recommended".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-use-an-extended-fab-for-the-most-common-or-important-action-on-a-screen-and-when-the-label",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                fab("ui-compose");
                gap(Dp(16.0));
                text(I18N.text("ui-do-keep-the-label-short-and-specific-to-the-action"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("ui-create-a-new-message");
                gap(Dp(16.0));
                text(I18N.text(
                    "ui-don-t-repeat-the-whole-task-description-the-floating-action-stays-concise",
                ))
                .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("ui-label-the-action-not-the-object-compose-not-new-message-composer-screen");
    gap(Dp(32.0));
    bullets(vec![
        "ui-keep-one-extended-fab-per-screen-and-reserve-it-for-the-primary-action".into(),
        "ui-avoid-covering-content-that-people-need-while-the-action-is-visible".into(),
        "ui-prefer-a-fab-without-a-label-when-the-icon-alone-is-universally-understood".into(),
    ]);
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "ui-an-extended-fab-is-a-single-container-with-a-leading-icon-and-a-label-keep-the-icon-option",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn guidelines_color() {
    prose(
        "ui-use-the-default-primary-container-tone-for-most-screens-secondary-and-tertiary-tones-separ",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(|| {
                for tone in [0_usize, 1, 2, 3] {
                    fab(TONES[tone]).tone(tone);
                }
            });
    }));
    gap(Dp(16.0));
    caption("ui-keep-one-tone-across-a-screen-so-the-floating-action-stays-identifiable");
}

#[tessera]
fn placement() {
    prose(
        "ui-place-an-extended-fab-at-the-bottom-end-of-the-screen-above-the-bottom-navigation-bar-or-a",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(240.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    text(I18N.text("ui-content"))
                        .color(scheme.on_surface_variant)
                        .style(type_style(14.0, 20.0));
                    spacer().modifier(Modifier::new().weight(1.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            spacer().modifier(Modifier::new().weight(1.0));
                            extended_floating_action_button(I18N.text("ui-compose"))
                                .size(ExtendedFloatingActionButtonSize::Small)
                                .icon(filled::EDIT_SVG)
                                .container_color(scheme.primary_container)
                                .content_color(scheme.on_primary_container)
                                .on_click(|| {});
                        });
                });
        });
    gap(Dp(16.0));
    caption("ui-extended-fabs-should-have-margins-of-16dp");
}

#[tessera]
fn responsive_layout() {
    prose(
        "ui-choose-the-size-by-window-class-the-small-extended-fab-suits-compact-and-medium-layouts-wh",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("ui-compact".into());
                gap(Dp(16.0));
                fab("ui-compose");
                gap(Dp(32.0));
                subheading("ui-expanded".into());
                gap(Dp(16.0));
                fab("ui-compose").size(ExtendedFloatingActionButtonSize::Large);
            });
    }));
    gap(Dp(16.0));
    caption(
        "ui-the-same-label-at-two-size-presets-the-large-size-carries-more-weight-in-wide-layouts",
    );
}

#[tessera]
fn behavior() {
    prose(
        "ui-an-extended-fab-can-collapse-to-an-icon-only-container-while-content-scrolls-then-expand-a",
    );
    gap(Dp(24.0));
    collapsible("ui-compose");
    gap(Dp(24.0));
    prose(
        "ui-when-the-label-collapses-keep-the-accessible-name-unchanged-so-assistive-technology-still",
    );
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-and-activate-the-extended-fab".into(),
        "ui-identify-the-action-from-its-label-and-icon".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-compose").counted(true);
    }));
    gap(Dp(16.0));
    caption(
        "ui-the-extended-fab-exposes-one-action-named-by-its-label-activating-it-increments-the-counte",
    );
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-extended-fabs-are-large-targets-by-construction-the-smallest-container-is-56dp-high-and-th",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                extended_floating_action_button(I18N.text("ui-compose"))
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .container_color(scheme.primary_container)
                    .content_color(scheme.on_primary_container)
                    .accessibility_label(I18N.text("ui-above-three-to-one-compose"))
                    .on_click(|| {});
                spacer().modifier(Modifier::new().width(Dp(32.0)));
                extended_floating_action_button(I18N.text("ui-compose"))
                    .size(ExtendedFloatingActionButtonSize::Small)
                    .icon(filled::EDIT_SVG)
                    .container_color(scheme.surface_container)
                    .content_color(scheme.outline)
                    .accessibility_label(I18N.text("ui-below-three-to-one-compose"))
                    .on_click(|| {});
            });
    }));
    gap(Dp(16.0));
    caption("ui-above-3-1-on-primary-container-versus-a-low-contrast-pairing");
    gap(Dp(24.0));
    prose(
        "ui-the-pressed-and-focused-state-layers-use-the-content-color-so-a-custom-container-keeps-rea",
    );
}

#[tessera]
fn initial_focus() {
    prose(
        "ui-initial-focus-should-land-on-the-extended-fab-when-it-is-the-only-floating-action-on-the-s",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-compose");
    }));
    gap(Dp(16.0));
    caption("ui-press-tab-to-reach-the-action-the-whole-container-receives-focus-once");
}

#[tessera]
fn keyboard_navigation() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec!["ui-tab".into(), "ui-navigate-to-the-extended-fab".into()],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-extended-fab".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-extended-fab-behaves-like-a-button-for-keyboard-input-a-single-focus-stop-activated-by",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        fab("ui-compose").counted(true);
    }));
}

#[tessera]
fn labeling() {
    prose(
        "ui-the-accessible-name-of-an-extended-fab-is-its-visible-label-so-the-icon-must-not-repeat-it",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                fab("ui-compose");
                gap(Dp(16.0));
                text(I18N.text("ui-do-one-label-one-announcement-compose-button"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                fab("ui-compose").tone(1);
                gap(Dp(16.0));
                text(I18N.text("ui-don-t-compose-edit-pencil-icon-button-the-icon-description-duplicates-the-action"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "ui-when-the-label-collapses-keep-the-same-accessible-name-provide-a-visible-label-on-request",
    );
}
