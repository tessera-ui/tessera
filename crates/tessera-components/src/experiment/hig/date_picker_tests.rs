use tessera_ui::{
    State,
    accesskit::{Action, Role},
    remember, tessera,
    testing::{TestProbe, UiTest},
    winit::keyboard::NamedKey,
};

use crate::column::column;

use super::{CalendarDate, DatePickerStyle, date_picker};

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("valid date")
}

#[test]
fn calendar_arithmetic_follows_the_gregorian_calendar() {
    assert_eq!(date(2026, 11, 21).weekday(), 6, "a Saturday");
    assert_eq!(date(2000, 1, 1).weekday(), 6, "a Saturday");
    assert_eq!(date(2026, 1, 31).adding_months(1), date(2026, 2, 28));
    assert_eq!(date(2024, 1, 31).adding_months(1), date(2024, 2, 29));
    assert_eq!(date(2000, 3, 1).adding_days(-1), date(2000, 2, 29));
    assert_eq!(date(2026, 12, 31).adding_days(1), date(2027, 1, 1));
    assert_eq!(date(2026, 3, 15).adding_months(-15), date(2024, 12, 15));
    assert!(CalendarDate::new(2026, 2, 29).is_none());
}

#[tessera]
fn fixture(style: DatePickerStyle, probe: TestProbe<Option<State<CalendarDate>>>) {
    let selection = remember(|| date(2026, 11, 21));
    probe.set(Some(selection));
    column().children(move || {
        date_picker(selection.get())
            .style(style)
            .range((date(2026, 1, 1), date(2026, 12, 31)))
            .accessibility_label("Due")
            .on_change(move |next| selection.set(next));
    });
}

#[test]
fn calendar_days_choose_dates_and_the_arrows_change_month() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 400)
        .content(|| fixture(DatePickerStyle::Graphical, probe.clone()));
    ui.frame();
    let selection = probe.get().expect("fixture state");
    let (day, _) = ui.accessibility_node("November 25, 2026");
    assert!(ui.accessibility_action(day, Action::Click));
    ui.frames(2);
    assert_eq!(selection.get(), date(2026, 11, 25));
    let (next, _) = ui.accessibility_node("Next Month");
    assert!(ui.accessibility_action(next, Action::Click));
    ui.frames(2);
    let (day, _) = ui.accessibility_node("December 3, 2026");
    assert!(ui.accessibility_action(day, Action::Click));
    ui.frames(2);
    assert_eq!(selection.get(), date(2026, 12, 3));
}

#[test]
fn field_segments_step_with_the_arrow_keys_within_the_range() {
    let probe = TestProbe::default();
    let mut ui = UiTest::new()
        .viewport(400, 200)
        .content(|| fixture(DatePickerStyle::Field, probe.clone()));
    ui.frame();
    let selection = probe.get().expect("fixture state");
    let snapshot = ui.inspect_accessibility().expect("published semantics");
    let (month, _) = snapshot
        .find("Month", Role::SpinButton)
        .expect("month segment");
    assert!(ui.accessibility_action(month, Action::Focus));
    ui.frame();
    ui.press_key(NamedKey::ArrowUp);
    ui.frames(2);
    assert_eq!(selection.get(), date(2026, 12, 21));
    ui.press_key(NamedKey::ArrowUp);
    ui.frames(2);
    assert_eq!(
        selection.get(),
        date(2026, 12, 31),
        "a date past the range clamps to its last day"
    );
    let (_, picker) = ui.accessibility_node("Due");
    assert_eq!(picker.value(), Some("December 31, 2026"));
}
