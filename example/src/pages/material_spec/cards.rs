//! Interactive card specifications by container style.
//!
//! ## Usage
//!
//! Compare related-content containers, emphasis and live interaction feedback.

use tessera_components::{
    alignment::{Alignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    card::{CardVariant, card},
    carousel::{CarouselVariant, carousel},
    column::column,
    flow_row::flow_row,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    ACCENT,
    annotations::{self, Geometry, SpaceGeometry, dotted, relative, watch, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "cards_figures_tests.rs"]
mod figures_tests;

const VARIANTS: [CardVariant; 3] = [
    CardVariant::Elevated,
    CardVariant::Filled,
    CardVariant::Outlined,
];

fn name(variant: CardVariant) -> &'static str {
    match variant {
        CardVariant::Elevated => "ui-elevated",
        CardVariant::Filled => "ui-filled",
        CardVariant::Outlined => "ui-outlined",
    }
}

#[tessera]
pub(super) fn cards_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-cards",
        "ui-present-content-and-actions-about-one-subject-in-a-distinct-container",
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-elevated-card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Elevated);
                }),
            },
            SpecSection {
                title: "ui-filled-card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Filled);
                }),
            },
            SpecSection {
                title: "ui-outlined-card",
                content: RenderSlot::new(|| {
                    detail(CardVariant::Outlined);
                }),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    ).route_optional(route).pages(vec![
        ArticlePage {title:"ui-overview",intro:Some(RenderSlot::new(|| {prose("ui-cards-contain-related-elements-about-a-single-subject-they-can-hold-images-headlines-suppo");gap(Dp(24.0));card_variants();})),sections:vec![
            SpecSection {title:"ui-differences-from-m2",content:RenderSlot::new(|| {prose("ui-cards-use-dynamic-color-mappings-and-lower-resting-elevation-choose-elevated-filled-or-out");gap(Dp(24.0));card_variants();})},
        ]},
        ArticlePage {title:"ui-guidelines",intro:None,sections:vec![
            SpecSection {title:"ui-usage",content:RenderSlot::new(|| {prose("ui-keep-each-card-about-one-subject-establish-a-readable-hierarchy-and-use-a-collection-when");gap(Dp(24.0));card_variants();})},
            SpecSection {title:"ui-anatomy",content:RenderSlot::new(card_guideline_anatomy)},
            SpecSection {title:"ui-cards-in-a-collection",content:RenderSlot::new(card_collections)},
            SpecSection {title:"ui-adaptive-design",content:RenderSlot::new(card_adaptive)},
            SpecSection {title:"ui-behavior",content:RenderSlot::new(card_behavior)},
        ]},
        ArticlePage {title:"ui-accessibility",intro:None,sections:vec![
            SpecSection {title:"ui-use-cases",content:RenderSlot::new(|| {prose("ui-people-must-be-able-to-reach-a-card-and-its-controls-understand-its-content-and-receive-fe");gap(Dp(24.0));action_models();})},
            SpecSection {title:"ui-interaction-style",content:RenderSlot::new(card_interaction)},
            SpecSection {title:"ui-focus",content:RenderSlot::new(|| {prose("ui-a-directly-actionable-card-is-one-tab-stop-a-passive-card-is-skipped-while-its-buttons-and");gap(Dp(24.0));action_models();})},
            SpecSection {title:"ui-keyboard-navigation",content:RenderSlot::new(|| {spec_table(vec!["ui-keys".into(),"ui-action".into()],vec![vec!["ui-tab".into(),"ui-next-actionable-card-or-contained-control".into()],vec!["ui-space-or-enter".into(),"ui-activate-the-focused-action".into()]]);})},
            SpecSection {title:"ui-labeling-elements",content:RenderSlot::new(|| {prose("ui-expose-meaningful-headings-text-and-actions-in-reading-order-hide-purely-decorative-media");gap(Dp(24.0));card_figure(true);})},
        ]},
    ]);
}

