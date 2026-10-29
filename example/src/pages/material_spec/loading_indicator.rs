//! Loading indicator specifications.
//!
//! ## Usage
//!
//! Compare indeterminate morphs, determinate progress and contained styles.

use tessera_components::{
    column::column,
    flow_row::flow_row,
    loading_indicator::loading_indicator,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, tessera, use_context};

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn loading_indicator_spec() {
    article(
        "Loading indicator",
        "Loading indicators show the progress of a process for a short wait time",
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(colors),
            },
            SpecSection {
                title: "States",
                content: RenderSlot::new(states),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(guidelines_anatomy),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive_layout),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text()
        .content(value)
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
                        text().content(item.clone()).style(type_style(16.0, 24.0));
                    });
            }
        });
}

/// Renders one loading indicator with optional configuration overrides.
#[tessera]
fn spinner(
    #[prop(into)] label: String,
    progress: Option<f32>,
    contained: Option<bool>,
    paused: Option<bool>,
    indicator_color: Option<Color>,
    container_color: Option<Color>,
) {
    loading_indicator()
        .progress_optional(progress)
        .contained(contained.unwrap_or(false))
        .paused(paused.unwrap_or(false))
        .indicator_color_optional(indicator_color)
        .container_color_optional(container_color)
        .accessibility_label(label);
}

#[tessera]
fn overview() {
    prose(
        "Loading indicators show the progress of a process for a short wait time. Use the indeterminate morph for brief, unknown waits, and a determinate value when completion is measurable.",
    );
    gap(Dp(16.0));
    bullets(vec![
        "Best for short, indeterminate wait times".into(),
        "Include loading indicators in a layout rather than blocking the whole screen".into(),
        "Show determinate progress when the wait can be measured".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        spinner("Loading content");
    }));
    gap(Dp(16.0));
    prose("1  Loading indicator");
}

#[tessera]
fn resources() {
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
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Flutter".into(),
                "Unavailable".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native loading_indicator component. The indeterminate morph, determinate progress, the contained style, a paused state, and an accessibility label are implemented.",
    );
}

#[tessera]
fn expressive() {
    prose(
        "May 2025: the loading indicator replaces the circular progress indicator for short, indeterminate waits. It animates a single morphing shape instead of a rotating arc, and adds a contained style with its own container color.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Change".into(), "Details".into()],
        vec![
            vec![
                "Shape".into(),
                "A morphing shape instead of a rotating arc".into(),
            ],
            vec![
                "Containment".into(),
                "A contained style with primary container color".into(),
            ],
            vec![
                "Determinate".into(),
                "The same shape can express a measured completion".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row().item_spacing(Dp(48.0)).children(|| {
            spinner("Default loading indicator");
            spinner("Contained loading indicator").contained(true);
        });
    }));
    caption("Default and contained styles share the same morph.");
}

#[tessera]
fn differences() {
    prose(
        "M2: a circular progress indicator rotated a stroke arc and had no contained style. M3 keeps the circular shape but pairs it with the expressive loading indicator, which morphs a solid shape and can sit in a container.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "M2".into(), "M3 Expressive".into()],
        vec![
            vec![
                "Shape".into(),
                "Rotating arc".into(),
                "Morphing shape".into(),
            ],
            vec![
                "Size".into(),
                "40dp".into(),
                "48dp target, 38dp shape".into(),
            ],
            vec![
                "Containment".into(),
                "None".into(),
                "Default or contained".into(),
            ],
            vec![
                "Determinate".into(),
                "Separate component".into(),
                "Same component with a progress value".into(),
            ],
        ],
    );
}

#[tessera]
fn variants() {
    prose(
        "Loading indicators have a single variant. Choose containment and a determinate value as configurations, not as separate components.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        spinner("Loading indicator");
    }));
    gap(Dp(16.0));
    caption("1 Loading indicator");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![vec![
            "Loading indicator".into(),
            "--".into(),
            "Available".into(),
        ]],
    );
}

