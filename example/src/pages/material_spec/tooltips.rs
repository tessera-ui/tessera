//! Plain and rich tooltip specification examples.
//!
//! ## Usage
//!
//! Explain actions with contextual help and optional follow-up actions.

use std::collections::BTreeMap;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, TextStyle, material_theme},
    tooltip::{
        TooltipController, TooltipPlacement, TooltipVariant, tooltip_anchor, tooltip_provider,
    },
};
use tessera_ui::{
    AxisConstraint, ComputedData, Constraint, Dp, LayoutModifierChild, LayoutModifierInput,
    LayoutModifierNode, LayoutResult, MeasurementError, Modifier, Px, PxPosition, PxSize,
    RenderSlot, State,
    accesskit::Role,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    annotations::{self, Rect, SpaceGeometry, dotted, space_geometry, watch_in_space},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
};

#[tessera]
pub(super) fn tooltips_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-tooltips",
        "ui-tooltips-display-brief-labels-or-messages",
        vec![
            section("ui-tokens-specs", RenderSlot::new(tokens)),
            section("ui-plain-tooltip", RenderSlot::new(|| {
                detail(false);
            })),
            section("ui-rich-tooltip", RenderSlot::new(|| {
                detail(true);
            })),
        ],
    ).route_optional(route)
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(|| {
                tooltip_points(vec![
                    "ui-use-tooltips-to-add-additional-context-to-a-button-or-other-ui-element",
                    "ui-two-variants-plain-and-rich",
                    "ui-use-plain-tooltips-to-describe-elements-or-actions-of-icon-buttons",
                    "ui-use-rich-tooltips-to-provide-more-details-like-describing-the-value-of-a-feature",
                    "ui-rich-tooltips-can-include-an-optional-title-link-and-buttons",
                ]);
                gap(Dp(32.0));
                tooltip_variant_examples();
            })),
            sections: vec![
                section("ui-availability-resources", RenderSlot::new(tooltip_availability)),
                section("ui-differences-from-m2", RenderSlot::new(|| {
                    tooltip_points(vec![
                        "ui-color-new-color-mappings-and-compatibility-with-dynamic-color",
                        "ui-shape-rich-tooltips-have-more-rounded-corners",
                    ]);
                    gap(Dp(24.0));
                    tooltip_material_comparison();
                })),
            ],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(tooltip_guidelines_intro)),
            sections: vec![
                section("ui-usage", RenderSlot::new(tooltip_usage)),
                section("ui-anatomy", RenderSlot::new(tooltip_anatomy)),
                section("ui-placement", RenderSlot::new(tooltip_placement)),
                section("ui-behavior", RenderSlot::new(tooltip_behavior)),
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                section("ui-use-cases", RenderSlot::new(|| {
                    prose("ui-people-should-be-able-to-do-the-following-using-assistive-technology".into());
                    gap(Dp(16.0));
                    tooltip_points(vec![
                        "ui-receive-a-tooltip-message",
                        "ui-activate-a-tooltip-with-a-keyboard-or-switch-input",
                    ]);
                })),
                section("ui-interaction-style", RenderSlot::new(tooltip_accessibility_interaction)),
                section("ui-focus-order", RenderSlot::new(|| {
                    tooltip_split()
                        .lead(|| {
                            prose("ui-tooltip-containers-should-not-block-important-information-or-prevent-a-person-from-complet".into());
                            gap(Dp(16.0));
                            prose("ui-focus-order-within-the-rich-tooltip-moves-top-to-bottom-between-interactive-elements".into());
                            gap(Dp(16.0));
                            prose("ui-avoid-trapping-screen-reader-and-keyboard-focus-on-rich-tooltips".into());
                            gap(Dp(16.0));
                            prose("ui-people-should-be-able-to-move-linearly-through-the-rest-of-the-page".into());
                        })
                        .figure(tooltip_focus_sequence);
                })),
                section("ui-keyboard-navigation", RenderSlot::new(tooltip_keyboard_table)),
                section("ui-labeling-elements", RenderSlot::new(|| {
                    tooltip_split()
                        .lead(|| {
                            prose("ui-tooltips-should-have-the-tooltip-role-or-similar".into());
                            gap(Dp(16.0));
                            prose("ui-label-all-elements-in-the-tooltip-according-to-their-own-accessibility-guidance".into());
                        })
                        .figure(|| {
                            tooltip_labeling_diagram();
                            gap(Dp(16.0));
                            tooltip_caption("ui-the-tooltip-container-should-have-the-tooltip-role".into());
                        });
                })),
            ],
        },
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn prose(content: String) {
    text(I18N.text(content)).style(type_style(16.0, 24.0));
}

#[tessera]
fn tooltip_points(items: Vec<&'static str>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in &items {
                prose(format!("•  {item}"));
                gap(Dp(12.0));
            }
        });
}

#[tessera]
fn tooltip_availability() {
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
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
}

#[tessera]
fn tooltip_split(lead: Option<RenderSlot>, figure: Option<RenderSlot>) {
    let lead = lead.unwrap_or_else(RenderSlot::empty);
    let figure = figure.unwrap_or_else(RenderSlot::empty);
    let wide = remember(|| true);
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width).0 >= 720.0;
                if wide.is_alive() && wide.get() != next {
                    wide.set(next);
                }
            },
        ))
        .children(move || {
            if wide.get() {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || lead.render());
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        column()
                            .modifier(Modifier::new().weight(2.0))
                            .children(move || figure.render());
                    });
            } else {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        lead.render();
                        gap(Dp(24.0));
                        figure.render();
                    });
            }
        });
}

#[tessera]
fn tooltip_term(title: String, body: String) {
    text(I18N.text(title)).style(emphasis_style(16.0, 24.0));
    prose(body);
}

fn emphasis_style(size: f64, line: f64) -> TextStyle {
    // The bundled text face is a static 400 instance, so emphasis uses the
    // matching static 500 face instead of relying on weight synthesis.
    let mut style = type_style(size, line);
    style.font_family = Some("Example Sans Tabs");
    style.font_weight = 500;
    style
}

#[tessera]
fn tooltip_caption(content: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO))
        .children(move || {
            text(I18N.text(&(content)))
                .style(type_style(16.0, 24.0))
                .color(scheme.on_surface_variant);
        });
}

fn tooltip_advice_label(tone: usize) -> &'static str {
    match tone {
        0 => "Do",
        1 => "ui-caution",
        _ => "Don't",
    }
}

#[tessera]
fn tooltip_guidance(
    tone: usize,
    caption: String,
    height: Option<f64>,
    neutral: Option<bool>,
    figure: Option<RenderSlot>,
) {
    let figure = figure.unwrap_or_else(RenderSlot::empty);
    let plate = if neutral.unwrap_or(false) {
        srgb(247, 242, 250)
    } else {
        srgb(227, 223, 250)
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let caption = caption.clone();
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .children(move || {
                    let modifier = Modifier::new().fill_max_width().background(plate);
                    let modifier = match height {
                        Some(value) => modifier.height(Dp(value)),
                        None => modifier.padding_all(Dp(24.0)),
                    };
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(modifier)
                        .children(move || figure.render());
                    if tone <= 2 {
                        tooltip_advice_bar(tooltip_advice_label(tone).into(), tone);
                    }
                });
            gap(Dp(16.0));
            text(I18N.text(&(caption)))
                .style(type_style(16.0, 24.0))
                .color(scheme.on_surface_variant)
                .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
        });
}

fn tooltip_guidance_text(kind: usize) -> (usize, &'static str) {
    match kind {
        1 => (0, "ui-use-plain-tooltips-to-label-icon-only-buttons"),
        2 => (
            2,
            "ui-plain-tooltips-aren-t-needed-when-the-ui-element-already-has-label-text",
        ),
        3 => (
            0,
            "ui-use-rich-tooltips-to-provide-extra-information-and-actions-about-a-ui-element-or-new-featu",
        ),
        4 => (
            2,
            "ui-don-t-hide-critical-information-within-tooltips-as-it-s-easy-to-miss-use-an-interruptive-d",
        ),
        5 => (0, "ui-briefly-describe-a-ui-element"),
        6 => (
            1,
            "ui-avoid-wrapping-text-to-multiple-lines-or-including-many-pieces-of-information",
        ),
        7 => (0, "ui-summarize-the-message-in-a-few-words"),
        8 => (2, "ui-avoid-wrapping-to-more-than-one-line"),
        9 => (1, "ui-avoid-stacking-buttons"),
        10 => (2, "ui-don-t-use-a-persistent-rich-tooltip-on-icon-buttons"),
        11 => (
            3,
            "ui-tooltips-can-appear-on-hover-or-focus-to-explain-actions",
        ),
        _ => (3, "ui-rich-tooltips-can-appear-when-an-element-is-selected"),
    }
}

#[tessera]
fn tooltip_guidance_card(kind: usize) {
    let (tone, caption) = tooltip_guidance_text(kind);
    let height = match kind {
        1..=4 => 260.0,
        11 | 12 => 200.0,
        _ => 220.0,
    };
    tooltip_guidance(tone, caption.into())
        .height(height)
        .neutral(kind == 9)
        .figure(move || {
            tooltip_mock(kind);
        });
}

#[tessera]
fn tooltip_guidance_pair(first: usize, second: usize) {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    tooltip_guidance_card(first);
                });
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    tooltip_guidance_card(second);
                });
        });
}

#[tessera]
fn tooltip_guidelines_intro() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            for kind in [13, 14] {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .weight(1.0)
                            .height(Dp(280.0))
                            .background_with_shape(
                                srgb(227, 223, 250),
                                Shape::rounded_rectangle(Dp(24.0)),
                            ),
                    )
                    .children(move || {
                        tooltip_mock(kind);
                    });
                if kind == 13 {
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                }
            }
        });
    gap(Dp(16.0));
    tooltip_caption("ui-plain-and-rich-tooltips-serve-different-purposes".into());
}

#[tessera]
fn tooltip_usage() {
    tooltip_split()
        .lead(|| {
            prose("ui-a-tooltip-provides-additional-context-for-a-ui-element".into());
            gap(Dp(16.0));
            tooltip_term(
                "ui-plain-tooltips".into(),
                "ui-plain-tooltips-briefly-describe-a-ui-element-they-re-best-used-for-labelling-ui-elements-w".into(),
            );
            gap(Dp(16.0));
            tooltip_term(
                "ui-rich-tooltips".into(),
                "ui-rich-tooltips-provide-additional-context-about-a-ui-element-they-can-optionally-contain-a".into(),
            );
            gap(Dp(16.0));
            prose("ui-rich-tooltips-are-best-used-for-longer-text-like-definitions-or-explanations".into());
        })
        .figure(|| {
            tooltip_variant_examples().guidelines(true);
        });
    gap(Dp(48.0));
    tooltip_guidance_pair(1, 2);
    gap(Dp(48.0));
    tooltip_guidance_pair(3, 4);
}

