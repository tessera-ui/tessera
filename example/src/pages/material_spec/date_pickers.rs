//! Interactive modal date picker and input specifications.
//!
//! ## Usage
//!
//! Choose single dates or ranges with calendars and keyboard input.

use std::collections::BTreeMap;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    date_picker::{
        CalendarDate, DatePickerDisplayMode, DatePickerState, DatePickerView, DateRangePickerState,
        DockedDatePickerView, Weekday, YearMonth, date_picker_dialog, date_range_picker,
        docked_date_picker,
    },
    dialog::dialog_provider,
    flow_row::flow_row,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    surface::surface,
    text::text,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Dp, FocusManager, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use super::{
    ACCENT,
    annotations::{self, ObservedGeometry, Rect, dotted, watch_observed},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, subheading, type_style,
};

#[derive(Clone, Copy, PartialEq)]
struct Sample {
    single: State<DatePickerState>,
    range: State<DateRangePickerState>,
    range_mode: State<bool>,
    dark: State<bool>,
    status: State<String>,
}

fn date(day: u8) -> CalendarDate {
    CalendarDate::new(2026, 7, day).expect("valid example date")
}

fn single(mode: DatePickerDisplayMode) -> DatePickerState {
    DatePickerState::new(
        Some(date(18)),
        YearMonth::new(2026, 7),
        2020..=2030,
        None,
        None,
        mode,
    )
}

fn range(mode: DatePickerDisplayMode) -> DateRangePickerState {
    DateRangePickerState::new(
        Some(date(10)),
        Some(date(16)),
        YearMonth::new(2026, 7),
        2020..=2030,
        None,
        None,
        mode,
    )
}

fn formatted(value: Option<CalendarDate>) -> String {
    value
        .map(|date| format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()))
        .unwrap_or_else(|| "None".into())
}

#[tessera]
pub(super) fn date_pickers_spec() {
    let calendar = Sample {
        single: remember(|| single(DatePickerDisplayMode::Picker)),
        range: remember(|| range(DatePickerDisplayMode::Picker)),
        range_mode: remember(|| false),
        dark: remember(|| false),
        status: remember(|| "Ready".to_owned()),
    };
    let input = Sample {
        single: remember(|| single(DatePickerDisplayMode::Input)),
        range: remember(|| range(DatePickerDisplayMode::Input)),
        range_mode: remember(|| false),
        dark: remember(|| false),
        status: remember(|| "Ready".to_owned()),
    };
    let modal = remember(|| false);
    let modal_input = remember(|| false);
    let mut pages = Vec::new();
    for (title, chapters) in date_chapters() {
        let mut sections = Vec::new();
        for chapter in chapters {
            sections.push(SpecSection {
                title: chapter.title,
                content: RenderSlot::new(move || {
                    date_chapter(chapter.clone());
                }),
            });
        }
        pages.push(ArticlePage {
            title,
            intro: if title == "Overview" {
                Some(RenderSlot::new(date_overview))
            } else {
                None
            },
            sections,
        });
    }
    dialog_provider()
        .is_open(modal.get())
        .padding(Dp::ZERO)
        .accessibility_label("Date picker dialog")
        .on_close_request(move || modal.set(false))
        .main_content(move || {
            article(
                "Date pickers",
                "Date pickers let people select a date, or a range of dates",
                vec![
                    SpecSection {
                        title: "Tokens & specs",
                        content: RenderSlot::new(tokens),
                    },
                    SpecSection {
                        title: "Docked date picker",
                        content: RenderSlot::new(docked_section),
                    },
                    SpecSection {
                        title: "Modal date picker",
                        content: RenderSlot::new(move || {
                            date_figure(Figure::Day, true, false);
                            gap(Dp(32.0));
                            example(calendar, "Calendar", modal, modal_input, false);
                            gap(Dp(24.0));
                            anatomy_legend(Figure::Day, DockedDatePickerView::Calendar);
                            gap(Dp(32.0));
                            subheading("Modal date picker color".into());
                            gap(Dp(24.0));
                            date_colors(Figure::Day);
                            gap(Dp(32.0));
                            subheading("Modal date picker measurements".into());
                            gap(Dp(24.0));
                            date_figure(Figure::Day, false, true);
                            gap(Dp(24.0));
                            measurements(false);
                            gap(Dp(32.0));
                            subheading("Modal date picker configurations".into());
                            gap(Dp(24.0));
                            modal_configurations();
                        }),
                    },
                    SpecSection {
                        title: "Modal date input",
                        content: RenderSlot::new(move || {
                            date_figure(Figure::Input, true, false);
                            gap(Dp(32.0));
                            example(input, "Input", modal, modal_input, true);
                            gap(Dp(24.0));
                            anatomy_legend(Figure::Input, DockedDatePickerView::Calendar);
                            gap(Dp(32.0));
                            subheading("Modal date input color".into());
                            gap(Dp(24.0));
                            date_colors(Figure::Input);
                            gap(Dp(32.0));
                            subheading("Modal date input measurements".into());
                            gap(Dp(24.0));
                            date_figure(Figure::Input, false, true);
                            gap(Dp(24.0));
                            measurements(true);
                            gap(Dp(32.0));
                            subheading("Modal date input configurations".into());
                            gap(Dp(24.0));
                            input_configurations();
                        }),
                    },
                    SpecSection {
                        title: "Element states",
                        content: RenderSlot::new(states),
                    },
                ],
            )
            .compact_content_inset(Dp(8.0))
            .pages(pages.clone());
        })
        .dialog_content(move || {
            themed_picker(if modal_input.get() { input } else { calendar }, "Modal").modal(modal);
        });
}

#[tessera]
fn example(
    sample: Sample,
    #[prop(into)] label: String,
    modal: State<bool>,
    modal_input: State<bool>,
    input: bool,
) {
    let caption = label.clone();
    flow_row()
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            button()
                .tonal()
                .selected(sample.range_mode.get())
                .accessibility_label(format!("{label} range selection"))
                .on_click(move || {
                    sample.range_mode.set(!sample.range_mode.get());
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(move || {
                    text().content(if sample.range_mode.get() {
                        "Date range"
                    } else {
                        "Single date"
                    });
                });
            button()
                .tonal()
                .selected(sample.dark.get())
                .accessibility_label(format!("{label} dark theme"))
                .on_click(move || sample.dark.set(!sample.dark.get()))
                .child(move || {
                    text().content(if sample.dark.get() {
                        "Dark theme"
                    } else {
                        "Light theme"
                    });
                });
        });
    gap(Dp(24.0));
    let label = caption.clone();
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            themed_picker(sample, label.clone());
        });
    gap(Dp(16.0));
    text()
        .content(if sample.range_mode.get() {
            sample.range.with(|state| {
                format!(
                    "{caption} range: {} – {}",
                    formatted(state.selected_start_date()),
                    formatted(state.selected_end_date())
                )
            })
        } else {
            sample
                .single
                .with(|state| format!("{caption} date: {}", formatted(state.selected_date())))
        })
        .style(type_style(16.0, 24.0));
    text()
        .content(sample.status.get())
        .style(type_style(14.0, 20.0));
    gap(Dp(16.0));
    button()
        .outlined()
        .accessibility_label(format!("Open {caption} date dialog"))
        .on_click(move || {
            modal_input.set(input);
            modal.set(true);
        })
        .child(|| {
            text().content("Open dialog");
        });
}

#[tessera]
fn themed_picker(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    material_theme()
        .theme(move || MaterialTheme::from_seed(ACCENT, sample.dark.get()))
        .child(move || {
            picker_surface(sample, label.clone()).modal_optional(modal);
        });
}

#[tessera]
fn picker_surface(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let confirm_label = format!("{label} confirm date");
    let cancel_label = format!("{label} cancel date");
    surface()
        .style(scheme.surface_container_high.into())
        .shape(Shape::rounded_rectangle(Dp(28.0)))
        .modifier(Modifier::new().semantics(SemanticsArgs {
            role: Some(Role::Group),
            label: Some(format!("{label} date preview")),
            merge_descendants: true,
            ..Default::default()
        }))
        .child(move || {
            let confirm_label = confirm_label.clone();
            let cancel_label = cancel_label.clone();
            if sample.range_mode.get() {
                column()
                    .modifier(Modifier::new().width(Dp(360.0)))
                    .children(move || {
                        date_range_picker()
                            .today(illustration_today())
                            .state(sample.range)
                            .first_day_of_week(Weekday::Sunday)
                            .calendar_height(Dp(336.0));
                        let confirm_label = confirm_label.clone();
                        let cancel_label = cancel_label.clone();
                        row()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
                            .main_axis_alignment(MainAxisAlignment::End)
                            .children(move || {
                                actions(sample, confirm_label.clone(), cancel_label.clone())
                                    .modal_optional(modal);
                            });
                    });
            } else {
                let confirm_label = confirm_label.clone();
                let cancel_label = cancel_label.clone();
                date_picker_dialog()
                    .picker_today(illustration_today())
                    .state(sample.single)
                    .picker_first_day_of_week(Weekday::Sunday)
                    .confirm_button(move || {
                        confirm(sample, confirm_label.clone()).modal_optional(modal);
                    })
                    .dismiss_button(move || {
                        cancel(sample, cancel_label.clone()).modal_optional(modal);
                    });
            }
        });
}

#[tessera]
fn actions(
    sample: Sample,
    #[prop(into)] confirm_label: String,
    #[prop(into)] cancel_label: String,
    modal: Option<State<bool>>,
) {
    cancel(sample, cancel_label).modal_optional(modal);
    confirm(sample, confirm_label).modal_optional(modal);
}

#[tessera]
fn confirm(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    button()
        .text()
        .enabled(if sample.range_mode.get() {
            sample.range.with(DateRangePickerState::is_complete)
        } else {
            sample.single.with(|state| state.selected_date().is_some())
        })
        .accessibility_label(label)
        .on_click(move || {
            sample.status.set(if sample.range_mode.get() {
                sample.range.with(|state| {
                    format!(
                        "Confirmed range: {} – {}",
                        formatted(state.selected_start_date()),
                        formatted(state.selected_end_date())
                    )
                })
            } else {
                sample
                    .single
                    .with(|state| format!("Confirmed date: {}", formatted(state.selected_date())))
            });
            if let Some(modal) = modal {
                modal.set(false);
            }
        })
        .child(|| {
            text().content("OK");
        });
}

#[tessera]
fn cancel(sample: Sample, #[prop(into)] label: String, modal: Option<State<bool>>) {
    button()
        .text()
        .accessibility_label(label)
        .on_click(move || {
            sample.status.set("Cancelled".into());
            if let Some(modal) = modal {
                modal.set(false);
            }
        })
        .child(|| {
            text().content("Cancel");
        });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec!["Modal container".into(), "Surface container high".into()],
            vec!["Selected date".into(), "Primary / on primary".into()],
            vec![
                "Range highlight".into(),
                "Secondary container / on secondary container".into(),
            ],
            vec!["Today outline".into(), "Primary".into()],
            vec!["Headline / unselected date".into(), "On surface".into()],
            vec![
                "Supporting text / weekdays / icons".into(),
                "On surface variant".into(),
            ],
            vec!["Divider".into(), "Outline variant".into()],
        ],
    );
}