#[tessera]
fn configurations() {
    prose(
        "Configure containment, a determinate progress value, and a paused state. Pausing is useful for reduced-motion settings and keeps the shape visible without animating.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("1  Containment".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("Default loading indicator");
                        spinner("Contained loading indicator").contained(true);
                    });
                gap(Dp(40.0));
                subheading("2  Progress".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("Indeterminate progress");
                        spinner("Loading 30%").progress(0.3);
                        spinner("Loading 70%").progress(0.7);
                    });
                gap(Dp(40.0));
                subheading("3  Paused".into());
                gap(Dp(24.0));
                flow_row().item_spacing(Dp(48.0)).children(|| {
                    spinner("Paused loading indicator").paused(true);
                    spinner("Paused contained indicator")
                        .paused(true)
                        .contained(true);
                });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec![
            "Configuration".into(),
            "Options".into(),
            "M3 Expressive".into(),
        ],
        vec![
            vec![
                "Containment".into(),
                "Default, contained".into(),
                "Available".into(),
            ],
            vec![
                "Progress".into(),
                "Indeterminate, determinate 0–100%".into(),
                "Available".into(),
            ],
            vec![
                "Paused".into(),
                "Animating, paused".into(),
                "Available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose("Loading indicators have a single token set, grouped by color, size, and shape.");
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "md.comp.loading-indicator.indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.loading-indicator.contained.indicator.color".into(),
                "On primary container".into(),
            ],
            vec![
                "md.comp.loading-indicator.contained.container.color".into(),
                "Primary container".into(),
            ],
            vec![
                "md.comp.loading-indicator.size".into(),
                "48dp target".into(),
            ],
            vec!["md.comp.loading-indicator.shape.size".into(), "38dp".into()],
            vec![
                "md.comp.loading-indicator.morph.duration".into(),
                "297ms".into(),
            ],
            vec![
                "md.comp.loading-indicator.morph.interval".into(),
                "650ms between shapes".into(),
            ],
            vec![
                "md.comp.loading-indicator.rotation.duration".into(),
                "4666ms per revolution".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "The morph interval is longer than the morph duration so each shape settles before the next one starts, and the slow rotation keeps the indicator readable without drawing attention.",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "A loading indicator is a single shape inside a 48dp target. The contained style adds a circular container behind the shape.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column().children(|| {
            row()
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    spinner("Default loading indicator");
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    spinner("Contained loading indicator").contained(true);
                });
            gap(Dp(32.0));
            for line in ["1  Shape", "2  Container (contained only)"] {
                text()
                    .content(line)
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption("The shape carries the animation; the container is optional.");
}

#[tessera]
fn colors() {
    prose(
        "The default style uses the primary color on the surface behind it. The contained style pairs the primary container with its on-color, so the indicator keeps at least 3:1 contrast in both light and dark schemes.",
    );
    gap(Dp(24.0));
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
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                flow_row()
                                    .item_spacing(Dp(48.0))
                                    .line_spacing(Dp(24.0))
                                    .children(move || {
                                        spinner(if dark {
                                            "Dark default loading indicator"
                                        } else {
                                            "Light default loading indicator"
                                        });
                                        spinner(if dark {
                                            "Dark contained loading indicator"
                                        } else {
                                            "Light contained loading indicator"
                                        })
                                        .contained(true);
                                    });
                                gap(Dp(16.0));
                                text()
                                    .content("1  Primary    2  Primary container with on primary container")
                                    .style(type_style(14.0, 20.0));
                            });
                    });
            });
        gap(Dp(32.0));
    }
}