#[tessera]
fn tooltip_anatomy() {
    subheading("ui-plain-tooltip".into());
    gap(Dp(24.0));
    tooltip_figure(false, false).guidelines(true);
    gap(Dp(48.0));
    subheading("ui-supporting-text".into());
    gap(Dp(24.0));
    tooltip_writing_examples(false);
    gap(Dp(48.0));
    subheading("ui-rich-tooltip".into());
    gap(Dp(24.0));
    tooltip_figure(true, false).guidelines(true);
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            subheading("ui-subhead-optional".into());
            gap(Dp(16.0));
            prose("ui-keep-subheads-brief-ideally-to-one-line-they-should-summarize-or-describe-the-message-of-t".into());
            gap(Dp(16.0));
            prose("ui-subheads-are-important-to-include-when-the-rich-tooltip-appears-automatically-like-when-th".into());
        })
        .figure(|| {
            tooltip_writing_examples(true);
        });
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            subheading("ui-text-buttons-optional".into());
            gap(Dp(16.0));
            prose("ui-rich-tooltips-can-have-up-to-two-text-buttons-these-should-be-brief-and-relevant-to-the-me".into());
            gap(Dp(16.0));
            prose("ui-keep-buttons-short-so-they-can-be-side-by-side-avoid-stacking-them-when-possible".into());
        })
        .figure(|| {
            tooltip_guidance_card(9);
        });
}

#[tessera]
fn tooltip_placement() {
    tooltip_split()
        .lead(|| {
            subheading("ui-plain-tooltips".into());
            gap(Dp(16.0));
            prose("ui-by-default-plain-tooltips-are-positioned-directly-above-the-parent-element".into());
            gap(Dp(16.0));
            tooltip_points(vec![
                "ui-if-there-s-a-visual-boundary-like-a-button-the-distance-is-4dp",
                "ui-if-there-s-no-visual-boundary-like-with-text-baselines-the-distance-is-8dp",
            ]);
            gap(Dp(4.0));
            prose("ui-if-the-element-is-in-an-app-bar-the-plain-tooltip-appears-below-the-element-at-the-same-di".into());
        })
        .figure(|| {
            tooltip_plain_placement();
            gap(Dp(16.0));
            tooltip_caption("ui-plain-tooltip-with-a-4dp-distance-between-the-target-and-tooltip".into());
        });
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            subheading("ui-rich-tooltips".into());
            gap(Dp(16.0));
            prose("ui-by-default-rich-tooltips-are-positioned-to-the-bottom-right-of-the-parent-element-they-adj".into());
            gap(Dp(16.0));
            tooltip_term(
                "ui-dynamic-positioning".into(),
                "ui-the-position-of-the-tooltip-adjusts-in-increments-of-8dp-to-avoid-going-off-screen".into(),
            );
            gap(Dp(16.0));
            tooltip_term(
                "ui-desktop-placement".into(),
                "ui-on-desktop-tooltips-may-appear-centered-below-the-parent-element-and-remain-visible-while".into(),
            );
        })
        .figure(|| {
            placement_examples();
            gap(Dp(16.0));
            tooltip_caption("ui-four-different-rich-tooltip-locations-based-on-dynamic-positioning".into());
        });
}

#[tessera]
fn tooltip_behavior() {
    prose("ui-to-show-a-tooltip-hover-on-the-parent-element-on-desktop-or-tap-and-hold-the-element-on-mo".into());
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            subheading("ui-transient-by-default".into());
            gap(Dp(16.0));
            prose("ui-both-plain-and-rich-tooltips-disappear-1-5-seconds-after-navigating-away-from-the-target-r".into());
            gap(Dp(16.0));
            prose("ui-triggering-a-new-tooltip-immediately-closes-any-other-open-tooltip".into());
        })
        .figure(|| {
            live_tooltips();
            gap(Dp(16.0));
            tooltip_caption("ui-tooltips-disappear-after-a-1-5-second-delay-when-no-other-element-is-hovered".into());
        });
    gap(Dp(48.0));
    tooltip_one_at_a_time();
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            subheading("ui-persistent-rich-tooltips".into());
            gap(Dp(16.0));
            prose("ui-persistent-rich-tooltips-appear-when-either".into());
            gap(Dp(16.0));
            tooltip_points(vec![
                "ui-the-parent-element-is-clicked",
                "ui-the-page-loads-and-a-new-feature-is-being-explained",
            ]);
            gap(Dp(4.0));
            prose("ui-persistent-rich-tooltips-remain-active-even-when-leaving-the-target-region-they-only-disap".into());
            gap(Dp(16.0));
            prose("ui-when-appearing-on-page-load-the-tooltip-can-introduce-and-explain-new-features-on-various".into());
            gap(Dp(16.0));
            prose("ui-avoid-using-persistent-rich-tooltips-on-icon-buttons".into());
        })
        .figure(|| {
            tooltip_guidance_card(10);
        });
}

#[tessera]
fn tooltip_accessibility_interaction() {
    tooltip_split()
        .lead(|| {
            prose("ui-plain-and-rich-tooltips-without-required-actions-should-remain-on-screen-long-enough-for-p".into());
        })
        .figure(|| {
            tooltip_guidance(
                0,
                "ui-plain-tooltips-should-remain-on-the-screen-temporarily-after-the-cursor-moves-away"
                    .into(),
            )
            .figure(|| {
                tooltip_one_at_a_time_stage().more(false);
            });
        });
    gap(Dp(48.0));
    tooltip_split()
        .lead(|| {
            prose("ui-tooltips-can-appear-when-an-actionable-element-like-a-button-or-navigation-rail-is-hovered".into());
            gap(Dp(16.0));
            prose("ui-rich-tooltips-can-also-appear-by-selecting-an-element-instead-of-hovering-or-focusing-on-i".into());
        })
        .figure(|| {
            tooltip_guidance_pair(11, 12);
        });
}

#[tessera]
fn tooltip_plain_mock(label: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .style(scheme.inverse_surface.into())
        .content_color(scheme.inverse_on_surface)
        .shape(Shape::rounded_rectangle(Dp(4.0)))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Tooltip),
            label: Some(I18N.text(&(label))),
            merge_descendants: true,
            ..Default::default()
        }))
        .child(move || {
            let label = label.clone();
            boxed()
                .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp(4.0)))
                .children(move || {
                    text(I18N.text(&(label))).style(type_style(12.0, 16.0));
                });
        });
}

#[tessera]
fn tooltip_rich_mock(
    headline: String,
    message: String,
    actions: Option<Vec<String>>,
    width: Option<f64>,
    stacked: Option<bool>,
    geometry: Option<State<SpaceGeometry>>,
) {
    let actions = actions.unwrap_or_default();
    let width = width.unwrap_or(240.0);
    let stacked = stacked.unwrap_or(false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .style(scheme.surface_container.into())
        .content_color(scheme.on_surface_variant)
        .shape(Shape::rounded_rectangle(Dp(12.0)))
        .elevation(Dp(3.0))
        .modifier(Modifier::new().width(Dp(width)))
        .child(move || {
            let headline = headline.clone();
            let message = message.clone();
            let actions = actions.clone();
            column()
                .modifier(
                    Modifier::new().fill_max_width().padding_symmetric(
                        Dp::ZERO,
                        Dp(if actions.is_empty() { 12.0 } else { 8.0 }),
                    ),
                )
                .children(move || {
                    let headline = headline.clone();
                    let message = message.clone();
                    let actions = actions.clone();
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_symmetric(
                            Dp(16.0),
                            Dp(if actions.is_empty() { 0.0 } else { 4.0 }),
                        ))
                        .children(move || {
                            if !headline.is_empty() {
                                text(I18N.text(&(headline))).style(emphasis_style(14.0, 20.0));
                                gap(Dp(4.0));
                            }
                            text(I18N.text(&(message))).style(type_style(14.0, 20.0));
                        });
                    if !actions.is_empty() {
                        gap(Dp(4.0));
                        let content = RenderSlot::new(move || {
                            let actions = actions.clone();
                            if stacked {
                                column()
                                    .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO))
                                    .children(move || {
                                        for (index, label) in actions.iter().enumerate() {
                                            match geometry {
                                                Some(geometry) if index == 0 => {
                                                    tooltip_rich_mock_action(label.clone())
                                                        .geometry(geometry);
                                                }
                                                _ => {
                                                    tooltip_rich_mock_action(label.clone());
                                                }
                                            }
                                        }
                                    });
                            } else {
                                row()
                                    .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO))
                                    .children(move || {
                                        for (index, label) in actions.iter().enumerate() {
                                            match geometry {
                                                Some(geometry) if index == 0 => {
                                                    tooltip_rich_mock_action(label.clone())
                                                        .geometry(geometry);
                                                }
                                                _ => {
                                                    tooltip_rich_mock_action(label.clone());
                                                }
                                            }
                                        }
                                    });
                            }
                        });
                        content.render();
                    }
                });
        });
}

#[tessera]
fn tooltip_rich_mock_action(label: String, geometry: Option<State<SpaceGeometry>>) {
    let modifier = match geometry {
        Some(geometry) => watch_in_space(geometry, "labeling_action"),
        None => Modifier::new(),
    };
    button()
        .text()
        .modifier(modifier)
        .accessibility_label(I18N.text(&(label)))
        .child(move || {
            text(I18N.text(&(label)));
        });
}

#[tessera]
fn tooltip_mock_icon(icon: usize, tone: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let symbol = match icon {
        0 => filled::ADD_SVG,
        1 => filled::SETTINGS_SVG,
        2 => filled::MIC_SVG,
        3 => filled::VIDEOCAM_SVG,
        4 => filled::MOOD_SVG,
        5 => filled::PRESENT_TO_ALL_SVG,
        6 => filled::MORE_VERT_SVG,
        7 => filled::PERSON_ADD_SVG,
        8 => filled::STAR_BORDER_SVG,
        9 => filled::SHOPPING_CART_SVG,
        10 => filled::MORE_HORIZ_SVG,
        11 => filled::AUTO_DELETE_SVG,
        12 => filled::NOTIFICATIONS_NONE_SVG,
        13 => filled::PERSON_SVG,
        14 => filled::INFO_SVG,
        _ => filled::STAR_SVG,
    };
    let (background, tint, size) = match tone {
        1 => (
            Some(scheme.surface_container_highest),
            scheme.on_surface_variant,
            40.0,
        ),
        2 => (Some(scheme.primary), scheme.on_primary, 40.0),
        3 => (
            Some(scheme.primary_container),
            scheme.on_primary_container,
            56.0,
        ),
        _ => (None, scheme.on_surface_variant, 40.0),
    };
    let modifier = Modifier::new().size(Dp(size), Dp(size));
    let modifier = match background {
        Some(color) => modifier.background_with_shape(
            color,
            if tone == 3 {
                Shape::rounded_rectangle(Dp(16.0))
            } else {
                Shape::Ellipse
            },
        ),
        None => modifier,
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            tessera_components::icon::icon()
                .painter(symbol)
                .size(Dp(24.0))
                .tint(tint);
        });
}

