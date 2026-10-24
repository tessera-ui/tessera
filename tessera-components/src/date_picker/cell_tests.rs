//! Authored calendar cell interaction regressions.
//!
//! ## Usage
//!
//! Verify real dates, selection, focus, and disabled calendar choices.

use std::cell::Cell;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    RenderModule,
    accesskit::{Action, ActionRequest, Node, NodeId, TreeId},
};

use crate::{column::column, row::row, theme::material_theme};

use super::*;

#[derive(Clone)]
struct Selection {
    day: CalendarDate,
    year: i32,
    actions: usize,
}

thread_local! {
    static SELECTION: Cell<Option<State<Selection>>> = const { Cell::new(None) };
}

struct Pipelines;
impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    let selection = remember(|| Selection {
        day: CalendarDate::new(2025, 8, 17).unwrap(),
        year: 2025,
        actions: 0,
    });
    SELECTION.set(Some(selection));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().padding_all(Dp(16.0)))
            .children(move || {
                row()
                    .modifier(Modifier::new().height(Dp(112.0)))
                    .children(move || {
                        for day in [17, 18, 19, 20] {
                            let date = CalendarDate::new(2025, 8, day).unwrap();
                            date_picker_day(date)
                                .selected(selection.with(|selection| selection.day == date))
                                .enabled(day != 19)
                                .modifier(if day == 18 {
                                    Modifier::new().offset(Dp(32.0), Dp(56.0))
                                } else {
                                    Modifier::new()
                                })
                                .presentation_state_optional(
                                    (day == 19 || day == 20)
                                        .then_some(InteractionVisualState::Focus),
                                )
                                .label_formatter(move |date| {
                                    if date.day() == 17 {
                                        "00".into()
                                    } else {
                                        date.day().to_string()
                                    }
                                })
                                .on_select(move |date| {
                                    selection.with_mut(|selection| {
                                        selection.day = date;
                                        selection.actions += 1;
                                    })
                                });
                        }
                    });
                row().children(move || {
                    for year in [2025, 2026] {
                        date_picker_year(year)
                            .selected(selection.with(|selection| selection.year == year))
                            .enabled(year != 2026)
                            .on_select(move |year| {
                                selection.with_mut(|selection| {
                                    selection.year = year;
                                    selection.actions += 1;
                                })
                            });
                    }
                });
            });
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.label() == Some(label) && node.role() == tessera_ui::accesskit::Role::Button
        })
        .unwrap_or_else(|| panic!("missing actual calendar cell {label}"))
}

fn action(renderer: &mut HeadlessRenderer, id: NodeId, action: Action) -> bool {
    let result = renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: id,
        data: None,
    });
    renderer.render_frames(4);
    result
}

fn pointer_click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = node(renderer, label).1.bounds().unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
}

