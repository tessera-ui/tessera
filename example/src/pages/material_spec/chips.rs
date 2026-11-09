//! Interactive chip specifications by purpose.
//!
//! ## Usage
//!
//! Compare compact actions, independent filters, input tokens and suggestions.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    chip::{ChipShapeStyle, ChipStyle, ChipVariant, chip},
    column::column,
    flow_row::flow_row,
    material_icons::filled,
    modifier::ModifierExt as _,
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    article_helpers::{bullets, caption, do_dont, legend, prose},
    gap, panel, scheme_panel, spec_table, subheading, type_style,
};

const VARIANTS: [ChipVariant; 4] = [
    ChipVariant::Assist,
    ChipVariant::Filter,
    ChipVariant::Input,
    ChipVariant::Suggestion,
];

fn name(variant: ChipVariant) -> &'static str {
    match variant {
        ChipVariant::Assist => "ui-assist",
        ChipVariant::Filter => "ui-filter",
        ChipVariant::Input => "ui-input",
        ChipVariant::Suggestion => "ui-suggestion",
    }
}

fn token_prefix(variant: ChipVariant) -> &'static str {
    match variant {
        ChipVariant::Assist => "md.comp.assist-chip",
        ChipVariant::Filter => "md.comp.filter-chip",
        ChipVariant::Input => "md.comp.input-chip",
        ChipVariant::Suggestion => "md.comp.suggestion-chip",
    }
}

fn selectable(variant: ChipVariant) -> bool {
    matches!(variant, ChipVariant::Filter | ChipVariant::Input)
}

#[tessera]
pub(super) fn chips_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-chips",
        "ui-chips-help-people-enter-information-make-selections-filter-content-or-trigger-actions",
        vec![
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-assist-chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Assist);
                }),
            },
            SpecSection {
                title: "ui-filter-chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Filter);
                }),
            },
            SpecSection {
                title: "ui-input-chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Input);
                }),
            },
            SpecSection {
                title: "ui-suggestion-chip",
                content: RenderSlot::new(|| {
                    detail(ChipVariant::Suggestion);
                }),
            },
        ],
    )
    .route_optional(route.clone())
    .hero(move || {
        panel(RenderSlot::new(move || {
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(move || {
                    for variant in VARIANTS {
                        sample(
                            variant,
                            ChipStyle::Flat,
                            false,
                            true,
                            I18N.format(
                                "ui-value-example",
                                [("v0", I18N.text(&name(variant)).into())],
                            ),
                        );
                    }
                });
        }));
    })
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(guidelines_intro)),
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(chip_usage),
                },
                SpecSection {
                    title: "ui-variants",
                    content: RenderSlot::new(variants_guideline),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(chip_guideline_anatomy),
                },
                SpecSection {
                    title: "ui-assist-chips",
                    content: RenderSlot::new(assist_guidelines),
                },
                SpecSection {
                    title: "ui-filter-chips",
                    content: RenderSlot::new(filter_guidelines),
                },
                SpecSection {
                    title: "ui-input-chips",
                    content: RenderSlot::new(input_guidelines),
                },
                SpecSection {
                    title: "ui-suggestion-chips",
                    content: RenderSlot::new(suggestion_guidelines),
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
                    content: RenderSlot::new(keyboard),
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
fn overview() {
    bullets(vec![
        "ui-use-chips-to-show-options-for-a-specific-context".into(),
        "ui-four-variants-assist-filter-input-and-suggestion".into(),
        "ui-chip-elevation-defaults-to-0-but-can-be-elevated-if-they-need-more-visual-separation"
            .into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(move || {
                        for variant in VARIANTS {
                            sample(
                                variant,
                                ChipStyle::Flat,
                                variant == ChipVariant::Filter,
                                true,
                                I18N.format(
                                    "ui-value-chip",
                                    [("v0", I18N.text(&name(variant)).into())],
                                ),
                            );
                        }
                    });
            });
    }));
    gap(Dp(16.0));
    column().children(move || {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(
                index + 1,
                I18N.format("ui-value-chip", [("v0", I18N.text(&name(variant)).into())]),
            );
        }
    });
    gap(Dp(12.0));
    caption("ui-there-are-four-chip-variants-assist-filter-input-and-suggestion");
}

#[tessera]
fn guidelines_intro() {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(24.0))
            .line_spacing(Dp(24.0))
            .children(move || {
                chip()
                    .assist(I18N.text("ui-assist"))
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::EVENT_SVG);
                chip()
                    .filter(I18N.text("ui-filter"))
                    .shape_style(ChipShapeStyle::Standard)
                    .selected(true)
                    .leading_icon(filled::CHECK_SVG);
                chip()
                    .input(I18N.text("ui-input"))
                    .shape_style(ChipShapeStyle::Standard)
                    .on_remove(|| {});
                chip()
                    .suggestion(I18N.text("ui-suggestion"))
                    .shape_style(ChipShapeStyle::Standard);
            });
    }));
    gap(Dp(16.0));
    caption("ui-there-are-four-variants-of-chips-assist-filter-input-and-suggestion");
}