#[tessera]
fn sample(
    variant: CardVariant,
    enabled: bool,
    #[prop(into)] label: String,
    modifier: Option<Modifier>,
) {
    let actions = remember(|| 0_u32);
    column().children(move || {
        let caption = label.clone();
        let label = label.clone();
        card()
            .variant(variant)
            .enabled(enabled)
            .modifier(modifier.clone().unwrap_or_default().width(Dp(280.0)))
            .accessibility_label(I18N.text(&(label)))
            .on_click(move || actions.set(actions.get() + 1))
            .content(move || {
                body(label.clone());
            });
        gap(Dp(8.0));
        text(I18N.format(
            "ui-value-activations-value",
            [
                ("v0", I18N.text(&caption).into()),
                ("v1", (actions.get()).into()),
            ],
        ))
        .style(type_style(12.0, 16.0));
    });
}

#[tessera]
fn body(#[prop(into)] label: String) {
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
        .children(move || {
            text(I18N.text(&(label))).style(type_style(24.0, 32.0));
            gap(Dp(8.0));
            text(I18N.text("ui-a-focused-collection-of-related-content"))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn tokens() {
    let variant = remember(|| CardVariant::Elevated);
    flow_row().item_spacing(Dp(8.0)).children(move || {
        for style in VARIANTS {
            button()
                .selected(variant.get() == style)
                .accessibility_label(I18N.format(
                    "ui-value-card-tokens",
                    [("v0", I18N.text(&name(style)).into())],
                ))
                .on_click(move || variant.set(style))
                .child(move || {
                    text(I18N.text(name(style)));
                });
        }
    });
    gap(Dp(24.0));
    let (container, outline, shadow, tint, resting, hover, dragged) = match variant.get() {
        CardVariant::Elevated => (
            "Surface container low",
            "ui-none",
            "#000000",
            "Primary",
            "1dp",
            "2dp",
            "4dp",
        ),
        CardVariant::Filled => (
            "Surface container highest",
            "ui-none",
            "ui-none",
            "ui-none",
            "0dp",
            "1dp",
            "3dp",
        ),
        CardVariant::Outlined => (
            "Surface",
            "Outline variant · 1dp",
            "ui-none",
            "ui-none",
            "0dp",
            "1dp",
            "3dp",
        ),
    };
    subheading("ui-element-tokens".to_owned());
    gap(Dp(16.0));
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-container".into(),
                "ui-container-color".into(),
                container.into(),
            ],
            vec![
                "ui-outline-element".into(),
                "ui-outline-color-and-width".into(),
                outline.into(),
            ],
            vec![
                "ui-shape".into(),
                "ui-container-shape".into(),
                "ui-12dp-corners".into(),
            ],
            vec!["ui-shadow".into(), "ui-shadow-color".into(), shadow.into()],
            vec![
                "ui-tint".into(),
                "ui-surface-tint-layer-color".into(),
                tint.into(),
            ],
            vec![
                "ui-label".into(),
                "ui-label-text-color".into(),
                "On surface".into(),
            ],
            vec![
                "ui-icon".into(),
                "ui-icon-color-and-size".into(),
                "On surface variant · 24dp".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-state-tokens".to_owned());
    gap(Dp(16.0));
    spec_table(
        vec!["ui-state".into(), "ui-change".into()],
        vec![
            vec![
                "ui-enabled".into(),
                I18N.format(
                    "ui-value-resting-elevation-with-the-element-tokens-above",
                    [("v0", I18N.text(&resting).into())],
                ),
            ],
            vec![
                "ui-disabled".into(),
                "ui-content-at-38-with-the-shadow-removed".into(),
            ],
            vec![
                "ui-hovered".into(),
                I18N.format(
                    "ui-value-elevation-and-an-8-state-layer",
                    [("v0", I18N.text(&hover).into())],
                ),
            ],
            vec![
                "ui-focused".into(),
                I18N.format(
                    "ui-3dp-focus-indicator-at-value-elevation",
                    [("v0", I18N.text(&resting).into())],
                ),
            ],
            vec![
                "ui-pressed-ripple".into(),
                "ui-10-state-layer-with-the-ripple".into(),
            ],
            vec![
                "ui-dragged".into(),
                I18N.format(
                    "ui-value-elevation-without-a-state-layer",
                    [("v0", I18N.text(&dragged).into())],
                ),
            ],
        ],
    );
}

#[tessera]
fn scheme(variant: CardVariant, dark: bool) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, dark))
        .child(move || {
            let theme = use_context::<MaterialTheme>()
                .expect("MaterialTheme must be provided")
                .get();
            surface()
                .style(theme.color_scheme.surface.into())
                .modifier(Modifier::new().fill_max_width())
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                        .children(move || {
                            subheading(
                                if dark {
                                    "ui-dark-theme"
                                } else {
                                    "ui-light-theme"
                                }
                                .to_owned(),
                            );
                            gap(Dp(24.0));
                            sample(
                                variant,
                                true,
                                I18N.format(
                                    "ui-value-value-format-25",
                                    [
                                        ("v0", I18N.text(&name(variant)).into()),
                                        (
                                            "v1",
                                            I18N.text(
                                                &(if dark {
                                                    "ui-dark-7069"
                                                } else {
                                                    "ui-light-7070"
                                                }),
                                            )
                                            .into(),
                                        ),
                                    ],
                                ),
                            );
                        });
                });
        });
}

