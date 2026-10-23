use std::cell::Cell;

use tessera_ui::{
    CallbackWith, Color, Dp, FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer,
    MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition, State,
    accesskit::{Rect, Role},
    remember, tessera,
};

use crate::{
    button::button,
    column::column,
    lazy_list::{LazyListController, lazy_column},
    modifier::ModifierExt as _,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    theme::{MaterialTheme, material_theme},
};

use super::{
    CalendarDate, DatePickerDisplayMode, DateRangePickerState, Weekday, YearMonth,
    date_range_picker,
};

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(2026, month, day).unwrap()
}

#[test]
fn range_selection_is_atomic_inclusive_and_restarts_after_completion() {
    let mut state = DateRangePickerState::default();
    assert!(!state.set_selection(None, Some(date(8, 3))));
    assert!(state.select_date(date(7, 29)));
    assert!(!state.is_complete());
    assert!(state.select_date(date(7, 27)));
    assert_eq!(state.selected_start_date(), Some(date(7, 27)));
    assert!(state.select_date(date(8, 3)));
    assert!(state.is_complete());
    assert!(!state.set_selection(Some(date(8, 4)), Some(date(8, 3))));
    assert_eq!(state.selected_start_date(), Some(date(7, 27)));
    assert_eq!(state.selected_end_date(), Some(date(8, 3)));
    assert!(state.select_date(date(8, 3)));
    assert!(!state.is_complete());
    assert!(state.select_date(date(8, 3)));
    assert!(state.is_complete());
    state.set_year_range(2027..=2028);
    assert!(!state.is_complete());
    assert_eq!(state.selected_start_date(), None);
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(state.displayed_month(), YearMonth::new(2027, 1).unwrap());
}

#[test]
fn range_input_resolves_both_endpoints_and_preserves_invalid_drafts() {
    let mut state = DateRangePickerState::default();
    let mut values = [String::new(), "08/03/2026".to_owned()];
    super::apply_draft(&mut state, &values);
    assert!(!state.is_complete());
    assert_eq!(
        super::input_error(&values[1], false, &state),
        Some("Enter a start date first")
    );
    values[0] = "07/29/2026".to_owned();
    super::apply_draft(&mut state, &values);
    assert!(state.is_complete());
    values[0] = "08/04/2026".to_owned();
    super::apply_draft(&mut state, &values);
    assert_eq!(state.selected_end_date(), None);
    assert_eq!(
        super::input_error(&values[1], false, &state),
        Some("End date must be on or after start date")
    );
    values[1] = "08/05/2026".to_owned();
    super::apply_draft(&mut state, &values);
    assert!(state.is_complete());
    values[1] = "02/29/2026".to_owned();
    super::apply_draft(&mut state, &values);
    assert!(!state.is_complete());
    assert_eq!(
        super::input_error(&values[1], false, &state),
        Some("Enter a date as MM/DD/YYYY")
    );
}

thread_local! {
    static STATE: Cell<Option<State<DateRangePickerState>>> = const { Cell::new(None) };
    static OUTER_SCROLL: Cell<Option<State<LazyListController>>> = const { Cell::new(None) };
}

#[tessera]
fn nested_fixture() {
    let state = remember(|| {
        DateRangePickerState::new(
            None,
            None,
            YearMonth::new(2026, 7),
            2026..=2027,
            None,
            None,
            DatePickerDisplayMode::Picker,
        )
    });
    let outer = remember(LazyListController::new);
    STATE.set(Some(state));
    OUTER_SCROLL.set(Some(outer));
    material_theme().child(move || {
        lazy_column()
            .modifier(Modifier::new().fill_max_size())
            .controller(outer)
            .overscan(1)
            .estimated_item_size(Dp(300.0))
            .item(|| {
                column().children(|| {
                    button()
                        .text()
                        .accessibility_label("Outside calendar")
                        .on_click(|| {})
                        .child(|| {
                            text().content("Outside calendar");
                        });
                    spacer().modifier(Modifier::new().height(Dp(300.0)));
                });
            })
            .item(move || {
                date_range_picker()
                    .state(state)
                    .first_day_of_week(Weekday::Sunday);
            })
            .item(|| {
                column().children(|| {
                    button()
                        .text()
                        .accessibility_label("After calendar")
                        .on_click(|| {})
                        .child(|| {
                            text().content("After calendar");
                        });
                    spacer().modifier(Modifier::new().height(Dp(400.0)));
                });
            });
    });
}

