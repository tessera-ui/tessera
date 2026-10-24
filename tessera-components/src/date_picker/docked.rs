//! Compact calendars for date fields and anchored date selection.
//!
//! ## Usage
//!
//! Place a calendar below a date field in an anchored popup or form.

use tessera_ui::{
    Callback, CallbackWith, Dp, Modifier, Px, PxPosition, State, accesskit::Role, layout::layout,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    divider::horizontal_divider,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    scrollable::ScrollableController,
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialAlpha, MaterialTheme},
};

use super::{
    CalendarDate, CalendarFocus, CalendarKeyboard, CalendarOwner, DATE_CELL_SIZE, DATE_COLUMNS,
    DATE_PICKER_MAX_WIDTH, DateGridLayout, DatePickerState, Weekday, YearMonth, calendar_day,
    civil_from_days, days_from_civil, format_month_name, is_date_selectable, weekday_for_date,
    weekday_narrow_label, weekday_sequence,
};

/// The content displayed beneath a docked calendar's navigation header.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DockedDatePickerView {
    /// The weekday headings and date grid.
    #[default]
    Calendar,
    /// The list of months in the displayed year.
    Month,
    /// The list of years allowed by the date-picker state.
    Year,
}

/// # docked_date_picker
///
/// Select a date with separate month and year menus beside a date-entry field.
///
/// ## Usage
///
/// Show an anchored calendar for dates in the near or distant past or future.
///
/// ## Parameters
///
/// - `modifier` — placement and optional constraints on the 360dp calendar.
/// - `state` — selection, displayed month and selectable-date restrictions.
/// - `view` — optional shared calendar/month/year view state; when omitted, the
///   calendar manages its own view.
/// - `first_day_of_week` — first weekday in the calendar, Monday by default.
/// - `today` — date outlined as today; defaults to the system date.
/// - `show_actions` — show Cancel and OK, enabled by default.
/// - `on_date_change` — called after an allowed date is selected.
/// - `on_cancel`, `on_confirm` — dismissal and confirmation actions; the owner
///   controls the popup and any rollback of an unconfirmed selection.
///
/// ## Examples
///
/// ```
/// use tessera_components::date_picker::{CalendarDate, DatePickerState};
/// let mut state = DatePickerState::default();
/// let selected = CalendarDate::new(2025, 8, 17).unwrap();
/// assert!(state.set_selected_date(selected));
/// assert_eq!(state.selected_date(), Some(selected));
/// ```
#[tessera]
pub fn docked_date_picker(
    modifier: Option<Modifier>,
    state: Option<State<DatePickerState>>,
    view: Option<State<DockedDatePickerView>>,
    first_day_of_week: Option<Weekday>,
    today: Option<CalendarDate>,
    show_actions: Option<bool>,
    on_date_change: Option<CallbackWith<CalendarDate>>,
    on_cancel: Option<Callback>,
    on_confirm: Option<Callback>,
) {
    let internal = remember(DatePickerState::default);
    let state = state.unwrap_or(internal);
    let internal_view = remember(DockedDatePickerView::default);
    let menu = view.unwrap_or(internal_view);
    let first_day = first_day_of_week.unwrap_or_default();
    let today = today.unwrap_or_else(CalendarDate::today);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let change = on_date_change.unwrap_or_else(|| CallbackWith::new(|_| {}));
    let corner = RoundedCorner::manual(Dp(16.0), 2.0);
    surface()
        .modifier(modifier.unwrap_or_default().width(DATE_PICKER_MAX_WIDTH))
        .shape(Shape::RoundedRectangle {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
        })
        .style(scheme.surface_container_high.into())
        .accessibility_role(Role::Group)
        .accessibility_label("Docked calendar")
        .block_input(false)
        .child(move || {
            if menu.get() != DockedDatePickerView::Calendar {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        menu_header(state, menu);
                        match menu.get() {
                            DockedDatePickerView::Month => {
                                month_menu(state, menu);
                            }
                            DockedDatePickerView::Year => {
                                year_menu(state, menu);
                            }
                            DockedDatePickerView::Calendar => unreachable!(),
                        }
                        spacer().modifier(Modifier::new().height(Dp(28.0)));
                    });
            } else {
                calendar_content(
                    state,
                    menu,
                    first_day,
                    today,
                    change,
                    show_actions.unwrap_or(true),
                    on_cancel.unwrap_or_default(),
                    on_confirm.unwrap_or_default(),
                );
            }
        });
}

