//! Calendar and manual input for inclusive date ranges.
//!
//! ## Usage
//!
//! Select a booking period, reporting interval, or multi-day event.

use std::ops::RangeInclusive;

use tessera_ui::{
    AxisConstraint, CallbackWith, ComputedData, Constraint, Dp, FocusDirection, FocusManager,
    LayoutResult, MeasurementError, Modifier, Px, PxPosition, RenderSlotWith, State,
    layout::{MeasureScope, layout},
    remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    column::column,
    divider::horizontal_divider,
    icon_button::icon_button,
    lazy_list::{LazyListController, lazy_column},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs, with_keyboard_input},
    row::row,
    scrollable::ScrollBarBehavior,
    spacer::spacer,
    surface::surface,
    text::text,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::MaterialTheme,
};

use super::{
    CalendarDate, DATE_CELL_SIZE, DATE_COLUMNS, DATE_PICKER_MAX_WIDTH, DateGridLayout,
    DatePickerDefaults, DatePickerDisplayMode, DatePickerState, Weekday, YearMonth,
    build_month_grid, calendar_day, format_date_input, format_month_short_name, format_month_year,
    parse_date_input, weekday_narrow_label, weekday_sequence,
};

const MONTH_HEIGHT: Dp = Dp(336.0);

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;

/// Selection, date restrictions, visible month, and input mode for a date
/// range.
///
/// A range includes both endpoints. Date restrictions apply to its endpoints;
/// unavailable dates between them do not split the selected range.
pub struct DateRangePickerState {
    calendar: DatePickerState,
    end: Option<CalendarDate>,
    selection_revision: u64,
    displayed_revision: u64,
}

impl DateRangePickerState {
    /// Creates a range picker. Invalid initial endpoints are discarded.
    pub fn new(
        start: Option<CalendarDate>,
        end: Option<CalendarDate>,
        displayed_month: Option<YearMonth>,
        year_range: RangeInclusive<i32>,
        is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
        is_selectable_year: Option<CallbackWith<i32, bool>>,
        display_mode: DatePickerDisplayMode,
    ) -> Self {
        let calendar = DatePickerState::new(
            start,
            displayed_month,
            year_range,
            is_selectable_date,
            is_selectable_year,
            display_mode,
        );
        let mut state = Self {
            calendar,
            end: None,
            selection_revision: 0,
            displayed_revision: 0,
        };
        if let Some(end) = end {
            state.set_selection(state.selected_start_date(), Some(end));
        }
        state
    }

    /// Returns the inclusive start date, including an unfinished selection.
    pub fn selected_start_date(&self) -> Option<CalendarDate> {
        self.calendar.selected_date()
    }

    /// Returns the inclusive end date when the selection is complete.
    pub fn selected_end_date(&self) -> Option<CalendarDate> {
        self.end
    }

    /// Returns whether both endpoints are selected.
    pub fn is_complete(&self) -> bool {
        self.selected_start_date().is_some() && self.end.is_some()
    }

    /// Sets both endpoints atomically, returning false without changing an
    /// invalid range.
    ///
    /// An end requires a start on or before it. Both endpoints must be
    /// selectable.
    pub fn set_selection(
        &mut self,
        start: Option<CalendarDate>,
        end: Option<CalendarDate>,
    ) -> bool {
        if end.is_some_and(|end| start.is_none_or(|start| date_key(end) < date_key(start)))
            || start.is_some_and(|date| !self.is_date_selectable(date))
            || end.is_some_and(|date| !self.is_date_selectable(date))
        {
            return false;
        }
        self.calendar.selected_date = start;
        self.end = end;
        self.selection_revision = self.selection_revision.wrapping_add(1);
        true
    }

