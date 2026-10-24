//! Native calendar focus indicator regressions.
//!
//! ## Usage
//!
//! Keep day and year focus visible without activating their selection
//! callbacks.

use std::cell::Cell;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, RenderModule,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
};

use crate::{column::column, theme::material_theme};

use super::*;

thread_local! {
    static YEAR: Cell<bool> = const { Cell::new(false) };
    static STATE: Cell<Option<State<DatePickerState>>> = const { Cell::new(None) };
    static VIEW: Cell<Option<State<DatePickerView>>> = const { Cell::new(None) };
}

struct Pipelines;
impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    let state = remember(|| {
        DatePickerState::new(
            CalendarDate::new(2025, 8, 17),
            YearMonth::new(2025, 8),
            2021..=2100,
            None,
            None,
            DatePickerDisplayMode::Picker,
        )
    });
    let view = remember(|| {
        if YEAR.get() {
            DatePickerView::Year
        } else {
            DatePickerView::Calendar
        }
    });
    STATE.set(Some(state));
    VIEW.set(Some(view));
    material_theme().child(move || {
        column().children(move || {
            date_picker()
                .state(state)
                .view(view)
                .today(CalendarDate::new(2025, 8, 5).unwrap());
        });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label) && node.supports_action(Action::Click))
        .unwrap_or_else(|| panic!("missing calendar choice {label}"))
}

fn action(renderer: &mut HeadlessRenderer, id: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    }));
    renderer.render_frames(4);
}

fn focus_outline_changes(
    before: &[u8],
    after: &[u8],
    width: usize,
    body: tessera_ui::accesskit::Rect,
) -> usize {
    let mut changed = 0;
    for y in (body.y0 - 5.0).floor().max(0.0) as usize..(body.y1 + 5.0).ceil() as usize {
        for x in (body.x0 - 5.0).floor().max(0.0) as usize..(body.x1 + 5.0).ceil() as usize {
            if (x as f64) >= body.x0
                && (x as f64) < body.x1
                && (y as f64) >= body.y0
                && (y as f64) < body.y1
            {
                continue;
            }
            let offset = (y * width + x) * 4;
            changed +=
                usize::from((0..3).any(|channel| {
                    before[offset + channel].abs_diff(after[offset + channel]) > 15
                }));
        }
    }
    changed
}

fn check_focus(year: bool) {
    YEAR.set(year);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 400,
            height: 650,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(5);
    let label = if year { "2025" } else { "August 17, 2025" };
    let (id, choice) = node(&renderer, label);
    let bounds = choice.bounds().unwrap();
    let (inset_x, inset_y) = if year { (20.0, 8.0) } else { (4.0, 4.0) };
    let body = tessera_ui::accesskit::Rect {
        x0: bounds.x0 + inset_x,
        y0: bounds.y0 + inset_y,
        x1: bounds.x1 - inset_x,
        y1: bounds.y1 - inset_y,
    };
    assert_eq!(body.width(), if year { 72.0 } else { 40.0 });
    assert_eq!(body.height(), if year { 36.0 } else { 40.0 });
    assert_ne!(
        renderer.accessibility_snapshot().unwrap().focus,
        id,
        "the fixture must start without focus on the target choice"
    );
    let before = renderer.read_pixels().unwrap();
    let selected = STATE.get().unwrap().with(DatePickerState::selected_date);
    action(&mut renderer, id, Action::Focus);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, id);
    assert_eq!(
        STATE.get().unwrap().with(DatePickerState::selected_date),
        selected
    );
    assert_eq!(
        VIEW.get().unwrap().get(),
        if year {
            DatePickerView::Year
        } else {
            DatePickerView::Calendar
        }
    );
    assert_eq!(node(&renderer, label).1.bounds(), Some(bounds));
    let after = renderer.read_pixels().unwrap();
    let changed = focus_outline_changes(&before, &after, 400, body);
    assert!(
        changed > 40,
        "actual AT focus must paint a distinct outline outside the visual body"
    );
    let other = node(&renderer, if year { "2026" } else { "August 19, 2025" }).0;
    action(&mut renderer, other, Action::Click);
    if year {
        assert_eq!(
            STATE
                .get()
                .unwrap()
                .with(|state| state.displayed_month().year()),
            2026
        );
        assert_eq!(VIEW.get().unwrap().get(), DatePickerView::Calendar);
        assert_eq!(
            STATE.get().unwrap().with(DatePickerState::selected_date),
            selected
        );
    } else {
        assert_eq!(
            STATE.get().unwrap().with(DatePickerState::selected_date),
            CalendarDate::new(2025, 8, 19)
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter for actual day and year focus outlines"]
fn native_calendar_focus_paints_day_and_year_outline_without_selecting() {
    crate::test_utils::run_isolated_test(
        "date_picker::focus_tests::native_calendar_focus_paints_day_and_year_outline_without_selecting",
        || {
            check_focus(false);
            check_focus(true);
        },
    );
}