#[tessera]
fn chip_usage() {
    prose(
        "ui-chips-help-people-enter-information-make-selections-filter-content-or-trigger-actions-they",
    );
    gap(Dp(24.0));
    subheading("ui-chips-aren-t-buttons".into());
    prose(
        "ui-chips-and-buttons-are-similar-they-both-provide-visual-cues-to-prompt-people-to-take-actio",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-chips-to-enhance-a-person-s-current-journey-and-encourage-action-use-buttons-to-progre",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-use-chips-to-present-contextual-supplemental-options",
        "ui-avoid-replacing-major-actions-with-chips-actions-that-progress-people-to-the-next-or-previ",
    );
    gap(Dp(32.0));
    prose(
        "ui-chips-should-dynamically-offer-various-actions-depending-on-the-current-task-whereas-a-but",
    );
    gap(Dp(16.0));
    prose(
        "ui-chips-represent-forking-paths-for-a-current-task-while-buttons-represent-linear-steps",
    );
    gap(Dp(24.0));
    assist_example();
    gap(Dp(24.0));
    do_dont(
        "ui-use-buttons-for-the-final-step-in-a-task",
        "ui-avoid-using-chips-to-finish-or-progress-a-task",
    );
    gap(Dp(32.0));
    prose(
        "ui-multiple-chips-should-appear-together-in-a-set-whereas-there-should-be-no-more-than-3-butt",
    );
    gap(Dp(16.0));
    prose("ui-chip-sets-can-be-scrolled-horizontally");
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(24.0));
    do_dont(
        "ui-chips-can-be-scrolled-horizontally",
        "ui-don-t-display-a-single-chip-by-itself-chips-should-appear-in-a-set",
    );
}

#[tessera]
fn variants_guideline() {
    prose("ui-choose-the-chip-variant-based-on-its-purpose-and-author");
    gap(Dp(16.0));
    prose("ui-does-the-chip-represent-an-action-assist-chip-or-filter-results-filter-chip");
    gap(Dp(16.0));
    prose(
        "ui-is-its-content-generated-by-the-product-suggestion-chip-or-by-the-person-using-the-product",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                flow_row()
                    .item_spacing(Dp(24.0))
                    .line_spacing(Dp(24.0))
                    .children(move || {
                        chip()
                            .assist(I18N.text("ui-assist"))
                            .shape_style(ChipShapeStyle::Standard)
                            .leading_icon(filled::EVENT_SVG);
                        chip()
                            .filter(I18N.text("ui-filter"))
                            .shape_style(ChipShapeStyle::Standard)
                            .leading_icon(filled::CHECK_SVG);
                        chip()
                            .input(I18N.text("ui-input"))
                            .shape_style(ChipShapeStyle::Standard)
                            .on_remove(|| {});
                        chip()
                            .suggestion(I18N.text("ui-suggestion"))
                            .shape_style(ChipShapeStyle::Standard);
                    });
            });
    }));
    gap(Dp(16.0));
    column().children(|| {
        for (index, variant) in VARIANTS.into_iter().enumerate() {
            legend(index + 1, name(variant));
        }
    });
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-purpose".into(),
            "ui-chip-variant".into(),
            "ui-rationale".into(),
            "ui-example".into(),
        ],
        vec![
            vec![
                "ui-action".into(),
                "ui-assist-chip".into(),
                "ui-assist-chips-represent-smart-or-automated-actions-that-can-span-multiple-apps"
                    .into(),
                "ui-add-to-calendar-action".into(),
            ],
            vec![
                "ui-filter".into(),
                "ui-filter-chip".into(),
                "ui-filter-chips-represent-filters-for-a-collection".into(),
                "ui-platform-selector-on-material-io-components".into(),
            ],
            vec![
                "ui-information-user-authored".into(),
                "ui-input-chip".into(),
                "ui-input-chips-represent-discrete-pieces-of-information-entered-by-a-person".into(),
                "ui-gmail-contact-in-the-to-field".into(),
            ],
            vec![
                "ui-information-product-authored".into(),
                "ui-suggestion-chip".into(),
                "ui-suggestion-chips-help-narrow-a-person-s-intent-by-presenting-dynamically-generated-suggest".into(),
                "ui-suggested-chat-response".into(),
            ],
        ],
    );
}