fn check_nested_scroll(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| nested_fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 950,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("nested renderer");
    renderer.render_frames(8);
    let outer = OUTER_SCROLL.get().expect("outer scroll");
    outer.with_mut(|scroll| scroll.scroll_to(Dp::ZERO, Dp(400.0 / 1.5)));
    renderer.render_frames(8);
    click(&mut renderer, "July 29, 2026");
    let state = STATE.get().expect("range state");
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29))
    );
    let accessibility = renderer.accessibility_snapshot().expect("tree");
    let focused = accessibility
        .nodes
        .iter()
        .find(|(id, _)| *id == accessibility.focus)
        .expect("focused date");
    assert_eq!(focused.1.label(), Some("July 29, 2026"));
    let before = outer.with(|list| list.scroll_controller().child_position());
    assert!(
        before.y < 0.0,
        "the outside button is above the page viewport"
    );
    let day = bounds(&renderer, "July 29, 2026");
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(
            ((day.x0 + day.x1) / 2.0) as f32,
            ((day.y0 + day.y1) / 2.0) as f32,
        ),
        0.0,
        -600.0,
    );
    renderer.render_frames(90);
    let accessibility = renderer.accessibility_snapshot().expect("tree");
    let focused = accessibility
        .nodes
        .iter()
        .find(|(id, _)| *id == accessibility.focus)
        .expect("restored focus");
    assert_eq!(
        focused.1.label(),
        Some("Date range calendar"),
        "virtualizing a focused month restores focus to the stable calendar viewport"
    );
    assert_ne!(
        state.with(DateRangePickerState::displayed_month),
        YearMonth::new(2026, 7).unwrap(),
        "the calendar must actually advance beyond the selected month"
    );
    assert!(
        renderer
            .accessibility_snapshot()
            .expect("tree")
            .nodes
            .iter()
            .all(|(_, node)| node.label() != Some("July 29, 2026")),
        "the previously focused month must be virtualized"
    );
    assert_eq!(
        outer.with(|list| list.scroll_controller().child_position()),
        before,
        "virtualizing the focused month must not reveal controls outside the calendar"
    );
    assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
    renderer.render_frames(8);
    let accessibility = renderer.accessibility_snapshot().expect("tree");
    let focused = accessibility
        .nodes
        .iter()
        .find(|(id, _)| *id == accessibility.focus)
        .expect("focused visible date");
    assert_eq!(focused.1.role(), Role::Button);
    assert_eq!(focused.1.label(), Some("August 1, 2026"));
    assert_eq!(
        outer.with(|list| list.scroll_controller().child_position()),
        before,
        "navigation from the viewport enters its calendar dates without moving the page"
    );
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29))
    );
    assert_eq!(state.with(DateRangePickerState::selected_end_date), None);
    assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
    renderer.render_frames(12);
    let accessibility = renderer.accessibility_snapshot().expect("tree");
    assert_eq!(
        accessibility
            .nodes
            .iter()
            .find(|(id, _)| *id == accessibility.focus)
            .and_then(|(_, node)| node.label()),
        Some("After calendar"),
        "Tab navigation leaves the calendar after its single date stop"
    );
}

#[tessera]
fn fixture() {
    let blocked = CallbackWith::new(|date: CalendarDate| date != self::date(7, 13));
    let state = remember(|| {
        DateRangePickerState::new(
            None,
            None,
            YearMonth::new(2026, 7),
            1900..=2100,
            Some(blocked),
            None,
            DatePickerDisplayMode::Picker,
        )
    });
    STATE.set(Some(state));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.primary = Color::new(0.05, 0.15, 0.8, 1.0);
            theme.color_scheme.secondary_container = Color::new(0.3, 0.6, 0.3, 1.0);
            theme.color_scheme.surface_container_high = Color::WHITE;
            theme
        })
        .child(move || {
            surface()
                .style(SurfaceStyle::Filled {
                    color: Color::WHITE,
                })
                .modifier(Modifier::new().fill_max_size())
                .child(move || {
                    date_range_picker()
                        .state(state)
                        .modifier(Modifier::new().offset(Dp(20.0), Dp(20.0)))
                        .first_day_of_week(Weekday::Sunday);
                });
        });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.role() != Role::Label)
        .unwrap_or_else(|| panic!("missing {label}"))
        .1
        .bounds()
        .expect("bounds")
}