#[tessera]
fn tooltip_mock(kind: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    match kind {
        1 => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    tooltip_plain_mock("ui-present-now".into());
                    gap(Dp(4.0));
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            for (icon, tone) in [(2, 1), (3, 1), (4, 1), (5, 1), (6, 1)] {
                                tooltip_mock_icon(icon, tone);
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                            }
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .size(Dp(72.0), Dp(40.0))
                                        .background_with_shape(scheme.primary, Shape::CAPSULE),
                                )
                                .children(move || {
                                    tessera_components::icon::icon()
                                        .painter(filled::CALL_END_SVG)
                                        .size(Dp(24.0))
                                        .tint(scheme.on_primary);
                                });
                        });
                });
        }
        2 => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    tooltip_plain_mock("ui-edit".into());
                    gap(Dp(4.0));
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .background_with_shape(
                                    scheme.surface_container_highest,
                                    Shape::rounded_rectangle(Dp(16.0)),
                                )
                                .padding_symmetric(Dp(16.0), Dp(16.0)),
                        )
                        .children(move || {
                            tessera_components::icon::icon()
                                .painter(filled::EDIT_SVG)
                                .size(Dp(24.0))
                                .tint(scheme.primary);
                            spacer().modifier(Modifier::new().width(Dp(8.0)));
                            text(I18N.text("ui-edit"))
                                .style(type_style(14.0, 20.0))
                                .color(scheme.primary);
                        });
                });
        }
        3 | 4 => {
            row().children(move || {
                column()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        for (index, icon) in [7, if kind == 3 { 8 } else { 11 }, 9, 10]
                            .into_iter()
                            .enumerate()
                        {
                            tooltip_mock_icon(
                                icon,
                                if (kind == 3 && index == 0) || (kind == 4 && index == 1) {
                                    1
                                } else {
                                    0
                                },
                            );
                            gap(Dp(4.0));
                        }
                    });
                spacer().modifier(Modifier::new().width(Dp(8.0)));
                column().children(move || {
                    gap(Dp(if kind == 3 { 32.0 } else { 76.0 }));
                    if kind == 3 {
                        tooltip_rich_mock(
                            "ui-add-others".into(),
                            "ui-share-this-collection-with-friends-and-family-people-you-add-to-this-album-will-also-be-ab".into(),
                        )
                        .actions(vec!["ui-learn-more".into()])
                        .width(220.0);
                    } else {
                        tooltip_rich_mock(
                            "ui-auto-delete-all".into(),
                            "ui-immediately-deletes-all-photos-in-this-collection-you-can-t-undo-this-action".into(),
                        )
                        .width(220.0);
                    }
                });
            });
        }
        5 | 6 => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::End)
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            tooltip_mock_icon(1, 0);
                            tooltip_mock_icon(12, if kind == 5 { 1 } else { 0 });
                            tooltip_mock_icon(13, if kind == 6 { 2 } else { 0 });
                        });
                    gap(Dp(4.0));
                    tooltip_plain_mock(
                        if kind == 5 {
                            "ui-notifications"
                        } else {
                            "ui-lee-villanueva-lortega-gmail-com"
                        }
                        .into(),
                    );
                });
        }
        7 | 8 => {
            tooltip_rich_mock(
                if kind == 7 {
                    "ui-new-settings-available"
                } else {
                    "ui-there-are-new-settings-available-for-image-quality"
                }
                .into(),
                "ui-now-you-can-adjust-the-uploaded-image-quality-and-upgrade-your-available-storage-space".into(),
            )
            .actions(vec!["ui-learn-more".into()])
            .width(240.0);
        }
        9 => {
            tooltip_rich_mock(
                "ui-new-settings-available".into(),
                "ui-now-you-can-adjust-the-uploaded-image-quality-and-upgrade-your-available-storage-space".into(),
            )
            .actions(vec!["ui-learn-more".into(), "ui-go-to-settings".into()])
            .width(240.0)
            .stacked(true);
        }
        10 => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            tooltip_mock_icon(7, 0);
                            tooltip_mock_icon(8, 1);
                            tooltip_mock_icon(6, 0);
                        });
                    gap(Dp(4.0));
                    tooltip_rich_mock(
                        "ui-share-with-friends".into(),
                        "ui-share-some-of-your-best-memories-with-friends".into(),
                    )
                    .actions(vec!["ui-learn-more".into()])
                    .width(200.0);
                });
        }
        11 | 12 => {
            row().children(move || {
                tooltip_mock_icon(
                    if kind == 11 { 8 } else { 14 },
                    if kind == 11 { 1 } else { 2 },
                );
                spacer().modifier(Modifier::new().width(Dp(8.0)));
                column().children(move || {
                    gap(Dp(32.0));
                    tooltip_rich_mock(
                        "ui-favorite".into(),
                        "ui-find-this-later-in-your-favorites".into(),
                    )
                    .width(160.0);
                });
            });
        }
        13 => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    tooltip_plain_mock("ui-upload".into());
                    gap(Dp(4.0));
                    tooltip_mock_icon(0, 3);
                });
        }
        _ => {
            column()
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(move || {
                    tooltip_rich_mock(
                        "ui-new-settings-available".into(),
                        "ui-now-you-can-adjust-the-uploaded-image-quality-and-upgrade-your-available-storage-space".into(),
                    )
                    .actions(vec!["ui-learn-more".into()])
                    .width(240.0);
                    gap(Dp(8.0));
                    tooltip_mock_icon(1, 1);
                });
        }
    }
}

#[tessera]
fn tooltip_variant_examples(guidelines: Option<bool>) {
    // The Guidelines usage figure is a narrower crop on a neutral plate with
    // its own reference copy; the Overview figure keeps the tinted hero plate.
    let guidelines = guidelines.unwrap_or(false);
    let (width, height) = if guidelines {
        (672.0, 348.0)
    } else {
        (1024.0, 385.0)
    };
    material_theme()
        .theme(|| tooltip_reference_theme(false))
        .child(move || {
            let geometry = remember(SpaceGeometry::new);
            tooltip_stage(
                width,
                height,
                "ui-plain-and-rich-tooltip-variants".into(),
                geometry,
            )
            .child(move || {
                boxed().modifier(
                    Modifier::new()
                        .required_size(Dp(width), Dp(height))
                        .background_with_shape(
                            if guidelines {
                                srgb(247, 242, 250)
                            } else {
                                srgb(227, 223, 250)
                            },
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                );
                let (plain_x, rich_x, center_y, badge_y) = if guidelines {
                    (143.0, 476.0, 150.0, 282.0)
                } else {
                    (280.0, 718.0, 166.0, 321.0)
                };
                tooltip_reference_popup(
                    geometry,
                    false,
                    false,
                    if guidelines {
                        "ui-plain-tooltip-6538".into()
                    } else {
                        "ui-plain-tooltip".into()
                    },
                    plain_x,
                    center_y,
                    width,
                    height,
                );
                tooltip_reference_popup(
                    geometry,
                    true,
                    true,
                    if guidelines {
                        RICH_USAGE_MESSAGE.into()
                    } else {
                        RICH_OVERVIEW_MESSAGE.into()
                    },
                    rich_x,
                    center_y,
                    width,
                    height,
                );
                let badges = if guidelines {
                    [(1, 143.0), (2, 476.0)]
                } else {
                    [(1, 278.0), (2, 712.0)]
                };
                for (number, x) in badges {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .offset(Dp(x - 20.0), Dp(badge_y - 20.0))
                                .size(Dp(40.0), Dp(40.0))
                                .background_with_shape(srgb(50, 47, 53), Shape::Ellipse),
                        )
                        .children(move || {
                            text((number).to_string())
                                .style(type_style(16.0, 24.0))
                                .color(srgb(255, 255, 255));
                        });
                }
            });
        });
    gap(Dp(16.0));
    tooltip_legend(vec!["ui-plain-tooltip".into(), "ui-rich-tooltip".into()]);
}

#[tessera]
fn tooltip_legend(labels: Vec<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    for (index, label) in labels.into_iter().enumerate() {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(24.0), Dp(32.0))
                            .background_with_shape(scheme.on_surface, Shape::CAPSULE),
                    )
                    .children(move || {
                        text((index + 1).to_string())
                            .style(type_style(14.0, 20.0))
                            .color(scheme.surface);
                    });
                text(I18N.text(&(label)))
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(4.0)));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn tooltip_material_comparison() {
    material_theme()
        .theme(|| tooltip_reference_theme(false))
        .child(move || {
            let geometry = remember(SpaceGeometry::new);
            tooltip_stage(
                1024.0,
                302.0,
                "ui-material-2-and-material-3-tooltip-comparison".into(),
                geometry,
            )
            .child(move || {
                boxed()
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(500.0), Dp(302.0))
                            .background_with_shape(
                                srgb(255, 255, 255),
                                Shape::rounded_rectangle(Dp(24.0)),
                            ),
                    )
                    .children(move || {
                        surface()
                            .modifier(Modifier::new().offset(Dp(94.0), Dp(64.0)).width(Dp(312.0)))
                            .style(srgb(255, 255, 255).into())
                            .shape(Shape::rounded_rectangle(Dp(8.0)))
                            .elevation(Dp(3.0))
                            .child(move || {
                                column()
                                    .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp(12.0)))
                                    .children(move || {
                                        text(I18N.text("ui-rich-tooltip"))
                                            .style(type_style(14.0, 20.0))
                                            .color(srgb(60, 64, 67));
                                        gap(Dp(4.0));
                                        text(I18N.text(RICH_OVERVIEW_MESSAGE))
                                            .style(type_style(14.0, 20.0))
                                            .color(srgb(60, 64, 67));
                                        gap(Dp(20.0));
                                        row().children(move || {
                                            for _ in 0..2 {
                                                text(I18N.text("ui-action"))
                                                    .style(type_style(14.0, 20.0))
                                                    .color(srgb(26, 115, 232))
                                                    .modifier(
                                                        Modifier::new()
                                                            .padding_symmetric(Dp(8.0), Dp(8.0)),
                                                    );
                                            }
                                        });
                                    });
                            });
                    });
                boxed().modifier(
                    Modifier::new()
                        .offset(Dp(524.0), Dp::ZERO)
                        .required_size(Dp(500.0), Dp(302.0))
                        .background_with_shape(
                            srgb(247, 242, 250),
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                );
                tooltip_reference_popup(
                    geometry,
                    false,
                    true,
                    RICH_OVERVIEW_MESSAGE.into(),
                    774.0,
                    151.0,
                    1024.0,
                    302.0,
                );
            });
        });
    gap(Dp(16.0));
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            for (index, caption) in [
                "ui-m2-rich-tooltips-have-slightly-rounded-corners",
                "ui-m3-rich-tooltips-have-more-rounded-corners-and-support-dynamic-color",
            ]
            .into_iter()
            .enumerate()
            {
                if index == 1 {
                    spacer().modifier(Modifier::new().width(Dp(24.0)));
                }
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .children(move || {
                        tooltip_caption(caption.into());
                    });
            }
        });
}

#[tessera]
fn tooltip_advice_bar(label: String, tone: usize) {
    let (background, ink, symbol) = match tone {
        0 => (srgb(52, 191, 73), srgb(29, 27, 32), filled::CHECK_SVG),
        1 => (
            srgb(255, 208, 41),
            srgb(29, 27, 32),
            filled::PRIORITY_HIGH_SVG,
        ),
        _ => (srgb(255, 99, 64), srgb(29, 27, 32), filled::CLOSE_SVG),
    };
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(background, Shape::rounded_rectangle(Dp::ZERO))
                .padding_symmetric(Dp(16.0), Dp(8.0)),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            tessera_components::icon::icon()
                .painter(symbol)
                .size(Dp(20.0))
                .tint(ink);
            text(I18N.text(&(label)))
                .style(type_style(16.0, 24.0))
                .color(ink)
                .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
        });
}

