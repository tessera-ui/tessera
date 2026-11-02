//! Tabs article interaction regressions.
//!
//! ## Usage
//!
//! Verify swipeable fixed tabs and badge dismissal through real pointer input.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Rect as AccessRect},
};

use super::*;

thread_local! {
    static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn swipe_fixture() {
    let selected = remember(|| 0usize);
    SELECTED.set(Some(selected));
    material_theme().child(move || {
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                swipe_tabs().selected(selected);
            });
    });
}

#[tessera]
fn badge_fixture() {
    material_theme().child(|| {
        boxed()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                native_tabs_figure(true, true, false, 0, 3);
            });
    });
}

fn renderer<Output: 'static>(fixture: fn() -> Output) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 600,
            height: 480,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    renderer
}

fn node(renderer: &HeadlessRenderer, matches: impl Fn(&Node) -> bool) -> Option<Node> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .map(|(_, node)| node)
        .find(|node| matches(node))
}

fn tab_bounds(renderer: &HeadlessRenderer, label: &str) -> AccessRect {
    node(renderer, |node| {
        node.role() == Role::Tab && node.label() == Some(label)
    })
    .unwrap_or_else(|| panic!("tab {label} must be published"))
    .bounds()
    .unwrap()
}

fn center(bounds: AccessRect) -> PxPosition {
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    )
}

fn click(renderer: &mut HeadlessRenderer, position: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.render_frames(2);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(8);
}

fn swipe(renderer: &mut HeadlessRenderer, from: PxPosition, dx: f32) {
    renderer.pointer_move(MOUSE_POINTER_ID, from);
    renderer.render_frames(2);
    renderer.pointer_press(MOUSE_POINTER_ID, from, PressKeyEventType::Left);
    renderer.render_frames(1);
    let mut last = from;
    for step in 1..=8 {
        last = PxPosition::from_f32_xy(from.x + dx * step as f32 / 8.0, from.y);
        renderer.pointer_move(MOUSE_POINTER_ID, last);
        renderer.render_frames(1);
    }
    renderer.pointer_release(MOUSE_POINTER_ID, last, PressKeyEventType::Left);
    renderer.render_frames(8);
}

fn panel_center(renderer: &HeadlessRenderer) -> PxPosition {
    center(
        node(renderer, |node| node.role() == Role::TabPanel)
            .expect("swipe content panel must be published")
            .bounds()
            .unwrap(),
    )
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn swiping_fixed_tab_content_moves_selection_between_neighbours() {
    let mut renderer = renderer(swipe_fixture);
    let selected = SELECTED.get().unwrap();
    assert_eq!(selected.get(), 0);

    let start = panel_center(&renderer);
    swipe(&mut renderer, start, -200.0);
    assert_eq!(selected.get(), 1, "a leftward swipe opens the next tab");
    assert!(
        node(&renderer, |node| node.label() == Some("Trips content")).is_some(),
        "the content panel follows the selected tab"
    );

    let from = panel_center(&renderer);
    swipe(&mut renderer, from, 20.0);
    assert_eq!(
        selected.get(),
        1,
        "a short drag below the threshold keeps the tab"
    );

    let from = panel_center(&renderer);
    swipe(&mut renderer, from, 200.0);
    assert_eq!(
        selected.get(),
        0,
        "a rightward swipe returns to the previous tab"
    );

    let from = panel_center(&renderer);
    swipe(&mut renderer, from, 200.0);
    assert_eq!(
        selected.get(),
        0,
        "swiping past the first tab does not wrap"
    );

    let explore = center(tab_bounds(&renderer, "Explore"));
    click(&mut renderer, explore);
    assert_eq!(selected.get(), 2, "tapping a tab selects it directly");
    let from = panel_center(&renderer);
    swipe(&mut renderer, from, -200.0);
    assert_eq!(selected.get(), 2, "swiping past the last tab does not wrap");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn viewing_badged_tab_removes_badge_from_its_accessibility_label() {
    let mut renderer = renderer(badge_fixture);
    let badged = center(tab_bounds(&renderer, "Flights, 3 new items"));
    click(&mut renderer, badged);
    assert!(
        node(&renderer, |node| node.role() == Role::Tab
            && node.label() == Some("Flights, 3 new items"))
        .is_none(),
        "the badge count is removed once the tab has been viewed"
    );
    tab_bounds(&renderer, "Flights");
}