    /// Selects the next endpoint, restarting after a complete range or an
    /// earlier date.
    pub fn select_date(&mut self, date: CalendarDate) -> bool {
        if !self.is_date_selectable(date) {
            return false;
        }
        if self.end.is_some()
            || self
                .selected_start_date()
                .is_none_or(|start| date_key(date) < date_key(start))
        {
            self.set_selection(Some(date), None)
        } else {
            self.set_selection(self.selected_start_date(), Some(date))
        }
    }

    /// Clears both endpoints without moving the calendar.
    pub fn clear_selection(&mut self) {
        self.set_selection(None, None);
    }

    /// Returns whether the date can be used as an endpoint.
    pub fn is_date_selectable(&self, date: CalendarDate) -> bool {
        super::is_date_selectable(
            date,
            self.year_range(),
            self.calendar.is_selectable_date.as_ref(),
            self.calendar.is_selectable_year.as_ref(),
        )
    }

    /// Returns the allowed year range.
    pub fn year_range(&self) -> &RangeInclusive<i32> {
        self.calendar.year_range()
    }

    /// Updates the year range, clearing endpoints that are no longer
    /// selectable.
    pub fn set_year_range(&mut self, range: RangeInclusive<i32>) {
        self.calendar.set_year_range(range);
        self.clear_unselectable_end();
    }

    /// Updates endpoint date restrictions, clearing invalid selections.
    pub fn set_is_selectable_date(&mut self, predicate: Option<CallbackWith<CalendarDate, bool>>) {
        self.calendar.set_is_selectable_date(predicate);
        self.clear_unselectable_end();
    }

    /// Updates endpoint year restrictions, clearing invalid selections.
    pub fn set_is_selectable_year(&mut self, predicate: Option<CallbackWith<i32, bool>>) {
        self.calendar.set_is_selectable_year(predicate);
        self.clear_unselectable_end();
    }

    /// Returns the first visible calendar month.
    pub fn displayed_month(&self) -> YearMonth {
        self.calendar.displayed_month()
    }

    /// Scrolls to a month, clamped to the allowed year range.
    pub fn set_displayed_month(&mut self, month: YearMonth) {
        self.calendar.set_displayed_month(month);
        self.displayed_revision = self.displayed_revision.wrapping_add(1);
    }

    /// Returns the calendar or manual-input mode.
    pub fn display_mode(&self) -> DatePickerDisplayMode {
        self.calendar.display_mode()
    }

    /// Changes input mode, showing the selected start month when returning to
    /// the calendar.
    pub fn set_display_mode(&mut self, mode: DatePickerDisplayMode) {
        if self.display_mode() != mode {
            if let Some(start) = self.selected_start_date() {
                self.set_displayed_month(
                    YearMonth::new(start.year(), start.month()).expect("valid month"),
                );
            }
            self.calendar.set_display_mode(mode);
        }
    }

    /// Switches between calendar and manual input.
    pub fn toggle_display_mode(&mut self) {
        self.set_display_mode(if self.display_mode() == DatePickerDisplayMode::Picker {
            DatePickerDisplayMode::Input
        } else {
            DatePickerDisplayMode::Picker
        });
    }

    fn clear_unselectable_end(&mut self) {
        if self.selected_start_date().is_none()
            || self.end.is_some_and(|date| !self.is_date_selectable(date))
        {
            self.end = None;
        }
    }
}

impl Default for DateRangePickerState {
    fn default() -> Self {
        Self::new(
            None,
            None,
            None,
            DatePickerDefaults::YEAR_RANGE,
            None,
            None,
            DatePickerDisplayMode::Picker,
        )
    }
}

fn date_key(date: CalendarDate) -> (i32, u8, u8) {
    (date.year(), date.month(), date.day())
}

