//! Date picker components for selecting calendar dates.
//!
//! ## Usage
//!
//! Use to let users choose a calendar date in scheduling or form flows.
use std::{
    ops::RangeInclusive,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use derive_setters::Setters;
use tessera_ui::{
    Color, DimensionValue, Dp, Modifier, State, provide_context, remember, tessera, use_context,
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgs, column},
    flow_row::{FlowRowArgs, flow_row},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{ContentColor, MaterialAlpha, MaterialTheme},
};

const DATE_COLUMNS: usize = 7;
const DATE_ROWS: usize = 6;
const DATE_CELL_SIZE: Dp = Dp(40.0);
const DATE_CELL_RADIUS: Dp = Dp(20.0);
const DATE_GRID_SPACING: Dp = Dp(4.0);
const HEADER_VERTICAL_PADDING: Dp = Dp(12.0);
const HEADER_HORIZONTAL_PADDING: Dp = Dp(16.0);
const NAV_BUTTON_SIZE: Dp = Dp(28.0);
const INPUT_ROW_GAP: Dp = Dp(12.0);

/// Days of the week in Monday-first order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    /// Monday.
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

/// Controls which dates are selectable in the date picker.
pub trait SelectableDates: Send + Sync {
    /// Returns true when the date can be selected.
    fn is_selectable_date(&self, _date: CalendarDate) -> bool {
        true
    }

    /// Returns true when the year can be selected.
    fn is_selectable_year(&self, _year: i32) -> bool {
        true
    }
}

struct AllDates;

impl SelectableDates for AllDates {}

/// Defaults for date picker behavior.
pub struct DatePickerDefaults;

impl DatePickerDefaults {
    /// Default selectable year range.
    pub const YEAR_RANGE: RangeInclusive<i32> = 1900..=2100;

    /// Returns a selectable-dates policy that allows every date.
    pub fn all_dates() -> Arc<dyn SelectableDates> {
        Arc::new(AllDates)
    }
}

/// Holds the current selection and display state for a date picker.
pub struct DatePickerState {
    selected_date: Option<CalendarDate>,
    displayed_month: YearMonth,
    year_range: RangeInclusive<i32>,
    selectable_dates: Arc<dyn SelectableDates>,
    display_mode: DatePickerDisplayMode,
}