#[tessera]
fn chip_guideline_anatomy() {
    chip_diagram(ChipVariant::Input, false);
    gap(Dp(16.0));
    caption("ui-a-chip-contains-a-container-a-label-and-optional-leading-and-trailing-elements");
    gap(Dp(32.0));
    subheading("ui-container".into());
    prose("ui-all-chips-are-slightly-rounded-with-an-8dp-corner");
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Filter, true);
    gap(Dp(16.0));
    caption("ui-chips-have-rounded-corners");
    gap(Dp(32.0));
    subheading("ui-shadows-elevation".into());
    prose("ui-chip-containers-can-be-elevated-if-placed-on-top-of-an-image-or-dynamic-background");
    gap(Dp(24.0));
    elevation_example();
    gap(Dp(16.0));
    caption("ui-when-on-complicated-backgrounds-chip-containers-can-be-elevated");
    gap(Dp(16.0));
    do_dont(
        "ui-use-an-outline-to-define-the-edge-of-the-chip-s-container-on-regular-backgrounds",
        "ui-chips-shouldn-t-be-elevated-when-placed-directly-on-the-page",
    );
    gap(Dp(16.0));
    do_dont(
        "ui-chips-may-use-elevation-when-placed-on-an-image",
        "ui-avoid-using-elevation-to-indicate-a-chip-s-pressed-state-instead-use-the-visual-ripple-eff",
    );
    gap(Dp(32.0));
    subheading("ui-label-text".into());
    prose(
        "ui-chip-label-text-should-be-20-characters-or-fewer-and-have-the-same-typography-style-as-but",
    );
    gap(Dp(16.0));
    prose(
        "ui-chip-labels-should-remain-brief-for-the-limited-space-available-skip-conventional-grammar",
    );
    gap(Dp(24.0));
    label_length_example();
    gap(Dp(16.0));
    do_dont(
        "ui-keep-chip-labels-short",
        "ui-avoid-chip-labels-longer-than-20-characters",
    );
    gap(Dp(32.0));
    subheading("ui-leading-icon-or-image-optional".into());
    prose(
        "ui-chips-can-contain-a-leading-icon-logo-or-circular-image-use-a-system-icon-to-help-identify",
    );
    gap(Dp(24.0));
    leading_examples();
    gap(Dp(16.0));
    caption("ui-chips-can-contain-a-logo-icon-or-circular-image");
    gap(Dp(24.0));
    prose(
        "ui-the-leading-icon-color-for-unselected-chips-can-be-customized-through-theming-while-the-de",
    );
    gap(Dp(16.0));
    prose(
        "ui-leading-circular-images-are-sized-larger-than-leading-icons-to-provide-more-space-for-deta",
    );
    gap(Dp(16.0));
    caption("ui-leading-images-such-as-avatars-are-sized-larger-than-leading-icons-or-logos");
    gap(Dp(32.0));
    subheading("ui-trailing-icon-optional-input-and-filter-chips-only".into());
    prose("ui-the-trailing-icon-is-present-for-input-and-filter-chips");
    gap(Dp(16.0));
    prose(
        "ui-on-input-chips-it-s-required-and-must-be-used-to-remove-the-chip-on-filter-chips-it-s-opti",
    );
    gap(Dp(16.0));
    prose(
        "ui-secondary-actions-such-as-a-trailing-icon-button-for-remove-must-have-a-48x48dp-interactio",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, true);
    gap(Dp(16.0));
    caption(
        "ui-interaction-targets-for-actions-like-edit-or-close-have-a-size-of-48x48-this-can-be-achiev",
    );
}

#[tessera]
fn elevation_example() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column().children(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_all(Dp(20.0))
                    .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0))),
            )
            .children(move || {
                flow_row().item_spacing(Dp(8.0)).children(move || {
                    chip()
                        .assist(I18N.text("ui-assist"))
                        .shape_style(ChipShapeStyle::Standard)
                        .leading_icon(filled::EVENT_SVG);
                    chip()
                        .filter(I18N.text("ui-filter"))
                        .shape_style(ChipShapeStyle::Standard);
                    chip()
                        .suggestion(I18N.text("ui-suggestion"))
                        .shape_style(ChipShapeStyle::Standard);
                });
            });
        gap(Dp(16.0));
        boxed()
            .modifier(
                Modifier::new()
                    .fill_max_width()
                    .padding_all(Dp(20.0))
                    .background_with_shape(
                        scheme.surface_container_highest,
                        Shape::rounded_rectangle(Dp(16.0)),
                    ),
            )
            .children(move || {
                flow_row().item_spacing(Dp(8.0)).children(move || {
                    chip()
                        .assist(I18N.text("ui-assist"))
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard)
                        .leading_icon(filled::EVENT_SVG);
                    chip()
                        .filter(I18N.text("ui-filter"))
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard);
                    chip()
                        .suggestion(I18N.text("ui-suggestion"))
                        .style(ChipStyle::Elevated)
                        .shape_style(ChipShapeStyle::Standard);
                });
            });
    });
    gap(Dp(12.0));
    caption(
        "ui-flat-chips-suit-regular-surfaces-elevated-chips-separate-containers-from-imagery-or-dynami",
    );
}

#[tessera]
fn label_length_example() {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                chip()
                    .assist(I18N.text("ui-take-walk"))
                    .shape_style(ChipShapeStyle::Standard);
                chip()
                    .assist(I18N.text("ui-take-a-long-walk-around-the-park"))
                    .shape_style(ChipShapeStyle::Standard);
            });
    }));
}

#[tessera]
fn leading_examples() {
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                chip()
                    .input(I18N.text("ui-ping-qiang"))
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::PERSON_SVG)
                    .on_remove(|| {});
                chip()
                    .filter(I18N.text("ui-type-images"))
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::IMAGE_SVG);
            });
    }));
}

#[tessera]
fn chip_diagram(variant: ChipVariant, measurements: bool) {
    let geometry = remember(Geometry::new);
    let selected = remember(|| false);
    let removed = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(280.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            let content = RenderSlot::new(move || {
                if removed.get() {
                    button()
                        .outlined()
                        .on_click(move || removed.set(false))
                        .child(move || {
                            text(I18N.text("ui-restore-input-chip"));
                        });
                } else {
                    let chip =
                        chip()
                            .variant(variant)
                            .label(I18N.format(
                                "ui-value-chip",
                                [("v0", I18N.text(&name(variant)).into())],
                            ))
                            .modifier(watch(geometry, "chip"))
                            .shape_style(ChipShapeStyle::Standard)
                            .selected(selected.get())
                            .on_click(move || selected.set(!selected.get()));
                    match variant {
                        ChipVariant::Input => {
                            chip.leading_icon(filled::IMAGE_SVG)
                                .on_remove(move || removed.set(true));
                        }
                        ChipVariant::Filter => {
                            chip.leading_icon(filled::ADD_SVG)
                                .trailing_icon(filled::ARROW_DROP_DOWN_SVG);
                        }
                        ChipVariant::Assist => {
                            chip.leading_icon(filled::ADD_SVG);
                        }
                        ChipVariant::Suggestion => {}
                    }
                }
            });
            content.render();
            if !removed.get() {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        chip_callouts(geometry, variant, measurements);
                    });
            }
        });
}

