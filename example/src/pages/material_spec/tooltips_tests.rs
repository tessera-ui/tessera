//! Native tooltip example interaction regressions.
//!
//! ## Usage
//!
//! Verify trigger actions and the active explanation in live examples.

use std::time::Duration;

use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
    accesskit::{Action, Node, NodeId, Rect, Toggled},
};

use super::*;

#[derive(Clone)]
enum Scene {
    Color(bool),
    Configuration,
    Figure(bool, bool),
    Overview,
    Live(f64, tessera_ui::testing::TestProbe<usize>),
    Placement,
    Anatomy(bool),
    Wording(bool),
    PlainPlacement,
    Keyboard,
}

fn reference_renderer(scene: Scene, width: u32) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || match scene.clone() {
            Scene::Live(width, actions) => {
                fixture(width, actions);
            }
            scene => {
                material_theme().child(move || match scene.clone() {
                    Scene::PlainPlacement => {
                        tooltip_plain_placement();
                    }
                    Scene::Keyboard => {
                        boxed()
                            .alignment(Alignment::TopStart)
                            .modifier(Modifier::new().fill_max_size())
                            .children(tooltip_keyboard_table);
                    }
                    scene => {
                        let modifier = if matches!(scene, Scene::Color(_) | Scene::Configuration) {
                            Modifier::new().fill_max_size()
                        } else {
                            Modifier::new().fill_max_width()
                        };
                        column().modifier(modifier).children(move || match scene {
                            Scene::Color(rich) => {
                                tooltip_color_pair(rich);
                            }
                            Scene::Configuration => {
                                tooltip_configurations();
                            }
                            Scene::Figure(rich, measurements) => {
                                tooltip_figure(rich, measurements);
                            }
                            Scene::Overview => {
                                tooltip_variant_examples();
                                tooltip_material_comparison();
                            }
                            Scene::Placement => {
                                placement_examples();
                            }
                            Scene::Anatomy(rich) => {
                                tooltip_figure(rich, false).guidelines(true);
                            }
                            Scene::Wording(rich) => {
                                tooltip_writing_examples(rich);
                            }
                            _ => unreachable!(),
                        });
                    }
                });
            }
        }),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width,
            height: 1000,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(4);
    renderer
}

fn semantic_node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
    let snapshot = renderer
        .inspect_accessibility()
        .expect("accessibility tree");
    let (id, node) = snapshot.find(label, role).expect("tooltip control");
    (id, node.clone())
}

