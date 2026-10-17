use std::cell::Cell;

use tessera_ui::{
    Dp, FocusRequester, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State, accesskit::Role, remember, tessera,
};

use crate::{
    column::column,
    lazy_list::{LazyListController, lazy_column},
    modifier::ModifierExt as _,
    surface::surface,
    theme::material_theme,
};

use super::{MenuController, close_for_traversal, menu_group, menu_item, menu_provider};

thread_local! {
    static CONTROLLER: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static EXPLICIT: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static OTHER_FOCUS: Cell<Option<FocusRequester>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let controller = remember(MenuController::new);
    let enabled = remember(|| true);
    let explicit_fallback = remember(|| false);
    let other = remember(FocusRequester::new).get();
    CONTROLLER.set(Some(controller));
    ENABLED.set(Some(enabled));
    EXPLICIT.set(Some(explicit_fallback));
    OTHER_FOCUS.set(Some(other));
    material_theme().child(move || {
        menu_provider()
            .grouped(true)
            .controller(controller)
            .focus_restorer_fallback_optional(explicit_fallback.get().then_some(other))
            .main_content(move || {
                column().children(move || {
                    surface()
                        .modifier(Modifier::new().width(Dp(180.0)).height(Dp(48.0)))
                        .accessibility_label("Unrelated action")
                        .focus_requester(other)
                        .on_click(|| {})
                        .child(|| {});
                    surface()
                        .modifier(Modifier::new().width(Dp(180.0)).height(Dp(48.0)))
                        .accessibility_label("Menu trigger")
                        .enabled(enabled.get())
                        .on_click(move || controller.with_mut(MenuController::open))
                        .child(|| {});
                    surface()
                        .modifier(Modifier::new().width(Dp(180.0)).height(Dp(48.0)))
                        .accessibility_label("Next action")
                        .on_click(|| {})
                        .child(|| {});
                });
            })
            .menu_content(move || {
                menu_group().children(move || {
                    menu_item()
                        .label("Choose action")
                        .on_click(move || controller.with_mut(MenuController::close));
                });
            });
    });
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let tree = renderer.accessibility_snapshot().expect("tree");
    let rect = tree
        .nodes
        .iter()
        .find(|(_, node)| {
            node.label() == Some(label) && matches!(node.role(), Role::Button | Role::MenuItem)
        })
        .expect("action")
        .1
        .bounds()
        .expect("bounds");
    let point = PxPosition::from_f32_xy(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
}

fn assert_focus(renderer: &HeadlessRenderer, label: &str, stage: &str) {
    let tree = renderer.accessibility_snapshot().expect("tree");
    assert_eq!(
        tree.nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .expect("focused node")
            .1
            .label(),
        Some(label),
        "{stage}"
    );
}

thread_local! {
    static LAZY_MENU: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static LAZY_TARGETS: Cell<Option<State<Vec<FocusRequester>>>> = const { Cell::new(None) };
}

#[tessera]
fn lazy_fixture() {
    let menu = remember(MenuController::new);
    let targets = remember(|| (0..30).map(|_| FocusRequester::new()).collect::<Vec<_>>());
    let list = remember(|| {
        let mut list = LazyListController::new();
        list.scroll_controller_mut()
            .set_scroll_position(PxPosition::from_f32_xy(0.0, -480.0));
        list
    });
    LAZY_MENU.set(Some(menu));
    LAZY_TARGETS.set(Some(targets));
    material_theme().child(move || {
        menu_provider()
            .controller(menu)
            .main_content(move || {
                lazy_column()
                    .controller(list)
                    .estimated_item_size(Dp(48.0))
                    .overscan(0)
                    .modifier(Modifier::new().width(Dp(180.0)).height(Dp(144.0)))
                    .items(30, move |index| {
                        surface()
                            .modifier(Modifier::new().width(Dp(180.0)).height(Dp(48.0)))
                            .accessibility_label(format!("Action {index}"))
                            .focus_requester(targets.get()[index])
                            .on_click(move || {
                                menu.with_mut(|menu| {
                                    menu.open_at(super::MenuAnchor::from_dp(
                                        (Dp(200.0), Dp(0.0)),
                                        (Dp(20.0), Dp(10.0)),
                                    ))
                                })
                            })
                            .child(|| {});
                    });
            })
            .menu_content(|| {
                menu_item().label("Choose action").on_click(|| {});
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_tab_exit_reveals_virtualized_neighbors_before_traversing() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            lazy_fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 350,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(5);
    assert!(
        !renderer
            .accessibility_snapshot()
            .expect("tree")
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Action 9")),
        "previous item starts unmaterialized"
    );
    click(&mut renderer, "Action 10");
    assert_focus(&renderer, "Choose action", "opened from a virtualized item");
    close_for_traversal(LAZY_MENU.get().expect("menu"), &None, true);
    renderer.render_frames(12);
    assert_focus(
        &renderer,
        "Action 9",
        "reverse Tab reveals the previous item",
    );
    for index in [8, 7] {
        assert!(
            tessera_ui::FocusManager::current()
                .request_move_focus(tessera_ui::FocusDirection::Previous)
        );
        renderer.render_frames(12);
        assert_focus(
            &renderer,
            &format!("Action {index}"),
            "repeated traversal uses current lazy bounds",
        );
    }
    assert!(
        tessera_ui::FocusManager::current().request_move_focus(tessera_ui::FocusDirection::Next)
    );
    renderer.render_frames(4);
    assert_focus(&renderer, "Action 8", "forward traversal after lazy reveal");
    let current = LAZY_TARGETS.get().expect("targets").get()[8];
    current.capture_focus();
    assert!(
        !tessera_ui::FocusManager::current()
            .request_move_focus(tessera_ui::FocusDirection::Previous)
    );
    renderer.render_frames(3);
    assert_focus(&renderer, "Action 8", "capture rejects queued traversal");
    current.free_focus();
    assert!(
        tessera_ui::FocusManager::current()
            .request_move_focus(tessera_ui::FocusDirection::Previous)
    );
    current.request_focus();
    renderer.render_frames(3);
    assert_focus(
        &renderer,
        "Action 8",
        "a later explicit request cancels queued traversal",
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_menu_restores_trigger_without_stealing_external_focus() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 400,
            height: 320,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(3);
    let controller = CONTROLLER.get().expect("controller");
    for explicit in [false, true] {
        EXPLICIT.get().expect("explicit fallback").set(explicit);
        renderer.render_frames(2);
        for _ in 0..3 {
            click(&mut renderer, "Menu trigger");
            assert_focus(&renderer, "Choose action", "opened");
            click(&mut renderer, "Choose action");
            assert!(!controller.with(MenuController::is_open));
            assert_focus(
                &renderer,
                if explicit {
                    "Unrelated action"
                } else {
                    "Menu trigger"
                },
                "closed after selection",
            );
        }
        if !explicit {
            for (reverse, expected) in [(false, "Next action"), (true, "Unrelated action")] {
                click(&mut renderer, "Menu trigger");
                close_for_traversal(controller, &None, reverse);
                renderer.render_frames(4);
                assert!(!controller.with(MenuController::is_open));
                assert_focus(&renderer, expected, "Tab exit");
            }
        }
        click(&mut renderer, "Menu trigger");
        OTHER_FOCUS.get().expect("other").request_focus();
        renderer.render_frames(2);
        controller.with_mut(MenuController::close);
        renderer.render_frames(3);
        assert_focus(&renderer, "Unrelated action", "external focus");

        click(&mut renderer, "Menu trigger");
        ENABLED.get().expect("enabled").set(false);
        controller.with_mut(MenuController::close);
        renderer.render_frames(3);
        let tree = renderer.accessibility_snapshot().expect("tree");
        let trigger = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Menu trigger"))
            .expect("trigger");
        assert_ne!(tree.focus, trigger.0);
        if explicit {
            assert_focus(&renderer, "Unrelated action", "disabled trigger");
        }
        ENABLED.get().expect("enabled").set(true);
        renderer.render_frames(3);
        assert_ne!(
            renderer.accessibility_snapshot().expect("tree").focus,
            trigger.0
        );
    }
}
