//! Keyboard focus and date arithmetic for calendar navigation.
//!
//! ## Usage
//!
//! Navigate scheduling dates without changing the selected value.

use tessera_ui::{
    FocusManager, FocusProperties, FocusRequester, FocusState, Modifier, State,
    modifier::FocusModifierExt as _,
    winit::{event::ElementState, keyboard::NamedKey},
};

use crate::modifier::with_keyboard_input;

use super::{
    CalendarDate, DatePickerState, DateRangePickerState, Weekday, YearMonth, civil_from_days,
    days_from_civil, days_in_month, is_date_selectable, weekday_for_date,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum CalendarOwner {
    Single(State<DatePickerState>),
    Range(State<DateRangePickerState>),
}

impl CalendarOwner {
    fn month(self) -> YearMonth {
        match self {
            Self::Single(state) => state.with(DatePickerState::displayed_month),
            Self::Range(state) => state.with(DateRangePickerState::displayed_month),
        }
    }

    fn years(self) -> std::ops::RangeInclusive<i32> {
        match self {
            Self::Single(state) => state.with(|state| state.year_range().clone()),
            Self::Range(state) => state.with(|state| state.year_range().clone()),
        }
    }

    fn selected(self) -> Option<CalendarDate> {
        match self {
            Self::Single(state) => state.with(DatePickerState::selected_date),
            Self::Range(state) => {
                state.with(|state| state.selected_end_date().or(state.selected_start_date()))
            }
        }
    }

    fn selectable(self, date: CalendarDate) -> bool {
        match self {
            Self::Single(state) => state.with(|state| {
                is_date_selectable(
                    date,
                    state.year_range(),
                    state.is_selectable_date().as_ref(),
                    state.is_selectable_year().as_ref(),
                )
            }),
            Self::Range(state) => state.with(|state| state.is_date_selectable(date)),
        }
    }

    fn show(self, date: CalendarDate) {
        let month = YearMonth::new(date.year(), date.month()).expect("calendar month");
        if month == self.month() {
            return;
        }
        match self {
            Self::Single(state) => state.with_mut(|state| state.set_displayed_month(month)),
            Self::Range(state) => state.with_mut(|state| state.set_displayed_month(month)),
        }
    }

    pub(super) fn initial_date(self, today: CalendarDate) -> CalendarDate {
        let month = self.month();
        self.selected()
            .into_iter()
            .chain(std::iter::once(today))
            .find(|date| {
                date.year() == month.year()
                    && date.month() == month.month()
                    && self.selectable(*date)
            })
            .or_else(|| {
                (1..=days_in_month(month.year(), month.month()))
                    .filter_map(|day| month.to_date(day))
                    .find(|date| self.selectable(*date))
            })
            .unwrap_or_else(|| month.to_date(1).expect("first calendar date"))
    }
}

pub(super) struct CalendarFocus {
    anchor: CalendarDate,
    pending: Option<CalendarDate>,
    requester: FocusRequester,
}

impl CalendarFocus {
    pub(super) fn new(anchor: CalendarDate) -> Self {
        Self {
            anchor,
            pending: None,
            requester: FocusRequester::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct CalendarKeyboard {
    pub(super) focus: State<CalendarFocus>,
    pub(super) owner: CalendarOwner,
    pub(super) first_day: Weekday,
}

#[cfg(test)]
thread_local! {
    static TEST_KEYBOARD: std::cell::Cell<Option<CalendarKeyboard>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(super) fn test_keyboard() -> CalendarKeyboard {
    TEST_KEYBOARD.get().expect("rendered calendar keyboard")
}

impl CalendarKeyboard {
    pub(super) fn focus_requester(self) -> FocusRequester {
        self.focus.with(|focus| focus.requester)
    }

    pub(super) fn sync(self, today: CalendarDate) {
        #[cfg(test)]
        TEST_KEYBOARD.set(Some(self));
        let (anchor, pending, requester) = self
            .focus
            .with(|focus| (focus.anchor, focus.pending, focus.requester));
        let month = self.owner.month();
        if !self.owner.years().contains(&anchor.year())
            || (pending.is_none()
                && !requester.is_focused()
                && (anchor.year() != month.year() || anchor.month() != month.month()))
        {
            let anchor = self.owner.initial_date(today);
            self.focus.with_mut(|focus| {
                focus.anchor = anchor;
                focus.pending = None;
            });
        }
    }

    pub(super) fn requester(self, date: CalendarDate) -> Option<FocusRequester> {
        self.focus
            .with(|focus| (focus.anchor == date).then_some(focus.requester))
    }

    pub(super) fn request(self, date: CalendarDate) {
        self.focus.with_mut(|focus| {
            focus.anchor = date;
            focus.pending = Some(date);
        });
        self.owner.show(date);
        FocusManager::current().defer_keyboard_until_layout();
    }

    pub(super) fn day_modifier(self, base: Modifier, date: CalendarDate) -> Modifier {
        let mut modifier = base
            .focusable()
            .focus_properties(FocusProperties::new().skip_traversal(self.requester(date).is_none()))
            .on_focus_event(move |state: FocusState| {
                if state.is_focused()
                    && self
                        .focus
                        .with(|focus| focus.pending.is_none() && focus.anchor != date)
                {
                    self.focus.with_mut(|focus| focus.anchor = date);
                }
            });
        if let Some(requester) = self.requester(date) {
            modifier = modifier.focus_requester(requester);
            if self.focus.with(|focus| focus.pending == Some(date)) {
                modifier = modifier.on_globally_positioned(move |_| {
                    if self.focus.with(|focus| focus.pending == Some(date)) {
                        requester.request_focus();
                        self.focus.with_mut(|focus| focus.pending = None);
                    }
                });
            }
        }
        self.input_modifier(modifier, Some(date))
    }

    pub(super) fn viewport_modifier(self, base: Modifier) -> Modifier {
        self.input_modifier(base, None)
    }

    fn input_modifier(self, base: Modifier, date: Option<CalendarDate>) -> Modifier {
        with_keyboard_input(base, move |mut input| {
            if input.key_modifiers.control_key()
                || input.key_modifiers.alt_key()
                || input.key_modifiers.super_key()
            {
                return;
            }
            for event in input.keyboard_events.iter() {
                if event.state != ElementState::Pressed {
                    continue;
                }
                let tessera_ui::winit::keyboard::Key::Named(key) = event.logical_key else {
                    continue;
                };
                let shift = input.key_modifiers.shift_key();
                if !is_navigation_key(key, shift) {
                    continue;
                }
                let anchor = date.unwrap_or_else(|| self.focus.with(|focus| focus.anchor));
                let target = if date.is_some() {
                    navigation_target(anchor, key, shift, self.first_day, self.owner.years())
                } else {
                    anchor
                };
                self.request(target);
                input.block_keyboard();
                return;
            }
        })
    }
}

fn is_navigation_key(key: NamedKey, shift: bool) -> bool {
    matches!(key, NamedKey::PageUp | NamedKey::PageDown)
        || (!shift
            && matches!(
                key,
                NamedKey::ArrowLeft
                    | NamedKey::ArrowRight
                    | NamedKey::ArrowUp
                    | NamedKey::ArrowDown
                    | NamedKey::Home
                    | NamedKey::End
            ))
}

fn navigation_target(
    date: CalendarDate,
    key: NamedKey,
    shift: bool,
    first_day: Weekday,
    years: std::ops::RangeInclusive<i32>,
) -> CalendarDate {
    let first = days_from_civil(*years.start(), 1, 1);
    let last = days_from_civil(*years.end(), 12, 31);
    let days = days_from_civil(date.year(), date.month(), date.day());
    let weekday =
        (weekday_for_date(date).index_from_monday() - first_day.index_from_monday()).rem_euclid(7);
    let target = match key {
        NamedKey::ArrowLeft => days - 1,
        NamedKey::ArrowRight => days + 1,
        NamedKey::ArrowUp => days - 7,
        NamedKey::ArrowDown => days + 7,
        NamedKey::Home => days - i64::from(weekday),
        NamedKey::End => days + i64::from(6 - weekday),
        NamedKey::PageUp | NamedKey::PageDown => {
            let delta = if key == NamedKey::PageUp { -1 } else { 1 } * if shift { 12 } else { 1 };
            let month = (i64::from(date.year()) * 12 + i64::from(date.month()) - 1 + delta).clamp(
                i64::from(*years.start()) * 12,
                i64::from(*years.end()) * 12 + 11,
            );
            let year = month.div_euclid(12) as i32;
            let month = (month.rem_euclid(12) + 1) as u8;
            days_from_civil(year, month, date.day().min(days_in_month(year, month)))
        }
        _ => days,
    };
    let (year, month, day) = civil_from_days(target.clamp(first, last));
    CalendarDate::new(year, month, day).expect("calendar navigation date")
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod tests;