#[tessera]
fn states() {
    prose(
        "A loading indicator has three meaningful states rather than interaction states: it runs an indeterminate morph, shows a determinate value, or pauses. It never takes focus or responds to input.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("Running indeterminate indicator");
                        spinner("Determinate indicator at 60%").progress(0.6);
                        spinner("Paused indicator").paused(true);
                    });
                gap(Dp(24.0));
                text()
                    .content("1  Running    2  Determinate    3  Paused")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["State".into(), "Behavior".into()],
        vec![
            vec!["Running".into(), "Morphs and rotates until removed".into()],
            vec![
                "Determinate".into(),
                "Holds the shape for the given progress value".into(),
            ],
            vec![
                "Paused".into(),
                "Keeps the current shape without animating".into(),
            ],
            vec![
                "Contained".into(),
                "Adds the container without changing the state".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    prose(
        "To ensure sufficient margins, the size is 48dp while the shape container is 38dp. Keep the 48dp target even when the visible shape is smaller, and reserve it in the layout so surrounding content does not shift.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                spinner("Default loading indicator");
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                spinner("Contained loading indicator").contained(true);
            });
    }));
    gap(Dp(16.0));
    caption("Default and contained styles share the same 48dp target.");
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Target size".into(), "48dp".into()],
            vec!["Shape size".into(), "38dp".into()],
            vec!["Contained container size".into(), "38dp".into()],
            vec![
                "Minimum margins".into(),
                "Sufficient to keep the 48dp target clear".into(),
            ],
            vec![
                "Minimum interactive target".into(),
                "Not interactive".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "Use a loading indicator for a short, indeterminate wait while content arrives. Keep it inside the area that is loading instead of covering the whole screen, and choose a determinate value when the wait can be measured.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                spinner("Loading article");
                gap(Dp(16.0));
                text()
                    .content("Do: place the indicator in the region that is loading.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                spinner("Loading image gallery").contained(true);
                gap(Dp(16.0));
                text()
                    .content("Don't: use a contained indicator as a full-screen blocker.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("Loading indicators are best for indicating a short, indeterminate wait time.");
    gap(Dp(32.0));
    bullets(vec![
        "Prefer a determinate value for waits longer than a few seconds".into(),
        "Avoid stacking several indicators in the same view".into(),
        "Do not use the indicator to signal an error or completion".into(),
    ]);
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "The shape is the only required part. Add the container when the indicator sits on media or a busy background, and keep the 48dp target clear around it.",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn placement() {
    prose(
        "Place the indicator where the content will appear: inside a card, a list row, or the media area. For a page-level wait, center it in the content region and keep navigation visible.",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .style(scheme.surface_container_low.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(Modifier::new().fill_max_width().height(Dp(200.0)))
        .child(move || {
            column()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    text()
                        .content("Content region")
                        .color(scheme.on_surface_variant)
                        .style(type_style(14.0, 20.0));
                    spacer().modifier(Modifier::new().weight(1.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            spacer().modifier(Modifier::new().weight(1.0));
                            spinner("Loading content region");
                            spacer().modifier(Modifier::new().weight(1.0));
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                });
        });
    gap(Dp(16.0));
    caption("Center the indicator in the region that is waiting for content.");
}

#[tessera]
fn responsive_layout() {
    prose(
        "The indicator keeps its 48dp target at every window size. In compact layouts keep the surrounding padding smaller rather than shrinking the indicator, and in expanded layouts place it beside the content that is loading.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row().item_spacing(Dp(48.0)).children(|| {
            spinner("Compact loading indicator");
            spinner("Expanded loading indicator").contained(true);
        });
    }));
    gap(Dp(16.0));
    caption("The size does not change across window classes.");
}

#[tessera]
fn behavior() {
    prose(
        "An indeterminate indicator morphs every 650ms, each morph lasting 297ms, while completing a slow rotation every 4666ms. Stop animating when the wait ends, when the component leaves the layout, or when reduced motion is requested.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                flow_row().item_spacing(Dp(48.0)).children(|| {
                    spinner("Animating loading indicator");
                    spinner("Paused for reduced motion").paused(true);
                });
                gap(Dp(16.0));
                text()
                    .content(
                        "The paused example holds its current shape and stops requesting frames.",
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Phase".into(), "Duration".into()],
        vec![
            vec!["Morph between shapes".into(), "297ms".into()],
            vec!["Hold before the next morph".into(), "650ms".into()],
            vec!["Full rotation".into(), "4666ms".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Frames are only requested while the indicator is indeterminate and not paused; a determinate or paused indicator stops requesting frames, so a page that stops loading does not keep animating in the background.",
    );
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(16.0));
    bullets(vec![
        "Navigate to the loading indicator".into(),
        "Understand what progress the indicator is communicating".into(),
        "Initiate a content refresh without relying on a gesture".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                spinner("Loading content");
                gap(Dp(16.0));
                text()
                    .content("The indicator exposes the label “Loading content” to assistive technology.")
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn interaction_style() {
    prose(
        "The indicator is not interactive: it takes no focus and has no hover or pressed state. Keep at least 3:1 contrast between the active indicator and the surface behind it, including the contained style.",
    );
    gap(Dp(24.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        spinner("Above three to one loading indicator");
                        spacer().modifier(Modifier::new().width(Dp(48.0)));
                        loading_indicator()
                            .paused(true)
                            .indicator_color(scheme.surface_container)
                            .accessibility_label("Below three to one loading indicator");
                    });
                gap(Dp(16.0));
                text()
                    .content("Above 3:1 in the primary role versus a low-contrast pairing.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    prose(
        "Do not place the indicator over content that people are reading, and never use it as a full-screen blocker without a way to cancel the wait.",
    );
}

#[tessera]
fn labeling() {
    prose(
        "Name the wait, not the animation. Screen readers should announce what is loading, such as “Loading content”, instead of describing the morph. Keep the label stable while the indicator is visible.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                spinner("Loading content");
                gap(Dp(16.0));
                text()
                    .content("Do: “Loading content” — names the pending work.")
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                spinner("Loading").contained(true);
                gap(Dp(16.0));
                text()
                    .content("Don't: rely on the animation alone; a bare “Loading” without context is ambiguous.")
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "Update the label when the wait changes stage, and remove the indicator once the content arrives so assistive technology stops announcing a pending state.",
    );
}
