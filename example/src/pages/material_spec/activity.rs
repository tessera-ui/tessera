//! Loading and progress specifications with live task states.
//!
//! ## Usage
//!
//! Compare short waits, known completion and ongoing progress.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    loading_indicator::loading_indicator,
    modifier::ModifierExt as _,
    progress::{circular_progress_indicator, linear_progress_indicator},
    pull_refresh::{PullRefreshController, pull_refresh},
    scrollable::ScrollableController,
    shape_def::Shape,
    slider::slider,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    wavy_progress::{circular_wavy_progress_indicator, linear_wavy_progress_indicator},
};
use tessera_ui::{Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use super::{
    ACCENT,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn activity_spec(loading: bool) {
    article(
        if loading {
            "Loading indicator"
        } else {
            "Progress indicators"
        },
        if loading {
            "A morphing shape for a short wait while content arrives."
        } else {
            "Show a task's completion or ongoing activity in linear and circular forms."
        },
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(move || {
                    variants(loading);
                }),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(move || {
                    configurations(loading);
                }),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(move || {
                    tokens(loading);
                }),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(move || {
                    anatomy(loading);
                }),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(move || {
                    colors(loading);
                }),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(move || {
                    measurements(loading);
                }),
            },
        ],
    ).pages(vec![
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(move || {
            prose(if loading { "Use a loading indicator for a short indeterminate process, including refreshing content. Keep it tied to an ongoing activity." } else { "Use linear or circular indicators for ongoing processes. Keep the same variant for the same process throughout the product." });
            gap(Dp(32.0)); variants(loading);
        })), sections: if loading { vec![
            section("M3 Expressive update", RenderSlot::new(|| {
                prose("Introduced in May 2025, loading indicators use seven morphing shapes to show short waits. They replace most indeterminate circular indicators for processes lasting under five seconds.");
                gap(Dp(24.0)); loading_pair("Expressive", false);
            })),
        ] } else { vec![
            section("M3 Expressive update", RenderSlot::new(|| {
                prose("The August 2024 update adds configurable track height and a wavy active indicator. Choose a shape and thickness that fit the product."); gap(Dp(24.0)); progress_pair("Expressive", true, true, false).value(0.5);
            })),
            section("Previous updates", RenderSlot::new(|| {
                prose("The December 2023 refresh adds an end stop, clearer contrast between active indicator and track, new motion, and rounded corners."); gap(Dp(24.0)); activity_figure(false,false);
            })),
            section("Differences from M2", RenderSlot::new(|| {
                prose("Material 3 progress indicators use dynamic color roles and rounded geometry. The active indicator and end stop remain distinguishable from the track."); gap(Dp(24.0)); colors(false);
            })),
        ] } },
        ArticlePage { title: "Guidelines", intro: None, sections: if loading { vec![
            section("Usage",RenderSlot::new(|| { wait_times(); prose("Use loading indicators when completion cannot be measured. Use one progress indicator that can become determinate when the process later reveals its progress; do not transition a loading indicator into a progress indicator."); gap(Dp(24.0)); loading_pair("Short wait",false); })),
            section("Anatomy",RenderSlot::new(|| { activity_figure(true,false); gap(Dp(24.0)); subheading("Active indicator".into()); prose("Seven Material shapes form a continuous morph sequence."); gap(Dp(24.0)); subheading("Container (optional)".into()); prose("Use a container over existing content and during pull-to-refresh. The active shape uses on-primary-container instead of primary."); })),
            section("Placement",RenderSlot::new(|| { prose("Center the indicator in the page or container that is loading. When appending content, put it in the empty area where new items will appear. It may also replace the content of a button during a short action."); gap(Dp(24.0)); refresh_example(); })),
            section("Responsive layout",RenderSlot::new(|| { prose("The default is 48dp. Scale from 24dp to 240dp while preserving the proportion of the shape and container."); gap(Dp(24.0)); loading_sizes(); gap(Dp(24.0)); subheading("Larger windows".into()); prose("Reserve very large indicators for large windows and keep their size proportional to the empty space."); })),
            section("Behavior",RenderSlot::new(|| { subheading("Pull-to-refresh".into()); prose("Use a contained loading indicator for refresh and keep it visible until the refresh completes or the user leaves the content."); gap(Dp(24.0)); refresh_example(); gap(Dp(24.0)); subheading("Threshold requirements".into()); prose("A refresh gesture must pass its activation threshold. Reversing past the threshold cancels the request; a completed gesture starts the refresh."); })),
        ] } else { vec![
            section("Usage",RenderSlot::new(|| { prose("Show overall progress for a group of items. Linear indicators fit a container edge; circular indicators fit its center. Determinate progress must report the actual completion fraction, while indeterminate progress communicates an unknown wait."); gap(Dp(24.0)); wait_times(); gap(Dp(24.0)); configurations(false); })),
            section("Anatomy",RenderSlot::new(|| { activity_figure(false,false); gap(Dp(24.0)); subheading("Active indicator".into()); prose("The active indicator shows completed work, or repeatedly grows and shrinks while progress is unknown. Flat and wavy shapes share the same meaning."); gap(Dp(24.0)); subheading("Stop indicator".into()); prose("The 4dp end stop marks the end of a determinate linear track. Keep it whenever the track contrasts below 3:1 against its surrounding surface."); })),
            section("Placement",RenderSlot::new(|| { prose("Place linear progress on the loading container's edge and circular progress at its center. When adding more items, reserve empty space for the new content instead of covering existing items."); gap(Dp(24.0)); progress_pair("Placement",false,false,false).value(0.5); gap(Dp(24.0)); subheading("Progress indicators in buttons".into()); prose("Use a small flat circular indicator in a button. Match the label color and remove the track to retain at least 3:1 contrast."); gap(Dp(24.0)); progress_button(); })),
            section("Responsive layout",RenderSlot::new(|| { subheading("Right-to-left languages".into()); prose("Mirror linear progress for right-to-left languages. Circular indicators keep their clockwise direction."); gap(Dp(24.0)); subheading("Large screens".into()); prose("Circular indicators can range from 24dp to 240dp. Linear indicators span their container, with at least 4dp at each end and a minimum length of 40dp."); gap(Dp(24.0)); activity_figure(false,true); })),
        ] } },
        ArticlePage { title: "Accessibility", intro: None, sections: vec![
            section("Use cases",RenderSlot::new(move || { prose(if loading { "Assistive technology must identify the loading activity. Provide a refresh action that does not require a swipe." } else { "Assistive technology must identify the ongoing process and understand its progress." }); gap(Dp(24.0)); if loading { refresh_example(); } else { configurations(false); } })),
            section("Interaction & style",RenderSlot::new(move || { prose("The active indicator must provide at least 3:1 contrast against its background. When embedded in a button, match its icon or text color. The container itself does not require that contrast."); gap(Dp(24.0)); if loading { colors(true); } else { progress_button(); gap(Dp(24.0)); prose("Retain the linear end stop whenever the track or adjacent container cannot clearly identify the end at 3:1 contrast."); } })),
            section("Labeling elements",RenderSlot::new(move || { prose("Use the progress bar role and a label describing both the activity and affected content, such as Loading news article or Refreshing page."); gap(Dp(24.0)); if loading { loading_indicator().accessibility_label("Loading news article"); } else { linear_progress_indicator().progress(0.5).accessibility_label("Loading news article"); } })),
        ] },
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn prose(#[prop(into)] content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn wait_times() {
    spec_table(
        vec!["Expected wait time".into(), "Recommendation".into()],
        vec![
            vec!["Under 200ms".into(), "Display content immediately".into()],
            vec!["200ms to 5s".into(), "Loading indicator".into()],
            vec!["Over 5s".into(), "Progress indicator".into()],
        ],
    );
}

#[tessera]
fn refresh_example() {
    let refreshing = remember(|| false);
    let controller = remember(PullRefreshController::new);
    let scroll = remember(ScrollableController::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .text()
                .accessibility_label("Refresh example content")
                .on_click(move || refreshing.set(!refreshing.get()))
                .child(move || {
                    text().content(if refreshing.get() {
                        "Finish refresh"
                    } else {
                        "Refresh"
                    });
                });
            gap(Dp(24.0));
            pull_refresh()
                .modifier(Modifier::new().fill_max_width().height(Dp(240.0)))
                .controller(controller)
                .refreshing(refreshing.get())
                .indicator_size(Dp(48.0))
                .on_refresh(move || refreshing.set(true))
                .indicator(move || {
                    loading_indicator()
                        .contained(true)
                        .paused(!refreshing.get())
                        .accessibility_label("Refreshing page");
                })
                .child(move || {
                    boxed()
                        .modifier(Modifier::new().fill_max_size().vertical_scroll(scroll))
                        .children(move || {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
                                .children(move || {
                                    column().children(move || {
                                        text().content("Pull down to refresh");
                                        gap(Dp(16.0));
                                        text().content(if refreshing.get() {
                                            "Refreshing content"
                                        } else {
                                            "Content is up to date"
                                        });
                                    });
                                });
                        });
                });
        });
}