#[tessera]
fn chip_callouts(geometry: State<Geometry>, variant: ChipVariant, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(chip)) = (values.get("figure"), relative(&values, "chip")) else {
        return;
    };
    let mut svg = String::new();
    let leading = Rect {
        x: chip.x + 8.0,
        y: chip.y + (chip.h - 18.0) / 2.0,
        w: 18.0,
        h: 18.0,
    };
    let trailing = Rect {
        x: chip.x + chip.w - 26.0,
        y: leading.y,
        w: 18.0,
        h: 18.0,
    };
    if measurements {
        let target = Rect {
            x: trailing.x - 15.0,
            y: chip.y + (chip.h - 48.0) / 2.0,
            w: 48.0,
            h: 48.0,
        };
        svg.push_str(&format!(
            "<rect x='{}' y='{}' width='48' height='48' fill='#ee7da4' fill-opacity='.25'/>",
            target.x, target.y
        ));
        annotations::outline(&mut svg, target);
        annotations::dimension("48".into(), Dp(target.x + 12.0), Dp(target.y - 32.0));
        annotations::dimension(
            "48".into(),
            Dp(target.x + target.w + 12.0),
            Dp(target.y + 14.0),
        );
        annotations::dimension("32".into(), Dp(chip.x - 40.0), Dp(chip.y + 6.0));
        annotations::dimension("ui-88dp-min".into(), Dp(chip.x), Dp(chip.y + chip.h + 40.0));
    } else {
        for (n, x, y, tx, ty) in [
            (1, chip.x + 16.0, chip.y - 64.0, chip.x + 28.0, chip.y),
            (
                2,
                chip.x + chip.w / 2.0 - 12.0,
                chip.y - 64.0,
                chip.x + chip.w / 2.0,
                chip.y + 16.0,
            ),
            (
                3,
                chip.x + chip.w + 32.0,
                chip.y,
                chip.x + chip.w - 17.0,
                chip.y + 16.0,
            ),
            (
                4,
                chip.x + 4.0,
                chip.y + chip.h + 24.0,
                leading.x + 9.0,
                leading.y + 18.0,
            ),
        ] {
            if n == 3 {
                dotted(&mut svg, tx, ty, x - tx, false, "#49454f");
            } else {
                dotted(
                    &mut svg,
                    tx,
                    if n == 4 { ty } else { y + 32.0 },
                    if n == 4 { y - ty } else { ty - y - 32.0 },
                    true,
                    "#49454f",
                );
            }
            if variant != ChipVariant::Suggestion || n < 3 {
                annotations::marker(n, Dp(x), Dp(y));
            }
        }
        if variant != ChipVariant::Suggestion {
            annotations::outline(&mut svg, leading);
        }
        if matches!(variant, ChipVariant::Input | ChipVariant::Filter) {
            annotations::outline(&mut svg, trailing);
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn assist_example() {
    let saved = remember(|| false);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text(I18N.text("ui-team-catch-up")).style(type_style(24.0, 32.0));
            prose("ui-wednesday-10-00-10-30");
            gap(Dp(16.0));
            flow_row().item_spacing(Dp(8.0)).children(move || {
                chip()
                    .assist(I18N.text(if saved.get() {
                        "ui-added"
                    } else {
                        "ui-add-to-calendar"
                    }))
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::EVENT_SVG)
                    .on_click(move || saved.set(!saved.get()));
                chip()
                    .assist(I18N.text("ui-get-directions"))
                    .shape_style(ChipShapeStyle::Standard)
                    .leading_icon(filled::DIRECTIONS_SVG)
                    .on_click(move || saved.set(true));
            });
        });
    }));
}

#[tessera]
fn assist_guidelines() {
    prose(
        "ui-assist-chips-represent-smart-or-automated-actions-that-can-span-multiple-apps-such-as-open",
    );
    gap(Dp(16.0));
    prose(
        "ui-an-alternative-to-assist-chips-are-buttons-which-should-appear-persistently-and-consistent",
    );
    gap(Dp(24.0));
    assist_example();
    gap(Dp(16.0));
    caption("ui-the-text-in-most-assist-chips-begins-with-a-short-verb-like-get-or-add");
    gap(Dp(24.0));
    prose(
        "ui-assist-chips-can-trigger-an-action-or-show-progress-and-confirmation-write-assist-chips-li",
    );
    gap(Dp(16.0));
    caption(
        "ui-tapping-an-assist-chip-triggers-a-contextual-action-and-the-chip-can-show-progress-and-con",
    );
    gap(Dp(24.0));
    prose(
        "ui-assist-chips-are-displayed-after-primary-content-such-as-below-a-card-or-persistently-at-t",
    );
}

