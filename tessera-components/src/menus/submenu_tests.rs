use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusRequester, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, HasPopup, Node, NodeId, Rect, Role, Toggled, TreeId},
    remember, tessera,
};

use crate::{
    modifier::ModifierExt as _,
    surface::surface,
    theme::{MaterialTheme, material_theme},
};

use super::{MenuAnchor, MenuController, MenuItemSelectionMode, menu_item, menu_provider};

thread_local! {
    static ROOT: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static ACTIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static SHOW: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static ENABLED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static EXTERNAL: Cell<Option<FocusRequester>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let root = remember(MenuController::new);
    let actions = remember(|| 0);
    let show = remember(|| true);
    let enabled = remember(|| true);
    let external = remember(FocusRequester::new).get();
    ROOT.set(Some(root));
    ACTIONS.set(Some(actions));
    SHOW.set(Some(show));
    ENABLED.set(Some(enabled));
    EXTERNAL.set(Some(external));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface_container_low = Color::new(0.2, 0.4, 0.6, 1.0);
            theme
        })
        .child(move || {
            menu_provider()
                .controller(root)
                .main_content(move || {
                    surface()
                        .modifier(Modifier::new().fill_max_size())
                        .child(move || {
                            crate::column::column().children(move || {
                                surface()
                                    .modifier(Modifier::new().width(Dp(120.0)).height(Dp(48.0)))
                                    .accessibility_label("Menu trigger")
                                    .on_click(move || root.with_mut(MenuController::open))
                                    .child(|| {});
                                surface()
                                    .modifier(Modifier::new().width(Dp(120.0)).height(Dp(48.0)))
                                    .accessibility_label("External action")
                                    .focus_requester(external)
                                    .on_click(|| {})
                                    .child(|| {});
                            });
                        });
                })
                .menu_content(move || {
                    if show.get() {
                        menu_item()
                            .label("Share")
                            .enabled(enabled.get())
                            .submenu_content(move || {
                                menu_item()
                                    .label("Copy link")
                                    .on_click(move || actions.set(actions.get() + 1));
                                menu_item().label("More options").submenu_content(move || {
                                    menu_item()
                                        .label("Export link")
                                        .on_click(move || actions.set(actions.get() + 10));
                                });
                            });
                    }
                    menu_item()
                        .label("Other action")
                        .on_click(move || actions.set(actions.get() + 100));
                });
        });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    renderer
        .accessibility_snapshot()
        .expect("tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::MenuItem && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing action: {label}"))
        .1
        .bounds()
        .expect("bounds")
}

fn click(renderer: &mut HeadlessRenderer, label: &str) {
    let bounds = bounds(renderer, label);
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    click_point(renderer, point);
}

fn click_point(renderer: &mut HeadlessRenderer, point: PxPosition) {
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(4);
}

#[track_caller]
fn assert_focus(renderer: &HeadlessRenderer, label: &str) {
    let tree = renderer.accessibility_snapshot().expect("tree");
    assert_eq!(
        tree.nodes
            .iter()
            .find(|(id, _)| *id == tree.focus)
            .expect("focus")
            .1
            .label(),
        Some(label)
    );
}

fn open(renderer: &mut HeadlessRenderer, x: f64) {
    ROOT.get().expect("root").with_mut(|root| {
        root.open_at(MenuAnchor::from_dp((Dp(x), Dp(40.0)), (Dp(80.0), Dp(40.0))))
    });
    renderer.render_frames(4);
    click(renderer, "Share");
}

