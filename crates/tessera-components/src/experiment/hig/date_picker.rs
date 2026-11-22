//! Date pickers with editable fields and a month calendar.
//!
//! ## Usage
//!
//! Choose a due date, a birthday, or a date range limit in a form.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, FloatRect, FocusRequester, FocusScopeNode,
    FocusTraversalPolicy, KeyboardInput, KeyboardInputModifierNode, Modifier, RenderSlot,
    accesskit::Role,
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    remember, tessera, use_context,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use crate::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    modifier::ModifierExt as _,
    row::row,
    shape_def::Shape,
    spacer::spacer,
};

use super::{
    button::{ButtonStyle, button},
    menu::{MenuContext, present},
    presentation::PopupPlacement,
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "date_picker_tests.rs"]
mod tests;

/// A day in the proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Year, such as 2026.
    pub year: i32,
    /// Month from 1 to 12.
    pub month: u8,
    /// Day of the month from 1.
    pub day: u8,
}

impl CalendarDate {
    /// Creates a date, or `None` when the month or day does not exist.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }

    /// Returns the date `days` days later, or earlier when negative.
    pub fn adding_days(self, days: i64) -> Self {
        Self::from_ordinal(self.ordinal() + days)
    }

    /// Returns the date `months` months later, keeping the day within the month.
    pub fn adding_months(self, months: i32) -> Self {
        let index = self.year * 12 + i32::from(self.month) - 1 + months;
        let year = index.div_euclid(12);
        let month = (index.rem_euclid(12) + 1) as u8;
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    /// Day of the week, from 0 for Sunday to 6 for Saturday.
    pub fn weekday(self) -> u8 {
        // The ordinal counts from 0001-01-01, a Monday.
        ((self.ordinal() + 1).rem_euclid(7)) as u8
    }

    fn ordinal(self) -> i64 {
        let year = i64::from(self.year) - 1;
        let mut days =
            year * 365 + year.div_euclid(4) - year.div_euclid(100) + year.div_euclid(400);
        for month in 1..self.month {
            days += i64::from(days_in_month(self.year, month));
        }
        days + i64::from(self.day) - 1
    }

    fn from_ordinal(ordinal: i64) -> Self {
        let mut year = (ordinal / 366 + 1) as i32;
        while Self::new(year + 1, 1, 1).is_some_and(|date| date.ordinal() <= ordinal) {
            year += 1;
        }
        while (Self {
            year,
            month: 1,
            day: 1,
        })
        .ordinal()
            > ordinal
        {
            year -= 1;
        }
        let mut remaining = ordinal
            - Self {
                year,
                month: 1,
                day: 1,
            }
            .ordinal();
        let mut month = 1;
        while remaining >= i64::from(days_in_month(year, month)) {
            remaining -= i64::from(days_in_month(year, month));
            month += 1;
        }
        Self {
            year,
            month,
            day: remaining as u8 + 1,
        }
    }

    fn clamped(self, range: Option<(CalendarDate, CalendarDate)>) -> Self {
        match range {
            Some((first, last)) => self.max(first).min(last),
            None => self,
        }
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The appearance of a [`date_picker`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DatePickerStyle {
    /// The platform default, a compact field on macOS.
    #[default]
    Automatic,
    /// A field that shows a calendar when clicked.
    Compact,
    /// A field with editable month, day, and year.
    Field,
    /// A field with a stepper beside it.
    StepperField,
    /// A month calendar.
    Graphical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Segment {
    Month,
    Day,
    Year,
}

impl Segment {
    fn step(self, date: CalendarDate, by: i32) -> CalendarDate {
        match self {
            Self::Month => date.adding_months(by),
            Self::Day => date.adding_days(i64::from(by)),
            Self::Year => date.adding_months(by * 12),
        }
    }
}

/// # date_picker
///
/// Chooses a calendar date in an editable field or a month calendar.
///
/// ## Usage
///
/// Pick due dates, birthdays, and other days in forms and inspectors.
///
/// ## Parameters
///
/// - `selection`, `on_change` — the chosen date and requested changes.
/// - `style` — field, stepper field, compact field with a calendar, or calendar; defaults to
///   [`DatePickerStyle::Automatic`].
/// - `range` — earliest and latest selectable dates.
/// - `enabled` — whether the date can change; defaults to true.
/// - `modifier` — layout and additional semantics.
/// - `accessibility_label` — spoken name of the picker.
///
/// The compact style's calendar appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::date_picker::{CalendarDate, date_picker};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let due = remember(|| CalendarDate::new(2026, 11, 21).expect("valid date"));
/// date_picker(due.get())
///     .accessibility_label("Due date")
///     .on_change(move |date| due.set(date));
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn date_picker(
    selection: CalendarDate,
    on_change: Option<CallbackWith<CalendarDate>>,
    style: Option<DatePickerStyle>,
    range: Option<(CalendarDate, CalendarDate)>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let change = CallbackWith::new(move |date: CalendarDate| {
        let date = date.clamped(range);
        if enabled
            && date != selection
            && let Some(on_change) = on_change
        {
            on_change.call(date);
        }
    });
    let modifier = modifier.unwrap_or_default().semantics(SemanticsArgs {
        role: Some(Role::DateInput),
        label: accessibility_label,
        value: Some(format!(
            "{} {}, {}",
            MONTHS[usize::from(selection.month) - 1],
            selection.day,
            selection.year
        )),
        ..Default::default()
    });
    match style.unwrap_or_default() {
        DatePickerStyle::Graphical => {
            calendar(selection, change).modifier(modifier);
        }
        DatePickerStyle::Field => {
            date_field(selection, change, false, enabled).modifier(modifier);
        }
        DatePickerStyle::StepperField => {
            date_field(selection, change, true, enabled).modifier(modifier);
        }
        DatePickerStyle::Automatic | DatePickerStyle::Compact => {
            compact_field(selection, change, enabled).modifier(modifier);
        }
    }
}

#[tessera]
fn compact_field(
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
    enabled: bool,
    modifier: Option<Modifier>,
) {
    let open = remember(|| false);
    let anchor = remember(|| FloatRect::ZERO);
    let scope = remember(FocusScopeNode::new).get();
    let dismiss = Callback::new(move || {
        if open.get() {
            open.set(false);
        }
    });
    present(
        open.get() && enabled,
        anchor.get(),
        PopupPlacement::Below,
        RenderSlot::new(move || calendar_panel(selection, change)),
        dismiss,
        scope,
        0.0,
    );
    date_field(selection, change, false, enabled).modifier(
        modifier
            .unwrap_or_default()
            .on_globally_positioned_precise(move |rect| {
                if anchor.get() != rect {
                    anchor.set(rect);
                }
            })
            .clickable(ClickableArgs {
                on_click: Callback::new(move || open.set(!open.get())),
                enabled,
                ..Default::default()
            }),
    );
}

#[tessera]
fn calendar_panel(selection: CalendarDate, change: CallbackWith<CalendarDate>) {
    let dismiss = use_context::<MenuContext>().map(|context| context.get().dismiss);
    let choose = CallbackWith::new(move |date: CalendarDate| {
        change.call(date);
        if let Some(dismiss) = dismiss {
            dismiss.call();
        }
    });
    boxed()
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp(3.0)))
        .children(move || {
            calendar(selection, choose);
        });
}

struct SegmentValue(String);

impl tessera_ui::SemanticsModifierNode for SegmentValue {
    fn apply(
        &self,
        node: &mut tessera_ui::AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.value = Some(self.0.clone());
    }
}

struct SegmentKeys {
    segment: Segment,
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
}

impl KeyboardInputModifierNode for SegmentKeys {
    fn on_keyboard_input(&self, input: KeyboardInput<'_>) {
        let mut step = 0;
        input.keyboard_events.retain(|event| {
            if event.state != ElementState::Pressed {
                return true;
            }
            match event.logical_key {
                Key::Named(NamedKey::ArrowUp) => step += 1,
                Key::Named(NamedKey::ArrowDown) => step -= 1,
                _ => return true,
            }
            false
        });
        if step != 0 {
            self.change.call(self.segment.step(self.selection, step));
        }
    }
}

#[tessera]
fn date_field(
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
    stepper: bool,
    enabled: bool,
    modifier: Option<Modifier>,
) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let last = remember(|| Segment::Day);
    let shape = Shape::rounded_rectangle(Dp(6.0));
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(modifier.unwrap_or_default())
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(
                    Modifier::new()
                        .height(Dp(22.0))
                        .background_with_shape(
                            if dark {
                                Color::from_rgb(0.0086, 0.0086, 0.0086)
                            } else {
                                Color::WHITE
                            },
                            shape,
                        )
                        .border_with_shape(Dp(1.0), tint.with_alpha(0.12), shape)
                        .padding_symmetric(Dp(5.0), Dp::ZERO)
                        .focus_group(None)
                        .focus_traversal_policy(FocusTraversalPolicy::linear()),
                )
                .children(move || {
                    for (index, segment) in [Segment::Month, Segment::Day, Segment::Year]
                        .into_iter()
                        .enumerate()
                    {
                        if index > 0 {
                            text("/").color(tint.with_alpha(0.5));
                        }
                        date_segment(segment, selection, change, enabled, last, tint);
                    }
                });
            if stepper {
                spacer().modifier(Modifier::new().width(Dp(4.0)));
                date_stepper(selection, change, enabled, last);
            }
        });
}