impl DatePickerState {
    /// Creates a date picker state.
    pub fn new(
        initial_selected_date: Option<CalendarDate>,
        initial_displayed_month: Option<YearMonth>,
        year_range: RangeInclusive<i32>,
        selectable_dates: Arc<dyn SelectableDates>,
        display_mode: DatePickerDisplayMode,
    ) -> Self {
        let year_range = normalize_year_range(year_range);
        let selectable_dates = selectable_dates;
        let selected_date = initial_selected_date
            .filter(|date| is_date_selectable(*date, &year_range, &selectable_dates));

        let displayed_month = initial_displayed_month
            .filter(|month| year_range.contains(&month.year()))
            .or_else(|| selected_date.and_then(|date| YearMonth::new(date.year(), date.month())))
            .unwrap_or_else(|| fallback_displayed_month(&year_range));

        Self {
            selected_date,
            displayed_month,
            year_range,
            selectable_dates,
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

    /// Returns the selectable-dates policy.
    pub fn selectable_dates(&self) -> &Arc<dyn SelectableDates> {
        &self.selectable_dates
    }

    /// Sets the selected date if it is allowed.
    pub fn set_selected_date(&mut self, date: CalendarDate) -> bool {
        if !is_date_selectable(date, &self.year_range, &self.selectable_dates) {
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
        if let Some(date) = self.selected_date
            && !is_date_selectable(date, &self.year_range, &self.selectable_dates)
        {
            self.selected_date = None;
        }
    }

    /// Updates the selectable-dates policy.
    pub fn set_selectable_dates(&mut self, selectable_dates: Arc<dyn SelectableDates>) {
        self.selectable_dates = selectable_dates;
        if let Some(date) = self.selected_date
            && !is_date_selectable(date, &self.year_range, &self.selectable_dates)
        {
            self.selected_date = None;
        }
    }

    fn snapshot(&self) -> DatePickerSnapshot {
        DatePickerSnapshot {
            selected_date: self.selected_date,
            displayed_month: self.displayed_month,
            year_range: self.year_range.clone(),
            selectable_dates: self.selectable_dates.clone(),
            display_mode: self.display_mode,
        }
    }
}

impl Default for DatePickerState {
    fn default() -> Self {
        DatePickerState::new(
            None,
            None,
            DatePickerDefaults::YEAR_RANGE,
            DatePickerDefaults::all_dates(),
            DatePickerDisplayMode::Picker,
        )
    }
}

#[derive(Clone)]
struct DatePickerSnapshot {
    selected_date: Option<CalendarDate>,
    displayed_month: YearMonth,
    year_range: RangeInclusive<i32>,
    selectable_dates: Arc<dyn SelectableDates>,
    display_mode: DatePickerDisplayMode,
}

/// Configuration options for [`date_picker`].
///
/// Initial-state fields are applied only when `date_picker` owns the state.
#[derive(Clone, Setters)]
pub struct DatePickerArgs {
    /// Optional modifier chain applied to the date picker.
    pub modifier: Modifier,
    /// Initial selected date for the internal state.
    #[setters(strip_option)]
    pub initial_selected_date: Option<CalendarDate>,
    /// Initial displayed month for the internal state.
    #[setters(strip_option)]
    pub initial_displayed_month: Option<YearMonth>,
    /// Year range allowed in the internal state.
    pub year_range: RangeInclusive<i32>,
    /// Selectable-dates policy used by the internal state.
    pub selectable_dates: Arc<dyn SelectableDates>,
    /// Initial display mode for the internal state.
    pub display_mode: DatePickerDisplayMode,
    /// First day of the week for the calendar grid.
    pub first_day_of_week: Weekday,
    /// Whether weekday labels are rendered.
    pub show_weekday_labels: bool,
    /// Whether the display mode toggle is shown.
    pub show_mode_toggle: bool,
    /// Optional override for the title text.
    #[setters(strip_option, into)]
    pub title: Option<String>,
    /// Optional override for the headline text.
    #[setters(strip_option, into)]
    pub headline: Option<String>,
}

impl Default for DatePickerArgs {
    fn default() -> Self {
        Self {
            modifier: Modifier::new()
                .constrain(Some(DimensionValue::WRAP), Some(DimensionValue::WRAP)),
            initial_selected_date: None,
            initial_displayed_month: None,
            year_range: DatePickerDefaults::YEAR_RANGE,
            selectable_dates: DatePickerDefaults::all_dates(),
            display_mode: DatePickerDisplayMode::Picker,
            first_day_of_week: Weekday::Monday,
            show_weekday_labels: true,
            show_mode_toggle: true,
            title: None,
            headline: None,
        }
    }
}

/// Configuration for [`date_picker_dialog`].
#[derive(Setters)]
pub struct DatePickerDialogArgs {
    /// State handle used by the embedded date picker.
    #[setters(skip)]
    pub state: State<DatePickerState>,
    /// Optional override for the dialog title.
    #[setters(strip_option, into)]
    pub title: Option<String>,
    /// Optional confirm button content.
    #[setters(skip)]
    pub confirm_button: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Optional dismiss button content.
    #[setters(skip)]
    pub dismiss_button: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Picker configuration forwarded to [`date_picker_with_state`].
    pub picker_args: DatePickerArgs,
}

impl DatePickerDialogArgs {
    /// Creates dialog args with the required date picker state.
    pub fn new(state: State<DatePickerState>) -> Self {
        Self {
            state,
            title: None,
            confirm_button: None,
            dismiss_button: None,
            picker_args: DatePickerArgs::default(),
        }
    }

    /// Sets the confirm button content.
    pub fn confirm_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.confirm_button = Some(Arc::new(f));
        self
    }

    /// Sets the confirm button content using a shared callback.
    pub fn confirm_button_shared(mut self, f: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.confirm_button = Some(f);
        self
    }

    /// Sets the dismiss button content.
    pub fn dismiss_button<F>(mut self, f: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.dismiss_button = Some(Arc::new(f));
        self
    }

    /// Sets the dismiss button content using a shared callback.
    pub fn dismiss_button_shared(mut self, f: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.dismiss_button = Some(f);
        self
    }
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
/// - `args` — configuration for the picker layout and internal state defaults;
///   see [`DatePickerArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui_basic_components::date_picker::{DatePickerArgs, DatePickerState, date_picker};
///
/// date_picker(DatePickerArgs::default());
///
/// let mut state = DatePickerState::default();
/// assert!(state.selected_date().is_none());
/// # }
/// # component();
/// ```
#[tessera]
pub fn date_picker(args: impl Into<DatePickerArgs>) {
    let args: DatePickerArgs = args.into();
    let initial_selected_date = args.initial_selected_date;
    let initial_displayed_month = args.initial_displayed_month;
    let year_range = args.year_range.clone();
    let selectable_dates = args.selectable_dates.clone();
    let display_mode = args.display_mode;

    let state = remember(|| {
        DatePickerState::new(
            initial_selected_date,
            initial_displayed_month,
            year_range,
            selectable_dates,
            display_mode,
        )
    });
    date_picker_with_state(args, state);
}

/// # date_picker_with_state
///
/// Render a date picker using an external state handle.
///
/// ## Usage
///
/// Use when you need to observe or control the selected date externally.
///
/// ## Parameters
///
/// - `args` — configuration for the picker layout; see [`DatePickerArgs`].
/// - `state` — a [`DatePickerState`] storing selection and display mode.
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// use tessera_ui_basic_components::date_picker::{
///     CalendarDate, DatePickerArgs, DatePickerState, date_picker_with_state,
/// };
///
/// let state = remember(DatePickerState::default);
/// date_picker_with_state(DatePickerArgs::default(), state);
///
/// if let Some(date) = CalendarDate::new(2024, 4, 15) {
///     state.with_mut(|s| {
///         assert!(s.set_selected_date(date));
///     });
/// }
/// # }
/// # component();
/// ```
#[tessera]
pub fn date_picker_with_state(args: impl Into<DatePickerArgs>, state: State<DatePickerState>) {
    let args: DatePickerArgs = args.into();
    let snapshot = state.with(|s| s.snapshot());
    let theme = use_context::<MaterialTheme>().get();
    let scheme = theme.color_scheme;
    let typography = theme.typography;

    let modifier = args.modifier;
    let first_day_of_week = args.first_day_of_week;
    let show_weekday_labels = args.show_weekday_labels;
    let show_mode_toggle = args.show_mode_toggle;
    let title_text = args
        .title
        .unwrap_or_else(|| default_title(snapshot.display_mode).to_string());
    let headline_text = args
        .headline
        .unwrap_or_else(|| default_headline(snapshot.selected_date));

    column(ColumnArgs::default().modifier(modifier), move |scope| {
        scope.child(move || {
            row(
                RowArgs::default()
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_all(HEADER_VERTICAL_PADDING),
                    )
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center),
                move |row_scope| {
                    row_scope.child(move || {
                        column(
                            ColumnArgs::default()
                                .modifier(Modifier::new().padding_all(HEADER_HORIZONTAL_PADDING)),
                            move |column_scope| {
                                let title_text = title_text;
                                column_scope.child(move || {
                                    text(
                                        TextArgs::default()
                                            .text(title_text)
                                            .size(typography.title_small.font_size)
                                            .color(scheme.on_surface_variant),
                                    );
                                });

                                let headline_text = headline_text;
                                column_scope.child(move || {
                                    text(
                                        TextArgs::default()
                                            .text(headline_text)
                                            .size(typography.headline_small.font_size)
                                            .color(scheme.on_surface),
                                    );
                                });
                            },
                        );
                    });

                    if show_mode_toggle {
                        row_scope.child(move || {
                            display_mode_toggle(state);
                        });
                    }
                },
            );
        });

        match snapshot.display_mode {
            DatePickerDisplayMode::Picker => {
                scope.child(move || {
                    calendar_view(
                        snapshot.clone(),
                        first_day_of_week,
                        show_weekday_labels,
                        state,
                    );
                });
            }
            DatePickerDisplayMode::Input => {
                scope.child(move || {
                    input_view(snapshot.clone(), state);
                });
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
/// Use inside `dialog_provider` when you need a modal date selection flow.
///
/// ## Parameters
///
/// - `args` — dialog layout and action configuration; see
///   [`DatePickerDialogArgs`].
///
/// ## Examples
///
/// ```
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
/// use tessera_ui::remember;
/// use tessera_ui_basic_components::date_picker::{
///     DatePickerDialogArgs, DatePickerState, date_picker_dialog,
/// };
///
/// let state = remember(DatePickerState::default);
/// date_picker_dialog(DatePickerDialogArgs::new(state));
/// # }
/// # component();
/// ```
#[tessera]
pub fn date_picker_dialog(args: impl Into<DatePickerDialogArgs>) {
    let args: DatePickerDialogArgs = args.into();
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let title = args.title;
    let picker_args = args.picker_args;
    let state = args.state;
    let confirm_button = args.confirm_button;
    let dismiss_button = args.dismiss_button;
    let has_confirm = confirm_button.is_some();
    let has_dismiss = dismiss_button.is_some();

    column(
        ColumnArgs::default().modifier(Modifier::new().constrain(
            Some(DimensionValue::Wrap {
                min: Some(Dp(320.0).into()),
                max: Some(Dp(560.0).into()),
            }),
            Some(DimensionValue::WRAP),
        )),
        move |scope| {
            if let Some(title) = title {
                scope.child(move || {
                    text(
                        TextArgs::default()
                            .text(title)
                            .size(
                                use_context::<MaterialTheme>()
                                    .get()
                                    .typography
                                    .title_medium
                                    .font_size,
                            )
                            .color(scheme.on_surface),
                    );
                });
                scope.child(|| spacer(Modifier::new().height(Dp(8.0))));
            }

            scope.child(move || {
                date_picker_with_state(picker_args, state);
            });

            if has_confirm || has_dismiss {
                scope.child(|| spacer(Modifier::new().height(Dp(16.0))));
                let action_color = scheme.primary;
                scope.child(move || {
                    provide_context(
                        ContentColor {
                            current: action_color,
                        },
                        || {
                            row(
                                RowArgs::default()
                                    .modifier(Modifier::new().fill_max_width())
                                    .main_axis_alignment(MainAxisAlignment::End)
                                    .cross_axis_alignment(CrossAxisAlignment::Center),
                                move |row_scope| {
                                    if let Some(dismiss) = dismiss_button {
                                        row_scope.child(move || dismiss());
                                    }
                                    if has_confirm && has_dismiss {
                                        row_scope.child(|| spacer(Modifier::new().width(Dp(8.0))));
                                    }
                                    if let Some(confirm) = confirm_button {
                                        row_scope.child(move || confirm());
                                    }
                                },
                            );
                        },
                    );
                });
            }
        },
    );
}

fn calendar_view(
    snapshot: DatePickerSnapshot,
    first_day_of_week: Weekday,
    show_weekday_labels: bool,
    state: State<DatePickerState>,
) {
    column(
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        move |scope| {
            let nav_snapshot = snapshot.clone();
            scope.child(move || {
                month_navigation(nav_snapshot, state);
            });

            if show_weekday_labels {
                scope.child(move || {
                    weekday_labels_row(first_day_of_week);
                });
            }

            scope.child(move || {
                date_grid(snapshot, first_day_of_week, state);
            });
        },
    );
}

fn month_navigation(snapshot: DatePickerSnapshot, state: State<DatePickerState>) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let can_prev = can_navigate_prev(snapshot.displayed_month, &snapshot.year_range);
    let can_next = can_navigate_next(snapshot.displayed_month, &snapshot.year_range);
    let month_label = format_month_year(snapshot.displayed_month);

    row(
        RowArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .cross_axis_alignment(CrossAxisAlignment::Center),
        move |scope| {
            scope.child(move || {
                nav_button("<", can_prev, move || {
                    state.with_mut(|s| s.previous_month());
                });
            });

            scope.child(move || {
                text(
                    TextArgs::default()
                        .text(month_label)
                        .size(
                            use_context::<MaterialTheme>()
                                .get()
                                .typography
                                .title_medium
                                .font_size,
                        )
                        .color(scheme.on_surface),
                );
            });

            scope.child(move || {
                nav_button(">", can_next, move || {
                    state.with_mut(|s| s.next_month());
                });
            });
        },
    );
}

fn weekday_labels_row(first_day_of_week: Weekday) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let labels = weekday_sequence(first_day_of_week);

    flow_row(
        FlowRowArgs::default()
            .max_items_per_line(DATE_COLUMNS)
            .item_spacing(DATE_GRID_SPACING),
        move |scope| {
            for weekday in labels {
                let label = weekday_short_label(weekday);
                scope.child(move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE))
                            .style(Color::TRANSPARENT.into())
                            .content_alignment(Alignment::Center),
                        move || {
                            text(
                                TextArgs::default()
                                    .text(label)
                                    .size(
                                        use_context::<MaterialTheme>()
                                            .get()
                                            .typography
                                            .label_small
                                            .font_size,
                                    )
                                    .color(scheme.on_surface_variant),
                            );
                        },
                    );
                });
            }
        },
    );
}

