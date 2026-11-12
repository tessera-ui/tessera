use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    list_item::list_item,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    search::{SearchBarColors, SearchBarController, docked_search_bar, search_bar},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    article_helpers::{bullets, caption, legend},
    gap, panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "search_layout_tests.rs"]
mod layout_tests;

#[tessera]
pub(super) fn search_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article("ui-search", "ui-search-helps-people-find-information-quickly", vec![
        SpecSection { title: "ui-variants", content: RenderSlot::new(variants) },
        SpecSection { title: "ui-configurations", content: RenderSlot::new(configurations) },
        SpecSection { title: "ui-tokens-specs", content: RenderSlot::new(tokens) },
        SpecSection { title: "ui-anatomy", content: RenderSlot::new(anatomy) },
        SpecSection { title: "ui-color", content: RenderSlot::new(colors) },
        SpecSection { title: "ui-states", content: RenderSlot::new(states) },
        SpecSection { title: "ui-measurements", content: RenderSlot::new(measurements) },
    ]).route_optional(route.clone()).pages(vec![
        ArticlePage { title: "ui-overview", intro: Some(RenderSlot::new(overview_intro)), sections: vec![] },
        ArticlePage { title: "ui-guidelines", intro: None, sections: vec![
            SpecSection { title: "ui-usage", content: RenderSlot::new(|| { prose("ui-use-search-for-products-with-many-items-to-manage-such-as-messages-or-files-keep-the-entry"); gap(Dp(24.0)); subheading("ui-different-ways-to-search".into()); prose("ui-a-search-bar-targets-the-current-view-a-search-app-bar-makes-global-search-a-primary-funct"); gap(Dp(24.0)); live_search(false,false,false); gap(Dp(24.0)); subheading("ui-search-suggestions-results".into()); prose("ui-use-list-items-for-suggestions-and-results-add-relevant-leading-icons-category-labels-avat"); }) },
            SpecSection { title: "ui-placement", content: RenderSlot::new(|| { prose("ui-place-search-near-the-top-of-the-content-and-close-to-what-it-searches-keep-its-internal-e"); gap(Dp(24.0)); subheading("ui-focused-search-layouts".into()); prose("ui-docked-search-opens-below-the-bar-on-medium-and-expanded-windows-full-screen-search-fills"); gap(Dp(24.0)); layout_examples(); }) },
            SpecSection { title: "ui-anatomy", content: RenderSlot::new(|| { anatomy(); gap(Dp(24.0)); subheading("ui-search-bar-container".into()); prose("ui-keep-the-rounded-container-visible-when-focus-changes-unfocused-outer-margins-are-24dp-foc"); gap(Dp(24.0)); subheading("ui-icons-icon-buttons".into()); prose("ui-use-a-leading-navigation-button-or-decorative-search-icon-include-no-more-than-two-trailin"); gap(Dp(24.0)); subheading("ui-text".into()); prose("ui-describe-the-searchable-information-with-a-concise-hint-such-as-search-your-messages-input"); }) },
            SpecSection { title: "ui-adaptive-design", content: RenderSlot::new(|| { prose("ui-keep-search-within-its-pane-and-scale-the-width-with-its-content-use-full-screen-suggestio"); gap(Dp(24.0)); layout_examples(); }) },
            SpecSection { title: "ui-behavior", content: RenderSlot::new(|| { subheading("ui-focused-search".into()); prose("ui-focus-can-reveal-historical-suggestions-before-typing-the-back-button-closes-the-results-a"); gap(Dp(24.0)); live_search(true,false,false); gap(Dp(24.0)); subheading("ui-scroll".into()); prose("ui-the-bar-may-remain-fixed-or-scroll-away-and-return-when-the-scrolling-direction-reverses-r"); gap(Dp(24.0)); subheading("ui-search-results".into()); prose("ui-execute-a-query-with-enter-or-choose-a-suggestion-directly-preserve-the-query-and-show-a-c"); }) },
        ] },
        ArticlePage { title: "ui-accessibility", intro: None, sections: vec![
            SpecSection { title: "ui-use-cases", content: RenderSlot::new(|| { prose("ui-people-must-be-able-to-reach-search-identify-its-hint-enter-and-execute-a-query-interact-w"); gap(Dp(24.0)); live_search(false,false,false); }) },
            SpecSection { title: "ui-interaction-style", content: RenderSlot::new(|| { subheading("ui-autosuggest".into()); prose("ui-announce-changes-when-suggestions-or-results-appear-so-screen-reader-users-know-that-new-c"); }) },
            SpecSection { title: "ui-initial-focus", content: RenderSlot::new(|| { prose("ui-initial-focus-reaches-the-first-interactive-element-often-a-leading-icon-button-or-the-tex"); }) },
            SpecSection { title: "ui-keyboard-navigation", content: RenderSlot::new(keyboard) },
            SpecSection { title: "ui-labeling-elements", content: RenderSlot::new(|| { prose("ui-use-the-hinted-search-text-as-the-input-s-accessible-name-name-icon-buttons-by-their-actio"); gap(Dp(24.0)); spec_table(vec!["ui-element".into(),"ui-accessible-label".into()],vec![vec!["ui-search-field".into(),"ui-search-your-messages".into()],vec!["ui-back-button".into(),"ui-close-search".into()],vec!["ui-clear-button".into(),"ui-clear-search".into()]]); }) },
        ] },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value))
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn variants() {
    live_search(true, false, false);
    gap(Dp(16.0));
    caption("ui-when-a-person-executes-a-search-results-appear-in-a-list-below-the-search-bar");
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "ui-use-search-for-navigating-a-product-with-queries".into(),
        "ui-a-search-bar-can-include-a-leading-search-icon-hinted-search-text-and-optional-trailing-ic".into(),
        "ui-search-can-display-suggested-keywords-or-phrases-as-a-person-types".into(),
        "ui-a-search-bar-displays-search-suggestions-or-results-in-a-list".into(),
        "ui-use-a-search-app-bar-to-provide-an-emphasized-global-entry-point".into(),
    ]);
    gap(Dp(32.0));
    live_search(false, false, false);
    gap(Dp(16.0));
    caption("ui-when-inputting-text-search-suggestions-or-results-appear-below-the-search-bar");
}

