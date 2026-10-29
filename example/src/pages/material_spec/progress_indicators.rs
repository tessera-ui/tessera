//! Progress indicator specifications with live determinate and indeterminate samples.
//!
//! ## Usage
//!
//! Compare linear and circular progress, wavy expressive shapes and reporting states.

use tessera_components::{
    button::button,
    column::column,
    flow_row::flow_row,
    modifier::ModifierExt as _,
    progress::{circular_progress_indicator, linear_progress_indicator},
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    wavy_progress::{circular_wavy_progress_indicator, linear_wavy_progress_indicator},
};
use tessera_ui::{
    Dp, FrameNanosControl, Modifier, RenderSlot, current_frame_nanos, receive_frame_nanos, remember,
    tessera, use_context,
};

use super::{
    ACCENT,
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn progress_indicators_spec() {
    article(
        "Progress indicators",
        "Progress indicators show the status of a process in real time.",
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
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "Baseline tokens",
                content: RenderSlot::new(baseline_tokens),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview_intro)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive_update),
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
                    content: RenderSlot::new(guideline_anatomy),
                },
                SpecSection {
                    title: "Placement",
                    content: RenderSlot::new(placement),
                },
                SpecSection {
                    title: "Responsive layout",
                    content: RenderSlot::new(responsive_layout),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(accessibility_use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(accessibility_interaction),
                },
                SpecSection {
                    title: "Labeling elements",
                    content: RenderSlot::new(accessibility_labeling),
                },
            ],
        },
    ])
    .hero(|| {
        panel(RenderSlot::new(|| {
            linear_sample("Preview".to_owned(), true, 0.6, true, false);
            gap(Dp(32.0));
            flow_row()
                .item_spacing(Dp(48.0))
                .children(|| {
                    circular_sample("Preview determinate".to_owned(), true, 0.65, true, false);
                    circular_sample("Preview indeterminate".to_owned(), false, 0.0, true, false);
                });
        }));
    });
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
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
            .accessibility_label(format!("{label} linear progress"));
    } else {
        linear_progress_indicator()
            .modifier(Modifier::new().fill_max_width().height(stroke))
            .progress_optional(progress)
            .accessibility_label(format!("{label} linear progress"));
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
            .accessibility_label(format!("{label} circular progress"));
    } else {
        circular_progress_indicator()
            .diameter(diameter)
            .stroke_width(stroke)
            .progress_optional(progress)
            .accessibility_label(format!("{label} circular progress"));
    }
}

#[tessera]
fn auto_determinate() {
    let progress = remember(|| 0.0_f32);
    let running = remember(|| true);
    let last = remember(current_frame_nanos);
    if running.get() {
        receive_frame_nanos(move |frame_nanos| {
            if !running.get() {
                return FrameNanosControl::Stop;
            }
            let elapsed = (frame_nanos.saturating_sub(last.get()) as f32 / 1_000_000_000.0).min(0.1);
            last.set(frame_nanos);
            let next = (progress.get() + elapsed / 5.0).min(1.0);
            progress.set(next);
            if next >= 1.0 {
                running.set(false);
                FrameNanosControl::Stop
            } else {
                FrameNanosControl::Continue
            }
        });
    }
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            linear_progress_indicator()
                .modifier(Modifier::new().fill_max_width().height(Dp(4.0)))
                .progress_optional(Some(progress.get()))
                .accessibility_label("Determinate download progress".to_owned());
            gap(Dp(16.0));
            text()
                .content(format!(
                    "{}% complete",
                    (progress.get() * 100.0).round() as u32
                ))
                .style(type_style(14.0, 20.0));
            gap(Dp(16.0));
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    button()
                        .tonal()
                        .accessibility_label("Restart determinate progress".to_owned())
                        .on_click(move || {
                            progress.set(0.0);
                            running.set(true);
                        })
                        .child(|| {
                            text().content("Restart");
                        });
                    button()
                        .tonal()
                        .selected(!running.get())
                        .accessibility_label("Pause determinate progress".to_owned())
                        .on_click(move || running.set(!running.get()))
                        .child(move || {
                            text().content(if running.get() { "Pause" } else { "Resume" });
                        });
                });
        });
}

