//! Fractional popup anchors and native overlay actions.
//!
//! ## Usage
//!
//! Keep popup placement and actions aligned across scaling boundaries.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    time::Duration,
};

use tessera_ui::{
    CursorEventContent, Dp, FocusModifierExt as _, GestureState, HeadlessConfig, HeadlessRenderer,
    LayoutCoordinates, MOUSE_POINTER_ID, Modifier, PlacementModifierNode, PointerChange,
    PressKeyEventType, PxPosition, State,
    accesskit::{Node, NodeId, Rect, Role},
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    time::Instant,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    menus::{MenuAnchor, MenuController, menu_item, menu_provider},
    modifier::ModifierExt as _,
    spacer::spacer,
    surface::surface,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::material_theme,
    tooltip::{
        TooltipController, TooltipPlacement, TooltipVariant, tooltip_anchor, tooltip_provider,
    },
};

thread_local! {
    static GEOMETRY: RefCell<BTreeMap<&'static str, LayoutCoordinates>> = const { RefCell::new(BTreeMap::new()) };
    static EDITOR: Cell<Option<State<TextInputController>>> = const { Cell::new(None) };
    static SHIFT: Cell<Option<State<f32>>> = const { Cell::new(None) };
    static TOOLTIP: Cell<Option<State<TooltipController>>> = const { Cell::new(None) };
    static MENU: Cell<Option<State<MenuController>>> = const { Cell::new(None) };
    static ACTIONS: Cell<Option<State<u32>>> = const { Cell::new(None) };
}

#[derive(Clone, Copy, PartialEq)]
struct FractionalOrigin(State<f32>);
impl PlacementModifierNode for FractionalOrigin {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + PxPosition::from_f32_xy(10.25 + self.0.get(), 20.5)
    }
}

fn observe(name: &'static str) -> Modifier {
    Modifier::new().on_layout_coordinates(move |coordinates| {
        GEOMETRY.with_borrow_mut(|values| {
            values.insert(name, coordinates);
        });
    })
}

#[tessera]
fn tooltip_fixture() {
    let tooltip = remember(TooltipController::new);
    let shift = remember(|| 0.0_f32);
    SHIFT.set(Some(shift));
    let actions = remember(|| 0_u32);
    TOOLTIP.set(Some(tooltip));
    ACTIONS.set(Some(actions));
    material_theme().child(move || {
        tooltip_provider("A precise contextual explanation.")
            .controller(tooltip)
            .variant(TooltipVariant::Rich)
            .placement(TooltipPlacement::Below)
            .headline("Precise tooltip")
            .action_label("Accept precise tooltip")
            .on_action(move || actions.set(actions.get() + 1))
            .popup_modifier(observe("popup").width(Dp(240.0)))
            .child(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        boxed()
                            .alignment(Alignment::TopStart)
                            .modifier(Modifier::new().push_placement(FractionalOrigin(shift)))
                            .children(move || {
                                scaled_box(0.625).child(move || {
                                    boxed()
                                        .modifier(Modifier::new().offset(Dp(240.0), Dp(160.0)))
                                        .children(move || {
                                            tooltip_anchor().controller(tooltip).child(|| {
                                                surface()
                                                    .modifier(
                                                        observe("anchor")
                                                            .width(Dp(241.0))
                                                            .height(Dp(57.0)),
                                                    )
                                                    .on_click(|| {})
                                                    .child(|| {});
                                            });
                                        });
                                });
                            });
                    });
            });
    });
}

#[tessera]
fn submenu_fixture() {
    let menu = remember(MenuController::new);
    let actions = remember(|| 0_u32);
    MENU.set(Some(menu));
    ACTIONS.set(Some(actions));
    material_theme().child(move || {
        menu_provider()
            .controller(menu)
            .anchor(MenuAnchor::at(PxPosition::from_f32_xy(30.25, 40.5)))
            .offset([Dp::ZERO; 2])
            .main_content(|| {
                spacer().modifier(Modifier::new().fill_max_size());
            })
            .menu_content(move || {
                scaled_box(0.625).child(move || {
                    boxed()
                        .modifier(Modifier::new().width(Dp(241.0)))
                        .children(move || {
                            menu_item()
                                .label("Share precise link")
                                .submenu_content(move || {
                                    menu_item()
                                        .label("Copy precise link")
                                        .on_click(move || actions.set(actions.get() + 1));
                                });
                        });
                });
            });
    });
}