#[tessera]
fn detail(variant: CardVariant) {
    let geometry = remember(SpaceGeometry::new);
    boxed()
        .modifier(watch_in_space(geometry, "figure").fill_max_width())
        .children(move || {
            panel(RenderSlot::new(move || {
                sample(
                    variant,
                    true,
                    I18N.format("ui-value-card", [("v0", I18N.text(&name(variant)).into())]),
                )
                .modifier(watch_in_space(geometry, "container"));
            }));
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    card_container_annotations(variant, geometry);
                });
        });
    gap(Dp(24.0));
    text(I18N.text(if variant == CardVariant::Outlined {
        "ui-1-container-2-outline"
    } else {
        "ui-1-container"
    }))
    .style(type_style(16.0, 24.0));
    gap(Dp(48.0));
    subheading(I18N.format(
        "ui-value-card-color",
        [("v0", I18N.text(&name(variant)).into())],
    ));
    gap(Dp(24.0));
    scheme(variant, false);
    gap(Dp(24.0));
    scheme(variant, true);
    gap(Dp(48.0));
    subheading(I18N.format(
        "ui-value-card-states",
        [("v0", I18N.text(&name(variant)).into())],
    ));
    gap(Dp(24.0));
    text(I18N.text("ui-hover-focus-with-tab-or-hold-a-card-to-inspect-actual-feedback-disabled-cards-do-not-activ"))
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                sample(
                    variant,
                    true,
                    I18N.format(
                        "ui-value-enabled",
                        [("v0", I18N.text(&name(variant)).into())],
                    ),
                );
                sample(
                    variant,
                    false,
                    I18N.format(
                        "ui-value-disabled",
                        [("v0", I18N.text(&name(variant)).into())],
                    ),
                );
            });
    }));
}

