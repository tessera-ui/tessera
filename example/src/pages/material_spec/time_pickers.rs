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
    theme::MaterialTheme,
    time_picker::{
        TimePickerDisplayMode, TimePickerLayout, TimePickerSelection, TimePickerState, time_picker,
        time_picker_dialog,
    },
};
use tessera_ui::{
    Dp, FocusManager, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    annotations::{self, Geometry, Rect, dotted, relative, watch},
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
pub(super) fn time_pickers_spec() {
    let modal = remember(|| false);
    let sample = Sample {
        state: remember(|| TimePickerState::new(9, 41, false, TimePickerDisplayMode::Picker)),
        layout: remember(TimePickerLayout::default),
        status: remember(|| "Ready".to_owned()),
    };
    dialog_provider().is_open(modal.get()).padding(Dp::ZERO).accessibility_label("Time picker dialog")
        .on_close_request(move || modal.set(false))
        .main_content(move || {
            article("Time pickers", "Select a specific time with a clock dial or keyboard input.", vec![
                SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
                SpecSection { title: "Anatomy", content: RenderSlot::new(move || {
                    subheading("Time picker dial".into());
                    time_figure(false,false,false,FigureKind::Specs,"Dial anatomy".into());
                    gap(Dp(24.0));
                    subheading("Time picker input".into());
                    time_figure(true,false,false,FigureKind::Specs,"Input anatomy".into());
                    gap(Dp(24.0));
                    controls(sample, "Anatomy");
                    gap(Dp(24.0)); visual(sample, "Anatomy");
                    gap(Dp(24.0));
                    button().outlined().accessibility_label("Open time picker dialog").on_click(move || modal.set(true))
                        .child(|| { text().content("Open dialog"); });
                    gap(Dp(24.0));
                }) },
                SpecSection { title: "Color", content: RenderSlot::new(colors) },
                SpecSection { title: "States", content: RenderSlot::new(states) },
                SpecSection { title: "Measurements", content: RenderSlot::new(measurements) },
                SpecSection { title: "Configurations", content: RenderSlot::new(configurations) },
            ]).compact_content_inset(Dp(8.0)).pages(vec![
                ArticlePage {title:"Overview",intro:Some(RenderSlot::new(time_variants)),sections:vec![
                    SpecSection {title:"Differences from M2",content:RenderSlot::new(|| {prose("Time pickers use updated dynamic color mappings while retaining dial and keyboard input variants.");gap(Dp(24.0));time_figure(false,false,false,FigureKind::Plain,"Overview dial".into());})},
                ]},
                ArticlePage {title:"Guidelines",intro:Some(RenderSlot::new(|| {time_figure(false,false,false,FigureKind::Plain,"Guideline introduction".into());})),sections:vec![
                    SpecSection {title:"Usage",content:RenderSlot::new(time_usage)},
                    SpecSection {title:"Anatomy",content:RenderSlot::new(time_guideline_anatomy)},
                    SpecSection {title:"Placement",content:RenderSlot::new(move || {prose("Show the picker above a scrim and keep the entire surface visible. Switch orientation or use input fields when space is limited.");gap(Dp(24.0));button().outlined().accessibility_label("Open placement time picker").on_click(move || modal.set(true)).child(|| {text().content("Open time picker");});})},
                    SpecSection {title:"Adaptive design",content:RenderSlot::new(time_adaptive)},
                    SpecSection {title:"Behavior",content:RenderSlot::new(move || {time_behavior(sample,modal);})},
                ]},
                ArticlePage {title:"Accessibility",intro:None,sections:vec![
                    SpecSection {title:"Use cases",content:RenderSlot::new(|| {prose("People must be able to enter hours and minutes manually, choose a clock format and operate the picker with assistive technology.");gap(Dp(24.0));time_figure(true,false,false,FigureKind::Plain,"Accessible input".into());})},
                    SpecSection {title:"Interaction & style",content:RenderSlot::new(|| {prose("Always make keyboard input available from the dial. Use the input variant when there is too little room for a comfortable dial.");gap(Dp(24.0));subheading("Targets".into());prose("Dial selection targets are 48 × 48dp.");gap(Dp(24.0));dial_target_figure();})},
                    SpecSection {title:"Keyboard navigation",content:RenderSlot::new(|| {spec_table(vec!["Keys".into(),"Action".into()],vec![vec!["Tab".into(),"Focus an enabled time slot or action".into()],vec!["Space or Enter".into(),"Activate the focused slot or action".into()]]);})},
                    SpecSection {title:"Labeling elements",content:RenderSlot::new(time_labels)},
                ]},
            ]);
        }).dialog_content(move || { picker_surface(sample, "Modal").modal(modal); });
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
                .accessibility_label(format!("{label} 24-hour format"))
                .on_click(move || {
                    sample
                        .state
                        .with_mut(|state| state.set_is_24_hour(!state.is_24_hour()));
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text().content(if sample.state.with(TimePickerState::is_24_hour) {
                        "24 hours"
                    } else {
                        "12 hours"
                    });
                });
            button()
                .tonal()
                .selected(sample.layout.get() == TimePickerLayout::Horizontal)
                .accessibility_label(format!("{label} horizontal layout"))
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
                    text().content(if sample.layout.get() == TimePickerLayout::Horizontal {
                        "Horizontal"
                    } else {
                        "Vertical"
                    });
                });
            button()
                .tonal()
                .selected(
                    sample
                        .state
                        .with(|state| state.display_mode() == TimePickerDisplayMode::Input),
                )
                .accessibility_label(format!("{label} keyboard input"))
                .on_click(move || {
                    sample.state.with_mut(TimePickerState::toggle_display_mode);
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text().content(
                        if sample
                            .state
                            .with(|state| state.display_mode() == TimePickerDisplayMode::Input)
                        {
                            "Keyboard input"
                        } else {
                            "Clock dial"
                        },
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
    text()
        .content(
            sample
                .state
                .with(|state| format!("{caption} time: {:02}:{:02}", state.hour(), state.minute())),
        )
        .style(type_style(16.0, 24.0));
    text()
        .content(sample.status.get())
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn picker_surface(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let confirm_label = format!("{label} confirm time");
    let cancel_label = format!("{label} cancel time");
    surface()
        .style(theme.color_scheme.surface_container_high.into())
        .shape(Shape::rounded_rectangle(Dp(28.0)))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} time preview")),
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
                        .modal_optional(modal);
                });
        });
}