#[tessera]
fn tooltip_plain_placement() {
    material_theme()
        .theme(|| tooltip_reference_theme(false))
        .child(move || {
            let geometry = remember(SpaceGeometry::new);
            let controller = remember(|| {
                let mut value = TooltipController::new();
                value.show();
                value
            });
            tooltip_stage(
                1024.0,
                300.0,
                "ui-plain-tooltip-target-placement".into(),
                geometry,
            )
            .child(move || {
                boxed().modifier(
                    Modifier::new()
                        .required_size(Dp(1024.0), Dp(300.0))
                        .background_with_shape(
                            srgb(247, 242, 250),
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                );
                tooltip_provider("ui-add-to-list")
                    .controller(controller)
                    .placement(TooltipPlacement::Above)
                    .popup_modifier(watch_in_space(geometry, "placement_popup"))
                    .child(move |_| {
                        boxed()
                            .modifier(Modifier::new().required_size(Dp(1024.0), Dp(300.0)))
                            .children(move || {
                                tooltip_anchor(controller)
                                    .modifier(Modifier::new().offset(Dp(488.0), Dp(150.0)))
                                    .child(move || {
                                        icon_button()
                                            .standard()
                                            .icon(filled::PLAYLIST_ADD_SVG)
                                            .accessibility_label(I18N.text("ui-add-to-list"))
                                            .modifier(watch_in_space(geometry, "placement_target"));
                                    });
                            });
                    });
                tooltip_plain_placement_annotations(geometry);
            });
        });
}

#[tessera]
fn tooltip_plain_placement_annotations(geometry: State<SpaceGeometry>) {
    let values = geometry.with(space_geometry);
    let (Some(popup), Some(target)) = (
        values.get("placement_popup"),
        values.get("placement_target"),
    ) else {
        return;
    };
    let gap = target.y - popup.y - popup.h;
    let x = popup.x + popup.w + 16.0;
    let y = popup.y + popup.h;
    let svg = format!(
        "<path d='M{},{}h8M{x},{y}v{gap}M{},{}h8' stroke='#e25b86' stroke-width='1' fill='none'/>",
        x - 4.0,
        y,
        x - 4.0,
        y + gap
    );
    annotations::lines(Dp(1024.0), Dp(300.0), svg);
    annotations::dimension(format!("{gap:.0}"), Dp(x + 8.0), Dp(y + gap * 0.5 - 10.0));
}

#[derive(Clone, Copy, PartialEq)]
struct TooltipTableColumnRule(State<PxSize>);

impl LayoutModifierNode for TooltipTableColumnRule {
    fn measure_content(
        &self,
        _input: &LayoutModifierInput<'_>,
        child: &mut dyn LayoutModifierChild,
    ) -> Result<LayoutResult, MeasurementError> {
        let size = self.0.get();
        child.measure(&Constraint::new(
            AxisConstraint::exact(Px::from(Dp(1.0))),
            AxisConstraint::exact(size.height),
        ))?;
        child.place(PxPosition::from_f32_xy(
            (size.width.to_f32() * 0.355).floor(),
            0.0,
        ));
        Ok(LayoutResult::new(ComputedData {
            width: size.width,
            height: size.height,
        }))
    }
}

#[tessera]
fn tooltip_keyboard_table() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let table_size = remember(|| PxSize::new(Px::ZERO, Px::ZERO));
    boxed()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            surface()
                .style(tessera_components::surface::SurfaceStyle::FilledOutlined {
                    fill_color: scheme.surface,
                    border_color: scheme.outline_variant,
                    border_width: Dp(1.0),
                })
                .shape(Shape::rounded_rectangle(Dp(24.0)))
                .clip_content(true)
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .on_layout_coordinates(move |coordinates: tessera_ui::LayoutCoordinates| {
                            if table_size.is_alive() && table_size.get() != coordinates.local_size {
                                table_size.set(coordinates.local_size);
                            }
                        })
                        .semantics(SemanticsArgs {
                            role: Some(Role::Table),
                            label: Some(I18N.text("ui-tooltip-keyboard-navigation")),
                            merge_descendants: true,
                            collection_info: Some((3, 2, false)),
                            ..Default::default()
                        }),
                )
                .child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            for (index, key, action) in [
                                (0, "ui-keys", "ui-actions"),
                                (1, "ui-tab", "ui-focus-lands-on-button-if-available"),
                                (2, "ui-space-or-enter", "ui-activates-the-focused-element"),
                            ] {
                                if index > 0 {
                                    spacer().modifier(
                                        Modifier::new()
                                            .fill_max_width()
                                            .height(Dp(1.0))
                                            .background_with_shape(
                                                scheme.outline_variant,
                                                Shape::rounded_rectangle(Dp::ZERO),
                                            ),
                                    );
                                }
                                row()
                                    .modifier(
                                        Modifier::new()
                                            .fill_max_width()
                                            .background_with_shape(
                                                if index == 0 {
                                                    srgb(248, 241, 246)
                                                } else {
                                                    scheme.surface
                                                },
                                                Shape::rounded_rectangle(Dp::ZERO),
                                            )
                                            .semantics(SemanticsArgs {
                                                role: Some(Role::Row),
                                                merge_descendants: true,
                                                collection_item_info: Some((index, 1, 0, 2, false)),
                                                ..Default::default()
                                            }),
                                    )
                                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                                    .children(move || {
                                        for (column_index, content, weight) in
                                            [(0, key, 0.355), (1, action, 0.645)]
                                        {
                                            boxed()
                                                .alignment(if index == 0 {
                                                    Alignment::Center
                                                } else {
                                                    Alignment::CenterStart
                                                })
                                                .modifier(
                                                    Modifier::new()
                                                        .weight(weight)
                                                        .padding_all(Dp(16.0))
                                                        .semantics(SemanticsArgs {
                                                            role: Some(if index == 0 {
                                                                Role::ColumnHeader
                                                            } else if column_index == 0 {
                                                                Role::RowHeader
                                                            } else {
                                                                Role::Cell
                                                            }),
                                                            collection_item_info: Some((
                                                                index,
                                                                1,
                                                                column_index,
                                                                1,
                                                                false,
                                                            )),
                                                            label: Some(I18N.text(content)),
                                                            merge_descendants: false,
                                                            ..Default::default()
                                                        }),
                                                )
                                                .children(move || {
                                                    let style = if index == 0 || column_index == 0 {
                                                        emphasis_style(16.0, 24.0)
                                                    } else {
                                                        type_style(16.0, 24.0)
                                                    };
                                                    let modifier = Modifier::new();
                                                    let modifier = if index == 0 {
                                                        modifier.semantics(SemanticsArgs { test_tag: Some(format!("tooltip-table-header-{column_index}")), merge_descendants: true,
..Default::default() })
                                                    } else { modifier };
                                                    text(I18N.text(content))
                                                        .modifier(modifier)
                                                        .style(style)
                                                        .color(scheme.on_surface);
                                                });
                                        }
                                    });
                            }
                        });
                });
            spacer().modifier(
                Modifier::new()
                    .push_layout(TooltipTableColumnRule(table_size))
                    .background(scheme.outline_variant),
            );
        });
}

#[tessera]
fn tooltip_focus_marker(number: usize, x: f64, y: f64) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(20.0), Dp(20.0))
                .background_with_shape(srgb(29, 27, 32), Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(
                        I18N.format("ui-focus-order-marker-value", [("v0", (number).into())]),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(12.0, 16.0))
                .color(srgb(255, 255, 255));
        });
}

#[tessera]
fn tooltip_focus_sequence() {
    material_theme().theme(|| tooltip_reference_theme(false)).child(move || {
        let geometry = remember(SpaceGeometry::new);
        let ring = srgb(130, 64, 150);
        tooltip_stage(512.0, 300.0, "ui-rich-tooltip-focus-order-illustration".into(), geometry).child(move || {
            tooltip_grid(Dp(512.0), Dp(300.0));
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_in_space(geometry, "focus_trigger")
                        .offset(Dp(112.0), Dp(36.0))
                        .size(Dp(48.0), Dp(48.0))
                        .border_with_shape(Dp(3.0), ring, Shape::Ellipse),
                )
                .children(move || {
                    let content = RenderSlot::new(move || {
                        icon_button()
                            .standard()
                            .icon(filled::LOCK_SVG)
                            .accessibility_label(I18N.text("ui-permissions-illustration"));
                    });
                    content.render();
                });
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            surface()
                .modifier(Modifier::new().offset(Dp(148.0), Dp(84.0)).width(Dp(296.0)))
                .style(scheme.surface_container.into())
                .content_color(scheme.on_surface_variant)
                .shape(Shape::rounded_rectangle(Dp(12.0)))
                .elevation(Dp(3.0))
                .child(move || {
                    column()
                        .modifier(Modifier::new().padding_symmetric(Dp::ZERO, Dp(12.0)))
                        .children(move || {
                            column()
                                .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO))
                                .children(move || {
                                    text(I18N.text("ui-permissions"))
                                        .style(emphasis_style(14.0, 20.0));
                                    gap(Dp(4.0));
                                    text(I18N.text("ui-configure-permissions-for-selected-service-accounts-you-can-add-and-remove-service-account"))
                                        .style(type_style(14.0, 20.0));
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            text("Visit ").style(type_style(14.0, 20.0));
                                            boxed()
                                                .modifier(
                                                    watch_in_space(geometry, "focus_link")
                                                        .border_with_shape(
                                                            Dp(2.0),
                                                            ring,
                                                            Shape::rounded_rectangle(Dp(6.0)),
                                                        )
                                                        .padding_symmetric(Dp(2.0), Dp::ZERO)
                                                        .semantics(SemanticsArgs {
                                                            role: Some(Role::Link),
                                                            label: Some(I18N.text("ui-permission-settings-link-illustration")),
                                                            merge_descendants: true,
                                                            ..Default::default()
                                                        }),
                                                )
                                                .children(move || {
                                                    text(I18N.text("ui-permission-settings"))
                                                        .style(type_style(14.0, 20.0))
                                                        .color(scheme.primary);
                                                });
                                            text(I18N.text("ui-for-details")).style(type_style(14.0, 20.0));
                                        });
                                });
                            gap(Dp(8.0));
                            boxed()
                                .modifier(
                                    watch_in_space(geometry, "focus_action")
                                        .padding_symmetric(Dp(8.0), Dp::ZERO)
                                        .border_with_shape(Dp(2.0), ring, Shape::CAPSULE),
                                )
                                .children(move || {
                                    let content = RenderSlot::new(move || {
                                        button()
                                            .text()
                                            .accessibility_label(I18N.text("ui-request-access-action-illustration"))
                                            .child(move || {
                                                text(I18N.text("ui-request-access"));
                                            });
                                    });
                                    content.render();
                                });
                        });
                });
            tooltip_focus_annotations(geometry);
        });
    });
    gap(Dp(16.0));
    tooltip_legend(vec![
        "ui-parent-element".into(),
        "ui-inline-link".into(),
        "ui-text-button".into(),
    ]);
}

#[tessera]
fn tooltip_focus_annotations(geometry: State<SpaceGeometry>) {
    let values = geometry.with(space_geometry);
    for (number, name) in [(1, "focus_trigger"), (2, "focus_link"), (3, "focus_action")] {
        let Some(rect) = values.get(name) else {
            return;
        };
        tooltip_focus_marker(number, rect.x - 12.0, rect.y - 12.0);
    }
}

