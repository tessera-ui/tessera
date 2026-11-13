//! Progress indicator specifications with live determinate and indeterminate
//! samples.
//!
//! ## Usage
//!
//! Compare linear and circular progress, wavy expressive shapes and reporting
//! states.

use tessera_components::{
    column::column,
    flow_row::flow_row,
    md3::{
        button::button,
        progress::{circular_progress_indicator, linear_progress_indicator},
        text::text,
        wavy_progress::{circular_wavy_progress_indicator, linear_wavy_progress_indicator},
    },
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{Dp, Modifier, RenderSlot, launch_once, remember, tessera, with_frame_time};

use crate::res::I18N;

use super::{
    article::{ArticlePage, SpecSection, article},
    article_helpers::prose,
    gap, panel, scheme_panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn progress_indicators_spec(
    route: Option<crate::pages::material_spec::article::ArticleRoute>,
) {
    article(
        "ui-progress-indicators",
        "ui-progress-indicators-show-the-status-of-a-process-in-real-time-4416",
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
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-baseline-tokens",
                content: RenderSlot::new(baseline_tokens),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview_intro)),
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
                    content: RenderSlot::new(anatomy),
                },
                SpecSection {
                    title: "ui-placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "ui-responsive-layout",
                    content: RenderSlot::new(responsive_layout),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(accessibility_use_cases),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(accessibility_interaction),
                },
                SpecSection {
                    title: "ui-labeling-elements",
                    content: RenderSlot::new(accessibility_labeling),
                },
            ],
        },
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            linear_sample("ui-preview".to_owned(), true, 0.6, true, false);
            gap(Dp(32.0));
            flow_row().item_spacing(Dp(48.0)).children(|| {
                circular_sample("ui-preview-determinate".to_owned(), true, 0.65, true, false);
                circular_sample(
                    "ui-preview-indeterminate".to_owned(),
                    false,
                    0.0,
                    true,
                    false,
                );
            });
        }));
    });
}

#[tessera]
fn linear_sample(
    #[prop(into)] label: String,
    determinate: bool,
    value: f32,
    wavy: bool,
    thick: bool,
) {
    let stroke = Dp(if thick { 8.0 } else { 4.0 });
    let height = Dp(if thick { 16.0 } else { 12.0 });
    let progress = determinate.then_some(value);
    if wavy {
        linear_wavy_progress_indicator()
            .modifier(Modifier::new().fill_max_width().height(height))
            .progress_optional(progress)
            .stroke_width(stroke)
            .track_stroke_width(stroke)
            .accessibility_label(I18N.format(
                "ui-value-linear-progress",
                [("v0", I18N.text(&label).into())],
            ));
    } else {
        linear_progress_indicator()
            .modifier(Modifier::new().fill_max_width().height(stroke))
            .progress_optional(progress)
            .accessibility_label(I18N.format(
                "ui-value-linear-progress",
                [("v0", I18N.text(&label).into())],
            ));
    }
}

#[tessera]
fn circular_sample(
    #[prop(into)] label: String,
    determinate: bool,
    value: f32,
    wavy: bool,
    thick: bool,
) {
    let stroke = Dp(if thick { 8.0 } else { 4.0 });
    let diameter = Dp(if wavy {
        if thick { 56.0 } else { 48.0 }
    } else if thick {
        48.0
    } else {
        40.0
    });
    let progress = determinate.then_some(value);
    if wavy {
        circular_wavy_progress_indicator()
            .modifier(Modifier::new().size(diameter, diameter))
            .progress_optional(progress)
            .stroke_width(stroke)
            .track_stroke_width(stroke)
            .accessibility_label(I18N.format(
                "ui-value-circular-progress",
                [("v0", I18N.text(&label).into())],
            ));
    } else {
        circular_progress_indicator()
            .diameter(diameter)
            .stroke_width(stroke)
            .progress_optional(progress)
            .accessibility_label(I18N.format(
                "ui-value-circular-progress",
                [("v0", I18N.text(&label).into())],
            ));
    }
}

