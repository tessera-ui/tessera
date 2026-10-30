//! Native date-input accessibility illustrations.
//!
//! ## Usage
//!
//! Explore date entry, formatting, clearing, and accessible labels.

use tessera_components::{
    boxed::boxed,
    button::button,
    date_picker::{
        CalendarDate, DatePickerDisplayMode, DatePickerState, DockedDatePickerView, Weekday,
        YearMonth, date_picker_day, date_picker_dialog, docked_date_picker,
    },
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ClickableArgs, InteractionVisualState, ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::{RoundedCorner, Shape},
    surface::surface,
    text::text,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, FocusManager, Modifier, RenderSlot, State,
    accesskit::{HasPopup, Role},
    modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use super::super::{
    annotations::{self, PreciseGeometry, watch_precise},
    article::modal_host::hosted_dialog,
    type_style,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Illustration {
    #[default]
    EntryMethods,
    Formatting,
    Clear,
    Labels,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum FieldGeometry {
    #[default]
    Field,
    Raw,
    Formatted,
}

impl FieldGeometry {
    fn name(self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::Raw => "raw",
            Self::Formatted => "formatted",
        }
    }
}

#[tessera]
pub(super) fn illustration(kind: Illustration) {
    match kind {
        Illustration::EntryMethods => entry_methods(),
        Illustration::Formatting => formatting(),
        Illustration::Clear => optional_clear(),
        Illustration::Labels => field_labels(),
    }
}

fn initial_state(year: i32) -> DatePickerState {
    DatePickerState::new(
        Some(CalendarDate::new(year, 8, 17).unwrap()),
        YearMonth::new(year, 8),
        2020..=2030,
        None,
        None,
        DatePickerDisplayMode::Picker,
    )
}

fn initial_controller(value: &str) -> TextInputController {
    let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
    controller.set_text(value);
    controller
}

fn format_date(date: CalendarDate, year_first: bool) -> String {
    if year_first {
        format!("{:04}/{:02}/{:02}", date.year(), date.month(), date.day())
    } else {
        format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year())
    }
}

#[tessera]
fn canvas(
    label: String,
    geometry: State<PreciseGeometry>,
    content: RenderSlot,
    width: Option<f64>,
) {
    let width = width.unwrap_or(1352.0);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    #[cfg(all(test, not(target_family = "wasm")))]
    tests::GEOMETRY.set(Some(geometry));
    super::date_canvas(
        width,
        600.0,
        RenderSlot::new(move || {
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
                        .size(Dp(width), Dp(600.0))
                        .background(scheme.surface_container_low)
                        .clip_to_bounds()
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(label.clone()),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    let content = content.clone();
                    scaled_box(2.0).child(move || {
                        boxed()
                            .modifier(Modifier::new().size(Dp(width / 2.0), Dp(300.0)))
                            .children_shared(content.clone());
                    });
                });
        }),
    );
}

fn annotation_color(color: Color) -> String {
    let channel = |value: f32| {
        let encoded = if value <= 0.0031308 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

#[tessera]
fn annotation_contour(path: String) {
    let primary = use_context::<MaterialTheme>()
        .unwrap()
        .get()
        .color_scheme
        .primary;
    boxed()
        .modifier(Modifier::new().semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }))
        .children(move || {
            annotations::lines(
                Dp(676.0),
                Dp(300.0),
                format!(
                    "<path d='{path}' fill='none' stroke='{}' stroke-width='4'/>",
                    annotation_color(primary)
                ),
            );
        });
}

#[tessera]
fn indication(x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().modifier(
        Modifier::new()
            .offset(Dp(x), Dp(y))
            .size(Dp(width), Dp(height))
            .border_with_shape(
                Dp(4.0),
                scheme.primary,
                Shape::rounded_rectangle(Dp(radius)),
            )
            .semantics(SemanticsArgs {
                hidden: true,
                ..Default::default()
            }),
    );
}