#[tessera]
fn loading_sizes() {
    flow_row()
        .item_spacing(Dp(32.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            for size in [24.0, 48.0, 96.0, 240.0] {
                loading_indicator()
                    .paused(true)
                    .modifier(Modifier::new().size(Dp(size), Dp(size)))
                    .accessibility_label(format!("{size:.0}dp loading indicator"));
            }
        });
}

#[tessera]
fn progress_button() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    button()
        .filled()
        .accessibility_label("Saving updates")
        .on_click(|| {})
        .child(move || {
            circular_progress_indicator()
                .diameter(Dp(24.0))
                .color(scheme.on_primary)
                .track_color(tessera_ui::Color::TRANSPARENT)
                .accessibility_label("Saving updates");
        });
}

#[tessera]
fn progress_pair(
    #[prop(into)] label: String,
    value: Option<f32>,
    wavy: bool,
    thick: bool,
    paused: bool,
) {
    let stroke = Dp(if thick { 8.0 } else { 4.0 });
    let circle = Dp(if wavy {
        if thick { 52.0 } else { 48.0 }
    } else {
        40.0
    });
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            if wavy {
                linear_wavy_progress_indicator()
                    .modifier(Modifier::new().fill_max_width().height(Dp(if thick {
                        14.0
                    } else {
                        10.0
                    })))
                    .progress_optional(value)
                    .stroke_width(stroke)
                    .track_stroke_width(stroke)
                    .paused(paused)
                    .accessibility_label(format!("{label} linear progress"));
                gap(Dp(32.0));
                circular_wavy_progress_indicator()
                    .modifier(Modifier::new().size(circle, circle))
                    .progress_optional(value)
                    .stroke_width(stroke)
                    .track_stroke_width(stroke)
                    .paused(paused)
                    .accessibility_label(format!("{label} circular progress"));
            } else {
                linear_progress_indicator()
                    .modifier(Modifier::new().fill_max_width().height(stroke))
                    .progress_optional(value)
                    .accessibility_label(format!("{label} linear progress"));
                gap(Dp(32.0));
                circular_progress_indicator()
                    .diameter(circle)
                    .stroke_width(stroke)
                    .progress_optional(value)
                    .accessibility_label(format!("{label} circular progress"));
            }
        });
}