/// # date_range_picker
///
/// Select an inclusive date range for bookings, reports, and multi-day events.
///
/// ## Usage
///
/// Collect a start and end date with a scrolling calendar or two date inputs.
///
/// ## Parameters
///
/// - `state` — optional external selection, restrictions, visible month, and
///   mode.
/// - `modifier` — sizing and layout of the picker body.
/// - `first_day_of_week` — weekday at the start of each calendar row.
/// - `calendar_height` — scrollable month viewport height; defaults to 360dp.
/// - `show_mode_toggle` — whether people can switch calendar and input modes.
/// - `title` — optional title override.
/// - `headline` — optional selection-summary override.
///
/// ## Examples
///
/// ```
/// use tessera_components::date_picker::{CalendarDate, DateRangePickerState, date_range_picker};
/// use tessera_components::theme::material_theme;
/// use tessera_ui::{remember, tessera};
/// #[tessera]
/// fn demo() {
///     let state = remember(DateRangePickerState::default);
///     material_theme().child(move || {
///         date_range_picker().state(state);
///     });
/// }
/// let mut state = DateRangePickerState::default();
/// let day = CalendarDate::new(2026, 10, 17).unwrap();
/// assert!(state.select_date(day));
/// assert!(!state.is_complete());
/// assert!(state.select_date(day));
/// assert!(state.is_complete());
/// demo();
/// ```
#[tessera]
pub fn date_range_picker(
    state: Option<State<DateRangePickerState>>,
    modifier: Option<Modifier>,
    first_day_of_week: Option<Weekday>,
    calendar_height: Option<Dp>,
    show_mode_toggle: Option<bool>,
    #[prop(into)] title: Option<String>,
    #[prop(into)] headline: Option<String>,
) {
    let state = if let Some(state) = state {
        state
    } else {
        remember(DateRangePickerState::default)
    };
    let theme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get();
    let input = state.with(|state| state.display_mode() == DatePickerDisplayMode::Input);
    let title =
        title.unwrap_or_else(|| if input { "Enter dates" } else { "Select dates" }.to_owned());
    let headline = headline.unwrap_or_else(|| {
        state.with(|state| {
            let cross_year = state
                .selected_start_date()
                .zip(state.selected_end_date())
                .is_some_and(|(start, end)| start.year() != end.year());
            let format = |date: CalendarDate| {
                let value = format!("{} {}", format_month_short_name(date.month()), date.day());
                if cross_year {
                    format!("{value}, {}", date.year())
                } else {
                    value
                }
            };
            format!(
                "{} – {}",
                state
                    .selected_start_date()
                    .map(format)
                    .unwrap_or_else(|| "Start date".to_owned()),
                state
                    .selected_end_date()
                    .map(format)
                    .unwrap_or_else(|| "End date".to_owned())
            )
        })
    });
    column()
        .modifier(modifier.unwrap_or_default().width(DATE_PICKER_MAX_WIDTH))
        .children(move || {
            let title = title.clone();
            let headline = headline.clone();
            row()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .constrain(None, Some(AxisConstraint::new(Dp(68.0).to_px(), None)))
                        .padding(Padding::new(Dp(64.0), Dp::ZERO, Dp(12.0), Dp(12.0))),
                )
                .cross_axis_alignment(CrossAxisAlignment::End)
                .children(move || {
                    let title = title.clone();
                    let headline = headline.clone();
                    column()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            text()
                                .content(title.clone())
                                .style(theme.typography.label_large)
                                .color(theme.color_scheme.on_surface_variant);
                            spacer().modifier(Modifier::new().height(Dp(4.0)));
                            text()
                                .content(headline.clone())
                                .style(theme.typography.title_large)
                                .color(theme.color_scheme.on_surface_variant);
                        });
                    if show_mode_toggle.unwrap_or(true) {
                        icon_button()
                            .standard()
                            .icon(if input {
                                filled::CALENDAR_TODAY_SVG
                            } else {
                                filled::EDIT_SVG
                            })
                            .accessibility_label(if input {
                                "Switch to range calendar"
                            } else {
                                "Switch to date range input"
                            })
                            .on_click(move || {
                                state.with_mut(DateRangePickerState::toggle_display_mode)
                            });
                    }
                });
            horizontal_divider().color(theme.color_scheme.outline_variant);
            if input {
                range_input(state);
            } else {
                range_calendar(
                    state,
                    first_day_of_week.unwrap_or_default(),
                    calendar_height.unwrap_or(Dp(360.0)),
                );
            }
        });
}