#[tessera]
fn date_field(
    controller: State<TextInputController>,
    open: State<bool>,
    geometry: State<PreciseGeometry>,
    geometry_key: FieldGeometry,
    label: String,
    accessible_name: String,
    format: String,
    x: f64,
    y: f64,
    on_change: CallbackWith<String, String>,
    on_submit: Callback,
    focused: Option<bool>,
    show_calendar: Option<bool>,
    error: Option<State<String>>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let icon_name = format!("Open {accessible_name} calendar");
    let show_calendar = show_calendar.unwrap_or(true);
    let error = error.map(|error| error.get()).unwrap_or_default();
    let had_focus = remember(|| false);
    TextFieldBuilder::outlined()
        .controller(controller)
        .label(label)
        .accessibility_label(accessible_name)
        .accessibility_description(if error.is_empty() {
            format.clone()
        } else {
            error.clone()
        })
        .is_error(!error.is_empty())
        .min_width(Dp::ZERO)
        .presentation_state(if focused.unwrap_or(false) {
            InteractionVisualState::Focus
        } else {
            InteractionVisualState::Rest
        })
        .modifier(
            watch_precise(geometry, geometry_key.name())
                .offset(Dp(x), Dp(y))
                .size(Dp(342.0), Dp(56.0))
                .focus_group()
                .on_focus_changed(move |focus: tessera_ui::FocusState| {
                    let focused = focus.has_focus();
                    let previously_focused = had_focus.get();
                    had_focus.set(focused);
                    if previously_focused && !focused {
                        on_submit.call();
                    }
                }),
        )
        .on_change_shared(on_change)
        .on_submit_shared(on_submit)
        .trailing_icon_shared(RenderSlot::new(move || {
            if show_calendar {
                icon_button()
                    .standard()
                    .icon(filled::CALENDAR_TODAY_SVG)
                    .presentation_state(InteractionVisualState::Hover)
                    .accessibility_label(icon_name.clone())
                    .on_click(move || {
                        open.set(!open.get());
                        FocusManager::current().defer_keyboard_until_layout();
                    });
            }
        }));
    text()
        .content(if error.is_empty() {
            format
        } else {
            error.clone()
        })
        .style(type_style(12.0, 16.0))
        .color(if error.is_empty() {
            scheme.on_surface_variant
        } else {
            scheme.error
        })
        .modifier(Modifier::new().offset(Dp(x + 16.0), Dp(y + 60.0)));
}

#[tessera]
fn calendar_dialog(
    open: State<bool>,
    state: State<DatePickerState>,
    on_confirm: Callback,
    content: RenderSlot,
) {
    hosted_dialog(open)
        .accessibility_label("Optional date selection")
        .on_close_request(move || open.set(false))
        .main_content_shared(content)
        .dialog_content(move || {
            date_picker_dialog()
                .state(state)
                .picker_today(CalendarDate::new(2023, 8, 1).unwrap())
                .picker_first_day_of_week(Weekday::Sunday)
                .dismiss_button(move || {
                    button()
                        .text()
                        .accessibility_label("Cancel optional date selection")
                        .on_click(move || open.set(false))
                        .child(|| {
                            text().content("Cancel");
                        });
                })
                .confirm_button(move || {
                    button()
                        .text()
                        .accessibility_label("Confirm optional date selection")
                        .on_click(move || {
                            on_confirm.call();
                            open.set(false);
                        })
                        .child(|| {
                            text().content("OK");
                        });
                });
        });
}

