//! Docked-calendar geometry and date-selection regressions.
//!
//! ## Usage
//!
//! Keep compact calendars usable through month changes and restricted dates.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, State,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
    remember, tessera,
};

use crate::theme::material_theme;

use super::{CalendarDate, DatePickerState, Weekday, YearMonth, docked_date_picker};

thread_local! {
    static STATE: Cell<Option<State<DatePickerState>>> = const { Cell::new(None) };
    static CHANGES: Cell<Option<State<u32>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        let selectable = tessera_ui::CallbackWith::new(|date: CalendarDate| date.day() != 18);
        let initial = DatePickerState::new(
            CalendarDate::new(2025, 8, 17),
            YearMonth::new(2025, 8),
            2024..=2026,
            Some(selectable),
            None,
            super::super::DatePickerDisplayMode::Picker,
        );
        let state = remember(move || initial);
        let changes = remember(|| 0_u32);
        STATE.set(Some(state));
        CHANGES.set(Some(changes));
        crate::boxed::boxed().children(move || {
            docked_date_picker()
                .state(state)
                .today(CalendarDate::new(2025, 8, 17).unwrap())
                .first_day_of_week(Weekday::Monday)
                .on_date_change(move |_| changes.with_mut(|count| *count += 1));
        });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn activate(renderer: &mut HeadlessRenderer, label: &str) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let target_node = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label) && node.supports_action(Action::Click))
        .unwrap_or_else(|| panic!("missing actionable {label}: {snapshot:#?}"))
        .0;
    assert!(
        renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        }),
        "failed to activate {label}: {:?}",
        node(renderer, label).1
    );
    renderer.render_frames(3);
}

fn check_calendar() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 420,
            height: 560,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let bounds = node(&renderer, "Docked calendar").1.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (360.0, 460.0));
    assert!(node(&renderer, "August 18, 2025").1.is_disabled());
    let state = STATE.get().unwrap();
    let changes = CHANGES.get().unwrap();
    activate(&mut renderer, "July 31, 2025");
    assert_eq!(
        state.with(DatePickerState::selected_date),
        CalendarDate::new(2025, 7, 31)
    );
    assert_eq!(changes.get(), 1);
    activate(&mut renderer, "Next year");
    activate(&mut renderer, "Previous year");
    activate(&mut renderer, "Previous year");
    assert_eq!(
        state.with(DatePickerState::displayed_month),
        YearMonth::new(2024, 7).unwrap()
    );
    assert!(node(&renderer, "Previous year").1.is_disabled());
    activate(&mut renderer, "Choose month");
    activate(&mut renderer, "June");
    assert_eq!(
        state.with(DatePickerState::displayed_month),
        YearMonth::new(2024, 6).unwrap()
    );
    activate(&mut renderer, "Choose year");
    activate(&mut renderer, "2026");
    assert_eq!(
        state.with(DatePickerState::displayed_month),
        YearMonth::new(2026, 6).unwrap()
    );
    assert_eq!(changes.get(), 1, "navigation must not commit a date");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn docked_calendar_preserves_size_adjacent_selection_and_current_navigation_state() {
    crate::test_utils::run_isolated_test(
        "date_picker::docked::tests::docked_calendar_preserves_size_adjacent_selection_and_current_navigation_state",
        check_calendar,
    );
}
