//! Date picker components for selecting dates and inclusive ranges.
//!
//! ## Usage
//!
//! Choose calendar dates or booking intervals in scheduling and form flows.
use std::ops::RangeInclusive;

use tessera_foundation::modifier::{ModifierExt as _, Padding, SemanticsArgs};
use tessera_ui::{
    AxisConstraint, CallbackWith, Color, ComputedData, Constraint, Dp, LayoutResult,
    MeasurementError, Modifier, Px, PxPosition, RenderSlot, State, key,
    layout::MeasureScope,
    modifier::FocusModifierExt as _,
    provide_context, remember, tessera,
    time::{SystemTime, UNIX_EPOCH},
    use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    column::column,
    md3::{
        divider::horizontal_divider,
        icon::icon,
        icon_button::icon_button,
        material_icons::filled,
        scrollable::{ScrollBarBehavior, ScrollableController, scrollable},
        surface::surface,
        text::text,
        text_field::text_field,
        text_input::TextInputController,
        theme::{ContentColor, MaterialTheme, TextStyle},
    },
    res::I18N,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};

use navigation::{CalendarFocus, CalendarKeyboard, CalendarOwner};

pub use cell::{CalendarRangePosition, date_picker_day, date_picker_year};
pub use docked::{DockedDatePickerView, docked_date_picker};
pub use range::{DateRangePickerController, date_range_picker};

mod cell;
mod docked;
mod focus;
mod navigation;
mod range;

const DATE_COLUMNS: usize = 7;
const DATE_ROWS: usize = 6;
const DATE_CELL_SIZE: Dp = Dp(48.0);
const DATE_PICKER_MAX_WIDTH: Dp = Dp(360.0);
const DATE_VISUAL_SIZE: Dp = Dp(40.0);

/// Days of the week in Monday-first order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weekday {
    /// Monday.
    #[default]
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    fn index_from_monday(self) -> i32 {
        match self {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }

    fn from_monday_index(index: i32) -> Self {
        match index.rem_euclid(7) {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }
}

/// A calendar date expressed as year, month, and day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

impl CalendarDate {
    /// Creates a calendar date if the values are valid.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Returns the year.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month (1-12).
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Returns the day of the month (1-31).
    pub fn day(&self) -> u8 {
        self.day
    }

    /// Returns the current date in UTC.
    pub fn today() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let days = (duration.as_secs() / 86_400) as i64;
        let (year, month, day) = civil_from_days(days);
        CalendarDate::new(year, month, day)
            .unwrap_or_else(|| CalendarDate::new_unchecked(1970, 1, 1))
    }

    fn new_unchecked(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

/// A year and month pair used for month navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YearMonth {
    year: i32,
    month: u8,
}

impl YearMonth {
    /// Creates a year/month pair if the values are valid.
    pub fn new(year: i32, month: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        Some(Self { year, month })
    }

    /// Returns the year.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the month (1-12).
    pub fn month(&self) -> u8 {
        self.month
    }

    /// Returns the date for this month at the provided day.
    pub fn to_date(&self, day: u8) -> Option<CalendarDate> {
        CalendarDate::new(self.year, self.month, day)
    }

    /// Adds or subtracts months, adjusting the year as needed.
    pub fn add_months(&self, delta: i32) -> Self {
        let total = self.year * 12 + (self.month as i32 - 1) + delta;
        let year = total.div_euclid(12);
        let month = (total.rem_euclid(12) + 1) as u8;
        Self { year, month }
    }

    fn new_unchecked(year: i32, month: u8) -> Self {
        Self { year, month }
    }
}

/// Display modes supported by [`date_picker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatePickerDisplayMode {
    /// Calendar grid selection.
    #[default]
    Picker,
    /// Manual input with stepper controls.
    Input,
}

/// Calendar content displayed by a modal date picker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerView {
    /// Show weekdays and dates for the displayed month.
    #[default]
    Calendar,
    /// Show the scrollable year selection grid.
    Year,
}

/// Defaults for date picker behavior.
pub struct DatePickerDefaults;

impl DatePickerDefaults {
    /// Default selectable year range.
    pub const YEAR_RANGE: RangeInclusive<i32> = 1900..=2100;
}

/// Holds the current selection and display state for a date picker.
///
/// `is_selectable_date` and `is_selectable_year` are optional predicates.
/// `None` means every date or year is selectable. A date is selectable only
/// when it is inside the year range and both predicates allow it.
///
/// [`CallbackWith`] must be created during a component build, so these
/// setters have to be called from inside a `#[tessera]` component. Closures
/// passed to [`date_picker`] are turned into callbacks at that component
/// boundary.
pub struct DatePickerController {
    selected_date: Option<CalendarDate>,
    displayed_month: YearMonth,
    year_range: RangeInclusive<i32>,
    is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
    is_selectable_year: Option<CallbackWith<i32, bool>>,
    display_mode: DatePickerDisplayMode,
}

impl DatePickerController {
    /// Creates a date picker state.
    ///
    /// `is_selectable_date` and `is_selectable_year` are optional. `None`
    /// allows every date or year. When a callback is supplied, it must already
    /// have been created during a component build.
    pub fn new(
        initial_selected_date: Option<CalendarDate>,
        initial_displayed_month: Option<YearMonth>,
        year_range: RangeInclusive<i32>,
        is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
        is_selectable_year: Option<CallbackWith<i32, bool>>,
        display_mode: DatePickerDisplayMode,
    ) -> Self {
        let year_range = normalize_year_range(year_range);
        let selected_date = initial_selected_date.filter(|date| {
            is_date_selectable(
                *date,
                &year_range,
                is_selectable_date.as_ref(),
                is_selectable_year.as_ref(),
            )
        });

        let displayed_month = initial_displayed_month
            .filter(|month| year_range.contains(&month.year()))
            .or_else(|| selected_date.and_then(|date| YearMonth::new(date.year(), date.month())))
            .unwrap_or_else(|| fallback_displayed_month(&year_range));

        Self {
            selected_date,
            displayed_month,
            year_range,
            is_selectable_date,
            is_selectable_year,
            display_mode,
        }
    }

    /// Returns the selected date, if any.
    pub fn selected_date(&self) -> Option<CalendarDate> {
        self.selected_date
    }

    /// Returns the month currently displayed by the picker.
    pub fn displayed_month(&self) -> YearMonth {
        self.displayed_month
    }

    /// Returns the display mode.
    pub fn display_mode(&self) -> DatePickerDisplayMode {
        self.display_mode
    }

    /// Returns the year range allowed by this picker.
    pub fn year_range(&self) -> &RangeInclusive<i32> {
        &self.year_range
    }

    /// Returns the date predicate, if one is set.
    ///
    /// `None` means every date is selectable, subject to the year range and
    /// year predicate.
    pub fn is_selectable_date(&self) -> Option<CallbackWith<CalendarDate, bool>> {
        self.is_selectable_date
    }

    /// Returns the year predicate, if one is set.
    ///
    /// `None` means every year is selectable, subject to the year range.
    pub fn is_selectable_year(&self) -> Option<CallbackWith<i32, bool>> {
        self.is_selectable_year
    }

    /// Sets the selected date if it is allowed.
    pub fn set_selected_date(&mut self, date: CalendarDate) -> bool {
        if !is_date_selectable(
            date,
            &self.year_range,
            self.is_selectable_date.as_ref(),
            self.is_selectable_year.as_ref(),
        ) {
            return false;
        }
        self.selected_date = Some(date);
        if let Some(month) = YearMonth::new(date.year(), date.month()) {
            self.displayed_month = clamp_month_to_range(month, &self.year_range);
        }
        true
    }

    /// Clears the selected date.
    pub fn clear_selected_date(&mut self) {
        self.selected_date = None;
    }

    /// Updates the displayed month, clamped to the year range.
    pub fn set_displayed_month(&mut self, month: YearMonth) {
        self.displayed_month = clamp_month_to_range(month, &self.year_range);
    }

    /// Moves the displayed month forward by one, staying within the year range.
    pub fn next_month(&mut self) {
        if can_navigate_next(self.displayed_month, &self.year_range) {
            self.displayed_month = self.displayed_month.add_months(1);
        }
    }

    /// Moves the displayed month backward by one, staying within the year
    /// range.
    pub fn previous_month(&mut self) {
        if can_navigate_prev(self.displayed_month, &self.year_range) {
            self.displayed_month = self.displayed_month.add_months(-1);
        }
    }

