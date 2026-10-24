//! Native tooltip example interaction regressions.
//!
//! ## Usage
//!
//! Verify trigger actions and the active explanation in live examples.

use std::{cell::Cell, time::Duration};

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, ActionRequest, Node, NodeId, Rect, TreeId},
};

use super::*;

thread_local! {
    static RICH: Cell<bool> = const { Cell::new(false) };
}

#[tessera]
fn color_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                tooltip_color_pair(RICH.get());
            });
    });
}

#[tessera]
fn configuration_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                tooltip_configurations();
            });
    });
}

fn reference_renderer(root: fn()) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(root),
        vec![Box::new(ComponentPipelines)],
        HeadlessConfig {
            width: 1024,
            height: 1000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    eprintln!(
        "tooltip fixture: {:?}",
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| matches!(
                node.role(),
                Role::Group | Role::Note | Role::Dialog | Role::Tooltip
            ))
            .map(|(_, node)| (node.role(), node.label(), node.bounds()))
            .collect::<Vec<_>>()
    );
    renderer
}

fn semantic_node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label} ({role:?})"))
}

fn activate(renderer: &mut HeadlessRenderer, label: &str) {
    let target_node = semantic_node(renderer, label, Role::Button).0;
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action: Action::Click,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(4);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_color_pairs_keep_native_popups_and_annotations_in_both_themes() {
    for rich in [false, true] {
        RICH.set(rich);
        let renderer = reference_renderer(color_fixture);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let kind = if rich { "Rich" } else { "Plain" };
        semantic_node(
            &renderer,
            &format!("{kind} tooltip color roles"),
            Role::Group,
        );
        let expected = if rich { 4 } else { 2 };
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note
                        && node
                            .label()
                            .is_some_and(|label| label.starts_with("Tooltip color role "))
                })
                .count(),
            expected * 2
        );
        let role = if rich { Role::Dialog } else { Role::Tooltip };
        let mut popups: Vec<_> = snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .map(|(_, node)| node.bounds().unwrap())
            .collect();
        popups.sort_by(|left, right| left.x0.total_cmp(&right.x0));
        assert_eq!(popups.len(), 2);
        assert!(popups[0].x1 < popups[1].x0);
        assert_eq!(popups[0].height(), popups[1].height());
        if rich {
            assert_eq!(popups[0].width(), 312.0);
        }
        let label = if rich {
            "Tooltip color role 4: Primary"
        } else {
            "Tooltip color role 2: Inverse surface"
        };
        let mut markers: Vec<_> = snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Note && node.label() == Some(label))
            .map(|(_, node)| node.bounds().unwrap())
            .collect();
        markers.sort_by(|left, right| left.x0.total_cmp(&right.x0));
        assert_eq!(markers.len(), 2);
        for (marker, popup) in markers.iter().zip(&popups) {
            if rich {
                let action = snapshot
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == Role::Button
                            && node.label() == Some("Action")
                            && node.bounds().is_some_and(|bounds| {
                                bounds.x0 >= popup.x0 && bounds.x1 <= popup.x1
                            })
                    })
                    .unwrap()
                    .1
                    .bounds()
                    .unwrap();
                assert!(((marker.y0 + marker.y1) - (action.y0 + action.y1)).abs() <= 2.0);
            } else {
                assert!(marker.y1 < popup.y0);
            }
        }
        let pixels = renderer.read_pixels().unwrap();
        let center = |popup: Rect| {
            let x = (popup.x0 + 8.0) as usize;
            let y = (popup.y0 + popup.height() / 2.0) as usize;
            &pixels[(y * 1024 + x) * 4..][..3]
        };
        assert_ne!(center(popups[0]), center(popups[1]));
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_configurations_keep_five_native_cards_and_dismiss_actions_independently() {
    let mut renderer = reference_renderer(configuration_fixture);
    let canvas = semantic_node(&renderer, "Rich tooltip five configurations", Role::Group)
        .1
        .bounds()
        .unwrap();
    assert!((canvas.height() - canvas.width() * 1394.0 / 2048.0).abs() <= 1.0);
    let mut cards = Vec::new();
    for (number, count) in [(1, 2), (2, 1), (3, 0), (4, 1), (5, 2)] {
        cards.push(
            semantic_node(
                &renderer,
                &format!("Rich tooltip configuration {number}"),
                Role::Group,
            )
            .1
            .bounds()
            .unwrap(),
        );
        semantic_node(
            &renderer,
            &format!("Show rich tooltip configuration {number}"),
            Role::Button,
        );
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Button
                    && node.label().is_some_and(|label| label
                        .starts_with(&format!("Tooltip configuration {number} action "))))
                .count(),
            count
        );
    }
    assert_eq!(cards[0].x0, cards[1].x0);
    assert_eq!(cards[2].x0, cards[3].x0);
    assert_eq!(cards[3].x0, cards[4].x0);
    assert!(cards[0].x1 < cards[2].x0);
    assert!(cards[0].y0 < cards[1].y0);
    assert!(cards[2].y0 < cards[3].y0 && cards[3].y0 < cards[4].y0);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    for (_, popup) in snapshot
        .nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
    {
        assert_eq!(
            popup.bounds().unwrap().width(),
            312.0,
            "native configuration tooltips must retain their reference container width"
        );
    }
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Dialog)
            .count(),
        4
    );
    semantic_node(&renderer, "Rich tooltip", Role::Tooltip);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(
                |(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip)
                    && node.label() == Some("Rich tooltip")
            )
            .count(),
        3
    );
    activate(&mut renderer, "Tooltip configuration 1 action 2");
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Dialog)
            .count(),
        3
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Tooltip configuration 1 actions: 1"))
    );
    semantic_node(&renderer, "Tooltip configuration 5 action 2", Role::Button);
    activate(&mut renderer, "Show rich tooltip configuration 1");
    semantic_node(&renderer, "Tooltip configuration 1 action 2", Role::Button);
    assert_eq!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Dialog)
            .count(),
        4
    );
}