#[tessera]
fn configurations() {
    subheading("ui-style".into());
    prose("ui-search-comes-in-two-styles");
    gap(Dp(16.0));
    bullets(vec![
        "ui-contained-has-an-expressive-look-and-feel-it-uses-a-filled-container-to-separate-a-search".into(),
        "ui-divided-baseline-doesn-t-have-the-latest-visual-style-motion-or-flexibility".into(),
    ]);
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(16.0));
    caption(
        "ui-the-contained-style-has-a-persistent-filled-container-expressive-motion-and-rounded-shape",
    );
    gap(Dp(32.0));
    live_search(true, false, true);
    gap(Dp(16.0));
    caption(
        "ui-the-divided-baseline-style-uses-a-divider-to-separate-the-search-bar-from-suggestions-and",
    );
    gap(Dp(32.0));
    subheading("ui-layout".into());
    prose(
        "ui-search-suggestions-and-results-appear-in-customizable-lists-with-two-layout-options-full-s",
    );
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(32.0))
        .children(|| {
            configuration_layout(true, false);
            configuration_layout(false, false);
            configuration_layout(true, true);
            configuration_layout(false, true);
        });
}

#[tessera]
fn configuration_layout(full_screen: bool, divided: bool) {
    let controller = remember(|| {
        let mut controller = SearchBarController::new(true);
        controller.set_query(I18N.text("ui-eli"));
        controller
    });
    let submitted = remember(|| true);
    let opened = remember(String::new);
    let width = remember(|| Dp(502.0));
    let scale = width.get().0 / 502.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(502.0)))
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                            let next = Dp::from_pixels_f32(size.width.to_f32());
                            width.set_if_changed(next);
                        })
                        .fill_max_width()
                        .height(Dp(456.0 * scale))
                        .clip_to_bounds()
                        .background_with_shape(
                            scheme.surface_container_highest,
                            Shape::rounded_rectangle(Dp(16.0)),
                        ),
                )
                .children(move || {
                    boxed().modifier(
                        Modifier::new()
                            .offset(
                                Dp(if full_screen { 37.0 } else { -80.0 } * scale),
                                Dp(44.0 * scale),
                            )
                            .size(
                                Dp(if full_screen { 428.0 } else { 560.0 } * scale),
                                Dp(600.0 * scale),
                            )
                            .background_with_shape(
                                scheme.surface,
                                Shape::rounded_rectangle(Dp(28.0 * scale)),
                            )
                            .border_with_shape(
                                Dp(8.0 * scale),
                                scheme.outline_variant,
                                Shape::rounded_rectangle(Dp(28.0 * scale)),
                            ),
                    );
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .offset(
                                    Dp(if full_screen { 69.0 } else { -48.0 } * scale),
                                    Dp(72.0 * scale),
                                )
                                .width(Dp(if full_screen { 364.0 } else { 496.0 } * scale)),
                        )
                        .children(move || {
                            text("9:30")
                                .style(type_style(14.0 * scale, 20.0 * scale))
                                .modifier(Modifier::new().weight(1.0));
                            icon().painter(filled::WIFI_SVG).size(Dp(16.0 * scale));
                            icon()
                                .painter(filled::SIGNAL_CELLULAR_4_BAR_SVG)
                                .size(Dp(16.0 * scale));
                            icon()
                                .painter(filled::BATTERY_FULL_SVG)
                                .size(Dp(16.0 * scale));
                        });
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(
                                    Dp(if full_screen {
                                        if divided { 45.0 } else { 57.0 }
                                    } else {
                                        79.0
                                    } * scale),
                                    Dp(if full_screen {
                                        if divided { 132.0 } else { 116.0 }
                                    } else {
                                        126.0
                                    } * scale),
                                )
                                .size(
                                    Dp(if full_screen {
                                        if divided { 412.0 } else { 388.0 }
                                    } else {
                                        378.0
                                    } * scale),
                                    Dp(480.0 * scale),
                                ),
                        )
                        .children(move || {
                            search_control(controller, submitted, full_screen, divided)
                                .results_content(move || {
                                    email_results(controller, submitted, opened);
                                });
                        });
                });
            gap(Dp(12.0));
            prose(
                I18N.format(
                    "ui-value-value-style",
                    [
                        (
                            "v0",
                            I18N.text(
                                &(if full_screen {
                                    "ui-full-screen-layout"
                                } else {
                                    "ui-docked-layout"
                                }),
                            )
                            .into(),
                        ),
                        (
                            "v1",
                            I18N.text(
                                &(if divided {
                                    "ui-divided-baseline"
                                } else {
                                    "ui-contained"
                                }),
                            )
                            .into(),
                        ),
                    ],
                ),
            );
            if !opened.with(String::is_empty) {
                gap(Dp(8.0));
                prose(I18N.format(
                    "ui-opened-message-value",
                    [("v0", I18N.text(&opened.get()).into())],
                ));
            }
        });
}