    /// Updates the display mode.
    pub fn set_display_mode(&mut self, mode: DatePickerDisplayMode) {
        self.display_mode = mode;
    }

    /// Toggles between picker and input modes.
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DatePickerDisplayMode::Picker => DatePickerDisplayMode::Input,
            DatePickerDisplayMode::Input => DatePickerDisplayMode::Picker,
        };
    }

    /// Updates the allowed year range.
    pub fn set_year_range(&mut self, year_range: RangeInclusive<i32>) {
        self.year_range = normalize_year_range(year_range);
        self.displayed_month = clamp_month_to_range(self.displayed_month, &self.year_range);
        self.clear_unselectable_date();
    }

    /// Updates the date predicate.
    ///
    /// `None` means every date is selectable. A callback must be created during
    /// a component build, so call this setter from inside a `#[tessera]`
    /// component.
    pub fn set_is_selectable_date(
        &mut self,
        is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
    ) {
        self.is_selectable_date = is_selectable_date;
        self.clear_unselectable_date();
    }

    /// Updates the year predicate.
    ///
    /// `None` means every year is selectable. A callback must be created during
    /// a component build, so call this setter from inside a `#[tessera]`
    /// component.
    pub fn set_is_selectable_year(&mut self, is_selectable_year: Option<CallbackWith<i32, bool>>) {
        self.is_selectable_year = is_selectable_year;
        self.clear_unselectable_date();
    }

    fn clear_unselectable_date(&mut self) {
        if let Some(date) = self.selected_date
            && !is_date_selectable(
                date,
                &self.year_range,
                self.is_selectable_date.as_ref(),
                self.is_selectable_year.as_ref(),
            )
        {
            self.selected_date = None;
        }
    }

    fn snapshot(&self) -> DatePickerSnapshot {
        DatePickerSnapshot {
            selected_date: self.selected_date,
            displayed_month: self.displayed_month,
            year_range: self.year_range.clone(),
            is_selectable_date: self.is_selectable_date,
            is_selectable_year: self.is_selectable_year,
            display_mode: self.display_mode,
        }
    }
}

impl Default for DatePickerController {
    fn default() -> Self {
        DatePickerController::new(
            None,
            None,
            DatePickerDefaults::YEAR_RANGE,
            None,
            None,
            DatePickerDisplayMode::Picker,
        )
    }
}

#[derive(Clone, PartialEq)]
struct DatePickerSnapshot {
    selected_date: Option<CalendarDate>,
    displayed_month: YearMonth,
    year_range: RangeInclusive<i32>,
    is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
    is_selectable_year: Option<CallbackWith<i32, bool>>,
    display_mode: DatePickerDisplayMode,
}

#[derive(Clone, PartialEq)]
struct DatePickerConfig {
    modifier: Modifier,
    today: CalendarDate,
    first_day_of_week: Weekday,
    show_weekday_labels: bool,
    show_mode_toggle: bool,
    title: Option<String>,
    headline: Option<String>,
    controller: Option<State<DatePickerController>>,
    view: Option<State<DatePickerView>>,
    focus_requester: Option<tessera_ui::FocusRequester>,
}

/// # date_picker
///
/// Render a calendar date picker for selecting a single date.
///
/// ## Usage
///
/// Use when you need a calendar grid for picking a specific date.
///
/// ## Parameters
///
/// - `modifier` — modifier chain applied to the date picker.
/// - `initial_selected_date` — initial selected date for the internal state.
/// - `initial_displayed_month` — initial displayed month for the internal
///   state.
/// - `year_range` — year range allowed in the internal state.
/// - `is_selectable_date` — optional date predicate for the internal state.
///   `None` allows every date. A date is selectable only when the year range,
///   the year predicate, and this predicate all allow it.
/// - `is_selectable_year` — optional year predicate for the internal state.
///   `None` allows every year.
/// - `display_mode` — initial display mode for the internal state.
/// - `first_day_of_week` — first day of the week for the calendar grid.
/// - `today` — current date used for date and year outlines and initial
///   keyboard focus; defaults to the system date.
/// - `show_weekday_labels` — whether weekday labels are rendered.
/// - `show_mode_toggle` — whether the display mode toggle is shown.
/// - `title` — optional override for the title text.
/// - `headline` — optional override for the headline text.
/// - `controller` — optional external state for selection, month navigation, and
///   mode.
/// - `view` — optional shared calendar or year selection view.
/// - `focus_requester` — requests focus on the calendar's current keyboard date.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::date_picker::{CalendarDate, DatePickerController, date_picker};
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// date_picker().today(CalendarDate::new(2026, 7, 16).unwrap());
///
/// let mut state = DatePickerController::default();
/// assert!(state.selected_date().is_none());
/// #     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn date_picker(
    modifier: Option<Modifier>,
    initial_selected_date: Option<CalendarDate>,
    initial_displayed_month: Option<YearMonth>,
    year_range: Option<RangeInclusive<i32>>,
    is_selectable_date: Option<CallbackWith<CalendarDate, bool>>,
    is_selectable_year: Option<CallbackWith<i32, bool>>,
    display_mode: Option<DatePickerDisplayMode>,
    first_day_of_week: Option<Weekday>,
    today: Option<CalendarDate>,
    show_weekday_labels: Option<bool>,
    show_mode_toggle: Option<bool>,
    #[prop(into)] title: Option<String>,
    #[prop(into)] headline: Option<String>,
    controller: Option<State<DatePickerController>>,
    view: Option<State<DatePickerView>>,
    focus_requester: Option<tessera_ui::FocusRequester>,
) {
    let today = today.unwrap_or_else(CalendarDate::today);
    let modifier = modifier.unwrap_or_default();
    let display_mode = display_mode.unwrap_or_default();
    let first_day_of_week = first_day_of_week.unwrap_or_default();
    let show_weekday_labels = show_weekday_labels.unwrap_or(true);
    let show_mode_toggle = show_mode_toggle.unwrap_or(true);
    let year_range = year_range.unwrap_or(DatePickerDefaults::YEAR_RANGE);
    let state = if let Some(state) = controller {
        state
    } else {
        // Resolve predicates before remember borrows the slot table for
        // initialization.
        let initial = DatePickerController::new(
            initial_selected_date,
            initial_displayed_month.or_else(|| {
                initial_selected_date
                    .and_then(|date| YearMonth::new(date.year(), date.month()))
                    .or_else(|| Some(fallback_displayed_month_at(&year_range, today)))
            }),
            year_range,
            is_selectable_date,
            is_selectable_year,
            display_mode,
        );
        remember(|| initial)
    };
    date_picker_content(DatePickerConfig {
        modifier,
        today,
        first_day_of_week,
        show_weekday_labels,
        show_mode_toggle,
        title,
        headline,
        controller: Some(state),
        view,
        focus_requester,
    });
}

#[tessera]
fn date_picker_content(args: DatePickerConfig) {
    let state = args.controller.expect("date_picker_content requires state");
    let snapshot = state.with(DatePickerController::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    let title = args
        .title
        .unwrap_or_else(|| default_title(snapshot.display_mode));
    let headline = args.headline.unwrap_or_else(|| {
        if snapshot.display_mode == DatePickerDisplayMode::Input && snapshot.selected_date.is_none()
        {
            I18N.text("md3-input-date")
        } else {
            default_headline(snapshot.selected_date)
        }
    });
    column()
        .modifier(
            args.modifier
                .width(DATE_PICKER_MAX_WIDTH)
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Group),
                    label: Some(title.clone()),
                    merge_descendants: true,
                    test_tag: cfg!(test).then(|| "date-picker".to_owned()),
                    ..Default::default()
                }),
        )
        .children(move || {
            let title = title.clone();
            let headline = headline.clone();
            boxed().children(move || {
                let title = title.clone();
                let headline = headline.clone();
                column()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(120.0))
                            .padding(Padding::new(Dp(24.0), Dp(16.0), Dp(12.0), Dp(12.0))),
                    )
                    .children(move || {
                        text(title.clone())
                            .style(theme.typography.label_large)
                            .color(scheme.on_surface_variant);
                        spacer().modifier(Modifier::new().height(Dp(24.0)));
                        let headline = headline.clone();
                        row()
                            .modifier(Modifier::new().fill_max_width().height(Dp(48.0)))
                            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                text(headline.clone())
                                    .style(theme.typography.headline_large)
                                    .color(scheme.on_surface);
                                if args.show_mode_toggle {
                                    date_mode_toggle(state);
                                }
                            });
                    });
                column()
                    .modifier(Modifier::new().fill_max_width().height(Dp(120.0)))
                    .main_axis_alignment(MainAxisAlignment::End)
                    .children(move || {
                        horizontal_divider().color(scheme.outline_variant);
                    });
            });
            match snapshot.display_mode {
                DatePickerDisplayMode::Picker => {
                    calendar_view(
                        state,
                        args.first_day_of_week,
                        args.show_weekday_labels,
                        args.today,
                    )
                    .view_optional(args.view)
                    .focus_requester_optional(args.focus_requester);
                }
                DatePickerDisplayMode::Input => {
                    date_input_view(state);
                }
            }
        });
}

