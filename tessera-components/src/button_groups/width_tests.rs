//! Native standard-group width and input regressions.
//!
//! ## Usage
//!
//! Keep press feedback local while preserving neighboring content and row
//! width.

use std::cell::Cell;

use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType,
    PxPosition, State,
    accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Toggled, TreeId},
    layout::layout,
    remember, tessera,
};

use crate::{
    button_groups::{ButtonGroupsSize, ButtonGroupsStyle, button_groups},
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    theme::material_theme,
};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    ratio: f32,
    connected: bool,
    enabled: bool,
    items: usize,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CLICKS: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        ratio: 0.15,
        connected: false,
        enabled: true,
        items: 3,
    });
    SETTINGS.set(Some(settings));
    material_theme().child(move || {
        column().children(move || {
            let mut group = button_groups()
                .size(ButtonGroupsSize::Small)
                .style(if settings.get().connected {
                    ButtonGroupsStyle::Connected
                } else {
                    ButtonGroupsStyle::Standard
                })
                .expanded_ratio(settings.get().ratio)
                .enabled(settings.get().enabled);
            for index in 0..settings.get().items {
                group = group.child(
                    move || {
                        layout().modifier(Modifier::new().size(Dp(68.0), Dp(20.0)).semantics(
                            SemanticsArgs {
                                label: Some(format!("Item {index}")),
                                ..Default::default()
                            },
                        ));
                    },
                    |_| CLICKS.set(CLICKS.get() + 1),
                );
            }
            drop(group);
        });
    });
}

fn nodes(renderer: &HeadlessRenderer, index: usize) -> (NodeId, Node, Node) {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let (mut id, content) = snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(format!("Item {index}").as_str()))
        .unwrap()
        .clone();
    loop {
        let node = &snapshot
            .nodes
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .unwrap()
            .1;
        if node.role() == Role::CheckBox {
            return (id, node.clone(), content);
        }
        id = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.children().contains(&id))
            .unwrap()
            .0;
    }
}