#[tessera]
fn email_results(
    controller: State<SearchBarController>,
    submitted: State<bool>,
    opened: State<String>,
) {
    let query = controller.with(|value| value.query().to_lowercase());
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            role: Some(Role::List),
            merge_descendants: true,
            live: Some(tessera_ui::accesskit::Live::Polite),
            label: Some(I18N.text(if submitted.get() {
                "ui-email-search-results"
            } else {
                "ui-email-search-suggestions"
            })),
            ..Default::default()
        }))
        .children(move || {
            text(I18N.text(if submitted.get() {
                "ui-results"
            } else {
                "ui-suggestions"
            }))
            .style(type_style(12.0, 16.0))
            .modifier(Modifier::new().padding_all(Dp(16.0)));
            let mut count = 0;
            for (sender, subject, preview, time, initial) in [
                (
                    "ui-eli-me",
                    "ui-adopt-a-pup",
                    "ui-the-pet-adoption-organization",
                    "8:22 AM",
                    "E",
                ),
                (
                    "ui-zita-odette-dagmar",
                    "ui-movie-marathon",
                    "ui-we-have-to-watch-one-of-these",
                    "7:15 AM",
                    "Z",
                ),
                (
                    "ui-in-aki-aro",
                    "ui-museum-field-trip",
                    "ui-so-far-about-20-students-signed",
                    "9:40 AM",
                    "I",
                ),
            ] {
                if query.is_empty()
                    || I18N
                        .format(
                            "ui-value-value-value-to-eli",
                            [
                                ("v0", I18N.text(&sender).into()),
                                ("v1", I18N.text(&subject).into()),
                                ("v2", I18N.text(&preview).into()),
                            ],
                        )
                        .to_lowercase()
                        .contains(&query)
                {
                    count += 1;
                    list_item()
                        .headline(I18N.text(sender))
                        .supporting_text(format!("{}\n{}", I18N.text(subject), I18N.text(preview)))
                        .min_height(Dp(92.0))
                        .leading(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .size(Dp(40.0), Dp(40.0))
                                        .background_with_shape(
                                            scheme.tertiary_container,
                                            Shape::Ellipse,
                                        ),
                                )
                                .children(move || {
                                    text(I18N.text(initial))
                                        .style(type_style(16.0, 24.0))
                                        .color(scheme.on_tertiary_container);
                                });
                        })
                        .trailing(move || {
                            column().children(move || {
                                text(I18N.text(time)).style(type_style(12.0, 16.0));
                                gap(Dp(16.0));
                                icon().painter(filled::STAR_BORDER_SVG).size(Dp(24.0));
                            });
                        })
                        .on_click(move || {
                            submitted.set(true);
                            opened.set(subject.into());
                        });
                }
            }
            if count == 0 {
                prose("ui-no-matching-messages");
            }
        });
}