#[tessera]
fn measurements(input: bool) {
    let mut rows = vec![
        vec![
            "Single-date container".into(),
            "360dp maximum width · 28dp corners".into(),
        ],
        vec!["Header left / right padding".into(), "24 / 12dp".into()],
        vec!["Date input".into(), "MM/DD/YYYY".into()],
        vec!["Date input height".into(), "56dp".into()],
    ];
    if !input {
        rows.push(vec!["Calendar date target".into(), "48 × 48dp".into()]);
        rows.push(vec!["Selected date indicator".into(), "40dp".into()]);
        rows.push(vec!["Calendar columns".into(), "7".into()]);
    }
    spec_table(vec!["Attribute".into(), "Value".into()], rows);
}

#[path = "date_instructional.rs"]
mod instructional_figures;

#[path = "date_states.rs"]
mod state_figures;

#[tessera]
fn states() {
    state_figures::states();
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Figure {
    #[default]
    None,
    Docked,
    Day,
    Input,
    Range,
    RangeInput,
    FullScreen,
}

#[derive(Clone, PartialEq)]
struct DateChapter {
    title: &'static str,
    text: &'static str,
    figure: Figure,
    children: Vec<DateChapter>,
}

fn chapter(title: &'static str, text: &'static str, figure: Figure) -> DateChapter {
    DateChapter {
        title,
        text,
        figure,
        children: Vec::new(),
    }
}

fn date_chapters() -> Vec<(&'static str, Vec<DateChapter>)> {
    vec![
        (
            "Overview",
            vec![chapter(
                "Differences from M2",
                "Larger typography and spacing support 48dp targets. Dynamic color maps the calendar to its context. Docked, modal and modal input names describe how each picker behaves.",
                Figure::Day,
            )],
        ),
        (
            "Guidelines",
            vec![
                chapter(
                    "Usage",
                    "Choose a calendar for nearby dates and text entry for dates in the distant past or future. Offer a single date or an inclusive range according to the task.",
                    Figure::Docked,
                ),
                DateChapter {
                    title: "Anatomy",
                    text: "Each variant exposes its selection, navigation and confirmation controls.",
                    figure: Figure::None,
                    children: vec![
                        chapter(
                            "Docked date picker",
                            "An input field anchors a calendar with separate month and year menus, date cells and optional actions.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Modal date picker",
                            "Supporting text and a headline describe the selection. The edit action switches entry method; month navigation, date cells and actions complete the dialog.",
                            Figure::Day,
                        ),
                        chapter(
                            "Modal date input",
                            "The calendar action returns to visual selection. A labeled input accepts the date before confirmation.",
                            Figure::Input,
                        ),
                        chapter(
                            "Full-screen date picker",
                            "A close affordance and Save action frame the range calendar on compact windows.",
                            Figure::FullScreen,
                        ),
                    ],
                },
                DateChapter {
                    title: "Docked date picker",
                    text: "Use an anchored calendar alongside direct date entry.",
                    figure: Figure::None,
                    children: vec![
                        chapter(
                            "Usage",
                            "Keep the editable field visible when its calendar is open. Either input method can complete the task.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Behavior",
                            "Activate the calendar icon to reveal the picker immediately below the field; confirm or cancel to dismiss it.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Month selection",
                            "Use previous and next controls or open the month menu to jump within the year.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Year selection",
                            "Use the year controls or open the year menu to navigate distant dates.",
                            Figure::Docked,
                        ),
                    ],
                },
                DateChapter {
                    title: "Modal date picker",
                    text: "A dialog keeps attention on a pending date selection.",
                    figure: Figure::None,
                    children: vec![
                        chapter(
                            "Behavior",
                            "Use previous and next month controls for nearby dates. The month and year selector opens year navigation.",
                            Figure::Day,
                        ),
                        chapter(
                            "Date range selection",
                            "Choose a start and end date for bookings and reservations. Intermediate dates share a connected highlight; vertical scrolling reveals other months.",
                            Figure::Range,
                        ),
                    ],
                },
                DateChapter {
                    title: "Modal date input",
                    text: "Direct entry is useful when the intended date is already known.",
                    figure: Figure::None,
                    children: vec![
                        chapter(
                            "Usage",
                            "Offer numeric entry for a date or date range, especially dates far from the current month.",
                            Figure::Input,
                        ),
                        chapter(
                            "Behavior",
                            "The edit and calendar controls switch entry methods without losing the current selection.",
                            Figure::RangeInput,
                        ),
                        chapter(
                            "Compact breakpoint",
                            "A full-screen picker provides readable content and touch targets in a compact window.",
                            Figure::FullScreen,
                        ),
                        chapter(
                            "Medium and expanded breakpoints",
                            "Anchor the calendar below its field on larger windows; keep the field available for direct entry.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Selection",
                            "A selected endpoint uses a filled indicator. The dates between the endpoints have a connected tonal highlight.",
                            Figure::Range,
                        ),
                        chapter(
                            "Appearing and disappearing",
                            "Confirm or cancel to dismiss modal selection. A docked calendar appears below the input field.",
                            Figure::Docked,
                        ),
                        chapter(
                            "Responsive layout",
                            "Keep the calendar at its intended component size; use surrounding space rather than stretching its date cells.",
                            Figure::Day,
                        ),
                    ],
                },
            ],
        ),
        (
            "Accessibility",
            vec![
                chapter(
                    "Use cases",
                    "Support visual date selection and manual text entry so the calendar is optional for keyboard and assistive technology users.",
                    Figure::Docked,
                ),
                chapter(
                    "Interaction & style",
                    "Preserve 48dp touch targets and provide a clearly labeled edit control for switching to date input.",
                    Figure::Day,
                ),
                chapter(
                    "Date entry methods",
                    "Keep the text field and calendar button as separate keyboard stops. The calendar button opens the optional visual picker.",
                    Figure::Docked,
                ),
                chapter(
                    "Accessible date input",
                    "Keep typed text unchanged while editing. Validate and format on submission rather than inserting separators as each character is typed.",
                    Figure::Docked,
                ),
                chapter(
                    "Optional Clear button",
                    "Omit a Clear action when the task does not need one, keeping keyboard traversal concise.",
                    Figure::Input,
                ),
                chapter(
                    "Affordance for keyboard shortcuts",
                    "Communicate available navigation shortcuts in contextual hints and accessible descriptions.",
                    Figure::Docked,
                ),
                chapter(
                    "Truncated labels & tooltips",
                    "Keep full accessible names for shortened visible labels. Noninteractive weekday headings do not add keyboard stops.",
                    Figure::Day,
                ),
                chapter(
                    "Color contrast between dates",
                    "Keep date text legible against both the calendar surface and the selected or range indicator.",
                    Figure::Range,
                ),
                chapter(
                    "Keyboard navigation",
                    "Use the calendar with keyboard navigation; confirmation and dismissal remain explicit actions.",
                    Figure::None,
                ),
                chapter(
                    "Labeling elements",
                    "Give the field a task-specific label and a description of its date format. Name month and year navigation controls.",
                    Figure::Docked,
                ),
                chapter(
                    "Screen reader verbalizations",
                    "Announce the complete day, month, date and year so a date remains understandable outside its visual calendar context.",
                    Figure::Day,
                ),
            ],
        ),
    ]
}

#[tessera]
fn date_chapter(chapter: DateChapter) {
    text().content(chapter.text).style(type_style(16.0, 24.0));
    if chapter.figure != Figure::None {
        gap(Dp(24.0));
        chapter_figure(
            chapter.figure,
            chapter.title.to_owned(),
            chapter.title == "Anatomy",
        );
    }
    if chapter.title == "Keyboard navigation" {
        gap(Dp(24.0));
        spec_table(
            vec!["Keys".into(), "Actions".into()],
            vec![
                vec!["Arrow keys".into(), "Move the focused date".into()],
                vec!["Page Up / Page Down".into(), "Move by one month".into()],
                vec![
                    "Shift + Page Up / Page Down".into(),
                    "Move by one year".into(),
                ],
                vec![
                    "Home / End".into(),
                    "Move to the first / last date in the month".into(),
                ],
                vec!["Enter / Space".into(), "Choose the focused date".into()],
            ],
        );
    }
    for child in chapter.children {
        gap(Dp(32.0));
        subheading(child.title.into());
        gap(Dp(24.0));
        text().content(child.text).style(type_style(16.0, 24.0));
        if child.figure != Figure::None {
            gap(Dp(24.0));
            chapter_figure(
                child.figure,
                child.title.to_owned(),
                chapter.title == "Anatomy",
            );
        }
    }
}

#[tessera]
fn chapter_figure(figure: Figure, title: String, anatomy: bool) {
    let instructional = match title.as_str() {
        "Date entry methods" => Some(instructional_figures::Illustration::EntryMethods),
        "Accessible date input" => Some(instructional_figures::Illustration::Formatting),
        "Optional Clear button" => Some(instructional_figures::Illustration::Clear),
        "Labeling elements" => Some(instructional_figures::Illustration::Labels),
        _ => None,
    };
    if let Some(kind) = instructional {
        instructional_figures::illustration(kind);
    } else {
        let view = remember(|| match title.as_str() {
            "Month selection" => DockedDatePickerView::Month,
            "Year selection" => DockedDatePickerView::Year,
            _ => DockedDatePickerView::Calendar,
        });
        date_figure(figure, anatomy, false).view(view);
    }
}

#[tessera]
fn date_canvas(width: f64, height: f64, content: RenderSlot) {
    let available = remember(|| Dp(width));
    let scale = available.get().0 / width;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if next.0 > 0.0 && next != available.get() {
                    available.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale as f32).child(move || {
                boxed()
                    .modifier(Modifier::new().required_size(Dp(width), Dp(height)))
                    .children_shared(content.clone());
            });
        });
}

#[tessera]
fn date_overview() {
    let geometry = remember(ObservedGeometry::new);
    annotations::publish_observed_geometry(geometry);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let scale = 1.0;
    date_canvas(
        1024.0,
        376.0,
        RenderSlot::new(move || {
            boxed()
                .modifier(
                    watch_observed(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(376.0 * scale))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some("Date picker variants".into()),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    date_overview_variant(0, scale);
                    date_overview_variant(1, scale);
                    date_overview_variant(2, scale);
                });
        }),
    );
}

#[tessera]
fn date_overview_variant(index: usize, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let x = [0.0, 348.0, 696.0][index] * scale;
    let width = if index == 2 { 328.0 } else { 348.0 } * scale;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp::ZERO)
                .size(Dp(width), Dp(376.0 * scale))
                .clip_to_bounds(),
        )
        .children(move || {
            if index != 0 {
                boxed().modifier(
                    Modifier::new()
                        .size(Dp(1.0), Dp(376.0 * scale))
                        .background(scheme.outline),
                );
            }
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .offset(Dp(26.0 * scale), Dp(50.0 * scale))
                        .size(Dp(40.0 * scale), Dp(40.0 * scale))
                        .background_with_shape(
                            scheme.on_surface,
                            Shape::rounded_rectangle(Dp(20.0 * scale)),
                        )
                        .semantics(SemanticsArgs {
                            role: Some(Role::Note),
                            label: Some(format!(
                                "{} {}",
                                index + 1,
                                [
                                    "Docked date picker",
                                    "Modal date picker",
                                    "Modal date input"
                                ][index]
                            )),
                            ..Default::default()
                        }),
                )
                .children(move || {
                    text()
                        .content((index + 1).to_string())
                        .style(type_style(16.0 * scale, 24.0 * scale))
                        .color(scheme.surface);
                });
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(
                            Dp(if index == 0 { 86.0 } else { 90.0 } * scale),
                            Dp(if index == 0 { 49.0 } else { 40.0 } * scale),
                        )
                        .required_size(Dp(360.0), Dp(620.0)),
                )
                .children(move || match index {
                    0 => overview_docked_picker(),
                    1 => overview_modal_picker(),
                    _ => overview_modal_input(),
                });
        });
}

