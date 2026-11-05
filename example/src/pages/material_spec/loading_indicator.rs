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
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{Dp, Modifier, RenderSlot, remember, tessera, use_context};

use crate::res::I18N;

use super::{
    ACCENT, INK,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn loading_indicator_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-loading-indicator",
        "ui-loading-indicators-show-the-progress-of-a-process-for-a-short-wait-time",
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
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "ui-availability-resources",
                    content: RenderSlot::new(resources),
                },
                SpecSection {
                    title: "ui-m3-expressive-update",
                    content: RenderSlot::new(expressive),
                },
                SpecSection {
                    title: "ui-differences-from-m2",
                    content: RenderSlot::new(differences),
                },
            ],
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
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(labeling),
                },
            ],
        },
    ]);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text()
        .content(I18N.text(value))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    gap(Dp(16.0));
    text()
        .content(I18N.text(value))
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
                        text()
                            .content(I18N.text(&(item)))
                            .style(type_style(16.0, 24.0));
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
) {
    loading_indicator()
        .progress_optional(progress)
        .contained(contained.unwrap_or(false))
        .paused(paused.unwrap_or(false))
        .accessibility_label(I18N.text(label));
}

#[tessera]
fn overview() {
    prose(
        "ui-loading-indicators-show-the-progress-of-a-process-for-a-short-wait-time-use-the-indetermin",
    );
    gap(Dp(16.0));
    bullets(vec![
        "ui-best-for-short-indeterminate-wait-times".into(),
        "ui-include-loading-indicators-in-a-layout-rather-than-blocking-the-whole-screen".into(),
        "ui-show-determinate-progress-when-the-wait-can-be-measured".into(),
    ]);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        spinner("ui-loading-content");
    }));
    gap(Dp(16.0));
    prose("ui-1-loading-indicator");
}

#[tessera]
fn resources() {
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
                "ui-jetpack-compose-expressive".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-flutter".into(),
                "ui-unavailable".into(),
            ],
            vec![
                "ui-implementation".into(),
                "ui-web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-tessera-examples-on-this-page-use-the-native-loading-indicator-component-the-indeterm",
    );
}

#[tessera]
fn expressive() {
    prose(
        "ui-may-2025-the-loading-indicator-replaces-the-circular-progress-indicator-for-short-indeterm",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-change".into(), "ui-details".into()],
        vec![
            vec![
                "ui-shape".into(),
                "ui-a-morphing-shape-instead-of-a-rotating-arc".into(),
            ],
            vec![
                "ui-containment".into(),
                "ui-a-contained-style-with-primary-container-color".into(),
            ],
            vec![
                "ui-determinate".into(),
                "ui-the-same-shape-can-express-a-measured-completion".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        flow_row().item_spacing(Dp(48.0)).children(|| {
            spinner("ui-default-loading-indicator");
            spinner("ui-contained-loading-indicator").contained(true);
        });
    }));
    caption("ui-default-and-contained-styles-share-the-same-morph");
}

#[tessera]
fn differences() {
    prose(
        "ui-m2-a-circular-progress-indicator-rotated-a-stroke-arc-and-had-no-contained-style-m3-keeps",
    );
    gap(Dp(24.0));
    spec_table(
        vec![
            "ui-attribute".into(),
            "M2".into(),
            "ui-m3-expressive".into(),
        ],
        vec![
            vec![
                "ui-shape".into(),
                "ui-rotating-arc".into(),
                "ui-morphing-shape".into(),
            ],
            vec![
                "ui-size".into(),
                "40dp".into(),
                "ui-48dp-target-38dp-shape".into(),
            ],
            vec![
                "ui-containment".into(),
                "ui-none".into(),
                "ui-default-or-contained".into(),
            ],
            vec![
                "ui-determinate".into(),
                "ui-separate-component".into(),
                "ui-same-component-with-a-progress-value".into(),
            ],
        ],
    );
}

#[tessera]
fn variants() {
    prose(
        "ui-loading-indicators-have-a-single-variant-choose-containment-and-a-determinate-value-as-con",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        spinner("ui-loading-indicator");
    }));
    gap(Dp(16.0));
    caption("ui-1-loading-indicator-3589");
    gap(Dp(32.0));
    spec_table(
        vec!["ui-variant".into(), "M3".into(), "ui-m3-expressive".into()],
        vec![vec![
            "ui-loading-indicator".into(),
            "--".into(),
            "ui-available".into(),
        ]],
    );
}