struct ComponentPipelines;
impl tessera_ui::RenderModule for ComponentPipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    material_theme().child(|| {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(600.0)))
                    .children(|| {
                        live_tooltips();
                    });
            });
    });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing live tooltip control {label}"))
        .1
        .bounds()
        .unwrap()
}

fn click(renderer: &mut HeadlessRenderer, bounds: Rect) {
    let position = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(MOUSE_POINTER_ID, position);
    renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
    renderer.render_frames(4);
}

#[test]
#[ignore = "Requires a GPU adapter for the real component pipelines"]
fn live_tooltips_keep_trigger_actions_and_replace_the_active_explanation() {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fixture),
        vec![Box::new(ComponentPipelines)],
        HeadlessConfig {
            width: 800,
            height: 600,
            sample_count: 1,
            ..Default::default()
        },
    )
    .expect("native tooltip examples");
    renderer.render_frames(4);
    let favorite = bounds(&renderer, "Favorite tooltip action");
    renderer.pointer_move(
        MOUSE_POINTER_ID,
        PxPosition::from_f32_xy(
            ((favorite.x0 + favorite.x1) * 0.5) as f32,
            ((favorite.y0 + favorite.y1) * 0.5) as f32,
        ),
    );
    renderer.render_frames(35);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Tooltip
                && node.label() == Some("Add to favorites"))
    );
    click(&mut renderer, favorite);
    assert!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Favorite: on; tooltip actions: 0")),
        "tooltip hover must preserve the trigger's actual toggle action"
    );
    let about = bounds(&renderer, "About collections");
    click(&mut renderer, about);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert!(
        !snapshot
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Tooltip
                && matches!(
                    node.label(),
                    Some("Add to favorites" | "Remove from favorites")
                )),
        "the shared live host dismisses the preceding plain tooltip"
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog && node.label() == Some("Collections"))
    );
    let action = bounds(&renderer, "Got it");
    click(&mut renderer, action);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    assert!(
        !snapshot
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Dialog && node.label() == Some("Collections"))
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Favorite: on; tooltip actions: 1")),
        "the real rich tooltip action updates its owner"
    );
}