#[tessera]
fn loading_pair(#[prop(into)] label: String, paused: bool, contained: Option<bool>) {
    flow_row()
        .item_spacing(Dp(48.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            loading_indicator()
                .contained(contained.unwrap_or(false))
                .paused(paused)
                .accessibility_label(format!("{label} loading"));
            if contained.is_none() {
                loading_indicator()
                    .contained(true)
                    .paused(paused)
                    .accessibility_label(format!("{label} contained loading"));
            }
        });
}

#[tessera]
fn variants(loading: bool) {
    panel(RenderSlot::new(move || {
        if loading {
            loading_pair("Variant", false).contained(false);
        } else {
            progress_pair("Variant", false, false, false).value(0.5);
        }
    }));
}

#[tessera]
fn configurations(loading: bool) {
    let paused = remember(|| false);
    let contained = remember(|| false);
    let wavy = remember(|| false);
    let thick = remember(|| false);
    let unknown = remember(|| false);
    let value = remember(|| 0.5_f32);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            if loading {
                button()
                    .tonal()
                    .selected(contained.get())
                    .accessibility_label("Contained loading")
                    .on_click(move || contained.set(!contained.get()))
                    .child(|| {
                        text().content("Contained");
                    });
                button()
                    .tonal()
                    .selected(paused.get())
                    .accessibility_label("Pause loading animation")
                    .on_click(move || paused.set(!paused.get()))
                    .child(|| {
                        text().content("Pause");
                    });
            } else {
                button()
                    .tonal()
                    .selected(unknown.get())
                    .accessibility_label("Indeterminate progress")
                    .on_click(move || unknown.set(!unknown.get()))
                    .child(|| {
                        text().content("Indeterminate");
                    });
                button()
                    .tonal()
                    .selected(wavy.get())
                    .accessibility_label("Wavy progress")
                    .on_click(move || wavy.set(!wavy.get()))
                    .child(|| {
                        text().content("Wavy");
                    });
                button()
                    .tonal()
                    .selected(thick.get())
                    .accessibility_label("Thick progress")
                    .on_click(move || thick.set(!thick.get()))
                    .child(|| {
                        text().content("Thicker");
                    });
                button()
                    .tonal()
                    .selected(paused.get())
                    .enabled(wavy.get())
                    .accessibility_label("Pause wave animation")
                    .on_click(move || paused.set(!paused.get()))
                    .child(|| {
                        text().content("Pause wave");
                    });
                button()
                    .text()
                    .accessibility_label("Advance progress")
                    .on_click(move || value.set((value.get() + 0.25).min(1.0)))
                    .child(|| {
                        text().content("Advance 25%");
                    });
                button()
                    .text()
                    .accessibility_label("Reset progress")
                    .on_click(move || value.set(0.0))
                    .child(|| {
                        text().content("Reset");
                    });
            }
        });
    gap(Dp(32.0));
    panel(RenderSlot::new(move || {
        if loading {
            loading_pair("Configured", paused.get()).contained(contained.get());
        } else {
            progress_pair("Configured", wavy.get(), thick.get(), paused.get()).value_optional(
                if unknown.get() {
                    None
                } else {
                    Some(value.get())
                },
            );
        }
    }));
    if !loading {
        gap(Dp(24.0));
        slider()
            .modifier(Modifier::new().fill_max_width())
            .value(value.get())
            .on_change(move |v| value.set(v))
            .accessibility_label("Task completion");
        text()
            .content(if unknown.get() {
                "Task progress: indeterminate".into()
            } else {
                format!("Task progress: {:.0}%", value.get() * 100.0)
            })
            .style(type_style(16.0, 24.0));
    }
}