#[tessera]
fn tooltip_labeling_diagram() {
    material_theme()
        .theme(|| tooltip_reference_theme(true))
        .child(|| {
            let geometry = remember(SpaceGeometry::new);
            tooltip_stage(
                512.0,
                500.0,
                "ui-tooltip-roles-and-labels-illustration".into(),
                geometry,
            )
            .child(move || {
                boxed().modifier(
                    Modifier::new()
                        .required_size(Dp(512.0), Dp(500.0))
                        .background_with_shape(
                            srgb(68, 66, 84),
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                );
                boxed()
                    .modifier(Modifier::new().offset(Dp(300.0), Dp(44.0)))
                    .children(|| {
                        tooltip_plain_mock("ui-supporting-text".into());
                    });
                boxed()
                    .modifier(watch_in_space(geometry, "labeling_rich").offset(Dp(244.0), Dp(170.0)))
                    .children(move || {
                        tooltip_rich_mock(
                            "ui-add-others".into(),
                            "ui-share-this-collection-with-friends-and-family-people-you-add-to-this-album-will-also-be-ab".into(),
                        )
                        .actions(vec!["ui-learn-more".into()])
                        .width(232.0)
                        .geometry(geometry);
                    });
                tooltip_labeling_annotations(geometry);
            });
        });
}

#[tessera]
fn tooltip_labeling_annotations(geometry: State<SpaceGeometry>) {
    let values = geometry.with(space_geometry);
    let (Some(rich), Some(action)) = (
        values.get("labeling_rich").copied(),
        values.get("labeling_action").copied(),
    ) else {
        return;
    };
    let pink = "#f4a8f6";
    let plain_width = 106.0;
    let rich_center = rich.y + rich.h * 0.5;
    let action_center = action.x + action.w * 0.5;
    let label_top = rich.y + rich.h + 32.0;
    let svg = format!(
        "<rect x='298' y='42' width='{plain_width}' height='28' rx='6' stroke='{pink}' stroke-width='2' fill='none'/>\
         <path d='M160,56H298' stroke='{pink}' stroke-width='2' fill='none'/>\
         <rect x='{rx}' y='{ry}' width='{rw}' height='{rh}' rx='14' stroke='{pink}' stroke-width='2' fill='none'/>\
         <path d='M196,{rich_center}H{rx}' stroke='{pink}' stroke-width='2' fill='none'/>\
         <rect x='{ax}' y='{ay}' width='{aw}' height='{ah}' rx='{ar}' stroke='{pink}' stroke-width='2' fill='none'/>\
         <path d='M{action_center},{line_top}V{label_top}' stroke='{pink}' stroke-width='2' fill='none'/>",
        rx = rich.x - 2.0,
        ry = rich.y - 2.0,
        rw = rich.w + 4.0,
        rh = rich.h + 4.0,
        ax = action.x - 2.0,
        ay = action.y - 2.0,
        aw = action.w + 4.0,
        ah = action.h + 4.0,
        ar = (action.h + 4.0) * 0.5,
        line_top = action.y + action.h + 2.0,
    );
    annotations::lines(Dp(512.0), Dp(500.0), svg);
    tooltip_role_box(
        vec![("ui-role".into(), "ui-tooltip".into())],
        24.0,
        24.0,
        136.0,
    );
    tooltip_role_box(
        vec![("ui-role".into(), "ui-tooltip".into())],
        24.0,
        rich_center - 32.0,
        172.0,
    );
    tooltip_role_box(
        vec![
            ("ui-label".into(), "ui-learn-more".into()),
            ("ui-role".into(), "ui-button".into()),
        ],
        action_center - 72.0,
        label_top,
        144.0,
    );
}

#[tessera]
fn tooltip_role_box(entries: Vec<(String, String)>, x: f64, y: f64, width: f64) {
    let pink = srgb(244, 168, 246);
    let label = entries
        .iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    column()
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .width(Dp(width))
                .background_with_shape(srgb(20, 18, 24), Shape::rounded_rectangle(Dp(12.0)))
                .border_with_shape(Dp(2.0), pink, Shape::rounded_rectangle(Dp(12.0)))
                .padding_symmetric(Dp(16.0), Dp(12.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(I18N.text(label)),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for (index, (name, value)) in entries.iter().enumerate() {
                if index > 0 {
                    gap(Dp(8.0));
                }
                text(I18N.text(&(name)))
                    .style(type_style(14.0, 20.0))
                    .color(srgb(230, 224, 233));
                text(I18N.text(&(value)))
                    .style(emphasis_style(14.0, 20.0))
                    .color(pink);
            }
        });
}

#[tessera]
fn tooltip_one_at_a_time() {
    tooltip_guidance(2, "ui-only-display-one-tooltip-at-a-time".into()).figure(|| {
        tooltip_one_at_a_time_stage();
    });
}

#[tessera]
fn tooltip_writing_examples(rich: bool) {
    if rich {
        tooltip_guidance_pair(7, 8);
    } else {
        tooltip_guidance_pair(5, 6);
    }
}

#[tessera]
fn tooltip_one_at_a_time_stage(more: Option<bool>) {
    let more = more.unwrap_or(true);
    material_theme()
        .theme(|| tooltip_reference_theme(false))
        .child(move || {
            let geometry = remember(SpaceGeometry::new);
            tooltip_stage(
                1024.0,
                300.0,
                if more {
                    "ui-multiple-tooltips-caution-illustration"
                } else {
                    "ui-lingering-plain-tooltip-illustration"
                }
                .into(),
                geometry,
            )
            .child(move || {
                tooltip_browser_mock();
                tooltip_reference_popup(
                    geometry,
                    false,
                    false,
                    "ui-star".into(),
                    840.0,
                    166.0,
                    1024.0,
                    300.0,
                );
                if more {
                    tooltip_reference_popup(
                        geometry,
                        true,
                        false,
                        "ui-more".into(),
                        896.0,
                        166.0,
                        1024.0,
                        300.0,
                    );
                }
                for (icon, label, x) in [
                    (filled::STAR_BORDER_SVG, "ui-star-illustration", 816.0),
                    (filled::MORE_VERT_SVG, "ui-more-illustration", 872.0),
                ] {
                    boxed()
                        .modifier(Modifier::new().offset(Dp(x), Dp(112.0)))
                        .children(move || {
                            icon_button()
                                .standard()
                                .icon(icon)
                                .accessibility_label(I18N.text(label));
                        });
                }
            });
        });
}

#[tessera]
fn tooltip_browser_mock() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().modifier(
        Modifier::new()
            .required_size(Dp(1024.0), Dp(300.0))
            .background_with_shape(srgb(227, 223, 250), Shape::rounded_rectangle(Dp(24.0))),
    );
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(72.0), Dp(32.0))
                .required_size(Dp(880.0), Dp(268.0))
                .background_with_shape(srgb(170, 167, 178), Shape::rounded_rectangle(Dp(28.0)))
                .padding_all(Dp(8.0)),
        )
        .children(move || {
            boxed().modifier(
                Modifier::new()
                    .fill_max_size()
                    .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(20.0))),
            );
        });
    boxed().modifier(
        Modifier::new()
            .offset(Dp(176.0), Dp(56.0))
            .size(Dp(600.0), Dp(32.0))
            .background_with_shape(scheme.surface_container, Shape::CAPSULE),
    );
    for (painter, x, y, size) in [
        (filled::ARROW_BACK_SVG, 100.0, 62.0, 20.0),
        (filled::ARROW_FORWARD_SVG, 132.0, 62.0, 20.0),
        (filled::LOCK_SVG, 188.0, 64.0, 16.0),
        (filled::STAR_BORDER_SVG, 748.0, 62.0, 20.0),
        (filled::MORE_VERT_SVG, 884.0, 62.0, 20.0),
        (filled::ARROW_BACK_SVG, 104.0, 120.0, 24.0),
        (filled::PERSON_ADD_SVG, 776.0, 120.0, 24.0),
    ] {
        boxed()
            .modifier(Modifier::new().offset(Dp(x), Dp(y)))
            .children(move || {
                tessera_components::icon::icon()
                    .painter(painter)
                    .size(Dp(size))
                    .tint(scheme.on_surface_variant);
            });
    }
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(836.0), Dp(60.0))
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(srgb(120, 116, 126), Shape::Ellipse),
        )
        .children(move || {
            text("M")
                .style(type_style(12.0, 16.0))
                .color(srgb(255, 255, 255));
        });
    boxed()
        .modifier(Modifier::new().offset(Dp(144.0), Dp(118.0)))
        .children(move || {
            text(I18N.text("ui-lunch-at-lee-s"))
                .style(type_style(22.0, 28.0))
                .color(scheme.on_surface);
        });
    for (x, width, color) in [
        (144.0, 176.0, srgb(122, 138, 104)),
        (336.0, 504.0, srgb(160, 150, 140)),
    ] {
        boxed().modifier(
            Modifier::new()
                .offset(Dp(x), Dp(196.0))
                .size(Dp(width), Dp(104.0))
                .background_with_shape(color, Shape::rounded_rectangle(Dp(20.0))),
        );
    }
}

#[tessera]
fn tokens() {
    let rich = remember(|| false);
    prose("ui-select-a-component-variant-below-to-see-its-attributes-tokens-and-values".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for (label, value) in [("ui-plain-tooltip", false), ("ui-rich-tooltip", true)] {
                button()
                    .tonal()
                    .selected(rich.get() == value)
                    .accessibility_label(
                        I18N.format("ui-show-value-tokens", [("v0", I18N.text(&label).into())]),
                    )
                    .on_click(move || rich.set(value))
                    .child(move || {
                        text(I18N.text(label));
                    });
            }
        });
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-token-value".into()],
        if rich.get() {
            vec![
                vec!["ui-container-color".into(), "Surface container".into()],
                vec![
                    "ui-subhead-supporting-text-color".into(),
                    "On surface variant".into(),
                ],
                vec!["ui-action-color".into(), "Primary".into()],
                vec!["ui-subhead-type".into(), "Title small · 14sp / 20sp".into()],
                vec![
                    "ui-supporting-text-type".into(),
                    "Body medium · 14sp / 20sp".into(),
                ],
                vec!["ui-action-type".into(), "Label large · 14sp / 20sp".into()],
                vec!["ui-container-corner-size".into(), "12dp".into()],
                vec![
                    "ui-container-elevation".into(),
                    "ui-level-2-3dp-6571".into(),
                ],
            ]
        } else {
            vec![
                vec!["ui-container-color".into(), "Inverse surface".into()],
                vec![
                    "ui-supporting-text-color".into(),
                    "Inverse on surface".into(),
                ],
                vec![
                    "ui-supporting-text-type".into(),
                    "Body small · 12sp / 16sp".into(),
                ],
                vec!["ui-container-corner-size".into(), "4dp".into()],
                vec!["ui-container-elevation".into(), "ui-level-0-0dp".into()],
            ]
        },
    );
}