#[tessera]
fn live_search(initial_active: bool, full_screen: bool, divided: bool) {
    let controller = remember(move || SearchBarController::new(initial_active));
    let submitted = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        boxed()
            .modifier(
                Modifier::new()
                    .width(Dp(480.0))
                    .height(Dp(if full_screen { 480.0 } else { 380.0 }))
                    .padding_symmetric(
                        Dp(if controller.with(SearchBarController::is_active) {
                            12.0
                        } else {
                            24.0
                        }),
                        Dp::ZERO,
                    ),
            )
            .children(move || {
                search_control(controller, submitted, full_screen, divided);
            });
        gap(Dp(16.0));
        text(I18N.text(if full_screen {
            "ui-full-screen-layout"
        } else if divided {
            "ui-divided-baseline-style"
        } else {
            "ui-docked-layout-contained-style"
        }))
        .style(type_style(14.0, 20.0))
        .color(scheme.on_surface_variant);
    }));
}

#[tessera]
fn search_control(
    controller: State<SearchBarController>,
    submitted: State<bool>,
    full_screen: bool,
    divided: bool,
    results_content: Option<RenderSlot>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let leading = RenderSlot::new(move || {
        if controller.with(SearchBarController::is_active) {
            icon_button()
                .standard()
                .content_color(scheme.on_surface_variant)
                .icon(filled::ARROW_BACK_SVG)
                .accessibility_label(I18N.text("ui-close-search"))
                .on_click(move || {
                    controller.with_mut(SearchBarController::close);
                    submitted.set(false);
                });
        } else {
            icon().painter(filled::SEARCH_SVG).size(Dp(24.0));
        }
    });
    let trailing = RenderSlot::new(move || {
        if controller.with(|value| !value.query().is_empty()) {
            icon_button()
                .standard()
                .content_color(scheme.on_surface_variant)
                .icon(filled::CLOSE_SVG)
                .accessibility_label(I18N.text("ui-clear-search"))
                .on_click(move || {
                    controller.with_mut(|value| value.set_query("".to_owned()));
                    submitted.set(false);
                });
        } else {
            icon().painter(filled::MIC_SVG).size(Dp(24.0));
        }
    });
    let content = if let Some(content) = results_content {
        content
    } else {
        RenderSlot::new(move || {
            results(controller, submitted);
        })
    };
    let colors = SearchBarColors {
        container_color: scheme.surface_container_high,
        divider_color: if divided {
            scheme.outline_variant
        } else {
            Color::TRANSPARENT
        },
    };
    if full_screen {
        search_bar()
            .controller(controller)
            .placeholder(I18N.text("ui-search-your-messages"))
            .leading_icon_shared(leading)
            .trailing_icon_shared(trailing)
            .shape(if divided {
                Shape::RECTANGLE
            } else {
                Shape::rounded_rectangle(Dp(28.0))
            })
            .results_container_color(if divided {
                scheme.surface_container_high
            } else {
                scheme.surface_container_low
            })
            .colors(colors)
            .show_divider(divided)
            .content_padding(Dp::ZERO)
            .on_search(move |_| submitted.set(true))
            .on_query_change(move |query| {
                submitted.set(false);
                query
            })
            .modifier(Modifier::new().fill_max_size())
            .content_shared(content);
    } else {
        docked_search_bar()
            .controller(controller)
            .placeholder(I18N.text("ui-search-your-messages"))
            .leading_icon_shared(leading)
            .trailing_icon_shared(trailing)
            .shape(if divided {
                Shape::RoundedRectangle {
                    top_left: RoundedCorner::manual(Dp(28.0), 3.0),
                    top_right: RoundedCorner::manual(Dp(28.0), 3.0),
                    bottom_left: RoundedCorner::manual(Dp::ZERO, 3.0),
                    bottom_right: RoundedCorner::manual(Dp::ZERO, 3.0),
                }
            } else {
                Shape::rounded_rectangle(Dp(28.0))
            })
            .dropdown_shape(Shape::rounded_rectangle(Dp(if divided {
                0.0
            } else {
                28.0
            })))
            .dropdown_gap(Dp(if divided { 0.0 } else { 8.0 }))
            .colors(colors)
            .show_divider(divided)
            .content_padding(Dp::ZERO)
            .shadow_elevation(Dp(0.0))
            .on_search(move |_| submitted.set(true))
            .on_query_change(move |query| {
                submitted.set(false);
                query
            })
            .modifier(Modifier::new().fill_max_width())
            .content_shared(content);
    }
}