#[tessera]
fn overview_intro() {
    prose(
        "Progress indicators express an unspecified wait time or display the length of a process. Use a determinate indicator when the completion rate is known and an indeterminate one when it is not.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Overview determinate".to_owned(), true, 0.4, false, false);
        gap(Dp(24.0));
        linear_sample("Overview indeterminate".to_owned(), false, 0.0, false, false);
        gap(Dp(32.0));
        flow_row()
            .item_spacing(Dp(48.0))
            .children(|| {
                circular_sample("Overview circular determinate".to_owned(), true, 0.4, false, false);
                circular_sample(
                    "Overview circular indeterminate".to_owned(),
                    false,
                    0.0,
                    false,
                    false,
                );
            });
    }));
    gap(Dp(24.0));
    for point in [
        "Determinate indicators show how much of a known process is complete.",
        "Indeterminate indicators animate continuously while the wait time is unknown.",
        "Use one linear or circular indicator per process, close to its content.",
        "Expressive wavy indicators keep the same meaning as the flat ones.",
    ] {
        prose(format!("• {point}"));
    }
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
                "Android Views (MDC-Android)".into(),
                "Available".into(),
            ],
            vec!["Implementation".into(), "Web".into(), "Unavailable".into()],
        ],
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "The Material 3 expressive update adds wavy progress indicators. The wavy active indicator uses the same color roles and states as the flat one, and its amplitude grows with the reported progress, so determinate and indeterminate behavior stay readable.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Expressive linear".to_owned(), true, 0.55, true, false);
        gap(Dp(32.0));
        flow_row()
            .item_spacing(Dp(48.0))
            .children(|| {
                circular_sample("Expressive circular determinate".to_owned(), true, 0.6, true, false);
                circular_sample("Expressive circular indeterminate".to_owned(), false, 0.0, true, false);
            });
    }));
}

#[tessera]
fn differences() {
    prose(
        "Color: progress indicators use the primary role for the active indicator and the secondary container role for the track, with dynamic color support. Shape: flat indicators keep fully rounded ends, and the expressive update adds an optional wavy form. Motion: indeterminate indicators animate continuously instead of using an M2 sweeping bar.",
    );
}