#[tessera]
fn detail(rich: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            tooltip_figure(rich, false);
            let title = if rich { "ui-rich-tooltip" } else { "ui-plain-tooltip" };
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            gap(Dp(112.0));
            subheading(I18N.format("ui-value-colors", [("v0", I18N.text(&title).into())]));
            gap(Dp(24.0));
            prose("ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo".into());
            gap(Dp(16.0));
            text(I18N.text("ui-learn-more-about-design-tokens"))
                .style(type_style(16.0, 24.0))
                .color(scheme.primary);
            gap(Dp(24.0));
            tooltip_color_pair(rich);
            gap(Dp(88.0));
            subheading(I18N.format("ui-value-measurements", [("v0", I18N.text(&title).into())]));
            gap(Dp(24.0));
            tooltip_figure(rich, true);
            gap(Dp(24.0));
            spec_table(
                vec!["ui-attribute".into(), "ui-value".into()],
                if rich {
                    vec![
                        vec!["ui-top-padding".into(), "12dp".into()],
                        vec!["ui-bottom-padding".into(), "8dp".into()],
                        vec!["ui-left-and-right-padding".into(), "16dp".into()],
                    ]
                } else {
                    vec![
                        vec!["ui-container-height".into(), "24dp".into()],
                        vec!["ui-padding".into(), "8dp".into()],
                    ]
                },
            );
            if rich {
                gap(Dp(112.0));
                subheading("ui-rich-tooltip-configurations".to_owned());
                gap(Dp(24.0));
                tooltip_configurations();
            }
        });
}

fn tooltip_color_labels(rich: bool) -> Vec<&'static str> {
    if rich {
        vec![
            "On surface variant",
            "Surface container",
            "On surface variant",
            "Primary",
        ]
    } else {
        vec!["Inverse on surface", "Inverse surface"]
    }
}

#[tessera]
fn tooltip_color_pair(rich: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        I18N.format(
                            "ui-value-tooltip-color-roles",
                            [(
                                "v0",
                                I18N.text(&(if rich { "ui-rich" } else { "ui-plain" }))
                                    .into(),
                            )],
                        ),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    for dark in [false, true] {
                        column()
                            .modifier(
                                Modifier::new().weight(1.0).semantics(SemanticsArgs {
                                    role: Some(Role::Group),
                                    label: Some(
                                        I18N.format(
                                            "ui-value-value-tooltip-color-example",
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
                                                    I18N.text(
                                                        &(if rich {
                                                            "ui-rich-7121"
                                                        } else {
                                                            "ui-plain-7122"
                                                        }),
                                                    )
                                                    .into(),
                                                ),
                                            ],
                                        ),
                                    ),
                                    merge_descendants: true,
                                    ..Default::default()
                                }),
                            )
                            .children(move || {
                                material_theme()
                                    .theme(move || tooltip_reference_theme(dark))
                                    .child(move || {
                                        tooltip_figure(rich, false).color_roles(true);
                                    });
                            });
                    }
                });
        });
    gap(Dp(24.0));
    prose(
        I18N.format(
            "ui-value-tooltip-color-roles-used-for-light-and-dark-themes",
            [(
                "v0",
                I18N.text(&(if rich { "ui-rich" } else { "ui-plain" }))
                    .into(),
            )],
        ),
    );
    gap(Dp(16.0));
    for (index, label) in tooltip_color_labels(rich).into_iter().enumerate() {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .children(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .size(Dp(24.0), Dp(32.0))
                            .background_with_shape(scheme.on_surface, Shape::CAPSULE),
                    )
                    .children(move || {
                        text((index + 1).to_string())
                            .style(type_style(14.0, 20.0))
                            .color(scheme.surface);
                    });
                text(I18N.text(label))
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(4.0)));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn tooltip_configurations() {
    prose("ui-rich-tooltips-can-have-a-headline-body-and-up-to-two-buttons-the-headline-and-number-of-bu".into());
    gap(Dp(24.0));
    material_theme()
        .theme(|| tooltip_reference_theme(true))
        .child(tooltip_configuration_canvas);
    gap(Dp(16.0));
    tooltip_legend(vec![
        "ui-subhead-supporting-text-and-two-buttons".into(),
        "ui-subhead-supporting-text-and-one-button".into(),
        "ui-subhead-and-supporting-text".into(),
        "ui-supporting-text-and-one-button".into(),
        "ui-supporting-text-and-two-buttons".into(),
    ]);
}

#[tessera]
fn tooltip_configuration_canvas() {
    let geometry = remember(SpaceGeometry::new);
    tooltip_stage(
        1024.0,
        697.0,
        "ui-rich-tooltip-five-configurations".into(),
        geometry,
    )
    .child(move || {
        tooltip_grid(Dp(1024.0), Dp(697.0));
        for (number, headline, actions, x, y) in [
            (1, true, 2, 122.0, 117.0),
            (2, true, 1, 122.0, 349.0),
            (3, true, 0, 590.0, 117.0),
            (4, false, 1, 590.0, 286.0),
            (5, false, 2, 590.0, 488.0),
        ] {
            tooltip_configuration(number, headline, actions, Dp(x), Dp(y));
        }
    });
}

#[tessera]
fn tooltip_configuration(number: usize, headline: bool, actions: usize, x: Dp, y: Dp) {
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let geometry = remember(SpaceGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let popup_height = geometry
        .with(|values| {
            values
                .get("popup")
                .map(|rect| Dp::from_pixels_f32(rect.height).0)
        })
        .unwrap_or(if actions == 0 {
            112.0
        } else if headline {
            156.0
        } else {
            132.0
        });
    boxed().modifier(Modifier::new().offset(Dp(x.0-8.0),Dp(y.0-48.0)).required_size(Dp(328.0),Dp(popup_height+56.0)).semantics(SemanticsArgs {
        role:Some(Role::Group),label:Some(I18N.format("ui-rich-tooltip-configuration-value", [("v0", (number).into())])),merge_descendants:true,..Default::default()
    })).children(move || {
        let content = RenderSlot::new(move || {
            let provider = tooltip_provider("ui-rich-tooltips-bring-attention-to-a-particular-element-or-feature-that-warrants-a-person-s")
                .controller(controller)
                .variant(TooltipVariant::Rich);
            if headline {
                provider.headline(I18N.text("ui-rich-tooltip"));
            }
            if actions > 0 {
                provider.actions(move || {
                    row().children(move || {
                        for action in 1..=actions {
                            button().text()
                                .accessibility_label(I18N.format("ui-tooltip-configuration-value-action-value", [("v0", (number).into()), ("v1", (action).into())]))
                                .child(move || { text(I18N.text("ui-action")); });
                        }
                    });
                });
            }
            provider
                .placement(TooltipPlacement::Below)
                .persistent(actions > 0)
                .popup_modifier(watch_in_space(geometry, "popup").width(Dp(312.0)))
                .child(move |_| {
                    boxed()
                        .modifier(Modifier::new().required_size(Dp(328.0), Dp(popup_height + 56.0)))
                        .children(move || {
                            tooltip_anchor(controller)
                                .modifier(Modifier::new().offset(Dp(8.0), Dp(8.0)).size(Dp(312.0), Dp(36.0)))
                                .child(move || {
                                    boxed()
                                        .modifier(Modifier::new().size(Dp(312.0), Dp(36.0)))
                                        .children(move || {
                                            boxed().alignment(Alignment::Center)
                                                .modifier(Modifier::new().required_size(Dp(24.0), Dp(24.0))
                                                    .border_with_shape(Dp(1.0), scheme.on_surface, Shape::Ellipse))
                                                .children(move || {
                                                    text((number).to_string())
                                                        .style(type_style(14.0, 20.0))
                                                        .color(scheme.on_surface);
                                                });
                                        });
                                });
                        });
                });
        });
        content.render();
    });
}

fn tooltip_reference_theme(dark: bool) -> MaterialTheme {
    let mut theme = MaterialTheme::from_seed(ACCENT, dark);
    let scheme = &mut theme.color_scheme;
    scheme.surface = if dark {
        srgb(20, 18, 24)
    } else {
        srgb(254, 247, 255)
    };
    scheme.surface_container_lowest = scheme.surface;
    scheme.surface_container_low = scheme.surface;
    scheme.surface_container = if dark {
        srgb(33, 31, 38)
    } else {
        srgb(243, 237, 247)
    };
    scheme.on_surface = if dark {
        srgb(230, 224, 233)
    } else {
        srgb(29, 27, 32)
    };
    scheme.on_surface_variant = if dark {
        srgb(202, 196, 208)
    } else {
        srgb(73, 69, 79)
    };
    scheme.inverse_surface = if dark {
        srgb(230, 224, 233)
    } else {
        srgb(50, 47, 53)
    };
    scheme.inverse_on_surface = if dark {
        srgb(50, 47, 53)
    } else {
        srgb(245, 239, 247)
    };
    scheme.primary = if dark {
        srgb(208, 188, 255)
    } else {
        srgb(103, 80, 164)
    };
    theme
}

#[tessera]
fn tooltip_stage(
    width: f64,
    height: f64,
    label: String,
    geometry: State<SpaceGeometry>,
    child: Option<RenderSlot>,
) {
    let child = child.unwrap_or_else(RenderSlot::empty);
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: tessera_ui::LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.is_alive() && measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            let label = label.clone();
            scaled_box((measured.get().0 / width) as f32).child(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(
                        watch_in_space(geometry, "figure")
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(
                                scheme.surface,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(I18N.text(&(label))),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || child.render());
            });
        });
}

#[tessera]
fn tooltip_figure(
    rich: bool,
    measurements: bool,
    color_roles: Option<bool>,
    guidelines: Option<bool>,
) {
    let color_roles = color_roles.unwrap_or(false);
    let guidelines = guidelines.unwrap_or(false);
    if color_roles {
        tooltip_figure_canvas(rich, measurements, color_roles, guidelines);
    } else {
        material_theme()
            .theme(|| tooltip_reference_theme(false))
            .child(move || {
                tooltip_figure_canvas(rich, measurements, color_roles, guidelines);
            });
    }
}

const RICH_REFERENCE_MESSAGE: &str = "ui-rich-tooltips-bring-attention-to-a-particular-element-or-feature-that-warrants-a-person-s-6584";
const RICH_USAGE_MESSAGE: &str =
    "ui-rich-tooltips-bring-attention-to-a-particular-element-or-feature-that-warrants-the-user-s";
const RICH_OVERVIEW_MESSAGE: &str = "ui-rich-tooltips-bring-attention-to-a-particular-element-or-feature-that-warrants-the-user-s-6586";