#[tessera]
fn entry_methods() {
    let controller = remember(|| initial_controller("08/17/2023"));
    let draft = remember(|| "08/17/2023".to_owned());
    #[cfg(test)]
    {
        tests::ENTRY_DRAFT.set(Some(draft));
    }
    let error = remember(String::new);
    let state = remember(|| initial_state(2023));
    let open = remember(|| false);
    let geometry = remember(PreciseGeometry::new);
    calendar_dialog(
        open,
        state,
        Callback::new(move || {
            if let Some(date) = state.with(DatePickerState::selected_date) {
                let value = format_date(date, false);
                error.set(String::new());
                draft.set(value.clone());
                controller.with_mut(|controller| controller.set_text(&value));
            }
        }),
        RenderSlot::new(move || {
            canvas(
                "Date entry methods diagram".into(),
                geometry,
                RenderSlot::new(move || {
                    date_field(
                        controller,
                        open,
                        geometry,
                        FieldGeometry::Field,
                        "Date".into(),
                        "Entry methods date".into(),
                        "MM/DD/YYYY".into(),
                        167.0,
                        122.0,
                        CallbackWith::new(move |value: String| {
                            #[cfg(test)]
                            tests::ENTRY_CHANGES.set(tests::ENTRY_CHANGES.get() + 1);
                            error.set(String::new());
                            draft.set(value.clone());
                            value
                        }),
                        Callback::new(move || {
                            if let Some(date) = super::parse_form_date(&draft.get()) {
                                state.with_mut(|state| state.set_selected_date(date));
                                error.set(String::new());
                                let value = format_date(date, false);
                                draft.set(value.clone());
                                controller.with_mut(|controller| controller.set_text(&value));
                            } else {
                                error.set("Enter a valid date (MM/DD/YYYY)".into());
                            }
                        }),
                    )
                    .error(error);
                    annotation_contour(
                        "M213.5 118 H441.5 A6 6 0 0 1 447.5 124 V176 A6 6 0 0 1 441.5 182 H169.5 A6 6 0 0 1 163.5 176 V124 A6 6 0 0 1 169.5 118 H179.5".into(),
                    );
                    annotation_contour(
                        "M461.5 118 H509.5 A6 6 0 0 1 515.5 124 V176 A6 6 0 0 1 509.5 182 H461.5 A6 6 0 0 1 455.5 176 V124 A6 6 0 0 1 461.5 118 Z".into(),
                    );
                }),
            )
            .width(1353.0);
        }),
    );
}

#[tessera]
fn formatting() {
    let raw = remember(|| initial_controller("08172023"));
    let formatted = remember(|| initial_controller("08/17/2023"));
    let raw_draft = remember(|| "08172023".to_owned());
    let formatted_draft = remember(|| "08/17/2023".to_owned());
    let raw_error = remember(String::new);
    let formatted_error = remember(String::new);
    let state = remember(|| initial_state(2023));
    let open = remember(|| false);
    let geometry = remember(PreciseGeometry::new);
    let submit = Callback::new(move || {
        if let Some(date) = super::parse_form_date(&raw_draft.get()) {
            state.with_mut(|state| state.set_selected_date(date));
            raw_error.set(String::new());
            formatted_error.set(String::new());
            let value = format_date(date, false);
            raw_draft.set(value.clone());
            formatted_draft.set(value.clone());
            raw.with_mut(|controller| controller.set_text(&value));
            formatted.with_mut(|controller| controller.set_text(&value));
        } else {
            raw_error.set("Enter a valid date (MM/DD/YYYY)".into());
        }
    });
    calendar_dialog(
        open,
        state,
        Callback::new(move || {
            if let Some(date) = state.with(DatePickerState::selected_date) {
                raw_error.set(String::new());
                formatted_error.set(String::new());
                let value = format_date(date, false);
                raw_draft.set(value.clone());
                formatted_draft.set(value.clone());
                raw.with_mut(|controller| controller.set_text(&value));
                formatted.with_mut(|controller| controller.set_text(&value));
            }
        }),
        RenderSlot::new(move || {
            canvas(
                "Deferred date formatting diagram".into(),
                geometry,
                RenderSlot::new(move || {
                    date_field(
                        raw,
                        open,
                        geometry,
                        FieldGeometry::Raw,
                        "Date".into(),
                        "Unformatted date".into(),
                        "MM/DD/YYYY".into(),
                        167.0,
                        62.0,
                        CallbackWith::new(move |value: String| {
                            raw_error.set(String::new());
                            raw_draft.set(value.clone());
                            value
                        }),
                        submit,
                    )
                    .focused(true)
                    .error(raw_error);
                    date_field(
                        formatted,
                        open,
                        geometry,
                        FieldGeometry::Formatted,
                        "Date".into(),
                        "Formatted date".into(),
                        "MM/DD/YYYY".into(),
                        167.0,
                        172.0,
                        CallbackWith::new(move |value: String| {
                            formatted_error.set(String::new());
                            formatted_draft.set(value.clone());
                            value
                        }),
                        Callback::new(move || {
                            if let Some(date) = super::parse_form_date(&formatted_draft.get()) {
                                state.with_mut(|state| state.set_selected_date(date));
                                formatted_error.set(String::new());
                                let value = format_date(date, false);
                                formatted_draft.set(value.clone());
                                formatted.with_mut(|c| c.set_text(&value));
                            } else {
                                formatted_error.set("Enter a valid date (MM/DD/YYYY)".into());
                            }
                        }),
                    )
                    .error(formatted_error);
                }),
            );
        }),
    );
}