#[tessera]
fn filter_example() {
    let selection = remember(|| [false; 3]);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text(I18N.text("ui-filter-results")).style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for index in 0..3 {
                        filter_choice(index, selection);
                    }
                });
            gap(Dp(16.0));
            text(I18N.format(
                "ui-value-filters-selected",
                [(
                    "v0",
                    (selection.get().into_iter().filter(|value| *value).count()).into(),
                )],
            ));
        });
    }));
}

#[tessera]
fn filter_choice(index: usize, selection: State<[bool; 3]>) {
    let selected = selection.get()[index];
    let chip = chip()
        .filter(I18N.text(["ui-unread", "ui-favorites", "ui-attachments"][index]))
        .shape_style(ChipShapeStyle::Standard)
        .selected(selected)
        .on_click(move || selection.with_mut(|value| value[index] = !value[index]));
    if selected {
        chip.leading_icon(filled::CHECK_SVG);
    }
}

#[tessera]
fn filter_guidelines() {
    prose(
        "ui-filter-chips-use-tags-or-descriptive-words-to-filter-content-they-can-be-a-good-alternativ",
    );
    gap(Dp(16.0));
    prose(
        "ui-tapping-on-a-filter-chip-activates-it-and-appends-a-leading-checkmark-icon-to-the-starting",
    );
    gap(Dp(16.0));
    prose(
        "ui-write-filter-chips-with-nouns-that-describe-the-category-to-include-in-the-results-avoid-n",
    );
    gap(Dp(24.0));
    filter_example();
    gap(Dp(16.0));
    caption("ui-filter-chips-rely-on-tags-or-descriptive-words-to-filter-content");
    gap(Dp(24.0));
    prose("ui-tap-a-chip-to-select-it-multiple-chips-can-be-selected-or-unselected");
    gap(Dp(16.0));
    caption("ui-filter-chips-in-a-shopping-app-and-in-a-real-estate-app");
    gap(Dp(24.0));
    do_dont(
        "ui-use-nouns-that-describe-the-category-to-include-in-results",
        "ui-avoid-negative-phrases-like-exclude-images",
    );
}

#[tessera]
fn input_example() {
    let visible = remember(|| [true; 3]);
    let selected = remember(|| [false; 3]);
    panel(RenderSlot::new(move || {
        column().children(move || {
            text(I18N.text("ui-to")).style(type_style(14.0, 20.0));
            gap(Dp(8.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for index in 0..3 {
                        recipient(index, visible, selected);
                    }
                });
            gap(Dp(16.0));
            button()
                .text()
                .accessibility_label(I18N.text("ui-restore-recipients"))
                .on_click(move || {
                    visible.set([true; 3]);
                    selected.set([false; 3]);
                })
                .child(move || {
                    text(I18N.text("ui-restore-recipients"));
                });
        });
    }));
}

#[tessera]
fn recipient(index: usize, visible: State<[bool; 3]>, selected: State<[bool; 3]>) {
    if visible.get()[index] {
        chip()
            .input(I18N.text(["ui-alex", "ui-sam", "ui-taylor"][index]))
            .shape_style(ChipShapeStyle::Standard)
            .leading_icon(filled::PERSON_SVG)
            .selected(selected.get()[index])
            .on_click(move || selected.with_mut(|value| value[index] = !value[index]))
            .on_remove(move || visible.with_mut(|value| value[index] = false));
    }
}

#[tessera]
fn input_guidelines() {
    prose(
        "ui-input-chips-represent-information-entered-by-a-person-their-body-can-select-or-edit-the-it",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-input-chips-for-discrete-pieces-of-information-such-as-recipients-filters-or-tags-a-le",
    );
    gap(Dp(24.0));
    input_example();
    gap(Dp(16.0));
    caption("ui-input-chips-represent-discrete-pieces-of-information-entered-by-a-person");
    gap(Dp(24.0));
    subheading("ui-placement".into());
    prose(
        "ui-place-input-chips-inline-in-a-wrapping-set-or-in-a-horizontally-scrolling-field-keep-all-v",
    );
    gap(Dp(24.0));
    chip_diagram(ChipVariant::Input, false);
    gap(Dp(16.0));
    do_dont(
        "ui-use-a-leading-avatar-or-icon-and-a-separate-trailing-remove-action",
        "ui-avoid-making-the-whole-chip-the-remove-action-the-chip-body-and-its-removal-stay-separate",
    );
}

#[tessera]
fn suggestion_example() {
    let reply = remember(String::new);
    panel(RenderSlot::new(move || {
        column().children(move || {
            prose("ui-would-you-like-to-meet-tomorrow");
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    for label in ["ui-sounds-good", "ui-maybe-later", "ui-what-time"] {
                        suggestion_choice(label.to_owned(), reply);
                    }
                });
            if !reply.with(String::is_empty) {
                gap(Dp(16.0));
                prose(I18N.format("ui-reply-value", [("v0", I18N.text(&reply.get()).into())]));
            }
        });
    }));
}

#[tessera]
fn suggestion_choice(#[prop(into)] label: String, reply: State<String>) {
    chip()
        .suggestion(I18N.text(&(label)))
        .shape_style(ChipShapeStyle::Standard)
        .on_click(move || reply.set(label.clone()));
}