#[tessera]
fn overview_docked_picker() {
    let state = remember(|| illustration_state(false));
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        controller.set_text("08/17/2025");
        controller
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    TextFieldBuilder::outlined()
        .controller(controller)
        .label("Date")
        .accessibility_label("Overview docked date")
        .accessibility_description("MM/DD/YYYY")
        .modifier(Modifier::new().width(Dp(312.0)).height(Dp(56.0)))
        .on_change(move |value: String| {
            if let Some(date) = parse_form_date(&value) {
                state.with_mut(|state| state.set_selected_date(date));
            }
            value
        });
    text()
        .content("MM/DD/YYYY")
        .style(type_style(12.0, 16.0))
        .color(scheme.on_surface_variant)
        .modifier(Modifier::new().offset(Dp(16.0), Dp(60.0)));
    boxed()
        .modifier(Modifier::new().offset(Dp::ZERO, Dp(82.0)))
        .children(move || {
            docked_date_picker()
                .today(illustration_today())
                .state(state)
                .first_day_of_week(Weekday::Sunday)
                .on_date_change(move |date: CalendarDate| {
                    let value = format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year());
                    controller.with_mut(|controller| controller.set_text(&value));
                });
        });
}

#[tessera]
fn overview_modal_picker() {
    let state = remember(|| illustration_state(false));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .style(scheme.surface_container_high.into())
        .shape(Shape::rounded_rectangle(Dp(28.0)))
        .block_input(false)
        .modifier(Modifier::new().width(Dp(360.0)))
        .child(move || {
            date_picker_dialog()
                .picker_today(illustration_today())
                .state(state)
                .picker_title("Select date")
                .picker_first_day_of_week(Weekday::Sunday);
        });
}

#[tessera]
fn overview_modal_input() {
    let range = remember(|| {
        DateRangePickerState::new(
            None,
            None,
            YearMonth::new(2025, 8),
            1900..=2100,
            None,
            None,
            DatePickerDisplayMode::Input,
        )
    });
    let committed = remember(|| (None, None));
    let open = remember(|| true);
    let status = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    if open.get() {
        surface()
            .style(scheme.surface_container_high.into())
            .shape(Shape::rounded_rectangle(Dp(28.0)))
            .block_input(false)
            .modifier(Modifier::new().width(Dp(360.0)))
            .child(move || {
                column().children(move || {
                    date_range_picker()
                        .today(illustration_today())
                        .state(range)
                        .title("Select date");
                    figure_actions(status, open, range, committed);
                });
            });
    } else {
        button()
            .text()
            .accessibility_label("Reopen overview modal date input")
            .on_click(move || open.set(true))
            .child(|| {
                text().content("Open date input");
            });
    }
}

#[tessera]
fn docked_section() {
    date_figure(Figure::Docked, true, false);
    gap(Dp(24.0));
    anatomy_legend(Figure::Docked, DockedDatePickerView::Calendar);
    gap(Dp(32.0));
    let month_view = remember(|| DockedDatePickerView::Month);
    date_figure(Figure::Docked, true, false).view(month_view);
    gap(Dp(24.0));
    anatomy_legend(Figure::Docked, month_view.get());
    gap(Dp(32.0));
    subheading("Docked date picker color".into());
    gap(Dp(24.0));
    date_colors(Figure::Docked);
    gap(Dp(32.0));
    subheading("Docked date picker measurements".into());
    gap(Dp(24.0));
    date_figure(Figure::Docked, false, true);
    gap(Dp(24.0));
    let measurement_view = remember(|| DockedDatePickerView::Month);
    date_figure(Figure::Docked, false, true).view(measurement_view);
    gap(Dp(24.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Calendar width".into(), "360dp".into()],
            vec!["Calendar height".into(), "460dp with actions".into()],
            vec!["Date cell / indicator".into(), "48 / 40dp".into()],
            vec!["Content inset".into(), "12dp".into()],
            vec!["Field to calendar gap".into(), "8dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Docked date picker configurations".into());
    gap(Dp(24.0));
    docked_configurations();
}

#[tessera]
fn date_colors(figure: Figure) {
    date_color_pair(figure, false, false);
    if figure == Figure::Docked {
        gap(Dp(24.0));
        date_color_pair(figure, true, false);
    } else if figure == Figure::Day {
        gap(Dp(24.0));
        date_color_pair(figure, false, true);
        gap(Dp(24.0));
        date_color_pair(Figure::FullScreen, false, false);
    }
}

fn date_color_title(figure: Figure, menu: bool, year: bool) -> &'static str {
    match figure {
        Figure::Docked if menu => "Docked date picker menu color roles",
        Figure::Docked => "Docked date picker color roles",
        Figure::Input => "Modal date input color roles",
        Figure::FullScreen => "Modal date range picker color roles",
        _ if year => "Modal date picker year color roles",
        _ => "Modal date picker color roles",
    }
}

fn date_color_labels(figure: Figure, menu: bool, year: bool) -> Vec<&'static str> {
    match figure {
        Figure::Docked if menu => vec![
            "Primary",
            "On surface variant",
            "On surface",
            "Outline variant",
            "Surface container high",
            "Surface variant",
            "On surface",
        ],
        Figure::Docked => vec![
            "Primary",
            "On surface variant",
            "On surface variant",
            "On surface",
            "On surface",
            "Primary",
            "On surface variant",
            "Primary",
            "Surface container high",
            "Primary",
            "On primary",
        ],
        Figure::Input => vec![
            "On surface",
            "On surface variant",
            "Surface container high",
            "On surface variant",
            "Primary",
            "Primary",
            "Outline variant",
        ],
        Figure::FullScreen => vec![
            "On surface",
            "On surface variant",
            "On surface variant",
            "Surface container high",
            "Primary",
            "On surface variant",
            "On surface",
            "Primary",
            "On surface",
            "Secondary container",
            "On secondary container",
            "Outline variant",
            "On surface variant",
            "Primary",
        ],
        _ if year => vec![
            "On surface",
            "On surface variant",
            "Surface container high",
            "On surface variant",
            "On surface variant",
            "Primary",
            "Primary",
            "Outline variant",
            "On surface variant",
        ],
        _ => vec![
            "On surface",
            "On surface variant",
            "Surface container high",
            "On surface variant",
            "On surface variant",
            "On surface",
            "Primary",
            "On surface",
            "Primary",
            "Primary",
            "On surface variant",
            "Outline variant",
        ],
    }
}

#[tessera]
fn date_color_pair(figure: Figure, menu: bool, year: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let height = if figure == Figure::FullScreen {
        980.0
    } else if figure == Figure::Input {
        480.0
    } else if figure == Figure::Docked && menu {
        628.0
    } else if figure == Figure::Docked || year {
        640.0
    } else {
        665.0
    };
    date_canvas(
        1024.0,
        height,
        RenderSlot::new(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(date_color_title(figure, menu, year).into()),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            for dark in [false, true] {
                                column().modifier(Modifier::new().weight(1.0)).children(
                                    move || {
                                        material_theme()
                                            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                                            .child(move || {
                                                date_color_half(figure, menu, year);
                                            });
                                    },
                                );
                            }
                        });
                });
        }),
    );
    gap(Dp(24.0));
    text()
        .content(format!(
            "{} used for light and dark themes:",
            date_color_title(figure, menu, year)
        ))
        .style(type_style(16.0, 24.0));
    gap(Dp(16.0));
    for (index, label) in date_color_labels(figure, menu, year)
        .into_iter()
        .enumerate()
    {
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
                        text()
                            .content((index + 1).to_string())
                            .color(scheme.surface)
                            .style(type_style(14.0, 20.0));
                    });
                text()
                    .content(label)
                    .style(type_style(16.0, 24.0))
                    .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp::ZERO));
            });
        gap(Dp(8.0));
    }
}

#[tessera]
fn date_color_half(figure: Figure, menu: bool, year: bool) {
    let view = remember(|| {
        if menu {
            DockedDatePickerView::Month
        } else {
            DockedDatePickerView::Calendar
        }
    });
    let modal_view = remember(|| {
        if year {
            DatePickerView::Year
        } else {
            DatePickerView::Calendar
        }
    });
    date_figure(figure, false, false)
        .view(view)
        .modal_view(modal_view)
        .color_roles(true);
}

fn illustration_date(year: i32, day: u8) -> CalendarDate {
    CalendarDate::new(year, 8, day).expect("valid reference date")
}

fn illustration_today() -> CalendarDate {
    illustration_date(2025, 5)
}

fn illustration_state(input: bool) -> DatePickerState {
    DatePickerState::new(
        if input {
            None
        } else {
            Some(illustration_date(2025, 17))
        },
        YearMonth::new(2025, 8),
        1900..=2100,
        None,
        None,
        if input {
            DatePickerDisplayMode::Input
        } else {
            DatePickerDisplayMode::Picker
        },
    )
}