#[tessera]
fn optional_clear() {
    let state = remember(|| initial_state(2020));
    let committed = remember(|| Some(CalendarDate::new(2020, 8, 17).unwrap()));
    #[cfg(all(test, not(target_family = "wasm")))]
    {
        tests::CLEAR_STATE.set(Some(state));
        tests::CLEAR_COMMITTED.set(Some(committed));
    }
    let geometry = remember(PreciseGeometry::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    canvas(
        "Optional clear action diagram".into(),
        geometry,
        RenderSlot::new(move || {
            dotted_background();
            surface()
                .style(scheme.surface_container_high.into())
                .shape(Shape::RoundedRectangle {
                    top_left: RoundedCorner::manual(Dp::ZERO, 2.0),
                    top_right: RoundedCorner::manual(Dp::ZERO, 2.0),
                    bottom_right: RoundedCorner::manual(Dp(28.0), 2.0),
                    bottom_left: RoundedCorner::manual(Dp(28.0), 2.0),
                })
                .block_input(false)
                .modifier(
                    watch_precise(geometry, "calendar")
                        .offset(Dp(158.0), Dp::ZERO)
                        .size(Dp(360.0), Dp(252.0)),
                )
                .child(move || {
                    for day in 9..=31 {
                        let date = CalendarDate::new(2020, 8, day).unwrap();
                        let index = (day - 9) as f64;
                        date_picker_day(date)
                            .selected(state.with(|state| state.selected_date() == Some(date)))
                            .modifier(Modifier::new().offset(
                                Dp(12.0 + (index % 7.0) * 48.0),
                                Dp((index / 7.0).floor() * 48.0),
                            ))
                            .on_select(move |date| {
                                state.with_mut(|state| state.set_selected_date(date));
                            });
                    }
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp::ZERO, Dp(192.0))
                                .size(Dp(360.0), Dp(60.0)),
                        )
                        .children(move || {
                            for (caption, x, width) in [
                                ("Clear", 12.0, 60.0),
                                ("Cancel", 212.0, 72.0),
                                ("OK", 300.0, 48.0),
                            ] {
                                button()
                                    .text()
                                    .content_padding(Padding::symmetric(Dp(12.0), Dp::ZERO))
                                    .modifier(
                                        Modifier::new()
                                            .offset(Dp(x), Dp(4.0))
                                            .size(Dp(width), Dp(48.0)),
                                    )
                                    .accessibility_label(format!("Optional clear {caption}"))
                                    .on_click(move || match caption {
                                        "Clear" => {
                                            state.with_mut(DatePickerState::clear_selected_date)
                                        }
                                        "Cancel" => {
                                            if let Some(date) = committed.get() {
                                                state.with_mut(|state| {
                                                    state.set_selected_date(date)
                                                });
                                            } else {
                                                state
                                                    .with_mut(DatePickerState::clear_selected_date);
                                            }
                                        }
                                        _ => committed
                                            .set(state.with(DatePickerState::selected_date)),
                                    })
                                    .child(move || {
                                        text().content(caption);
                                    });
                            }
                        });
                });
            indication(175.0, 202.0, 49.0, 36.0, 10.0);
        }),
    );
}

#[tessera]
fn dotted_background() {
    let mut svg = String::new();
    for y in (10..300).step_by(8) {
        for x in (10..677).step_by(8) {
            svg.push_str(&format!(
                "<circle cx='{x}' cy='{y}' r='.45' fill='#79747e'/>"
            ));
        }
    }
    annotations::lines(Dp(676.5), Dp(300.0), svg);
}

