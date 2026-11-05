//! Interactive time picker specifications.
//!
//! ## Usage
//!
//! Select times with clock or keyboard controls in either hour format.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    dialog::dialog_provider,
    flow_row::flow_row,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    time_picker::{
        TimePickerDisplayMode, TimePickerLayout, TimePickerSelection, TimePickerState, time_picker,
        time_picker_dialog,
    },
};
use tessera_ui::{
    Dp, FocusManager, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::{
    ACCENT,
    annotations::{self, PreciseGeometry, Rect, dotted, watch_precise},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, PartialEq)]
struct Sample {
    state: State<TimePickerState>,
    layout: State<TimePickerLayout>,
    status: State<String>,
}

#[tessera]
pub(super) fn time_pickers_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    let modal = remember(|| false);
    let sample = Sample {
        state: remember(|| TimePickerState::new(9, 41, false, TimePickerDisplayMode::Picker)),
        layout: remember(TimePickerLayout::default),
        status: remember(|| "ui-ready".to_owned()),
    };
    let committed = remember(|| (9_u8, 41_u8));
    let draft = Sample {
        state: remember(|| TimePickerState::new(9, 41, false, TimePickerDisplayMode::Picker)),
        layout: remember(TimePickerLayout::default),
        status: sample.status,
    };
    dialog_provider()
        .is_open(modal.get())
        .padding(Dp::ZERO)
        .accessibility_label(I18N.text("ui-time-picker-dialog"))
        .on_close_request(move || modal.set(false))
        .main_content(move || {
            article(
                "ui-time-pickers",
                "ui-time-pickers-help-people-select-and-set-a-specific-time",
                vec![
                    SpecSection {
                        title: "ui-tokens-specs",
                        content: RenderSlot::new(tokens),
                    },
                    SpecSection {
                        title: "ui-anatomy",
                        content: RenderSlot::new(move || {
                            subheading("ui-time-picker-dial".into());
                            time_figure(
                                false,
                                false,
                                false,
                                FigureKind::Specs,
                                "ui-dial-anatomy".into(),
                            );
                            gap(Dp(24.0));
                            subheading("ui-time-picker-input".into());
                            time_figure(
                                true,
                                false,
                                false,
                                FigureKind::Specs,
                                "ui-input-anatomy".into(),
                            );
                            gap(Dp(24.0));
                            controls(sample, "ui-anatomy");
                            gap(Dp(24.0));
                            visual(sample, "ui-anatomy");
                            gap(Dp(24.0));
                            button()
                                .outlined()
                                .accessibility_label(I18N.text("ui-open-time-picker-dialog"))
                                .on_click(move || open_picker(modal, draft, committed))
                                .child(move || {
                                    text(I18N.text("ui-open-dialog"));
                                });
                            confirmed_time(committed);
                            gap(Dp(24.0));
                        }),
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
                    SpecSection {
                        title: "ui-configurations",
                        content: RenderSlot::new(configurations),
                    },
                ],
            )
            .route_optional(route.clone())
            .compact_content_inset(Dp(8.0))
            .pages(vec![
                ArticlePage {
                    title: "ui-overview",
                    intro: Some(RenderSlot::new(time_variants)),
                    sections: vec![
                        SpecSection {
                            title: "ui-availability-resources",
                            content: RenderSlot::new(time_resources),
                        },
                        SpecSection {
                            title: "ui-differences-from-m2",
                            content: RenderSlot::new(time_m2_difference),
                        },
                    ],
                },
                ArticlePage {
                    title: "ui-guidelines",
                    intro: Some(RenderSlot::new(|| {
                        time_figure(
                            false,
                            false,
                            false,
                            FigureKind::Plain,
                            "ui-guideline-introduction".into(),
                        );
                    })),
                    sections: vec![
                        SpecSection {
                            title: "ui-usage",
                            content: RenderSlot::new(time_usage),
                        },
                        SpecSection {
                            title: "ui-anatomy",
                            content: RenderSlot::new(time_guideline_anatomy),
                        },
                        SpecSection {
                            title: "ui-placement",
                            content: RenderSlot::new(move || {
                                time_placement();
                                gap(Dp(24.0));
                                button()
                                    .outlined()
                                    .accessibility_label(I18N.text("ui-open-placement-time-picker"))
                                    .on_click(move || open_picker(modal, draft, committed))
                                    .child(move || {
                                        text(I18N.text("ui-open-time-picker"));
                                    });
                                confirmed_time(committed);
                            }),
                        },
                        SpecSection {
                            title: "ui-adaptive-design",
                            content: RenderSlot::new(time_adaptive),
                        },
                        SpecSection {
                            title: "ui-behavior",
                            content: RenderSlot::new(move || {
                                time_behavior(sample, modal, draft, committed);
                            }),
                        },
                    ],
                },
                ArticlePage {
                    title: "ui-accessibility",
                    intro: None,
                    sections: vec![
                        SpecSection {
                            title: "ui-use-cases",
                            content: RenderSlot::new(time_accessibility_use_cases),
                        },
                        SpecSection {
                            title: "ui-interaction-style",
                            content: RenderSlot::new(time_accessibility_interaction),
                        },
                        SpecSection {
                            title: "ui-keyboard-navigation",
                            content: RenderSlot::new(time_keyboard),
                        },
                        SpecSection {
                            title: "ui-labeling-elements",
                            content: RenderSlot::new(time_labels),
                        },
                    ],
                },
            ]);
        })
        .dialog_content(move |dialog_scope| {
            picker_surface(draft, "ui-modal")
                .dialog(dialog_scope)
                .modal(modal)
                .committed(committed);
        });
}

fn open_picker(modal: State<bool>, draft: Sample, committed: State<(u8, u8)>) {
    let (hour, minute) = committed.get();
    draft.state.set(TimePickerState::new(
        hour,
        minute,
        false,
        TimePickerDisplayMode::Picker,
    ));
    modal.set(true);
}

#[tessera]
fn confirmed_time(committed: State<(u8, u8)>) {
    let (hour, minute) = committed.get();
    gap(Dp(16.0));
    text(I18N.format(
        "ui-saved-time-value-value",
        [
            ("v0", format!("{:02}", hour).into()),
            ("v1", format!("{:02}", minute).into()),
        ],
    ))
    .style(type_style(16.0, 24.0));
}