#[tessera]
fn results(controller: State<SearchBarController>, submitted: State<bool>) {
    let query = controller.with(|value| value.query().to_lowercase());
    let choices = [
        "ui-weekend-plans",
        "ui-dinner-with-friends",
        "ui-project-updates",
        "ui-travel-itinerary",
    ];
    let count = choices
        .iter()
        .filter(|label| query.is_empty() || I18N.text(label).to_lowercase().contains(&query))
        .count();
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(244.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::List),
                    merge_descendants: true,
                    live: Some(tessera_ui::accesskit::Live::Polite),
                    label: Some(if submitted.get() {
                        I18N.format("ui-value-search-results", [("v0", (count).into())])
                    } else {
                        I18N.format("ui-value-search-suggestions", [("v0", (count).into())])
                    }),
                    ..Default::default()
                }),
        )
        .children(move || {
            text(I18N.text(if submitted.get() {
                "ui-search-results"
            } else if query.is_empty() {
                "ui-recent-searches"
            } else {
                "ui-quick-results"
            }))
            .style(type_style(12.0, 16.0))
            .modifier(Modifier::new().padding_all(Dp(16.0)));
            for label in choices {
                result_item(
                    controller,
                    submitted,
                    label.to_owned(),
                    query.is_empty() || I18N.text(label).to_lowercase().contains(&query),
                );
            }
            if count == 0 {
                prose("ui-no-matching-messages");
            }
        });
}

#[tessera]
fn result_item(
    controller: State<SearchBarController>,
    submitted: State<bool>,
    label: String,
    shown: bool,
) {
    if shown {
        list_item()
            .headline(I18N.text(&label))
            .min_height(Dp(48.0))
            .leading(|| {
                icon().painter(filled::HISTORY_SVG).size(Dp(24.0));
            })
            .on_click(move || {
                controller.with_mut(|value| value.set_query(I18N.text(&label)));
                submitted.set(true);
            });
    }
}

#[tessera]
fn layout_examples() {
    subheading("ui-full-screen-7087".into());
    live_search(true, true, false);
    gap(Dp(24.0));
    subheading("ui-docked".into());
    live_search(true, false, false);
}

#[tessera]
fn anatomy() {
    search_figure(false);
    gap(Dp(16.0));
    column().children(|| {
        for (number, label) in [
            (1_usize, "ui-search-bar-container"),
            (2, "ui-leading-icon"),
            (3, "ui-hinted-search-text"),
            (4, "ui-trailing-icon-or-avatar"),
            (5, "ui-input-text"),
            (6, "ui-suggestions-or-results-container"),
        ] {
            legend(number, label);
        }
    });
    gap(Dp(16.0));
    caption(
        "ui-search-bar-anatomy-container-leading-icon-hinted-text-trailing-actions-input-text-and-the",
    );
    gap(Dp(32.0));
    subheading("ui-examples".into());
    panel(RenderSlot::new(|| {
        bar_example(0);
        gap(Dp(24.0));
        bar_example(1);
        gap(Dp(24.0));
        bar_example(2);
        gap(Dp(24.0));
        bar_example(3);
    }));
}

#[tessera]
fn bar_example(kind: usize) {
    let controller = remember(|| SearchBarController::new(false));
    docked_search_bar()
        .controller(controller)
        .placeholder(I18N.text("ui-hinted-search-text"))
        .modifier(Modifier::new().width(Dp(400.0)))
        .leading_icon(move || {
            icon_button()
                .standard()
                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                .icon(filled::MENU_SVG)
                .accessibility_label(I18N.text("ui-search-menu"));
        })
        .trailing_icon(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    if kind != 0 {
                        icon_button()
                            .standard()
                            .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                            .icon(filled::MIC_SVG)
                            .accessibility_label(I18N.text("ui-voice-search"));
                    }
                    if kind == 2 {
                        icon_button()
                            .standard()
                            .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                            .icon(filled::SEARCH_SVG)
                            .accessibility_label(I18N.text("ui-submit-search"));
                    }
                    if kind == 0 || kind == 3 {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                            .children(|| avatar());
                    }
                });
        });
}

#[tessera]
fn avatar() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(30.0), Dp(30.0))
                .background_with_shape(scheme.tertiary_container, Shape::Ellipse),
        )
        .children(move || {
            icon()
                .painter(filled::PERSON_SVG)
                .size(Dp(20.0))
                .tint(scheme.on_tertiary_container);
        });
}

#[tessera]
fn search_figure(measurements: bool) {
    let geometry = remember(Geometry::new);
    let compact = geometry.with(|value| {
        value
            .get("figure")
            .is_some_and(|bounds| Dp::from_pixels_f32(bounds.width.to_f32()).0 < 760.0)
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if measurements {
                    320.0
                } else if compact {
                    720.0
                } else {
                    400.0
                }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            if measurements {
                boxed()
                    .modifier(Modifier::new().width(Dp(400.0)))
                    .children(move || {
                        anatomy_bar(geometry, false);
                    });
            } else if compact {
                column().children(move || {
                    phone(geometry, false);
                    gap(Dp(80.0));
                    phone(geometry, true);
                });
            } else {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .children(move || {
                        phone(geometry, false);
                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                        phone(geometry, true);
                    });
            }
            boxed()
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    search_annotations(geometry, measurements);
                });
        });
}