#[test]
#[ignore = "requires a native GPU adapter for authored calendar cells"]
fn authored_cells_keep_actual_dates_callbacks_focus_and_disabled_state() {
    crate::test_utils::run_isolated_test(
        "date_picker::cell::tests::authored_cells_keep_actual_dates_callbacks_focus_and_disabled_state",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(fixture),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 400,
                    height: 200,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(5);
            let (first, day) = node(&renderer, "August 17, 2025");
            assert_eq!(day.bounds().unwrap().width(), 48.0);
            assert_eq!(day.bounds().unwrap().height(), 48.0);
            assert_eq!(day.is_selected(), Some(true));
            let published = renderer.accessibility_snapshot().unwrap();
            let captions: Vec<_> = published
                .nodes
                .iter()
                .filter(|(_, node)| node.label() == Some("00"))
                .map(|(id, node)| {
                    let parents: Vec<_> = published
                        .nodes
                        .iter()
                        .filter(|(_, parent)| parent.children().contains(id))
                        .map(|(id, parent)| (*id, parent.role(), parent.label()))
                        .collect();
                    (*id, node.role(), node.is_hidden(), parents)
                })
                .collect();
            assert!(
                published
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.label() == Some("00"))
                    .all(|(_, node)| node.is_hidden()
                        && !node.supports_action(Action::Click)
                        && !node.supports_action(Action::Focus)),
                "visual captions must remain hidden and non-actionable beside the actual accessible date: {captions:?}"
            );
            let raw = renderer.snapshot();
            let mut pending = vec![&raw["root"]];
            let mut caption = false;
            while let Some(node) = pending.pop() {
                if node["accessibility"]["label"].as_str() == Some("00") {
                    caption = node["bounds"]["width"]
                        .as_f64()
                        .is_some_and(|width| width > 0.0)
                        && node["bounds"]["height"]
                            .as_f64()
                            .is_some_and(|height| height > 0.0);
                }
                if let Some(children) = node["children"].as_array() {
                    pending.extend(children);
                }
            }
            assert!(
                caption,
                "the native text child must carry the visible formatted day"
            );
            let moved = node(&renderer, "August 18, 2025").1.bounds().unwrap();
            assert_eq!(
                (moved.x0, moved.y0, moved.width(), moved.height()),
                (96.0, 72.0, 48.0, 48.0)
            );
            let preview = node(&renderer, "August 20, 2025").0;
            assert_ne!(renderer.accessibility_snapshot().unwrap().focus, preview);
            let preview_bounds = node(&renderer, "August 20, 2025").1.bounds().unwrap();
            let root_focus = renderer.accessibility_snapshot().unwrap().focus;
            let preview_pixels = renderer.read_pixels().unwrap();
            let preview_ring = (40 * 400 + preview_bounds.x0 as usize) * 4;
            renderer.pointer_move(
                MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(
                    (preview_bounds.x0 + 24.0) as f32,
                    (preview_bounds.y0 + 24.0) as f32,
                ),
            );
            renderer.render_frames(4);
            let hover_pixels = renderer.read_pixels().unwrap();
            assert_ne!(
                &preview_pixels[preview_ring..preview_ring + 3],
                &hover_pixels[preview_ring..preview_ring + 3],
                "live hover must override the focused presentation outline"
            );
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, root_focus);
            renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(380.0, 180.0));
            renderer.render_frames(4);
            let actions = SELECTION.get().unwrap().with(|selection| selection.actions);
            let before = renderer.read_pixels().unwrap();
            assert!(action(&mut renderer, first, Action::Focus));
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, first);
            assert_eq!(
                SELECTION.get().unwrap().with(|selection| selection.actions),
                actions
            );
            let after = renderer.read_pixels().unwrap();
            let left_ring = ((40 * 400 + 16) * 4)..((40 * 400 + 18) * 4);
            assert_ne!(&before[left_ring.clone()], &after[left_ring]);
            pointer_click(&mut renderer, "August 18, 2025");
            assert_eq!(
                SELECTION.get().unwrap().with(|selection| selection.day),
                CalendarDate::new(2025, 8, 18).unwrap()
            );
            assert_eq!(
                node(&renderer, "August 17, 2025").1.is_selected(),
                Some(false)
            );
            assert_eq!(
                node(&renderer, "August 18, 2025").1.is_selected(),
                Some(true)
            );
            let disabled = node(&renderer, "August 19, 2025");
            assert!(disabled.1.is_disabled());
            let count = SELECTION.get().unwrap().with(|selection| selection.actions);
            assert!(!action(&mut renderer, disabled.0, Action::Click));
            assert_eq!(
                SELECTION.get().unwrap().with(|selection| selection.actions),
                count
            );
            let (year, choice) = node(&renderer, "2025");
            assert_eq!(choice.bounds().unwrap().width(), 112.0);
            assert_eq!(choice.bounds().unwrap().height(), 52.0);
            assert!(action(&mut renderer, year, Action::Focus));
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, year);
            assert_eq!(
                SELECTION.get().unwrap().with(|selection| selection.actions),
                count
            );
            let disabled_year = node(&renderer, "2026");
            assert!(disabled_year.1.is_disabled());
            assert!(!action(&mut renderer, disabled_year.0, Action::Click));
            assert_eq!(
                SELECTION.get().unwrap().with(|selection| selection.actions),
                count
            );
        },
    );
}