#[tessera]
fn card_container_annotations(variant: CardVariant, geometry: State<SpaceGeometry>) {
    let values = geometry.with(annotations::space_geometry);
    let (Some(root), Some(container)) = (values.get("figure"), values.get("container")) else {
        return;
    };
    let mut svg = String::new();
    let y = container.y + container.h / 2.0;
    let left = container.x - 48.0;
    dotted(
        &mut svg,
        left + 24.0,
        y,
        container.x - left - 20.0,
        false,
        "#49454f",
    );
    annotations::marker(1, Dp(left), Dp(y - 16.0));
    if variant == CardVariant::Outlined {
        let right = container.x + container.w;
        dotted(&mut svg, right, y, 32.0, false, "#49454f");
        annotations::marker(2, Dp(right + 32.0), Dp(y - 16.0));
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                sample(CardVariant::Outlined, true, "ui-first-card");
                sample(CardVariant::Outlined, true, "ui-second-card");
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-shape".into(), "ui-12dp-corner-radius".into()],
            vec!["ui-content-left-right-padding".into(), "16dp".into()],
            vec!["ui-between-cards".into(), "ui-8dp-maximum".into()],
            vec!["ui-label-alignment".into(), "ui-start-aligned".into()],
        ],
    );
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn card_variants() {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                for variant in VARIANTS {
                    sample(
                        variant,
                        true,
                        I18N.format(
                            "ui-value-overview-card",
                            [("v0", I18N.text(&name(variant)).into())],
                        ),
                    );
                }
            });
    }));
}

#[tessera]
fn card_figure(annotated: bool) {
    let geometry = remember(Geometry::new);
    let calls = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().alignment(Alignment::Center).modifier(watch(geometry,"figure").fill_max_width().height(Dp(584.0))
        .background_with_shape(scheme.surface_container_low,Shape::rounded_rectangle(Dp(24.0))))
        .children(move || {
            card().elevated().modifier(watch(geometry,"container").width(Dp(448.0))).content(move || {
                column().modifier(Modifier::new().fill_max_width()).children(move || {
                    spacer().modifier(watch(geometry,"media").fill_max_width().height(Dp(194.0)).background(scheme.tertiary_container));
                    column().modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0))).children(move || {
                        text(I18N.text("ui-headline")).style(type_style(40.0,48.0)).modifier(watch(geometry,"headline"));
                        text(I18N.text("ui-subhead")).style(type_style(24.0,32.0)).modifier(watch(geometry,"subhead"));
                        gap(Dp(32.0));
                        text(I18N.text("ui-explain-more-about-the-topic-shown-in-the-headline-and-subhead-through-supporting-text"))
                            .style(type_style(14.0,20.0)).modifier(watch(geometry,"support"));
                        gap(Dp(32.0));
                        row().main_axis_alignment(MainAxisAlignment::End).modifier(Modifier::new().fill_max_width()).children(move || {
                            button().outlined().accessibility_label(I18N.text("ui-card-secondary-action")).on_click(move || calls.set(calls.get()+1)).child(move || {text(I18N.text("ui-action"));});
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            button().modifier(watch(geometry,"action")).accessibility_label(I18N.text("ui-card-primary-button")).on_click(move || calls.set(calls.get()+1)).child(move || {text(I18N.text("ui-action"));});
                        });
                    });
                });
            });
            if annotated {boxed().modifier(Modifier::new().fill_max_size()).children(move || {card_callouts(geometry);});}
        });
    gap(Dp(8.0));
    text(I18N.format(
        "ui-card-button-actions-value",
        [("v0", (calls.get()).into())],
    ))
    .style(type_style(12.0, 16.0));
}