#[tessera]
fn phone(geometry: State<Geometry>, active: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(336.0), Dp(272.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    merge_descendants: true,
                    label: Some(I18N.text(if active {
                        "ui-focused-search-phone"
                    } else {
                        "ui-unfocused-search-phone"
                    })),
                    ..Default::default()
                })
                .background_with_shape(scheme.outline_variant, Shape::rounded_rectangle(Dp(28.0)))
                .clip_shape(Shape::rounded_rectangle(Dp(28.0)))
                .padding_all(Dp(8.0)),
        )
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(20.0)))
                        .clip_shape(Shape::rounded_rectangle(Dp(20.0)))
                        .padding_all(Dp(12.0)),
                )
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
                        .children(move || {
                            text("9:30")
                                .style(type_style(14.0, 20.0))
                                .modifier(Modifier::new().weight(1.0));
                            icon().painter(filled::WIFI_SVG).size(Dp(16.0));
                            icon()
                                .painter(filled::SIGNAL_CELLULAR_4_BAR_SVG)
                                .size(Dp(16.0));
                            icon().painter(filled::BATTERY_FULL_SVG).size(Dp(16.0));
                        });
                    gap(Dp(16.0));
                    anatomy_bar(geometry, active);
                });
        });
}

#[tessera]
fn anatomy_bar(geometry: State<Geometry>, active: bool) {
    let controller = remember(move || {
        let mut value = SearchBarController::new(active);
        if active {
            value.set_query(I18N.text("ui-search-query"));
        }
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    search_bar()
        .controller(controller)
        .placeholder(I18N.text("ui-hinted-search-text"))
        .read_only(true)
        .modifier(watch(geometry, if active { "query" } else { "bar" }).fill_max_width())
        .show_divider(false)
        .results_container_color(scheme.surface)
        .content_padding(Dp::ZERO)
        .leading_icon(move || {
            if active {
                icon_button()
                    .standard()
                    .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                    .content_color(scheme.on_surface_variant)
                    .icon(filled::ARROW_BACK_SVG)
                    .accessibility_label(I18N.text("ui-close-search"))
                    .on_click(move || controller.with_mut(SearchBarController::close));
            } else {
                boxed().modifier(watch(geometry, "leading")).children(|| {
                    icon().painter(filled::MENU_SVG).size(Dp(24.0));
                });
            }
        })
        .trailing_icon(move || {
            boxed()
                .modifier(if active {
                    Modifier::new()
                } else {
                    watch(geometry, "trailing")
                })
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if active {
                                icon_button()
                                    .standard()
                                    .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                    .content_color(scheme.on_surface_variant)
                                    .icon(filled::CLOSE_SVG)
                                    .accessibility_label(I18N.text("ui-clear-search"))
                                    .on_click(move || {
                                        controller.with_mut(|value| value.set_query("".to_owned()))
                                    });
                            }
                            icon_button()
                                .standard()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .icon(filled::AUTO_AWESOME_SVG)
                                .accessibility_label(I18N.text("ui-search-suggestions"));
                            if !active {
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                    .children(|| avatar());
                            }
                        });
                });
        })
        .content(move || {
            spacer().modifier(
                watch(geometry, "results")
                    .fill_max_width()
                    .height(Dp(100.0)),
            );
        });
}