/// # date_picker_dialog
///
/// Render a date picker dialog body with optional action buttons.
///
/// ## Usage
///
/// Use inside `dialog_provider` with `.padding(Dp(0.0))` for modal date
/// selection.
///
/// ## Parameters
///
/// - `dialog` — registration scope received from the provider content slot.
/// - `controller` — state handle used by the embedded date picker.
/// - `title` — optional override for the dialog title.
/// - `confirm_button` — optional confirm button content.
/// - `dismiss_button` — optional dismiss button content.
/// - `picker_modifier` — modifier chain applied to the embedded picker.
/// - `picker_first_day_of_week` — first day of the week for the embedded
///   picker.
/// - `picker_today` — current date used for outlines and initial keyboard
///   focus; defaults to the system date.
/// - `picker_show_weekday_labels` — whether weekday labels are rendered.
/// - `picker_show_mode_toggle` — whether the display mode toggle is shown.
/// - `picker_title` — optional picker title override.
/// - `picker_headline` — optional picker headline override.
/// - `picker_view` — optional shared calendar or year selection view.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_components::md3::date_picker::{CalendarDate, DatePickerController, date_picker_dialog};
/// use tessera_ui::remember;
/// # use tessera_components::md3::theme::{MaterialTheme, material_theme};
///
/// # material_theme()
/// #     .theme(|| MaterialTheme::default())
/// #     .child(|| {
/// let state = remember(DatePickerController::default);
/// date_picker_dialog()
///     .controller(state)
///     .picker_today(CalendarDate::new(2026, 7, 16).unwrap());
/// #     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn date_picker_dialog(
    dialog: Option<crate::md3::dialog::DialogScope>,
    controller: Option<State<DatePickerController>>,
    #[prop(into)] title: Option<String>,
    confirm_button: Option<RenderSlot>,
    dismiss_button: Option<RenderSlot>,
    picker_modifier: Option<Modifier>,
    picker_first_day_of_week: Option<Weekday>,
    picker_today: Option<CalendarDate>,
    picker_show_weekday_labels: Option<bool>,
    picker_show_mode_toggle: Option<bool>,
    #[prop(into)] picker_title: Option<String>,
    #[prop(into)] picker_headline: Option<String>,
    picker_view: Option<State<DatePickerView>>,
    picker_focus_requester: Option<tessera_ui::FocusRequester>,
) {
    let today = picker_today.unwrap_or_else(CalendarDate::today);
    let state = if let Some(state) = controller {
        state
    } else {
        remember(|| {
            DatePickerController::new(
                None,
                Some(fallback_displayed_month_at(
                    &DatePickerDefaults::YEAR_RANGE,
                    today,
                )),
                DatePickerDefaults::YEAR_RANGE,
                None,
                None,
                DatePickerDisplayMode::Picker,
            )
        })
    };
    let title = picker_title.or(title);
    crate::md3::dialog::register_dialog_accessibility()
        .dialog_optional(dialog)
        .label(
            title
                .clone()
                .unwrap_or_else(|| I18N.text("md3-select-date")),
        );
    let theme = use_context::<MaterialTheme>().unwrap().get();
    column()
        .modifier(Modifier::new().width(DATE_PICKER_MAX_WIDTH))
        .children(move || {
            date_picker_content(DatePickerConfig {
                modifier: picker_modifier.clone().unwrap_or_default(),
                today,
                first_day_of_week: picker_first_day_of_week.unwrap_or_default(),
                show_weekday_labels: picker_show_weekday_labels.unwrap_or(true),
                show_mode_toggle: picker_show_mode_toggle.unwrap_or(true),
                title: title.clone(),
                headline: picker_headline.clone(),
                controller: Some(state),
                view: picker_view,
                focus_requester: picker_focus_requester,
            });
            if confirm_button.is_some() || dismiss_button.is_some() {
                provide_context(
                    ContentColor {
                        current: theme.color_scheme.primary,
                    },
                    || {
                        row()
                            .modifier(
                                Modifier::new()
                                    .fill_max_width()
                                    .height(Dp(60.0))
                                    .padding(Padding::new(Dp(12.0), Dp(4.0), Dp(12.0), Dp(8.0))),
                            )
                            .main_axis_alignment(MainAxisAlignment::End)
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .children(move || {
                                if let Some(dismiss) = dismiss_button {
                                    dismiss.render();
                                }
                                if confirm_button.is_some() && dismiss_button.is_some() {
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                }
                                if let Some(confirm) = confirm_button {
                                    confirm.render();
                                }
                            });
                    },
                );
            }
        });
}

#[tessera]
fn weekday_label_row(first_day_of_week: Weekday, text_style: TextStyle, text_color: Color) {
    tessera_ui::layout::layout(DateGridLayout {
        columns: DATE_COLUMNS,
        row_height: DATE_CELL_SIZE,
    })
    .child(move || {
        for weekday in weekday_sequence(first_day_of_week) {
            surface()
                .style(Color::TRANSPARENT.into())
                .modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE))
                .content_alignment(Alignment::Center)
                .child(move || {
                    text(weekday_narrow_label(weekday))
                        .style(text_style)
                        .color(text_color);
                });
        }
    });
}

#[tessera]
fn calendar_view(
    state: State<DatePickerController>,
    first_day_of_week: Weekday,
    show_weekday_labels: bool,
    today: CalendarDate,
    view: Option<State<DatePickerView>>,
    focus_requester: Option<tessera_ui::FocusRequester>,
) {
    let snapshot = state.with(DatePickerController::snapshot);
    let view = if let Some(view) = view {
        view
    } else {
        remember(DatePickerView::default)
    };
    let can_prev = can_navigate_prev(snapshot.displayed_month, &snapshot.year_range);
    let can_next = can_navigate_next(snapshot.displayed_month, &snapshot.year_range);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let scheme = theme.color_scheme;
    column()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_symmetric(Dp(12.0), Dp(0.0)),
        )
        .children(move || {
            row()
                .modifier(Modifier::new().fill_max_width().height(Dp(56.0)))
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    surface()
                        .modifier(
                            Modifier::new()
                                .height(Dp(48.0))
                                .padding_symmetric(Dp(12.0), Dp(0.0)),
                        )
                        .shape(Shape::CAPSULE)
                        .style(Color::TRANSPARENT.into())
                        .content_alignment(Alignment::Center)
                        .on_click(move || {
                            view.with_mut(|value| {
                                *value = if *value == DatePickerView::Calendar {
                                    DatePickerView::Year
                                } else {
                                    DatePickerView::Calendar
                                }
                            })
                        })
                        .accessibility_label(I18N.text("md3-choose-year"))
                        .child(move || {
                            row()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    text(format_month_year(snapshot.displayed_month))
                                        .style(theme.typography.label_large)
                                        .color(scheme.on_surface_variant);
                                    spacer().modifier(Modifier::new().width(Dp(8.0)));
                                    icon()
                                        .painter(if view.get() == DatePickerView::Year {
                                            filled::ARROW_DROP_UP_SVG
                                        } else {
                                            filled::ARROW_DROP_DOWN_SVG
                                        })
                                        .tint(scheme.on_surface_variant);
                                });
                        });
                    if view.get() != DatePickerView::Year {
                        row().children(move || {
                            boxed()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .alignment(Alignment::Center)
                                .children(move || {
                                    icon_button()
                                        .standard()
                                        .icon(filled::CHEVRON_LEFT_SVG)
                                        .enabled(can_prev)
                                        .accessibility_label(I18N.text("md3-previous-month"))
                                        .on_click(move || {
                                            state.with_mut(DatePickerController::previous_month)
                                        });
                                });
                            boxed()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .alignment(Alignment::Center)
                                .children(move || {
                                    icon_button()
                                        .standard()
                                        .icon(filled::CHEVRON_RIGHT_SVG)
                                        .enabled(can_next)
                                        .accessibility_label(I18N.text("md3-next-month"))
                                        .on_click(move || {
                                            state.with_mut(DatePickerController::next_month)
                                        });
                                });
                        });
                    } else {
                        spacer().modifier(Modifier::new().size(Dp(96.0), Dp(48.0)));
                    }
                });
            if view.get() == DatePickerView::Year {
                date_year_grid(state, view, today);
            } else {
                if show_weekday_labels {
                    weekday_label_row(
                        first_day_of_week,
                        theme.typography.body_large,
                        scheme.on_surface,
                    );
                }
                date_grid(state, first_day_of_week, today)
                    .focus_requester_optional(focus_requester);
            }
        });
}