#[tessera]
fn picker_contents(
    sample: Sample,
    #[prop(into)] confirm_label: String,
    #[prop(into)] cancel_label: String,
    modal: Option<State<bool>>,
) {
    time_picker_dialog()
        .state(sample.state)
        .picker_layout(sample.layout.get())
        .confirm_button(move || {
            button()
                .text()
                .accessibility_label(confirm_label.clone())
                .on_click(move || {
                    sample.status.set(sample.state.with(|state| {
                        format!("Confirmed {:02}:{:02}", state.hour(), state.minute())
                    }));
                    if let Some(modal) = modal {
                        modal.set(false);
                    }
                })
                .child(|| {
                    text().content("OK");
                });
        })
        .dismiss_button(move || {
            button()
                .text()
                .accessibility_label(cancel_label.clone())
                .on_click(move || {
                    sample.status.set("Cancelled".into());
                    if let Some(modal) = modal {
                        modal.set(false);
                    }
                })
                .child(|| {
                    text().content("Cancel");
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
        status: remember(|| "Ready".to_owned()),
    };
    controls(sample, label.clone());
    gap(Dp(24.0));
    visual(sample, label);
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Container".into(), "Surface container high".into()],
            vec![
                "Selected time field".into(),
                "Primary container / on primary container".into(),
            ],
            vec![
                "Unselected time field / clock dial".into(),
                "Surface container highest / on surface".into(),
            ],
            vec![
                "Selected period".into(),
                "Tertiary container / on tertiary container".into(),
            ],
            vec!["Dial selector".into(), "Primary / on primary".into()],
            vec!["Period outline".into(), "Outline".into()],
        ],
    );
}

#[tessera]
fn colors() {
    subheading("Time picker dial color".into());
    prose(
        "The container uses surface container high. The selected field uses primary container; the clock handle and track use primary. Period selection uses tertiary container.",
    );
    gap(Dp(24.0));
    time_figure(false, false, false, FigureKind::Plain, "Dial color".into());
    gap(Dp(32.0));
    subheading("Time picker input color".into());
    prose(
        "Input fields retain the same selected and unselected color roles. Supporting labels and the mode action use on surface variant.",
    );
    gap(Dp(24.0));
    time_figure(true, false, false, FigureKind::Plain, "Input color".into());
}

#[tessera]
fn states() {
    text().content("Hover, focus with Tab, or use the arrow keys on the dial. Activate a value with Enter or Space.").style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    example("States", false);
}

#[tessera]
fn measurements() {
    subheading("Time picker dial - vertical".into());
    time_figure(
        false,
        false,
        false,
        FigureKind::Measurements,
        "Vertical measurements".into(),
    );
    gap(Dp(32.0));
    subheading("Time picker dial - horizontal".into());
    time_figure(
        false,
        true,
        false,
        FigureKind::Measurements,
        "Horizontal measurements".into(),
    );
    gap(Dp(32.0));
    subheading("Time picker input".into());
    time_figure(
        true,
        false,
        false,
        FigureKind::Measurements,
        "Input measurements".into(),
    );
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Measurement".into()],
        vec![
            vec!["Container padding".into(), "24dp".into()],
            vec![
                "Dial time selector".into(),
                "96 × 80dp · 114dp wide for vertical 24h".into(),
            ],
            vec!["Input time field".into(), "96 × 72dp".into()],
            vec![
                "Vertical period selector".into(),
                "52 × 80dp · 72dp high for input".into(),
            ],
            vec!["Horizontal period selector".into(), "216 × 38dp".into()],
            vec!["Clock dial".into(), "256dp".into()],
            vec!["Selector handle / center".into(), "48 / 8dp".into()],
            vec!["Selector track".into(), "2dp".into()],
        ],
    );
}