#[tessera]
fn date_figure(
    figure: Figure,
    anatomy: bool,
    measurements: bool,
    view: Option<State<DockedDatePickerView>>,
    modal_view: Option<State<DatePickerView>>,
    color_roles: Option<bool>,
) {
    let color_roles = color_roles.unwrap_or(false);
    let geometry = remember(ObservedGeometry::new);
    annotations::publish_observed_geometry(geometry);
    let internal_view = remember(DockedDatePickerView::default);
    let view = view.unwrap_or(internal_view);
    let internal_modal_view = remember(DatePickerView::default);
    let modal_view = modal_view.unwrap_or(internal_modal_view);
    let initial_year_view = modal_view.get() == DatePickerView::Year;
    #[cfg(test)]
    tests::GEOMETRY.set(Some(geometry));
    let state = remember(|| {
        if initial_year_view {
            DatePickerState::new(
                Some(illustration_date(2025, 17)),
                YearMonth::new(2025, 8),
                2021..=2100,
                None,
                None,
                DatePickerDisplayMode::Picker,
            )
        } else {
            illustration_state(figure == Figure::Input)
        }
    });
    let range = remember(|| {
        DateRangePickerState::new(
            Some(illustration_date(2025, 17)),
            Some(illustration_date(2025, 23)),
            YearMonth::new(2025, 8),
            1900..=2100,
            None,
            None,
            if figure == Figure::RangeInput {
                DatePickerDisplayMode::Input
            } else {
                DatePickerDisplayMode::Picker
            },
        )
    });
    let committed_range = remember(|| {
        (
            Some(illustration_date(2025, 17)),
            Some(illustration_date(2025, 23)),
        )
    });
    let open = remember(|| true);
    let status = remember(|| String::new());
    let draft = remember(|| "08/17/2025".to_owned());
    let committed = remember(|| illustration_date(2025, 17));
    let committed_draft = remember(|| "08/17/2025".to_owned());
    let error = remember(String::new);
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text("08/17/2025");
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let height = if color_roles && figure == Figure::FullScreen {
        Dp(980.0)
    } else if figure == Figure::Input {
        Dp(480.0)
    } else if figure == Figure::Docked {
        Dp(if view.get() != DockedDatePickerView::Calendar {
            if measurements { 640.0 } else { 628.0 }
        } else if measurements {
            676.0
        } else {
            640.0
        })
    } else if modal_view.get() == DatePickerView::Year {
        Dp(640.0)
    } else if figure == Figure::Day {
        Dp(665.0)
    } else {
        Dp(680.0)
    };
    let canonical_width = if color_roles { 512.0 } else { 1024.0 };
    date_canvas(
        canonical_width,
        height.0,
        RenderSlot::new(move || {
            boxed().alignment(Alignment::Center).modifier(watch_observed(geometry,"figure").fill_max_width().height(height).background_with_shape(scheme.surface_container_lowest,Shape::rounded_rectangle(if color_roles {Dp::ZERO} else {Dp(24.0)}))).children(move || {
        boxed().alignment(Alignment::Center).modifier(Modifier::new().fill_max_size()).children(move || {
            boxed().modifier(if figure == Figure::Docked {watch_observed(geometry,"container").size(Dp(360.0), Dp(544.0))} else {watch_observed(geometry,"container").width(Dp(360.0))}).children(move || {
                if figure==Figure::Docked {
                    column().modifier(Modifier::new().width(Dp(312.0))).children(move || {
                        TextFieldBuilder::outlined().controller(controller).label("Date").accessibility_label("Event date").accessibility_description(if error.with(String::is_empty) {"MM/DD/YYYY".to_owned()} else {error.get()}).is_error(!error.with(String::is_empty))
                            .modifier(watch_observed(geometry,"field").width(Dp(312.0)).height(Dp(56.0)).semantics(SemanticsArgs {test_tag:Some("docked-date-field".into()), merge_descendants:true,..Default::default()})).min_width(Dp::ZERO)
                            .on_change(move |raw: String| {draft.set(raw.clone());raw})
                            .on_submit(move || {submit_docked_date(state,controller,draft,committed,committed_draft,error);})
                            .trailing_icon(move || {icon_button().standard().icon(filled::CALENDAR_TODAY_SVG).accessibility_label("Toggle event date calendar").on_click(move || {
                                if open.get() {restore_docked_date(state,controller,draft,committed,committed_draft,error);}
                                open.set(!open.get());FocusManager::current().defer_keyboard_until_layout();
                            });});
                        gap(Dp(4.0));text().content(if error.with(String::is_empty) {"MM/DD/YYYY".to_owned()} else {error.get()}).style(type_style(12.0,16.0)).color(if error.with(String::is_empty) {scheme.on_surface_variant} else {scheme.error});
                    });
                    if open.get() {
                        boxed().modifier(Modifier::new().offset(Dp::ZERO,Dp(84.0))).children(move || {
                            docked_date_picker().today(illustration_today()).state(state).view(view).first_day_of_week(Weekday::Sunday).modifier(watch_observed(geometry,"calendar"))
                                .on_date_change(move |date:CalendarDate|{let formatted=format!("{:02}/{:02}/{:04}",date.month(),date.day(),date.year()); draft.set(formatted.clone());controller.with_mut(|value|value.set_text(&formatted));error.set(String::new());})
                                .on_cancel(move || {restore_docked_date(state,controller,draft,committed,committed_draft,error);open.set(false);status.set("Date selection cancelled".into());FocusManager::current().defer_keyboard_until_layout();})
                                .on_confirm(move || {if submit_docked_date(state,controller,draft,committed,committed_draft,error) {open.set(false);status.set("Event date confirmed".into());FocusManager::current().defer_keyboard_until_layout();}});
                        });
                    }
                } else if open.get() {
                    surface().modifier(watch_observed(geometry,"calendar").width(Dp(360.0))).style(scheme.surface_container_high.into()).shape(Shape::rounded_rectangle(if figure==Figure::FullScreen {Dp::ZERO}else{Dp(28.0)})).block_input(false).child(move || {
                        if matches!(figure,Figure::Range|Figure::RangeInput|Figure::FullScreen) {
                            column().modifier(Modifier::new().width(Dp(360.0))).children(move || {
                                if figure==Figure::FullScreen {
                                    row().modifier(Modifier::new().fill_max_width().padding_all(Dp(8.0))).main_axis_alignment(MainAxisAlignment::SpaceBetween).children(move || {
                                        icon_button().standard().icon(filled::CLOSE_SVG).accessibility_label("Close full-screen date selection").on_click(move ||{ let (start,end)=committed_range.get(); range.with_mut(|range|range.set_selection(start,end)); open.set(false); status.set("Full-screen date selection closed".into()); FocusManager::current().defer_keyboard_until_layout(); });
                                        button().text().accessibility_label("Save").enabled(range.with(DateRangePickerState::is_complete)).on_click(move ||{ if !range.with(DateRangePickerState::is_complete) {return;} committed_range.set(range.with(|range|(range.selected_start_date(),range.selected_end_date()))); open.set(false); status.set("Date range saved".into()); FocusManager::current().defer_keyboard_until_layout(); }).child(||{text().content("Save");});
                                    });
                                }
                                date_range_picker().today(illustration_today()).state(range).first_day_of_week(Weekday::Sunday).calendar_height(Dp(if color_roles {624.0} else {336.0}));
                                if figure!=Figure::FullScreen {figure_actions(status, open, range, committed_range);}
                            });
                        } else {
                            date_picker_dialog().picker_today(illustration_today()).state(state).picker_view(modal_view).picker_title("Select date").picker_headline_optional(if state.with(DatePickerState::display_mode)==DatePickerDisplayMode::Input {Some("Enter dates".into())} else {None}).picker_first_day_of_week(Weekday::Sunday).dismiss_button(move || {button().text().accessibility_label("Reference date cancel").on_click(move ||{ open.set(false); status.set("Date selection cancelled".into()); FocusManager::current().defer_keyboard_until_layout(); }).child(||{text().content("Cancel");});})
                                .confirm_button(move || {button().text().accessibility_label("Reference date confirm").on_click(move ||{ open.set(false); status.set("Date selection confirmed".into()); FocusManager::current().defer_keyboard_until_layout(); }).child(||{text().content("OK");});});
                        }
                    });
                }
            });
        });
        boxed().modifier(Modifier::new().fill_max_size()).children(move || {
            if open.get() && (anatomy || measurements) {date_annotations(figure,measurements,geometry,state,range,view.get());}
            if open.get() && color_roles {date_color_annotations(figure, geometry, state, range, view.get(), modal_view.get());}
        });
    });
        }),
    );
    if !open.get() {
        gap(Dp(12.0));
        button()
            .outlined()
            .accessibility_label("Reopen reference date picker")
            .on_click(move || open.set(true))
            .child(|| {
                text().content("Open date picker");
            });
    }
    if !status.with(String::is_empty) {
        gap(Dp(12.0));
        text().content(status.get()).style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn date_color_annotations(
    figure: Figure,
    geometry: State<ObservedGeometry>,
    single: State<DatePickerState>,
    range: State<DateRangePickerState>,
    view: DockedDatePickerView,
    modal_view: DatePickerView,
) {
    #[cfg(all(test, not(target_family = "wasm")))]
    tests::ANNOTATION_BUILDS.set(tests::ANNOTATION_BUILDS.get() + 1);
    let values = geometry.with(annotations::observed_relative_geometry);
    date_color_annotation_content(figure, values, single, range, view, modal_view);
}

#[tessera]
fn date_color_annotation_content(
    figure: Figure,
    values: BTreeMap<&'static str, Rect>,
    single: State<DatePickerState>,
    range: State<DateRangePickerState>,
    view: DockedDatePickerView,
    modal_view: DatePickerView,
) {
    let (Some(root), Some(calendar)) = (values.get("figure"), values.get("calendar").copied())
    else {
        return;
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = Dp(root.w);
    let height = Dp(root.h);
    let menu = figure == Figure::Docked && view != DockedDatePickerView::Calendar;
    let year = modal_view == DatePickerView::Year;
    let input = single.with(DatePickerState::display_mode) == DatePickerDisplayMode::Input;
    let effective_figure = if matches!(figure, Figure::Day | Figure::Input) {
        if input { Figure::Input } else { Figure::Day }
    } else {
        figure
    };
    let labels = date_color_labels(effective_figure, menu, year);
    let rect = |x, y, w, h| Rect {
        x: calendar.x + x,
        y: calendar.y + y,
        w,
        h,
    };
    let month = single.with(DatePickerState::displayed_month);
    let mut regions: Vec<(usize, Rect, i8, f64)> = Vec::new();
    if figure == Figure::Docked {
        if let Some(field) = values.get("field").copied() {
            regions.push((
                1,
                Rect {
                    w: 2.0,
                    h: 2.0,
                    y: field.y + 28.0,
                    ..field
                },
                -1,
                0.0,
            ));
        }
        if menu {
            let header = docked_header_regions(calendar);
            regions.extend([
                (2, header[0], 0, -80.0),
                (3, header[1], 0, -80.0),
                (4, rect(312.0, 64.0, 2.0, 2.0), 1, 0.0),
                (5, rect(300.0, 112.0, 2.0, 2.0), 1, 0.0),
            ]);
            let selected = docked_menu_selected_row(single, view) as f64;
            regions.push((6, rect(0.0, 64.0 + selected * 48.0, 344.0, 48.0), -1, 0.0));
            regions.push((
                7,
                rect(
                    56.0,
                    64.0 + (selected - 1.0).max(0.0) * 48.0 + 12.0,
                    52.0,
                    24.0,
                ),
                -1,
                0.0,
            ));
        } else {
            let header = docked_calendar_header_regions(calendar);
            regions.extend([
                (2, header[0], 0, -80.0),
                (3, rect(312.0, 12.0, 36.0, 44.0), 1, 0.0),
                (4, rect(12.0, 64.0, 336.0, 40.0), 1, 0.0),
                (8, rect(12.0, calendar.h - 48.0, 336.0, 40.0), 1, 0.0),
                (9, rect(12.0, calendar.h - 80.0, 2.0, 2.0), -1, 0.0),
                (7, rect(304.0, 392.0, 40.0, 40.0), 1, 0.0),
            ]);
            for (number, date) in [(5, month.to_date(1)), (6, Some(illustration_today()))] {
                if let Some(cell) = date.and_then(|date| docked_day_bounds(calendar, month, date)) {
                    regions.push((
                        number,
                        Rect {
                            x: cell.x + 4.0,
                            y: cell.y + 4.0,
                            w: 40.0,
                            h: 40.0,
                        },
                        1,
                        0.0,
                    ));
                }
            }
            if let Some(cell) = single
                .with(DatePickerState::selected_date)
                .and_then(|date| docked_day_bounds(calendar, month, date))
            {
                regions.push((
                    10,
                    Rect {
                        x: cell.x + 4.0,
                        y: cell.y + 40.0,
                        w: 40.0,
                        h: 2.0,
                    },
                    -1,
                    0.0,
                ));
                regions.push((
                    11,
                    Rect {
                        x: cell.x + 16.0,
                        y: cell.y + 16.0,
                        w: 16.0,
                        h: 16.0,
                    },
                    -1,
                    0.0,
                ));
            }
        }
    } else if figure == Figure::FullScreen {
        let header_y = 64.0;
        regions.extend([
            (1, rect(12.0, 12.0, 24.0, 24.0), -1, 0.0),
            (2, rect(64.0, header_y + 4.0, 150.0, 20.0), -1, 0.0),
            (3, rect(64.0, header_y + 28.0, 220.0, 28.0), -1, 0.0),
            (4, rect(8.0, calendar.h - 32.0, 2.0, 2.0), -1, 0.0),
            (5, rect(292.0, 12.0, 56.0, 40.0), 1, 0.0),
            (6, rect(312.0, header_y + 24.0, 24.0, 24.0), 1, 0.0),
            (7, rect(12.0, header_y + 69.0, 336.0, 48.0), 1, 0.0),
            (12, rect(0.0, header_y + 68.0, 2.0, 2.0), -1, 0.0),
            (13, rect(36.0, header_y + 137.0, 160.0, 20.0), -1, 0.0),
        ]);
        let month = range.with(DateRangePickerState::displayed_month);
        let range_cell = |date| {
            docked_day_bounds(calendar, month, date).map(|cell| Rect {
                y: cell.y + header_y + 61.0,
                ..cell
            })
        };
        if let Some(cell) = range_cell(illustration_today()) {
            regions.push((8, cell, 1, 0.0));
        }
        if let Some(cell) = month.to_date(25).and_then(range_cell) {
            regions.push((9, cell, 1, 0.0));
        }
        if let Some(cell) = range
            .with(DateRangePickerState::selected_start_date)
            .and_then(range_cell)
        {
            regions.push((
                14,
                Rect {
                    x: cell.x + 4.0,
                    y: cell.y + 4.0,
                    w: 40.0,
                    h: 40.0,
                },
                -1,
                0.0,
            ));
            regions.push((
                10,
                Rect {
                    x: cell.x + 48.0,
                    w: 48.0,
                    ..cell
                },
                -1,
                48.0,
            ));
            regions.push((
                11,
                Rect {
                    x: cell.x + 64.0,
                    y: cell.y + 16.0,
                    w: 16.0,
                    h: 16.0,
                },
                1,
                0.0,
            ));
        }
    } else {
        regions.extend([
            (1, rect(24.0, 60.0, 210.0, 40.0), -1, 0.0),
            (2, rect(24.0, 16.0, 80.0, 20.0), -1, 0.0),
            (
                3,
                rect(180.0, 12.0, 2.0, 2.0),
                0,
                if year { -46.0 } else { -36.0 },
            ),
            (4, rect(312.0, 72.0, 24.0, 24.0), 1, 0.0),
        ]);
        if input {
            regions.extend([
                (5, rect(336.0, 168.0, 2.0, 2.0), 1, 0.0),
                (6, rect(204.0, calendar.h - 52.0, 144.0, 40.0), 1, 0.0),
                (7, rect(8.0, 120.0, 2.0, 2.0), -1, 0.0),
            ]);
        } else if year {
            regions.extend([
                (5, rect(256.0, 192.0, 72.0, 36.0), 1, 0.0),
                (6, rect(144.0, 244.0, 72.0, 36.0), 1, 0.0),
                (7, rect(204.0, calendar.h - 52.0, 144.0, 40.0), 1, 0.0),
                (8, rect(8.0, calendar.h - 60.0, 2.0, 2.0), -1, 0.0),
                (9, rect(12.0, 128.0, 144.0, 40.0), -1, 0.0),
            ]);
        } else {
            regions.extend([
                (5, rect(264.0, 128.0, 72.0, 40.0), 1, 0.0),
                (6, rect(12.0, 176.0, 336.0, 48.0), 1, 0.0),
                (9, rect(204.0, calendar.h - 52.0, 144.0, 40.0), 1, 0.0),
                (11, rect(12.0, 128.0, 144.0, 40.0), -1, 0.0),
                (12, rect(8.0, 120.0, 2.0, 2.0), -1, 0.0),
            ]);
            for (number, date, side) in [
                (7, Some(illustration_today()), 1),
                (8, month.to_date(25), 1),
                (10, single.with(DatePickerState::selected_date), -1),
            ] {
                if let Some(cell) = date.and_then(|date| docked_day_bounds(calendar, month, date)) {
                    regions.push((number, modal_day_visual(cell), side, 0.0));
                }
            }
        }
    }
    let mut svg = String::new();
    let ink = if scheme.is_dark { "#cac4d0" } else { "#49454f" };
    for (number, region, side, shift) in regions {
        if region.w > 4.0 && region.h > 4.0 {
            dotted(&mut svg, region.x, region.y, region.w, false, ink);
            dotted(
                &mut svg,
                region.x,
                region.y + region.h,
                region.w,
                false,
                ink,
            );
            dotted(&mut svg, region.x, region.y, region.h, true, ink);
            dotted(&mut svg, region.x + region.w, region.y, region.h, true, ink);
        }
        let center_y = region.y + region.h * 0.5;
        let (x, y) = if side == 0 {
            let x = region.x + region.w * 0.5;
            let y = calendar.y + shift;
            dotted(&mut svg, x, y + 32.0, region.y - y - 32.0, true, ink);
            (x - 12.0, y)
        } else {
            let x = if side < 0 { 16.0 } else { width.0 - 40.0 };
            let y = center_y + shift;
            let from = if side < 0 {
                x + 24.0
            } else {
                region.x + region.w
            };
            let to = if side < 0 { region.x } else { x };
            dotted(&mut svg, from, y, to - from, false, ink);
            if shift != 0.0 {
                dotted(&mut svg, region.x, center_y, shift, true, ink);
            }
            (x, y - 16.0)
        };
        date_color_marker(number, labels[number - 1].into(), Dp(x), Dp(y));
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn date_color_marker(number: usize, label: String, x: Dp, y: Dp) {
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
                    label: Some(format!("Color role {number}: {label}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.surface);
        });
}

#[tessera]
fn figure_actions(
    status: State<String>,
    open: State<bool>,
    range: State<DateRangePickerState>,
    committed: State<(Option<CalendarDate>, Option<CalendarDate>)>,
) {
    row()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
        .main_axis_alignment(MainAxisAlignment::End)
        .children(move || {
            button()
                .text()
                .accessibility_label("Reference range cancel")
                .on_click(move || {
                    let (start, end) = committed.get();
                    range.with_mut(|range| range.set_selection(start, end));
                    open.set(false);
                    status.set("Date range cancelled".into());
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(|| {
                    text().content("Cancel");
                });
            button()
                .text()
                .accessibility_label("Reference range confirm")
                .enabled(range.with(DateRangePickerState::is_complete))
                .on_click(move || {
                    if !range.with(DateRangePickerState::is_complete) {
                        return;
                    }
                    committed
                        .set(range.with(|range| {
                            (range.selected_start_date(), range.selected_end_date())
                        }));
                    open.set(false);
                    status.set("Date range confirmed".into());
                    FocusManager::current().defer_keyboard_until_layout();
                })
                .child(|| {
                    text().content("OK");
                });
        });
}

fn parse_form_date(value: &str) -> Option<CalendarDate> {
    let parts: Vec<&str> = value
        .split(['/', '-', '.', ' '])
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() == 3 {
        if parts[0].len() == 4 {
            CalendarDate::new(
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            )
        } else {
            CalendarDate::new(
                parts[2].parse().ok()?,
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
            )
        }
    } else if value.len() == 8
        && value.is_ascii()
        && value.bytes().all(|byte| byte.is_ascii_digit())
    {
        CalendarDate::new(
            value[4..].parse().ok()?,
            value[..2].parse().ok()?,
            value[2..4].parse().ok()?,
        )
    } else {
        None
    }
}

fn submit_docked_date(
    state: State<DatePickerState>,
    controller: State<TextInputController>,
    draft: State<String>,
    committed: State<CalendarDate>,
    committed_draft: State<String>,
    error: State<String>,
) -> bool {
    let Some(date) = parse_form_date(&draft.get()) else {
        error.set("Enter a valid date (MM/DD/YYYY)".into());
        return false;
    };
    if !state.with_mut(|state| state.set_selected_date(date)) {
        error.set(state.with(|state| {
            format!(
                "Date must be from {} to {}",
                state.year_range().start(),
                state.year_range().end()
            )
        }));
        return false;
    }
    let formatted = format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year());
    draft.set(formatted.clone());
    committed.set(date);
    committed_draft.set(formatted.clone());
    controller.with_mut(|controller| controller.set_text(&formatted));
    error.set(String::new());
    true
}

fn restore_docked_date(
    state: State<DatePickerState>,
    controller: State<TextInputController>,
    draft: State<String>,
    committed: State<CalendarDate>,
    committed_draft: State<String>,
    error: State<String>,
) {
    state.with_mut(|state| state.set_selected_date(committed.get()));
    let formatted = committed_draft.get();
    controller.with_mut(|controller| controller.set_text(&formatted));
    draft.set(formatted);
    error.set(String::new());
}

#[tessera]
fn date_annotations(
    figure: Figure,
    measurements: bool,
    geometry: State<ObservedGeometry>,
    single: State<DatePickerState>,
    range: State<DateRangePickerState>,
    view: DockedDatePickerView,
) {
    #[cfg(all(test, not(target_family = "wasm")))]
    tests::ANNOTATION_BUILDS.set(tests::ANNOTATION_BUILDS.get() + 1);
    let values = geometry.with(annotations::observed_relative_geometry);
    date_annotation_content(figure, measurements, values, single, range, view);
}

#[tessera]
fn date_annotation_content(
    figure: Figure,
    measurements: bool,
    values: BTreeMap<&'static str, Rect>,
    single: State<DatePickerState>,
    range: State<DateRangePickerState>,
    view: DockedDatePickerView,
) {
    let (Some(root), Some(calendar)) = (values.get("figure"), values.get("calendar").copied())
    else {
        return;
    };
    let width = Dp(root.w);
    let height = Dp(root.h);
    let mut svg = String::new();
    let mut callouts = Vec::new();
    let range_figure = matches!(
        figure,
        Figure::Range | Figure::RangeInput | Figure::FullScreen
    );
    let input = if range_figure {
        range.with(DateRangePickerState::display_mode) == DatePickerDisplayMode::Input
    } else {
        single.with(DatePickerState::display_mode) == DatePickerDisplayMode::Input
    };
    if range_figure {
        let top = if figure == Figure::FullScreen {
            64.0
        } else {
            0.0
        };
        let header = Rect {
            x: calendar.x,
            y: calendar.y + top,
            w: calendar.w,
            h: if input { 120.0 } else { 68.0 },
        };
        if measurements {
            let tint = "#ec407a";
            let leading = if input { 24.0 } else { 64.0 };
            let header_height = header.h;
            svg.push_str(&format!("<rect x='{}' y='{}' width='{leading}' height='{header_height}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='12' height='{header_height}' fill='{tint}' fill-opacity='.25'/>", header.x, header.y, header.x+header.w-12.0, header.y));
            annotations::dimension(
                format!("{}", leading as u32),
                Dp(header.x + 16.0),
                Dp(header.y + 24.0),
            );
            annotations::dimension(
                "12".into(),
                Dp(header.x + header.w + 12.0),
                Dp(header.y + 24.0),
            );
            dotted(
                &mut svg,
                calendar.x,
                calendar.y + calendar.h + 24.0,
                calendar.w,
                false,
                tint,
            );
            annotations::dimension(
                "360".into(),
                Dp(calendar.x + calendar.w * 0.5 - 12.0),
                Dp(calendar.y + calendar.h + 32.0),
            );
            let content_y = header.y + header.h + 1.0;
            let inset = if input { 24.0 } else { 12.0 };
            let content_height = if input { 104.0 } else { 384.0 };
            svg.push_str(&format!("<rect x='{}' y='{content_y}' width='{inset}' height='{content_height}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{content_y}' width='{inset}' height='{content_height}' fill='{tint}' fill-opacity='.25'/>",calendar.x,calendar.x+calendar.w-inset));
            annotations::dimension(
                format!("{}", inset as u32),
                Dp(calendar.x + inset + 4.0),
                Dp(content_y + 8.0),
            );
            if input {
                annotations::dimension("16".into(), Dp(calendar.x + 172.0), Dp(content_y + 44.0));
                annotations::dimension(
                    "56".into(),
                    Dp(calendar.x + calendar.w + 12.0),
                    Dp(content_y + 44.0),
                );
            } else {
                let cell = Rect {
                    x: calendar.x + 12.0,
                    y: content_y + 96.0,
                    w: 48.0,
                    h: 48.0,
                };
                annotations::outline(&mut svg, cell);
                annotations::dimension("48".into(), Dp(cell.x + 8.0), Dp(cell.y + cell.h + 8.0));
            }
        } else {
            callouts.extend(range_annotation_regions(figure, calendar, input));
        }
    } else if measurements && figure == Figure::Docked && view != DockedDatePickerView::Calendar {
        let tint = "#ec407a";
        let selected_row = docked_menu_selected_row(single, view) as f64;
        svg.push_str(&format!("<rect x='{}' y='{}' width='{}' height='20' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='{}' height='20' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='16' height='{}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='16' height='{}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='16' height='336' fill='{tint}' fill-opacity='.25'/>",calendar.x,calendar.y,calendar.w,calendar.x,calendar.y+44.0,calendar.w,calendar.x,calendar.y,calendar.h,calendar.x+calendar.w-16.0,calendar.y,calendar.h,calendar.x+40.0,calendar.y+64.0));
        annotations::dimension("20".into(), Dp(calendar.x - 40.0), Dp(calendar.y));
        annotations::dimension("24".into(), Dp(calendar.x - 40.0), Dp(calendar.y + 20.0));
        annotations::dimension("20".into(), Dp(calendar.x - 40.0), Dp(calendar.y + 44.0));
        annotations::dimension(
            "48".into(),
            Dp(calendar.x - 48.0),
            Dp(calendar.y + 64.0 + selected_row * 48.0 + 12.0),
        );
        annotations::dimension(
            "16".into(),
            Dp(calendar.x),
            Dp(calendar.y + calendar.h + 24.0),
        );
        annotations::dimension(
            "16".into(),
            Dp(calendar.x + 40.0),
            Dp(calendar.y + calendar.h + 24.0),
        );
        annotations::dimension(
            "16".into(),
            Dp(calendar.x + calendar.w - 16.0),
            Dp(calendar.y + calendar.h + 24.0),
        );
        annotations::dimension(
            "28".into(),
            Dp(calendar.x - 40.0),
            Dp(calendar.y + calendar.h - 24.0),
        );
        dotted(
            &mut svg,
            calendar.x - 12.0,
            calendar.y + 64.0 + selected_row * 48.0,
            48.0,
            true,
            tint,
        );
        let check = Rect {
            x: calendar.x + 16.0,
            y: calendar.y + 64.0 + selected_row * 48.0 + 12.0,
            w: 24.0,
            h: 24.0,
        };
        annotations::outline(&mut svg, check);
    } else if measurements {
        let tint = "#ec407a";
        let inset = if figure == Figure::Docked { 12.0 } else { 24.0 };
        svg.push_str(&format!("<rect x='{}' y='{}' width='{}' height='{inset}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='{inset}' height='{}' fill='{tint}' fill-opacity='.25'/><rect x='{}' y='{}' width='{inset}' height='{}' fill='{tint}' fill-opacity='.25'/>",calendar.x,calendar.y,calendar.w,calendar.x,calendar.y,calendar.h,calendar.x+calendar.w-inset,calendar.y,calendar.h));
        annotations::dimension(
            format!("{}", inset as i32),
            Dp(calendar.x + inset + 8.0),
            Dp(calendar.y - 24.0),
        );
        dotted(
            &mut svg,
            calendar.x,
            calendar.y + calendar.h + 24.0,
            calendar.w,
            false,
            tint,
        );
        annotations::dimension(
            "360".into(),
            Dp(calendar.x + calendar.w * 0.5 - 12.0),
            Dp(calendar.y + calendar.h + 32.0),
        );
        if figure == Figure::Docked {
            annotations::dimension(
                "460".into(),
                Dp(calendar.x + calendar.w + 24.0),
                Dp(calendar.y + calendar.h * 0.5),
            );
        }
        if !input {
            let cell = Rect {
                x: calendar.x + 12.0,
                y: calendar.y
                    + if figure == Figure::Docked {
                        104.0
                    } else {
                        224.0
                    },
                w: 48.0,
                h: 48.0,
            };
            annotations::outline(&mut svg, cell);
            dotted(&mut svg, cell.x, cell.y + cell.h + 8.0, cell.w, false, tint);
            annotations::dimension("48".into(), Dp(cell.x + 8.0), Dp(cell.y + cell.h + 14.0));
        }
    } else if figure == Figure::Docked && view != DockedDatePickerView::Calendar {
        if let Some(field) = values.get("field").copied() {
            callouts.push((1, field, false));
        }
        let header = docked_header_regions(calendar);
        callouts.push((2, header[0], false));
        callouts.push((3, header[1], true));
        callouts.push((
            4,
            Rect {
                x: calendar.x,
                y: calendar.y,
                w: calendar.w,
                h: 64.0,
            },
            true,
        ));
        callouts.push((
            5,
            Rect {
                x: calendar.x,
                y: calendar.y + 64.0,
                w: calendar.w,
                h: 336.0,
            },
            false,
        ));
        let selected_row = docked_menu_selected_row(single, view);
        callouts.push((
            6,
            Rect {
                x: calendar.x,
                y: calendar.y + 64.0 + selected_row as f64 * 48.0,
                w: calendar.w,
                h: 48.0,
            },
            true,
        ));
        let other = if selected_row == 0 {
            1
        } else {
            selected_row - 1
        };
        callouts.push((
            7,
            Rect {
                x: calendar.x,
                y: calendar.y + 64.0 + other as f64 * 48.0,
                w: calendar.w,
                h: 48.0,
            },
            true,
        ));
        annotations::marker(8, Dp(calendar.x - 76.0), Dp(calendar.y + calendar.h - 40.0));
        dotted(
            &mut svg,
            calendar.x - 52.0,
            calendar.y + calendar.h - 24.0,
            52.0,
            false,
            "#49454f",
        );
    } else if figure == Figure::Docked {
        if let Some(field) = values.get("field").copied() {
            callouts.push((1, field, false));
        }
        let header = docked_calendar_header_regions(calendar);
        callouts.push((2, header[0], false));
        callouts.push((3, header[1], true));
        callouts.push((
            4,
            Rect {
                x: calendar.x + 304.0,
                y: calendar.y + 12.0,
                w: 44.0,
                h: 44.0,
            },
            true,
        ));
        callouts.push((
            5,
            Rect {
                x: calendar.x + 12.0,
                y: calendar.y + 56.0,
                w: 336.0,
                h: 48.0,
            },
            true,
        ));
        if let Some(rect) = single
            .with(DatePickerState::displayed_month)
            .to_date(1)
            .and_then(|date| {
                docked_day_bounds(
                    calendar,
                    single.with(DatePickerState::displayed_month),
                    date,
                )
            })
        {
            callouts.push((6, rect, true));
        }
        if let Some(rect) = Some(illustration_today()).and_then(|date| {
            docked_day_bounds(
                calendar,
                single.with(DatePickerState::displayed_month),
                date,
            )
        }) {
            callouts.push((7, rect, true));
        }
        callouts.push((
            8,
            Rect {
                x: calendar.x + 300.0,
                y: calendar.y + 344.0,
                w: 48.0,
                h: 48.0,
            },
            true,
        ));
        callouts.push((
            9,
            Rect {
                x: calendar.x + 180.0,
                y: calendar.y + 400.0,
                w: 168.0,
                h: 48.0,
            },
            true,
        ));
        if let Some(rect) = single
            .with(DatePickerState::selected_date)
            .and_then(|date| {
                docked_day_bounds(
                    calendar,
                    single.with(DatePickerState::displayed_month),
                    date,
                )
            })
        {
            callouts.push((10, rect, false));
        }
        let mut container_y = calendar.y + calendar.h * 0.5;
        if single
            .with(DatePickerState::selected_date)
            .and_then(|date| {
                docked_day_bounds(
                    calendar,
                    single.with(DatePickerState::displayed_month),
                    date,
                )
            })
            .is_some_and(|rect| (rect.y + rect.h * 0.5 - container_y).abs() < 32.0)
        {
            container_y -= 64.0;
        }
        annotations::marker(11, Dp(calendar.x - 76.0), Dp(container_y - 16.0));
        dotted(
            &mut svg,
            calendar.x - 52.0,
            container_y,
            52.0,
            false,
            "#49454f",
        );
    } else {
        callouts.push((
            1,
            Rect {
                x: calendar.x + 24.0,
                y: calendar.y + 60.0,
                w: 240.0,
                h: 48.0,
            },
            false,
        ));
        callouts.push((
            2,
            Rect {
                x: calendar.x + 24.0,
                y: calendar.y + 16.0,
                w: 100.0,
                h: 20.0,
            },
            false,
        ));
        annotations::marker(
            3,
            Dp(calendar.x + calendar.w * 0.5 - 12.0),
            Dp(calendar.y - 60.0),
        );
        dotted(
            &mut svg,
            calendar.x + calendar.w * 0.5,
            calendar.y - 28.0,
            28.0,
            true,
            "#49454f",
        );
        callouts.push((
            4,
            Rect {
                x: calendar.x,
                y: calendar.y,
                w: calendar.w,
                h: 120.0,
            },
            true,
        ));
        callouts.push((
            5,
            Rect {
                x: calendar.x + 300.0,
                y: calendar.y + 60.0,
                w: 24.0,
                h: 24.0,
            },
            true,
        ));
        if input {
            callouts.push((
                6,
                Rect {
                    x: calendar.x + 24.0,
                    y: calendar.y + 140.0,
                    w: 312.0,
                    h: 56.0,
                },
                true,
            ));
            callouts.push((
                7,
                Rect {
                    x: calendar.x + 204.0,
                    y: calendar.y + calendar.h - 56.0,
                    w: 144.0,
                    h: 40.0,
                },
                true,
            ));
            callouts.push((
                8,
                Rect {
                    x: calendar.x,
                    y: calendar.y + 119.0,
                    w: calendar.w,
                    h: 1.0,
                },
                false,
            ));
        } else {
            callouts.push((
                6,
                Rect {
                    x: calendar.x + 252.0,
                    y: calendar.y + 124.0,
                    w: 96.0,
                    h: 48.0,
                },
                true,
            ));
            callouts.push((
                7,
                Rect {
                    x: calendar.x + 12.0,
                    y: calendar.y + 176.0,
                    w: 336.0,
                    h: 48.0,
                },
                true,
            ));
            let month = single.with(DatePickerState::displayed_month);
            let today = illustration_today();
            if let Some(cell) = docked_day_bounds(calendar, month, today) {
                callouts.push((8, modal_day_visual(cell), true));
            }
            let unselected = month.to_date(
                if single
                    .with(DatePickerState::selected_date)
                    .is_some_and(|date| date.day() == 25)
                {
                    24
                } else {
                    25
                },
            );
            if let Some(cell) = unselected.and_then(|date| docked_day_bounds(calendar, month, date))
            {
                callouts.push((9, modal_day_visual(cell), true));
            }
            callouts.push((
                10,
                Rect {
                    x: calendar.x + 204.0,
                    y: calendar.y + calendar.h - 56.0,
                    w: 144.0,
                    h: 40.0,
                },
                true,
            ));
            if let Some(cell) = single
                .with(DatePickerState::selected_date)
                .and_then(|date| docked_day_bounds(calendar, month, date))
            {
                callouts.push((11, modal_day_visual(cell), false));
            }
            callouts.push((
                12,
                Rect {
                    x: calendar.x + 12.0,
                    y: calendar.y + 124.0,
                    w: 180.0,
                    h: 48.0,
                },
                false,
            ));
            callouts.push((
                13,
                Rect {
                    x: calendar.x,
                    y: calendar.y + 119.0,
                    w: calendar.w,
                    h: 1.0,
                },
                false,
            ));
        }
    }
    for (number, rect, trailing) in callouts {
        annotations::outline(&mut svg, rect);
        let field = if figure == Figure::Docked && matches!(number, 2 | 3) {
            values.get("field").copied()
        } else {
            None
        };
        let (marker_x, marker_y) = if let Some(field) = field {
            let x = rect.x + rect.w * 0.5;
            let y = field.y - 60.0;
            dotted(&mut svg, x, y + 32.0, rect.y - y - 32.0, true, "#49454f");
            (x - 12.0, y)
        } else {
            let marker_x = if figure == Figure::Docked && matches!(number, 6 | 7) {
                calendar.x + calendar.w + 48.0
            } else if figure == Figure::Docked && number == 10 {
                calendar.x - 64.0
            } else if trailing {
                rect.x + rect.w + 60.0
            } else {
                rect.x - 76.0
            };
            let y = rect.y + rect.h * 0.5;
            dotted(
                &mut svg,
                if trailing {
                    rect.x + rect.w
                } else {
                    marker_x + 24.0
                },
                y,
                if trailing {
                    marker_x - rect.x - rect.w
                } else {
                    rect.x - marker_x - 24.0
                },
                false,
                "#49454f",
            );
            (marker_x, y - 16.0)
        };
        annotations::marker(number, Dp(marker_x), Dp(marker_y));
    }
    annotations::lines(width, height, svg);
}

fn range_annotation_regions(
    figure: Figure,
    calendar: Rect,
    input: bool,
) -> Vec<(usize, Rect, bool)> {
    let full_screen = figure == Figure::FullScreen;
    let top = if full_screen { 64.0 } else { 0.0 };
    let header_y = calendar.y + top;
    let mut regions = vec![
        (
            1,
            Rect {
                x: calendar.x + if input { 24.0 } else { 64.0 },
                y: header_y + if input { 60.0 } else { 28.0 },
                w: 220.0,
                h: if input { 48.0 } else { 28.0 },
            },
            false,
        ),
        (
            2,
            Rect {
                x: calendar.x + if input { 24.0 } else { 64.0 },
                y: header_y + if input { 16.0 } else { 4.0 },
                w: 140.0,
                h: 20.0,
            },
            false,
        ),
        (
            if full_screen {
                6
            } else if input {
                5
            } else {
                3
            },
            Rect {
                x: calendar.x + 300.0,
                y: header_y + if input { 60.0 } else { 12.0 },
                w: 48.0,
                h: 48.0,
            },
            true,
        ),
        (
            if full_screen {
                7
            } else if input {
                8
            } else {
                15
            },
            Rect {
                x: calendar.x,
                y: header_y + if input { 120.0 } else { 68.0 },
                w: calendar.w,
                h: 1.0,
            },
            false,
        ),
    ];
    if full_screen {
        regions.push((
            3,
            Rect {
                x: calendar.x + 8.0,
                y: calendar.y + 8.0,
                w: 48.0,
                h: 48.0,
            },
            false,
        ));
        regions.push((
            5,
            Rect {
                x: calendar.x + calendar.w - 72.0,
                y: calendar.y + 8.0,
                w: 64.0,
                h: 48.0,
            },
            true,
        ));
        regions.push((
            4,
            Rect {
                x: calendar.x,
                y: calendar.y,
                w: calendar.w,
                h: calendar.h,
            },
            true,
        ));
    } else if input {
        regions.push((
            3,
            Rect {
                x: calendar.x,
                y: header_y,
                w: calendar.w,
                h: if input { 120.0 } else { 68.0 },
            },
            false,
        ));
        regions.push((
            4,
            Rect {
                x: calendar.x,
                y: calendar.y,
                w: calendar.w,
                h: calendar.h,
            },
            false,
        ));
        regions.push((
            7,
            Rect {
                x: calendar.x + 204.0,
                y: calendar.y + calendar.h - 60.0,
                w: 144.0,
                h: 48.0,
            },
            true,
        ));
    } else {
        regions.push((
            4,
            Rect {
                x: calendar.x,
                y: header_y,
                w: calendar.w,
                h: if input { 120.0 } else { 68.0 },
            },
            false,
        ));
        regions.push((
            8,
            Rect {
                x: calendar.x,
                y: calendar.y,
                w: calendar.w,
                h: calendar.h,
            },
            false,
        ));
    }
    if input && !full_screen {
        regions.push((
            6,
            Rect {
                x: calendar.x + 24.0,
                y: header_y + 145.0,
                w: 312.0,
                h: 56.0,
            },
            false,
        ));
    } else if !input {
        regions.push((
            if full_screen { 8 } else { 7 },
            Rect {
                x: calendar.x + 12.0,
                y: header_y + 69.0,
                w: 336.0,
                h: 48.0,
            },
            true,
        ));
        regions.push((
            if full_screen { 14 } else { 13 },
            Rect {
                x: calendar.x + 36.0,
                y: header_y + 137.0,
                w: 200.0,
                h: 20.0,
            },
            false,
        ));
    }
    regions
}

#[tessera]
fn modal_configurations() {
    configuration_canvas(
        1024.0,
        1353.0,
        RenderSlot::new(|| {
            column()
                .modifier(Modifier::new().width(Dp(360.0)))
                .children(|| {
                    modal_configuration(1, DatePickerView::Calendar, false);
                    gap(Dp(88.0));
                    modal_configuration(3, DatePickerView::Year, false);
                });
            range_configuration(2, false);
        }),
    );
    gap(Dp(24.0));
    text()
        .content("1  Single date selection     2  Date range selection     3  Year selection")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn input_configurations() {
    configuration_canvas(
        1018.0,
        480.0,
        RenderSlot::new(|| {
            modal_configuration(1, DatePickerView::Calendar, true);
            range_configuration(2, true);
        }),
    );
    gap(Dp(24.0));
    text()
        .content("1  Single date input     2  Date range input")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn configuration_canvas(width: f64, height: f64, content: RenderSlot) {
    date_canvas(
        width,
        height,
        RenderSlot::new(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .alignment(Alignment::TopCenter)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background_with_shape(
                            scheme.surface_container_lowest,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .padding_symmetric(Dp(40.0), Dp(48.0)),
                )
                .children(move || {
                    flow_row()
                        .modifier(Modifier::new().width(Dp(788.0)))
                        .item_spacing(Dp(68.0))
                        .line_spacing(Dp(48.0))
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children_shared(content.clone());
                });
        }),
    );
}

#[tessera]
fn configuration_number(number: usize) {
    boxed()
        .modifier(Modifier::new().width(Dp(360.0)).height(Dp(48.0)))
        .alignment(Alignment::TopCenter)
        .children(move || {
            annotations::marker(number, Dp::ZERO, Dp::ZERO);
        });
}

#[tessera]
fn modal_configuration(number: usize, initial_view: DatePickerView, input: bool) {
    let view = remember(|| initial_view);
    let state = remember(|| {
        DatePickerState::new(
            if input {
                None
            } else {
                Some(illustration_date(2025, 17))
            },
            YearMonth::new(2025, 8),
            2021..=2100,
            None,
            None,
            if input {
                DatePickerDisplayMode::Input
            } else {
                DatePickerDisplayMode::Picker
            },
        )
    });
    let open = remember(|| true);
    let status = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(360.0)))
        .children(move || {
            configuration_number(number);
            if open.get() {
                surface()
                    .modifier(Modifier::new().width(Dp(360.0)))
                    .style(scheme.surface_container_high.into())
                    .shape(Shape::rounded_rectangle(Dp(28.0)))
                    .block_input(false)
                    .child(move || {
                        date_picker_dialog()
                            .picker_today(illustration_today())
                            .state(state)
                            .picker_view(view)
                            .picker_title("Select date")
                            .picker_headline_optional(
                                if state.with(DatePickerState::display_mode)
                                    == DatePickerDisplayMode::Input
                                {
                                    Some("Enter dates".into())
                                } else {
                                    None
                                },
                            )
                            .picker_first_day_of_week(Weekday::Sunday)
                            .dismiss_button(move || {
                                button()
                                    .text()
                                    .accessibility_label(format!(
                                        "Cancel modal configuration {number}"
                                    ))
                                    .on_click(move || {
                                        open.set(false);
                                        status.set("Date selection cancelled".into());
                                    })
                                    .child(|| {
                                        text().content("Cancel");
                                    });
                            })
                            .confirm_button(move || {
                                button()
                                    .text()
                                    .accessibility_label(format!(
                                        "Confirm modal configuration {number}"
                                    ))
                                    .on_click(move || {
                                        open.set(false);
                                        status.set("Date selection confirmed".into());
                                    })
                                    .child(|| {
                                        text().content("OK");
                                    });
                            });
                    });
            } else {
                text().content(status.get()).style(type_style(14.0, 20.0));
                button()
                    .text()
                    .accessibility_label(format!("Reopen modal configuration {number}"))
                    .on_click(move || open.set(true))
                    .child(|| {
                        text().content("Open date picker");
                    });
            }
        });
}

#[tessera]
fn range_configuration(number: usize, input: bool) {
    let state = remember(|| {
        DateRangePickerState::new(
            if input {
                None
            } else {
                Some(illustration_date(2025, 17))
            },
            if input {
                None
            } else {
                Some(illustration_date(2025, 23))
            },
            YearMonth::new(2025, 8),
            2021..=2100,
            None,
            None,
            if input {
                DatePickerDisplayMode::Input
            } else {
                DatePickerDisplayMode::Picker
            },
        )
    });
    let committed = remember(|| {
        if input {
            (None, None)
        } else {
            (
                Some(illustration_date(2025, 17)),
                Some(illustration_date(2025, 23)),
            )
        }
    });
    let open = remember(|| true);
    let status = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().width(Dp(360.0)))
        .children(move || {
            configuration_number(number);
            if open.get() {
                surface()
                    .modifier(Modifier::new().width(Dp(360.0)))
                    .style(scheme.surface_container_high.into())
                    .shape(Shape::rounded_rectangle(if input {
                        Dp(28.0)
                    } else {
                        Dp::ZERO
                    }))
                    .block_input(false)
                    .child(move || {
                        column().children(move || {
                            if !input {
                                row()
                                    .modifier(
                                        Modifier::new()
                                            .fill_max_width()
                                            .height(Dp(64.0))
                                            .padding_all(Dp(8.0)),
                                    )
                                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                                    .children(move || {
                                        icon_button()
                                            .standard()
                                            .icon(filled::CLOSE_SVG)
                                            .accessibility_label("Close configuration date range")
                                            .on_click(move || {
                                                let (start, end) = committed.get();
                                                state.with_mut(|state| {
                                                    state.set_selection(start, end)
                                                });
                                                open.set(false);
                                            });
                                        button()
                                            .text()
                                            .accessibility_label("Save configuration date range")
                                            .enabled(state.with(DateRangePickerState::is_complete))
                                            .on_click(move || {
                                                if !state.with(DateRangePickerState::is_complete) {
                                                    return;
                                                }
                                                committed.set(state.with(|state| {
                                                    (
                                                        state.selected_start_date(),
                                                        state.selected_end_date(),
                                                    )
                                                }));
                                                open.set(false);
                                                status.set("Date range saved".into());
                                            })
                                            .child(|| {
                                                text().content("Save");
                                            });
                                    });
                            }
                            date_range_picker()
                                .today(illustration_today())
                                .state(state)
                                .first_day_of_week(Weekday::Sunday)
                                .title(if input {
                                    "Select date"
                                } else {
                                    "Depart - Return dates"
                                })
                                .headline_optional(if input {
                                    Some("Enter dates".into())
                                } else {
                                    None
                                })
                                .calendar_height(Dp(624.0));
                            figure_actions(status, open, state, committed);
                        });
                    });
            } else {
                text().content(status.get()).style(type_style(14.0, 20.0));
                button()
                    .text()
                    .accessibility_label("Reopen configuration date range")
                    .on_click(move || open.set(true))
                    .child(|| {
                        text().content("Open date picker");
                    });
            }
        });
}

#[tessera]
fn docked_configurations() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    date_canvas(
        1024.0,
        1365.0,
        RenderSlot::new(move || {
            boxed()
                .alignment(Alignment::TopCenter)
                .modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background_with_shape(
                            scheme.surface_container_lowest,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .padding_all(Dp(24.0)),
                )
                .children(|| {
                    flow_row()
                        .modifier(Modifier::new().width(Dp(788.0)))
                        .item_spacing(Dp(68.0))
                        .line_spacing(Dp(32.0))
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children(|| {
                            docked_configuration(1, DockedDatePickerView::Calendar);
                            docked_configuration(2, DockedDatePickerView::Month);
                            docked_configuration(3, DockedDatePickerView::Year);
                        });
                });
        }),
    );
    gap(Dp(24.0));
    text()
        .content("1  Day selection     2  Month selection     3  Year selection")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn docked_configuration(number: usize, initial_view: DockedDatePickerView) {
    let view = remember(|| initial_view);
    let open = remember(|| true);
    let state = remember(|| {
        DatePickerState::new(
            Some(illustration_date(2025, 17)),
            YearMonth::new(2025, 8),
            2025..=2100,
            None,
            None,
            DatePickerDisplayMode::Picker,
        )
    });
    let draft = remember(|| "08/17/2025".to_owned());
    let committed = remember(|| illustration_date(2025, 17));
    let committed_draft = remember(|| "08/17/2025".to_owned());
    let error = remember(String::new);
    let controller = remember(|| {
        let mut value = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        value.set_text("08/17/2025");
        value
    });
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .width(Dp(360.0))
                .default_min_size(Dp::ZERO, Dp(606.0)),
        )
        .children(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(Modifier::new().size(Dp(24.0), Dp(24.0)).border_with_shape(
                            Dp(1.0),
                            scheme.outline,
                            Shape::CAPSULE,
                        ))
                        .children(move || {
                            text()
                                .content(number.to_string())
                                .style(type_style(14.0, 20.0));
                        });
                });
            gap(Dp(38.0));
            TextFieldBuilder::outlined()
                .modifier(Modifier::new().width(Dp(312.0)).height(Dp(56.0)))
                .min_width(Dp::ZERO)
                .controller(controller)
                .label("Date")
                .accessibility_label(format!("Configuration {number} date"))
                .accessibility_description(if error.with(String::is_empty) {
                    "MM/DD/YYYY".to_owned()
                } else {
                    error.get()
                })
                .is_error(!error.with(String::is_empty))
                .on_change(move |raw: String| {
                    draft.set(raw.clone());
                    raw
                })
                .on_submit(move || {
                    submit_docked_date(state, controller, draft, committed, committed_draft, error);
                })
                .trailing_icon(move || {
                    icon_button()
                        .standard()
                        .icon(filled::CALENDAR_TODAY_SVG)
                        .accessibility_label(format!("Toggle configuration {number} calendar"))
                        .on_click(move || {
                            if open.get() {
                                restore_docked_date(
                                    state,
                                    controller,
                                    draft,
                                    committed,
                                    committed_draft,
                                    error,
                                );
                            }
                            open.set(!open.get());
                            FocusManager::current().defer_keyboard_until_layout();
                        });
                });
            gap(Dp(4.0));
            text()
                .content(if error.with(String::is_empty) {
                    "MM/DD/YYYY".to_owned()
                } else {
                    error.get()
                })
                .style(type_style(12.0, 16.0));
            gap(Dp(8.0));
            if open.get() {
                docked_date_picker()
                    .today(illustration_today())
                    .state(state)
                    .view(view)
                    .first_day_of_week(Weekday::Sunday)
                    .on_date_change(move |date: CalendarDate| {
                        let formatted =
                            format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year());
                        draft.set(formatted.clone());
                        controller.with_mut(|controller| controller.set_text(&formatted));
                        error.set(String::new());
                    })
                    .on_cancel(move || {
                        restore_docked_date(
                            state,
                            controller,
                            draft,
                            committed,
                            committed_draft,
                            error,
                        );
                        open.set(false);
                        FocusManager::current().defer_keyboard_until_layout();
                    })
                    .on_confirm(move || {
                        if submit_docked_date(
                            state,
                            controller,
                            draft,
                            committed,
                            committed_draft,
                            error,
                        ) {
                            open.set(false);
                            FocusManager::current().defer_keyboard_until_layout();
                        }
                    });
            }
        });
}