fn date_grid(
    snapshot: DatePickerSnapshot,
    first_day_of_week: Weekday,
    state: State<DatePickerState>,
) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let today = CalendarDate::today();
    let grid = build_month_grid(snapshot.displayed_month, first_day_of_week);

    flow_row(
        FlowRowArgs::default()
            .max_items_per_line(DATE_COLUMNS)
            .max_lines(DATE_ROWS)
            .item_spacing(DATE_GRID_SPACING)
            .line_spacing(DATE_GRID_SPACING),
        move |scope| {
            for cell in grid {
                let snapshot = snapshot.clone();
                scope.child(move || {
                    if let Some(date) = cell {
                        let is_selected = snapshot.selected_date == Some(date);
                        let is_today = date == today;
                        let is_enabled = is_date_selectable(
                            date,
                            &snapshot.year_range,
                            &snapshot.selectable_dates,
                        );
                        let text_color = if is_selected {
                            scheme.on_primary
                        } else if is_enabled {
                            scheme.on_surface
                        } else {
                            scheme
                                .on_surface_variant
                                .with_alpha(MaterialAlpha::DISABLED_CONTENT)
                        };
                        let style = if is_selected {
                            SurfaceStyle::Filled {
                                color: scheme.primary,
                            }
                        } else if is_today {
                            SurfaceStyle::Outlined {
                                color: scheme.primary,
                                width: Dp(1.0),
                            }
                        } else {
                            SurfaceStyle::Filled {
                                color: Color::TRANSPARENT,
                            }
                        };

                        let on_click = if is_enabled {
                            Some(Arc::new(move || {
                                state.with_mut(|s| {
                                    s.set_selected_date(date);
                                });
                            }))
                        } else {
                            None
                        };

                        let mut surface_args = SurfaceArgs::default()
                            .modifier(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE))
                            .style(style)
                            .shape(Shape::rounded_rectangle(DATE_CELL_RADIUS))
                            .content_alignment(Alignment::Center)
                            .enabled(is_enabled);
                        if let Some(on_click) = on_click {
                            surface_args = surface_args.on_click_shared(on_click);
                        }
                        surface(surface_args, move || {
                            text(
                                TextArgs::default()
                                    .text(format!("{}", date.day()))
                                    .size(
                                        use_context::<MaterialTheme>()
                                            .get()
                                            .typography
                                            .body_medium
                                            .font_size,
                                    )
                                    .color(text_color),
                            );
                        });
                    } else {
                        spacer(Modifier::new().size(DATE_CELL_SIZE, DATE_CELL_SIZE));
                    }
                });
            }
        },
    );
}