#[tessera]
fn configurations() {
    subheading("Vertical orientation and horizontal orientation".into());
    time_figure(
        false,
        false,
        false,
        FigureKind::Plain,
        "Vertical configuration".into(),
    );
    gap(Dp(24.0));
    time_figure(
        false,
        true,
        false,
        FigureKind::Plain,
        "Horizontal configuration".into(),
    );
    gap(Dp(32.0));
    subheading("24-hour time picker dial".into());
    time_figure(false, false, true, FigureKind::Plain, "24-hour dial".into());
    gap(Dp(32.0));
    subheading("12-hour and 24-hour time picker inputs".into());
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "12-hour input".into(),
    );
    gap(Dp(24.0));
    time_figure(true, false, true, FigureKind::Plain, "24-hour input".into());
    gap(Dp(24.0));
    text().content("Switch orientation, clock format and input mode. Narrow containers automatically use the vertical layout.").style(type_style(16.0, 24.0));
    gap(Dp(24.0));
    example("Configurations", false);
}

#[derive(Clone, Copy, Default, PartialEq)]
enum FigureKind {
    #[default]
    Plain,
    Specs,
    Guidelines,
    Measurements,
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn time_variants() {
    prose(
        "Time pickers select one specific time in a modal dialog. Dial and keyboard variants support hours, minutes and a 12- or 24-hour clock.",
    );
    gap(Dp(24.0));
    time_figure(
        false,
        false,
        false,
        FigureKind::Plain,
        "Overview time picker dial".into(),
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "Overview time picker input".into(),
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
        status: remember(|| "Ready".to_owned()),
    };
    let geometry = remember(Geometry::new);
    #[cfg(test)]
    tests::GEOMETRY.set(Some(geometry));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let input = sample
        .state
        .with(|value| value.display_mode() == TimePickerDisplayMode::Input);
    let actual_horizontal = horizontal
        && !input
        && geometry.with(|value| {
            value
                .get("figure")
                .is_none_or(|bounds| Dp::from_pixels_f32(bounds.width.to_f32()).0 >= 668.0)
        });
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(if input {
                    400.0
                } else if actual_horizontal {
                    552.0
                } else {
                    680.0
                }))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            let label = label.clone();
            surface()
                .style(scheme.surface_container_high.into())
                .shape(Shape::rounded_rectangle(Dp(28.0)))
                .modifier(watch(geometry, "container").semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!("{label} annotated time picker")),
                    merge_descendants: true,
                    ..Default::default()
                }))
                .child(move || {
                    time_figure_contents(sample, geometry, actual_horizontal, label.clone());
                });
            if kind != FigureKind::Plain {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        time_annotations(sample, geometry, actual_horizontal, kind);
                    });
            }
        });
    if matches!(kind, FigureKind::Specs | FigureKind::Guidelines) {
        gap(Dp(24.0));
        time_anatomy_legend(input, kind);
    }
    gap(Dp(12.0));
    text()
        .content(sample.status.get())
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn time_anatomy_legend(input: bool, kind: FigureKind) {
    let labels: Vec<&str> = if kind == FigureKind::Guidelines {
        let mut labels = vec![
            "Headline",
            "Separator",
            "Unselected input container",
            "Unselected input text",
            "Selected period container",
            "Selected period label",
            "Container",
            "Period selector",
            "Unselected period label",
        ];
        if input {
            labels.extend([
                "Text buttons",
                "Clock icon button",
                "Selected input text",
                "Selected input container",
            ]);
        } else {
            labels.extend([
                "Selector track",
                "Selector handle",
                "Text buttons",
                "Keyboard icon button",
                "Dial label",
                "Dial container",
                "Selected input text",
                "Selected input container",
            ]);
        }
        labels
    } else if input {
        vec![
            "Headline",
            "Separator",
            "Container",
            "AM period",
            "PM period",
            "Text buttons",
            "Clock icon button",
            "Supporting labels",
            "Selected input text",
            "Selected input container",
        ]
    } else {
        vec![
            "Headline",
            "Separator",
            "Container",
            "AM period",
            "PM period",
            "Selector center",
            "Selector track",
            "Text buttons",
            "Keyboard icon button",
            "Selector handle",
            "Dial label",
            "Dial container",
            "Selected input text",
            "Selected input container",
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
            text().content(label.clone()).style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn time_figure_contents(
    sample: Sample,
    geometry: State<Geometry>,
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
            text()
                .content(if input { "Enter time" } else { "Select time" })
                .style(theme.typography.label_medium)
                .color(theme.color_scheme.on_surface_variant)
                .modifier(watch(geometry, "headline"));
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
                .modifier(watch(geometry, "picker"));
            gap(Dp(24.0));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(watch(geometry, "actions").fill_max_width().height(Dp(40.0)))
                .children(move || {
                    icon_button()
                        .standard()
                        .icon(if input {
                            filled::SCHEDULE_SVG
                        } else {
                            filled::KEYBOARD_SVG
                        })
                        .content_color(theme.color_scheme.on_surface_variant)
                        .modifier(watch(geometry, "toggle"))
                        .accessibility_label(format!("{label} switch input mode"))
                        .on_click(move || {
                            sample.state.with_mut(TimePickerState::toggle_display_mode);
                            FocusManager::current().defer_keyboard_until_layout();
                        });
                    spacer().modifier(Modifier::new().weight(1.0));
                    button()
                        .text()
                        .modifier(watch(geometry, "cancel"))
                        .accessibility_label(format!("{label} cancel time"))
                        .on_click(move || sample.status.set("Cancelled".into()))
                        .child(|| {
                            text().content("Cancel");
                        });
                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                    button()
                        .text()
                        .modifier(watch(geometry, "confirm"))
                        .accessibility_label(format!("{label} confirm time"))
                        .on_click(move || {
                            sample.status.set(sample.state.with(|value| {
                                format!("Confirmed {:02}:{:02}", value.hour(), value.minute())
                            }))
                        })
                        .child(|| {
                            text().content("OK");
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
fn time_annotations(sample: Sample, geometry: State<Geometry>, horizontal: bool, kind: FigureKind) {
    let values = geometry.get();
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
        relative(&values, "container"),
        relative(&values, "headline"),
        relative(&values, "picker"),
        relative(&values, "actions"),
        relative(&values, "toggle"),
        relative(&values, "cancel"),
        relative(&values, "confirm"),
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
        let targets: Vec<(usize, Rect, u8)> = if kind == FigureKind::Guidelines {
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
        for (number, target, side) in targets {
            let ty = if target.w == container.w {
                container.y + 28.0
            } else if !input && target.w == clock.w && target.h == clock.h {
                clock.y + 68.0
            } else if kind == FigureKind::Guidelines && number == 5 {
                am.y + 4.0
            } else if kind == FigureKind::Guidelines && number == 6 {
                am.y + am.h - 4.0
            } else if (kind == FigureKind::Guidelines
                && (number == 3 || number == if input { 13 } else { 17 }))
                || (kind == FigureKind::Specs && number == if input { 10 } else { 14 })
            {
                target.y + target.h * 0.24
            } else if (kind == FigureKind::Guidelines
                && (number == 4 || number == if input { 12 } else { 16 }))
                || (kind == FigureKind::Specs && number == if input { 9 } else { 13 })
            {
                target.y + target.h * 0.74
            } else {
                target.y + target.h / 2.0
            };
            let x = if side == 0 {
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
            let tx = if side == 0 {
                target.x + 4.0
            } else if side == 1 {
                target.x + target.w - 4.0
            } else {
                target.x + target.w / 2.0
            };
            if side == 2 {
                dotted(&mut svg, tx, y + 32.0, ty - y - 32.0, true, "#49454f");
            } else {
                dotted(
                    &mut svg,
                    if side == 0 { x + 24.0 } else { tx },
                    ty,
                    if side == 0 { tx - x - 24.0 } else { x - tx },
                    false,
                    "#49454f",
                );
            }
            annotations::marker(number, Dp(x), Dp(y));
        }
        for target in [headline, hour_text, minute_text, buttons, toggle] {
            annotations::outline(&mut svg, target);
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn time_usage() {
    prose(
        "Choose a specific time for an alarm or meeting. A modal picker supports hours and minutes, rather than fine-grained stopwatch measurements.",
    );
    gap(Dp(24.0));
    subheading("Time input picker".into());
    prose("Manual number entry must remain available from the dial through its keyboard button.");
    gap(Dp(24.0));
    time_figure(true, false, false, FigureKind::Plain, "Usage input".into());
    gap(Dp(24.0));
    subheading("24-hour time selection".into());
    prose(
        "Choose the clock format outside the picker, usually according to system or app preferences. A 24-hour picker omits AM and PM.",
    );
    gap(Dp(24.0));
    time_figure(
        false,
        false,
        true,
        FigureKind::Plain,
        "Usage 24-hour".into(),
    );
}

#[tessera]
fn time_guideline_anatomy() {
    time_figure(
        false,
        false,
        false,
        FigureKind::Guidelines,
        "Guidelines dial anatomy".into(),
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Guidelines,
        "Guidelines input anatomy".into(),
    );
    gap(Dp(24.0));
    subheading("Container".into());
    prose("The modal surface contains all controls and appears above a scrim.");
    gap(Dp(24.0));
    subheading("Input selector".into());
    prose(
        "Hours and minutes have distinct large fields. The focused field is highlighted, and input mode includes a supporting label below each field.",
    );
    gap(Dp(24.0));
    subheading("Dial selector".into());
    prose(
        "Select a value by tapping its label or dragging the dial. The 24-hour hour view adds an inner ring.",
    );
    gap(Dp(24.0));
    dial_target_figure();
    gap(Dp(24.0));
    subheading("Text & icon buttons".into());
    prose(
        "The keyboard and clock buttons switch input methods. Cancel dismisses the selection; OK confirms it.",
    );
    gap(Dp(24.0));
    subheading("Landscape orientation".into());
    prose("Place the time selectors beside the dial when horizontal space is available.");
    gap(Dp(24.0));
    time_figure(
        false,
        true,
        false,
        FigureKind::Plain,
        "Guidelines landscape".into(),
    );
}

#[tessera]
fn time_adaptive() {
    prose(
        "Choose orientation or input mode to fit the viewport. Prefer manual input when neither dial orientation can remain fully visible.",
    );
    gap(Dp(24.0));
    time_figure(
        false,
        true,
        false,
        FigureKind::Plain,
        "Adaptive landscape".into(),
    );
    gap(Dp(24.0));
    subheading("Density".into());
    prose(
        "Keep the dial's targets at their intended size. Switch to the input variant instead of shrinking the dial.",
    );
    gap(Dp(24.0));
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "Adaptive input".into(),
    );
}

#[tessera]
fn time_behavior(sample: Sample, modal: State<bool>) {
    prose(
        "Select a field and adjust the corresponding dial, or type directly using keyboard input.",
    );
    gap(Dp(24.0));
    subheading("Appearing & disappearing".into());
    prose(
        "The modal keeps focus until OK, Cancel or dismissal outside the surface. Confirming saves the chosen time; cancellation dismisses it.",
    );
    button()
        .outlined()
        .accessibility_label("Open behavior time picker")
        .on_click(move || modal.set(true))
        .child(|| {
            text().content("Open time picker");
        });
    gap(Dp(24.0));
    subheading("Toggle between dial & input".into());
    controls(sample, "Behavior");
    gap(Dp(24.0));
    visual(sample, "Behavior");
    gap(Dp(24.0));
    subheading("Scrolling".into());
    prose(
        "Keep the picker fully visible and stationary above the page. Change orientation or variant rather than making the dial scroll.",
    );
}

#[tessera]
fn dial_target_figure() {
    let state = remember(|| TimePickerState::new(7, 0, false, TimePickerDisplayMode::Picker));
    let geometry = remember(Geometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(352.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    watch(geometry, "clock")
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
}

#[tessera]
fn dial_target_annotations(geometry: State<Geometry>, state: State<TimePickerState>) {
    let values = geometry.get();
    let (Some(root), Some(clock)) = (values.get("figure"), relative(&values, "clock")) else {
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
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn time_labels() {
    prose(
        "Name the hour and minute fields separately. A dial value communicates its value and whether it represents hours or minutes. Name mode and dialog actions by what they do.",
    );
    gap(Dp(24.0));
    subheading("Dial selector".into());
    spec_table(
        vec!["Element".into(), "Accessible label".into(), "Role".into()],
        vec![
            vec![
                "Hour / Minute".into(),
                "Hour / Minute".into(),
                "Text input or selection action".into(),
            ],
            vec!["Period".into(), "AM / PM".into(), "Radio button".into()],
            vec![
                "Mode action".into(),
                "Switch to time input".into(),
                "Button".into(),
            ],
            vec![
                "Dial value".into(),
                "Value and time unit".into(),
                "Button".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    subheading("Input selector".into());
    time_figure(
        true,
        false,
        false,
        FigureKind::Plain,
        "Labeled input".into(),
    );
}

#[cfg(test)]
#[path = "time_pickers_tests.rs"]
mod tests;