fn click_bounds(renderer: &mut HeadlessRenderer, bounds: Rect) {
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(3);
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = bounds(renderer, label);
    click_bounds(renderer, bounds);
}

fn input_bounds(renderer: &HeadlessRenderer) -> Vec<Rect> {
    let nodes = renderer.accessibility_snapshot().expect("tree").nodes;
    assert_eq!(
        nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::TextInput)
            .count(),
        2
    );
    let input = |label| {
        nodes
            .iter()
            .find(|(_, node)| node.role() == Role::TextInput && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing named input {label}"))
            .1
            .bounds()
            .expect("input bounds")
    };
    let start = input("Start date");
    let end = input("End date");
    assert!(start.height() <= 56.0 && end.height() <= 56.0);
    assert!(
        start.x0 < end.x0,
        "start and end inputs retain their reading order"
    );
    vec![start, end]
}

fn assert_color(renderer: &HeadlessRenderer, x: f64, y: f64, expected: [u8; 3]) {
    let offset = (y as usize * renderer.physical_size().width as usize + x as usize) * 4;
    let pixels = renderer.read_pixels().expect("pixels");
    for channel in 0..3 {
        assert!(
            (pixels[offset + channel] as i32 - expected[channel] as i32).abs() <= 2,
            "pixel ({x}, {y}) = {:?}, expected {expected:?}",
            &pixels[offset..offset + 4]
        );
    }
}