#[tessera]
fn configurations() {
    let contained = remember(|| false);
    let paused = remember(|| false);
    prose(
        "ui-configure-containment-a-determinate-progress-value-and-a-paused-state-pausing-is-useful-fo",
    );
    gap(Dp(24.0));
    spinner("ui-configured-loading")
        .contained(contained.get())
        .paused(paused.get());
    gap(Dp(16.0));
    flow_row().item_spacing(Dp(16.0)).children(move || {
        text()
            .content(I18N.text("ui-contained-loading"))
            .style(type_style(14.0, 20.0));
        switch()
            .checked(contained.get())
            .accessibility_label(I18N.text("ui-contained-loading"))
            .on_toggle(move |value| contained.set(value));
        text()
            .content(I18N.text("ui-pause-animation"))
            .style(type_style(14.0, 20.0));
        switch()
            .checked(paused.get())
            .accessibility_label(I18N.text("ui-pause-loading-animation"))
            .on_toggle(move |value| paused.set(value));
    });
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                subheading("ui-1-containment".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("ui-default-loading-indicator");
                        spinner("ui-contained-loading-indicator").contained(true);
                    });
                gap(Dp(40.0));
                subheading("ui-2-progress".into());
                gap(Dp(24.0));
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("ui-indeterminate-progress");
                        spinner("ui-loading-30").progress(0.3);
                        spinner("ui-loading-70").progress(0.7);
                    });
                gap(Dp(40.0));
                subheading("ui-3-paused".into());
                gap(Dp(24.0));
                flow_row().item_spacing(Dp(48.0)).children(|| {
                    spinner("ui-paused-loading-indicator").paused(true);
                    spinner("ui-paused-contained-indicator")
                        .paused(true)
                        .contained(true);
                });
            });
    }));
    gap(Dp(32.0));
    spec_table(
        vec![
            "ui-configuration".into(),
            "ui-options".into(),
            "ui-m3-expressive".into(),
        ],
        vec![
            vec![
                "ui-containment".into(),
                "ui-default-contained".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-progress".into(),
                "ui-indeterminate-determinate-0-100".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-paused".into(),
                "ui-animating-paused".into(),
                "ui-available".into(),
            ],
        ],
    );
}

#[tessera]
fn tokens() {
    prose("ui-loading-indicators-have-a-single-token-set-grouped-by-color-size-and-shape");
    gap(Dp(24.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "md.comp.loading-indicator.indicator.color".into(),
                "ui-primary".into(),
            ],
            vec![
                "md.comp.loading-indicator.contained.indicator.color".into(),
                "ui-on-primary-container".into(),
            ],
            vec![
                "md.comp.loading-indicator.contained.container.color".into(),
                "ui-primary-container".into(),
            ],
            vec![
                "md.comp.loading-indicator.size".into(),
                "ui-48dp-target".into(),
            ],
            vec!["md.comp.loading-indicator.shape.size".into(), "38dp".into()],
            vec![
                "md.comp.loading-indicator.morph.duration".into(),
                "297ms".into(),
            ],
            vec![
                "md.comp.loading-indicator.morph.interval".into(),
                "ui-650ms-between-shapes".into(),
            ],
            vec![
                "md.comp.loading-indicator.rotation.duration".into(),
                "ui-4666ms-per-revolution".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-the-morph-interval-is-longer-than-the-morph-duration-so-each-shape-settles-before-the-next",
    );
}

#[tessera]
fn anatomy() {
    prose(
        "ui-a-loading-indicator-is-a-single-shape-inside-a-48dp-target-the-contained-style-adds-a-circ",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column().children(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .children(|| {
                    spinner("ui-default-loading-indicator");
                    spacer().modifier(Modifier::new().width(Dp(48.0)));
                    spinner("ui-contained-loading-indicator").contained(true);
                });
            gap(Dp(32.0));
            for line in ["ui-1-shape", "ui-2-container-contained-only"] {
                text()
                    .content(I18N.text(line))
                    .color(INK)
                    .style(type_style(16.0, 28.0));
            }
        });
    }));
    gap(Dp(16.0));
    caption("ui-the-shape-carries-the-animation-the-container-is-optional");
}