#[tessera]
fn auto_determinate() {
    let progress = remember(|| 0.0_f32);
    let running = remember(|| true);
    if running.get() {
        launch_once(async move {
            let mut last = None;
            loop {
                let done = with_frame_time(|now| {
                    if !running.get() {
                        return true;
                    }
                    let elapsed = last.map_or(0.0, |last| {
                        now.saturating_duration_since(last).as_secs_f32().min(0.1)
                    });
                    last = Some(now);
                    let next = (progress.get() + elapsed / 5.0).min(1.0);
                    progress.set(next);
                    if next >= 1.0 {
                        running.set(false);
                    }
                    next >= 1.0
                })
                .await;
                if done {
                    break;
                }
            }
        });
    }
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            linear_progress_indicator()
                .modifier(Modifier::new().fill_max_width().height(Dp(4.0)))
                .progress_optional(Some(progress.get()))
                .accessibility_label(I18N.text("ui-determinate-download-progress"));
            gap(Dp(16.0));
            text(I18N.format(
                "ui-value-complete",
                [("v0", ((progress.get() * 100.0).round() as u32).into())],
            ))
            .style(type_style(14.0, 20.0));
            gap(Dp(16.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    button()
                        .tonal()
                        .accessibility_label(I18N.text("ui-restart-determinate-progress"))
                        .on_click(move || {
                            progress.set(0.0);
                            running.set(true);
                        })
                        .child(move || {
                            text(I18N.text("ui-restart"));
                        });
                    button()
                        .tonal()
                        .selected(!running.get())
                        .accessibility_label(I18N.text("ui-pause-determinate-progress"))
                        .on_click(move || running.set(!running.get()))
                        .child(move || {
                            text(I18N.text(if running.get() {
                                "ui-pause"
                            } else {
                                "ui-resume"
                            }));
                        });
                });
        });
}

#[tessera]
fn overview_intro() {
    prose(
        "ui-progress-indicators-express-an-unspecified-wait-time-or-display-the-length-of-a-process-us",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample(
            "ui-overview-determinate".to_owned(),
            true,
            0.4,
            false,
            false,
        );
        gap(Dp(24.0));
        linear_sample(
            "ui-overview-indeterminate".to_owned(),
            false,
            0.0,
            false,
            false,
        );
        gap(Dp(32.0));
        flow_row().item_spacing(Dp(48.0)).children(|| {
            circular_sample(
                "ui-overview-circular-determinate".to_owned(),
                true,
                0.4,
                false,
                false,
            );
            circular_sample(
                "ui-overview-circular-indeterminate".to_owned(),
                false,
                0.0,
                false,
                false,
            );
        });
    }));
    gap(Dp(24.0));
    for point in [
        "ui-determinate-indicators-show-how-much-of-a-known-process-is-complete",
        "ui-indeterminate-indicators-animate-continuously-while-the-wait-time-is-unknown",
        "ui-use-one-linear-or-circular-indicator-per-process-close-to-its-content",
        "ui-expressive-wavy-indicators-keep-the-same-meaning-as-the-flat-ones",
    ] {
        prose(format!("• {}", I18N.text(point)));
    }
}

#[tessera]
fn variants() {
    prose(
        "ui-two-shapes-cover-four-combinations-a-linear-or-circular-track-each-with-a-determinate-or-i",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("ui-variant-determinate".to_owned(), true, 0.5, false, false);
        gap(Dp(32.0));
        linear_sample(
            "ui-variant-indeterminate".to_owned(),
            false,
            0.0,
            false,
            false,
        );
        gap(Dp(32.0));
        flow_row().item_spacing(Dp(48.0)).children(|| {
            circular_sample(
                "ui-variant-circular-determinate".to_owned(),
                true,
                0.5,
                false,
                false,
            );
            circular_sample(
                "ui-variant-circular-indeterminate".to_owned(),
                false,
                0.0,
                false,
                false,
            );
        });
    }));
    gap(Dp(32.0));
    subheading("ui-wavy-variants".into());
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        linear_sample("ui-wavy-determinate".to_owned(), true, 0.5, true, false);
        gap(Dp(32.0));
        linear_sample("ui-wavy-indeterminate".to_owned(), false, 0.0, true, false);
        gap(Dp(32.0));
        flow_row().item_spacing(Dp(48.0)).children(|| {
            circular_sample(
                "ui-wavy-circular-determinate".to_owned(),
                true,
                0.5,
                true,
                false,
            );
            circular_sample(
                "ui-wavy-circular-indeterminate".to_owned(),
                false,
                0.0,
                true,
                false,
            );
        });
    }));
}