#[tessera]
fn controls(sample: Sample, #[prop(into)] label: String) {
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            let label = label.clone();
            button()
                .tonal()
                .selected(sample.state.with(TimePickerState::is_24_hour))
                .accessibility_label(I18N.format(
                    "ui-value-24-hour-format",
                    [("v0", I18N.text(&label).into())],
                ))
                .on_click(move || {
                    sample
                        .state
                        .with_mut(|state| state.set_is_24_hour(!state.is_24_hour()));
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text(
                        I18N.text(if sample.state.with(TimePickerState::is_24_hour) {
                            "ui-24-hours"
                        } else {
                            "ui-12-hours"
                        }),
                    );
                });
            button()
                .tonal()
                .selected(sample.layout.get() == TimePickerLayout::Horizontal)
                .accessibility_label(I18N.format(
                    "ui-value-horizontal-layout",
                    [("v0", I18N.text(&label).into())],
                ))
                .on_click(move || {
                    sample
                        .layout
                        .set(if sample.layout.get() == TimePickerLayout::Vertical {
                            TimePickerLayout::Horizontal
                        } else {
                            TimePickerLayout::Vertical
                        });
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text(
                        I18N.text(if sample.layout.get() == TimePickerLayout::Horizontal {
                            "ui-horizontal"
                        } else {
                            "ui-vertical"
                        }),
                    );
                });
            button()
                .tonal()
                .selected(
                    sample
                        .state
                        .with(|state| state.display_mode() == TimePickerDisplayMode::Input),
                )
                .accessibility_label(I18N.format(
                    "ui-value-keyboard-input",
                    [("v0", I18N.text(&label).into())],
                ))
                .on_click(move || {
                    sample.state.with_mut(TimePickerState::toggle_display_mode);
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text(
                        I18N.text(
                            if sample
                                .state
                                .with(|state| state.display_mode() == TimePickerDisplayMode::Input)
                            {
                                "ui-keyboard-input"
                            } else {
                                "ui-clock-dial"
                            },
                        ),
                    );
                });
        });
}

#[tessera]
fn visual(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    let caption = label.clone();
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            picker_surface(sample, label.clone()).modal_optional(modal);
        });
    gap(Dp(16.0));
    text(
        (sample.state.with(|state| {
            I18N.format(
                "ui-value-time-value-value",
                [
                    ("v0", I18N.text(&caption).into()),
                    ("v1", format!("{:02}", state.hour()).into()),
                    ("v2", format!("{:02}", state.minute()).into()),
                ],
            )
        }))
        .to_string(),
    )
    .style(type_style(16.0, 24.0));
    text(I18N.text(sample.status.get())).style(type_style(14.0, 20.0));
}

#[tessera]
fn picker_surface(
    sample: Sample,
    #[prop(into)] label: String,
    modal: Option<State<bool>>,
    committed: Option<State<(u8, u8)>>,
    dialog: Option<tessera_components::dialog::DialogScope>,
) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let confirm_label = I18N.format("ui-value-confirm-time", [("v0", I18N.text(&label).into())]);
    let cancel_label = I18N.format("ui-value-cancel-time", [("v0", I18N.text(&label).into())]);
    surface()
        .style(theme.color_scheme.surface_container_high.into())
        .shape(Shape::rounded_rectangle(Dp(28.0)))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(I18N.format("ui-value-time-preview", [("v0", I18N.text(&label).into())])),
            merge_descendants: true,
            ..Default::default()
        }))
        .child(move || {
            let confirm_label = confirm_label.clone();
            let cancel_label = cancel_label.clone();
            boxed()
                .modifier(Modifier::new().padding_all(Dp(24.0)))
                .children(move || {
                    picker_contents(sample, confirm_label.clone(), cancel_label.clone())
                        .dialog_optional(dialog)
                        .modal_optional(modal)
                        .committed_optional(committed);
                });
        });
}

#[tessera]
fn picker_contents(
    sample: Sample,
    #[prop(into)] confirm_label: String,
    #[prop(into)] cancel_label: String,
    modal: Option<State<bool>>,
    committed: Option<State<(u8, u8)>>,
    dialog: Option<tessera_components::dialog::DialogScope>,
) {
    time_picker_dialog()
        .dialog_optional(dialog)
        .state(sample.state)
        .title(I18N.text(sample.state.with(|state| {
            if state.display_mode() == TimePickerDisplayMode::Input {
                "ui-enter-time"
            } else {
                "ui-select-time"
            }
        })))
        .picker_layout(sample.layout.get())
        .confirm_button(move || {
            button()
                .text()
                .accessibility_label(I18N.text(&(confirm_label)))
                .on_click(move || {
                    let time = sample.state.with(|state| (state.hour(), state.minute()));
                    if let Some(committed) = committed {
                        committed.set(time);
                    }
                    sample.status.set(I18N.format(
                        "ui-confirmed-value-value",
                        [
                            ("v0", format!("{:02}", time.0).into()),
                            ("v1", format!("{:02}", time.1).into()),
                        ],
                    ));
                    if let Some(modal) = modal {
                        modal.set(false);
                    }
                })
                .child(move || {
                    text(I18N.text("ui-ok"));
                });
        })
        .dismiss_button(move || {
            button()
                .text()
                .accessibility_label(I18N.text(&(cancel_label)))
                .on_click(move || {
                    sample.status.set("ui-cancelled".into());
                    if let Some(modal) = modal {
                        modal.set(false);
                    }
                })
                .child(move || {
                    text(I18N.text("ui-cancel"));
                });
        });
}

#[tessera]
fn example(#[prop(into)] label: String, input: bool) {
    let sample = Sample {
        state: remember(|| {
            TimePickerState::new(
                9,
                41,
                false,
                if input {
                    TimePickerDisplayMode::Input
                } else {
                    TimePickerDisplayMode::Picker
                },
            )
        }),
        layout: remember(TimePickerLayout::default),
        status: remember(|| "ui-ready".to_owned()),
    };
    controls(sample, label.clone());
    gap(Dp(24.0));
    visual(sample, label);
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["ui-element".into(), "ui-token-value".into()],
        vec![
            vec!["ui-container".into(), "Surface container high".into()],
            vec![
                "ui-selected-time-field".into(),
                "Primary container / on primary container".into(),
            ],
            vec![
                "ui-unselected-time-field-clock-dial".into(),
                "Surface container highest / on surface".into(),
            ],
            vec![
                "ui-selected-period".into(),
                "Tertiary container / on tertiary container".into(),
            ],
            vec!["ui-dial-selector".into(), "Primary / on primary".into()],
            vec!["ui-period-outline".into(), "Outline".into()],
        ],
    );
}

#[tessera]
fn colors() {
    subheading("ui-time-picker-dial-color".into());
    gap(Dp(24.0));
    time_color_pair(false);
    gap(Dp(32.0));
    subheading("ui-time-picker-input-color".into());
    gap(Dp(24.0));
    time_color_pair(true);
}

fn time_color_labels(input: bool) -> Vec<&'static str> {
    let mut labels = vec![
        "On surface variant",
        "On surface",
        "Surface container highest",
        "On surface",
        "Tertiary container",
        "On tertiary container",
        "Surface container high",
        "Outline",
        "On surface",
        "Primary",
        "On surface variant",
    ];
    if input {
        labels.extend(["On primary container", "Primary container"]);
    } else {
        labels[10] = "On primary";
        labels.extend([
            "Primary",
            "On surface variant",
            "On surface",
            "Surface container highest",
            "On primary container",
            "Primary container",
        ]);
    }
    labels
}