#[tessera]
fn range_calendar(state: State<DateRangePickerState>, first_day: Weekday, height: Dp) {
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    let controller = remember(LazyListController::new);
    let first_year = state.with(|state| *state.year_range().start());
    let months = state
        .with(|state| ((*state.year_range().end() as i64 - first_year as i64 + 1) * 12) as usize);
    let displayed = state.with(DateRangePickerState::displayed_month);
    let revision = state.with(|state| state.displayed_revision);
    let month_height = MONTH_HEIGHT.to_px();
    let previous = remember(|| None);
    let scroll = controller.with(|controller| controller.scroll_controller().child_position());
    if previous.get() != Some((first_year, displayed, revision, month_height)) {
        let index = (displayed.year() - first_year) * 12 + displayed.month() as i32 - 1;
        controller.with_mut(|controller| {
            controller.scroll_to(Dp::ZERO, Dp(MONTH_HEIGHT.0 * index as f64))
        });
        previous.set(Some((first_year, displayed, revision, month_height)));
    } else {
        let index = (-scroll.y / MONTH_HEIGHT.to_px().to_f32().max(1.0)).max(0.0) as i32;
        let month = YearMonth::new(first_year, 1)
            .expect("January")
            .add_months(index);
        if month != displayed {
            state.with_mut(|state| state.calendar.set_displayed_month(month));
            previous.set(Some((first_year, month, revision, month_height)));
        }
    }
    let modifier = with_keyboard_input(
        Modifier::new()
            .fill_max_width()
            .padding_symmetric(Dp(12.0), Dp::ZERO),
        |mut input| {
            if input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
                || input.key_modifiers.shift_key()
            {
                return;
            }
            let direction = input.keyboard_events.iter().find_map(|event| {
                if event.state != tessera_ui::winit::event::ElementState::Pressed {
                    return None;
                }
                match event.logical_key {
                    tessera_ui::winit::keyboard::Key::Named(
                        tessera_ui::winit::keyboard::NamedKey::ArrowRight,
                    ) => Some(FocusDirection::Next),
                    tessera_ui::winit::keyboard::Key::Named(
                        tessera_ui::winit::keyboard::NamedKey::ArrowLeft,
                    ) => Some(FocusDirection::Previous),
                    _ => None,
                }
            });
            if let Some(direction) = direction
                && FocusManager::current().request_move_focus(direction)
            {
                input.block_keyboard();
            }
        },
    );
    column().modifier(modifier).children(move || {
        layout()
            .policy(DateGridLayout {
                columns: DATE_COLUMNS,
            })
            .child(move || {
                for day in weekday_sequence(first_day) {
                    surface()
                        .style(tessera_ui::Color::TRANSPARENT.into())
                        .modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE))
                        .content_alignment(Alignment::Center)
                        .child(move || {
                            text()
                                .content(weekday_narrow_label(day))
                                .style(theme.typography.body_large)
                                .color(theme.color_scheme.on_surface);
                        });
                }
            });
        lazy_column()
            .modifier(Modifier::new().fill_max_width().height(height))
            .estimated_item_size(MONTH_HEIGHT)
            .scrollbar_behavior(ScrollBarBehavior::Hidden)
            .controller(controller)
            .items_with_key_shared(
                months,
                move |index| state.with(|state| (*state.year_range().start(), index)),
                RenderSlotWith::new(move |index| {
                    let first_year = state.with(|state| *state.year_range().start());
                    let month = YearMonth::new(first_year, 1)
                        .expect("January")
                        .add_months(index as i32);
                    range_month(state, month, first_day);
                }),
            );
    });
}

