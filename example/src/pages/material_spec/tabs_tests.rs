use tessera_ui::{
    MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Node, Rect as AccessRect},
    testing::UiTest,
};

use super::*;

fn renderer(
    selection: Option<tessera_ui::testing::TestProbe<Option<State<usize>>>>,
) -> UiTest<'static> {
    let mut renderer = UiTest::new()
        .viewport(600, 480)
        .rendered(crate::TestPipelines)
        .content(move || {
            material_theme().child({
                let selection = selection.clone();
                move || {
                    boxed().modifier(Modifier::new().fill_max_size()).children({
                        let selection = selection.clone();
                        move || match &selection {
                            Some(selection) => {
                                let selected = remember(|| 0usize);
                                selection.set(Some(selected));
                                swipe_tabs(selected);
                            }
                            None => {
                                native_tabs_figure(true, true, false, 0, 3);
                            }
                        }
                    });
                }
            })
        });
    renderer.frames(8);
    renderer
}

fn node(renderer: &UiTest, matches: impl Fn(&Node) -> bool) -> Option<Node> {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .map(|(_, node)| node)
        .find(|node| matches(node))
}

fn tab_bounds(renderer: &UiTest, label: &str) -> AccessRect {
    node(renderer, |node| {
        node.role() == Role::Tab && crate::has_label(&node, label)
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

fn click(renderer: &mut UiTest, position: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.frames(2);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(2);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.frames(8);
}

fn swipe(renderer: &mut UiTest, from: PxPosition, dx: f32) {
    renderer.pointer_move(MOUSE_POINTER_ID, from);
    renderer.frames(2);
    renderer.pointer_press(MOUSE_POINTER_ID, from, PressKeyEventType::Left);
    renderer.frames(1);
    let mut last = from;
    for step in 1..=8 {
        last = PxPosition::from_f32_xy(from.x + dx * step as f32 / 8.0, from.y);
        renderer.pointer_move(MOUSE_POINTER_ID, last);
        renderer.frames(1);
    }
    renderer.pointer_release(MOUSE_POINTER_ID, last, PressKeyEventType::Left);
    renderer.frames(8);
}

fn panel_center(renderer: &UiTest) -> PxPosition {
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
    let selection = tessera_ui::testing::TestProbe::default();
    let observed = selection.clone();
    let mut renderer = renderer(Some(observed));
    let selected = selection.get().expect("mounted swipe tabs");
    assert_eq!(selected.get(), 0);

    let start = panel_center(&renderer);
    swipe(&mut renderer, start, -200.0);
    assert_eq!(selected.get(), 1, "a leftward swipe opens the next tab");
    assert!(
        node(&renderer, |node| crate::has_label(&node, "Trips content")).is_some(),
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
    let mut renderer = renderer(None);
    let badged = center(tab_bounds(&renderer, "Flights, 3 new items"));
    click(&mut renderer, badged);
    assert!(
        node(&renderer, |node| node.role() == Role::Tab
            && crate::has_label(&node, "Flights, 3 new items"))
        .is_none(),
        "the badge count is removed once the tab has been viewed"
    );
    tab_bounds(&renderer, "Flights");
}