fn input_view(snapshot: DatePickerSnapshot, state: State<DatePickerState>) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let current_date = snapshot
        .selected_date
        .or_else(|| snapshot.displayed_month.to_date(1))
        .unwrap_or_else(CalendarDate::today);
    let snapshot_year = snapshot.clone();
    let snapshot_month = snapshot.clone();
    let snapshot_day = snapshot.clone();
    let snapshot_desc = snapshot.clone();

    column(
        ColumnArgs::default().modifier(Modifier::new().padding_all(HEADER_HORIZONTAL_PADDING)),
        move |scope| {
            scope.child(move || {
                let decrement_snapshot = snapshot_year.clone();
                let increment_snapshot = snapshot_year.clone();
                input_row(
                    "Year",
                    format!("{}", current_date.year()),
                    move || {
                        adjust_input_date(state, decrement_snapshot.clone(), InputField::Year, -1);
                    },
                    move || {
                        adjust_input_date(state, increment_snapshot.clone(), InputField::Year, 1);
                    },
                );
            });
            scope.child(|| spacer(Modifier::new().height(INPUT_ROW_GAP)));
            scope.child(move || {
                let decrement_snapshot = snapshot_month.clone();
                let increment_snapshot = snapshot_month.clone();
                input_row(
                    "Month",
                    format_month_name(current_date.month()).to_string(),
                    move || {
                        adjust_input_date(state, decrement_snapshot.clone(), InputField::Month, -1);
                    },
                    move || {
                        adjust_input_date(state, increment_snapshot.clone(), InputField::Month, 1);
                    },
                );
            });
            scope.child(|| spacer(Modifier::new().height(INPUT_ROW_GAP)));
            scope.child(move || {
                let decrement_snapshot = snapshot_day.clone();
                let increment_snapshot = snapshot_day.clone();
                input_row(
                    "Day",
                    format!("{}", current_date.day()),
                    move || {
                        adjust_input_date(state, decrement_snapshot.clone(), InputField::Day, -1);
                    },
                    move || {
                        adjust_input_date(state, increment_snapshot.clone(), InputField::Day, 1);
                    },
                );
            });
            scope.child(|| spacer(Modifier::new().height(INPUT_ROW_GAP)));
            scope.child(move || {
                let description = if snapshot_desc.selected_date.is_some() {
                    "Use the steppers to adjust the selected date."
                } else {
                    "Use the steppers to pick a date."
                };
                text(
                    TextArgs::default()
                        .text(description)
                        .size(
                            use_context::<MaterialTheme>()
                                .get()
                                .typography
                                .body_small
                                .font_size,
                        )
                        .color(scheme.on_surface_variant),
                );
            });
        },
    );
}