fn renderer(entry: fn(), samples: u32) -> HeadlessRenderer {
    GEOMETRY.with_borrow_mut(BTreeMap::clear);
    let mut renderer = HeadlessRenderer::new(
        Box::new(entry),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 600,
            height: 400,
            sample_count: samples,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("Native GPU adapter");
    renderer.render_frames(8);
    renderer
}

fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .expect("Accessibility tree")
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .expect("Native overlay control")
}

fn click(renderer: &mut HeadlessRenderer, bounds: Rect) {
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, point);
    renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.step_frame();
    renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(6);
}

fn menu_bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    let update = renderer
        .accessibility_snapshot()
        .expect("Accessibility tree");
    let item = update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::MenuItem && node.label() == Some(label))
        .expect("Menu action")
        .0;
    update
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Menu && node.children().contains(&item))
        .expect("Action's semantic menu")
        .1
        .bounds()
        .expect("Menu bounds")
}

#[test]
#[ignore = "requires a native GPU adapter for fractional tooltip placement and actions"]
fn scaled_tooltip_keeps_fractional_anchor_reflows_and_accepts_real_action_click() {
    crate::test_utils::run_isolated_test(
        "overlay_precision_tests::scaled_tooltip_keeps_fractional_anchor_reflows_and_accepts_real_action_click",
        || {
            for samples in [1, 4] {
                let mut renderer = renderer(tooltip_fixture, samples);
                TOOLTIP
                    .get()
                    .expect("Tooltip controller")
                    .with_mut(TooltipController::show);
                renderer.render_frames(8);
                let (anchor, popup) = GEOMETRY.with_borrow(|values| {
                    (
                        values["anchor"].window_bounds(),
                        values["popup"].window_bounds(),
                    )
                });
                assert_eq!(
                    anchor,
                    tessera_ui::FloatRect {
                        x: 160.25,
                        y: 120.5,
                        width: 150.625,
                        height: 35.625
                    }
                );
                assert_eq!(
                    popup.x, 115.5625,
                    "Centering must preserve the fractional physical anchor width"
                );
                assert_eq!(
                    popup.y, 160.125,
                    "Below placement includes the precise anchor bottom and 4px gap"
                );
                assert_eq!(popup.width, 240.0);
                renderer.resize(200, 400);
                renderer.render_frames(8);
                let popup = GEOMETRY.with_borrow(|values| values["popup"].window_bounds());
                assert!(popup.x >= 8.0 && popup.x + popup.width <= 192.0);
                let action = node(&renderer, "Accept precise tooltip", Role::Button)
                    .1
                    .bounds()
                    .expect("Action bounds");
                assert!(action.x0 >= 0.0 && action.x1 <= 200.0);
                click(&mut renderer, action);
                assert_eq!(ACTIONS.get().expect("Action state").get(), 1);
                assert!(
                    !TOOLTIP
                        .get()
                        .expect("Tooltip controller")
                        .with(TooltipController::is_open)
                );
            }
        },
    );
}

#[test]
#[ignore = "requires a native GPU adapter for fractional submenu placement and actions"]
fn scaled_submenu_uses_exact_anchor_edge_reflows_and_accepts_real_action_click() {
    crate::test_utils::run_isolated_test(
        "overlay_precision_tests::scaled_submenu_uses_exact_anchor_edge_reflows_and_accepts_real_action_click",
        || {
            for samples in [1, 4] {
                let mut renderer = renderer(submenu_fixture, samples);
                MENU.get()
                    .expect("Menu controller")
                    .with_mut(MenuController::open);
                renderer.render_frames(8);
                let parent = node(&renderer, "Share precise link", Role::MenuItem)
                    .1
                    .bounds()
                    .expect("Scaled parent bounds");
                assert_eq!(parent.width(), 150.625);
                click(&mut renderer, parent);
                let popup = menu_bounds(&renderer, "Copy precise link");
                assert_eq!(
                    popup.x0, parent.x1,
                    "Submenu begins at the exact physical parent edge"
                );
                renderer.resize(200, 400);
                renderer.render_frames(8);
                let popup = menu_bounds(&renderer, "Copy precise link");
                assert!(popup.x0 >= 0.0 && popup.x1 <= 200.0);
                let action = node(&renderer, "Copy precise link", Role::MenuItem)
                    .1
                    .bounds()
                    .expect("Submenu action bounds");
                click(&mut renderer, action);
                assert_eq!(ACTIONS.get().expect("Action state").get(), 1);
                assert!(
                    !MENU
                        .get()
                        .expect("Menu controller")
                        .with(MenuController::is_open)
                );
            }
        },
    );
}