#[tessera]
fn suggestion_guidelines() {
    prose(
        "ui-suggestion-chips-help-narrow-a-person-s-intent-by-presenting-dynamically-generated-suggest-1772",
    );
    gap(Dp(24.0));
    suggestion_example();
    gap(Dp(16.0));
    caption("ui-suggestion-chips-offer-short-product-generated-replies-or-search-suggestions");
    gap(Dp(24.0));
    subheading("ui-placement".into());
    prose(
        "ui-keep-suggestions-in-an-inline-set-that-wraps-as-needed-preserve-at-least-8dp-between-visua",
    );
    gap(Dp(24.0));
    do_dont(
        "ui-offer-a-short-set-of-suggestions-next-to-the-content-they-answer",
        "ui-avoid-presenting-a-single-suggestion-chip-on-its-own",
    );
}

#[tessera]
fn tokens() {
    let selected = remember(|| ChipVariant::Assist);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for variant in VARIANTS {
                button()
                    .text()
                    .accessibility_label(I18N.format(
                        "ui-value-chip-tokens",
                        [("v0", I18N.text(&name(variant)).into())],
                    ))
                    .on_click(move || selected.set(variant))
                    .child(move || {
                        text(I18N.text(name(variant)));
                    });
            }
        });
    gap(Dp(24.0));
    token_table(selected.get());
    gap(Dp(32.0));
    value_table(selected.get());
}

#[tessera]
fn token_table(variant: ChipVariant) {
    let prefix = token_prefix(variant);
    let selectable = selectable(variant);
    let mut rows = vec![
        vec![
            "ui-container-color".into(),
            format!("{prefix}.container.color"),
            if selectable {
                "ui-transparent"
            } else {
                "Surface"
            }
            .into(),
        ],
        vec![
            "ui-container-elevation".into(),
            format!("{prefix}.container.elevation"),
            "ui-0dp-elevated-1dp".into(),
        ],
        vec![
            "ui-label-color".into(),
            format!("{prefix}.label.text.color"),
            if variant == ChipVariant::Assist {
                "On surface"
            } else {
                "On surface variant"
            }
            .into(),
        ],
        vec![
            "ui-outline-color".into(),
            format!("{prefix}.outline.color"),
            if selectable {
                "Outline variant"
            } else {
                "Outline"
            }
            .into(),
        ],
        vec![
            "ui-shape".into(),
            format!("{prefix}.container.shape"),
            "ui-8dp-corner".into(),
        ],
    ];
    if variant != ChipVariant::Suggestion {
        rows.push(vec![
            "ui-leading-icon-color".into(),
            format!("{prefix}.leading-icon.color"),
            if variant == ChipVariant::Assist {
                "Primary"
            } else {
                "On surface variant"
            }
            .into(),
        ]);
    }
    if matches!(variant, ChipVariant::Filter | ChipVariant::Input) {
        rows.push(vec![
            "ui-trailing-icon-color".into(),
            format!("{prefix}.trailing-icon.color"),
            "On surface variant".into(),
        ]);
    }
    if selectable {
        rows.push(vec![
            "ui-selected-container".into(),
            format!("{prefix}.container.color"),
            "Secondary container".into(),
        ]);
        rows.push(vec![
            "ui-selected-label".into(),
            format!("{prefix}.label.text.color"),
            "On secondary container".into(),
        ]);
        rows.push(vec![
            "ui-selected-icon".into(),
            format!("{prefix}.leading-icon.color"),
            "On secondary container".into(),
        ]);
    }
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        rows,
    );
}

#[tessera]
fn value_table(variant: ChipVariant) {
    if variant == ChipVariant::Suggestion {
        spec_table(
            vec!["ui-attribute".into(), "ui-value".into()],
            vec![
                vec!["ui-container-height".into(), "32dp".into()],
                vec!["ui-container-shape".into(), "ui-8dp-corner-radius".into()],
                vec![
                    "ui-label-alignment".into(),
                    "ui-center-aligned-start-aligned".into(),
                ],
                vec!["ui-left-right-padding".into(), "16dp".into()],
                vec!["ui-padding-between-elements".into(), "8dp".into()],
            ],
        );
        return;
    }
    let mut rows = vec![
        vec!["ui-container-height".into(), "32dp".into()],
        vec!["ui-container-shape".into(), "ui-8dp-corner-radius".into()],
        vec!["ui-icon-size".into(), "18dp".into()],
        vec![
            "ui-vertical-label-text-alignment".into(),
            "ui-center-aligned".into(),
        ],
        vec![
            "ui-horizontal-label-text-alignment".into(),
            "ui-start-aligned".into(),
        ],
        vec!["ui-left-right-padding-without-icon".into(), "16dp".into()],
        vec!["ui-left-right-padding-with-icon".into(), "8dp".into()],
        vec!["ui-padding-between-elements".into(), "8dp".into()],
    ];
    if variant == ChipVariant::Input {
        rows.push(vec![
            "ui-avatar-shape".into(),
            "ui-12dp-corner-radius".into(),
        ]);
        rows.push(vec!["ui-avatar-size".into(), "24dp".into()]);
        rows.push(vec!["ui-left-padding-for-avatar".into(), "4dp".into()]);
        rows.push(vec!["ui-right-padding-for-avatar".into(), "8dp".into()]);
        rows.push(vec![
            "ui-target-size-for-close-icon".into(),
            "ui-min-48dp".into(),
        ]);
    }
    spec_table(vec!["ui-attribute".into(), "ui-value".into()], rows);
}