fn input_row(
    label: &'static str,
    value: String,
    on_decrement: impl Fn() + Send + Sync + 'static,
    on_increment: impl Fn() + Send + Sync + 'static,
) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    row(
        RowArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .cross_axis_alignment(CrossAxisAlignment::Center),
        move |scope| {
            scope.child(move || {
                text(
                    TextArgs::default()
                        .text(label)
                        .size(
                            use_context::<MaterialTheme>()
                                .get()
                                .typography
                                .body_medium
                                .font_size,
                        )
                        .color(scheme.on_surface_variant),
                );
            });
            scope.child(move || {
                row(
                    RowArgs::default()
                        .main_axis_alignment(MainAxisAlignment::Center)
                        .cross_axis_alignment(CrossAxisAlignment::Center),
                    move |row_scope| {
                        row_scope.child(move || {
                            nav_button("-", true, on_decrement);
                        });
                        row_scope.child(|| spacer(Modifier::new().width(Dp(8.0))));
                        row_scope.child(move || {
                            text(
                                TextArgs::default()
                                    .text(value)
                                    .size(
                                        use_context::<MaterialTheme>()
                                            .get()
                                            .typography
                                            .body_large
                                            .font_size,
                                    )
                                    .color(scheme.on_surface),
                            );
                        });
                        row_scope.child(|| spacer(Modifier::new().width(Dp(8.0))));
                        row_scope.child(move || {
                            nav_button("+", true, on_increment);
                        });
                    },
                );
            });
        },
    );
}