#[tessera]
fn date_grid(
    state: State<DatePickerController>,
    first_day_of_week: Weekday,
    today: CalendarDate,
    focus_requester: Option<tessera_ui::FocusRequester>,
) {
    let snapshot = state.with(DatePickerController::snapshot);
    let owner = CalendarOwner::Single(state);
    let initial = owner.initial_date(today);
    let focus = remember(move || CalendarFocus::new(initial, focus_requester));
    let keyboard = CalendarKeyboard {
        focus,
        owner,
        first_day: first_day_of_week,
    };
    keyboard.sync(today);
    let grid = build_month_grid(snapshot.displayed_month, first_day_of_week);
    let on_select = CallbackWith::new(move |date| {
        state.with_mut(|state| state.set_selected_date(date));
    });
    tessera_ui::layout::layout(DateGridLayout {
        columns: DATE_COLUMNS,
        row_height: DATE_CELL_SIZE,
    })
    .modifier(
        Modifier::new()
            .focus_restorer(None, Some(keyboard.focus_requester()))
            .semantics(SemanticsArgs {
                role: Some(tessera_ui::accesskit::Role::Group),
                label: Some(format_month_year(snapshot.displayed_month)),
                merge_descendants: true,
                test_tag: cfg!(test).then(|| "date-grid".to_owned()),
                ..Default::default()
            }),
    )
    .child(move || {
        for cell in grid.iter().copied() {
            let enabled = cell.is_some_and(|date| {
                is_date_selectable(
                    date,
                    &snapshot.year_range,
                    snapshot.is_selectable_date.as_ref(),
                    snapshot.is_selectable_year.as_ref(),
                )
            });
            calendar_day(on_select, today)
                .cell_optional(cell)
                .selected(cell.is_some() && snapshot.selected_date == cell)
                .enabled(enabled)
                .keyboard(keyboard);
        }
    });
}

#[derive(Clone, PartialEq)]
struct DateGridLayout {
    columns: usize,
    row_height: Dp,
}

impl tessera_ui::LayoutPolicy for DateGridLayout {
    fn measure(&self, input: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let children = input.children();
        let width = input.parent_constraint().width().clamp(Dp(336.0).to_px());
        let row_height = self.row_height.to_px();
        let height = row_height * children.len().div_ceil(self.columns) as i32;
        let mut result = LayoutResult::new(ComputedData { width, height });
        for (index, child) in children.into_iter().enumerate() {
            let column = index % self.columns;
            let x = Px(width.0 * column as i32 / self.columns as i32);
            let end = Px(width.0 * (column + 1) as i32 / self.columns as i32);
            child.measure(
                input,
                &Constraint::new(
                    AxisConstraint::exact(end - x),
                    AxisConstraint::exact(row_height),
                ),
            )?;
            result.place_child(
                child,
                PxPosition::new(x, row_height * (index / self.columns) as i32),
            );
        }
        Ok(result)
    }
}

#[tessera]
fn calendar_day(
    cell: Option<CalendarDate>,
    selected: Option<bool>,
    enabled: Option<bool>,
    in_range: Option<bool>,
    outside_month: Option<bool>,
    #[prop(into)] description: Option<String>,
    on_select: CallbackWith<CalendarDate>,
    today: CalendarDate,
    keyboard: Option<CalendarKeyboard>,
    range_band: Option<CalendarRangeBand>,
) {
    if let Some(date) = cell {
        let modifier = keyboard.map_or_else(Modifier::new, |keyboard| {
            keyboard.day_modifier(Modifier::new(), date)
        });
        let range_position = range_band.map(|band| match (band.start, band.end) {
            (true, true) => CalendarRangePosition::Isolated,
            (true, false) => CalendarRangePosition::Start,
            (false, false) => CalendarRangePosition::Middle,
            (false, true) => CalendarRangePosition::End,
        });
        date_picker_day(date)
            .on_select_shared(on_select)
            .selected_optional(selected)
            .enabled_optional(enabled)
            .in_range_optional(in_range)
            .outside_month_optional(outside_month)
            .is_today(date == today)
            .modifier(modifier)
            .focus_requester_optional(keyboard.and_then(|keyboard| keyboard.requester(date)))
            .range_position_optional(range_position)
            .accessibility_description_optional(description);
    } else {
        spacer().modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE));
    }
}

#[derive(Clone, Copy, PartialEq)]
struct CalendarRangeBand {
    start: bool,
    end: bool,
}

#[tessera]
fn year_grid_block(
    state: State<DatePickerController>,
    view: State<DatePickerView>,
    today: CalendarDate,
    first: i32,
    start: usize,
    end: usize,
    count: usize,
) {
    let snapshot = state.with(DatePickerController::snapshot);
    tessera_ui::layout::layout(DateGridLayout {
        columns: 3,
        row_height: Dp(52.0),
    })
    .child(move || {
        for index in start * 3..(end * 3).min(count) {
            let year = (first as i64 + index as i64) as i32;
            let selected = snapshot.displayed_month.year() == year;
            let enabled = snapshot
                .is_selectable_year
                .as_ref()
                .is_none_or(|predicate| predicate.call(year));
            let is_today = today.year() == year;
            key(year, move || {
                date_picker_year(year)
                    .selected(selected)
                    .is_today(is_today)
                    .enabled(enabled)
                    .on_select(move |year: i32| {
                        state.with_mut(|s| {
                            s.set_displayed_month(YearMonth::new_unchecked(
                                year,
                                s.displayed_month.month(),
                            ))
                        });
                        view.set(DatePickerView::Calendar);
                    });
            });
        }
    });
}

#[tessera]
fn date_year_grid(
    state: State<DatePickerController>,
    view: State<DatePickerView>,
    today: CalendarDate,
) {
    let snapshot = state.with(DatePickerController::snapshot);
    let first = *snapshot.year_range.start();
    let count = (*snapshot.year_range.end() as i64 - first as i64 + 1) as usize;
    let rows = count.div_ceil(3);
    let scroll = remember(|| {
        let mut controller = ScrollableController::new();
        let row = ((snapshot.displayed_month.year() as i64 - first as i64) / 3)
            .saturating_sub(3)
            .max(0);
        controller.set_scroll_position(PxPosition::new(Px::ZERO, -Dp(row as f64 * 52.0).to_px()));
        controller
    });
    let offset = scroll.with(|c| -c.child_position().y);
    let start = (((offset - Dp(8.0).to_pixels_f32()).max(0.0) / Dp(52.0).to_pixels_f32())
        .floor()
        .max(0.0) as usize)
        .min(rows.saturating_sub(1));
    let end = (start + 8).min(rows);
    scrollable()
        .modifier(
            Modifier::new()
                .size(Dp(336.0), Dp(288.0))
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Group),
                    label: Some(I18N.text("md3-choose-year")),
                    merge_descendants: true,
                    test_tag: cfg!(test).then(|| "date-years".to_owned()),
                    ..Default::default()
                }),
        )
        .vertical(true)
        .horizontal(false)
        .scrollbar_behavior(ScrollBarBehavior::Hidden)
        .controller(scroll)
        .child(move || {
            column()
                .modifier(Modifier::new().width(Dp(336.0)))
                .children(move || {
                    spacer().modifier(Modifier::new().height(Dp(8.0 + start as f64 * 52.0)));
                    year_grid_block(state, view, today, first, start, end, count);
                    spacer().modifier(Modifier::new().height(Dp((rows - end) as f64 * 52.0)));
                });
        });
}