#[tessera]
fn card_callouts(geometry: State<Geometry>) {
    let values = geometry.get();
    let (
        Some(root),
        Some(card),
        Some(media),
        Some(headline),
        Some(subhead),
        Some(support),
        Some(action),
    ) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "media"),
        relative(&values, "headline"),
        relative(&values, "subhead"),
        relative(&values, "support"),
        relative(&values, "action"),
    )
    else {
        return;
    };
    let mut svg = String::new();
    for (number, target, right) in [
        (1, card, false),
        (2, media, true),
        (3, action, true),
        (4, support, false),
        (5, subhead, false),
        (6, headline, false),
    ] {
        let y = if number == 1 {
            card.y + 96.0
        } else if number == 2 {
            media.y + media.h - 16.0
        } else {
            target.y + target.h / 2.0
        };
        let x = if right {
            card.x + card.w + 32.0
        } else {
            card.x - 48.0
        };
        let tx = if right {
            target.x + target.w - 4.0
        } else {
            target.x + 4.0
        };
        dotted(
            &mut svg,
            if right { tx } else { x + 24.0 },
            y,
            if right { x - tx } else { tx - x - 24.0 },
            false,
            "#49454f",
        );
        annotations::marker(number, Dp(x), Dp(y - 16.0));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn card_guideline_anatomy() {
    prose(
        "ui-only-the-container-is-required-optional-content-blocks-establish-hierarchy-through-media-a",
    );
    gap(Dp(24.0));
    card_figure(true);
    gap(Dp(24.0));
    subheading("ui-container".into());
    prose(
        "ui-the-content-determines-the-container-s-dimensions-its-surface-and-elevation-distinguish-th",
    );
    gap(Dp(24.0));
    subheading("ui-content-blocks".into());
    prose(
        "ui-group-related-elements-into-clear-blocks-and-give-the-most-important-information-stronger",
    );
    gap(Dp(24.0));
    subheading("ui-dividers".into());
    prose(
        "ui-use-full-width-dividers-around-expandable-content-and-inset-dividers-between-related-regio",
    );
    gap(Dp(24.0));
    subheading("ui-media".into());
    prose(
        "ui-a-card-can-contain-a-thumbnail-image-illustration-or-video-keep-decorative-media-outside-t",
    );
    gap(Dp(24.0));
    subheading("ui-text".into());
    prose(
        "ui-use-the-headline-for-the-subject-a-subhead-for-context-and-supporting-text-for-details-pre",
    );
    gap(Dp(24.0));
    subheading("ui-actions".into());
    action_models();
}

#[tessera]
fn collection_card(index: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    card()
        .filled()
        .modifier(Modifier::new().width(Dp(176.0)))
        .content(move || {
            column().children(move || {
                spacer().modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(96.0))
                        .background(if index % 2 == 0 {
                            scheme.secondary_container
                        } else {
                            scheme.tertiary_container
                        }),
                );
                text(I18N.format("ui-collection-value", [("v0", (index + 1).into())]))
                    .style(type_style(18.0, 24.0))
                    .modifier(Modifier::new().padding_all(Dp(16.0)));
            });
        });
}

#[tessera]
fn card_collections() {
    prose(
        "ui-a-collection-shares-a-resting-elevation-place-sorting-and-filtering-controls-outside-the-c",
    );
    gap(Dp(24.0));
    subheading("ui-grid".into());
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                for index in 0..6 {
                    collection_card(index);
                }
            });
    }));
    gap(Dp(24.0));
    subheading("ui-vertical-list".into());
    panel(RenderSlot::new(|| {
        column().children(|| {
            for index in 0..3 {
                collection_card(index);
                gap(Dp(8.0));
            }
        });
    }));
    gap(Dp(24.0));
    subheading("ui-carousel".into());
    panel(RenderSlot::new(move || {
        carousel(5)
            .variant(CarouselVariant::Uncontained)
            .height(Dp(152.0))
            .preferred_item_width(Dp(176.0))
            .item_spacing(Dp(8.0))
            .accessibility_label(I18N.text("ui-card-collection-carousel"))
            .content(|item| {
                collection_card(item.index);
            });
    }));
}

#[tessera]
fn card_adaptive() {
    subheading("ui-ergonomics".into());
    prose(
        "ui-adapt-the-orientation-and-size-of-content-blocks-to-the-available-space-keeping-the-main-c",
    );
    gap(Dp(24.0));
    subheading("ui-visual-presentation".into());
    prose(
        "ui-adjust-spacing-before-stretching-text-heavy-cards-let-media-and-content-blocks-respond-to",
    );
    gap(Dp(24.0));
    subheading("ui-column-based-layouts".into());
    prose(
        "ui-use-multiple-columns-on-wide-screens-and-a-single-readable-column-in-compact-layouts",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(|| {
                for index in 0..4 {
                    collection_card(index);
                }
            });
    }));
    gap(Dp(24.0));
    subheading("ui-small-screens".into());
    prose(
        "ui-a-compact-list-can-replace-a-card-collection-when-space-is-limited-preserve-the-same-infor",
    );
}