fn display_mode_toggle(state: State<DatePickerState>) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let label = state.with(|s| match s.display_mode() {
        DatePickerDisplayMode::Picker => "Input",
        DatePickerDisplayMode::Input => "Calendar",
    });
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().padding_all(Dp(4.0)))
            .style(SurfaceStyle::Filled {
                color: scheme.surface_container_high,
            })
            .shape(Shape::capsule())
            .content_alignment(Alignment::Center)
            .on_click(move || {
                state.with_mut(|s| s.toggle_display_mode());
            }),
        move || {
            text(
                TextArgs::default()
                    .text(label)
                    .size(
                        use_context::<MaterialTheme>()
                            .get()
                            .typography
                            .label_small
                            .font_size,
                    )
                    .color(scheme.primary),
            );
        },
    );
}

fn nav_button(label: &'static str, enabled: bool, on_click: impl Fn() + Send + Sync + 'static) {
    let scheme = use_context::<MaterialTheme>().get().color_scheme;
    let text_color = if enabled {
        scheme.on_surface
    } else {
        scheme
            .on_surface_variant
            .with_alpha(MaterialAlpha::DISABLED_CONTENT)
    };
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().size(NAV_BUTTON_SIZE, NAV_BUTTON_SIZE))
            .style(SurfaceStyle::Filled {
                color: scheme.surface_container_low,
            })
            .shape(Shape::capsule())
            .content_alignment(Alignment::Center)
            .enabled(enabled)
            .on_click(move || {
                if enabled {
                    on_click();
                }
            }),
        move || {
            text(
                TextArgs::default()
                    .text(label)
                    .size(
                        use_context::<MaterialTheme>()
                            .get()
                            .typography
                            .body_medium
                            .font_size,
                    )
                    .color(text_color),
            );
        },
    );
}