#[tessera]
fn tooltip_reference_popup(
    geometry: State<SpaceGeometry>,
    secondary: bool,
    rich: bool,
    message: String,
    center_x: f64,
    center_y: f64,
    width: f64,
    height: f64,
    guidelines: Option<bool>,
) {
    let guidelines = guidelines.unwrap_or(false);
    let headline = if guidelines {
        "ui-new-settings-available"
    } else {
        "ui-rich-tooltip"
    };
    let action_text = if guidelines {
        "ui-learn-more"
    } else {
        "ui-action"
    };
    let name = if secondary {
        "secondary_popup"
    } else {
        "popup"
    };
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let popup_height = geometry
        .with(|values| {
            values
                .get(name)
                .map(|rect| Dp::from_pixels_f32(rect.height).0)
        })
        .unwrap_or(if rich { 156.0 } else { 24.0 });
    let modifier = watch_in_space(geometry, name);
    let modifier = if rich {
        modifier.width(Dp(312.0))
    } else {
        modifier
    };
    let content = RenderSlot::new(move || {
        let anchor =
            RenderSlot::new(move || {
                boxed()
                    .modifier(Modifier::new().required_size(Dp(width), Dp(height)))
                    .children(move || {
                        tooltip_anchor(controller)
                            .modifier(Modifier::new().offset(
                                Dp(center_x - 0.5),
                                Dp(center_y + popup_height * 0.5 + 4.0),
                            ))
                            .child(|| {
                                spacer().modifier(Modifier::new().size(Dp(1.0), Dp(1.0)));
                            });
                    });
            });
        if rich {
            tooltip_provider(message.clone())
                .controller(controller)
                .variant(TooltipVariant::Rich)
                .headline(I18N.text(headline))
                .popup_modifier(modifier.clone())
                .actions(move || {
                    button()
                        .text()
                        .modifier(watch_in_space(geometry, "action"))
                        .accessibility_label(I18N.text(action_text))
                        .child(move || {
                            let style = use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .typography
                                .label_large;
                            text(I18N.text(action_text))
                                .style(style)
                                .modifier(watch_in_space(geometry, "action_label"));
                        });
                })
                .placement(TooltipPlacement::Above)
                .child(move |_| {
                    (anchor).render();
                });
        } else {
            tooltip_provider(message.clone())
                .controller(controller)
                .variant(TooltipVariant::Plain)
                .popup_modifier(modifier.clone())
                .placement(TooltipPlacement::Above)
                .child(move |_| {
                    (anchor).render();
                });
        }
    });
    content.render();
    if rich {
        let style = use_context::<MaterialTheme>()
            .unwrap()
            .get()
            .typography
            .title_small;
        // The provider exposes a title string, so annotations measure an
        // equivalent hidden label.
        text(I18N.text(headline)).style(style).modifier(
            watch_in_space(geometry, "headline_label")
                .size_in(None, Some(Dp(280.0)), None, None)
                .alpha(0.0)
                .semantics(SemanticsArgs {
                    hidden: true,
                    ..Default::default()
                }),
        );
    }
}

#[tessera]
fn tooltip_figure_canvas(rich: bool, measurements: bool, color_roles: bool, guidelines: bool) {
    let geometry = remember(SpaceGeometry::new);
    let width = if color_roles { 512.0 } else { 1024.0 };
    let height = if color_roles && rich { 338.0 } else { 300.0 };
    tooltip_stage(width, height, I18N.format("ui-value-tooltip-value-figure", [("v0", I18N.text(&(if rich { "ui-rich" } else { "ui-plain" })).into()), ("v1", I18N.text(&(if color_roles { "ui-color-7123" } else if measurements { "ui-measurement-7071" } else { "ui-anatomy-7072" })).into())]), geometry).child(move || {
        if guidelines {
            boxed().modifier(Modifier::new().required_size(Dp(width), Dp(height)).background_with_shape(srgb(247, 242, 250), Shape::rounded_rectangle(Dp(24.0))));
        } else {
            tooltip_grid(Dp(width), Dp(height));
        }
        if !rich && measurements {
            tooltip_reference_popup(geometry, false, false, "ui-save-to-favorites".into(), 282.0, 150.0, width, height);
            tooltip_reference_popup(geometry, true, false, "ui-grant-value-is-calculated-using-the-closing-stock-price-from-the-day-before-the-grant-date".into(), 710.0, 150.0, width, height);
        } else {
            tooltip_reference_popup(geometry, false, rich, if rich { if guidelines { "ui-now-you-can-adjust-the-uploaded-image-quality-and-upgrade-your-available-storage-space".into() } else { RICH_REFERENCE_MESSAGE.into() } } else { "ui-plain-tooltip".into() }, width * 0.5, height * 0.5, width, height).guidelines(guidelines);
        }
        boxed().modifier(Modifier::new().required_size(Dp(width), Dp(height))).children(move || {
            tooltip_annotations(geometry, rich, measurements).color_roles(color_roles).guidelines(guidelines);
        });
    });
    if !color_roles {
        gap(Dp(16.0));
        if guidelines && !measurements {
            tooltip_legend(if rich {
                vec![
                    "ui-subhead-optional".into(),
                    "ui-container".into(),
                    "ui-supporting-text".into(),
                    "ui-text-button-optional".into(),
                ]
            } else {
                vec!["ui-container".into(), "ui-supporting-text".into()]
            });
        } else if measurements {
            prose(
                I18N.format(
                    "ui-value-tooltip-padding-and-size-measurements",
                    [(
                        "v0",
                        I18N.text(&(if rich { "ui-rich" } else { "ui-plain" }))
                            .into(),
                    )],
                ),
            );
        } else {
            tooltip_legend(if rich {
                vec![
                    "ui-subhead".into(),
                    "ui-container".into(),
                    "ui-supporting-text".into(),
                    "ui-text-button".into(),
                ]
            } else {
                vec!["ui-supporting-text".into(), "ui-container".into()]
            });
        }
    }
}

#[tessera]
fn tooltip_grid(width: Dp, height: Dp) {
    let mut path = String::new();
    for row in 1..(height.0 / 8.0).ceil() as usize {
        for column in 1..(width.0 / 8.0).ceil() as usize {
            let x = column as f64 * 8.0 + 2.0;
            let y = row as f64 * 8.0 + 2.0;
            path.push_str(&format!("M{x},{y}h1v1h-1z"));
        }
    }
    annotations::lines(
        width,
        height,
        format!("<path d='{path}' fill='#79747e' fill-opacity='.6'/>"),
    );
}

#[tessera]
fn tooltip_annotations(
    geometry: State<SpaceGeometry>,
    rich: bool,
    measurements: bool,
    color_roles: Option<bool>,
    guidelines: Option<bool>,
) {
    let values = geometry.with(space_geometry);
    tooltip_annotation_content(
        values,
        rich,
        measurements,
        color_roles.unwrap_or(false),
        guidelines.unwrap_or(false),
    );
}