#[tessera]
fn calendar_content(
    state: State<DatePickerState>,
    menu: State<DockedDatePickerView>,
    first_day: Weekday,
    today: CalendarDate,
    change: CallbackWith<CalendarDate>,
    show_actions: bool,
    on_cancel: Callback,
    on_confirm: Callback,
) {
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
        .children(move || {
            boxed()
                .modifier(Modifier::new().fill_max_width().height(Dp(44.0)))
                .children(move || {
                    header(state, menu);
                });
            calendar(state, first_day, today, change);
            if show_actions {
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                row()
                    .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
                    .main_axis_alignment(MainAxisAlignment::End)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        button()
                            .text()
                            .accessibility_label("Cancel")
                            .on_click_shared(on_cancel)
                            .child(|| {
                                text().content("Cancel");
                            });
                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                        button()
                            .text()
                            .accessibility_label("OK")
                            .on_click_shared(on_confirm)
                            .child(|| {
                                text().content("OK");
                            });
                    });
            }
        });
}

#[tessera]
fn menu_header(state: State<DatePickerState>, menu: State<DockedDatePickerView>) {
    let current = state.with(DatePickerState::displayed_month);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    boxed()
        .modifier(Modifier::new().fill_max_width().height(Dp(64.0)))
        .children(move || {
            row()
                .modifier(Modifier::new().fill_max_size())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    for (view, label, description) in [
                        (
                            DockedDatePickerView::Month,
                            format_month_name(current.month())[..3].to_owned(),
                            "Choose month",
                        ),
                        (
                            DockedDatePickerView::Year,
                            current.year().to_string(),
                            "Choose year",
                        ),
                    ] {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().weight(1.0).fill_max_height())
                            .children(move || {
                                if menu.get() == view {
                                    menu_button(
                                        label.clone(),
                                        description.into(),
                                        if view == DockedDatePickerView::Month {
                                            Dp(64.0)
                                        } else {
                                            Dp(72.0)
                                        },
                                        true,
                                        Callback::new(move || {
                                            menu.set(DockedDatePickerView::Calendar)
                                        }),
                                    );
                                } else {
                                    text()
                                        .content(label.clone())
                                        .style(theme.typography.label_large)
                                        .color(
                                            theme
                                                .color_scheme
                                                .on_surface
                                                .with_alpha(MaterialAlpha::DISABLED_CONTENT),
                                        );
                                }
                            });
                    }
                });
            boxed()
                .modifier(
                    Modifier::new()
                        .align(Alignment::BottomStart)
                        .fill_max_width(),
                )
                .children(|| {
                    horizontal_divider();
                });
        });
}

#[tessera]
fn header(state: State<DatePickerState>, menu: State<DockedDatePickerView>) {
    let month = state.with(DatePickerState::displayed_month);
    let years = state.with(|state| state.year_range().clone());
    row()
        .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .children(move || {
            arrow(
                "Previous month".into(),
                false,
                month.month() > 1 || month.year() > *years.start(),
                Callback::new(move || state.with_mut(DatePickerState::previous_month)),
            );
            menu_button(
                format_month_name(month.month())[..3].to_owned(),
                "Choose month".into(),
                Dp(64.0),
                menu.get() == DockedDatePickerView::Month,
                Callback::new(move || {
                    menu.set(if menu.get() == DockedDatePickerView::Month {
                        DockedDatePickerView::Calendar
                    } else {
                        DockedDatePickerView::Month
                    })
                }),
            );
            arrow(
                "Next month".into(),
                true,
                month.month() < 12 || month.year() < *years.end(),
                Callback::new(move || state.with_mut(DatePickerState::next_month)),
            );
            arrow(
                "Previous year".into(),
                false,
                menu.get() != DockedDatePickerView::Month && month.year() > *years.start(),
                Callback::new(move || {
                    state.with_mut(|state| {
                        let current = state.displayed_month();
                        state.set_displayed_month(
                            YearMonth::new(current.year().saturating_sub(1), current.month())
                                .unwrap(),
                        );
                    })
                }),
            );
            menu_button(
                month.year().to_string(),
                "Choose year".into(),
                Dp(72.0),
                menu.get() == DockedDatePickerView::Year,
                Callback::new(move || {
                    menu.set(if menu.get() == DockedDatePickerView::Year {
                        DockedDatePickerView::Calendar
                    } else {
                        DockedDatePickerView::Year
                    })
                }),
            );
            arrow(
                "Next year".into(),
                true,
                menu.get() != DockedDatePickerView::Month && month.year() < *years.end(),
                Callback::new(move || {
                    state.with_mut(|state| {
                        let current = state.displayed_month();
                        state.set_displayed_month(
                            YearMonth::new(current.year().saturating_add(1), current.month())
                                .unwrap(),
                        );
                    })
                }),
            );
        });
}

