//! Search specifications with editable queries and results.
//!
//! ## Usage
//!
//! Inspect search layouts, anatomy and keyboard actions.

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
    Color, Dp, Modifier, RenderSlot, State, accesskit::Role, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "search_layout_tests.rs"]
mod layout_tests;

#[tessera]
pub(super) fn search_spec() {
    article("Search", "Search helps people find information quickly", vec![
        SpecSection { title: "Variants", content: RenderSlot::new(variants) },
        SpecSection { title: "Configurations", content: RenderSlot::new(configurations) },
        SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
        SpecSection { title: "Anatomy", content: RenderSlot::new(anatomy) },
        SpecSection { title: "Color", content: RenderSlot::new(colors) },
        SpecSection { title: "States", content: RenderSlot::new(states) },
        SpecSection { title: "Measurements", content: RenderSlot::new(measurements) },
    ]).pages(vec![
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(overview_intro)), sections: vec![
            SpecSection { title: "Availability & resources", content: RenderSlot::new(availability) },
            SpecSection { title: "M3 Expressive update", content: RenderSlot::new(expressive_update) },
            SpecSection { title: "Differences from M2 to M3 baseline", content: RenderSlot::new(differences) },
        ] },
        ArticlePage { title: "Guidelines", intro: None, sections: vec![
            SpecSection { title: "Usage", content: RenderSlot::new(|| { prose("Use search for products with many items to manage, such as messages or files. Keep the entry point easy to find."); gap(Dp(24.0)); subheading("Different ways to search".into()); prose("A search bar targets the current view. A search app bar makes global search a primary function. An icon button works when search is a secondary action."); gap(Dp(24.0)); live_search(false,false,false); gap(Dp(24.0)); subheading("Search suggestions & results".into()); prose("Use list items for suggestions and results. Add relevant leading icons, category labels, avatars or filter chips. Use gaps to separate related groups."); }) },
            SpecSection { title: "Placement", content: RenderSlot::new(|| { prose("Place search near the top of the content and close to what it searches. Keep its internal elements anchored to the leading and trailing edges as it grows."); gap(Dp(24.0)); subheading("Focused search layouts".into()); prose("Docked search opens below the bar on medium and expanded windows. Full-screen search fills compact windows."); gap(Dp(24.0)); layout_examples(); }) },
            SpecSection { title: "Anatomy", content: RenderSlot::new(|| { anatomy(); gap(Dp(24.0)); subheading("Search bar container".into()); prose("Keep the rounded container visible when focus changes. Unfocused outer margins are 24dp; focused margins are 12dp. Use surface container high against a surface at least two tonal steps away."); gap(Dp(24.0)); subheading("Icons & icon buttons".into()); prose("Use a leading navigation button or decorative search icon. Include no more than two trailing actions, or an avatar with one other action. A focused bar can include a clear button."); gap(Dp(24.0)); subheading("Text".into()); prose("Describe the searchable information with a concise hint, such as Search your messages. Input text replaces the hint as a person types."); }) },
            SpecSection { title: "Adaptive design", content: RenderSlot::new(|| { prose("Keep search within its pane and scale the width with its content. Use full-screen suggestions and results at compact widths, then switch to docked search on larger windows."); gap(Dp(24.0)); layout_examples(); }) },
            SpecSection { title: "Behavior", content: RenderSlot::new(|| { subheading("Focused search".into()); prose("Focus can reveal historical suggestions before typing. The back button closes the results and returns to the original bar. The clear button removes input without leaving search."); gap(Dp(24.0)); live_search(true,false,false); gap(Dp(24.0)); subheading("Scroll".into()); prose("The bar may remain fixed or scroll away and return when the scrolling direction reverses. Results scroll below the bar."); gap(Dp(24.0)); subheading("Search results".into()); prose("Execute a query with Enter, or choose a suggestion directly. Preserve the query and show a clear results indicator after searching."); }) },
        ] },
        ArticlePage { title: "Accessibility", intro: None, sections: vec![
            SpecSection { title: "Use cases", content: RenderSlot::new(|| { prose("People must be able to reach search, identify its hint, enter and execute a query, interact with suggestions and results, and clear the input."); gap(Dp(24.0)); live_search(false,false,false); }) },
            SpecSection { title: "Interaction & style", content: RenderSlot::new(|| { subheading("Autosuggest".into()); prose("Announce changes when suggestions or results appear, so screen reader users know that new choices are available. Label the results region and preserve a logical reading order."); }) },
            SpecSection { title: "Initial focus", content: RenderSlot::new(|| { prose("Initial focus reaches the first interactive element, often a leading icon button or the text field. A decorative leading search icon does not add a focus stop."); }) },
            SpecSection { title: "Keyboard navigation", content: RenderSlot::new(keyboard) },
            SpecSection { title: "Labeling elements", content: RenderSlot::new(|| { prose("Use the hinted search text as the input's accessible name. Name icon buttons by their action. Suggestions and results form a list with meaningful item labels."); gap(Dp(24.0)); spec_table(vec!["Element".into(),"Accessible label".into()],vec![vec!["Search field".into(),"Search your messages".into()],vec!["Back button".into(),"Close search".into()],vec!["Clear button".into(),"Clear search".into()]]); }) },
        ] },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

#[tessera]
fn legend(number: usize, #[prop(into)] label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width().height(Dp(32.0)))
        .children(move || {
            badge(number);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text().content(label.clone()).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn bullets(items: Vec<String>) {
    column().children(move || {
        for item in items.clone() {
            text()
                .content(format!("•  {item}"))
                .style(type_style(16.0, 24.0));
            gap(Dp(8.0));
        }
    });
}

#[tessera]
fn variants() {
    live_search(true, false, false);
    gap(Dp(16.0));
    caption("When a person executes a search, results appear in a list below the search bar");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![vec![
            "Search".into(),
            "Available".into(),
            "Available".into(),
        ]],
    );
}

#[tessera]
fn overview_intro() {
    bullets(vec![
        "Use search for navigating a product with queries".into(),
        "A search bar can include a leading search icon, hinted search text, and optional trailing icons".into(),
        "Search can display suggested keywords or phrases as a person types".into(),
        "A search bar displays search suggestions or results in a list".into(),
        "Use a search app bar to provide an emphasized, global entry-point".into(),
    ]);
    gap(Dp(32.0));
    live_search(false, false, false);
    gap(Dp(16.0));
    caption("When inputting text, search suggestions or results appear below the search bar");
}

#[tessera]
fn availability() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Unavailable".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native search_bar and docked_search_bar components. The contained style keeps a filled container above the results; the divided style is the baseline treatment.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "Search has a new visual style, motion, and more flexibility for trailing icons. The search bar and search view are now collectively named search.",
    );
    gap(Dp(24.0));
    subheading("February 2025".into());
    bullets(vec![
        "Naming: Search bar and search view are now collectively named search".into(),
        "Configurations: Styles: Search can be contained (recommended) or divided; gaps can separate results into groups".into(),
        "Motion: The search bar grows wider when focused".into(),
        "Supported platforms: Jetpack Compose".into(),
    ]);
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(16.0));
    caption("The contained search style features a persistent, filled search container");
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color".into(),
        "Elevation: Lower elevation and no shadow by default".into(),
        "Name: Search was formerly known as open search bar".into(),
        "Variants: Two official variants of search components: search bar and search view".into(),
    ]);
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(16.0));
    caption("M3 search bars are rounded, use tonal surface, and support dynamic color");
}
#[tessera]
fn configurations() {
    subheading("Style".into());
    prose("Search comes in two styles:");
    gap(Dp(16.0));
    bullets(vec![
        "Contained: Has an expressive look and feel. It uses a filled container to separate a search bar from a list of suggestions or results".into(),
        "Divided (baseline): Doesn’t have the latest visual style, motion, or flexibility".into(),
    ]);
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(16.0));
    caption(
        "The contained style has a persistent, filled container, expressive motion, and rounded shape",
    );
    gap(Dp(32.0));
    live_search(true, false, true);
    gap(Dp(16.0));
    caption(
        "The divided (baseline) style uses a divider to separate the search bar from suggestions and results",
    );
    gap(Dp(32.0));
    subheading("Layout".into());
    prose(
        "Search suggestions and results appear in customizable lists, with two layout options: full-screen and docked.",
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
    gap(Dp(32.0));
    spec_table(
        vec![
            "Category".into(),
            "Configuration".into(),
            "M3".into(),
            "M3 Expressive".into(),
        ],
        vec![
            vec![
                "Style".into(),
                "Contained".into(),
                "--".into(),
                "Available".into(),
            ],
            vec![
                "Style".into(),
                "Divided".into(),
                "Available".into(),
                "Not recommended. Use contained.".into(),
            ],
            vec![
                "Layout".into(),
                "Docked, full-screen".into(),
                "Available".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn configuration_layout(full_screen: bool, divided: bool) {
    let controller = remember(|| {
        let mut controller = SearchBarController::new(true);
        controller.set_query("Eli");
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
                        .on_globally_positioned(move |bounds: tessera_ui::PxRect| {
                            let next = Dp::from_pixels_f32(bounds.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
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
                            text()
                                .content("9:30")
                                .style(type_style(14.0 * scale, 20.0 * scale))
                                .modifier(Modifier::new().weight(1.0));
                            icon().vector(filled::WIFI_SVG).size(Dp(16.0 * scale));
                            icon()
                                .vector(filled::SIGNAL_CELLULAR_4_BAR_SVG)
                                .size(Dp(16.0 * scale));
                            icon()
                                .vector(filled::BATTERY_FULL_SVG)
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
            prose(format!(
                "{} · {} style",
                if full_screen {
                    "Full-screen layout"
                } else {
                    "Docked layout"
                },
                if divided {
                    "Divided (baseline)"
                } else {
                    "Contained"
                }
            ));
            if !opened.with(String::is_empty) {
                gap(Dp(8.0));
                prose(format!("Opened message: {}", opened.get()));
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
        .modifier(
            Modifier::new().fill_max_width().semantics(SemanticsArgs {
                role: Some(Role::List),
                merge_descendants: true,
                live: Some(tessera_ui::accesskit::Live::Polite),
                label: Some(
                    if submitted.get() {
                        "Email search results"
                    } else {
                        "Email search suggestions"
                    }
                    .into(),
                ),
                ..Default::default()
            }),
        )
        .children(move || {
            text()
                .content(if submitted.get() {
                    "Results"
                } else {
                    "Suggestions"
                })
                .style(type_style(12.0, 16.0))
                .modifier(Modifier::new().padding_all(Dp(16.0)));
            let mut count = 0;
            for (sender, subject, preview, time, initial) in [
                (
                    "Eli, me",
                    "Adopt a pup?",
                    "The pet adoption organization...",
                    "8:22 AM",
                    "E",
                ),
                (
                    "Zita, Odette, Dagmar",
                    "Movie marathon",
                    "We have to watch one of these...",
                    "7:15 AM",
                    "Z",
                ),
                (
                    "In, Aki, Aro",
                    "Museum field trip",
                    "So far about 20 students signed...",
                    "9:40 AM",
                    "I",
                ),
            ] {
                if query.is_empty()
                    || format!("{sender} {subject} {preview} to Eli")
                        .to_lowercase()
                        .contains(&query)
                {
                    count += 1;
                    list_item()
                        .headline(sender)
                        .supporting_text(format!("{subject}\n{preview}"))
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
                                    text()
                                        .content(initial)
                                        .style(type_style(16.0, 24.0))
                                        .color(scheme.on_tertiary_container);
                                });
                        })
                        .trailing(move || {
                            column().children(move || {
                                text().content(time).style(type_style(12.0, 16.0));
                                gap(Dp(16.0));
                                icon().vector(filled::STAR_BORDER_SVG).size(Dp(24.0));
                            });
                        })
                        .on_click(move || {
                            submitted.set(true);
                            opened.set(subject.into());
                        });
                }
            }
            if count == 0 {
                prose("No matching messages");
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
        text()
            .content(if full_screen {
                "Full-screen layout"
            } else if divided {
                "Divided (baseline) style"
            } else {
                "Docked layout · Contained style"
            })
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
                .accessibility_label("Close search")
                .on_click(move || {
                    controller.with_mut(SearchBarController::close);
                    submitted.set(false);
                });
        } else {
            icon().vector(filled::SEARCH_SVG).size(Dp(24.0));
        }
    });
    let trailing = RenderSlot::new(move || {
        if controller.with(|value| !value.query().is_empty()) {
            icon_button()
                .standard()
                .content_color(scheme.on_surface_variant)
                .icon(filled::CLOSE_SVG)
                .accessibility_label("Clear search")
                .on_click(move || {
                    controller.with_mut(|value| value.set_query(""));
                    submitted.set(false);
                });
        } else {
            icon().vector(filled::MIC_SVG).size(Dp(24.0));
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
            .placeholder("Search your messages")
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
            .placeholder("Search your messages")
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
        "Weekend plans",
        "Dinner with friends",
        "Project updates",
        "Travel itinerary",
    ];
    let count = choices
        .iter()
        .filter(|label| query.is_empty() || label.to_lowercase().contains(&query))
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
                        format!("{count} search results")
                    } else {
                        format!("{count} search suggestions")
                    }),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(if submitted.get() {
                    "Search results"
                } else if query.is_empty() {
                    "Recent searches"
                } else {
                    "Quick results"
                })
                .style(type_style(12.0, 16.0))
                .modifier(Modifier::new().padding_all(Dp(16.0)));
            for label in choices {
                result_item(
                    controller,
                    submitted,
                    label.to_owned(),
                    query.is_empty() || label.to_lowercase().contains(&query),
                );
            }
            if count == 0 {
                prose("No matching messages");
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
            .headline(label.clone())
            .min_height(Dp(48.0))
            .leading(|| {
                icon().vector(filled::HISTORY_SVG).size(Dp(24.0));
            })
            .on_click(move || {
                controller.with_mut(|value| value.set_query(label.clone()));
                submitted.set(true);
            });
    }
}

#[tessera]
fn layout_examples() {
    subheading("Full-screen".into());
    live_search(true, true, false);
    gap(Dp(24.0));
    subheading("Docked".into());
    live_search(true, false, false);
}

#[tessera]
fn anatomy() {
    search_figure(false);
    gap(Dp(16.0));
    column().children(|| {
        for (number, label) in [
            (1_usize, "Search bar container"),
            (2, "Leading icon"),
            (3, "Hinted search text"),
            (4, "Trailing icon or avatar"),
            (5, "Input text"),
            (6, "Suggestions or results container"),
        ] {
            legend(number, label);
        }
    });
    gap(Dp(16.0));
    caption(
        "Search bar anatomy: container, leading icon, hinted text, trailing actions, input text, and the suggestions or results container",
    );
    gap(Dp(32.0));
    subheading("Examples".into());
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
        .placeholder("Hinted search text")
        .modifier(Modifier::new().width(Dp(400.0)))
        .leading_icon(|| {
            icon_button()
                .standard()
                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                .icon(filled::MENU_SVG)
                .accessibility_label("Search menu");
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
                            .accessibility_label("Voice search");
                    }
                    if kind == 2 {
                        icon_button()
                            .standard()
                            .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                            .icon(filled::SEARCH_SVG)
                            .accessibility_label("Submit search");
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
                .vector(filled::PERSON_SVG)
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
                    label: Some(
                        if active {
                            "Focused search phone"
                        } else {
                            "Unfocused search phone"
                        }
                        .into(),
                    ),
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
                        .children(|| {
                            text()
                                .content("9:30")
                                .style(type_style(14.0, 20.0))
                                .modifier(Modifier::new().weight(1.0));
                            icon().vector(filled::WIFI_SVG).size(Dp(16.0));
                            icon()
                                .vector(filled::SIGNAL_CELLULAR_4_BAR_SVG)
                                .size(Dp(16.0));
                            icon().vector(filled::BATTERY_FULL_SVG).size(Dp(16.0));
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
            value.set_query("Search query");
        }
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    search_bar()
        .controller(controller)
        .placeholder("Hinted search text")
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
                    .accessibility_label("Close search")
                    .on_click(move || controller.with_mut(SearchBarController::close));
            } else {
                boxed().modifier(watch(geometry, "leading")).children(|| {
                    icon().vector(filled::MENU_SVG).size(Dp(24.0));
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
                                    .accessibility_label("Clear search")
                                    .on_click(move || {
                                        controller.with_mut(|value| value.set_query(""))
                                    });
                            }
                            icon_button()
                                .standard()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .icon(filled::AUTO_AWESOME_SVG)
                                .accessibility_label("Search suggestions");
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
        "Keep the current theme's surface and content roles. The search field uses surface container high; full-screen results use surface container low. Docked results use surface container high.",
    );
    gap(Dp(24.0));
    live_search(true, false, false);
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Search bar".into(), "Surface container high".into()],
            vec!["Input text".into(), "On surface".into()],
            vec!["Hint and icons".into(), "On surface variant".into()],
            vec!["Full-screen results".into(), "Surface container low".into()],
            vec!["Docked results".into(), "Surface container high".into()],
        ],
    );
}

#[tessera]
fn states() {
    subheading("Search bar".into());
    prose(
        "Enabled, hovered, focused and pressed. Interact with the field to inspect its live state layers.",
    );
    gap(Dp(24.0));
    live_search(false, false, false);
    gap(Dp(24.0));
    subheading("Search suggestions & results".into());
    prose(
        "Individual result items retain their own hover, focus and pressed states while search is active.",
    );
    gap(Dp(24.0));
    live_search(true, false, false);
}

#[tessera]
fn measurements() {
    subheading("Search bar".into());
    search_figure(true);
    gap(Dp(24.0));
    tokens();
    gap(Dp(32.0));
    subheading("Focused search".into());
    prose(
        "The contained style keeps a 56dp bar and expands its outer margins from 24dp to 12dp when focused. Docked results occupy at least 240dp and at most two-thirds of the window height.",
    );
    gap(Dp(24.0));
    layout_examples();
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Attribute".into(), "Value".into()],
        vec![
            vec![
                "Full-screen container".into(),
                "Width / height".into(),
                "Full screen".into(),
            ],
            vec![
                "Docked container".into(),
                "Width".into(),
                "360–720dp".into(),
            ],
            vec![
                "Docked container".into(),
                "Height".into(),
                "240dp min; 2/3 screen max".into(),
            ],
            vec![
                "Focused bar".into(),
                "Leading / trailing padding".into(),
                "16dp".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose(
        "Use the token table to compare the search bar set with the search view set. The search bar set only contains tokens for the unfocused search bar. The search view set contains all other tokens when interacting with search, including all styles and layouts.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "md.comp.search-bar.container.color".into(),
                "Surface container high".into(),
            ],
            vec![
                "md.comp.search-bar.container.shape".into(),
                "Full (28dp radius)".into(),
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
        vec!["Element".into(), "Attribute".into(), "Value".into()],
        vec![
            vec!["Container".into(), "Width".into(), "360–720dp".into()],
            vec!["Container".into(), "Height".into(), "56dp".into()],
            vec![
                "Container".into(),
                "Shape".into(),
                "Full (28dp radius)".into(),
            ],
            vec![
                "Container".into(),
                "Outer margins".into(),
                "Unfocused 24dp; focused 12dp".into(),
            ],
            vec!["Label".into(), "Alignment".into(), "Start".into()],
            vec![
                "Icon and label".into(),
                "Gap from tap target".into(),
                "4dp".into(),
            ],
            vec!["Avatar".into(), "Size".into(), "30dp".into()],
        ],
    );
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Action".into()],
        vec![
            vec![
                "Tab / Shift + Tab".into(),
                "Navigate between controls".into(),
            ],
            vec!["Space or Enter".into(), "Activate a search control".into()],
            vec!["Enter in the field".into(), "Execute the query".into()],
        ],
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::time::Duration;

    use tessera_components::{
        modifier::MinimumInteractiveComponentEnforcement, theme::material_theme,
    };
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        provide_context,
    };

    use super::*;

    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

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
            .find(|(_, node)| node.label() == Some(label))
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
            provide_context(
                || MinimumInteractiveComponentEnforcement { enabled: false },
                || {
                    bar_example(2);
                },
            );
        });
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn search_anatomy_keeps_two_full_trailing_targets_and_centered_input() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(trailing_targets_fixture),
            vec![Box::new(Pipelines)],
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
                .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
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
            vec![Box::new(Pipelines)],
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
                    && node.label() == Some("Search your messages"))
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
                .any(|(_, node)| node.label() == Some("Weekend plans")
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
                .any(|(_, node)| node.label() == Some("Dinner with friends"))
        );
        click(&mut renderer, "Weekend plans");
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::List
                    && node.label() == Some("1 search results"))
        );
        click(&mut renderer, "Clear search");
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Dinner with friends")),
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
                    && node.label() == Some("4 search suggestions"))
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
            vec![Box::new(Pipelines)],
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
                .find(|(_, node)| node.label() == Some(label))
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