fn settle_color_pair_geometry(renderer: &mut HeadlessRenderer) {
    let mut previous = None;
    let mut history = Vec::new();
    let mut stable_frames = 0;
    for _ in 0..16 {
        renderer.step_frame();
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let mut geometry: Vec<_> = snapshot
            .nodes
            .iter()
            .filter(|(_, node)| {
                matches!(node.role(), Role::Dialog | Role::Tooltip)
                    || (node.role() == Role::Note
                        && node
                            .label()
                            .is_some_and(|label| label.starts_with("Tooltip color role ")))
                    || (matches!(node.role(), Role::Button | Role::Label)
                        && node.label() == Some("Action"))
            })
            .map(|(id, node)| {
                (
                    *id,
                    node.role(),
                    node.label().map(str::to_owned),
                    node.bounds(),
                )
            })
            .collect();
        geometry.sort_by_key(|(id, _, _, _)| id.0);
        stable_frames = if previous.as_ref() == Some(&geometry) {
            stable_frames + 1
        } else {
            1
        };
        history.push((renderer.frame_index(), geometry.clone()));
        previous = Some(geometry);
        if stable_frames == 3 {
            return;
        }
    }
    panic!("tooltip color geometry did not stabilize within 16 frames: {history:#?}");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_color_pairs_keep_native_popups_and_annotations_in_both_themes() {
    for rich in [false, true] {
        let mut renderer = reference_renderer(Scene::Color(rich), 1024);
        settle_color_pair_geometry(&mut renderer);
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
                let action_label = snapshot
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == Role::Label
                            && node.label() == Some("Action")
                            && node.bounds().is_some_and(|bounds| {
                                bounds.x0 >= popup.x0 && bounds.x1 <= popup.x1
                            })
                    })
                    .expect("rich tooltip action text")
                    .1
                    .bounds()
                    .unwrap();
                assert!(
                    action_label.x0 >= action.x0
                        && action_label.x1 <= action.x1
                        && action_label.y0 >= action.y0
                        && action_label.y1 <= action.y1,
                    "action text {action_label:?} must remain inside its native button {action:?}"
                );
                assert!(
                    ((marker.y0 + marker.y1) - (action_label.y0 + action_label.y1)).abs() <= 2.0,
                    "rich tooltip color marker {marker:?} must align with its primary action text {action_label:?} inside {popup:?}"
                );
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
        for dark in [false, true] {
            let half = semantic_node(
                &renderer,
                &format!(
                    "{} {} tooltip color example",
                    if dark { "Dark" } else { "Light" },
                    if rich { "rich" } else { "plain" }
                ),
                Role::Group,
            )
            .1
            .bounds()
            .unwrap();
            let at = |x: usize, y: usize| -> [u8; 3] {
                let offset = (((half.y0 as usize + y) * 1024) + half.x0 as usize + x) * 4;
                pixels[offset..offset + 3].try_into().unwrap()
            };
            let near = |actual: [u8; 3], expected: [u8; 3]| {
                actual
                    .into_iter()
                    .zip(expected)
                    .all(|(a, b)| a.abs_diff(b) <= 2)
            };
            assert_eq!(half.width(), 512.0);
            assert_eq!(half.x0.fract(), 0.0);
            assert_eq!(half.y0.fract(), 0.0);
            let background = if dark { [20, 18, 24] } else { [254, 247, 255] };
            let grid = if dark { [96, 92, 101] } else { [189, 184, 192] };
            let mut covered = 0;
            for y in 40..64 {
                for x in 40..64 {
                    let actual = at(x, y);
                    let dot = x % 8 == 2 && y % 8 == 2;
                    assert!(
                        near(actual, if dot { grid } else { background }),
                        "rich={rich} dark={dark} grid ({x},{y}): {actual:?}, dot={dot}"
                    );
                    covered += usize::from(!near(actual, background));
                }
            }
            assert_eq!(covered, 9, "three rows of three grid dots in each theme");
            let popup = popups[usize::from(dark)];
            let expected_popup = match (rich, dark) {
                (false, false) => [50, 47, 53],
                (false, true) => [230, 224, 233],
                (true, false) => [243, 237, 247],
                (true, true) => [33, 31, 38],
            };
            let x = (popup.x0 + 16.0).floor() as usize;
            let y = (popup.y0 + 2.0).floor() as usize;
            let offset = (y * 1024 + x) * 4;
            let actual: [u8; 3] = pixels[offset..offset + 3].try_into().unwrap();
            assert!(
                near(actual, expected_popup),
                "rich={rich} dark={dark} popup palette: {actual:?}, expected {expected_popup:?}"
            );
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_configurations_keep_five_native_cards_static_after_pointer_clicks() {
    let mut renderer = reference_renderer(Scene::Configuration, 1024);
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
    assert!(!snapshot.nodes.iter().any(|(_, node)| {
        node.label().is_some_and(|label| {
            label.starts_with("Show rich tooltip configuration") || label.contains(" actions:")
        })
    }));
    let mut initial: Vec<_> = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
        .map(|(id, node)| (*id, node.bounds()))
        .collect();
    initial.sort_by_key(|(id, _)| id.0);
    let actions: Vec<_> = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Tooltip configuration "))
        })
        .map(|(_, node)| {
            assert!(!node.supports_action(Action::Click));
            node.bounds().unwrap()
        })
        .collect();
    assert_eq!(actions.len(), 6);
    for action in actions {
        click(&mut renderer, action);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(1000.0, 900.0));
        renderer.render_frames(4);
        let mut current: Vec<_> = renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
            .map(|(id, node)| (*id, node.bounds()))
            .collect();
        current.sort_by_key(|(id, _)| id.0);
        assert_eq!(
            current, initial,
            "reference actions must not dismiss or move any popup"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_measurements_scale_native_content_and_keep_rulers_inside_the_canvas() {
    for width in [1024, 640, 390] {
        for rich in [false, true] {
            let mut renderer = reference_renderer(Scene::Figure(rich, true), width);
            settle_color_pair_geometry(&mut renderer);
            let tree = renderer.accessibility_snapshot().unwrap();
            let canvas = semantic_node(
                &renderer,
                &format!(
                    "{} tooltip measurement figure",
                    if rich { "Rich" } else { "Plain" }
                ),
                Role::Group,
            )
            .1
            .bounds()
            .unwrap();
            let popups: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
                .map(|(_, node)| node.bounds().unwrap())
                .collect();
            assert_eq!(popups.len(), if rich { 1 } else { 2 });
            let scale = f64::from(width) / 1024.0;
            assert_eq!(canvas.width(), f64::from(width));
            for popup in &popups {
                assert!(
                    popup.x0 >= canvas.x0
                        && popup.x1 <= canvas.x1
                        && popup.y0 >= canvas.y0
                        && popup.y1 <= canvas.y1
                );
                if rich {
                    assert!((popup.width() - 312.0 * scale).abs() <= 1.0);
                }
            }
            let dimensions: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| {
                    node.role() == Role::Note
                        && node
                            .label()
                            .is_some_and(|label| label.starts_with("Dimension "))
                })
                .collect();
            assert_eq!(dimensions.len(), if rich { 6 } else { 5 });
            for (_, note) in dimensions {
                assert!(!matches!(note.label(), Some("Dimension 164")));
                let bounds = note.bounds().unwrap();
                assert!(
                    bounds.x0 >= canvas.x0
                        && bounds.y0 >= canvas.y0
                        && bounds.x1 <= canvas.x1
                        && bounds.y1 <= canvas.y1,
                    "{width} {rich}: {:?} outside {canvas:?}",
                    note
                );
            }
            if rich {
                let measured = renderer.read_pixels().unwrap();
                let popup = popups[0];
                let x = (popup.x0 + 8.0 * scale).floor() as usize;
                let y = (popup.y0 + 48.0 * scale).floor() as usize;
                let offset = (y * width as usize + x) * 4;
                let initial = renderer.accessibility_snapshot().unwrap();
                let action = semantic_node(&renderer, "Action", Role::Button).1;
                assert!(!action.supports_action(Action::Click));
                click(&mut renderer, action.bounds().unwrap());
                renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(0.0, 900.0));
                renderer.render_frames(4);
                let current = renderer.accessibility_snapshot().unwrap();
                let initial: Vec<_> = initial
                    .nodes
                    .iter()
                    .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
                    .map(|(id, node)| (*id, node.bounds()))
                    .collect();
                let current: Vec<_> = current
                    .nodes
                    .iter()
                    .filter(|(_, node)| matches!(node.role(), Role::Dialog | Role::Tooltip))
                    .map(|(id, node)| (*id, node.bounds()))
                    .collect();
                assert_eq!(
                    initial, current,
                    "measurement reference actions must stay static"
                );
                drop(renderer);
                let mut anatomy = reference_renderer(Scene::Figure(rich, false), width);
                settle_color_pair_geometry(&mut anatomy);
                let plain = anatomy.read_pixels().unwrap();
                assert!(
                    i16::from(measured[offset]) - i16::from(measured[offset + 1])
                        > i16::from(plain[offset]) - i16::from(plain[offset + 1]) + 10,
                    "padding bands must draw above the native popup at width {width}"
                );
            }
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn tooltip_overview_keeps_both_variants_on_one_canvas_and_the_material_comparison() {
    let mut renderer = reference_renderer(Scene::Overview, 1024);
    settle_color_pair_geometry(&mut renderer);
    let canvas = semantic_node(&renderer, "Plain and rich tooltip variants", Role::Group)
        .1
        .bounds()
        .unwrap();
    let comparison = semantic_node(
        &renderer,
        "Material 2 and Material 3 tooltip comparison",
        Role::Group,
    )
    .1
    .bounds()
    .unwrap();
    assert_eq!(canvas.width(), 1024.0);
    assert_eq!(canvas.height(), 385.0);
    assert_eq!(comparison.width(), canvas.width());
    assert!(comparison.y0 > canvas.y1);
    let tree = renderer.accessibility_snapshot().unwrap();
    let examples: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| {
            matches!(node.role(), Role::Dialog | Role::Tooltip)
                && node
                    .bounds()
                    .is_some_and(|rect| rect.y0 >= canvas.y0 && rect.y1 <= canvas.y1)
        })
        .collect();
    assert_eq!(examples.len(), 2);
    let plain = examples
        .iter()
        .find(|(_, node)| node.role() == Role::Tooltip)
        .unwrap()
        .1
        .bounds()
        .unwrap();
    let rich = examples
        .iter()
        .find(|(_, node)| node.role() == Role::Dialog)
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert!(plain.x1 < rich.x0);
    assert!(((plain.y0 + plain.y1) - (rich.y0 + rich.y1)).abs() <= 1.0);
    assert_eq!(rich.width(), 312.0);
    assert!(tree.nodes.iter().any(|(_, node)| {
        node.role() == Role::Dialog
            && node
                .bounds()
                .is_some_and(|rect| rect.y0 >= comparison.y0 && rect.y1 <= comparison.y1)
    }));
}

#[tessera]
fn fixture(width: f64, actions: tessera_ui::testing::TestProbe<usize>) {
    let on_acknowledge = tessera_ui::Callback::new(move || actions.with_mut(|count| *count += 1));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                column()
                    .modifier(Modifier::new().width(Dp(width)))
                    .children(move || {
                        live_tooltips().on_acknowledge_shared(on_acknowledge);
                    });
            });
    });
}

fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
    semantic_node(renderer, label, Role::Button)
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
    let actions = tessera_ui::testing::TestProbe::new(0usize);
    let output = actions.clone();
    let mut renderer = HeadlessRenderer::new(
        Box::new(move || fixture(600.0, output.clone())),
        vec![Box::new(crate::TestPipelines)],
        HeadlessConfig {
            width: 800,
            height: 600,
            sample_count: 1,
            ..Default::default()
        },
    )
    .expect("native tooltip examples");
    renderer.render_frames(4);
    let favorite = bounds(&renderer, "Add to favorites");
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
    assert_eq!(actions.get(), 0);
    let selected = semantic_node(&renderer, "Remove from favorites", Role::Button).1;
    assert_eq!(selected.toggled(), Some(Toggled::True));
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
    assert_eq!(
        actions.get(),
        1,
        "the actual action updates its production owner exactly once"
    );
    renderer.render_frames(4);
    assert_eq!(actions.get(), 1);
    semantic_node(&renderer, "Collection guidance acknowledged.", Role::Label);
    let selected = bounds(&renderer, "Remove from favorites");
    click(&mut renderer, selected);
    assert_eq!(
        semantic_node(&renderer, "Add to favorites", Role::Button)
            .1
            .toggled(),
        Some(Toggled::False)
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn persistent_live_tooltips_leave_the_trigger_visible_after_flow_wrapping() {
    for width in [600, 320] {
        let actions = tessera_ui::testing::TestProbe::new(0usize);
        let output = actions.clone();
        let mut renderer = reference_renderer(Scene::Live(f64::from(width), output), width);
        let about = bounds(&renderer, "About collections");
        click(&mut renderer, about);
        settle_color_pair_geometry(&mut renderer);
        let popup = semantic_node(&renderer, "Collections", Role::Dialog)
            .1
            .bounds()
            .unwrap();
        assert!(
            (popup.y0 - about.y1 - 4.0).abs() <= 0.01,
            "width {width}: popup {popup:?} must stay4dp below trigger {about:?}"
        );
        assert!(popup.y0 >= about.y1);
        assert_eq!(actions.get(), 0);
        let got_it = bounds(&renderer, "Got it");
        click(&mut renderer, got_it);
        assert_eq!(actions.get(), 1);
        renderer.render_frames(4);
        assert_eq!(actions.get(), 1);
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Dialog)
        );
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node
                    .label()
                    .is_some_and(|label| label.starts_with("Favorite:")
                        || label.contains("tooltip actions:")))
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn corner_tooltips_flip_inside_their_device_frames_and_stay_next_to_the_trigger() {
    for width in [600, 300] {
        let mut renderer = reference_renderer(Scene::Placement, width);
        settle_color_pair_geometry(&mut renderer);
        let tree = renderer.accessibility_snapshot().unwrap();
        let frames: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.role() == Role::Group
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Placement preview "))
            })
            .collect();
        assert_eq!(frames.len(), 4);
        for (_, frame) in frames {
            let rect = frame.bounds().unwrap();
            let alignment = frame
                .label()
                .unwrap()
                .strip_prefix("Placement preview ")
                .unwrap();
            let trigger = bounds(&renderer, &format!("Corner tooltip {alignment}"));
            let popups: Vec<_> = tree
                .nodes
                .iter()
                .filter(|(_, node)| {
                    // The corner tooltips carry a Learn more action, so they
                    // publish the interactive dialog role.
                    node.role() == Role::Dialog
                        && node.label() == Some("Paint Tool")
                        && node.bounds().is_some_and(|popup| {
                            popup.x0 >= rect.x0
                                && popup.y0 >= rect.y0
                                && popup.x1 <= rect.x1
                                && popup.y1 <= rect.y1
                        })
                })
                .collect();
            assert_eq!(
                popups.len(),
                1,
                "width {width}: {alignment} must have one native tooltip inside its device"
            );
            let popup = popups[0].1.bounds().unwrap();
            assert!(
                popup.x0 >= rect.x0 + 8.0
                    && popup.x1 <= rect.x1 - 8.0
                    && popup.y0 >= rect.y0 + 8.0
                    && popup.y1 <= rect.y1 - 8.0
            );
            let gap = if alignment.starts_with("Top") {
                popup.y0 - trigger.y1
            } else {
                trigger.y0 - popup.y1
            };
            assert!(
                (gap - 4.0).abs() <= 0.01,
                "width {width}: {alignment} popup {popup:?}, trigger {trigger:?}, gap {gap}"
            );
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn guideline_anatomy_uses_its_own_numbers_and_optional_rich_elements() {
    let mut renderer = reference_renderer(Scene::Anatomy(false), 1024);
    settle_color_pair_geometry(&mut renderer);
    let popup = semantic_node(&renderer, "Plain tooltip", Role::Tooltip)
        .1
        .bounds()
        .unwrap();
    let container = semantic_node(&renderer, "Anatomy marker 1", Role::Note)
        .1
        .bounds()
        .unwrap();
    let supporting = semantic_node(&renderer, "Anatomy marker 2", Role::Note)
        .1
        .bounds()
        .unwrap();
    assert!(container.y1 < popup.y0);
    assert!(supporting.y0 > popup.y1);
    drop(renderer);
    let mut renderer = reference_renderer(Scene::Anatomy(true), 1024);
    settle_color_pair_geometry(&mut renderer);
    semantic_node(&renderer, "New settings available", Role::Dialog);
    semantic_node(&renderer, "Subhead (optional)", Role::Label);
    semantic_node(&renderer, "Text button (optional)", Role::Label);
    let action = semantic_node(&renderer, "Learn more", Role::Button)
        .1
        .bounds()
        .unwrap();
    let marker = semantic_node(&renderer, "Anatomy marker 4", Role::Note)
        .1
        .bounds()
        .unwrap();
    assert!(marker.x1 < action.x0);
    assert!(((marker.y0 + marker.y1) - (action.y0 + action.y1)).abs() <= 1.0);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn writing_examples_keep_plain_tooltips_content_sized_and_publish_advice_bars() {
    let mut renderer = reference_renderer(Scene::Wording(false), 640);
    settle_color_pair_geometry(&mut renderer);
    let popup = semantic_node(&renderer, "Notifications", Role::Tooltip)
        .1
        .bounds()
        .unwrap();
    let text = semantic_node(&renderer, "Notifications", Role::Label)
        .1
        .bounds()
        .unwrap();
    assert_eq!(popup.width() - text.width(), 16.0);
    assert_eq!(popup.height(), 24.0);
    semantic_node(&renderer, "Caution", Role::Label);
    semantic_node(
        &renderer,
        "Avoid wrapping text to multiple lines or including many pieces of information",
        Role::Label,
    );
    drop(renderer);
    let mut renderer = reference_renderer(Scene::Wording(true), 640);
    renderer.render_frames(4);
    assert_eq!(
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button && node.label() == Some("Learn more"))
            .count(),
        2
    );
    semantic_node(&renderer, "Do", Role::Label);
    semantic_node(&renderer, "Don't", Role::Label);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn plain_placement_reference_measures_four_dp_from_the_actual_icon_target() {
    let mut renderer = reference_renderer(Scene::PlainPlacement, 1024);
    settle_color_pair_geometry(&mut renderer);
    let popup = semantic_node(&renderer, "Add to list", Role::Tooltip)
        .1
        .bounds()
        .unwrap();
    let target = semantic_node(&renderer, "Add to list", Role::Button)
        .1
        .bounds()
        .unwrap();
    assert!(
        (target.y0 - popup.y1 - 4.0).abs() <= 0.01,
        "popup {popup:?}, actual icon target {target:?}"
    );
    semantic_node(&renderer, "Dimension 4", Role::Note);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn keyboard_table_keeps_headers_cells_and_centered_header_text() {
    let mut renderer = reference_renderer(Scene::Keyboard, 640);
    renderer.render_frames(4);
    semantic_node(&renderer, "Tooltip keyboard navigation", Role::Table);
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let table = semantic_node(&renderer, "Tooltip keyboard navigation", Role::Table)
        .1
        .bounds()
        .unwrap();
    let rows = snapshot
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Row)
        .map(|(_, node)| node.bounds().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].height(), 56.0);
    assert_eq!(
        table.height(),
        rows.iter().map(|row| row.height()).sum::<f64>() + (rows.len() - 1) as f64
    );
    for row in &rows {
        assert!(row.y0 >= table.y0 && row.y1 <= table.y1);
    }
    let pixels = renderer.read_pixels().unwrap();
    let width = renderer.physical_size().width;
    let x = (table.x0 + (table.width() * 0.355).floor()) as u32;
    for row in &rows {
        let y = ((row.y0 + row.y1) / 2.0) as u32;
        let offset = ((y * width + x) * 4) as usize;
        let adjacent = ((y * width + x - 2) * 4) as usize;
        assert_ne!(
            &pixels[offset..offset + 3],
            &pixels[adjacent..adjacent + 3],
            "each content-sized row must retain its painted column rule"
        );
    }
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::ColumnHeader)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::RowHeader)
            .count(),
        2
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Cell)
            .count(),
        2
    );
    for (index, label) in ["Keys", "Actions"].into_iter().enumerate() {
        let cell = semantic_node(&renderer, label, Role::ColumnHeader)
            .1
            .bounds()
            .unwrap();
        let text = renderer
            .inspect_components()
            .tagged(&format!("tooltip-table-header-{index}"))
            .bounds()
            .unwrap()
            .rounded();
        let text_center = f64::from(text.x.to_f32()) + f64::from(text.width.to_f32()) / 2.0;
        assert!(((cell.x0 + cell.x1) / 2.0 - text_center).abs() <= 0.5);
    }
}
