//! Native lazy item retention and index navigation regressions.
//!
//! ## Usage
//!
//! Preserve interactive item state while navigating unequal-height lists.

use std::cell::Cell;

use tessera_ui::{
    FocusDirection, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{boxed::boxed, checkbox::checkbox, modifier::ModifierExt as _, theme::material_theme};

use super::*;

thread_local! {
    static LIST: Cell<Option<State<LazyListController>>> = const { Cell::new(None) };
    static KEEP: Cell<bool> = const { Cell::new(true) };
    static MOUNTED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static COUNT: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static NAVIGATION: Cell<Option<State<Option<FocusDirection>>>> = const { Cell::new(None) };
    static NAVIGATION_ACCEPTED: Cell<bool> = const { Cell::new(false) };
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn retained_option(index: usize) {
    let checked = remember(|| false);
    boxed()
        .modifier(
            Modifier::new()
                .width(Dp(180.0))
                .height(Dp([80.0, 240.0, 56.0, 360.0][index % 4])),
        )
        .children(move || {
            checkbox()
                .checked(checked.get())
                .accessibility_label(format!("Retained option {index}"))
                .on_toggle(move |value| checked.set(value));
        });
}

#[tessera]
fn fixture() {
    let controller = remember(LazyListController::new);
    let viewport = remember(ScrollableController::new);
    let mounted = remember(|| true);
    let count = remember(|| 12_usize);
    COUNT.set(Some(count));
    let navigation = remember(|| None::<FocusDirection>);
    NAVIGATION.set(Some(navigation));
    if let Some(direction) = navigation.get() {
        navigation.set(None);
        NAVIGATION_ACCEPTED.set(FocusManager::current().request_move_focus(direction));
    }
    LIST.set(Some(controller));
    MOUNTED.set(Some(mounted));
    let items = RenderSlotWith::new(|index| {
        retained_option(index);
    });
    material_theme().child(move || {
        if !mounted.get() {
            return;
        }
        boxed()
            .alignment(crate::alignment::Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                let mut list = lazy_column()
                    .modifier(Modifier::new().size(Dp(180.0), Dp(200.0)))
                    .controller(controller)
                    .viewport_controller(viewport)
                    .estimated_item_size(Dp(50.0))
                    .max_viewport_main(Px(200))
                    .scroll_smoothing(0.0)
                    .scrollbar_behavior(ScrollBarBehavior::Hidden);
                if KEEP.get() {
                    list = list.keep_composed_items(true);
                }
                list.items_with_key_shared(count.get(), |index| index, items);
            });
    });
}

fn node(renderer: &HeadlessRenderer, index: usize) -> (tessera_ui::accesskit::NodeId, Node) {
    let label = format!("Retained option {index}");
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::CheckBox && node.label() == Some(label.as_str()))
        .unwrap_or_else(|| panic!("missing actual checkbox {label}"))
}

#[test]
#[ignore = "requires a native GPU adapter for lazy focus and checkbox interaction"]
fn retained_items_keep_real_checkbox_state_and_focus_through_cold_index_navigation_and_wheel() {
    KEEP.set(true);
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 200,
            height: 220,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    let first = node(&renderer, 0).0;
    let point = PxPosition::from_f32_xy(24.0, 24.0);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::True));
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, first);
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(8, Dp(20.0)));
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 8).1.bounds().unwrap().y0, 20.0);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::True));
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, first);
    assert_eq!(node(&renderer, 8).1.toggled(), Some(Toggled::False));
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(9, Dp(20.0)));
    renderer.pointer_scroll(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(140.0, 100.0),
        0.0,
        -30.0,
    );
    renderer.render_frames(8);
    assert_eq!(
        node(&renderer, 9).1.bounds().unwrap().y0,
        -10.0,
        "later wheel input must survive target measurement correction"
    );
    let ninth = node(&renderer, 9).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: ninth,
        data: None,
    }));
    renderer.render_frames(8);
    assert_eq!(renderer.accessibility_snapshot().unwrap().focus, ninth);
    let bounds = node(&renderer, 9).1.bounds().unwrap();
    assert!(
        bounds.y0 >= 0.0 && bounds.y1 <= 200.0,
        "newly focused clipped item must reveal inside the viewport: {bounds:?}"
    );
    assert_eq!(
        node(&renderer, 9).1.toggled(),
        Some(Toggled::False),
        "focus reveal cannot activate the checkbox"
    );
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(0, Dp::ZERO));
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).0, first);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::True));
    let point = PxPosition::from_f32_xy(24.0, 104.0);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(8);
    assert_eq!(
        renderer.accessibility_snapshot().unwrap().focus,
        node(&renderer, 1).0
    );
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(8, Dp(20.0)));
    renderer.render_frames(8);
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Retained option 0")),
        "An unfocused retained item must be unplaced and absent from accessibility"
    );
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(0, Dp::ZERO));
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).0, first);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::True));
}

