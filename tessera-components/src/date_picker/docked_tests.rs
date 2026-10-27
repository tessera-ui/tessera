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

use crate::{modifier::ModifierExt as _, theme::material_theme};

use super::{
    CalendarDate, DatePickerState, DockedDatePickerView, Weekday, YearMonth, docked_date_picker,
};

thread_local! {
    static STATE: Cell<Option<State<DatePickerState>>> = const { Cell::new(None) };
    static CHANGES: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static VIEW: Cell<Option<State<DockedDatePickerView>>> = const { Cell::new(None) };
    static OFFSET: Cell<Option<State<tessera_ui::Dp>>> = const { Cell::new(None) };
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
        let view = remember(|| DockedDatePickerView::Month);
        STATE.set(Some(state));
        CHANGES.set(Some(changes));
        VIEW.set(Some(view));
        let offset = remember(|| tessera_ui::Dp::ZERO);
        OFFSET.set(Some(offset));
        crate::boxed::boxed()
            .modifier(tessera_ui::Modifier::new().offset(tessera_ui::Dp::ZERO, offset.get()))
            .children(move || {
                docked_date_picker()
                    .state(state)
                    .view(view)
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
    assert_eq!((bounds.width(), bounds.height()), (360.0, 428.0));
    let pixels = renderer.read_pixels().unwrap();
    let pixel = |x: usize, y: usize| {
        let x = bounds.x0 as usize + x;
        let y = bounds.y0 as usize + y;
        &pixels[(y * 420 + x) * 4..][..4]
    };
    assert_eq!(pixel(5, 1), pixel(1, 1));
    assert_ne!(pixel(16, 1), pixel(1, 1));
    let month_options = renderer.accessibility_snapshot().unwrap();
    assert!(!month_options.nodes.iter().any(|(_, node)| {
        matches!(
            node.label(),
            Some("Cancel" | "OK" | "Previous month" | "Next year")
        )
    }));
    let state = STATE.get().unwrap();
    let changes = CHANGES.get().unwrap();
    let view = VIEW.get().unwrap();
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == tessera_ui::accesskit::Role::ListBoxOption
                    && node.label() == Some("August")
                    && node.is_selected() == Some(true)
            })
    );
    activate(&mut renderer, "Choose month");
    assert_eq!(view.get(), DockedDatePickerView::Calendar);
    let bounds = node(&renderer, "Docked calendar").1.bounds().unwrap();
    assert_eq!((bounds.width(), bounds.height()), (360.0, 460.0));
    assert!(node(&renderer, "August 18, 2025").1.is_disabled());
    let snapshot = renderer.snapshot();
    for (button, caption) in [("Choose month", "Aug"), ("Choose year", "2025")] {
        let target = node(&renderer, button).1.bounds().unwrap();
        let mut pending = vec![&snapshot["root"]];
        let mut caption_center = None;
        while let Some(entry) = pending.pop() {
            if entry["accessibility"]["label"].as_str() == Some(caption)
                && let (Some(x), Some(y), Some(height)) = (
                    entry["bounds"]["x"].as_f64(),
                    entry["bounds"]["y"].as_f64(),
                    entry["bounds"]["height"].as_f64(),
                )
                && x >= target.x0
                && x < target.x1
                && y >= target.y0
                && y < target.y1
            {
                caption_center = Some(y + height / 2.0);
            }
            if let Some(children) = entry["children"].as_array() {
                pending.extend(children);
            }
        }
        assert!(
            (caption_center.expect("visible navigation caption")
                - (target.y0 + target.height() / 2.0))
                .abs()
                <= 1.0,
            "month and year captions must be vertically centered in their targets"
        );
    }
    let offset = OFFSET.get().unwrap();
    offset.set(tessera_ui::Dp(600.0));
    renderer.render_frames(3);
    offset.set(tessera_ui::Dp::ZERO);
    renderer.render_frames(3);

    let date_bounds = node(&renderer, "July 28, 2025").1.bounds().unwrap();
    let date_ink = |pixels: &[u8]| {
        (date_bounds.y0.ceil() as usize..date_bounds.y1.floor() as usize)
            .flat_map(|y| {
                (date_bounds.x0.ceil() as usize..date_bounds.x1.floor() as usize)
                    .map(move |x| (y * 420 + x) * 4)
            })
            .filter(|&offset| {
                pixels[offset..offset + 3]
                    .iter()
                    .map(|&value| u32::from(value))
                    .sum::<u32>()
                    < 400
            })
            .count()
    };
    assert!(date_ink(&renderer.read_pixels().unwrap()) > 4);
    renderer.pointer_move(
        tessera_ui::MOUSE_POINTER_ID,
        tessera_ui::PxPosition::from_f32_xy(
            (date_bounds.x0 + date_bounds.width() / 2.0) as f32,
            (date_bounds.y0 + date_bounds.height() / 2.0) as f32,
        ),
    );
    renderer.render_frames(4);
    assert!(
        date_ink(&renderer.read_pixels().unwrap()) > 4,
        "hover must retain the visible date caption"
    );
    renderer.pointer_move(
        tessera_ui::MOUSE_POINTER_ID,
        tessera_ui::PxPosition::from_f32_xy(410.0, 550.0),
    );
    renderer.render_frames(4);
    assert!(
        date_ink(&renderer.read_pixels().unwrap()) > 4,
        "leaving hover must retain the visible date caption"
    );

    view.set(DockedDatePickerView::Year);
    renderer.render_frames(3);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == tessera_ui::accesskit::Role::ListBoxOption
                    && node.label() == Some("2025")
                    && node.is_selected() == Some(true)
            })
    );
    activate(&mut renderer, "2025");
    assert_eq!(view.get(), DockedDatePickerView::Calendar);
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
    assert_eq!(view.get(), DockedDatePickerView::Month);
    activate(&mut renderer, "June");
    assert_eq!(view.get(), DockedDatePickerView::Calendar);
    assert_eq!(
        state.with(DatePickerState::displayed_month),
        YearMonth::new(2024, 6).unwrap()
    );
    activate(&mut renderer, "Choose year");
    assert_eq!(view.get(), DockedDatePickerView::Year);
    activate(&mut renderer, "2026");
    assert_eq!(view.get(), DockedDatePickerView::Calendar);
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