#[tessera]
fn examples(variant: ChipVariant, enabled: bool) {
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            for style in [ChipStyle::Flat, ChipStyle::Elevated] {
                sample(
                    variant,
                    style,
                    false,
                    enabled,
                    I18N.format(
                        "ui-value-value-format-2",
                        [
                            ("v0", I18N.text(&name(variant)).into()),
                            ("v1", format!("{:?}", style).into()),
                        ],
                    ),
                );
            }
            if selectable(variant) {
                sample(
                    variant,
                    ChipStyle::Flat,
                    true,
                    enabled,
                    I18N.format(
                        "ui-value-selected",
                        [("v0", I18N.text(&name(variant)).into())],
                    ),
                );
            }
        });
}

#[tessera]
fn sample(
    variant: ChipVariant,
    style: ChipStyle,
    initial: bool,
    enabled: bool,
    #[prop(into)] label: String,
) {
    let selected = remember(|| initial);
    let actions = remember(|| 0_u32);
    let toggles = selectable(variant);
    column().children(move || {
        let chip = chip()
            .variant(variant)
            .style(style)
            .shape_style(ChipShapeStyle::Standard)
            .label(I18N.text(&(label)))
            .accessibility_label(I18N.text(&(label)))
            .enabled(enabled)
            .selected(selected.get())
            .on_click(move || {
                actions.set(actions.get() + 1);
                if toggles {
                    selected.set(!selected.get());
                }
            });
        if variant != ChipVariant::Suggestion {
            chip.leading_icon(if selected.get() && toggles {
                filled::CHECK_SVG
            } else {
                filled::ADD_SVG
            });
        }
        if !toggles {
            gap(Dp(8.0));
            text(I18N.format("ui-actions-value", [("v0", (actions.get()).into())]))
                .style(type_style(12.0, 16.0));
        }
    });
}

#[tessera]
fn scheme(variant: ChipVariant, dark: bool) {
    scheme_panel(
        dark,
        RenderSlot::new(move || {
            examples(variant, true);
        }),
    );
}

#[tessera]
fn detail(variant: ChipVariant) {
    chip_diagram(variant, false);
    gap(Dp(16.0));
    let labels = match variant {
        ChipVariant::Assist => vec![
            (1_usize, "ui-container"),
            (2, "ui-label-text"),
            (3, "ui-leading-icon"),
        ],
        ChipVariant::Filter => vec![
            (1_usize, "ui-container"),
            (2, "ui-label-text"),
            (3, "ui-leading-icon"),
            (4, "ui-trailing-icon"),
        ],
        ChipVariant::Input => vec![
            (1_usize, "ui-container"),
            (2, "ui-label-text"),
            (3, "ui-trailing-icon"),
            (4, "ui-leading-icon"),
        ],
        ChipVariant::Suggestion => vec![(1_usize, "ui-container"), (2, "ui-label-text")],
    };
    column().children(move || {
        for (number, label) in labels.clone() {
            legend(number, label.to_owned());
        }
    });
    gap(Dp(32.0));
    subheading(I18N.format(
        "ui-value-chip-color",
        [("v0", I18N.text(&name(variant)).into())],
    ));
    gap(Dp(16.0));
    prose(I18N.format("ui-value-chip-color-roles-used-for-light-and-dark-themes-value", [("v0", I18N.text(&name(variant)).into()), ("v1", I18N.text(&(match variant {
            ChipVariant::Assist => "ui-surface-container-low-optional-on-surface-outline-primary",
            ChipVariant::Filter =>
                "ui-on-surface-variant-on-secondary-container-secondary-container-outline-variant-surface-cont",
            ChipVariant::Input =>
                "ui-on-surface-variant-surface-container-low-optional-outline-variant-primary-secondary-contai",
            ChipVariant::Suggestion =>
                "ui-outline-surface-container-low-optional-on-surface-variant",
        })).into())]));
    gap(Dp(24.0));
    scheme(variant, false);
    gap(Dp(24.0));
    scheme(variant, true);
    gap(Dp(48.0));
    subheading(I18N.format(
        "ui-value-chip-states",
        [("v0", I18N.text(&name(variant)).into())],
    ));
    gap(Dp(16.0));
    prose(
        "ui-enabled-disabled-hovered-focused-pressed-and-dragged-states-use-the-same-container-label-a",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            subheading("ui-enabled".into());
            gap(Dp(16.0));
            examples(variant, true);
            gap(Dp(24.0));
            subheading("ui-disabled".into());
            gap(Dp(16.0));
            examples(variant, false);
        });
    }));
    gap(Dp(48.0));
    subheading(I18N.format(
        "ui-value-chip-measurements",
        [("v0", I18N.text(&name(variant)).into())],
    ));
    gap(Dp(24.0));
    value_table(variant);
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-with-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-use-a-chip-to-perform-an-action".into(),
        "ui-navigate-to-a-chip".into(),
        "ui-activate-a-chip".into(),
    ]);
    gap(Dp(24.0));
    filter_example();
}