#[tessera]
fn tooltip_annotation_content(
    values: BTreeMap<&'static str, Rect>,
    rich: bool,
    measurements: bool,
    color_roles: bool,
    guidelines: bool,
) {
    let (Some(root), Some(popup)) = (values.get("figure").copied(), values.get("popup").copied())
    else {
        return;
    };
    let inset = if rich { 16.0 } else { 8.0 };
    let top = if rich { 12.0 } else { 4.0 };
    let bottom = if rich { 8.0 } else { 4.0 };
    let mut svg = String::new();
    if color_roles {
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        let ink = if scheme.is_dark { "#cac4d0" } else { "#49454f" };
        let labels = tooltip_color_labels(rich);
        let regions = if rich {
            let headline = Rect {
                x: popup.x + 16.0,
                y: popup.y + 12.0,
                w: values.get("headline_label").map_or(0.0, |label| label.w),
                h: values.get("headline_label").map_or(0.0, |label| label.h),
            };
            let Some(action) = values.get("action").copied() else {
                return;
            };
            let Some(action_label) = values.get("action_label").copied() else {
                return;
            };
            vec![
                (1, headline, headline.x + 40.0, true),
                (2, popup, popup.x + popup.w - 88.0, true),
                (
                    3,
                    Rect {
                        x: headline.x,
                        y: headline.y + 24.0,
                        w: popup.w - 32.0,
                        h: (action.y - headline.y - 36.0).max(0.0),
                    },
                    popup.x + popup.w - 64.0,
                    false,
                ),
                (
                    4,
                    action_label,
                    action_label.x + action_label.w * 0.5,
                    false,
                ),
            ]
        } else {
            vec![
                (
                    1,
                    Rect {
                        x: popup.x + 8.0,
                        y: popup.y + 4.0,
                        w: popup.w - 16.0,
                        h: popup.h - 8.0,
                    },
                    popup.x + 20.0,
                    false,
                ),
                (2, popup, popup.x + popup.w - 12.0, true),
            ]
        };
        for (number, rect, x, above) in regions {
            let end = if above { rect.y } else { rect.y + rect.h };
            if rich && number == 4 {
                let marker_x = popup.x - 40.0;
                let marker_y = rect.y + rect.h * 0.5 - 16.0;
                dotted(
                    &mut svg,
                    marker_x + 24.0,
                    marker_y + 16.0,
                    rect.x - marker_x - 24.0,
                    false,
                    ink,
                );
                tooltip_color_marker(
                    number,
                    labels[number - 1].into(),
                    Dp(marker_x),
                    Dp(marker_y),
                );
                dotted(&mut svg, rect.x, rect.y, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y + rect.h, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y, rect.h, true, ink);
                dotted(&mut svg, rect.x + rect.w, rect.y, rect.h, true, ink);
                continue;
            }
            let y = if above {
                popup.y
                    - if rich && number == 1 {
                        36.0
                    } else if rich {
                        56.0
                    } else {
                        52.0
                    }
            } else if rich {
                popup.y + popup.h - 24.0
            } else {
                popup.y + popup.h + 36.0
            };
            let start = if above { y + 32.0 } else { end };
            let length = if above { end - start } else { y - end };
            if number == 2 {
                svg.push_str(&format!("<path d='M{x},{start}v{length}' stroke='{ink}' stroke-width='1' fill='none'/><circle cx='{x}' cy='{end}' r='2.5' stroke='{ink}' stroke-width='1' fill='none'/>"));
            } else {
                dotted(&mut svg, x, start, length, true, ink);
            }
            if number != 2 {
                dotted(&mut svg, rect.x, rect.y, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y + rect.h, rect.w, false, ink);
                dotted(&mut svg, rect.x, rect.y, rect.h, true, ink);
                dotted(&mut svg, rect.x + rect.w, rect.y, rect.h, true, ink);
            }
            tooltip_color_marker(number, labels[number - 1].into(), Dp(x - 12.0), Dp(y));
        }
    } else if measurements {
        let mut bands = vec![
            Rect {
                x: popup.x,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
            Rect {
                x: popup.x + popup.w - inset,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
        ];
        if rich {
            let Some(action) = values.get("action").copied() else {
                return;
            };
            bands.extend([
                Rect {
                    x: popup.x,
                    y: popup.y,
                    w: popup.w,
                    h: top,
                },
                Rect {
                    x: popup.x,
                    y: popup.y + 32.0,
                    w: popup.w,
                    h: 4.0,
                },
                Rect {
                    x: popup.x,
                    y: action.y - 12.0,
                    w: popup.w,
                    h: 12.0,
                },
                Rect {
                    x: popup.x,
                    y: popup.y + popup.h - bottom,
                    w: popup.w,
                    h: bottom,
                },
            ]);
            for (label, y) in [
                ("12", popup.y + 6.0),
                ("4", popup.y + 34.0),
                ("12", action.y - 6.0),
                ("8", popup.y + popup.h - 4.0),
            ] {
                dotted(&mut svg, popup.x + popup.w, y, 18.0, false, "#e25b86");
                annotations::dimension(label.into(), Dp(popup.x + popup.w + 20.0), Dp(y - 10.0));
            }
        }
        let radius = if rich { 12.0 } else { 4.0 };
        let right = popup.x + popup.w;
        let bottom = popup.y + popup.h;
        svg.push_str(&format!("<path d='M{},{}H{}V{bottom}H{}Q{},{bottom} {},{}V{}Q{},{} {},{}Z M{},{}H{}V{bottom}H{}Q{right},{bottom} {right},{}V{}Q{right},{} {},{}Z' fill='#e883aa' fill-opacity='.32'/>",
            popup.x + radius, popup.y, popup.x + inset, popup.x + radius, popup.x, popup.x, bottom-radius, popup.y+radius, popup.x, popup.y, popup.x+radius, popup.y,
            right-radius, popup.y, right-inset, right-radius, bottom-radius, popup.y+radius, popup.y, right-radius, popup.y));
        for rect in bands.into_iter().skip(2) {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                rect.x + inset,
                rect.y,
                rect.w - inset * 2.0,
                rect.h
            ));
        }
        for x in [popup.x + inset * 0.5, popup.x + popup.w - inset * 0.5] {
            dotted(&mut svg, x, popup.y + popup.h + 4.0, 18.0, true, "#e25b86");
            annotations::dimension(
                format!("{inset:.0}"),
                Dp(x - 8.0),
                Dp(popup.y + popup.h + 24.0),
            );
        }
        if !rich {
            if let Some(other) = values.get("secondary_popup") {
                for x in [other.x, other.x + other.w - 8.0] {
                    svg.push_str(&format!("<rect x='{x}' y='{}' width='8' height='{}' fill='#e883aa' fill-opacity='.32'/>", other.y, other.h));
                    dotted(
                        &mut svg,
                        x + 4.0,
                        other.y + other.h + 4.0,
                        18.0,
                        true,
                        "#e25b86",
                    );
                    annotations::dimension("8".into(), Dp(x), Dp(other.y + other.h + 24.0));
                }
            }
            let x = popup.x - 24.0;
            svg.push_str(&format!(
                "<path d='M{},{}h8M{x},{}v{}M{},{}h8' stroke='#e25b86' fill='none'/>",
                x - 4.0,
                popup.y,
                popup.y,
                popup.h,
                x - 4.0,
                popup.y + popup.h
            ));
            annotations::dimension(
                "24".into(),
                Dp(x - 28.0),
                Dp(popup.y + popup.h * 0.5 - 10.0),
            );
        }
    } else if rich {
        let Some(action) = values.get("action").copied() else {
            return;
        };
        let Some(action_label) = values.get("action_label").copied() else {
            return;
        };
        let headline = Rect {
            x: popup.x + 16.0,
            y: popup.y + 12.0,
            w: values.get("headline_label").map_or(0.0, |label| label.w),
            h: values.get("headline_label").map_or(0.0, |label| label.h),
        };
        let body = Rect {
            x: headline.x,
            y: headline.y + headline.h + 4.0,
            w: popup.w - 32.0,
            h: action.y - 12.0 - headline.y - headline.h - 4.0,
        };
        let action_outline = Rect {
            x: action_label.x - 6.0,
            y: action_label.y - 2.0,
            w: action_label.w + 12.0,
            h: action_label.h + 4.0,
        };
        for rect in [headline, body, action_outline] {
            annotations::outline(&mut svg, rect);
        }
        let title_x = headline.x + 38.0;
        dotted(&mut svg, title_x, popup.y - 4.0, 16.0, true, "#49454f");
        annotations::marker(1, Dp(title_x - 12.0), Dp(popup.y - 36.0));
        let container_x = popup.x + popup.w - 88.0;
        svg.push_str(&format!("<path d='M{container_x},{}V{}' stroke='#49454f' stroke-width='1' fill='none'/><circle cx='{container_x}' cy='{}' r='2.5' stroke='#49454f' stroke-width='1' fill='none'/>", popup.y - 24.0, popup.y, popup.y));
        annotations::marker(2, Dp(container_x - 12.0), Dp(popup.y - 56.0));
        let body_x = popup.x + popup.w - 68.0;
        let body_marker_y = popup.y + popup.h + 8.0;
        dotted(
            &mut svg,
            body_x,
            body.y + body.h,
            body_marker_y - body.y - body.h,
            true,
            "#49454f",
        );
        annotations::marker(3, Dp(body_x - 12.0), Dp(body_marker_y));
        if guidelines {
            let marker_x = popup.x - 48.0;
            let marker_y = action_label.y + action_label.h * 0.5 - 16.0;
            dotted(
                &mut svg,
                marker_x + 24.0,
                marker_y + 16.0,
                action_outline.x - marker_x - 24.0,
                false,
                "#49454f",
            );
            annotations::marker(4, Dp(marker_x), Dp(marker_y));
        } else {
            let action_x = action_label.x + action_label.w * 0.5;
            dotted(
                &mut svg,
                action_x,
                action_outline.y + action_outline.h,
                popup.y + popup.h + 8.0 - action_outline.y - action_outline.h,
                true,
                "#49454f",
            );
            annotations::marker(4, Dp(action_x - 12.0), Dp(popup.y + popup.h + 8.0));
        }
    } else {
        let text = Rect {
            x: popup.x + 8.0,
            y: popup.y + 4.0,
            w: popup.w - 16.0,
            h: popup.h - 8.0,
        };
        let text_x = text.x + 8.0;
        dotted(&mut svg, text_x, text.y + text.h, 36.0, true, "#49454f");
        annotations::marker(
            if guidelines { 2 } else { 1 },
            Dp(text_x - 12.0),
            Dp(text.y + text.h + 36.0),
        );
        let container_x = popup.x + popup.w - 8.0;
        svg.push_str(&format!("<path d='M{container_x},{}V{}' stroke='#49454f' stroke-width='1' fill='none'/><circle cx='{container_x}' cy='{}' r='2.5' stroke='#49454f' stroke-width='1' fill='none'/>", popup.y - 36.0, popup.y, popup.y));
        annotations::marker(
            if guidelines { 1 } else { 2 },
            Dp(container_x - 12.0),
            Dp(popup.y - 68.0),
        );
        annotations::outline(&mut svg, text);
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn tooltip_color_marker(number: usize, label: String, x: Dp, y: Dp) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.on_surface, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some(I18N.format(
                        "ui-tooltip-color-role-value-value",
                        [("v0", (number).into()), ("v1", I18N.text(&label).into())],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text((number).to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn live_tooltips(on_acknowledge: Option<tessera_ui::Callback>) {
    let group = remember(|| None::<State<TooltipController>>);
    let plain = remember(TooltipController::new);
    let rich = remember(TooltipController::new);
    let favorite = remember(|| false);
    let acknowledged = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    tooltip_provider(if favorite.get() {
        "ui-remove-from-favorites"
    } else {
        "ui-add-to-favorites"
    })
    .controller(plain)
    .group(group)
    .persistent(false)
    .child(move |_| {
        tooltip_provider("ui-keep-related-items-together-so-you-can-find-them-again")
            .controller(rich)
            .group(group)
            .variant(TooltipVariant::Rich)
            .headline(I18N.text("ui-collections"))
            .placement(TooltipPlacement::Below)
            .persistent(true)
            .action_label(I18N.text("ui-got-it"))
            .on_action(move || {
                acknowledged.set(true);
                if let Some(callback) = on_acknowledge {
                    callback.call();
                }
            })
            .child(move |_| {
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .padding_all(Dp(24.0)),
                    )
                    .children(move || {
                        flow_row()
                            .main_axis_alignment(
                                tessera_components::alignment::MainAxisAlignment::Center,
                            )
                            .item_spacing(Dp(24.0))
                            .line_spacing(Dp(16.0))
                            .children(move || {
                                tooltip_anchor(plain).child(move || {
                                    icon_button()
                                        .standard()
                                        .selected(favorite.get())
                                        .icon(if favorite.get() {
                                            filled::FAVORITE_SVG
                                        } else {
                                            filled::FAVORITE_BORDER_SVG
                                        })
                                        .accessibility_label(I18N.text(if favorite.get() {
                                            "ui-remove-from-favorites"
                                        } else {
                                            "ui-add-to-favorites"
                                        }))
                                        .on_click(move || favorite.set(!favorite.get()));
                                });
                                tooltip_anchor(rich).child(move || {
                                    button()
                                        .text()
                                        .accessibility_label(I18N.text("ui-about-collections"))
                                        .on_click(move || rich.with_mut(TooltipController::show))
                                        .child(move || {
                                            text(I18N.text("ui-about-collections"));
                                        });
                                });
                            });
                        spacer().modifier(Modifier::new().height(Dp(192.0)));
                    });
            });
    });
    if acknowledged.get() {
        gap(Dp(16.0));
        prose("ui-collection-guidance-acknowledged".into());
    }
}

#[tessera]
fn placement_examples() {
    flow_row()
        .item_spacing(Dp(1.0))
        .line_spacing(Dp(1.0))
        .modifier(Modifier::new().clip_shape(Shape::rounded_rectangle(Dp(24.0))))
        .children(move || {
            for (alignment, placement) in [
                (Alignment::TopStart, TooltipPlacement::Above),
                (Alignment::TopEnd, TooltipPlacement::Above),
                (Alignment::BottomStart, TooltipPlacement::Below),
                (Alignment::BottomEnd, TooltipPlacement::Below),
            ] {
                let controller = remember(|| {
                    let mut value = TooltipController::new();
                    value.show();
                    value
                });
                boxed()
                    .modifier(
                        Modifier::new()
                            .size(Dp(288.0), Dp(232.0))
                            .background(srgb(227, 223, 250))
                            .clip_to_bounds()
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(I18N.format(
                                    "ui-placement-preview-value",
                                    [("v0", format!("{:?}", alignment).into())],
                                )),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        // Each cell crops one corner of a device frame so the
                        // trigger sits near the edge that forces repositioning.
                        let (x, y) = match alignment {
                            Alignment::TopStart => (24.0, 24.0),
                            Alignment::TopEnd => (-72.0, 24.0),
                            Alignment::BottomStart => (24.0, -128.0),
                            _ => (-72.0, -128.0),
                        };
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(x), Dp(y))
                                    .required_size(Dp(336.0), Dp(336.0))
                                    .background_with_shape(
                                        srgb(170, 167, 178),
                                        Shape::rounded_rectangle(Dp(32.0)),
                                    )
                                    .padding_all(Dp(8.0)),
                            )
                            .children(|| {
                                boxed().modifier(
                                    Modifier::new().fill_max_size().background_with_shape(
                                        srgb(254, 247, 255),
                                        Shape::rounded_rectangle(Dp(24.0)),
                                    ),
                                );
                            });
                        tooltip_provider("ui-add-annotations-and-highlights-with-the-paint-tool")
                            .controller(controller)
                            .variant(TooltipVariant::Rich)
                            .headline(I18N.text("ui-paint-tool"))
                            .placement(placement)
                            .persistent(false)
                            .modifier(Modifier::new().fill_max_size())
                            .actions(move || {
                                button()
                                    .text()
                                    .accessibility_label(I18N.text("ui-learn-more"))
                                    .child(move || {
                                        text(I18N.text("ui-learn-more"));
                                    });
                            })
                            .child(move |_| {
                                boxed()
                                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(36.0)))
                                    .children(move || {
                                        tooltip_anchor(controller)
                                            .modifier(Modifier::new().align(alignment))
                                            .child(move || {
                                                icon_button()
                                                    .standard()
                                                    .icon(filled::BRUSH_SVG)
                                                    .accessibility_label(I18N.format(
                                                        "ui-corner-tooltip-value",
                                                        [("v0", format!("{:?}", alignment).into())],
                                                    ))
                                                    .on_click(move || {
                                                        controller.with_mut(TooltipController::show)
                                                    });
                                            });
                                    });
                            });
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "tooltips_tests.rs"]
mod tests;