#[tessera]
fn variants() {
    prose(
        "Two shapes cover four combinations: a linear or circular track, each with a determinate or indeterminate active indicator.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Variant determinate".to_owned(), true, 0.5, false, false);
        gap(Dp(32.0));
        linear_sample("Variant indeterminate".to_owned(), false, 0.0, false, false);
        gap(Dp(32.0));
        flow_row()
            .item_spacing(Dp(48.0))
            .children(|| {
                circular_sample("Variant circular determinate".to_owned(), true, 0.5, false, false);
                circular_sample("Variant circular indeterminate".to_owned(), false, 0.0, false, false);
            });
    }));
    gap(Dp(32.0));
    subheading("Wavy variants".into());
    gap(Dp(16.0));
    panel(RenderSlot::new(|| {
        linear_sample("Wavy determinate".to_owned(), true, 0.5, true, false);
        gap(Dp(32.0));
        linear_sample("Wavy indeterminate".to_owned(), false, 0.0, true, false);
        gap(Dp(32.0));
        flow_row()
            .item_spacing(Dp(48.0))
            .children(|| {
                circular_sample("Wavy circular determinate".to_owned(), true, 0.5, true, false);
                circular_sample("Wavy circular indeterminate".to_owned(), false, 0.0, true, false);
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
                .accessibility_label("Progress configuration wavy".to_owned())
                .on_click(move || wavy.set(!wavy.get()))
                .child(|| {
                    text().content("Wavy");
                });
            button()
                .tonal()
                .selected(thick.get())
                .accessibility_label("Progress configuration thick".to_owned())
                .on_click(move || thick.set(!thick.get()))
                .child(|| {
                    text().content("Thick stroke");
                });
            button()
                .tonal()
                .selected(determinate.get())
                .accessibility_label("Progress configuration determinate".to_owned())
                .on_click(move || determinate.set(!determinate.get()))
                .child(|| {
                    text().content("Determinate");
                });
        });
    gap(Dp(24.0));
    panel(RenderSlot::new(move || {
        linear_sample(
            "Configuration linear".to_owned(),
            determinate.get(),
            0.5,
            wavy.get(),
            thick.get(),
        );
        gap(Dp(32.0));
        circular_sample(
            "Configuration circular".to_owned(),
            determinate.get(),
            0.5,
            wavy.get(),
            thick.get(),
        );
        gap(Dp(16.0));
        text()
            .content(format!(
                "{} · {} stroke",
                if determinate.get() {
                    "determinate"
                } else {
                    "indeterminate"
                },
                if thick.get() { "thick" } else { "default" }
            ))
            .style(type_style(14.0, 20.0));
    }));
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token".into(), "Value".into()],
        vec![
            vec![
                "Track".into(),
                "md.comp.linear-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "Active indicator".into(),
                "md.comp.linear-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "Stop indicator".into(),
                "md.comp.linear-progress-indicator.stop-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "Track thickness".into(),
                "md.comp.linear-progress-indicator.track.thickness".into(),
                "4dp".into(),
            ],
            vec![
                "Active thickness".into(),
                "md.comp.linear-progress-indicator.active-indicator.thickness".into(),
                "4dp".into(),
            ],
            vec![
                "Shape".into(),
                "md.comp.linear-progress-indicator.track.shape".into(),
                "Full · round caps".into(),
            ],
            vec![
                "Circular track".into(),
                "md.comp.circular-progress-indicator.track.color".into(),
                "Secondary container".into(),
            ],
            vec![
                "Circular active".into(),
                "md.comp.circular-progress-indicator.active-indicator.color".into(),
                "Primary".into(),
            ],
            vec![
                "Circular size / thickness".into(),
                "md.comp.circular-progress-indicator.size / thickness".into(),
                "40dp / 4dp".into(),
            ],
            vec![
                "Indeterminate motion".into(),
                "md.comp.progress-indicator.indeterminate.animation".into(),
                "Continuous advance and retract".into(),
            ],
        ],
    );
}

#[tessera]
fn anatomy() {
    prose("A linear progress indicator contains a track, an active indicator and a stop indicator.");
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Anatomy linear".to_owned(), true, 0.6, false, false);
        gap(Dp(32.0));
        flow_row()
            .item_spacing(Dp(48.0))
            .children(|| {
                circular_sample("Anatomy circular".to_owned(), true, 0.6, false, false);
                circular_sample("Anatomy circular track".to_owned(), false, 0.0, false, false);
            });
    }));
    gap(Dp(16.0));
    prose("Linear: 1  Track    2  Active indicator    3  Stop indicator");
    gap(Dp(8.0));
    prose("Circular: 1  Track    2  Active indicator");
}

#[tessera]
fn guideline_anatomy() {
    anatomy();
}

#[tessera]
fn colors() {
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                let scheme = use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme;
                surface()
                    .style(scheme.surface.into())
                    .modifier(Modifier::new().fill_max_width())
                    .child(move || {
                        column()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                            .children(move || {
                                subheading(
                                    if dark { "Dark scheme" } else { "Light scheme" }.to_owned(),
                                );
                                gap(Dp(24.0));
                                linear_sample(
                                    if dark { "Dark linear" } else { "Light linear" }.to_owned(),
                                    true,
                                    0.5,
                                    false,
                                    false,
                                );
                                gap(Dp(32.0));
                                circular_sample(
                                    if dark { "Dark circular" } else { "Light circular" }.to_owned(),
                                    true,
                                    0.5,
                                    false,
                                    false,
                                );
                            });
                    });
            });
        gap(Dp(24.0));
    }
    subheading("Color roles".into());
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Active indicator".into(), "Primary".into()],
            vec!["Track".into(), "Secondary container".into()],
            vec!["Stop indicator".into(), "Primary".into()],
            vec!["Track on a surface container".into(), "Surface container highest".into()],
        ],
    );
}