#[tessera]
fn action_models() {
    let clicks = remember(|| 0_usize);
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                card()
                    .outlined()
                    .modifier(Modifier::new().width(Dp(248.0)))
                    .content(move || {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(move || {
                                text(I18N.text("ui-non-actionable-card"))
                                    .style(type_style(20.0, 28.0));
                                gap(Dp(16.0));
                                button()
                                    .on_click(move || clicks.set(clicks.get() + 1))
                                    .child(move || {
                                        text(I18N.text("ui-action"));
                                    });
                            });
                    });
                card()
                    .outlined()
                    .modifier(Modifier::new().width(Dp(248.0)))
                    .accessibility_label(I18N.text("ui-open-card-details"))
                    .on_click(move || clicks.set(clicks.get() + 1))
                    .content(|| {
                        body("ui-directly-actionable-card");
                    });
            });
        gap(Dp(16.0));
        prose(I18N.format("ui-card-actions-value", [("v0", (clicks.get()).into())]));
    }));
}

#[tessera]
fn card_behavior() {
    subheading("ui-expanding".into());
    prose(
        "ui-reveal-additional-content-by-expanding-the-card-avoid-an-independent-scrolling-region-insi",
    );
    gap(Dp(24.0));
    expanding_card();
    gap(Dp(24.0));
    subheading("ui-navigation".into());
    prose(
        "ui-a-card-may-navigate-to-details-at-the-next-level-of-hierarchy-keep-its-destination-clear-f",
    );
    gap(Dp(24.0));
    subheading("ui-gestures".into());
    prose(
        "ui-assign-one-swipe-action-per-card-and-provide-a-visible-alternative-for-moving-or-dismissin",
    );
}

#[tessera]
fn expanding_card() {
    let expanded = remember(|| false);
    panel(RenderSlot::new(move || {
        card().outlined().modifier(Modifier::new().width(Dp(360.0))).content(move || {column().modifier(Modifier::new().padding_all(Dp(16.0))).children(move || {text(I18N.text("ui-explore-the-details")).style(type_style(24.0,32.0));gap(Dp(16.0));if expanded.get() {prose("ui-expanded-content-stays-within-the-card-s-hierarchy-and-grows-the-card-instead-of-introduci");gap(Dp(16.0));}button().text().accessibility_label(I18N.text("ui-toggle-card-details")).on_click(move || expanded.set(!expanded.get())).child(move || {text(I18N.text(if expanded.get() {"ui-show-less"} else {"ui-learn-more"}));});});});
    }));
}

#[tessera]
fn card_interaction() {
    prose(
        "ui-use-either-a-directly-actionable-card-or-a-passive-container-with-internal-actions-avoid-s",
    );
    gap(Dp(24.0));
    subheading("ui-touch".into());
    prose(
        "ui-directly-actionable-cards-ripple-across-their-surface-passive-cards-leave-feedback-to-thei",
    );
    gap(Dp(24.0));
    subheading("ui-dragging-and-dismissing".into());
    prose(
        "ui-offer-a-single-pointer-action-as-an-alternative-to-dragging-or-swiping-keep-that-action-re",
    );
    gap(Dp(24.0));
    subheading("ui-cursor".into());
    prose("ui-hover-and-click-feedback-belong-to-the-actionable-surface-or-control");
    gap(Dp(24.0));
    subheading("ui-keyboard".into());
    prose("ui-tab-reaches-every-action-space-or-enter-activates-the-focused-card-or-button");
    gap(Dp(24.0));
    action_models();
}
