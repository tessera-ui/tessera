use super::*;

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).unwrap()
}

#[test]
fn calendar_keys_preserve_week_columns_and_respect_the_first_weekday() {
    let current = date(2026, 7, 29);
    for (key, expected) in [
        (NamedKey::ArrowLeft, date(2026, 7, 28)),
        (NamedKey::ArrowRight, date(2026, 7, 30)),
        (NamedKey::ArrowUp, date(2026, 7, 22)),
        (NamedKey::ArrowDown, date(2026, 8, 5)),
        (NamedKey::Home, date(2026, 7, 26)),
        (NamedKey::End, date(2026, 8, 1)),
    ] {
        assert_eq!(
            navigation_target(current, key, false, Weekday::Sunday, 2020..=2030),
            expected
        );
    }
    assert_eq!(
        navigation_target(current, NamedKey::Home, false, Weekday::Monday, 2020..=2030),
        date(2026, 7, 27)
    );
    assert_eq!(
        navigation_target(current, NamedKey::End, false, Weekday::Monday, 2020..=2030),
        date(2026, 8, 2)
    );
}

#[test]
fn month_and_year_keys_clamp_short_months_and_leap_days() {
    for (current, key, shift, expected) in [
        (
            date(2024, 1, 31),
            NamedKey::PageDown,
            false,
            date(2024, 2, 29),
        ),
        (
            date(2024, 3, 31),
            NamedKey::PageUp,
            false,
            date(2024, 2, 29),
        ),
        (
            date(2024, 2, 29),
            NamedKey::PageDown,
            true,
            date(2025, 2, 28),
        ),
        (date(2024, 2, 29), NamedKey::PageUp, true, date(2023, 2, 28)),
        (
            date(2026, 8, 31),
            NamedKey::PageDown,
            false,
            date(2026, 9, 30),
        ),
        (
            date(2026, 12, 31),
            NamedKey::PageDown,
            false,
            date(2027, 1, 31),
        ),
    ] {
        assert_eq!(
            navigation_target(current, key, shift, Weekday::Sunday, 2020..=2030),
            expected
        );
    }
}

#[test]
fn date_navigation_stays_inside_year_limits_including_integer_extremes() {
    for key in [
        NamedKey::ArrowLeft,
        NamedKey::ArrowUp,
        NamedKey::Home,
        NamedKey::PageUp,
    ] {
        assert_eq!(
            navigation_target(date(2026, 1, 1), key, false, Weekday::Sunday, 2026..=2026),
            date(2026, 1, 1)
        );
    }
    for key in [
        NamedKey::ArrowRight,
        NamedKey::ArrowDown,
        NamedKey::End,
        NamedKey::PageDown,
    ] {
        assert_eq!(
            navigation_target(date(2026, 12, 31), key, true, Weekday::Sunday, 2026..=2026),
            date(2026, 12, 31)
        );
    }
    assert_eq!(
        navigation_target(
            date(i32::MIN, 1, 1),
            NamedKey::ArrowLeft,
            false,
            Weekday::Sunday,
            i32::MIN..=i32::MAX
        ),
        date(i32::MIN, 1, 1)
    );
    assert_eq!(
        navigation_target(
            date(i32::MAX, 12, 31),
            NamedKey::ArrowRight,
            false,
            Weekday::Sunday,
            i32::MIN..=i32::MAX
        ),
        date(i32::MAX, 12, 31)
    );
}