fn check_submenu_chain(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(|| {
            fixture();
        }),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 1000,
            height: 700,
            sample_count: samples,
            ..Default::default()
        },
    )
    .expect("renderer");
    renderer.render_frames(3);
    ROOT.get().expect("root").with_mut(|root| {
        root.open_at(MenuAnchor::from_dp(
            (Dp(40.0), Dp(40.0)),
            (Dp(80.0), Dp(40.0)),
        ))
    });
    renderer.render_frames(4);
    let parent = bounds(&renderer, "Share");
    let previous = renderer.read_pixels().expect("before submenu");
    click(&mut renderer, "Share");
    let child = bounds(&renderer, "Copy link");
    assert!(
        child.x0 >= parent.x1 - 1.0,
        "submenu must open beside its parent: {parent:?}, {child:?}"
    );
    assert!(
        child.y1 - child.y0 >= 48.0,
        "submenu target must not inherit the parent's remaining height"
    );
    let second = bounds(&renderer, "More options");
    let pixel = (((second.y0 + second.y1) / 2.0).round() as usize * 1000
        + (second.x1 - 48.0).round() as usize)
        * 4;
    let shown = renderer.read_pixels().expect("submenu pixels");
    for (index, value) in [0.2_f32, 0.4, 0.6].into_iter().enumerate() {
        let encoded = (1.055 * value.powf(1.0 / 2.4) - 0.055) * 255.0;
        assert!(
            (shown[pixel + index] as f32 - encoded).abs() <= 2.0,
            "submenu fill must render outside the parent clip"
        );
    }
    assert_ne!(&shown[pixel..pixel + 4], &previous[pixel..pixel + 4]);
    click(&mut renderer, "Copy link");
    let restored = renderer.read_pixels().expect("closed pixels");
    assert_eq!(
        &restored[pixel..pixel + 4],
        &previous[pixel..pixel + 4],
        "closing removes the child surface pixels"
    );
    assert_eq!(ACTIONS.get().expect("actions").get(), 1);
    assert!(
        !ROOT.get().expect("root").with(MenuController::is_open),
        "a leaf selection closes the entire menu chain"
    );
    assert_focus(&renderer, "Menu trigger");

    open(&mut renderer, 40.0);
    click(&mut renderer, "More options");
    let parent = bounds(&renderer, "More options");
    let deep = bounds(&renderer, "Export link");
    assert!(
        deep.x0 >= parent.x1 - 1.0,
        "third level: {parent:?}, {deep:?}"
    );
    click(&mut renderer, "Export link");
    assert_eq!(ACTIONS.get().expect("actions").get(), 11);
    assert_focus(&renderer, "Menu trigger");

    open(&mut renderer, 40.0);
    click(&mut renderer, "Other action");
    assert_eq!(
        ACTIONS.get().expect("actions").get(),
        111,
        "one click on an ancestor action must pass through the child scrim"
    );
    assert!(!ROOT.get().expect("root").with(MenuController::is_open));
    assert_focus(&renderer, "Menu trigger");

    open(&mut renderer, 40.0);
    click_point(&mut renderer, PxPosition::from_f32_xy(900.0, 650.0));
    assert!(!ROOT.get().expect("root").with(MenuController::is_open));
    assert_focus(&renderer, "Menu trigger");

    open(&mut renderer, 690.0);
    let parent = bounds(&renderer, "Share");
    let child = bounds(&renderer, "Copy link");
    assert!(
        child.x1 <= parent.x0 + 1.0,
        "right edge flips the child to the left: {parent:?}, {child:?}"
    );
    click(&mut renderer, "More options");
    renderer.resize(390, 640);
    renderer.render_frames(5);
    let narrow = bounds(&renderer, "Export link");
    assert!(
        narrow.x0 >= 0.0 && narrow.x1 <= 390.0 && narrow.y0 >= 0.0 && narrow.y1 <= 640.0,
        "narrow popup: {narrow:?}"
    );
    click(&mut renderer, "Export link");
    assert_eq!(ACTIONS.get().expect("actions").get(), 121);
    assert!(!ROOT.get().expect("root").with(MenuController::is_open));

    renderer.resize(1000, 700);
    renderer.render_frames(3);
    for disable in [false, true] {
        for _ in 0..3 {
            open(&mut renderer, 40.0);
            click(&mut renderer, "More options");
            if disable {
                ENABLED.get().expect("enabled").set(false);
            } else {
                SHOW.get().expect("show").set(false);
            }
            renderer.render_frames(4);
            assert!(ROOT.get().expect("root").with(MenuController::is_open));
            let tree = renderer.accessibility_snapshot().expect("tree");
            assert!(!tree.nodes.iter().any(|(_, node)| matches!(
                node.label(),
                Some("Copy link" | "More options" | "Export link")
            )));
            assert_focus(&renderer, "Other action");
            SHOW.get().expect("show").set(true);
            ENABLED.get().expect("enabled").set(true);
            renderer.render_frames(3);
            assert_focus(&renderer, "Other action");
            ROOT.get().expect("root").with_mut(MenuController::close);
            renderer.render_frames(4);
        }
    }
    open(&mut renderer, 40.0);
    ROOT.get().expect("root").with_mut(MenuController::close);
    renderer.render_frames(4);
    assert_focus(&renderer, "Menu trigger");
    open(&mut renderer, 40.0);
    EXTERNAL.get().expect("external focus").request_focus();
    ROOT.get().expect("root").with_mut(MenuController::close);
    renderer.render_frames(4);
    assert_focus(&renderer, "External action");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_submenus_escape_parent_clip_and_close_the_complete_chain() {
    check_submenu_chain(1);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_submenus_escape_parent_clip_and_close_the_complete_chain_msaa() {
    check_submenu_chain(4);
}

thread_local! {
    static TRAILING: Cell<Option<State<u8>>> = const { Cell::new(None) };
}

#[tessera]
fn disclosure_fixture() {
    let trailing = remember(|| 0_u8);
    TRAILING.set(Some(trailing));
    material_theme().child(move || {
        menu_provider()
            .is_open(true)
            .modifier(Modifier::new().width(Dp(240.0)))
            .menu_content(move || {
                let mut item = menu_item().label("Share").submenu_content(|| {
                    menu_item().label("Copy link").on_click(|| {});
                });
                match trailing.get() {
                    1 => item = item.trailing_text("Hint"),
                    2 => item = item.trailing_icon(crate::material_icons::filled::STAR_SVG),
                    _ => {}
                }
                drop(item);
            });
    });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_submenu_disclosure_remains_visible_with_custom_trailing_content() {
    crate::test_utils::run_isolated_test(
        "menus::submenu_tests::gpu_submenu_disclosure_remains_visible_with_custom_trailing_content",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(disclosure_fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 400,
                    height: 300,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(30);
            let item = bounds(&renderer, "Share");
            let crop = |pixels: Vec<u8>| {
                let mut region = Vec::new();
                for y in (item.y0 as usize + 16)..(item.y0 as usize + 32) {
                    for x in (item.x1 as usize - 36)..(item.x1 as usize - 16) {
                        let offset = (y * 400 + x) * 4;
                        region.extend_from_slice(&pixels[offset..offset + 4]);
                    }
                }
                region
            };
            let initial = crop(renderer.read_pixels().unwrap());
            assert!(
                initial.chunks_exact(4).any(|pixel| pixel != &initial[..4]),
                "the disclosure region must contain painted arrow pixels"
            );
            for variant in [1, 2] {
                TRAILING.get().unwrap().set(variant);
                renderer.render_frames(30);
                let changed = crop(renderer.read_pixels().unwrap());
                assert_eq!(
                    initial, changed,
                    "trailing content must not replace the arrow"
                );
            }
        },
    );
}

thread_local! {
    static CHECKED_ROOT: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static CHECKED_CONFIG: Cell<Option<State<(MenuItemSelectionMode, bool)>>> = const { Cell::new(None) };
    static CHECKED_CALLS: Cell<Option<State<(usize, usize)>>> = const { Cell::new(None) };
}

#[tessera]
fn checked_branch_fixture() {
    let controller = remember(MenuController::new);
    let config = remember(|| (MenuItemSelectionMode::Checkbox, false));
    let calls = remember(|| (0_usize, 0_usize));
    CHECKED_ROOT.set(Some(controller));
    CHECKED_CONFIG.set(Some(config));
    CHECKED_CALLS.set(Some(calls));
    material_theme().child(move || {
        menu_provider()
            .controller(controller)
            .menu_content(move || {
                menu_item()
                    .label("Checked branch")
                    .selection_mode(config.get().0)
                    .selected(config.get().1)
                    .close_on_click(true)
                    .on_click(move || calls.with_mut(|calls| calls.0 += 1))
                    .submenu_content(move || {
                        menu_item()
                            .label("Checked child leaf")
                            .close_on_click(true)
                            .on_click(move || calls.with_mut(|calls| calls.1 += 1));
                    });
            });
    });
}

fn checked_node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {role:?} {label}"))
}

fn checked_action(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
    let target_node = checked_node(renderer, label, role).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(12);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn gpu_checkable_branches_preserve_controlled_state_and_expand_before_leaf_activation() {
    crate::test_utils::run_isolated_test(
        "menus::submenu_tests::gpu_checkable_branches_preserve_controlled_state_and_expand_before_leaf_activation",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(checked_branch_fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 700,
                    height: 300,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(4);
            let controller = CHECKED_ROOT.get().unwrap();
            let config = CHECKED_CONFIG.get().unwrap();
            let calls = CHECKED_CALLS.get().unwrap();
            let mut completed_leaves = 0;
            for (mode, role) in [
                (MenuItemSelectionMode::Radio, Role::MenuItemRadio),
                (MenuItemSelectionMode::Checkbox, Role::MenuItemCheckBox),
            ] {
                for selected in [false, true] {
                    config.set((mode, selected));
                    controller.with_mut(MenuController::open);
                    renderer.render_frames(12);
                    let checked = if selected {
                        Toggled::True
                    } else {
                        Toggled::False
                    };
                    let before = checked_node(&renderer, "Checked branch", role).1;
                    assert_eq!(before.toggled(), Some(checked));
                    assert_eq!(before.is_selected(), None);
                    assert_eq!(before.has_popup(), Some(HasPopup::Menu));
                    assert_eq!(before.is_expanded(), Some(false));
                    checked_action(&mut renderer, "Checked branch", role, Action::Focus);
                    assert_focus(&renderer, "Checked branch");
                    assert_eq!(calls.get(), (0, completed_leaves));
                    assert_eq!(
                        checked_node(&renderer, "Checked branch", role)
                            .1
                            .is_expanded(),
                        Some(false)
                    );
                    checked_action(&mut renderer, "Checked branch", role, Action::Click);
                    assert!(controller.with(MenuController::is_open));
                    assert_eq!(calls.get(), (0, completed_leaves));
                    assert_eq!(config.get(), (mode, selected));
                    let opened = checked_node(&renderer, "Checked branch", role).1;
                    assert_eq!(opened.toggled(), Some(checked));
                    assert_eq!(opened.has_popup(), Some(HasPopup::Menu));
                    assert_eq!(opened.is_expanded(), Some(true));
                    checked_action(
                        &mut renderer,
                        "Checked child leaf",
                        Role::MenuItem,
                        Action::Focus,
                    );
                    assert_focus(&renderer, "Checked child leaf");
                    assert_eq!(calls.get(), (0, completed_leaves));
                    checked_action(
                        &mut renderer,
                        "Checked child leaf",
                        Role::MenuItem,
                        Action::Click,
                    );
                    completed_leaves += 1;
                    assert_eq!(calls.get(), (0, completed_leaves));
                    assert!(!controller.with(MenuController::is_open));
                    assert_eq!(config.get(), (mode, selected));
                    controller.with_mut(MenuController::open);
                    renderer.render_frames(12);
                    let reopened = checked_node(&renderer, "Checked branch", role).1;
                    assert_eq!(reopened.toggled(), Some(checked));
                    assert_eq!(reopened.is_expanded(), Some(false));
                    assert_eq!(calls.get(), (0, completed_leaves));
                    controller.with_mut(MenuController::close);
                    renderer.render_frames(12);
                }
            }
        },
    );
}