#[tessera]
fn interaction_style() {
    prose("ui-the-chip-label-needs-at-least-3-1-contrast-with-the-background");
    gap(Dp(16.0));
    prose(
        "ui-a-chip-that-performs-an-action-should-present-the-same-semantics-as-a-button-to-a-platform",
    );
    gap(Dp(24.0));
    prose(
        "ui-this-native-sample-exposes-assist-chips-as-buttons-filter-and-input-chips-as-toggleable-co",
    );
    gap(Dp(24.0));
    filter_example();
    gap(Dp(16.0));
    caption("ui-high-contrast-helps-differentiate-chips-clustered-together");
    gap(Dp(32.0));
    subheading("ui-horizontal-overflow".into());
    prose(
        "ui-when-there-are-too-many-chips-to-fit-on-one-row-provide-a-way-to-display-them-all-at-once",
    );
    gap(Dp(16.0));
    prose(
        "ui-reflow-method-use-a-filter-chip-as-a-leading-element-to-reflow-the-horizontal-list-this-sh",
    );
    gap(Dp(24.0));
    overflow_example();
    gap(Dp(16.0));
    caption(
        "ui-the-show-all-filter-chip-is-used-to-reflow-the-list-displaying-all-chips-at-once-and-pushi",
    );
    gap(Dp(32.0));
    prose(
        "ui-menu-method-create-a-leading-button-to-display-all-chip-options-in-a-menu-use-this-option",
    );
    gap(Dp(16.0));
    prose("ui-don-t-use-the-menu-method-on-chips-with-a-second-action-like-a-remove-icon");
    gap(Dp(32.0));
    subheading("ui-avoid-applying-density-by-default".into());
    prose(
        "ui-don-t-apply-density-to-chips-by-default-this-lowers-their-targets-below-our-best-practice",
    );
    gap(Dp(16.0));
    prose(
        "ui-to-ensure-that-this-density-setting-can-be-easily-reverted-when-it-s-active-keep-all-the-t",
    );
}

#[tessera]
fn overflow_example() {
    let all = remember(|| false);
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                chip()
                    .filter(I18N.text(if all.get() {
                        "ui-show-less"
                    } else {
                        "ui-show-all"
                    }))
                    .shape_style(ChipShapeStyle::Standard)
                    .selected(all.get())
                    .on_click(move || all.set(!all.get()));
                for index in 0..6 {
                    overflow_choice(index, all.get());
                }
            });
    }));
}

#[tessera]
fn overflow_choice(index: usize, all: bool) {
    let selected = remember(|| false);
    if all || index < 2 {
        chip()
            .filter(I18N.text(
                [
                    "ui-music",
                    "ui-movies",
                    "ui-books",
                    "ui-travel",
                    "ui-food",
                    "ui-sports",
                ][index],
            ))
            .shape_style(ChipShapeStyle::Standard)
            .selected(selected.get())
            .on_click(move || selected.set(!selected.get()));
    }
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab".into(),
                "ui-moves-focus-to-enabled-chip-or-chip-group".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activates-selects-or-deselects-the-focused-chip".into(),
            ],
            vec![
                "ui-backspace-or-delete".into(),
                "ui-removes-currently-focused-input-chip".into(),
            ],
            vec!["ui-arrows".into(), "ui-moves-focus-between-chips".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-this-native-sample-follows-the-table-tab-reaches-each-chip-space-or-enter-activates-the-fo",
    );
    gap(Dp(24.0));
    input_example();
}

#[tessera]
fn labeling() {
    prose(
        "ui-use-the-visible-chip-label-as-its-accessible-name-decorative-icons-are-not-repeated-additi",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-a11y-label".into(),
            "ui-role-web".into(),
            "ui-role-android-views".into(),
            "ui-role-jetpack-compose".into(),
        ],
        vec![
            vec![
                "ui-image-icon-within-chip".into(),
                "ui-hide-image".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            ],
            vec![
                "ui-basic-chip-one-action".into(),
                "ui-chip-content".into(),
                "gridcell".into(),
                "button".into(),
                "button".into(),
            ],
            vec![
                "ui-selectable-chip".into(),
                "ui-chip-content".into(),
                "gridcell".into(),
                "ui-radio-button-1835".into(),
                "checkbox".into(),
            ],
            vec![
                "ui-remove-icon-no-other-action".into(),
                "ui-remove-chip-content".into(),
                "-".into(),
                "-".into(),
                "-".into(),
            ],
            vec![
                "ui-two-actions-e-g-select-remove".into(),
                "ui-chip-content-then-remove-chip-content".into(),
                "ui-button-or-checkbox".into(),
                "ui-button-or-checkbox".into(),
                "ui-button-or-checkbox".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-multi-select-7064".into());
    prose("ui-space-or-enter-toggles-a-focused-selectable-chip-without-changing-other-choices");
    gap(Dp(24.0));
    filter_example();
    gap(Dp(32.0));
    subheading("ui-drop-down-list".into());
    prose(
        "ui-menu-item-labels-match-their-visible-text-decorative-icons-do-not-add-redundant-announceme",
    );
    gap(Dp(32.0));
    subheading("ui-input-chip-remove-action".into());
    prose(
        "ui-a-chip-with-selection-and-removal-exposes-two-independent-focusable-actions-name-the-secon",
    );
    gap(Dp(24.0));
    input_example();
    gap(Dp(32.0));
    subheading("ui-showing-chip-interactivity".into());
    prose(
        "ui-introduce-the-chip-group-with-a-meaningful-instruction-such-as-filter-results-or-use-an-ac",
    );
}