fn make_renderer() -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width: 200,
            height: 220,
            sample_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(8);
    assert_eq!(
        LIST.get().unwrap().with(|list| {
            list.viewport
                .unwrap()
                .with(|viewport| viewport.visible_size().height)
        }),
        Px(200),
        "The fixture must exercise the actual 200px viewport"
    );
    renderer
}

fn toggle_first(renderer: &mut HeadlessRenderer) {
    let point = PxPosition::from_f32_xy(24.0, 24.0);
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(8);
    assert_eq!(node(renderer, 0).1.toggled(), Some(Toggled::True));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn default_lazy_items_release_local_state_when_they_leave_composition() {
    KEEP.set(false);
    let mut renderer = make_renderer();
    toggle_first(&mut renderer);
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(8, Dp::ZERO));
    renderer.render_frames(8);
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Retained option 0"))
    );
    LIST.get()
        .unwrap()
        .with_mut(|list| list.scroll_to_item(0, Dp::ZERO));
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::False));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn unmounting_the_list_clears_visit_history_from_a_live_external_controller() {
    KEEP.set(true);
    let mut renderer = make_renderer();
    toggle_first(&mut renderer);
    let controller = LIST.get().unwrap();
    assert!(controller.with(|list| !list.composed.is_empty()));
    MOUNTED.get().unwrap().set(false);
    renderer.render_frames(8);
    assert!(controller.is_alive());
    assert!(controller.with(|list| list.composed.is_empty()
        && list.viewport.is_none()
        && list.geometry.is_none()
        && list.focused_item.is_none()));
    MOUNTED.get().unwrap().set(true);
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::False));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn emptying_a_mounted_list_discards_removed_item_state_and_focus_history() {
    KEEP.set(true);
    let mut renderer = make_renderer();
    toggle_first(&mut renderer);
    let controller = LIST.get().unwrap();
    COUNT.get().unwrap().set(0);
    renderer.render_frames(8);
    assert!(controller.is_alive());
    assert!(controller.with(|list| list.composed.is_empty() && list.focused_item.is_none()));
    assert!(
        !renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| { node.role() == Role::CheckBox })
    );
    COUNT.get().unwrap().set(12);
    renderer.render_frames(8);
    assert_eq!(node(&renderer, 0).1.toggled(), Some(Toggled::False));
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn requested_focus_traversal_reveals_items_beyond_the_initial_visible_range() {
    KEEP.set(true);
    let mut renderer = make_renderer();
    toggle_first(&mut renderer);
    for index in 1..12 {
        NAVIGATION.get().unwrap().set(Some(FocusDirection::Next));
        renderer.render_frames(12);
        assert!(NAVIGATION_ACCEPTED.get());
        let target = node(&renderer, index);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            target.0,
            "Traversal must reach item {index}"
        );
        assert_eq!(target.1.toggled(), Some(Toggled::False));
        let bounds = target.1.bounds().unwrap();
        assert!(
            bounds.y0 >= 0.0 && bounds.y1 <= 200.0,
            "Focused item {index} must be revealed: {bounds:?}"
        );
    }
}