#[tessera]
fn time_color_pair(input: bool) {
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
                            "ui-time-picker-value-color-roles",
                            [(
                                "v0",
                                I18N.text(&(if input { "ui-input-7117" } else { "ui-dial" }))
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
                            .modifier(Modifier::new().weight(1.0))
                            .children(move || {
                                material_theme()
                                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                    .child(move || {
                                        time_figure(
                                            input,
                                            false,
                                            false,
                                            FigureKind::Colors,
                                            I18N.format(
                                                "ui-value-value-color",
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
                                                            &(if input {
                                                                "ui-input-7117"
                                                            } else {
                                                                "ui-dial"
                                                            }),
                                                        )
                                                        .into(),
                                                    ),
                                                ],
                                            ),
                                        );
                                    });
                            });
                    }
                });
        });
    gap(Dp(24.0));
    prose(
        I18N.format(
            "ui-time-picker-value-color-roles-used-for-light-and-dark-themes",
            [(
                "v0",
                I18N.text(&(if input { "ui-input-7117" } else { "ui-dial" }))
                    .into(),
            )],
        ),
    );
    gap(Dp(16.0));
    for (index, label) in time_color_labels(input).into_iter().enumerate() {
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
                            .color(scheme.surface)
                            .style(type_style(14.0, 20.0));
                    });
                text(I18N.text(label))
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp::ZERO));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn states() {
    text(I18N.text("ui-hover-focus-with-tab-or-use-the-arrow-keys-on-the-dial-activate-a-value-with-enter-or-spac")).style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    example("ui-states", false);
}

#[tessera]
fn measurements() {
    subheading("ui-time-picker-dial-vertical".into());
    time_figure(
        false,
        false,
        false,
        FigureKind::Measurements,
        "ui-vertical-measurements".into(),
    );
    gap(Dp(32.0));
    subheading("ui-time-picker-dial-horizontal".into());
    time_figure(
        false,
        true,
        false,
        FigureKind::Measurements,
        "ui-horizontal-measurements".into(),
    );
    gap(Dp(32.0));
    subheading("ui-time-picker-input".into());
    time_figure(
        true,
        false,
        false,
        FigureKind::Measurements,
        "ui-input-measurements".into(),
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "ui-measurement".into()],
        vec![
            vec!["ui-container-padding".into(), "24dp".into()],
            vec![
                "ui-dial-time-selector".into(),
                "ui-96-80dp-114dp-wide-for-vertical-24h".into(),
            ],
            vec!["ui-input-time-field".into(), "96 × 72dp".into()],
            vec![
                "ui-vertical-period-selector".into(),
                "ui-52-80dp-72dp-high-for-input".into(),
            ],
            vec!["ui-horizontal-period-selector".into(), "216 × 38dp".into()],
            vec!["ui-clock-dial".into(), "256dp".into()],
            vec!["ui-selector-handle-center".into(), "48 / 8dp".into()],
            vec!["ui-selector-track".into(), "2dp".into()],
        ],
    );
}

#[tessera]
fn configurations() {
    subheading("ui-vertical-orientation-and-horizontal-orientation".into());
    gap(Dp(24.0));
    time_configuration_pair(false, false);
    gap(Dp(32.0));
    subheading("ui-24-hour-time-picker-dial".into());
    gap(Dp(24.0));
    time_configuration_pair(false, true);
    gap(Dp(32.0));
    subheading("ui-12-hour-and-24-hour-time-picker-inputs".into());
    gap(Dp(24.0));
    time_configuration_pair(true, false);
    gap(Dp(24.0));
    text(I18N.text("ui-switch-orientation-clock-format-and-input-mode"))
        .style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    example("ui-configurations", false);
}

#[tessera]
fn time_configuration_pair(input: bool, twenty_four: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let title = if input {
        "ui-time-input-format-comparison"
    } else if twenty_four {
        "ui-24-hour-time-orientation-comparison"
    } else {
        "ui-12-hour-time-orientation-comparison"
    };
    let available_width = remember(|| Dp(1024.0));
    let scale = (available_width.get().0 / 1024.0) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(1024.0), Dp(if input { 400.0 } else { 640.0 }))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .semantics(SemanticsArgs {
                                role: Some(Role::Group),
                                label: Some(I18N.text(title)),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        row()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                for second in [false, true] {
                                    let horizontal = !input && second;
                                    let twenty_four = twenty_four || (input && second);
                                    let label = if input {
                                        if twenty_four {
                                            "ui-24-hour-input"
                                        } else {
                                            "ui-12-hour-input"
                                        }
                                    } else if twenty_four {
                                        if horizontal {
                                            "ui-24-hour-horizontal-dial"
                                        } else {
                                            "ui-24-hour-vertical-dial"
                                        }
                                    } else if horizontal {
                                        "ui-12-hour-horizontal-dial"
                                    } else {
                                        "ui-12-hour-vertical-dial"
                                    };
                                    if second {
                                        spacer().modifier(Modifier::new().width(Dp(32.0)));
                                    }
                                    time_configuration_card(
                                        input,
                                        horizontal,
                                        twenty_four,
                                        label.into(),
                                    );
                                }
                            });
                    });
            });
        });
}

#[tessera]
fn time_configuration_card(input: bool, horizontal: bool, twenty_four: bool, label: String) {
    let sample = Sample {
        state: remember(|| {
            TimePickerState::new(
                if twenty_four { 20 } else { 7 },
                0,
                twenty_four,
                if input {
                    TimePickerDisplayMode::Input
                } else {
                    TimePickerDisplayMode::Picker
                },
            )
        }),
        layout: remember(|| {
            if horizontal {
                TimePickerLayout::Horizontal
            } else {
                TimePickerLayout::Vertical
            }
        }),
        status: remember(String::new),
    };
    let open = remember(|| true);
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .width(Dp(if horizontal {
                    572.0
                } else if twenty_four {
                    304.0
                } else {
                    328.0
                }))
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(I18N.format(
                        "ui-value-time-configuration",
                        [("v0", I18N.text(&label).into())],
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            if open.get() {
                picker_surface(sample, label.clone()).modal(open);
            } else {
                text(I18N.text(sample.status.get())).style(type_style(16.0, 24.0));
                button()
                    .text()
                    .accessibility_label(
                        I18N.format("ui-reopen-value", [("v0", I18N.text(&label).into())]),
                    )
                    .on_click(move || open.set(true))
                    .child(move || {
                        text(I18N.text("ui-open-time-picker"));
                    });
            }
            gap(Dp(16.0));
            text(I18N.text(&(label))).style(type_style(16.0, 24.0));
        });
}