fn adjust_input_date(
    state: State<DatePickerState>,
    snapshot: DatePickerSnapshot,
    field: InputField,
    delta: i32,
) {
    let current = snapshot
        .selected_date
        .or_else(|| snapshot.displayed_month.to_date(1))
        .unwrap_or_else(CalendarDate::today);

    let mut year = current.year();
    let mut month = current.month();
    let mut day = current.day();

    match field {
        InputField::Year => {
            year = year.saturating_add(delta);
        }
        InputField::Month => {
            let updated = YearMonth::new(year, month)
                .unwrap_or_else(|| YearMonth::new_unchecked(year, month))
                .add_months(delta);
            year = updated.year();
            month = updated.month();
        }
        InputField::Day => {
            let max_day = days_in_month(year, month);
            let next_day = (day as i32 + delta).clamp(1, max_day as i32) as u8;
            day = next_day;
        }
    }

    let max_day = days_in_month(year, month);
    if day > max_day {
        day = max_day;
    }

    if let Some(date) = CalendarDate::new(year, month, day) {
        state.with_mut(|s| {
            s.set_selected_date(date);
        });
    }
}

enum InputField {
    Year,
    Month,
    Day,
}

fn default_title(mode: DatePickerDisplayMode) -> &'static str {
    match mode {
        DatePickerDisplayMode::Picker => "Select date",
        DatePickerDisplayMode::Input => "Enter date",
    }
}

fn default_headline(selected: Option<CalendarDate>) -> String {
    selected
        .map(format_selected_date)
        .unwrap_or_else(|| "No date selected".to_string())
}

fn format_selected_date(date: CalendarDate) -> String {
    format!(
        "{} {}, {}",
        format_month_short_name(date.month()),
        date.day(),
        date.year()
    )
}

fn format_month_year(month: YearMonth) -> String {
    format!("{} {}", format_month_name(month.month()), month.year())
}

fn format_month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        _ => "December",
    }
}

fn format_month_short_name(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        _ => "Dec",
    }
}

fn weekday_sequence(first_day_of_week: Weekday) -> [Weekday; DATE_COLUMNS] {
    let mut days = [Weekday::Monday; DATE_COLUMNS];
    let start = first_day_of_week.index_from_monday();
    for (idx, slot) in days.iter_mut().enumerate() {
        *slot = Weekday::from_monday_index(start + idx as i32);
    }
    days
}

fn weekday_short_label(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
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
    selectable_dates: &Arc<dyn SelectableDates>,
) -> bool {
    year_range.contains(&date.year())
        && selectable_dates.is_selectable_year(date.year())
        && selectable_dates.is_selectable_date(date)
}

fn normalize_year_range(range: RangeInclusive<i32>) -> RangeInclusive<i32> {
    let start = *range.start();
    let end = *range.end();
    if start <= end { range } else { end..=start }
}

fn fallback_displayed_month(year_range: &RangeInclusive<i32>) -> YearMonth {
    let today = CalendarDate::today();
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
    let mut y = year;
    let m = month as i32;
    let d = day as i32;
    y -= if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as i64
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