#[tessera]
fn colors() {
    prose(
        "ui-the-default-style-uses-the-primary-color-on-the-surface-behind-it-the-contained-style-pair",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        subheading(if dark {
            "ui-dark-scheme".into()
        } else {
            "ui-light-scheme".into()
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
                                            "ui-dark-default-loading-indicator"
                                        } else {
                                            "ui-light-default-loading-indicator"
                                        });
                                        spinner(if dark {
                                            "ui-dark-contained-loading-indicator"
                                        } else {
                                            "ui-light-contained-loading-indicator"
                                        })
                                        .contained(true);
                                    });
                                gap(Dp(16.0));
                                text()
                                    .content(I18N.text("ui-1-primary-2-primary-container-with-on-primary-container"))
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
        "ui-a-loading-indicator-has-three-meaningful-states-rather-than-interaction-states-it-runs-an",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row()
                    .item_spacing(Dp(48.0))
                    .line_spacing(Dp(24.0))
                    .children(|| {
                        spinner("ui-running-indeterminate-indicator");
                        spinner("ui-determinate-indicator-at-60").progress(0.6);
                        spinner("ui-paused-indicator").paused(true);
                    });
                gap(Dp(24.0));
                text()
                    .content(I18N.text("ui-1-running-2-determinate-3-paused"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-state".into(), "ui-behavior".into()],
        vec![
            vec![
                "ui-running".into(),
                "ui-morphs-and-rotates-until-removed".into(),
            ],
            vec![
                "ui-determinate".into(),
                "ui-holds-the-shape-for-the-given-progress-value".into(),
            ],
            vec![
                "ui-paused".into(),
                "ui-keeps-the-current-shape-without-animating".into(),
            ],
            vec![
                "ui-contained".into(),
                "ui-adds-the-container-without-changing-the-state".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    prose(
        "ui-to-ensure-sufficient-margins-the-size-is-48dp-while-the-shape-container-is-38dp-keep-the-4",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        row()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                spinner("ui-default-loading-indicator");
                spacer().modifier(Modifier::new().width(Dp(48.0)));
                spinner("ui-contained-loading-indicator").contained(true);
            });
    }));
    gap(Dp(16.0));
    caption("ui-default-and-contained-styles-share-the-same-48dp-target");
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-target-size".into(), "48dp".into()],
            vec!["ui-shape-size".into(), "38dp".into()],
            vec!["ui-contained-container-size".into(), "38dp".into()],
            vec![
                "ui-minimum-margins".into(),
                "ui-sufficient-to-keep-the-48dp-target-clear".into(),
            ],
            vec![
                "ui-minimum-interactive-target".into(),
                "ui-not-interactive".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-use-a-loading-indicator-for-a-short-indeterminate-wait-while-content-arrives-keep-it-insid",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                spinner("ui-loading-article");
                gap(Dp(16.0));
                text()
                    .content(I18N.text("ui-do-place-the-indicator-in-the-region-that-is-loading"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                spinner("ui-loading-image-gallery").contained(true);
                gap(Dp(16.0));
                text()
                    .content(
                        I18N.text("ui-don-t-use-a-contained-indicator-as-a-full-screen-blocker"),
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(16.0));
    caption("ui-loading-indicators-are-best-for-indicating-a-short-indeterminate-wait-time");
    gap(Dp(32.0));
    bullets(vec![
        "ui-prefer-a-determinate-value-for-waits-longer-than-a-few-seconds".into(),
        "ui-avoid-stacking-several-indicators-in-the-same-view".into(),
        "ui-do-not-use-the-indicator-to-signal-an-error-or-completion".into(),
    ]);
}

#[tessera]
fn guidelines_anatomy() {
    prose(
        "ui-the-shape-is-the-only-required-part-add-the-container-when-the-indicator-sits-on-media-or",
    );
    gap(Dp(24.0));
    anatomy();
}

#[tessera]
fn placement() {
    prose(
        "ui-place-the-indicator-where-the-content-will-appear-inside-a-card-a-list-row-or-the-media-ar",
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
                        .content(I18N.text("ui-content-region"))
                        .color(scheme.on_surface_variant)
                        .style(type_style(14.0, 20.0));
                    spacer().modifier(Modifier::new().weight(1.0));
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            spacer().modifier(Modifier::new().weight(1.0));
                            spinner("ui-loading-content-region");
                            spacer().modifier(Modifier::new().weight(1.0));
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                });
        });
    gap(Dp(16.0));
    caption("ui-center-the-indicator-in-the-region-that-is-waiting-for-content");
}