fn check_range(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| fixture()),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 950,
            scale_factor: 1.5,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(8);
    let state = STATE.get().expect("state");
    let month = bounds(&renderer, "July 2026");
    let heading = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Label && node.label() == Some("July 2026"))
        .expect("month heading")
        .1
        .bounds()
        .expect("heading bounds");
    assert!(
        (heading.x0 - month.x0 - 36.0).abs() <= 1.0,
        "24dp month-heading inset"
    );
    assert!(
        (heading.y0 - month.y0 - 30.0).abs() <= 1.0,
        "20dp month-heading top space"
    );
    let weekdays = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .filter(|(_, node)| node.role() == Role::Label && node.label() == Some("S"))
        .map(|(_, node)| node.bounds().expect("weekday bounds"))
        .collect::<Vec<_>>();
    assert_eq!(weekdays.len(), 2);
    for (weekday, date) in weekdays.iter().zip(["July 26, 2026", "July 25, 2026"]) {
        let day = bounds(&renderer, date);
        assert!(
            (day.x1 - day.x0 - 72.0).abs() <= 1.0,
            "48dp date target width"
        );
        assert!(
            ((weekday.x0 + weekday.x1) / 2.0 - (day.x0 + day.x1) / 2.0).abs() <= 1.0,
            "weekday and date columns align"
        );
    }
    let day = bounds(&renderer, "July 12, 2026");
    let x = day.x0 as usize + 12;
    let y = ((day.y0 + day.y1) / 2.0) as usize;
    let offset = (y * renderer.physical_size().width as usize + x) * 4;
    let idle = renderer.read_pixels().expect("idle date");
    renderer.pointer_move(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(x as f32, y as f32),
    );
    renderer.render_frames(3);
    let hovered = renderer.read_pixels().expect("hovered date");
    assert_ne!(
        &idle[offset..offset + 4],
        &hovered[offset..offset + 4],
        "hover paints the date's state layer"
    );
    assert_color(&renderer, day.x0 + 2.0, y as f64, [255, 255, 255]);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(590.0, 940.0));
    renderer.render_frames(3);
    assert_eq!(
        &renderer.read_pixels().expect("idle again")[offset..offset + 4],
        &idle[offset..offset + 4],
        "leaving the date clears its state layer"
    );
    assert!(
        renderer.accessibility_snapshot().expect("tree").nodes.len() < 1200,
        "only visible months should be built"
    );
    let labels = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .take(35)
        .collect::<Vec<_>>();
    let blocked = renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some("July 13, 2026") && node.role() != Role::Label)
        .unwrap_or_else(|| {
            panic!(
                "missing July date; displayed {:?}; labels {labels:?}",
                state.with(DateRangePickerState::displayed_month)
            )
        })
        .1;
    assert!(blocked.is_disabled());
    click(&mut renderer, "July 13, 2026");
    assert_eq!(state.with(DateRangePickerState::selected_start_date), None);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(590.0, 940.0));
    renderer.render_frames(3);
    let blocked = bounds(&renderer, "July 13, 2026");
    let focus_x = (blocked.x0 + blocked.x1) / 2.0 + 20.0;
    let focus_y = (blocked.y0 + blocked.y1) / 2.0;
    assert_color(&renderer, focus_x, focus_y, [255, 255, 255]);
    crate::date_picker::navigation::test_keyboard().request(date(7, 13));
    renderer.render_frames(8);
    let accessibility = renderer.accessibility_snapshot().expect("tree");
    let focused = accessibility
        .nodes
        .iter()
        .find(|(id, _)| *id == accessibility.focus)
        .expect("focused unavailable date");
    assert_eq!(focused.1.label(), Some("July 13, 2026"));
    assert!(focused.1.is_disabled());
    let pixels = renderer.read_pixels().expect("focused pixels");
    let offset =
        (focus_y as usize * renderer.physical_size().width as usize + focus_x as usize) * 4;
    assert!(
        pixels[offset..offset + 3]
            .iter()
            .any(|channel| *channel < 250),
        "an unavailable date still shows a visible keyboard focus indicator"
    );
    assert_eq!(state.with(DateRangePickerState::selected_start_date), None);
    FocusManager::current().clear_focus(true);
    renderer.render_frames(3);
    assert_color(&renderer, focus_x, focus_y, [255, 255, 255]);
    let start = bounds(&renderer, "July 29, 2026");
    let corner = Rect::new(
        start.x0 + 1.0,
        start.y0 + 1.0,
        start.x0 + 3.0,
        start.y0 + 3.0,
    );
    click_bounds(&mut renderer, corner);
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29)),
        "the 48dp target includes the area outside the 40dp visual circle"
    );
    click(&mut renderer, "July 31, 2026");
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29))
    );
    assert_eq!(
        state.with(DateRangePickerState::selected_end_date),
        Some(date(7, 31))
    );
    FocusManager::current().clear_focus(true);
    renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(590.0, 940.0));
    renderer.render_frames(90);
    let start = bounds(&renderer, "July 29, 2026");
    let middle = bounds(&renderer, "July 30, 2026");
    let end = bounds(&renderer, "July 31, 2026");
    assert!((start.y1 - start.y0 - 72.0).abs() <= 1.0);
    let y = (middle.y0 + middle.y1) / 2.0;
    assert_color(&renderer, middle.x0 + 2.0, y, [149, 203, 149]);
    assert_color(&renderer, start.x0 + 2.0, y, [255, 255, 255]);
    assert_color(&renderer, start.x1 - 2.0, y, [149, 203, 149]);
    assert_color(&renderer, end.x0 + 2.0, y, [149, 203, 149]);
    assert_color(&renderer, end.x1 - 2.0, y, [255, 255, 255]);
    let selected = renderer.read_pixels().expect("selected");
    renderer.resize(450, 950);
    renderer.render_frames(8);
    let narrow = bounds(&renderer, "July 30, 2026");
    assert!(narrow.x1 <= 450.0 && narrow.x0 >= 0.0);
    renderer.resize(1, 950);
    renderer.render_frames(4);
    renderer.resize(600, 950);
    renderer.render_frames(8);
    assert!(
        renderer.read_pixels().expect("restored") == selected,
        "resize restores all pixels"
    );

    state.with_mut(|state| state.set_selection(Some(date(7, 30)), Some(date(7, 30))));
    renderer.render_frames(8);
    let day = bounds(&renderer, "July 30, 2026");
    let y = (day.y0 + day.y1) / 2.0;
    assert_color(&renderer, day.x0 + 2.0, y, [255, 255, 255]);
    assert_color(&renderer, day.x1 - 2.0, y, [255, 255, 255]);

    state.with_mut(|state| state.set_displayed_month(YearMonth::new(2027, 1).unwrap()));
    renderer.render_frames(8);
    bounds(&renderer, "January 13, 2027");
    state.with_mut(|state| state.set_year_range(2028..=2028));
    renderer.render_frames(8);
    bounds(&renderer, "January 13, 2028");
    state.with_mut(|state| {
        state.set_year_range(1900..=2100);
        state.set_displayed_month(YearMonth::new(2026, 7).unwrap());
    });
    renderer.render_frames(8);

    state.with_mut(DateRangePickerState::clear_selection);
    click(&mut renderer, "July 29, 2026");
    let point = PxPosition::from_f32_xy(middle.x0 as f32 + 8.0, y as f32);
    renderer.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, -504.0);
    renderer.render_frames(90);
    click(&mut renderer, "August 3, 2026");
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29))
    );
    assert_eq!(
        state.with(DateRangePickerState::selected_end_date),
        Some(date(8, 3))
    );
    assert_eq!(
        state.with(DateRangePickerState::displayed_month),
        YearMonth::new(2026, 8).unwrap()
    );
    let august = bounds(&renderer, "August 3, 2026");
    renderer.pointer_scroll(MOUSE_POINTER_ID, point, 0.0, -72.0);
    renderer.render_frames(8);
    assert!(bounds(&renderer, "August 3, 2026").y0 < august.y0);
    state.with_mut(|state| state.set_displayed_month(YearMonth::new(2026, 8).unwrap()));
    renderer.render_frames(8);
    assert_eq!(
        bounds(&renderer, "August 3, 2026"),
        august,
        "requesting the current month restores its start"
    );

    state.with_mut(|state| {
        state.clear_selection();
        state.set_display_mode(DatePickerDisplayMode::Input);
    });
    renderer.render_frames(5);
    let inputs = input_bounds(&renderer);
    click_bounds(&mut renderer, inputs[1]);
    renderer.inject_text("08/03/2026");
    renderer.render_frames(4);
    assert!(!state.with(DateRangePickerState::is_complete));
    click_bounds(&mut renderer, inputs[0]);
    renderer.inject_text("07/29/2026");
    renderer.render_frames(4);
    assert!(state.with(DateRangePickerState::is_complete));
    assert_eq!(
        state.with(DateRangePickerState::selected_end_date),
        Some(date(8, 3))
    );
    state.with_mut(DateRangePickerState::clear_selection);
    renderer.render_frames(4);
    click_bounds(&mut renderer, inputs[0]);
    renderer.inject_text("07/13/2026");
    renderer.render_frames(4);
    assert_eq!(state.with(DateRangePickerState::selected_start_date), None);
    state.with_mut(DateRangePickerState::clear_selection);
    renderer.render_frames(4);
    let start_input = input_bounds(&renderer)[0];
    click_bounds(&mut renderer, start_input);
    renderer.inject_text("07/29/2026");
    renderer.render_frames(4);
    assert_eq!(
        state.with(DateRangePickerState::selected_start_date),
        Some(date(7, 29)),
        "clearing invalid drafts resets both editors"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_date_range_nested_scroll_preserves_outer_position() {
    crate::test_utils::run_isolated_test(
        "date_picker::range::tests::gpu_date_range_nested_scroll_preserves_outer_position",
        || check_nested_scroll(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_date_range_nested_scroll_preserves_outer_position_msaa() {
    crate::test_utils::run_isolated_test(
        "date_picker::range::tests::gpu_date_range_nested_scroll_preserves_outer_position_msaa",
        || check_nested_scroll(4),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_date_range_selection_scrolling_input_and_resize() {
    check_range(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_date_range_selection_scrolling_input_and_resize_msaa() {
    check_range(4);
}