#[tessera]
fn range_month(state: State<DateRangePickerState>, month: YearMonth, first_day: Weekday) {
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    let grid: [Option<CalendarDate>; DATE_COLUMNS * super::DATE_ROWS] =
        build_month_grid(month, first_day)
            .try_into()
            .expect("six calendar rows");
    let start = state.with(DateRangePickerState::selected_start_date);
    let end = state.with(DateRangePickerState::selected_end_date);
    let today = CalendarDate::today();
    let on_select = CallbackWith::new(move |date| {
        state.with_mut(|state| state.select_date(date));
    });
    column()
        .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
            label: Some(format_month_year(month)),
            role: Some(tessera_ui::accesskit::Role::Group),
            merge_descendants: true,
            ..Default::default()
        }))
        .children(move || {
            column()
                .modifier(Modifier::new().padding(Padding::new(
                    Dp(24.0),
                    Dp(20.0),
                    Dp::ZERO,
                    Dp(8.0),
                )))
                .children(move || {
                    text()
                        .content(format_month_year(month))
                        .style(theme.typography.title_small)
                        .color(theme.color_scheme.on_surface_variant);
                });
            layout()
                .policy(DateGridLayout {
                    columns: DATE_COLUMNS,
                })
                .child(move || {
                    for date in grid.iter().copied() {
                        let in_range =
                            date.zip(start)
                                .zip(end)
                                .is_some_and(|((date, start), end)| {
                                    date_key(date) >= date_key(start)
                                        && date_key(date) <= date_key(end)
                                });
                        let selected = date.is_some() && (date == start || date == end);
                        let enabled = date
                            .is_some_and(|date| state.with(|state| state.is_date_selectable(date)));
                        layout()
                            .policy(RangeCellLayout {
                                fill: in_range,
                                start: date == start,
                                end: date == end,
                            })
                            .child(move || {
                                if in_range {
                                    layout().modifier(
                                        Modifier::new()
                                            .background(theme.color_scheme.secondary_container),
                                    );
                                }
                                calendar_day(on_select, today)
                                    .cell_optional(date)
                                    .selected(selected)
                                    .enabled(enabled)
                                    .in_range(in_range)
                                    .description_optional(date.map(|_| {
                                        if date == start {
                                            "Start date"
                                        } else if date == end {
                                            "End date"
                                        } else if in_range {
                                            "In selected range"
                                        } else {
                                            "Not selected"
                                        }
                                        .to_owned()
                                    }));
                            });
                    }
                });
        });
}

#[derive(Clone, PartialEq)]
struct RangeCellLayout {
    fill: bool,
    start: bool,
    end: bool,
}

impl tessera_ui::LayoutModifierNode for RangeCellLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let width = scope
            .parent_constraint()
            .width()
            .clamp(DATE_CELL_SIZE.to_px());
        let height = scope
            .parent_constraint()
            .height()
            .clamp(DATE_CELL_SIZE.to_px());
        let mut result = LayoutResult::new(ComputedData { width, height });
        let children = scope.children();
        let visual = super::DATE_VISUAL_SIZE.to_px().min(height);
        let x = if self.start { width / 2 } else { Px::ZERO };
        let end = if self.end { width / 2 } else { width };
        let fill_width = if self.fill {
            (end - x).max(Px::ZERO)
        } else {
            Px::ZERO
        };
        if self.fill {
            children[0].measure(scope, &Constraint::new(fill_width, visual))?;
            result.place_child(children[0], PxPosition::new(x, (height - visual) / 2));
        }
        let day = *children.last().expect("calendar day");
        day.measure(scope, &Constraint::new(width, height))?;
        result.place_child(day, PxPosition::ZERO);
        Ok(result)
    }
}