#[tessera]
fn field_labels() {
    let controller = remember(|| initial_controller("2023/08/17"));
    let draft = remember(|| "2023/08/17".to_owned());
    let error = remember(String::new);
    let initial = {
        let mut state = initial_state(2023);
        state.set_displayed_month(YearMonth::new(2025, 8).unwrap());
        state
    };
    let state = remember(|| initial);
    let geometry = remember(PreciseGeometry::new);
    let chooser = remember(|| false);
    let view = remember(|| DockedDatePickerView::Month);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    hosted_dialog(chooser)
        .accessibility_label("Labeled date calendar chooser")
        .on_close_request(move || chooser.set(false))
        .dialog_content(move || {
            docked_date_picker()
                .state(state)
                .view(view)
                .today(CalendarDate::new(2025, 8, 1).unwrap())
                .first_day_of_week(Weekday::Sunday)
                .on_date_change(move |date| {
                    let value = format_date(date, true);
                    error.set(String::new());
                    draft.set(value.clone());
                    controller.with_mut(|c| c.set_text(&value));
                })
                .on_cancel(move || chooser.set(false))
                .on_confirm(move || chooser.set(false));
        })
        .main_content(move || {
            canvas(
                "Accessible date label diagram".into(),
                geometry,
                RenderSlot::new(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(34.0), Dp(50.0))
                                .size(Dp(327.0), Dp(148.0))
                                .background_with_shape(
                                    scheme.surface,
                                    Shape::rounded_rectangle(Dp(10.0)),
                                )
                                ,
                        )
                        .children(move || {
                            for (value, y, emphasized) in [
                                ("Label", 23.0, false),
                                ("UI text (“Event date, YYYY/MM/DD”)", 45.0, true),
                                ("Role", 79.0, false),
                                ("Combo box", 103.0, true),
                            ] {
                                let mut style = type_style(16.0, 24.0);
                                if emphasized {
                                    style.font_weight = 700;
                                }
                                text()
                                    .content(value)
                                    .style(style)
                                    .color(if emphasized {
                                        scheme.primary
                                    } else {
                                        scheme.on_surface
                                    })
                                    .modifier(Modifier::new().offset(Dp(22.0), Dp(y)));
                            }
                        });
                    annotation_contour(
                        "M44 50 H351 A10 10 0 0 1 361 60 V188 A10 10 0 0 1 351 198 H44 A10 10 0 0 1 34 188 V60 A10 10 0 0 1 44 50 Z".into(),
                    );
                    boxed().modifier(
                        Modifier::new()
                            .offset(Dp(361.0), Dp(98.0))
                            .size(Dp(30.0), Dp(4.0))
                            .background(scheme.primary),
                    );
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(395.0), Dp(70.0))
                                .size(Dp(342.0), Dp(56.0))
                                .semantics(SemanticsArgs {
                                    expanded: Some(chooser.get()),
                                    has_popup: Some(HasPopup::Dialog),
                                    merge_descendants: true,
                                    ..Default::default()
                                })
                                .clickable_with(ClickableArgs {
                                    role: Some(Role::ComboBox),
                                    label: Some("Event date, YYYY/MM/DD".into()),
                                    on_click: Callback::new(move || {
                                        view.set(DockedDatePickerView::Calendar);
                                        chooser.set(true);
                                        FocusManager::current().defer_keyboard_until_layout();
                                    }),
                                    ..Default::default()
                                }),
                        )
                        .children(move || {
                            date_field(
                                controller,
                                chooser,
                                geometry,
                                FieldGeometry::Field,
                                "Event Date".into(),
                                "Event date".into(),
                                "YYYY/MM/DD".into(),
                                0.0,
                                0.0,
                                CallbackWith::new(move |value: String| {
                                    error.set(String::new());
                                    draft.set(value.clone());
                                    value
                                }),
                                Callback::new(move || {
                                    if let Some(date) = super::parse_form_date(&draft.get()) {
                                        state.with_mut(|state| state.set_selected_date(date));
                                        error.set(String::new());
                                        let value = format_date(date, true);
                                        draft.set(value.clone());
                                        controller.with_mut(|c| c.set_text(&value));
                                    } else {
                                        error.set("Enter a valid date (YYYY/MM/DD)".into());
                                    }
                                }),
                            )
                            .focused(true)
                            .show_calendar(false)
                            .error(error);
                        });
                    visible_calendar_header(state, chooser, view);
                    annotation_contour(
                        "M475 65 H735 A6 6 0 0 1 741 71 V122 A6 6 0 0 1 735 128 H397 A6 6 0 0 1 391 122 V71 A6 6 0 0 1 397 65 H407".into(),
                    );
                }),
            );
        });
}