#[tessera]
fn date_segment(
    segment: Segment,
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
    enabled: bool,
    last: tessera_ui::State<Segment>,
    tint: Color,
) {
    let requester = remember(FocusRequester::new).get();
    let focused = requester.is_focused();
    if focused && last.get() != segment {
        last.set(segment);
    }
    let (value, name) = match segment {
        Segment::Month => (selection.month.to_string(), "Month"),
        Segment::Day => (selection.day.to_string(), "Day"),
        Segment::Year => (selection.year.to_string(), "Year"),
    };
    let ink = if focused {
        Color::WHITE
    } else {
        tint.with_alpha(if enabled { 0.85 } else { 0.25 })
    };
    let mut modifier = Modifier::new().padding_symmetric(Dp(2.0), Dp(1.0));
    if focused {
        modifier = modifier.background_with_shape(
            Color::from_rgb(0.0, 0.19461783, 1.0),
            Shape::rounded_rectangle(Dp(3.0)),
        );
    }
    modifier = modifier.clickable(ClickableArgs {
        on_click: Callback::noop(),
        enabled,
        role: Some(Role::SpinButton),
        label: Some(name.into()),
        focus_requester: Some(requester),
        ..Default::default()
    });
    if enabled {
        modifier = modifier.push_keyboard_input(SegmentKeys {
            segment,
            selection,
            change,
        });
    }
    modifier = modifier.push_semantics(SegmentValue(value.clone()));
    boxed().modifier(modifier).children(move || {
        text(value.clone()).color(ink).single_line(true);
    });
}