#[tessera]
fn measurements() {
    panel(RenderSlot::new(|| {
        linear_sample("Measurement linear".to_owned(), true, 0.5, false, false);
        gap(Dp(32.0));
        circular_sample("Measurement circular".to_owned(), true, 0.5, false, false);
    }));
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Linear indicator height".into(), "4dp".into()],
            vec!["Linear default width".into(), "240dp".into()],
            vec!["Linear track gap".into(), "4dp".into()],
            vec!["Stop indicator size".into(), "4dp".into()],
            vec!["Circular diameter".into(), "40dp".into()],
            vec!["Circular stroke width".into(), "4dp".into()],
            vec!["Circular track gap".into(), "4dp".into()],
            vec!["Wavy linear height".into(), "12dp".into()],
            vec!["Wavy circular diameter".into(), "48dp".into()],
            vec!["Minimum interactive target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn baseline_tokens() {
    spec_table(
        vec!["Token".into(), "Baseline value".into()],
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
        "Show progress as soon as a process starts and keep the indicator visible until it finishes. Determinate indicators must report the real completion fraction; never animate them at an arbitrary rate.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Do".into(), "Don't".into()],
        vec![
            vec![
                "Use a determinate indicator when completion can be measured".into(),
                "Use a determinate indicator for an unknown wait time".into(),
            ],
            vec![
                "Use an indeterminate indicator while the length is unknown".into(),
                "Switch an indeterminate indicator into a determinate one mid-process".into(),
            ],
            vec![
                "Keep one indicator per process, close to its content".into(),
                "Show several competing indicators for one process".into(),
            ],
            vec![
                "Keep the indicator visible until the process finishes".into(),
                "Remove progress feedback while work continues".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    auto_determinate();
}

#[tessera]
fn placement() {
    prose(
        "Place a linear indicator along the edge of the container it describes and center a circular indicator inside its container. Progress indicators do not take focus and never replace the content they describe.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Context".into(), "Do".into(), "Avoid".into()],
        vec![
            vec![
                "Page or card load".into(),
                "Linear indicator on the top edge".into(),
                "A circular indicator floating over content".into(),
            ],
            vec![
                "Button or short action".into(),
                "Small circular indicator inside the button".into(),
                "A full-width linear indicator inside a button".into(),
            ],
            vec![
                "Media or image load".into(),
                "Circular indicator at the center".into(),
                "A linear indicator crossing the media".into(),
            ],
        ],
    );
}

#[tessera]
fn responsive_layout() {
    prose(
        "Let a linear indicator stretch with its container, and keep a circular indicator at its fixed 40dp size so it stays legible at every window width.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Responsive linear".to_owned(), true, 0.35, false, false);
        gap(Dp(32.0));
        circular_sample("Responsive circular".to_owned(), true, 0.35, false, false);
    }));
}

#[tessera]
fn accessibility_use_cases() {
    prose("People should be able to use assistive technology to:");
    gap(Dp(16.0));
    for point in [
        "Navigate to the progress indicator.",
        "Understand what progress the indicator is communicating.",
        "Hear the completion value of a determinate indicator.",
    ] {
        prose(format!("• {point}"));
    }
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Accessible determinate".to_owned(), true, 0.75, false, false);
    }));
}

#[tessera]
fn accessibility_interaction() {
    prose(
        "A determinate indicator publishes its value through the progress indicator role, so assistive technology can report a percentage. An indeterminate indicator publishes no numeric value; it must never announce a fake percentage.",
    );
    gap(Dp(24.0));
    panel(RenderSlot::new(|| {
        linear_sample("Accessible indeterminate".to_owned(), false, 0.0, false, false);
        gap(Dp(32.0));
        circular_sample("Accessible circular indeterminate".to_owned(), false, 0.0, false, false);
    }));
}

#[tessera]
fn accessibility_labeling() {
    prose(
        "Name the process rather than the shape. A determinate indicator reports its percentage automatically, while an indeterminate one is announced as in progress.",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Accessible information".into()],
        vec![
            vec![
                "Determinate indicator".into(),
                "Process name plus the reported percentage".into(),
            ],
            vec![
                "Indeterminate indicator".into(),
                "Process name with an in-progress state".into(),
            ],
            vec![
                "Track and stop indicator".into(),
                "Decorative, hidden from assistive technology".into(),
            ],
            vec![
                "Visible percentage text".into(),
                "Keep it out of the name to avoid double announcements".into(),
            ],
        ],
    );
}