#[test]
#[ignore = "requires a native GPU adapter for scaled held-touch tooltip gestures"]
fn scaled_tooltip_touch_slop_uses_window_samples_after_ancestor_translation() {
    crate::test_utils::run_isolated_test(
        "overlay_precision_tests::scaled_tooltip_touch_slop_uses_window_samples_after_ancestor_translation",
        || {
            for (movement, translation) in [(6.0_f32, 0.0_f32), (0.0, 12.0)] {
                let mut renderer = renderer(tooltip_fixture, 1);
                let anchor = GEOMETRY.with_borrow(|values| values["anchor"].window_bounds());
                let press = PxPosition::from_f32_xy(
                    anchor.x + anchor.width * 0.5,
                    anchor.y + anchor.height * 0.5,
                );
                renderer.pointer_press(7, press, PressKeyEventType::Left);
                renderer.step_frame();
                assert!(
                    !TOOLTIP
                        .get()
                        .expect("Tooltip controller")
                        .with(TooltipController::is_open)
                );
                if translation != 0.0 {
                    SHIFT
                        .get()
                        .expect("Ancestor placement state")
                        .set(translation);
                    renderer.render_frames(4);
                }
                let moved = press + PxPosition::from_f32_xy(movement, 0.0);
                renderer.pointer_move(7, moved);
                renderer.render_frames(40);
                assert!(
                    TOOLTIP
                        .get()
                        .expect("Tooltip controller")
                        .with(TooltipController::is_open),
                    "A physical move below 8px or an ancestor translation must preserve held touch: movement={movement}, translation={translation}"
                );
                renderer.pointer_release(7, moved, PressKeyEventType::Left);
                renderer.render_frames(4);
            }
        },
    );
}

#[tessera]
fn text_fixture() {
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("tessera framework");
        editor
    });
    let shift = remember(|| 0.0_f32);
    EDITOR.set(Some(editor));
    SHIFT.set(Some(shift));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                boxed()
                    .alignment(Alignment::TopStart)
                    .modifier(Modifier::new().push_placement(FractionalOrigin(shift)))
                    .children(move || {
                        scaled_box(0.625).child(move || {
                            TextFieldBuilder::filled()
                                .controller(editor)
                                .label("Message")
                                .accessibility_label("Precise editor")
                                .modifier(Modifier::new().width(Dp(300.0)));
                        });
                    });
            });
    });
}

fn timed_click(renderer: &mut HeadlessRenderer, point: PxPosition, timestamp: Instant) {
    renderer.inject_pointer(
        PointerChange::new(
            timestamp,
            MOUSE_POINTER_ID,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        )
        .with_position(point),
    );
    renderer.step_frame();
    renderer.inject_pointer(
        PointerChange::new(
            timestamp + Duration::from_millis(1),
            MOUSE_POINTER_ID,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        )
        .with_position(point),
    );
    renderer.render_frames(2);
}

#[test]
#[ignore = "requires a native GPU adapter for scaled editor multi-click selection"]
fn scaled_editor_multi_click_uses_window_history_and_keeps_local_selection_geometry() {
    crate::test_utils::run_isolated_test(
        "overlay_precision_tests::scaled_editor_multi_click_uses_window_history_and_keeps_local_selection_geometry",
        || {
            for (first_offset, movement, translation) in
                [(4.0_f32, 4.0_f32, 0.0_f32), (24.0, 0.0, 12.0)]
            {
                let mut renderer = renderer(text_fixture, 1);
                let bounds = node(&renderer, "Precise editor", Role::TextInput)
                    .1
                    .bounds()
                    .expect("Native editor bounds");
                let first = PxPosition::from_f32_xy(
                    bounds.x0 as f32 + first_offset,
                    ((bounds.y0 + bounds.y1) * 0.5) as f32,
                );
                let timestamp = Instant::now();
                timed_click(&mut renderer, first, timestamp);
                let editor = EDITOR.get().expect("Native editor controller");
                let cursor = editor.with(|editor| editor.cursor_offset());
                assert!(
                    cursor > 0 && cursor < 7,
                    "The first click must hit inside the first native word: {cursor}"
                );
                assert!(editor.with(|editor| editor.selection().is_collapsed()));
                if translation != 0.0 {
                    SHIFT
                        .get()
                        .expect("Ancestor placement state")
                        .set(translation);
                    renderer.render_frames(4);
                }
                timed_click(
                    &mut renderer,
                    first + PxPosition::from_f32_xy(movement, 0.0),
                    timestamp + Duration::from_millis(100),
                );
                assert_eq!(
                    editor.with(|editor| editor.selected_text()).as_deref(),
                    Some("tessera"),
                    "A physical click pair within 5px must select the word despite local scaling or ancestor movement: movement={movement}, translation={translation}"
                );
            }
        },
    );
}