#[tessera]
fn date_input_view(state: State<DatePickerController>) {
    let snapshot = state.with(DatePickerController::snapshot);
    let theme = use_context::<MaterialTheme>().unwrap().get();
    let controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        if let Some(date) = snapshot.selected_date {
            controller.set_text(&format_date_input(date));
        }
        controller
    });
    let last_selected = remember(|| snapshot.selected_date);
    let previous = remember(|| {
        snapshot
            .selected_date
            .map(format_date_input)
            .unwrap_or_default()
    });
    if last_selected.get() != snapshot.selected_date {
        last_selected.set(snapshot.selected_date);
        let expected = snapshot
            .selected_date
            .map(format_date_input)
            .unwrap_or_default();
        if controller.with(|c| c.text()) != expected {
            controller.with_mut(|c| c.set_text(&expected));
            previous.set(expected);
        }
    }
    let value = controller.with(|c| c.text());
    let error = if value.is_empty() {
        None
    } else {
        match parse_date_input(&value) {
            None => Some(I18N.text("md3-enter-a-date-as-mm-dd-yyyy")),
            Some(date)
                if !is_date_selectable(
                    date,
                    &snapshot.year_range,
                    snapshot.is_selectable_date.as_ref(),
                    snapshot.is_selectable_year.as_ref(),
                ) =>
            {
                Some(I18N.text("md3-date-is-not-allowed"))
            }
            Some(_) => None,
        }
    };
    column()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
        .children(move || {
            text_field()
                .outlined()
                .modifier(Modifier::new().fill_max_width().height(Dp(56.0)))
                .background_color(theme.color_scheme.surface_container_high)
                .focus_background_color(theme.color_scheme.surface_container_high)
                .controller(controller)
                .label(I18N.text("md3-date"))
                .placeholder("MM/DD/YYYY")
                .is_error(error.is_some())
                .on_change(move |value| {
                    if value.len() > 10 || !value.bytes().all(|c| c.is_ascii_digit() || c == b'/') {
                        return previous.get();
                    }
                    state.with_mut(|s| {
                        s.clear_selected_date();
                        if let Some(date) = parse_date_input(&value) {
                            s.set_selected_date(date);
                        }
                    });
                    last_selected.set(state.with(DatePickerController::selected_date));
                    previous.set(value.clone());
                    value
                });
            if let Some(error) = &error {
                text(error.clone())
                    .style(theme.typography.body_small)
                    .color(theme.color_scheme.error);
            }
        });
}

#[tessera]
fn date_mode_toggle(state: State<DatePickerController>) {
    let input = state.with(|s| s.display_mode == DatePickerDisplayMode::Input);
    icon_button()
        .standard()
        .icon(if input {
            filled::CALENDAR_TODAY_SVG
        } else {
            filled::EDIT_SVG
        })
        .accessibility_label(if input {
            I18N.text("md3-switch-to-calendar")
        } else {
            I18N.text("md3-switch-to-date-input")
        })
        .on_click(move || state.with_mut(DatePickerController::toggle_display_mode));
}

fn format_date_input(date: CalendarDate) -> String {
    format!("{:02}/{:02}/{:04}", date.month(), date.day(), date.year())
}

fn parse_date_input(value: &str) -> Option<CalendarDate> {
    if value.len() != 10
        || !value.is_ascii()
        || value.as_bytes()[2] != b'/'
        || value.as_bytes()[5] != b'/'
    {
        return None;
    }
    CalendarDate::new(
        value[6..].parse().ok()?,
        value[..2].parse().ok()?,
        value[3..5].parse().ok()?,
    )
}

fn default_title(mode: DatePickerDisplayMode) -> String {
    I18N.text(match mode {
        DatePickerDisplayMode::Picker => "md3-select-date",
        DatePickerDisplayMode::Input => "md3-enter-date",
    })
}

fn default_headline(selected: Option<CalendarDate>) -> String {
    selected
        .map(format_selected_date)
        .unwrap_or_else(|| I18N.text("md3-select-date"))
}

fn format_selected_date(date: CalendarDate) -> String {
    I18N.format(
        "md3-date-headline",
        [
            (
                "weekday",
                weekday_short_label(weekday_for_date(date)).into(),
            ),
            ("month", format_month_short_name(date.month()).into()),
            ("day", date.day().into()),
        ],
    )
}

fn weekday_narrow_label(day: Weekday) -> String {
    I18N.text(match day {
        Weekday::Monday => "md3-weekday-mon-narrow",
        Weekday::Tuesday => "md3-weekday-tue-narrow",
        Weekday::Wednesday => "md3-weekday-wed-narrow",
        Weekday::Thursday => "md3-weekday-thu-narrow",
        Weekday::Friday => "md3-weekday-fri-narrow",
        Weekday::Saturday => "md3-weekday-sat-narrow",
        Weekday::Sunday => "md3-weekday-sun-narrow",
    })
}

fn format_month_year(month: YearMonth) -> String {
    I18N.format(
        "md3-month-year",
        [
            ("month", format_month_name(month.month()).into()),
            ("year", (month.year()).to_string().into()),
        ],
    )
}

fn format_month_name(month: u8) -> String {
    I18N.text(match month {
        1 => "md3-january",
        2 => "md3-february",
        3 => "md3-march",
        4 => "md3-april",
        5 => "md3-may",
        6 => "md3-june",
        7 => "md3-july",
        8 => "md3-august",
        9 => "md3-september",
        10 => "md3-october",
        11 => "md3-november",
        _ => "md3-december",
    })
}

fn format_month_short_name(month: u8) -> String {
    I18N.text(match month {
        1 => "md3-jan",
        2 => "md3-feb",
        3 => "md3-mar",
        4 => "md3-apr",
        5 => "md3-may",
        6 => "md3-jun",
        7 => "md3-jul",
        8 => "md3-aug",
        9 => "md3-sep",
        10 => "md3-oct",
        11 => "md3-nov",
        _ => "md3-dec",
    })
}

fn weekday_sequence(first_day_of_week: Weekday) -> [Weekday; DATE_COLUMNS] {
    let mut days = [Weekday::Monday; DATE_COLUMNS];
    let start = first_day_of_week.index_from_monday();
    for (idx, slot) in days.iter_mut().enumerate() {
        *slot = Weekday::from_monday_index(start + idx as i32);
    }
    days
}

fn weekday_short_label(day: Weekday) -> String {
    I18N.text(match day {
        Weekday::Monday => "md3-mon",
        Weekday::Tuesday => "md3-tue",
        Weekday::Wednesday => "md3-wed",
        Weekday::Thursday => "md3-thu",
        Weekday::Friday => "md3-fri",
        Weekday::Saturday => "md3-sat",
        Weekday::Sunday => "md3-sun",
    })
}

fn build_month_grid(month: YearMonth, first_day_of_week: Weekday) -> Vec<Option<CalendarDate>> {
    let mut cells = vec![None; DATE_COLUMNS * DATE_ROWS];
    let first_date = month.to_date(1);
    let first_date =
        first_date.unwrap_or_else(|| CalendarDate::new_unchecked(month.year(), month.month(), 1));
    let first_weekday = weekday_for_date(first_date);
    let offset = (first_weekday.index_from_monday() - first_day_of_week.index_from_monday())
        .rem_euclid(7) as usize;
    let max_day = days_in_month(month.year(), month.month());
    for day in 1..=max_day {
        let index = offset + day as usize - 1;
        if index < cells.len() {
            cells[index] = CalendarDate::new(month.year(), month.month(), day);
        }
    }
    cells
}

fn is_date_selectable(
    date: CalendarDate,
    year_range: &RangeInclusive<i32>,
    is_selectable_date: Option<&CallbackWith<CalendarDate, bool>>,
    is_selectable_year: Option<&CallbackWith<i32, bool>>,
) -> bool {
    let year_allowed = is_selectable_year
        .map(|predicate| predicate.call(date.year()))
        .unwrap_or(true);
    let date_allowed = is_selectable_date
        .map(|predicate| predicate.call(date))
        .unwrap_or(true);
    year_range.contains(&date.year()) && year_allowed && date_allowed
}