fn center(bounds: Rect) -> PxPosition {
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn check_widths(renderer: &HeadlessRenderer, widths: [f64; 3], scale: f64, gap: f64) {
    let mut x = 0.0;
    for (index, width) in widths.into_iter().enumerate() {
        let (_, node, content) = nodes(renderer, index);
        let bounds = node.bounds().unwrap();
        assert_eq!(
            [bounds.x0, bounds.width(), bounds.height()],
            [x, width, 48.0 * scale]
        );
        let content = content.bounds().unwrap();
        assert_eq!(
            [content.width(), content.height()],
            [68.0 * scale, 20.0 * scale],
            "compression preserves content dimensions"
        );
        assert_eq!(
            (content.x0 + content.x1) / 2.0,
            (bounds.x0 + bounds.x1) / 2.0
        );
        x += width + gap;
    }
    assert_eq!(x - gap, widths.iter().sum::<f64>() + 2.0 * gap);
}

fn check_press_widths(samples: u32) {
    for scale in [1.0, 1.5] {
        CLICKS.set(0);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 800,
                height: 180,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(12);
        let idle_builds = super::GROUP_ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(
            super::GROUP_ITEM_BUILDS.get(),
            idle_builds,
            "an idle group must retain its items"
        );
        let settings = SETTINGS.get().unwrap();
        let base = [100.0 * scale; 3];
        let gap = 12.0 * scale;
        check_widths(&renderer, base, scale, gap);
        let middle = nodes(&renderer, 1);
        let point = center(middle.1.bounds().unwrap());
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(30);
        check_widths(
            &renderer,
            if scale == 1.0 {
                [92.0, 116.0, 92.0]
            } else {
                [139.0, 172.0, 139.0]
            },
            scale,
            gap,
        );
        assert_eq!(nodes(&renderer, 1).1.toggled(), Some(Toggled::False));
        assert_eq!(CLICKS.get(), 0);
        let held_builds = super::GROUP_ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(
            super::GROUP_ITEM_BUILDS.get(),
            held_builds,
            "a settled held press must stop rebuilding"
        );
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(nodes(&renderer, 1).0, middle.0);
        assert_eq!(nodes(&renderer, 1).1.toggled(), Some(Toggled::True));
        assert_eq!(CLICKS.get(), 1);
        let settled_builds = super::GROUP_ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(
            super::GROUP_ITEM_BUILDS.get(),
            settled_builds,
            "release must stop the animation driver"
        );

        let first = center(nodes(&renderer, 0).1.bounds().unwrap());
        renderer.pointer_move(MOUSE_POINTER_ID, first);
        renderer.pointer_press(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(30);
        check_widths(
            &renderer,
            if scale == 1.0 {
                [115.0, 85.0, 100.0]
            } else {
                [173.0, 127.0, 150.0]
            },
            scale,
            gap,
        );
        renderer.pointer_release(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(CLICKS.get(), 2);

        renderer.pointer_press(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(2);
        assert_ne!(
            nodes(&renderer, 0).1.bounds().unwrap().width(),
            base[0],
            "same-frame click must animate"
        );
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(CLICKS.get(), 3);
        let target_node = nodes(&renderer, 0).0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node,
            data: None
        }));
        renderer.render_frames(2);
        assert_ne!(
            nodes(&renderer, 0).1.bounds().unwrap().width(),
            base[0],
            "accessibility activation must animate"
        );
        renderer.render_frames(60);
        assert_eq!(CLICKS.get(), 4);
        check_widths(&renderer, base, scale, gap);

        settings.with_mut(|value| value.ratio = 0.0);
        renderer.render_frames(5);
        renderer.pointer_press(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(30);
        check_widths(&renderer, base, scale, gap);
        renderer.pointer_release(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(60);
        settings.with_mut(|value| value.ratio = 1.0);
        renderer.render_frames(5);
        renderer.pointer_press(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(30);
        check_widths(
            &renderer,
            if scale == 1.0 {
                [116.0, 84.0, 100.0]
            } else {
                [174.0, 126.0, 150.0]
            },
            scale,
            gap,
        );
        settings.with_mut(|value| value.connected = true);
        renderer.render_frames(10);
        check_widths(&renderer, base, scale, 2.0 * scale);
        renderer.pointer_release(MOUSE_POINTER_ID, first, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, 2.0 * scale);
        assert_eq!(CLICKS.get(), 6);

        settings.with_mut(|value| {
            value.connected = false;
            value.ratio = 0.15;
        });
        renderer.render_frames(10);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(2);
        settings.with_mut(|value| value.enabled = false);
        renderer.render_frames(10);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(CLICKS.get(), 6);
        settings.with_mut(|value| value.enabled = true);
        renderer.render_frames(10);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(2);
        settings.with_mut(|value| value.items = 0);
        renderer.render_frames(10);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::CheckBox)
        );
        settings.with_mut(|value| value.items = 1);
        renderer.render_frames(10);
        let point = center(nodes(&renderer, 0).1.bounds().unwrap());
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(30);
        assert_eq!(nodes(&renderer, 0).1.bounds().unwrap().width(), base[0]);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        assert_eq!(CLICKS.get(), 7);
        settings.with_mut(|value| value.items = 3);
        renderer.render_frames(10);
        let point = center(nodes(&renderer, 1).1.bounds().unwrap());
        let outside = PxPosition::from_f32_xy(700.0, 150.0);
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(2);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        renderer.render_frames(2);
        renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(
            CLICKS.get(),
            7,
            "cancelled presses must not activate a button"
        );
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(30);
        check_widths(
            &renderer,
            if scale == 1.0 {
                [92.0, 116.0, 92.0]
            } else {
                [139.0, 172.0, 139.0]
            },
            scale,
            gap,
        );
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        check_widths(&renderer, base, scale, gap);
        assert_eq!(
            CLICKS.get(),
            8,
            "a later press must recover after cancellation"
        );
        let baseline = renderer.read_pixels().unwrap();
        for _ in 0..2 {
            renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
            renderer.render_frames(60);
        }
        assert!(
            renderer.read_pixels().unwrap() == baseline,
            "width, shape, and ripple pixels must fully recover after two toggles"
        );
        assert_eq!(CLICKS.get(), 10);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn standard_width_exchange_preserves_content_pixels_and_input_msaa1() {
    crate::test_utils::run_isolated_test(
        "button_groups::width_tests::standard_width_exchange_preserves_content_pixels_and_input_msaa1",
        || check_press_widths(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn standard_width_exchange_preserves_content_pixels_and_input_msaa4() {
    crate::test_utils::run_isolated_test(
        "button_groups::width_tests::standard_width_exchange_preserves_content_pixels_and_input_msaa4",
        || check_press_widths(4),
    );
}
