use std::{cell::Cell, time::Duration};

use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, modifier::ModifierExt as _,
    theme::material_theme,
};
use tessera_ui::{
    Dp, HeadlessConfig, HeadlessRenderer, Modifier,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    tessera,
};

use super::{baseline_figure, color_figure};

thread_local! {
    static RAIL: Cell<bool> = const { Cell::new(false) };
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn baseline_fixture() {
    material_theme().child(|| {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(|| {
                        baseline_figure(RAIL.get(), false);
                    });
            });
    });
}

#[tessera]
fn color_fixture() {
    material_theme().child(|| {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(|| {
                        color_figure(RAIL.get(), false);
                        color_figure(RAIL.get(), true);
                    });
            });
    });
}

fn nodes(renderer: &HeadlessRenderer) -> Vec<(NodeId, Node)> {
    renderer
        .accessibility_snapshot()
        .expect("navigation figure semantics")
        .nodes
}

fn destination(renderer: &HeadlessRenderer, description: &str) -> (NodeId, Node) {
    nodes(renderer)
        .into_iter()
        .find(|(_, node)| node.role() == Role::Tab && node.description() == Some(description))
        .unwrap_or_else(|| panic!("missing navigation destination {description}"))
}

fn dispatch(renderer: &mut HeadlessRenderer, target_node: NodeId, action: Action) {
    assert!(renderer.dispatch_accessibility_action(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node,
        data: None,
    }));
    renderer.render_frames(6);
}

fn assert_selection(renderer: &HeadlessRenderer, prefix: &str, selected: usize, count: usize) {
    for index in 1..=count {
        let (_, node) = destination(renderer, &format!("{prefix} destination {index}"));
        assert_eq!(
            node.is_selected(),
            Some(index == selected),
            "{prefix} destination {index}: published {node:?}"
        );
        assert_eq!(
            node.toggled(),
            None,
            "destination selection is distinct from toggling"
        );
    }
}

fn active_icon_marker(renderer: &HeadlessRenderer, prefix: &str) -> tessera_ui::accesskit::Rect {
    let group = nodes(renderer)
        .into_iter()
        .find(|(_, node)| {
            node.role() == Role::Group
                && node.label() == Some(format!("{prefix} color roles").as_str())
        })
        .expect("native navigation color panel")
        .1
        .bounds()
        .unwrap();
    nodes(renderer)
        .into_iter()
        .find_map(|(_, node)| {
            if node.role() != Role::Note
                || node.label() != Some("Navigation color role 2: On secondary container")
            {
                return None;
            }
            let bounds = node.bounds()?;
            let x = (bounds.x0 + bounds.x1) * 0.5;
            let y = (bounds.y0 + bounds.y1) * 0.5;
            (x >= group.x0 && x < group.x1 && y >= group.y0 && y < group.y1).then_some(bounds)
        })
        .unwrap_or_else(|| panic!("missing active icon callout in {prefix}"))
}

#[test]
#[ignore = "Requires a native GPU adapter for actual baseline navigation geometry and focus"]
fn baseline_figures_keep_native_destination_dimensions_and_focus_without_selecting() {
    for rail in [false, true] {
        RAIL.set(rail);
        let mut renderer = HeadlessRenderer::new(
            Box::new(baseline_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1100,
                height: 700,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native baseline navigation renderer");
        renderer.render_frames(6);
        let prefix = if rail {
            "Baseline navigation rail"
        } else {
            "Baseline navigation bar"
        };
        assert_eq!(
            nodes(&renderer)
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab)
                .count(),
            4
        );
        let container = nodes(&renderer)
            .into_iter()
            .find(|(_, node)| node.role() == Role::TabList && node.label() == Some(prefix))
            .expect("baseline navigation container")
            .1
            .bounds()
            .unwrap();
        assert_eq!(
            (container.width(), container.height()),
            if rail { (80.0, 480.0) } else { (412.0, 80.0) }
        );
        let mut bounds = Vec::new();
        for index in 1..=4 {
            let node = destination(&renderer, &format!("{prefix} destination {index}")).1;
            let rect = node.bounds().unwrap();
            assert_eq!(
                (rect.width(), rect.height()),
                if rail { (80.0, 56.0) } else { (103.0, 80.0) }
            );
            bounds.push(rect);
        }
        for pair in bounds.windows(2) {
            if rail {
                assert_eq!(pair[0].x0, pair[1].x0);
                assert_eq!(pair[1].y0 - pair[0].y1, 4.0);
            } else {
                assert_eq!(pair[0].x1, pair[1].x0);
                assert_eq!(pair[0].y0, pair[1].y0);
            }
        }
        assert_selection(&renderer, prefix, 1, 4);
        let second = destination(&renderer, &format!("{prefix} destination 2")).0;
        dispatch(&mut renderer, second, Action::Focus);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, second);
        assert_selection(&renderer, prefix, 1, 4);
        dispatch(&mut renderer, second, Action::Click);
        assert_selection(&renderer, prefix, 2, 4);
    }
}