#[tessera]
fn tokens(loading: bool) {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        if loading {
            vec![
                vec!["Default active shape".into(), "Primary".into()],
                vec![
                    "Contained active shape".into(),
                    "On primary container".into(),
                ],
                vec!["Container".into(), "Primary container".into()],
                vec!["Component / shape container".into(), "48 / 38dp".into()],
            ]
        } else {
            vec![
                vec!["Active indicator and stop".into(), "Primary".into()],
                vec!["Track".into(), "Secondary container".into()],
                vec!["Default stroke / track gap".into(), "4 / 4dp".into()],
                vec!["Linear stop diameter".into(), "4dp".into()],
            ]
        },
    );
}

#[tessera]
fn anatomy(loading: bool) {
    activity_figure(loading, false);
    gap(Dp(24.0));
    text()
        .content(if loading {
            "1  Active indicator    2  Optional container"
        } else {
            "1  Active indicator    2  Track    3  Linear stop indicator"
        })
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn colors(loading: bool) {
    for dark in [false, true] {
        subheading(if dark {
            "Dark scheme".into()
        } else {
            "Light scheme".into()
        });
        gap(Dp(24.0));
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                surface()
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(32.0)))
                            .children(move || {
                                if loading {
                                    loading_pair(if dark { "Dark" } else { "Light" }, true);
                                } else {
                                    progress_pair(
                                        if dark { "Dark" } else { "Light" },
                                        true,
                                        false,
                                        true,
                                    )
                                    .value(0.5);
                                }
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn measurements(loading: bool) {
    activity_figure(loading, true);
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        if loading {
            vec![
                vec!["Component size".into(), "48 × 48dp".into()],
                vec!["Shape container".into(), "38 × 38dp".into()],
                vec!["Outer space".into(), "5dp per side".into()],
            ]
        } else {
            vec![
                vec!["Flat circular diameter".into(), "40dp".into()],
                vec!["Wavy circular container".into(), "48dp".into()],
                vec!["Default stroke".into(), "4dp".into()],
                vec!["Wavy linear height".into(), "10dp".into()],
                vec![
                    "Linear wavelength · determinate / indeterminate".into(),
                    "40 / 20dp".into(),
                ],
                vec!["Circular wavelength".into(), "15dp".into()],
                vec!["Screen edge inset".into(), "4dp".into()],
            ]
        },
    );
}

#[tessera]
fn activity_figure(loading: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|r| Dp::from_pixels_f32(r.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let height = width * (600.0 / 2048.0);
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            if measurements {
                activity_grid(Dp(width), Dp(height));
            }
            if loading {
                if measurements {
                    for (contained, x) in [(false, 0.413), (true, 0.577)] {
                        loading_indicator()
                            .contained(contained)
                            .paused(true)
                            .modifier(
                                watch(geometry, if contained { "loading" } else { "plain" })
                                    .offset(Dp(width * x - 24.0), Dp(height * 0.48 - 24.0)),
                            )
                            .accessibility_label(if contained {
                                "Anatomy contained loading"
                            } else {
                                "Anatomy loading"
                            });
                    }
                } else {
                    loading_indicator()
                        .contained(true)
                        .paused(true)
                        .modifier(watch(geometry, "loading").align(Alignment::Center))
                        .accessibility_label("Anatomy contained loading");
                }
            } else {
                linear_wavy_progress_indicator()
                    .progress(0.5)
                    .paused(true)
                    .modifier(
                        watch(geometry, "linear")
                            .width(Dp(width * 0.392))
                            .height(Dp(10.0))
                            .offset(Dp(width * 0.183), Dp(height * 0.397 - 5.0)),
                    )
                    .accessibility_label("Anatomy linear progress");
                circular_wavy_progress_indicator()
                    .progress(0.5)
                    .paused(true)
                    .modifier(
                        watch(geometry, "circular")
                            .size(Dp(48.0), Dp(48.0))
                            .offset(Dp(width * 0.795 - 24.0), Dp(height * 0.5 - 24.0)),
                    )
                    .accessibility_label("Anatomy circular progress");
            }
            activity_annotations(geometry, loading, measurements);
        });
}

#[tessera]
fn activity_annotations(geometry: State<Geometry>, loading: bool, measurements: bool) {
    let values = geometry.get();
    let Some(root) = values.get("figure") else {
        return;
    };
    let mut svg = String::new();
    if loading {
        let Some(rect) = relative(&values, "loading") else {
            return;
        };
        if measurements {
            for name in ["plain", "loading"] {
                let Some(rect) = relative(&values, name) else {
                    continue;
                };
                annotations::outline(&mut svg, rect);
                let shape = annotations::Rect {
                    x: rect.x + 5.0,
                    y: rect.y + 5.0,
                    w: rect.w - 10.0,
                    h: rect.h - 10.0,
                };
                svg.push_str(&format!("<path d='M{},{}h{}v5h-{}zM{},{}h{}v5h-{}zM{},{}h5v{}h-5zM{},{}h5v{}h-5z' fill='#e883aa' fill-opacity='.32'/>",rect.x,rect.y,rect.w,rect.w,rect.x,rect.y+rect.h-5.0,rect.w,rect.w,rect.x,rect.y,rect.h,rect.x+rect.w-5.0,rect.y,rect.h));
                annotations::outline(&mut svg, shape);
                dotted(
                    &mut svg,
                    rect.x + rect.w + 8.0,
                    shape.y,
                    shape.h,
                    true,
                    "#b14e70",
                );
                dotted(
                    &mut svg,
                    rect.x,
                    rect.y + rect.h + 8.0,
                    rect.w,
                    false,
                    "#b14e70",
                );
                annotations::dimension("38".into(), Dp(rect.x + rect.w + 16.0), Dp(rect.y + 14.0));
                annotations::dimension(
                    format!("{:.0}", rect.w),
                    Dp(rect.x + 12.0),
                    Dp(rect.y + rect.h + 16.0),
                );
            }
        } else {
            let y = rect.y + rect.h * 0.5;
            dotted(
                &mut svg,
                rect.x - 24.0,
                y,
                rect.w * 0.3 + 24.0,
                false,
                "#49454f",
            );
            dotted(&mut svg, rect.x + rect.w, y, 32.0, false, "#49454f");
            annotations::marker(1, Dp(rect.x - 48.0), Dp(y - 16.0));
            annotations::marker(2, Dp(rect.x + rect.w + 32.0), Dp(y - 16.0));
        }
    } else {
        let (Some(line), Some(circle)) =
            (relative(&values, "linear"), relative(&values, "circular"))
        else {
            return;
        };
        if measurements {
            annotations::outline(&mut svg, line);
            annotations::outline(&mut svg, circle);
            annotations::dimension("4".into(), Dp(line.x + 24.0), Dp(line.y - 36.0));
            annotations::dimension("4".into(), Dp(line.x + line.w * 0.5), Dp(line.y + 28.0));
            annotations::dimension(
                format!("{:.0}", circle.w),
                Dp(circle.x),
                Dp(circle.y + circle.h + 32.0),
            );
            dotted(
                &mut svg,
                circle.x,
                circle.y + circle.h + 24.0,
                circle.w,
                false,
                "#b14e70",
            );
        } else {
            for (number, x) in [
                (1, line.x + line.w * 0.18),
                (2, line.x + line.w * 0.65),
                (3, line.x + line.w - 2.0),
            ] {
                dotted(&mut svg, x, line.y + line.h * 0.5, 38.0, true, "#49454f");
                annotations::marker(number, Dp(x - 12.0), Dp(line.y + line.h * 0.5 + 38.0));
            }
            let x = circle.x + circle.w * 0.65;
            dotted(&mut svg, x, circle.y - 36.0, 36.0, true, "#49454f");
            annotations::marker(1, Dp(x - 12.0), Dp(circle.y - 68.0));
            dotted(
                &mut svg,
                circle.x + circle.w * 0.3,
                circle.y + circle.h,
                36.0,
                true,
                "#49454f",
            );
            annotations::marker(
                2,
                Dp(circle.x + circle.w * 0.3 - 12.0),
                Dp(circle.y + circle.h + 36.0),
            );
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn activity_grid(width: Dp, height: Dp) {
    let mut path = String::new();
    for row in 1..(height.0 / 8.0).ceil() as usize {
        for column in 1..(width.0 / 8.0).ceil() as usize {
            let x = column as f64 * 8.0 + 2.0;
            let y = row as f64 * 8.0 + 2.0;
            path.push_str(&format!("M{x},{y}h0.6v0.6h-0.6z"));
        }
    }
    annotations::lines(
        width,
        height,
        format!("<path d='{path}' fill='#79747e' fill-opacity='.6'/>"),
    );
}