fn normalize_year_range(range: RangeInclusive<i32>) -> RangeInclusive<i32> {
    let start = *range.start();
    let end = *range.end();
    if start <= end { range } else { end..=start }
}

fn fallback_displayed_month(year_range: &RangeInclusive<i32>) -> YearMonth {
    fallback_displayed_month_at(year_range, CalendarDate::today())
}

fn fallback_displayed_month_at(year_range: &RangeInclusive<i32>, today: CalendarDate) -> YearMonth {
    let year = if year_range.contains(&today.year()) {
        today.year()
    } else {
        *year_range.start()
    };
    let month = if year_range.contains(&today.year()) {
        today.month()
    } else {
        1
    };
    YearMonth::new(year, month).unwrap_or_else(|| YearMonth::new_unchecked(year, 1))
}

fn clamp_month_to_range(month: YearMonth, year_range: &RangeInclusive<i32>) -> YearMonth {
    let start = *year_range.start();
    let end = *year_range.end();
    if month.year() < start {
        YearMonth::new_unchecked(start, 1)
    } else if month.year() > end {
        YearMonth::new_unchecked(end, 12)
    } else {
        month
    }
}

fn can_navigate_prev(month: YearMonth, year_range: &RangeInclusive<i32>) -> bool {
    let start = *year_range.start();
    month.year() > start || (month.year() == start && month.month() > 1)
}

fn can_navigate_next(month: YearMonth, year_range: &RangeInclusive<i32>) -> bool {
    let end = *year_range.end();
    month.year() < end || (month.year() == end && month.month() < 12)
}

fn weekday_for_date(date: CalendarDate) -> Weekday {
    let days = days_from_civil(date.year(), date.month(), date.day());
    let index = (days + 3).rem_euclid(7) as i32;
    Weekday::from_monday_index(index)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let mut y = i64::from(year);
    let m = i64::from(month);
    let d = i64::from(day);
    y -= if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u8, day as u8)
}

#[cfg(test)]
mod tests {
    use tessera_ui::testing::{text_matches, visible_text};

    use tessera_ui::{PressKeyEventType, PxPosition, State, remember, tessera, testing::UiTest};

    use crate::{
        column::column,
        md3::{button::button, text::text, theme::material_theme},
    };

    use super::{
        CalendarDate, DatePickerController, DatePickerView, Weekday, YearMonth, date_picker,
        date_picker_dialog, format_date_input, parse_date_input,
    };

    #[derive(Clone, Default, PartialEq)]
    struct Probe {
        state: tessera_ui::testing::TestProbe<Option<State<DatePickerController>>>,
        view: tessera_ui::testing::TestProbe<Option<State<DatePickerView>>>,
        today: tessera_ui::testing::TestProbe<Option<State<CalendarDate>>>,
        range: tessera_ui::testing::TestProbe<Option<State<super::DateRangePickerController>>>,
        action: tessera_ui::testing::TestProbe<u8>,
        focus: tessera_ui::testing::TestProbe<Option<tessera_ui::FocusRequester>>,
    }

    #[tessera]
    fn today_fixture(range: bool, dialog: bool, probe: Probe) {
        let today = remember(|| CalendarDate::new(2026, 7, 16).unwrap());
        let state = remember(|| {
            DatePickerController::new(
                None,
                YearMonth::new(2026, 7),
                2025..=2028,
                None,
                None,
                Default::default(),
            )
        });
        let range_state = remember(|| {
            super::DateRangePickerController::new(
                None,
                None,
                YearMonth::new(2026, 7),
                2025..=2028,
                None,
                None,
                Default::default(),
            )
        });
        let view = remember(DatePickerView::default);
        let focus = remember(tessera_ui::FocusRequester::new).get();
        probe.focus.set(Some(focus));
        probe.today.set(Some(today));
        probe.range.set(Some(range_state));
        probe.state.set(Some(state));
        probe.view.set(Some(view));
        material_theme()
            .theme(|| {
                let mut theme = crate::md3::theme::MaterialTheme::default();
                theme.color_scheme.primary = tessera_ui::Color::new(0.05, 0.15, 0.8, 1.0);
                theme
            })
            .child(move || {
                if range {
                    super::date_range_picker()
                        .controller(range_state)
                        .focus_requester(focus)
                        .today(today.get())
                        .first_day_of_week(Weekday::Sunday);
                } else if dialog {
                    date_picker_dialog()
                        .controller(state)
                        .picker_focus_requester(focus)
                        .picker_today(today.get())
                        .picker_view(view)
                        .picker_first_day_of_week(Weekday::Sunday);
                } else {
                    date_picker()
                        .controller(state)
                        .focus_requester(focus)
                        .today(today.get())
                        .view(view)
                        .first_day_of_week(Weekday::Sunday);
                }
            });
    }