fn docked_menu_selected_row(state: State<DatePickerState>, view: DockedDatePickerView) -> usize {
    let displayed = state.with(DatePickerState::displayed_month);
    if view == DockedDatePickerView::Month {
        let month = displayed.month();
        usize::from(month - 1 - month.saturating_sub(4).min(5))
    } else {
        let first = state.with(|state| *state.year_range().start());
        usize::try_from((displayed.year() - first).clamp(0, 3)).unwrap_or(0)
    }
}

fn docked_calendar_header_regions(calendar: Rect) -> [Rect; 2] {
    let spacing = ((calendar.w - 24.0 - 328.0) / 5.0).max(0.0);
    [
        Rect {
            x: calendar.x + 60.0 + spacing,
            y: calendar.y + 12.0,
            w: 64.0,
            h: 44.0,
        },
        Rect {
            x: calendar.x + 220.0 + spacing * 4.0,
            y: calendar.y + 12.0,
            w: 72.0,
            h: 44.0,
        },
    ]
}

fn docked_header_regions(calendar: Rect) -> [Rect; 2] {
    [
        Rect {
            x: calendar.x + 58.0,
            y: calendar.y + 8.0,
            w: 64.0,
            h: 48.0,
        },
        Rect {
            x: calendar.x + 234.0,
            y: calendar.y + 8.0,
            w: 72.0,
            h: 48.0,
        },
    ]
}