#[tessera]
fn configurations() {
    let wavy = remember(|| false);
    let thick = remember(|| false);
    let determinate = remember(|| true);
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .selected(wavy.get())
                .accessibility_label(I18N.text("ui-progress-configuration-wavy"))
                .on_click(move || wavy.set(!wavy.get()))
                .child(move || {
                    text(I18N.text("ui-wavy"));
                });
            button()
                .tonal()
                .selected(thick.get())
                .accessibility_label(I18N.text("ui-progress-configuration-thick"))
                .on_click(move || thick.set(!thick.get()))
                .child(move || {
                    text(I18N.text("ui-thick-stroke"));
                });
            button()
                .tonal()
                .selected(determinate.get())
                .accessibility_label(I18N.text("ui-progress-configuration-determinate"))
                .on_click(move || determinate.set(!determinate.get()))
                .child(move || {
                    text(I18N.text("ui-determinate"));
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        linear_sample(
            "ui-configuration-linear".to_owned(),
            determinate.get(),
            0.5,
            wavy.get(),
            thick.get(),
        );
        gap(Dp(32.0));
        circular_sample(
            "ui-configuration-circular".to_owned(),
            determinate.get(),
            0.5,
            wavy.get(),
            thick.get(),
        );
        gap(Dp(16.0));
        text(
            I18N.format(
                "ui-value-value-stroke",
                [
                    (
                        "v0",
                        I18N.text(
                            &(if determinate.get() {
                                "ui-determinate-7102"
                            } else {
                                "ui-indeterminate"
                            }),
                        )
                        .into(),
                    ),
                    (
                        "v1",
                        I18N.text(
                            &(if thick.get() {
                                "ui-thick"
                            } else {
                                "ui-default-7074"
                            }),
                        )
                        .into(),
                    ),
                ],
            ),
        )
        .style(type_style(14.0, 20.0));
    }));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["ui-element".into(), "ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-track".into(),
                "md.comp.linear-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "ui-active-indicator".into(),
                "md.comp.linear-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "ui-stop-indicator".into(),
                "md.comp.linear-progress-indicator.stop-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "ui-track-thickness".into(),
                "md.comp.linear-progress-indicator.track.thickness".into(),
                "4dp".into(),
            ],
            vec![
                "ui-active-thickness".into(),
                "md.comp.linear-progress-indicator.active-indicator.thickness".into(),
                "4dp".into(),
            ],
            vec![
                "ui-shape".into(),
                "md.comp.linear-progress-indicator.track.shape".into(),
                "ui-full-round-caps".into(),
            ],
            vec![
                "ui-circular-track".into(),
                "md.comp.circular-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "ui-circular-active".into(),
                "md.comp.circular-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "ui-circular-size-thickness".into(),
                "md.comp.circular-progress-indicator.size / thickness".into(),
                "40dp / 4dp".into(),
            ],
            vec![
                "ui-indeterminate-motion".into(),
                "md.comp.progress-indicator.indeterminate.animation".into(),
                "ui-continuous-advance-and-retract".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    prose(
        "ui-a-linear-progress-indicator-contains-a-track-an-active-indicator-and-a-stop-indicator",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("ui-anatomy-linear".to_owned(), true, 0.6, false, false);
        gap(Dp(32.0));
        flow_row().item_spacing(Dp(48.0)).children(|| {
            circular_sample("ui-anatomy-circular".to_owned(), true, 0.6, false, false);
            circular_sample(
                "ui-anatomy-circular-track".to_owned(),
                false,
                0.0,
                false,
                false,
            );
        });
    }));
    gap(Dp(16.0));
    prose("ui-linear-1-track-2-active-indicator-3-stop-indicator");
    gap(Dp(8.0));
    prose("ui-circular-1-track-2-active-indicator");
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        scheme_panel(
            dark,
            RenderSlot::new(move || {
                linear_sample(
                    if dark {
                        "ui-dark-linear"
                    } else {
                        "ui-light-linear"
                    }
                    .to_owned(),
                    true,
                    0.5,
                    false,
                    false,
                );
                gap(Dp(32.0));
                circular_sample(
                    if dark {
                        "ui-dark-circular"
                    } else {
                        "ui-light-circular"
                    }
                    .to_owned(),
                    true,
                    0.5,
                    false,
                    false,
                );
            }),
        );
        gap(Dp(24.0));
    }
    subheading("ui-color-roles-4475".into());
    spec_table(
        vec!["ui-element".into(), "ui-color-role".into()],
        vec![
            vec!["ui-active-indicator".into(), "Primary".into()],
            vec!["ui-track".into(), "Secondary container".into()],
            vec!["ui-stop-indicator".into(), "Primary".into()],
            vec![
                "ui-track-on-a-surface-container".into(),
                "Surface container highest".into(),
            ],
        ],
    );
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        linear_sample("ui-measurement-linear".to_owned(), true, 0.5, false, false);
        gap(Dp(32.0));
        circular_sample(
            "ui-measurement-circular".to_owned(),
            true,
            0.5,
            false,
            false,
        );
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-linear-indicator-height".into(), "4dp".into()],
            vec!["ui-linear-default-width".into(), "240dp".into()],
            vec!["ui-linear-track-gap".into(), "4dp".into()],
            vec!["ui-stop-indicator-size".into(), "4dp".into()],
            vec!["ui-circular-diameter".into(), "40dp".into()],
            vec!["ui-circular-stroke-width".into(), "4dp".into()],
            vec!["ui-circular-track-gap".into(), "4dp".into()],
            vec!["ui-wavy-linear-height".into(), "12dp".into()],
            vec!["ui-wavy-circular-diameter".into(), "48dp".into()],
            vec!["ui-minimum-interactive-target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn baseline_tokens() {
    spec_table(
        vec!["ui-token".into(), "ui-baseline-value".into()],
        vec![
            vec![
                "md.comp.linear-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "md.comp.linear-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.linear-progress-indicator.stop-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.linear-progress-indicator.track.thickness".into(),
                "4dp".into(),
            ],
            vec![
                "md.comp.circular-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "md.comp.circular-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "md.comp.circular-progress-indicator.size".into(),
                "40dp".into(),
            ],
            vec![
                "md.comp.circular-progress-indicator.track.thickness".into(),
                "4dp".into(),
            ],
        ],
    );
}