#[derive(Clone, Copy, Default, PartialEq)]
enum FigureKind {
    #[default]
    Plain,
    Specs,
    Guidelines,
    Measurements,
    Colors,
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn time_variants() {
    for item in [
        "ui-time-pickers-are-modal-and-cover-the-main-content",
        "ui-two-variants-dial-and-input-the-keyboard-icon-makes-input-available-from-the-dial",
        "ui-select-hours-minutes-and-for-a-12-hour-clock-am-or-pm",
        "ui-keep-touch-selection-comfortable-on-mobile-devices",
    ] {
        prose(format!("• {item}"));
        gap(Dp(12.0));
    }
    gap(Dp(12.0));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    flow_row()
        .item_spacing(Dp(32.0))
        .line_spacing(Dp(32.0))
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(|| {
            time_configuration_card(false, false, false, "ui-time-picker-dial".into());
            time_configuration_card(true, false, false, "ui-time-picker-input".into());
        });
}

#[tessera]
fn time_resources() {
    spec_table(
        vec!["ui-type".into(), "ui-resource".into(), "ui-status".into()],
        vec![
            vec![
                "ui-design".into(),
                "Material 3 Design Kit (Figma)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Flutter".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Jetpack Compose".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Android Views (MDC-Android)".into(),
                "ui-available".into(),
            ],
            vec![
                "ui-implementation".into(),
                "Material Web".into(),
                "ui-unavailable".into(),
            ],
        ],
    );
    gap(Dp(16.0));
    prose(
        "ui-the-availability-above-describes-the-material-libraries-the-live-examples-on-this-page-are",
    );
}

#[tessera]
fn time_m2_difference() {
    prose(
        "ui-material-3-keeps-the-dial-and-keyboard-variants-with-new-color-mappings-that-support-dynam",
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-element".into(), "M2".into(), "M3".into()],
        vec![
            vec![
                "ui-selected-time-field".into(),
                "ui-accent-emphasis".into(),
                "Primary container / on primary container".into(),
            ],
            vec![
                "ui-selected-period".into(),
                "ui-accent-emphasis".into(),
                "Tertiary container / on tertiary container".into(),
            ],
            vec![
                "ui-color-system".into(),
                "ui-previous-color-mappings".into(),
                "ui-dynamic-color-compatible-semantic-roles".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    time_figure(
        false,
        false,
        false,
        FigureKind::Plain,
        "ui-m3-dynamic-color-mapping".into(),
    );
    prose("ui-m3-selected-hour-and-selected-period-have-distinct-color-mappings");
}

#[tessera]
fn time_placement() {
    prose(
        "ui-center-a-modal-time-picker-above-a-scrim-the-main-content-remains-visible-for-context-but",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-the-dialog-below-to-try-a-real-modal-changes-remain-a-draft-until-ok-cancel-or-a-click",
    );
}

#[tessera]
fn time_accessibility_use_cases() {
    prose(
        "ui-offer-both-touch-and-keyboard-entry-people-using-a-screen-reader-a-physical-keyboard-or-an",
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "ui-accessible-input".into(),
    );
}

#[tessera]
fn time_accessibility_interaction() {
    prose(
        "ui-always-expose-manual-time-entry-through-the-keyboard-icon-on-the-dial-a-clock-icon-returns",
    );
    gap(Dp(24.0));
    subheading("ui-targets".into());
    prose(
        "ui-dial-selection-targets-are-48-48dp-the-annotated-illustrations-may-scale-to-fit-this-artic",
    );
    gap(Dp(24.0));
    dial_target_figure();
    gap(Dp(24.0));
    subheading("ui-focus-and-selection".into());
    prose(
        "ui-keep-a-visible-focus-indicator-as-well-as-selected-colors-hour-and-minute-supporting-label",
    );
}

#[tessera]
fn time_keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-action".into()],
        vec![
            vec![
                "ui-tab-shift-tab-6921".into(),
                "ui-move-forward-backward-through-enabled-fields-dial-period-and-actions".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-time-slot-or-button".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    subheading("ui-dial-and-period-navigation-in-this-example".into());
    spec_table(
        vec!["ui-keys".into(), "ui-action".into()],
        vec![
            vec![
                "ui-right-down-arrow".into(),
                "ui-focus-the-next-dial-value-switch-between-am-and-pm".into(),
            ],
            vec![
                "ui-left-up-arrow".into(),
                "ui-focus-the-previous-dial-value-switch-between-am-and-pm".into(),
            ],
            vec![
                "ui-home-end".into(),
                "ui-focus-the-first-last-dial-value-or-am-pm".into(),
            ],
            vec![
                "ui-number-keys".into(),
                "ui-edit-the-focused-hour-or-minute-field-in-input-mode".into(),
            ],
            vec![
                "ui-escape".into(),
                "ui-dismiss-the-modal-without-saving-its-draft".into(),
            ],
        ],
    );
    gap(Dp(16.0));
    prose(
        "ui-arrow-keys-move-dial-focus-space-or-enter-selects-the-focused-value-use-ok-to-save-the-dia",
    );
}

#[tessera]
fn time_figure(
    initial_input: bool,
    horizontal: bool,
    twenty_four: bool,
    kind: FigureKind,
    label: String,
) {
    let sample = Sample {
        state: remember(move || {
            TimePickerState::new(
                if twenty_four { 20 } else { 7 },
                0,
                twenty_four,
                if initial_input {
                    TimePickerDisplayMode::Input
                } else {
                    TimePickerDisplayMode::Picker
                },
            )
        }),
        layout: remember(move || {
            if horizontal {
                TimePickerLayout::Horizontal
            } else {
                TimePickerLayout::Vertical
            }
        }),
        status: remember(|| "ui-ready".to_owned()),
    };
    let geometry = remember(PreciseGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let input = sample
        .state
        .with(|value| value.display_mode() == TimePickerDisplayMode::Input);
    let actual_horizontal = horizontal && !input;
    let canonical_width = if kind == FigureKind::Colors {
        512.0
    } else {
        1024.0
    };
    let canonical_height = if input {
        374.0
    } else if actual_horizontal {
        520.0
    } else {
        640.0
    };
    let available_width = remember(move || Dp(canonical_width));
    let scale = (available_width.get().0 / canonical_width) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            let label = label.clone();
            scaled_box(scale).child(move || {
                let label = label.clone();
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        watch_precise(geometry, "figure")
                            .required_size(Dp(canonical_width), Dp(canonical_height))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(if kind == FigureKind::Colors {
                                    Dp::ZERO
                                } else {
                                    Dp(24.0)
                                }),
                            ),
                    )
                    .children(move || {
                        let label = label.clone();
                        surface()
                            .style(scheme.surface_container_high.into())
                            .shape(Shape::rounded_rectangle(Dp(28.0)))
                            .modifier(watch_precise(geometry, "container").semantics(
                                SemanticsArgs {
                                    test_tag: Some("time-spec-container".into()),
                                    role: Some(Role::Group),
                                    label: Some(I18N.format(
                                        "ui-value-annotated-time-picker",
                                        [("v0", I18N.text(&label).into())],
                                    )),
                                    merge_descendants: true,
                                    ..Default::default()
                                },
                            ))
                            .child(move || {
                                time_figure_contents(
                                    sample,
                                    geometry,
                                    actual_horizontal,
                                    label.clone(),
                                );
                            });
                        if kind != FigureKind::Plain {
                            boxed()
                                .modifier(Modifier::new().fill_max_size())
                                .children(move || {
                                    time_annotations(sample, geometry, actual_horizontal, kind);
                                });
                        }
                    });
            });
        });
    if matches!(kind, FigureKind::Specs | FigureKind::Guidelines) {
        gap(Dp(24.0));
        time_anatomy_legend(input, kind);
    }
    if kind != FigureKind::Colors {
        gap(Dp(12.0));
        text(I18N.text(sample.status.get())).style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn time_anatomy_legend(input: bool, kind: FigureKind) {
    let labels: Vec<&str> = if kind == FigureKind::Guidelines {
        let mut labels = vec![
            "ui-headline",
            "ui-separator",
            "ui-unselected-input-container",
            "ui-unselected-input-text",
            "ui-selected-period-container",
            "ui-selected-period-label",
            "ui-container",
            "ui-period-selector",
            "ui-unselected-period-label",
        ];
        if input {
            labels.extend([
                "ui-text-buttons",
                "ui-clock-icon-button",
                "ui-selected-input-text",
                "ui-selected-input-container",
            ]);
        } else {
            labels.extend([
                "ui-selector-track",
                "ui-selector-handle",
                "ui-text-buttons",
                "ui-keyboard-icon-button",
                "ui-dial-label",
                "ui-dial-container",
                "ui-selected-input-text",
                "ui-selected-input-container",
            ]);
        }
        labels
    } else if input {
        vec![
            "ui-headline",
            "ui-separator",
            "ui-container",
            "ui-am-period",
            "ui-pm-period",
            "ui-text-buttons",
            "ui-clock-icon-button",
            "ui-supporting-labels",
            "ui-selected-input-text",
            "ui-selected-input-container",
        ]
    } else {
        vec![
            "ui-headline",
            "ui-separator",
            "ui-container",
            "ui-am-period",
            "ui-pm-period",
            "ui-selector-center",
            "ui-selector-track",
            "ui-text-buttons",
            "ui-keyboard-icon-button",
            "ui-selector-handle",
            "ui-dial-label",
            "ui-dial-container",
            "ui-selected-input-text",
            "ui-selected-input-container",
        ]
    };
    column().children(move || {
        for (index, label) in labels.iter().enumerate() {
            time_legend_item(index + 1, (*label).to_owned());
        }
    });
}

#[tessera]
fn time_legend_item(number: usize, label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            annotations::marker(number, Dp::ZERO, Dp::ZERO);
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(label))).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn time_figure_contents(
    sample: Sample,
    geometry: State<PreciseGeometry>,
    horizontal: bool,
    label: String,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let input = sample
        .state
        .with(|value| value.display_mode() == TimePickerDisplayMode::Input);
    let body_width = if horizontal {
        524.0
    } else if sample.state.with(TimePickerState::is_24_hour) {
        256.0
    } else {
        280.0
    };
    column()
        .modifier(Modifier::new().padding_all(Dp(24.0)).width(Dp(body_width)))
        .children(move || {
            let label = label.clone();
            text(I18N.text(if input {
                "ui-enter-time"
            } else {
                "ui-select-time"
            }))
            .style(theme.typography.label_medium)
            .color(theme.color_scheme.on_surface_variant)
            .modifier(watch_precise(geometry, "headline").semantics(
                tessera_components::modifier::SemanticsArgs {
                    test_tag: Some("time-spec-headline".into()),
                    merge_descendants: true,
                    ..Default::default()
                },
            ));
            if !horizontal {
                gap(Dp(20.0));
            }
            time_picker()
                .state(sample.state)
                .layout(if horizontal {
                    TimePickerLayout::Horizontal
                } else {
                    TimePickerLayout::Vertical
                })
                .modifier(watch_precise(geometry, "picker"));
            gap(Dp(24.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    watch_precise(geometry, "actions")
                        .fill_max_width()
                        .height(Dp(40.0)),
                )
                .children(move || {
                    icon_button()
                        .standard()
                        .icon(if input {
                            filled::SCHEDULE_SVG
                        } else {
                            filled::KEYBOARD_SVG
                        })
                        .content_color(theme.color_scheme.on_surface_variant)
                        .modifier(watch_precise(geometry, "toggle"))
                        .accessibility_label(
                            I18N.format(
                                "ui-value-value-format-2",
                                [
                                    ("v0", I18N.text(&label).into()),
                                    (
                                        "v1",
                                        I18N.text(
                                            &(if input {
                                                "ui-switch-to-dial"
                                            } else {
                                                "ui-switch-to-time-input-7120"
                                            }),
                                        )
                                        .into(),
                                    ),
                                ],
                            ),
                        )
                        .on_click(move || {
                            sample.state.with_mut(TimePickerState::toggle_display_mode);
                            FocusManager::current().defer_keyboard_until_layout();
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    button()
                        .text()
                        .modifier(watch_precise(geometry, "cancel"))
                        .accessibility_label(
                            I18N.format("ui-value-cancel-time", [("v0", I18N.text(&label).into())]),
                        )
                        .on_click(move || sample.status.set("ui-cancelled".into()))
                        .child(move || {
                            text(I18N.text("ui-cancel"));
                        });
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    button()
                        .text()
                        .modifier(watch_precise(geometry, "confirm"))
                        .accessibility_label(
                            I18N.format(
                                "ui-value-confirm-time",
                                [("v0", I18N.text(&label).into())],
                            ),
                        )
                        .on_click(move || {
                            sample.status.set(sample.state.with(|value| {
                                I18N.format(
                                    "ui-confirmed-value-value",
                                    [
                                        ("v0", format!("{:02}", value.hour()).into()),
                                        ("v1", format!("{:02}", value.minute()).into()),
                                    ],
                                )
                            }))
                        })
                        .child(move || {
                            text(I18N.text("ui-ok"));
                        });
                });
        });
}

fn clock_annotation(state: &TimePickerState) -> (f64, f64) {
    let hour = state.selection() == TimePickerSelection::Hour;
    let fraction = if hour {
        (state.hour() % 12) as f64 / 12.0
    } else {
        state.minute() as f64 / 60.0
    };
    let radius = if hour && state.is_24_hour() && (state.hour() == 0 || state.hour() > 12) {
        69.0
    } else {
        101.0
    };
    (
        fraction * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2,
        radius,
    )
}

fn picker_regions(picker: Rect, input: bool, horizontal: bool, twenty_four: bool) -> [Rect; 5] {
    let field_width = if twenty_four && !horizontal && !input {
        114.0
    } else {
        96.0
    };
    let field_height = if input { 72.0 } else { 80.0 };
    let controls = field_width * 2.0
        + 24.0
        + if !horizontal && !twenty_four {
            64.0
        } else {
            0.0
        };
    let x = picker.x
        + if horizontal {
            0.0
        } else {
            (picker.w - controls).max(0.0) / 2.0
        };
    let y = picker.y
        + if horizontal {
            if twenty_four { 88.0 } else { 72.0 }
        } else {
            0.0
        };
    [
        Rect {
            x,
            y,
            w: field_width,
            h: field_height,
        },
        Rect {
            x: x + field_width,
            y,
            w: 24.0,
            h: field_height,
        },
        Rect {
            x: x + field_width + 24.0,
            y,
            w: field_width,
            h: field_height,
        },
        if horizontal {
            Rect {
                x,
                y: y + field_height + 16.0,
                w: 216.0,
                h: 38.0,
            }
        } else {
            Rect {
                x: x + field_width * 2.0 + 36.0,
                y,
                w: 52.0,
                h: field_height,
            }
        },
        Rect {
            x: picker.x
                + if horizontal {
                    268.0
                } else {
                    (picker.w - 256.0).max(0.0) / 2.0
                },
            y: picker.y + if horizontal { 0.0 } else { 116.0 },
            w: 256.0,
            h: 256.0,
        },
    ]
}

#[tessera]
fn time_annotations(
    sample: Sample,
    geometry: State<PreciseGeometry>,
    horizontal: bool,
    kind: FigureKind,
) {
    let values = geometry.with(annotations::relative_precise_geometry);
    time_annotation_content(sample, values, horizontal, kind);
}

#[tessera]
fn time_annotation_content(
    sample: Sample,
    values: std::collections::BTreeMap<&'static str, Rect>,
    horizontal: bool,
    kind: FigureKind,
) {
    let (
        Some(root),
        Some(container),
        Some(headline),
        Some(picker),
        Some(actions),
        Some(toggle),
        Some(cancel),
        Some(confirm),
    ) = (
        values.get("figure"),
        values.get("container").copied(),
        values.get("headline").copied(),
        values.get("picker").copied(),
        values.get("actions").copied(),
        values.get("toggle").copied(),
        values.get("cancel").copied(),
        values.get("confirm").copied(),
    )
    else {
        return;
    };
    let input = sample
        .state
        .with(|value| value.display_mode() == TimePickerDisplayMode::Input);
    let twenty_four = sample.state.with(TimePickerState::is_24_hour);
    let [hour, separator, minute, period, clock] =
        picker_regions(picker, input, horizontal, twenty_four);
    let hour_text = Rect {
        x: hour.x + 14.0,
        y: hour.y + 8.0,
        w: hour.w - 28.0,
        h: hour.h - 16.0,
    };
    let minute_text = Rect {
        x: minute.x + 14.0,
        y: minute.y + 8.0,
        w: minute.w - 28.0,
        h: minute.h - 16.0,
    };
    let am = Rect {
        x: period.x,
        y: period.y,
        w: period.w,
        h: if horizontal { period.h } else { period.h / 2.0 },
    };
    let pm = Rect {
        x: period.x + if horizontal { period.w / 2.0 } else { 0.0 },
        y: period.y + if horizontal { 0.0 } else { period.h / 2.0 },
        w: if horizontal { period.w / 2.0 } else { period.w },
        h: if horizontal { period.h } else { period.h / 2.0 },
    };
    let (angle, radius) = sample.state.with(clock_annotation);
    let center = Rect {
        x: clock.x + 124.0,
        y: clock.y + 124.0,
        w: 8.0,
        h: 8.0,
    };
    let handle = Rect {
        x: clock.x + 128.0 + angle.cos() * radius - 24.0,
        y: clock.y + 128.0 + angle.sin() * radius - 24.0,
        w: 48.0,
        h: 48.0,
    };
    let track = Rect {
        x: (center.x + 4.0 + handle.x + 24.0) / 2.0,
        y: (center.y + 4.0 + handle.y + 24.0) / 2.0,
        w: 2.0,
        h: 2.0,
    };
    let buttons = Rect {
        x: cancel.x,
        y: cancel.y,
        w: confirm.x + confirm.w - cancel.x,
        h: cancel.h.max(confirm.h),
    };
    let support = Rect {
        x: hour.x,
        y: hour.y + hour.h + 4.0,
        w: minute.x + minute.w - hour.x,
        h: 16.0,
    };
    let dial_label = Rect {
        x: clock.x + 15.0,
        y: clock.y + 116.0,
        w: 24.0,
        h: 24.0,
    };
    let mut svg = String::new();
    if kind == FigureKind::Measurements {
        for rect in [
            Rect {
                x: container.x,
                y: container.y,
                w: 24.0,
                h: container.h,
            },
            Rect {
                x: container.x + container.w - 24.0,
                y: container.y,
                w: 24.0,
                h: container.h,
            },
            Rect {
                x: container.x,
                y: container.y,
                w: container.w,
                h: 24.0,
            },
            Rect {
                x: container.x,
                y: container.y + container.h - 24.0,
                w: container.w,
                h: 24.0,
            },
            Rect {
                x: picker.x,
                y: headline.y + headline.h,
                w: picker.w,
                h: (picker.y - headline.y - headline.h).max(0.0),
            },
            Rect {
                x: picker.x,
                y: picker.y + picker.h,
                w: picker.w,
                h: (actions.y - picker.y - picker.h).max(0.0),
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#ee7da4' fill-opacity='.3'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        for (label, x, y) in [
            ("24".to_owned(), container.x - 36.0, container.y + 4.0),
            (
                format!("{:.0}", hour.w),
                hour.x + hour.w / 2.0 - 10.0,
                container.y - 40.0,
            ),
            ("24".into(), separator.x, container.y - 40.0),
            (
                format!("{:.0}", minute.w),
                minute.x + minute.w / 2.0 - 10.0,
                container.y - 40.0,
            ),
            (
                format!("{:.0}", hour.h),
                hour.x - 48.0,
                hour.y + hour.h / 2.0 - 10.0,
            ),
            (
                "24".into(),
                container.x - 36.0,
                container.y + container.h - 20.0,
            ),
        ] {
            annotations::dimension(label, Dp(x), Dp(y));
        }
        if !twenty_four {
            annotations::dimension(
                if horizontal { "216" } else { "52" }.into(),
                Dp(period.x + period.w / 2.0 - 10.0),
                Dp(period.y - 28.0),
            );
        }
        if !input {
            annotations::outline(&mut svg, clock);
            annotations::outline(&mut svg, handle);
            annotations::dimension(
                "256".into(),
                Dp(clock.x + clock.w + 20.0),
                Dp(clock.y + clock.h / 2.0 - 10.0),
            );
            annotations::dimension(
                "48".into(),
                Dp(clock.x + clock.w + 20.0),
                Dp(handle.y + 14.0),
            );
        }
        annotations::dimension(
            if horizontal {
                "72"
            } else if input {
                "20"
            } else {
                "36"
            }
            .into(),
            Dp(container.x - 36.0),
            Dp(if horizontal {
                hour.y - 50.0
            } else if input {
                headline.y + headline.h + 2.0
            } else {
                hour.y + hour.h + 8.0
            }),
        );
    } else {
        let detailed = matches!(kind, FigureKind::Guidelines | FigureKind::Colors);
        let ink = if kind == FigureKind::Colors
            && use_context::<MaterialTheme>()
                .unwrap()
                .get()
                .color_scheme
                .is_dark
        {
            "#cac4d0"
        } else {
            "#49454f"
        };
        let targets: Vec<(usize, Rect, u8)> = if detailed {
            let mut list = vec![
                (1, headline, 0),
                (2, separator, 2),
                (3, minute, 1),
                (4, minute_text, 1),
                (7, container, 1),
            ];
            if !twenty_four {
                list.extend([(5, am, 1), (6, am, 1), (8, period, 1), (9, pm, 1)]);
            }
            if input {
                list.extend([
                    (10, buttons, 1),
                    (11, toggle, 0),
                    (12, hour_text, 0),
                    (13, hour, 0),
                ]);
            } else {
                list.extend([
                    (10, track, 1),
                    (11, handle, 1),
                    (12, buttons, 1),
                    (13, toggle, 0),
                    (14, dial_label, 0),
                    (15, clock, 0),
                    (16, hour_text, 0),
                    (17, hour, 0),
                ]);
            }
            list
        } else if input {
            vec![
                (1, headline, 0),
                (2, separator, 2),
                (3, container, 1),
                (4, am, 1),
                (5, pm, 1),
                (6, buttons, 1),
                (7, toggle, 0),
                (8, support, 0),
                (9, hour_text, 0),
                (10, hour, 0),
            ]
        } else {
            vec![
                (1, headline, 0),
                (2, separator, 2),
                (3, container, 1),
                (4, am, 1),
                (5, pm, 1),
                (6, center, 1),
                (7, track, 1),
                (8, buttons, 1),
                (9, toggle, 0),
                (10, handle, 0),
                (11, dial_label, 0),
                (12, clock, 0),
                (13, hour_text, 0),
                (14, hour, 0),
            ]
        };
        let mut labels = Vec::with_capacity(targets.len());
        for (number, target, side) in targets {
            let ty = if target.w == container.w {
                container.y + 28.0
            } else if !input && target.w == clock.w && target.h == clock.h {
                clock.y + 68.0
            } else if detailed && number == 5 {
                am.y + 4.0
            } else if detailed && number == 6 {
                am.y + am.h - 4.0
            } else if (detailed && (number == 3 || number == if input { 13 } else { 17 }))
                || (kind == FigureKind::Specs && number == if input { 10 } else { 14 })
            {
                target.y + target.h * 0.24
            } else if (detailed && (number == 4 || number == if input { 12 } else { 16 }))
                || (kind == FigureKind::Specs && number == if input { 9 } else { 13 })
            {
                target.y + target.h * 0.74
            } else {
                target.y + target.h / 2.0
            };
            let x = if kind == FigureKind::Colors && side == 0 {
                16.0
            } else if kind == FigureKind::Colors && side == 1 {
                root.w - 40.0
            } else if side == 0 {
                container.x - 60.0
            } else if side == 1 {
                container.x + container.w + 40.0
            } else {
                target.x + target.w / 2.0 - 12.0
            };
            let y = if side == 2 {
                container.y - 64.0
            } else {
                ty - 16.0
            };
            labels.push((number, target, side, x, y, ty));
        }
        for side in [0, 1] {
            let mut indices = labels
                .iter()
                .enumerate()
                .filter(|(_, label)| label.2 == side)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            indices.sort_by(|a, b| labels[*a].4.total_cmp(&labels[*b].4));
            let mut next = 8.0_f64;
            for &index in &indices {
                labels[index].4 = labels[index].4.max(next);
                next = labels[index].4 + 40.0;
            }
            let mut previous = root.h - 40.0;
            for &index in indices.iter().rev() {
                labels[index].4 = labels[index].4.min(previous);
                previous = labels[index].4 - 40.0;
            }
        }
        for (number, target, side, x, y, ty) in labels {
            let tx = if side == 0 {
                target.x + 4.0
            } else if side == 1 {
                target.x + target.w - 4.0
            } else {
                target.x + target.w / 2.0
            };
            if side == 2 {
                dotted(&mut svg, tx, y + 32.0, ty - y - 32.0, true, ink);
            } else {
                let end_x = if side == 0 { x + 24.0 } else { x };
                let elbow = if side == 0 {
                    end_x + 12.0
                } else {
                    end_x - 12.0
                };
                let center_y = y + 16.0;
                dotted(&mut svg, tx.min(elbow), ty, (tx - elbow).abs(), false, ink);
                dotted(
                    &mut svg,
                    elbow,
                    ty.min(center_y),
                    (ty - center_y).abs(),
                    true,
                    ink,
                );
                dotted(
                    &mut svg,
                    elbow.min(end_x),
                    center_y,
                    (elbow - end_x).abs(),
                    false,
                    ink,
                );
            }
            if kind == FigureKind::Colors {
                time_color_marker(
                    number,
                    time_color_labels(input)[number - 1].into(),
                    Dp(x),
                    Dp(y),
                );
                if target.w > 4.0 && target.w != container.w && target.w != clock.w {
                    dotted(&mut svg, target.x, target.y, target.w, false, ink);
                    dotted(
                        &mut svg,
                        target.x,
                        target.y + target.h,
                        target.w,
                        false,
                        ink,
                    );
                    dotted(&mut svg, target.x, target.y, target.h, true, ink);
                    dotted(&mut svg, target.x + target.w, target.y, target.h, true, ink);
                }
            } else {
                annotations::marker(number, Dp(x), Dp(y));
            }
        }
        if kind != FigureKind::Colors {
            for target in [headline, hour_text, minute_text, buttons, toggle] {
                annotations::outline(&mut svg, target);
            }
        }
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn time_color_marker(number: usize, label: String, x: Dp, y: Dp) {
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
                        "ui-time-color-role-value-value",
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
fn time_usage() {
    prose(
        "ui-set-an-alarm-or-schedule-a-meeting-with-a-specific-hour-and-minute-time-pickers-appear-in",
    );
    gap(Dp(24.0));
    subheading("ui-time-input-picker".into());
    prose(
        "ui-manual-number-entry-must-remain-available-from-the-dial-through-its-keyboard-button-the-in",
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "ui-usage-input".into(),
    );
    gap(Dp(24.0));
    subheading("ui-24-hour-time-selection".into());
    prose(
        "ui-choose-the-clock-format-outside-the-picker-usually-according-to-system-or-app-preferences",
    );
    gap(Dp(24.0));
    time_figure(
        false,
        false,
        true,
        FigureKind::Plain,
        "ui-usage-24-hour".into(),
    );
}

#[tessera]
fn time_guideline_anatomy() {
    time_figure(
        false,
        false,
        false,
        FigureKind::Guidelines,
        "ui-guidelines-dial-anatomy".into(),
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Guidelines,
        "ui-guidelines-input-anatomy".into(),
    );
    gap(Dp(24.0));
    subheading("ui-container".into());
    prose("ui-the-modal-surface-contains-all-controls-and-appears-above-a-scrim");
    gap(Dp(24.0));
    subheading("ui-input-selector".into());
    prose(
        "ui-hours-and-minutes-have-distinct-large-fields-the-focused-field-is-highlighted-and-input-mo",
    );
    gap(Dp(24.0));
    subheading("ui-dial-selector".into());
    prose(
        "ui-select-a-value-by-tapping-its-label-or-dragging-the-dial-the-24-hour-hour-view-adds-an-inn",
    );
    gap(Dp(24.0));
    dial_target_figure();
    gap(Dp(24.0));
    subheading("ui-text-icon-buttons".into());
    prose(
        "ui-the-keyboard-and-clock-buttons-switch-input-methods-cancel-dismisses-the-selection-ok-conf",
    );
    gap(Dp(24.0));
    subheading("ui-landscape-orientation".into());
    prose("ui-place-the-time-selectors-beside-the-dial-when-horizontal-space-is-available");
    gap(Dp(24.0));
    time_figure(
        false,
        true,
        false,
        FigureKind::Plain,
        "ui-guidelines-landscape".into(),
    );
}

#[tessera]
fn time_adaptive() {
    prose(
        "ui-choose-orientation-or-input-mode-to-fit-the-viewport-prefer-manual-input-when-neither-dial",
    );
    gap(Dp(24.0));
    time_figure(
        false,
        true,
        false,
        FigureKind::Plain,
        "ui-adaptive-landscape".into(),
    );
    gap(Dp(24.0));
    subheading("ui-density".into());
    prose(
        "ui-keep-the-dial-s-targets-at-their-intended-size-switch-to-the-input-variant-instead-of-shri",
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "ui-adaptive-input".into(),
    );
}

#[tessera]
fn time_behavior(sample: Sample, modal: State<bool>, draft: Sample, committed: State<(u8, u8)>) {
    prose(
        "ui-select-a-field-and-adjust-the-corresponding-dial-or-type-directly-using-keyboard-input-rel",
    );
    gap(Dp(24.0));
    subheading("ui-appearing-disappearing".into());
    prose(
        "ui-the-modal-keeps-focus-until-ok-cancel-or-dismissal-outside-the-surface-ok-saves-the-draft",
    );
    button()
        .outlined()
        .accessibility_label(I18N.text("ui-open-behavior-time-picker"))
        .on_click(move || open_picker(modal, draft, committed))
        .child(move || {
            text(I18N.text("ui-open-time-picker"));
        });
    gap(Dp(24.0));
    confirmed_time(committed);
    gap(Dp(24.0));
    subheading("ui-toggle-between-dial-input".into());
    controls(sample, "ui-behavior");
    gap(Dp(24.0));
    visual(sample, "ui-behavior");
    gap(Dp(24.0));
    subheading("ui-scrolling".into());
    prose(
        "ui-keep-the-picker-fully-visible-and-stationary-above-the-page-change-orientation-or-variant",
    );
}

#[tessera]
fn dial_target_figure() {
    let state = remember(|| TimePickerState::new(7, 0, false, TimePickerDisplayMode::Picker));
    let geometry = remember(PreciseGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let available_width = remember(|| Dp(676.0));
    let scale = (available_width.get().0 / 676.0) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        watch_precise(geometry, "figure")
                            .required_size(Dp(676.0), Dp(300.0))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(Dp(24.0)),
                            ),
                    )
                    .children(move || {
                        boxed()
                            .modifier(
                                watch_precise(geometry, "clock")
                                    .size(Dp(256.0), Dp(256.0))
                                    .clip_to_bounds(),
                            )
                            .children(move || {
                                time_picker()
                                    .state(state)
                                    .layout(TimePickerLayout::Vertical)
                                    .modifier(
                                        Modifier::new()
                                            .required_size(Dp(280.0), Dp(372.0))
                                            .offset(Dp(-12.0), Dp(-116.0)),
                                    );
                            });
                        boxed()
                            .modifier(Modifier::new().fill_max_size())
                            .children(move || {
                                dial_target_annotations(geometry, state);
                            });
                    });
            });
        });
}

#[tessera]
fn dial_target_annotations(geometry: State<PreciseGeometry>, state: State<TimePickerState>) {
    let values = geometry.with(annotations::relative_precise_geometry);
    let (Some(root), Some(clock)) = (values.get("figure"), values.get("clock").copied()) else {
        return;
    };
    let (angle, radius) = state.with(clock_annotation);
    let x = clock.x + 128.0 + angle.cos() * radius - 24.0;
    let y = clock.y + 128.0 + angle.sin() * radius - 24.0;
    let target = Rect {
        x,
        y,
        w: 48.0,
        h: 48.0,
    };
    let mut svg = String::new();
    annotations::outline(&mut svg, target);
    annotations::dimension("48".into(), Dp(x + 12.0), Dp(y - 28.0));
    annotations::dimension("48".into(), Dp(x - 28.0), Dp(y + 14.0));
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn time_labels() {
    prose(
        "ui-name-the-hour-and-minute-fields-separately-and-associate-each-label-with-its-control-assis",
    );
    gap(Dp(24.0));
    subheading("ui-dial-selector".into());
    spec_table(
        vec![
            "ui-element".into(),
            "ui-accessible-label".into(),
            "ui-role".into(),
        ],
        vec![
            vec![
                "ui-hour-minute".into(),
                "ui-hour-minute".into(),
                "ui-text-input-or-selection-action".into(),
            ],
            vec![
                "ui-period".into(),
                "AM / PM".into(),
                "ui-radio-button".into(),
            ],
            vec![
                "ui-mode-action".into(),
                "ui-switch-to-time-input".into(),
                "ui-button".into(),
            ],
            vec![
                "ui-dial-value".into(),
                "ui-hour-7-of-12-minute-35-of-60".into(),
                "ui-button".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    subheading("ui-input-selector".into());
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "ui-labeled-input".into(),
    );
}

#[cfg(test)]
#[path = "time_pickers_tests.rs"]
mod tests;