fn modal_day_visual(cell: Rect) -> Rect {
    Rect {
        x: cell.x + 4.0,
        y: cell.y + 124.0,
        w: 40.0,
        h: 40.0,
    }
}

#[tessera]
fn anatomy_legend(figure: Figure, view: DockedDatePickerView) {
    let labels: Vec<&str> = match figure {
        Figure::Docked if view != DockedDatePickerView::Calendar => vec![
            "Outlined text field",
            if view == DockedDatePickerView::Month {
                "Month selection (pressed)"
            } else {
                "Month selection (disabled)"
            },
            if view == DockedDatePickerView::Year {
                "Year selection (pressed)"
            } else {
                "Year selection (disabled)"
            },
            "Header",
            "Menu",
            "Selected list item",
            "Unselected menu list item",
            "Container",
        ],
        Figure::Docked => vec![
            "Outlined text field",
            "Menu button: Month selection",
            "Menu button: Year selection",
            "Icon button",
            "Weekdays label text",
            "Unselected date",
            "Today's date",
            "Outside month date",
            "Text buttons",
            "Selected date",
            "Container",
        ],
        Figure::Input | Figure::RangeInput => vec![
            "Headline",
            "Supporting text",
            "Header",
            "Container",
            "Icon button",
            "Outlined text field",
            "Text buttons",
            "Divider",
        ],
        Figure::Range => vec![
            "Headline",
            "Supporting text",
            "Icon button",
            "Header",
            "Text button",
            "Icon button",
            "Weekdays label text",
            "Container",
            "Today's date",
            "Unselected date",
            "In-range active indicator",
            "In-range date",
            "Month subhead",
            "Selected date",
            "Divider",
        ],
        _ => vec![
            "Headline",
            "Supporting text",
            "Header",
            "Container",
            "Icon button",
            "Icon buttons",
            "Weekdays",
            "Today's date",
            "Unselected date",
            "Text buttons",
            "Selected date",
            "Menu button",
            "Divider",
        ],
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, label) in labels.iter().copied().enumerate() {
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
                                text()
                                    .content((index + 1).to_string())
                                    .color(scheme.surface)
                                    .style(type_style(14.0, 20.0));
                            });
                        boxed()
                            .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO))
                            .children(move || {
                                text().content(label).style(type_style(16.0, 24.0));
                            });
                    });
                gap(Dp(8.0));
            }
        });
}

#[cfg(test)]
#[path = "date_pickers_tests.rs"]
mod tests;

fn docked_day_bounds(calendar: Rect, month: YearMonth, date: CalendarDate) -> Option<Rect> {
    if date.year() != month.year() || date.month() != month.month() {
        return None;
    }
    let year = i64::from(month.year()) - i64::from(month.month() < 3);
    let month_offset = [0_i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4][usize::from(month.month() - 1)];
    let first_weekday = (year + year.div_euclid(4) - year.div_euclid(100)
        + year.div_euclid(400)
        + month_offset
        + 1)
    .rem_euclid(7);
    let index = first_weekday + i64::from(date.day() - 1);
    Some(Rect {
        x: calendar.x + 12.0 + (index % 7) as f64 * 48.0,
        y: calendar.y + 104.0 + (index / 7) as f64 * 48.0,
        w: 48.0,
        h: 48.0,
    })
}