#[tessera]
fn search_annotations(geometry: State<Geometry>, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(bar), Some(leading), Some(trailing)) = (
        values.get("figure"),
        relative(&values, "bar"),
        relative(&values, "leading"),
        relative(&values, "trailing"),
    ) else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if measurements {
        svg.push_str(&format!("<rect x='{}' y='{}' width='16' height='{}' fill='#ee7da4' fill-opacity='.35'/><rect x='{}' y='{}' width='16' height='{}' fill='#ee7da4' fill-opacity='.35'/>",bar.x,bar.y,bar.h,bar.x+bar.w-16.0,bar.y,bar.h));
        for (label, x, y) in [
            ("56", bar.x + bar.w + 28.0, bar.y + 18.0),
            ("24", leading.x, bar.y - 40.0),
            ("24", trailing.x, bar.y - 40.0),
            ("16", bar.x, bar.y + bar.h + 24.0),
            ("4", leading.x + leading.w + 4.0, bar.y + bar.h + 24.0),
            ("4", trailing.x - 16.0, bar.y + bar.h + 24.0),
            ("16", bar.x + bar.w - 16.0, bar.y + bar.h + 24.0),
        ] {
            annotations::dimension(label.into(), Dp(x), Dp(y));
        }
        dotted(
            &mut svg,
            bar.x + bar.w + 16.0,
            bar.y,
            bar.h,
            true,
            "#ee7da4",
        );
    } else {
        for (n, x, y, tx, ty) in [
            (1, bar.x - 52.0, bar.y + 12.0, bar.x, bar.y + 28.0),
            (2, leading.x, bar.y - 64.0, leading.x + 12.0, leading.y),
            (3, bar.x + 102.0, bar.y - 64.0, bar.x + 114.0, bar.y + 28.0),
            (4, trailing.x, bar.y - 64.0, trailing.x + 12.0, trailing.y),
        ] {
            if n == 1 {
                dotted(&mut svg, x + 24.0, ty, tx - x - 24.0, false, "#49454f");
            } else {
                dotted(&mut svg, tx, y + 32.0, ty - y - 32.0, true, "#49454f");
            }
            annotations::marker(n, Dp(x), Dp(y));
        }
        annotations::outline(&mut svg, leading);
        annotations::outline(&mut svg, trailing);
        if let Some(query) = relative(&values, "query") {
            dotted(
                &mut svg,
                query.x + 110.0,
                query.y - 32.0,
                56.0,
                true,
                "#49454f",
            );
            annotations::marker(5, Dp(query.x + 98.0), Dp(query.y - 64.0));
        }
        if let Some(results) = relative(&values, "results") {
            dotted(
                &mut svg,
                results.x + results.w - 4.0,
                results.y + results.h / 2.0,
                40.0,
                false,
                "#49454f",
            );
            annotations::marker(
                6,
                Dp(results.x + results.w + 36.0),
                Dp(results.y + results.h / 2.0 - 16.0),
            );
        }
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn colors() {
    prose(
        "ui-keep-the-current-theme-s-surface-and-content-roles-the-search-field-uses-surface-container",
    );
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-color-role".into()],
        vec![
            vec!["ui-search-bar".into(), "Surface container high".into()],
            vec!["ui-input-text".into(), "On surface".into()],
            vec!["ui-hint-and-icons".into(), "On surface variant".into()],
            vec![
                "ui-full-screen-results".into(),
                "Surface container low".into(),
            ],
            vec!["ui-docked-results".into(), "Surface container high".into()],
        ],
    );
}

#[tessera]
fn states() {
    subheading("ui-search-bar".into());
    prose(
        "ui-enabled-hovered-focused-and-pressed-interact-with-the-field-to-inspect-its-live-state-laye",
    );
    gap(Dp(24.0));
    live_search(false, false, false);
    gap(Dp(24.0));
    subheading("ui-search-suggestions-results".into());
    prose(
        "ui-individual-result-items-retain-their-own-hover-focus-and-pressed-states-while-search-is-ac",
    );
    gap(Dp(24.0));
    live_search(true, false, false);
}