#[tessera]
fn arrow(label: String, next: bool, enabled: bool, action: Callback) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
        .children(move || {
            icon_button()
                .standard()
                .icon(if next {
                    filled::CHEVRON_RIGHT_SVG
                } else {
                    filled::CHEVRON_LEFT_SVG
                })
                .enabled(enabled)
                .accessibility_label(label.clone())
                .on_click_shared(action);
        });
}

#[tessera]
fn menu_button(label: String, description: String, width: Dp, expanded: bool, action: Callback) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    surface()
        .modifier(
            Modifier::new()
                .width(width)
                .height(Dp(48.0))
                .semantics(SemanticsArgs {
                    expanded: Some(expanded),
                    ..Default::default()
                }),
        )
        .shape(Shape::rounded_rectangle(Dp(8.0)))
        .style(tessera_ui::Color::TRANSPARENT.into())
        .on_click_shared(action)
        .accessibility_label(description)
        .child(move || {
            let label = label.clone();
            row()
                .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    text()
                        .content(label.clone())
                        .style(theme.typography.label_large)
                        .color(theme.color_scheme.on_surface);
                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                    icon()
                        .vector(filled::ARROW_DROP_DOWN_SVG)
                        .size(Dp(18.0))
                        .tint(theme.color_scheme.on_surface_variant);
                });
        });
}

fn cells(month: YearMonth, first: Weekday) -> [CalendarDate; 42] {
    let date = month.to_date(1).unwrap();
    let offset =
        (weekday_for_date(date).index_from_monday() - first.index_from_monday()).rem_euclid(7);
    let origin = days_from_civil(month.year(), month.month(), 1) - i64::from(offset);
    std::array::from_fn(|index| {
        let (year, month, day) = civil_from_days(origin + index as i64);
        CalendarDate::new(year, month, day).unwrap()
    })
}

#[tessera]
fn calendar(
    state: State<DatePickerState>,
    first: Weekday,
    today: CalendarDate,
    change: CallbackWith<CalendarDate>,
) {
    let snapshot = state.with(DatePickerState::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let owner = CalendarOwner::Single(state);
    let initial_date = owner.initial_date(today);
    let focus = remember(move || CalendarFocus::new(initial_date));
    let keyboard = CalendarKeyboard {
        focus,
        owner,
        first_day: first,
    };
    keyboard.sync(today);
    column()
        .modifier(Modifier::new().fill_max_width().height(Dp(336.0)))
        .children(move || {
            layout()
                .policy(DateGridLayout {
                    columns: DATE_COLUMNS,
                    row_height: DATE_CELL_SIZE,
                })
                .child(move || {
                    for weekday in weekday_sequence(first) {
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE))
                            .children(move || {
                                text()
                                    .content(weekday_narrow_label(weekday))
                                    .style(theme.typography.body_large)
                                    .color(theme.color_scheme.on_surface);
                            });
                    }
                });
            layout()
                .policy(DateGridLayout {
                    columns: DATE_COLUMNS,
                    row_height: DATE_CELL_SIZE,
                })
                .modifier(
                    Modifier::new()
                        .focus_restorer(Some(keyboard.focus_requester()))
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(super::format_month_year(snapshot.displayed_month)),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .child(move || {
                    for date in cells(snapshot.displayed_month, first) {
                        day(state, date, today, keyboard, change);
                    }
                });
        });
}

#[tessera]
fn day(
    state: State<DatePickerState>,
    date: CalendarDate,
    today: CalendarDate,
    keyboard: CalendarKeyboard,
    change: CallbackWith<CalendarDate>,
) {
    let snapshot = state.with(DatePickerState::snapshot);
    calendar_day(
        CallbackWith::new(move |date| {
            if state.with_mut(|state| state.set_selected_date(date)) {
                change.call(date);
            }
        }),
        today,
    )
    .cell(date)
    .selected(snapshot.selected_date == Some(date))
    .outside_month(
        date.month() != snapshot.displayed_month.month()
            || date.year() != snapshot.displayed_month.year(),
    )
    .enabled(is_date_selectable(
        date,
        &snapshot.year_range,
        snapshot.is_selectable_date.as_ref(),
        snapshot.is_selectable_year.as_ref(),
    ))
    .keyboard(keyboard);
}

