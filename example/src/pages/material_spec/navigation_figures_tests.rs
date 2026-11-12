use tessera_components::{
    alignment::Alignment, boxed::boxed, column::column, theme::material_theme,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Dp, Modifier,
    accesskit::{Action, Node, NodeId, Role},
    tessera,
    testing::UiTest,
};

use super::{baseline_figure, color_figure};

#[tessera]
fn baseline_fixture(rail: bool) {
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(move || {
                        baseline_figure(rail, false);
                    });
            });
    });
}

#[tessera]
fn color_fixture(rail: bool) {
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                column()
                    .modifier(Modifier::new().width(Dp(1024.0)))
                    .children(move || {
                        color_figure(rail, false);
                        color_figure(rail, true);
                    });
            });
    });
}

fn nodes(renderer: &UiTest) -> Vec<(NodeId, Node)> {
    renderer
        .accessibility_snapshot()
        .expect("navigation figure semantics")
        .nodes
}

fn destination(renderer: &UiTest, description: &str) -> (NodeId, Node) {
    nodes(renderer)
        .into_iter()
        .find(|(_, node)| {
            node.role() == Role::Tab && crate::text_matches(node.description(), description)
        })
        .unwrap_or_else(|| panic!("missing navigation destination {description}"))
}

fn dispatch(renderer: &mut UiTest, target_node: NodeId, action: Action) {
    assert!(renderer.accessibility_action(target_node, action));
    renderer.frames(6);
}

fn assert_selection(renderer: &UiTest, prefix: &str, selected: usize, count: usize) {
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

fn active_icon_marker(renderer: &UiTest, prefix: &str) -> tessera_ui::accesskit::Rect {
    let group = nodes(renderer)
        .into_iter()
        .find(|(_, node)| {
            node.role() == Role::Group
                && crate::has_label(&node, format!("{prefix} color roles").as_str())
        })
        .expect("native navigation color panel")
        .1
        .bounds()
        .unwrap();
    nodes(renderer)
        .into_iter()
        .find_map(|(_, node)| {
            if node.role() != Role::Note
                || !crate::has_label(&node, "Navigation color role 2: On secondary container")
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
        let mut renderer = UiTest::new()
            .viewport(1100, 700)
            .rendered(crate::TestPipelines)
            .content(move || baseline_fixture(rail));
        renderer.frames(6);
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
            .find(|(_, node)| node.role() == Role::TabList && crate::has_label(&node, prefix))
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
        let mut renderer = UiTest::new()
            .viewport(1100, 1300)
            .rendered(crate::TestPipelines)
            .content(move || color_fixture(rail));
        renderer.frames(24);
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
                        .map(crate::visible_text)
                        .is_some_and(|label| label.starts_with("Navigation color role "))
            })
            .collect();
        assert_eq!(markers.len(), if rail { 36 } else { 12 });
        for (index, role) in roles.iter().enumerate() {
            let label = format!("Navigation color role {}: {role}", index + 1);
            let matching: Vec<_> = markers
                .iter()
                .filter(|(_, node)| crate::has_label(&node, label.as_str()))
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
                assert_eq!(bounds.height(), if rail { 56.0 } else { 64.0 });
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
