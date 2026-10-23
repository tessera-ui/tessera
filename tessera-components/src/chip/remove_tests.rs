//! Input-chip body and removal interaction regressions.
//!
//! ## Usage
//!
//! Preserve separate selection and removal targets in compact chips.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier, PressKeyEventType, PxPosition,
    State,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    remember, tessera,
};

use crate::{
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    theme::material_theme,
};

use super::chip;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Counts {
    body: usize,
    remove: usize,
    selected: bool,
}

thread_local! {static COUNTS: Cell<Option<State<Counts>>> = const { Cell::new(None) };}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        let counts = remember(Counts::default);
        COUNTS.set(Some(counts));
        chip()
            .input("Alice")
            .selected(counts.get().selected)
            .leading_icon(filled::PERSON_SVG)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                test_tag: Some("removable-chip".into()),
                ..Default::default()
            }))
            .on_click(move || {
                counts.with_mut(|value| {
                    value.body += 1;
                    value.selected = !value.selected;
                })
            })
            .on_remove(move || counts.with_mut(|value| value.remove += 1));
    });
}

fn node(renderer: &HeadlessRenderer, label: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
}

fn click(renderer: &mut HeadlessRenderer, x: f64, y: f64) {
    let position = PxPosition::from_f32_xy(x as f32, y as f32);
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(3);
}

fn check_removal() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 360,
            height: 120,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    let counts = COUNTS.get().unwrap();
    let remove = node(&renderer, "Remove Alice");
    assert_eq!(remove.1.role(), Role::Button);
    let target = remove.1.bounds().unwrap();
    assert_eq!((target.width(), target.height()), (48.0, 48.0));
    click(
        &mut renderer,
        (target.x0 + target.x1) / 2.0,
        (target.y0 + target.y1) / 2.0,
    );
    assert_eq!(
        counts.get(),
        Counts {
            remove: 1,
            ..Default::default()
        }
    );
    let body = node(&renderer, "Alice").1.bounds().unwrap();
    click(&mut renderer, body.x0 + 16.0, (body.y0 + body.y1) / 2.0);
    assert_eq!(
        counts.get(),
        Counts {
            body: 1,
            remove: 1,
            selected: true
        }
    );
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Focus,
        target_tree: TreeId::ROOT,
        target_node: node(&renderer, "Remove Alice").0,
        data: None
    }));
    renderer.render_frames(2);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|(id, _)| *id == snapshot.focus)
            .unwrap()
            .1
            .label(),
        Some("Remove Alice")
    );
    assert_eq!(
        counts.get().remove,
        1,
        "focusing removal must not activate it"
    );
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node: node(&renderer, "Remove Alice").0,
        data: None
    }));
    renderer.render_frames(2);
    assert_eq!(
        counts.get(),
        Counts {
            body: 1,
            remove: 2,
            selected: true
        }
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn input_chip_removal_has_independent_48dp_target_focus_and_activation() {
    crate::test_utils::run_isolated_test(
        "chip::remove_tests::input_chip_removal_has_independent_48dp_target_focus_and_activation",
        check_removal,
    );
}