#[tessera]
fn responsive_layout() {
    prose(
        "ui-the-indicator-keeps-its-48dp-target-at-every-window-size-in-compact-layouts-keep-the-surro",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        flow_row().item_spacing(Dp(48.0)).children(|| {
            spinner("ui-compact-loading-indicator");
            spinner("ui-expanded-loading-indicator").contained(true);
        });
    }));
    gap(Dp(16.0));
    caption("ui-the-size-does-not-change-across-window-classes");
}

#[tessera]
fn behavior() {
    prose(
        "ui-an-indeterminate-indicator-morphs-every-650ms-each-morph-lasting-297ms-while-completing-a",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                flow_row().item_spacing(Dp(48.0)).children(|| {
                    spinner("ui-animating-loading-indicator");
                    spinner("ui-paused-for-reduced-motion").paused(true);
                });
                gap(Dp(16.0));
                text()
                    .content(I18N.text(
                        "ui-the-paused-example-holds-its-current-shape-and-stops-requesting-frames",
                    ))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-phase".into(), "ui-duration".into()],
        vec![
            vec!["ui-morph-between-shapes".into(), "297ms".into()],
            vec!["ui-hold-before-the-next-morph".into(), "650ms".into()],
            vec!["ui-full-rotation".into(), "4666ms".into()],
        ],
    );
    gap(Dp(24.0));
    prose(
        "ui-frames-are-only-requested-while-the-indicator-is-indeterminate-and-not-paused-a-determinat",
    );
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-with-assistive-technology");
    gap(Dp(16.0));
    bullets(vec![
        "ui-navigate-to-the-loading-indicator".into(),
        "ui-understand-what-progress-the-indicator-is-communicating".into(),
        "ui-initiate-a-content-refresh-without-relying-on-a-gesture".into(),
    ]);
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                spinner("ui-loading-content");
                gap(Dp(16.0));
                text()
                    .content(I18N.text("ui-the-indicator-exposes-the-label-loading-content-to-assistive-technology"))
                    .style(type_style(14.0, 20.0));
            });
    }));
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-the-indicator-is-not-interactive-it-takes-no-focus-and-has-no-hover-or-pressed-state-keep",
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
                        spinner("ui-above-three-to-one-loading-indicator");
                        spacer().modifier(Modifier::new().width(Dp(48.0)));
                        loading_indicator()
                            .paused(true)
                            .indicator_color(scheme.surface_container)
                            .accessibility_label(
                                I18N.text("ui-below-three-to-one-loading-indicator"),
                            );
                    });
                gap(Dp(16.0));
                text()
                    .content(
                        I18N.text("ui-above-3-1-in-the-primary-role-versus-a-low-contrast-pairing"),
                    )
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(24.0));
    prose(
        "ui-do-not-place-the-indicator-over-content-that-people-are-reading-and-never-use-it-as-a-full",
    );
}

#[tessera]
fn labeling() {
    prose(
        "ui-name-the-wait-not-the-animation-screen-readers-should-announce-what-is-loading-such-as-loa",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || {
                spinner("ui-loading-content");
                gap(Dp(16.0));
                text()
                    .content(I18N.text("ui-do-loading-content-names-the-pending-work"))
                    .style(type_style(14.0, 20.0));
                gap(Dp(32.0));
                spinner("ui-loading").contained(true);
                gap(Dp(16.0));
                text()
                    .content(I18N.text("ui-don-t-rely-on-the-animation-alone-a-bare-loading-without-context-is-ambiguous"))
                    .style(type_style(14.0, 20.0));
            });
    }));
    gap(Dp(32.0));
    prose(
        "ui-update-the-label-when-the-wait-changes-stage-and-remove-the-indicator-once-the-content-arr",
    );
}