#[tessera]
fn visible_calendar_header(
    state: State<DatePickerState>,
    chooser: State<bool>,
    view: State<DockedDatePickerView>,
) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let month = state.with(DatePickerState::displayed_month);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(395.0), Dp(152.0))
                .size(Dp(360.0), Dp(500.0))
                .background_with_shape(
                    theme.color_scheme.surface_container_high,
                    Shape::rounded_rectangle(Dp(16.0)),
                ),
        )
        .children(move || {
            for (x, name, year) in [
                (0.0, "Previous labeled month", false),
                (192.0, "Previous labeled year", true),
            ] {
                icon_button()
                    .standard()
                    .icon(filled::CHEVRON_LEFT_SVG)
                    .modifier(Modifier::new().offset(Dp(x), Dp(8.0)))
                    .accessibility_label(name)
                    .enabled(if year {
                        month.year() > 2020
                    } else {
                        month.year() > 2020 || month.month() > 1
                    })
                    .on_click(move || {
                        if year {
                            state.with_mut(|state| {
                                let month = state.displayed_month();
                                state.set_displayed_month(
                                    YearMonth::new((month.year() - 1).max(2020), month.month())
                                        .unwrap(),
                                );
                            });
                        } else {
                            state.with_mut(DatePickerState::previous_month);
                        }
                    });
            }
            icon_button()
                .standard()
                .icon(filled::CHEVRON_RIGHT_SVG)
                .modifier(Modifier::new().offset(Dp(112.0), Dp(8.0)))
                .accessibility_label("Next labeled month")
                .enabled(month.year() < 2030 || month.month() < 12)
                .on_click(move || state.with_mut(DatePickerState::next_month));
            for (x, width, label, target) in [
                (
                    48.0,
                    64.0,
                    months[month.month() as usize - 1].to_owned(),
                    DockedDatePickerView::Month,
                ),
                (
                    240.0,
                    72.0,
                    month.year().to_string(),
                    DockedDatePickerView::Year,
                ),
            ] {
                button()
                    .text()
                    .shape(Shape::rounded_rectangle(Dp(8.0)))
                    .content_padding(Padding::symmetric(Dp(8.0), Dp::ZERO))
                    .modifier(
                        Modifier::new()
                            .offset(Dp(x), Dp(8.0))
                            .size(Dp(width), Dp(48.0)),
                    )
                    .accessibility_label(if target == DockedDatePickerView::Month {
                        "Choose labeled month"
                    } else {
                        "Choose labeled year"
                    })
                    .on_click(move || {
                        view.set(target);
                        chooser.set(true);
                        FocusManager::current().defer_keyboard_until_layout();
                    })
                    .child(move || {
                        let label = label.clone();
                        row()
                            .cross_axis_alignment(
                                tessera_components::alignment::CrossAxisAlignment::Center,
                            )
                            .children(move || {
                                text()
                                    .content(label.clone())
                                    .style(theme.typography.label_large);
                                boxed().modifier(Modifier::new().width(Dp(4.0)));
                                icon().vector(filled::ARROW_DROP_DOWN_SVG).size(Dp(18.0));
                            });
                    });
            }
            for (index, name) in ["S", "M", "T", "W", "T", "F"].into_iter().enumerate() {
                boxed()
                    .modifier(
                        Modifier::new()
                            .offset(Dp(12.0 + index as f64 * 48.0), Dp(64.0))
                            .size(Dp(48.0), Dp(48.0)),
                    )
                    .alignment(tessera_components::alignment::Alignment::Center)
                    .children(move || {
                        text().content(name).style(theme.typography.body_large);
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "date_instructional_tests.rs"]
mod tests;