    #[cfg(not(target_family = "wasm"))]
    fn assert_today_outline(
        renderer: &tessera_ui::testing::UiTest,
        label: &str,
        radius: f64,
        expected: bool,
    ) {
        let tree = renderer.accessibility_snapshot().unwrap();
        let node = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                text_matches(node.label(), label)
                    && node.role() == tessera_ui::accesskit::Role::Button
            })
            .unwrap_or_else(|| panic!("missing actionable {label}"));
        let rect = node.1.bounds().expect("date target bounds");
        let x = (rect.x0 + rect.x1) / 2.0;
        let y = (rect.y0 + rect.y1) / 2.0;
        let pixels = renderer.read_pixels();
        let width = renderer.physical_size().width as usize;
        let blue = ((y - radius - 1.0).floor() as usize..(y - radius + 4.0).ceil() as usize)
            .flat_map(|y| ((x - 8.0) as usize..(x + 8.0) as usize).map(move |x| (x, y)))
            .any(|(x, y)| {
                let offset = (y * width + x) * 4;
                let color = &pixels[offset..offset + 3];
                i32::from(color[2]) > i32::from(color[0]) + 60
                    && i32::from(color[2]) > i32::from(color[1]) + 60
            });
        assert_eq!(blue, expected, "current-date outline for {label}");
    }

    #[cfg(not(target_family = "wasm"))]
    fn check_today_override(range: bool, dialog: bool) {
        use tessera_ui::testing::UiTest;

        let observed = Probe::default();

        let mut renderer = UiTest::new()
            .viewport(400, 650)
            .rendered(crate::TesseraComponents)
            .content(|| today_fixture(range, dialog, observed.clone()));
        renderer.frames(8);
        assert_today_outline(&renderer, "July 16, 2026", 20.0, true);
        assert_today_outline(&renderer, "July 17, 2026", 20.0, false);
        observed.focus.get().unwrap().request_focus();
        renderer.frames(4);
        let tree = renderer.accessibility_snapshot().unwrap();
        assert!(text_matches(
            tree.nodes
                .iter()
                .find(|(id, _)| *id == tree.focus)
                .and_then(|(_, node)| node.label()),
            "July 16, 2026"
        ));
        tessera_ui::focus::clear_focus(true);
        observed
            .today
            .get()
            .unwrap()
            .set(CalendarDate::new(2026, 7, 17).unwrap());
        renderer.frames(4);
        assert_today_outline(&renderer, "July 16, 2026", 20.0, false);
        assert_today_outline(&renderer, "July 17, 2026", 20.0, true);
        if !range {
            observed.view.get().unwrap().set(DatePickerView::Year);
            observed
                .today
                .get()
                .unwrap()
                .set(CalendarDate::new(2027, 7, 17).unwrap());
            renderer.frames(4);
            assert_today_outline(&renderer, "2027", 18.0, true);
            assert_today_outline(&renderer, "2028", 18.0, false);
            observed
                .today
                .get()
                .unwrap()
                .set(CalendarDate::new(2028, 7, 17).unwrap());
            renderer.frames(4);
            assert_today_outline(&renderer, "2027", 18.0, false);
            assert_today_outline(&renderer, "2028", 18.0, true);
            observed.view.get().unwrap().set(DatePickerView::Calendar);
            observed
                .state
                .get()
                .unwrap()
                .with_mut(|state| state.set_displayed_month(YearMonth::new(2028, 7).unwrap()));
            renderer.frames(4);
            observed.focus.get().unwrap().request_focus();
            renderer.frames(4);
            let tree = renderer.accessibility_snapshot().unwrap();
            assert!(text_matches(
                tree.nodes
                    .iter()
                    .find(|(id, _)| *id == tree.focus)
                    .and_then(|(_, node)| node.label()),
                "July 17, 2028"
            ));
        } else {
            observed
                .today
                .get()
                .unwrap()
                .set(CalendarDate::new(2028, 7, 17).unwrap());
            observed
                .range
                .get()
                .unwrap()
                .with_mut(|state| state.set_displayed_month(YearMonth::new(2028, 7).unwrap()));
            renderer.frames(8);
            observed.focus.get().unwrap().request_focus();
            renderer.frames(4);
            let tree = renderer.accessibility_snapshot().unwrap();
            assert!(text_matches(
                tree.nodes
                    .iter()
                    .find(|(id, _)| *id == tree.focus)
                    .and_then(|(_, node)| node.label()),
                "July 17, 2028"
            ));
        }
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    #[cfg(not(target_family = "wasm"))]
    fn gpu_today_override_controls_single_and_dialog_outlines_and_keyboard_fallback() {
        crate::test_utils::run_isolated_test(
            "md3::date_picker::tests::gpu_today_override_controls_single_and_dialog_outlines_and_keyboard_fallback",
            || {
                check_today_override(false, false);
                check_today_override(false, true);
            },
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    #[cfg(not(target_family = "wasm"))]
    fn gpu_today_override_controls_range_outline_and_keyboard_fallback() {
        crate::test_utils::run_isolated_test(
            "md3::date_picker::tests::gpu_today_override_controls_range_outline_and_keyboard_fallback",
            || check_today_override(true, false),
        );
    }

    #[tessera]
    fn external_year_fixture(probe: Probe) {
        let state = remember(|| {
            DatePickerController::new(
                CalendarDate::new(2025, 8, 17),
                YearMonth::new(2025, 8),
                2021..=2100,
                None,
                None,
                Default::default(),
            )
        });
        let view = remember(|| DatePickerView::Year);
        probe.state.set(Some(state));
        probe.view.set(Some(view));
        let on_action = tessera_ui::CallbackWith::new(move |value| probe.action.set(value));
        material_theme().child(move || {
            date_picker_dialog()
                .controller(state)
                .picker_view(view)
                .confirm_button(move || {
                    button()
                        .text()
                        .accessibility_label("Confirm external view")
                        .on_click(move || on_action.call(1))
                        .child(|| {
                            text("OK");
                        });
                })
                .dismiss_button(move || {
                    button()
                        .text()
                        .accessibility_label("Cancel external view")
                        .on_click(move || on_action.call(2))
                        .child(|| {
                            text("Cancel");
                        });
                });
        });
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn gpu_external_year_view_tracks_header_toggle_selection_and_dialog_actions() {
        use tessera_ui::{accesskit::Action, testing::UiTest};
        fn activate(renderer: &mut UiTest, label: &str) {
            let target_node = renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .into_iter()
                .find(|(_, node)| {
                    text_matches(node.label(), label) && node.supports_action(Action::Click)
                })
                .unwrap_or_else(|| panic!("missing actionable {label}"))
                .0;
            assert!(renderer.accessibility_action(target_node, Action::Click));
            renderer.frames(4);
        }
        let observed = Probe::default();
        let mut renderer = UiTest::new()
            .viewport(400, 650)
            .rendered(crate::TesseraComponents)
            .content(|| external_year_fixture(observed.clone()));
        renderer.frames(4);
        let state = observed.state.get().unwrap();
        let view = observed.view.get().unwrap();
        let selected = CalendarDate::new(2025, 8, 17);
        let initial_tree = renderer.accessibility_snapshot().unwrap();
        assert!(
            initial_tree
                .nodes
                .iter()
                .any(|(_, node)| text_matches(node.label(), "2025")
                    && node.supports_action(Action::Click)),
            "initial Year view must expose its selected year as a named action: {:?}",
            initial_tree.nodes
        );
        let bounds = |label: &str| {
            initial_tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    text_matches(node.label(), label) && node.supports_action(Action::Click)
                })
                .unwrap()
                .1
                .bounds()
                .unwrap()
        };
        let first = bounds("2021");
        let second = bounds("2022");
        let next_row = bounds("2024");
        assert_eq!((first.width(), first.height()), (112.0, 52.0));
        assert_eq!(second.x0 - first.x0, 112.0);
        assert_eq!(next_row.y0 - first.y0, 52.0);
        let viewport = initial_tree
            .nodes
            .iter()
            .find(|(_, node)| {
                text_matches(node.label(), "Choose year")
                    && node.bounds().is_some_and(|rect| rect.height() == 288.0)
            })
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert_eq!(viewport.width(), 336.0);
        assert_eq!(first.y0 - viewport.y0, 8.0);
        let confirm = bounds("Confirm external view");
        assert_eq!(confirm.height(), 48.0);
        assert_eq!((confirm.y0 + confirm.y1) / 2.0 - 20.0 - viewport.y1, 8.0);
        let target_node = initial_tree
            .nodes
            .iter()
            .find(|(_, node)| {
                text_matches(node.label(), "2039")
                    && node.role() == tessera_ui::accesskit::Role::Button
                    && node.supports_action(Action::Focus)
            })
            .expect("virtualized overscan must retain the next focusable rows")
            .0;
        assert!(renderer.accessibility_action(target_node, Action::Focus));
        renderer.frames(4);
        let focused_tree = renderer.accessibility_snapshot().unwrap();
        let focused = focused_tree
            .nodes
            .iter()
            .find(|(id, _)| *id == focused_tree.focus)
            .unwrap();
        assert!(text_matches(focused.1.label(), "2039"));
        let revealed = focused.1.bounds().unwrap();
        assert!(revealed.y0 >= viewport.y0 && revealed.y1 <= viewport.y1);
        assert_eq!(state.with(DatePickerController::selected_date), selected);
        activate(&mut renderer, "Choose year");
        assert_eq!(view.get(), DatePickerView::Calendar);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| text_matches(node.label(), "August 17, 2025"))
        );
        view.set(DatePickerView::Year);
        renderer.frames(4);
        activate(&mut renderer, "2027");
        assert_eq!(view.get(), DatePickerView::Calendar);
        assert_eq!(
            state.with(DatePickerController::displayed_month),
            YearMonth::new(2027, 8).unwrap()
        );
        assert_eq!(state.with(DatePickerController::selected_date), selected);
        activate(&mut renderer, "Confirm external view");
        assert_eq!(observed.action.get(), 1);
        activate(&mut renderer, "Cancel external view");
        assert_eq!(observed.action.get(), 2);
        assert_eq!(state.with(DatePickerController::selected_date), selected);
    }

    #[test]
    fn date_input_validates_calendar_dates_and_requires_a_complete_format() {
        let leap_day = CalendarDate::new(2024, 2, 29).unwrap();
        assert_eq!(parse_date_input("02/29/2024"), Some(leap_day));
        assert_eq!(format_date_input(leap_day), "02/29/2024");
        for invalid in [
            "02/29/2023",
            "13/01/2024",
            "04/31/2024",
            "2/29/2024",
            "02/29/24",
            "02/２9/2024",
            "02-29-2024",
        ] {
            assert_eq!(parse_date_input(invalid), None, "{invalid}");
        }
    }

    #[tessera]
    fn calendar_fixture(probe: Probe, today: Option<CalendarDate>) {
        let state = remember(|| {
            DatePickerController::new(
                None,
                YearMonth::new(2026, 7),
                2020..=2030,
                None,
                None,
                Default::default(),
            )
        });
        probe.state.set(Some(state));
        let focus = remember(tessera_ui::FocusRequester::new).get();
        probe.focus.set(Some(focus));
        material_theme().child(move || {
            column().children(move || {
                date_picker()
                    .controller(state)
                    .today(today.unwrap_or_else(|| CalendarDate::new(2026, 7, 1).unwrap()))
                    .focus_requester(focus)
                    .first_day_of_week(Weekday::Sunday);
                button()
                    .text()
                    .accessibility_label("After single calendar")
                    .on_click(|| {})
                    .child(|| {
                        text("After single calendar");
                    });
            });
        });
    }

    #[cfg(not(target_family = "wasm"))]
    fn check_single_calendar_accessibility() {
        use tessera_ui::{
            FocusDirection, MOUSE_POINTER_ID, accesskit::Role, focus, testing::UiTest,
        };

        let observed = Probe::default();

        let mut renderer = UiTest::new()
            .viewport(600, 950)
            .scale_factor(1.5)
            .rendered(crate::TesseraComponents)
            .content(|| {
                calendar_fixture(observed.clone()).today(CalendarDate::new(2026, 7, 16).unwrap())
            });
        renderer.frames(8);
        let accessibility = renderer.accessibility_snapshot().expect("tree");
        assert_eq!(
            accessibility
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Button
                    && node.label().map(visible_text).is_some_and(|label| label
                        .starts_with("July ")
                        && label.ends_with(", 2026")))
                .count(),
            31,
            "the published calendar must expose every July date"
        );
        observed.focus.get().unwrap().request_focus();
        renderer.frames(8);
        let accessibility = renderer.accessibility_snapshot().expect("tree");
        assert!(text_matches(
            accessibility
                .nodes
                .iter()
                .find(|(id, _)| *id == accessibility.focus)
                .and_then(|(_, node)| node.label()),
            "July 16, 2026"
        ));
        assert!(focus::request_move_focus(FocusDirection::Next));
        renderer.frames(8);
        let accessibility = renderer.accessibility_snapshot().expect("tree");
        assert!(text_matches(
            accessibility
                .nodes
                .iter()
                .find(|(id, _)| *id == accessibility.focus)
                .and_then(|(_, node)| node.label()),
            "After single calendar"
        ));
        let state = observed.state.get().expect("single state");
        assert_eq!(state.with(DatePickerController::selected_date), None);
        for label in ["Choose year", "2027"] {
            let accessibility = renderer.accessibility_snapshot().expect("tree");
            let bounds = accessibility
                .nodes
                .iter()
                .find(|(_, node)| text_matches(node.label(), label) && node.bounds().is_some())
                .expect("published year target")
                .1
                .bounds()
                .unwrap();
            let point = PxPosition::from_f32_xy(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            );
            renderer.pointer_move(MOUSE_POINTER_ID, point);
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.frames(1);
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.frames(8);
        }
        assert_eq!(
            state.with(DatePickerController::displayed_month),
            YearMonth::new(2027, 7).unwrap()
        );
        assert_eq!(state.with(DatePickerController::selected_date), None);
        assert!(
            renderer
                .accessibility_snapshot()
                .expect("tree")
                .nodes
                .iter()
                .any(|(_, node)| text_matches(node.label(), "July 1, 2027"))
        );
        state.with_mut(|state| state.set_display_mode(super::DatePickerDisplayMode::Input));
        renderer.frames(8);
        assert_eq!(
            renderer
                .accessibility_snapshot()
                .expect("tree")
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::TextInput && node.bounds().is_some())
                .count(),
            1,
            "manual entry must publish its editable input"
        );
    }

    #[test]
    #[ignore = "requires a native GPU adapter"]
    #[cfg(not(target_family = "wasm"))]
    fn gpu_single_calendar_publishes_dates_focus_and_year_actions() {
        crate::test_utils::run_isolated_test(
            "md3::date_picker::tests::gpu_single_calendar_publishes_dates_focus_and_year_actions",
            check_single_calendar_accessibility,
        );
    }

    #[tessera]
    fn internal_predicate_fixture() {
        material_theme().child(|| {
            column().children(|| {
                date_picker()
                    .initial_selected_date(CalendarDate::new(2026, 7, 18).unwrap())
                    .initial_displayed_month(YearMonth::new(2026, 7).unwrap())
                    .year_range(2026..=2026)
                    .is_selectable_date(|date| date.day() % 2 == 0)
                    .is_selectable_year(|year| year == 2026);
            });
        });
    }

    #[test]
    fn internal_calendar_initializes_predicates_without_a_slot_borrow_conflict() {
        crate::test_utils::run_isolated_test(
            "md3::date_picker::tests::internal_calendar_initializes_predicates_without_a_slot_borrow_conflict",
            || {
                let mut session = UiTest::new()
                    .content(internal_predicate_fixture)
                    .viewport(400, 600);
                let snapshot = session.frame();
                snapshot.node("date-picker").size(360, 512);
                snapshot.node("date-grid").size(336, 288);
            },
        );
    }

    #[test]
    fn calendar_uses_48dp_targets_and_selects_the_date_under_the_pointer() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        tessera_ui::dp::set_scale_factor(1.0);
        let observed = Probe::default();
        let mut session = UiTest::new()
            .content(|| calendar_fixture(observed.clone()))
            .viewport(400, 600);
        let snapshot = session.frame();
        snapshot.node("date-picker").size(360, 512);
        snapshot.node("date-grid").size(336, 288);
        let bounds = snapshot.node("date-grid").bounds();
        let cursor = Some(PxPosition::from_f32_xy(
            bounds.x.to_f32() + 4.5 * 48.0,
            bounds.y.to_f32() + 2.5 * 48.0,
        ));
        let day = session
            .inspect_components()
            .labeled(&super::I18N.format(
                "md3-full-date",
                [
                    ("month", "July".into()),
                    ("day", 16.into()),
                    ("year", "2026".into()),
                ],
            ))
            .bounds()
            .unwrap();
        let point = cursor.unwrap();
        assert!(
            point.x >= day.x
                && point.x < day.x + day.width
                && point.y >= day.y
                && point.y < day.y + day.height,
            "grid-derived pointer {point:?} must land in July 16 target {day:?}"
        );
        session.click_at(point);
        assert_eq!(
            observed
                .state
                .get()
                .unwrap()
                .with(DatePickerController::selected_date),
            CalendarDate::new(2026, 7, 16)
        );
    }
    #[test]
    fn selecting_a_year_returns_to_the_calendar_without_selecting_a_date() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        tessera_ui::dp::set_scale_factor(1.0);
        let observed = Probe::default();
        let mut session = UiTest::new()
            .content(|| calendar_fixture(observed.clone()))
            .viewport(400, 600);
        session.frame();
        session.click_at(PxPosition::from_f32_xy(100.0, 148.0));
        session.frames(1).node("date-years").size(336, 288);
        session.click_at(PxPosition::from_f32_xy(68.0, 248.0));
        let state = observed.state.get().unwrap();
        assert_eq!(
            state.with(DatePickerController::displayed_month),
            YearMonth::new(2023, 7).unwrap()
        );
        assert_eq!(state.with(DatePickerController::selected_date), None);
    }
    #[test]
    fn narrow_calendars_preserve_seven_columns() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        tessera_ui::dp::set_scale_factor(1.0);
        let observed = Probe::default();
        let mut session = UiTest::new()
            .content(|| calendar_fixture(observed.clone()))
            .viewport(320, 600);
        let snapshot = session.frame();
        snapshot.node("date-grid").size(296, 288);
        let bounds = snapshot.node("date-grid").bounds();
        let cursor = Some(PxPosition::from_f32_xy(
            bounds.x.to_f32() + bounds.width.to_f32() * 4.5 / 7.0,
            bounds.y.to_f32() + 2.5 * 48.0,
        ));
        let day = session
            .inspect_components()
            .labeled(&super::I18N.format(
                "md3-full-date",
                [
                    ("month", "July".into()),
                    ("day", 16.into()),
                    ("year", "2026".into()),
                ],
            ))
            .bounds()
            .unwrap();
        let point = cursor.unwrap();
        assert!(
            point.x >= day.x
                && point.x < day.x + day.width
                && point.y >= day.y
                && point.y < day.y + day.height,
            "grid-derived pointer {point:?} must land in July 16 target {day:?}"
        );
        session.click_at(point);
        assert_eq!(
            observed
                .state
                .get()
                .unwrap()
                .with(DatePickerController::selected_date),
            CalendarDate::new(2026, 7, 16)
        );
    }
    #[test]
    fn edit_button_switches_calendar_to_input_mode() {
        let runtime = tessera_ui::RuntimeSession::new();
        let _scope = runtime.enter();
        tessera_ui::dp::set_scale_factor(1.0);
        let observed = Probe::default();
        let mut session = UiTest::new()
            .content(|| calendar_fixture(observed.clone()))
            .viewport(400, 600);
        session.frame();
        session.click_at(PxPosition::from_f32_xy(328.0, 84.0));
        session.frames(1);
        assert_eq!(
            observed
                .state
                .get()
                .unwrap()
                .with(DatePickerController::display_mode),
            super::DatePickerDisplayMode::Input
        );
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "date_picker/focus_tests.rs"]
mod focus_tests;