#[test]
#[ignore = "Requires a native GPU adapter for actual navigation color figures and role annotations"]
fn color_figures_publish_official_role_markers_and_independent_live_destination_selections() {
    for rail in [false, true] {
        RAIL.set(rail);
        let mut renderer = HeadlessRenderer::new(
            Box::new(color_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1100,
                height: 1300,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native navigation color renderer");
        renderer.render_frames(24);
        let roles: &[&str] = if rail {
            &[
                "Surface container (optional)",
                "On secondary container",
                "Secondary container",
                "Secondary (vertical) / On secondary container (horizontal)",
                "On surface variant",
                "On surface variant",
                "Error",
                "On error",
                "Error",
            ]
        } else {
            &[
                "Surface container",
                "On secondary container",
                "Secondary",
                "Secondary container",
                "On surface variant",
                "On surface variant",
            ]
        };
        let markers: Vec<_> = nodes(&renderer)
            .into_iter()
            .filter(|(_, node)| {
                node.role() == Role::Note
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Navigation color role "))
            })
            .collect();
        assert_eq!(markers.len(), if rail { 36 } else { 12 });
        for (index, role) in roles.iter().enumerate() {
            let label = format!("Navigation color role {}: {role}", index + 1);
            let matching: Vec<_> = markers
                .iter()
                .filter(|(_, node)| node.label() == Some(label.as_str()))
                .collect();
            assert_eq!(matching.len(), if rail { 4 } else { 2 }, "{label}");
            for (_, node) in matching {
                let bounds = node.bounds().unwrap();
                assert_eq!((bounds.width(), bounds.height()), (24.0, 32.0));
                assert!(
                    bounds.x0 >= 0.0 && bounds.x1 <= 1024.0,
                    "{label}, rail={rail}, bounds={bounds:?}"
                );
                assert!(
                    bounds.y0 >= 0.0 && bounds.y1 <= if rail { 1203.0 } else { 824.0 },
                    "{label}, rail={rail}, bounds={bounds:?}; panels={:?}",
                    nodes(&renderer)
                        .into_iter()
                        .filter(|(_, node)| node.role() == Role::Group)
                        .map(|(_, node)| (node.label().map(str::to_owned), node.bounds()))
                        .collect::<Vec<_>>()
                );
            }
        }
        let prefixes: &[&str] = if rail {
            &[
                "Light collapsed rail",
                "Light expanded rail",
                "Dark collapsed rail",
                "Dark expanded rail",
            ]
        } else {
            &["Light bar", "Dark bar"]
        };
        let count = if rail { 4 } else { 3 };
        assert_eq!(
            nodes(&renderer)
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab)
                .count(),
            prefixes.len() * count
        );
        for prefix in prefixes {
            assert_selection(&renderer, prefix, 1, count);
            for index in 1..=count {
                let bounds = destination(&renderer, &format!("{prefix} destination {index}"))
                    .1
                    .bounds()
                    .unwrap();
                assert_eq!(
                    bounds.height(),
                    if rail && prefix.contains("expanded") {
                        56.0
                    } else {
                        64.0
                    }
                );
                if rail {
                    assert_eq!(
                        bounds.width(),
                        if prefix.contains("expanded") {
                            220.0
                        } else {
                            96.0
                        }
                    );
                }
            }
        }
        for (group, prefix) in prefixes.iter().enumerate() {
            let original_markers: Vec<_> = prefixes
                .iter()
                .map(|prefix| active_icon_marker(&renderer, prefix))
                .collect();
            let last = destination(&renderer, &format!("{prefix} destination {count}")).0;
            dispatch(&mut renderer, last, Action::Focus);
            assert_eq!(renderer.accessibility_snapshot().unwrap().focus, last);
            assert_selection(&renderer, prefix, 1, count);
            let second = destination(&renderer, &format!("{prefix} destination 2")).0;
            dispatch(&mut renderer, second, Action::Click);
            for (other, other_prefix) in prefixes.iter().enumerate() {
                assert_selection(
                    &renderer,
                    other_prefix,
                    if other <= group { 2 } else { 1 },
                    count,
                );
                let marker = active_icon_marker(&renderer, other_prefix);
                if other == group {
                    if rail {
                        assert_eq!(marker.x0, original_markers[other].x0);
                        assert!(marker.y0 - original_markers[other].y0 >= 56.0);
                    } else {
                        assert_eq!(marker.y0, original_markers[other].y0);
                        assert!(marker.x0 - original_markers[other].x0 > 100.0);
                    }
                } else {
                    assert_eq!(
                        marker, original_markers[other],
                        "{other_prefix} stays independent"
                    );
                }
            }
        }
    }
}