#[tessera]
fn date_stepper(
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
    enabled: bool,
    last: tessera_ui::State<Segment>,
) {
    column().children(move || {
        for (label, by) in [("Increment", 1), ("Decrement", -1)] {
            button(if by > 0 { "▲" } else { "▼" })
                .style(ButtonStyle::Borderless)
                .control_size(super::theme::ControlSize::Mini)
                .enabled(enabled)
                .accessibility_label(label)
                .on_click(move || change.call(last.get().step(selection, by)));
        }
    });
}

/// A month calendar showing the selection's month, with buttons to change month.
#[tessera]
fn calendar(
    selection: CalendarDate,
    change: CallbackWith<CalendarDate>,
    modifier: Option<Modifier>,
) {
    let shown = remember(|| (selection.year, selection.month));
    let (year, month) = shown.get();
    let first = CalendarDate::new(year, month, 1).unwrap_or(selection);
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    column()
        .modifier(modifier.unwrap_or_default().width(Dp(7.0 * 26.0)))
        .children(move || {
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(Modifier::new().fill_max_width().height(Dp(24.0)))
                .children(move || {
                    text(format!("{} {}", MONTHS[usize::from(month) - 1], year))
                        .style(TextStyle {
                            font_weight: 600,
                            ..Default::default()
                        })
                        .color(tint.with_alpha(0.85));
                    spacer().modifier(Modifier::new().weight(1.0));
                    for (label, by) in [("Previous Month", -1), ("Next Month", 1)] {
                        button(if by < 0 { "‹" } else { "›" })
                            .style(ButtonStyle::Borderless)
                            .control_size(super::theme::ControlSize::Small)
                            .accessibility_label(label)
                            .on_click(move || {
                                let next = first.adding_months(by);
                                shown.set((next.year, next.month));
                            });
                    }
                });
            row().children(move || {
                for initial in ["S", "M", "T", "W", "T", "F", "S"] {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(Modifier::new().size(Dp(26.0), Dp(20.0)))
                        .children(move || {
                            text(initial).color(tint.with_alpha(0.5));
                        });
                }
            });
            let lead = i64::from(first.weekday());
            for week in 0..6 {
                row().children(move || {
                    for weekday in 0..7 {
                        let date = first.adding_days(week * 7 + weekday - lead);
                        day_cell(date, date == selection, date.month == month, tint, change);
                    }
                });
            }
        });
}

#[tessera]
fn day_cell(
    date: CalendarDate,
    selected: bool,
    in_month: bool,
    tint: Color,
    change: CallbackWith<CalendarDate>,
) {
    let ink = if selected {
        Color::WHITE
    } else if in_month {
        tint.with_alpha(0.85)
    } else {
        tint.with_alpha(0.25)
    };
    let mut modifier = Modifier::new()
        .size(Dp(26.0), Dp(24.0))
        .clickable(ClickableArgs {
            on_click: Callback::new(move || change.call(date)),
            role: Some(Role::Button),
            label: Some(format!(
                "{} {}, {}",
                MONTHS[usize::from(date.month) - 1],
                date.day,
                date.year
            )),
            ..Default::default()
        });
    if selected {
        modifier = modifier.background_with_shape(
            Color::from_rgb(0.0, 0.19461783, 1.0),
            Shape::rounded_rectangle(Dp(12.0)),
        );
    }
    boxed()
        .alignment(Alignment::Center)
        .modifier(modifier)
        .children(move || {
            text(date.day.to_string()).color(ink).single_line(true);
        });
}