#[tessera]
fn usage() {
    prose(
        "ui-show-progress-as-soon-as-a-process-starts-and-keep-the-indicator-visible-until-it-finishes",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Do".into(), "Don't".into()],
        vec![
            vec![
                "ui-use-a-determinate-indicator-when-completion-can-be-measured".into(),
                "ui-use-a-determinate-indicator-for-an-unknown-wait-time".into(),
            ],
            vec![
                "ui-use-an-indeterminate-indicator-while-the-length-is-unknown".into(),
                "ui-switch-an-indeterminate-indicator-into-a-determinate-one-mid-process".into(),
            ],
            vec![
                "ui-keep-one-indicator-per-process-close-to-its-content".into(),
                "ui-show-several-competing-indicators-for-one-process".into(),
            ],
            vec![
                "ui-keep-the-indicator-visible-until-the-process-finishes".into(),
                "ui-remove-progress-feedback-while-work-continues".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    auto_determinate();
}

#[tessera]
fn placement() {
    prose(
        "ui-place-a-linear-indicator-along-the-edge-of-the-container-it-describes-and-center-a-circula",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-context".into(), "Do".into(), "ui-avoid".into()],
        vec![
            vec![
                "ui-page-or-card-load".into(),
                "ui-linear-indicator-on-the-top-edge".into(),
                "ui-a-circular-indicator-floating-over-content".into(),
            ],
            vec![
                "ui-button-or-short-action".into(),
                "ui-small-circular-indicator-inside-the-button".into(),
                "ui-a-full-width-linear-indicator-inside-a-button".into(),
            ],
            vec![
                "ui-media-or-image-load".into(),
                "ui-circular-indicator-at-the-center".into(),
                "ui-a-linear-indicator-crossing-the-media".into(),
            ],
        ],
    );
}

#[tessera]
fn responsive_layout() {
    prose(
        "ui-let-a-linear-indicator-stretch-with-its-container-and-keep-a-circular-indicator-at-its-fix",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("ui-responsive-linear".to_owned(), true, 0.35, false, false);
        gap(Dp(32.0));
        circular_sample(
            "ui-responsive-circular".to_owned(),
            true,
            0.35,
            false,
            false,
        );
    }));
}

#[tessera]
fn accessibility_use_cases() {
    prose("ui-people-should-be-able-to-use-assistive-technology-to");
    gap(Dp(16.0));
    for point in [
        "ui-navigate-to-the-progress-indicator",
        "ui-understand-what-progress-the-indicator-is-communicating-4511",
        "ui-hear-the-completion-value-of-a-determinate-indicator",
    ] {
        prose(format!("• {}", I18N.text(point)));
    }
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample(
            "ui-accessible-determinate".to_owned(),
            true,
            0.75,
            false,
            false,
        );
    }));
}

#[tessera]
fn accessibility_interaction() {
    prose(
        "ui-a-determinate-indicator-publishes-its-value-through-the-progress-indicator-role-so-assisti",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample(
            "ui-accessible-indeterminate".to_owned(),
            false,
            0.0,
            false,
            false,
        );
        gap(Dp(32.0));
        circular_sample(
            "ui-accessible-circular-indeterminate".to_owned(),
            false,
            0.0,
            false,
            false,
        );
    }));
}

#[tessera]
fn accessibility_labeling() {
    prose(
        "ui-name-the-process-rather-than-the-shape-a-determinate-indicator-reports-its-percentage-auto",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-accessible-information".into()],
        vec![
            vec![
                "ui-determinate-indicator".into(),
                "ui-process-name-plus-the-reported-percentage".into(),
            ],
            vec![
                "ui-indeterminate-indicator".into(),
                "ui-process-name-with-an-in-progress-state".into(),
            ],
            vec![
                "ui-track-and-stop-indicator".into(),
                "ui-decorative-hidden-from-assistive-technology".into(),
            ],
            vec![
                "ui-visible-percentage-text".into(),
                "ui-keep-it-out-of-the-name-to-avoid-double-announcements".into(),
            ],
        ],
    );
}