#[tessera]
fn month_menu(state: State<DatePickerState>, menu: State<DockedDatePickerView>) {
    let current = state.with(DatePickerState::displayed_month);
    let scroll = remember(move || {
        let mut scroll = ScrollableController::new();
        let index = i32::from(current.month()).saturating_sub(4).clamp(0, 5);
        scroll.set_scroll_position(PxPosition::new(
            Px::ZERO,
            -Dp(f64::from(index) * 48.0).to_px(),
        ));
        scroll
    });
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(336.0))
                .vertical_scroll(scroll)
                .semantics(SemanticsArgs {
                    role: Some(Role::ListBox),
                    label: Some("Choose month".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            for month in 1..=12 {
                month_item(state, menu, month);
            }
        });
}

#[tessera]
fn month_item(state: State<DatePickerState>, menu: State<DockedDatePickerView>, month: u8) {
    let current = state.with(DatePickerState::displayed_month);
    let year = current.year();
    let enabled = (1..=super::days_in_month(year, month)).any(|day| {
        state.with(|state| {
            is_date_selectable(
                CalendarDate::new(year, month, day).unwrap(),
                state.year_range(),
                state.is_selectable_date().as_ref(),
                state.is_selectable_year().as_ref(),
            )
        })
    });
    choice(
        format_month_name(month).into(),
        current.month() == month,
        enabled,
        Callback::new(move || {
            state.with_mut(|state| {
                state.set_displayed_month(
                    YearMonth::new(state.displayed_month().year(), month).unwrap(),
                )
            });
            menu.set(DockedDatePickerView::Calendar);
        }),
    );
}

#[tessera]
fn year_menu(state: State<DatePickerState>, menu: State<DockedDatePickerView>) {
    let current = state.with(DatePickerState::displayed_month);
    let years = state.with(|state| state.year_range().clone());
    let first = i64::from(*years.start());
    let count = (i64::from(*years.end()) - first + 1).max(0) as usize;
    let scroll = remember(move || {
        let mut scroll = ScrollableController::new();
        let index = (i64::from(current.year()) - first - 3).max(0);
        scroll.set_scroll_position(PxPosition::new(Px::ZERO, -Dp(index as f64 * 48.0).to_px()));
        scroll
    });
    let top = scroll.with(|scroll| -scroll.child_position().y);
    let start =
        ((top / Dp(48.0).to_pixels_f32()).floor().max(0.0) as usize).min(count.saturating_sub(1));
    let end = (start + 9).min(count);
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(336.0))
                .vertical_scroll(scroll)
                .semantics(SemanticsArgs {
                    role: Some(Role::ListBox),
                    label: Some("Choose year".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            spacer().modifier(Modifier::new().height(Dp(start as f64 * 48.0)));
            for index in start..end {
                year_item(state, menu, (first + index as i64) as i32);
            }
            spacer().modifier(Modifier::new().height(Dp((count - end) as f64 * 48.0)));
        });
}

#[tessera]
fn year_item(state: State<DatePickerState>, menu: State<DockedDatePickerView>, year: i32) {
    let current = state.with(DatePickerState::displayed_month);
    let enabled = state.with(|state| {
        state
            .is_selectable_year()
            .is_none_or(|predicate| predicate.call(year))
    });
    choice(
        year.to_string(),
        current.year() == year,
        enabled,
        Callback::new(move || {
            state.with_mut(|state| {
                state.set_displayed_month(
                    YearMonth::new(year, state.displayed_month().month()).unwrap(),
                )
            });
            menu.set(DockedDatePickerView::Calendar);
        }),
    );
}

#[tessera]
fn choice(label: String, selected: bool, enabled: bool, action: Callback) {
    let theme = use_context::<MaterialTheme>().unwrap().get();
    surface()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(48.0))
                .semantics(SemanticsArgs {
                    selected: Some(selected),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .style(
            if selected {
                theme.color_scheme.surface_container_highest
            } else {
                tessera_ui::Color::TRANSPARENT
            }
            .into(),
        )
        .enabled(enabled)
        .on_click_shared(action)
        .accessibility_role(Role::ListBoxOption)
        .accessibility_label(label.clone())
        .content_alignment(Alignment::CenterStart)
        .child(move || {
            let label = label.clone();
            row()
                .modifier(Modifier::new().padding_symmetric(Dp(16.0), Dp::ZERO))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    if selected {
                        icon()
                            .vector(filled::CHECK_SVG)
                            .size(Dp(24.0))
                            .tint(theme.color_scheme.on_surface);
                    } else {
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                    }
                    spacer().modifier(Modifier::new().width(Dp(16.0)));
                    text()
                        .content(label.clone())
                        .style(theme.typography.body_large)
                        .color(theme.color_scheme.on_surface);
                });
        });
}

#[cfg(test)]
#[path = "docked_tests.rs"]
mod tests;