#[tessera]
fn range_input(state: State<DateRangePickerState>) {
    let selected = selection(state);
    let controllers = [
        remember(|| input_controller(selected.1)),
        remember(|| input_controller(selected.2)),
    ];
    let draft = remember(|| {
        [
            selected.1.map(format_date_input).unwrap_or_default(),
            selected.2.map(format_date_input).unwrap_or_default(),
        ]
    });
    let last_selected = remember(|| selected);
    if last_selected.get() != selected {
        let values = [
            selected.1.map(format_date_input).unwrap_or_default(),
            selected.2.map(format_date_input).unwrap_or_default(),
        ];
        for (controller, value) in controllers.into_iter().zip(&values) {
            if controller.with(|controller| controller.text()) != *value {
                controller.with_mut(|controller| controller.set_text(value));
            }
        }
        draft.set(values);
        last_selected.set(selected);
    }
    row()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .children(move || {
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    range_input_field(state, true, controllers[0], draft, last_selected);
                });
            spacer().modifier(Modifier::new().width(Dp(16.0)));
            column()
                .modifier(Modifier::new().weight(1.0))
                .children(move || {
                    range_input_field(state, false, controllers[1], draft, last_selected);
                });
        });
}

fn selection(
    state: State<DateRangePickerState>,
) -> (u64, Option<CalendarDate>, Option<CalendarDate>) {
    state.with(|state| {
        (
            state.selection_revision,
            state.selected_start_date(),
            state.selected_end_date(),
        )
    })
}

fn input_controller(date: Option<CalendarDate>) -> TextInputController {
    let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
    controller.set_text(&date.map(format_date_input).unwrap_or_default());
    controller
}

fn apply_draft(state: &mut DateRangePickerState, values: &[String; 2]) {
    let start = parse_date_input(&values[0]).filter(|date| state.is_date_selectable(*date));
    let end = parse_date_input(&values[1]).filter(|date| {
        state.is_date_selectable(*date)
            && start.is_some_and(|start| date_key(start) <= date_key(*date))
    });
    state.set_selection(start, end);
}

#[tessera]
fn range_input_field(
    state: State<DateRangePickerState>,
    start: bool,
    controller: State<TextInputController>,
    draft: State<[String; 2]>,
    last_selected: State<(u64, Option<CalendarDate>, Option<CalendarDate>)>,
) {
    let value = controller.with(|controller| controller.text());
    let error = state.with(|state| input_error(&value, start, state));
    let theme = use_context::<MaterialTheme>().expect("theme").get();
    TextFieldBuilder::outlined()
        .modifier(Modifier::new().fill_max_width())
        .controller(controller)
        .background_color(theme.color_scheme.surface_container_high)
        .focus_background_color(theme.color_scheme.surface_container_high)
        .label(if start { "Start date" } else { "End date" })
        .placeholder("MM/DD/YYYY")
        .is_error(error.is_some())
        .on_change(move |value| {
            if value.len() > 10
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || byte == b'/')
            {
                return draft.with(|draft| draft[usize::from(!start)].clone());
            }
            draft.with_mut(|draft| draft[usize::from(!start)] = value.clone());
            state.with_mut(|state| {
                draft.with(|draft| apply_draft(state, draft));
            });
            last_selected.set(selection(state));
            value
        });
    if let Some(error) = error {
        text()
            .content(error)
            .style(theme.typography.body_small)
            .color(theme.color_scheme.error);
    }
}

fn input_error(value: &str, start: bool, state: &DateRangePickerState) -> Option<&'static str> {
    if value.is_empty() {
        return None;
    }
    let Some(date) = parse_date_input(value) else {
        return Some("Enter a date as MM/DD/YYYY");
    };
    if !state.is_date_selectable(date) {
        return Some("Date is not allowed");
    }
    if !start && state.selected_start_date().is_none() {
        return Some("Enter a start date first");
    }
    if !start
        && state
            .selected_start_date()
            .is_some_and(|start| date_key(date) < date_key(start))
    {
        return Some("End date must be on or after start date");
    }
    None
}