#[tessera]
fn measurements() {
    subheading("ui-search-bar".into());
    search_figure(true);
    gap(Dp(24.0));
    tokens();
    gap(Dp(32.0));
    subheading("ui-focused-search".into());
    prose(
        "ui-the-contained-style-keeps-a-56dp-bar-and-expands-its-outer-margins-from-24dp-to-12dp-when",
    );
    gap(Dp(24.0));
    layout_examples();
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-value".into(),
        ],
        vec![
            vec![
                "ui-full-screen-container".into(),
                "ui-width-height".into(),
                "ui-full-screen".into(),
            ],
            vec![
                "ui-docked-container".into(),
                "ui-width".into(),
                "360–720dp".into(),
            ],
            vec![
                "ui-docked-container".into(),
                "ui-height".into(),
                "ui-240dp-min-2-3-screen-max".into(),
            ],
            vec![
                "ui-focused-bar".into(),
                "ui-leading-trailing-padding-6983".into(),
                "16dp".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "ui-use-the-token-table-to-compare-the-search-bar-set-with-the-search-view-set-the-search-bar",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "md.comp.search-bar.container.color".into(),
                "Surface container high".into(),
            ],
            vec![
                "md.comp.search-bar.container.shape".into(),
                "ui-full-28dp-radius".into(),
            ],
            vec!["md.comp.search-bar.container.height".into(), "56dp".into()],
            vec![
                "md.comp.search-bar.input-text.color".into(),
                "On surface".into(),
            ],
            vec![
                "md.comp.search-bar.leading-icon.color".into(),
                "On surface variant".into(),
            ],
            vec![
                "md.comp.search-bar.trailing-icon.color".into(),
                "On surface variant".into(),
            ],
            vec![
                "md.comp.search-view.container.color".into(),
                "Surface container low".into(),
            ],
            vec![
                "md.comp.search-view.divider.color".into(),
                "Outline variant".into(),
            ],
            vec![
                "md.comp.search-view.header.color".into(),
                "On surface variant".into(),
            ],
            vec![
                "md.comp.search-view.list-item.label.color".into(),
                "On surface".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-element".into(),
            "ui-attribute".into(),
            "ui-value".into(),
        ],
        vec![
            vec!["ui-container".into(), "ui-width".into(), "360–720dp".into()],
            vec!["ui-container".into(), "ui-height".into(), "56dp".into()],
            vec![
                "ui-container".into(),
                "ui-shape".into(),
                "ui-full-28dp-radius".into(),
            ],
            vec![
                "ui-container".into(),
                "ui-outer-margins".into(),
                "ui-unfocused-24dp-focused-12dp".into(),
            ],
            vec!["ui-label".into(), "ui-alignment".into(), "ui-start".into()],
            vec![
                "ui-icon-and-label".into(),
                "ui-gap-from-tap-target".into(),
                "4dp".into(),
            ],
            vec!["ui-avatar".into(), "ui-size".into(), "30dp".into()],
        ],
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-action".into()],
        vec![
            vec![
                "ui-tab-shift-tab".into(),
                "ui-navigate-between-controls".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-a-search-control".into(),
            ],
            vec![
                "ui-enter-in-the-field".into(),
                "ui-execute-the-query".into(),
            ],
        ],
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::time::Duration;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        headless::{HeadlessConfig, HeadlessRenderer},
    };

    use super::*;

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            let controller = remember(|| SearchBarController::new(false));
            let submitted = remember(|| false);
            boxed()
                .modifier(Modifier::new().width(Dp(480.0)).height(Dp(420.0)))
                .children(move || {
                    search_control(controller, submitted, false, false);
                });
        });
    }

    fn click(renderer: &mut HeadlessRenderer, label: &str) {
        let node = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .into_iter()
            .find(|(_, node)| crate::has_label(&node, label))
            .unwrap_or_else(|| panic!("missing {label}"))
            .1;
        let bounds = node.bounds().unwrap();
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(2);
    }

    #[tessera]
    fn trailing_targets_fixture() {
        material_theme().child(|| {
            {
                bar_example(2);
            };
        });
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn search_anatomy_keeps_two_full_trailing_targets_and_centered_input() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(trailing_targets_fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 400,
                height: 100,
                sample_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(4);
        let tree = renderer.accessibility_snapshot().unwrap();
        for (label, x) in [("Voice search", 300.0), ("Submit search", 348.0)] {
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Button && crate::has_label(&node, label))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert_eq!(
                (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
                (x, 4.0, 48.0, 48.0)
            );
        }
        let input = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::TextInput)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(input.x0, 56.0);
        assert_eq!((input.y0 + input.y1) * 0.5, 28.0);
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn live_search_filters_real_input_and_result_activation_preserves_query() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 600,
                height: 500,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::TextInput
                    && crate::has_label(&node, "Search your messages"))
        );
        click(&mut renderer, "Search your messages");
        for character in "Weekend".chars() {
            renderer.inject_text(&character.to_string());
        }
        renderer.render_frames(3);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::TextInput && node.value() == Some("Weekend")),
            "batched character commits must preserve the full query"
        );
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Weekend plans")
                    && node.role() == Role::ListItem),
            "published focus {:?}; nodes {:?}",
            snapshot.focus,
            snapshot
                .nodes
                .iter()
                .map(|(id, node)| (id, node.role(), node.label(), node.value(), node.bounds()))
                .collect::<Vec<_>>()
        );
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Dinner with friends"))
        );
        click(&mut renderer, "Weekend plans");
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::List
                    && crate::has_label(&node, "1 search results"))
        );
        click(&mut renderer, "Clear search");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| crate::has_label(&node, "Dinner with friends")),
            "after clear {:?}",
            snapshot
                .nodes
                .iter()
                .map(|(_, node)| (node.role(), node.label(), node.value(), node.bounds()))
                .collect::<Vec<_>>()
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::List
                    && crate::has_label(&node, "4 search suggestions"))
        );
    }
    #[tessera]
    fn anatomy_fixture() {
        material_theme().child(|| {
            search_figure(false);
        });
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn search_anatomy_keeps_six_callouts_and_adapts_phone_geometry() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(anatomy_fixture),
            vec![Box::new(crate::TestPipelines)],
            HeadlessConfig {
                width: 900,
                height: 800,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(6);
        let bounds = |renderer: &HeadlessRenderer, label: &str| {
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| crate::has_label(&node, label))
                .unwrap_or_else(|| panic!("missing {label}"))
                .1
                .bounds()
                .unwrap()
        };
        let left = bounds(&renderer, "Unfocused search phone");
        let right = bounds(&renderer, "Focused search phone");
        assert_eq!(left.y0, right.y0);
        assert!(left.x1 < right.x0);
        for number in 1..=6 {
            bounds(&renderer, &format!("Anatomy marker {number}"));
        }
        renderer.resize(600, 800);
        renderer.render_frames(6);
        let top = bounds(&renderer, "Unfocused search phone");
        let bottom = bounds(&renderer, "Focused search phone");
        assert_eq!(top.x0, bottom.x0);
        assert!(top.y1 < bottom.y0);
        for number in 1..=6 {
            bounds(&renderer, &format!("Anatomy marker {number}"));
        }
    }
}
