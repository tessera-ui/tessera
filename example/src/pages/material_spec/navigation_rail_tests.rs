//! Navigation rail reference color and native rendering regressions.
//!
//! ## Usage
//!
//! Verify unmodified theme colors separately from reference role mappings.

use std::time::Duration;

use tessera_ui::{
    Color, HeadlessConfig, HeadlessRenderer,
    accesskit::{Action, ActionRequest, Node, NodeId, Rect, TreeId},
};

use super::*;

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn unmodified_dark_surface_fixture() {
    material_theme()
        .theme(|| MaterialTheme::from_seed(ACCENT, true))
        .child(|| {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
                .children(move || {
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(16.0), Dp(16.0))
                                .size(Dp(64.0), Dp(64.0))
                                .background(scheme.surface),
                        )
                        .children(|| {});
                    boxed()
                        .modifier(
                            Modifier::new()
                                .offset(Dp(112.0), Dp(16.0))
                                .size(Dp(96.0), Dp(560.0)),
                        )
                        .children(|| {
                            navigation_rail().content(|| {
                                navigation_rail_item()
                                    .label("Home")
                                    .selected(true)
                                    .icon(|| {
                                        destination_icon(0, true);
                                    });
                            });
                        });
                });
        });
}

#[tessera]
fn color_reference_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(rail_color_pair);
    });
}

fn renderer(root: fn(), width: u32, height: u32) -> HeadlessRenderer {
    let mut renderer = HeadlessRenderer::new(
        Box::new(root),
        vec![Box::new(Pipelines)],
        HeadlessConfig {
            width,
            height,
            sample_count: 1,
            frame_time: Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("native GPU adapter");
    renderer.render_frames(6);
    renderer
}

fn rgba(pixels: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    pixels[offset..offset + 4].try_into().unwrap()
}

fn group(nodes: &[(NodeId, Node)], label: &str) -> Rect {
    nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Group && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing group {label}"))
        .1
        .bounds()
        .unwrap()
}

fn settled_reference_nodes(renderer: &mut HeadlessRenderer) -> Vec<(NodeId, Node)> {
    let start = renderer.frame_index();
    let mut previous = None;
    let mut stable_frames = 0;
    let mut history = Vec::new();
    for _ in 0..60 {
        renderer.step_frame();
        let nodes = renderer.accessibility_snapshot().unwrap().nodes;
        let mut geometry: Vec<_> = nodes
            .iter()
            .filter(|(_, node)| matches!(node.role(), Role::Group | Role::Tab))
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
        let elapsed = Duration::from_nanos((renderer.frame_index() - start) * 16_666_667);
        // The native rail's expansion transition lasts 300ms; its wrapper has
        // fixed width.
        if elapsed >= Duration::from_millis(300) && stable_frames >= 3 {
            return nodes;
        }
    }
    panic!("native rail geometry did not settle within 60 virtual frames: {history:#?}");
}

fn contains(outer: Rect, node: &Node) -> bool {
    node.bounds().is_some_and(|inner| {
        inner.x0 >= outer.x0 && inner.x1 <= outer.x1 && inner.y0 >= outer.y0 && inner.y1 <= outer.y1
    })
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn unmodified_dark_theme_surface_and_native_rail_preserve_chromatic_channels() {
    let renderer = renderer(unmodified_dark_surface_fixture, 256, 600);
    let pixels = renderer.read_pixels().unwrap();
    let patch = rgba(&pixels, 256, 48, 48);
    let native = rgba(&pixels, 256, 160, 520);
    eprintln!(
        "unmodified Material 2025 dark surface patch RGBA={patch:?}, native rail RGBA={native:?}"
    );
    assert_eq!(patch, [15, 13, 18, 255], "unmodified surface patch");
    assert_eq!(native, [15, 13, 18, 255], "unmodified native rail surface");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn color_references_keep_independent_theme_backgrounds_containers_and_native_selection() {
    let mut renderer = renderer(color_reference_fixture, 1024, 1320);
    let initial_nodes = renderer.accessibility_snapshot().unwrap().nodes;
    let initial_pixels = renderer.read_pixels().unwrap();
    for mode in ["Light", "Dark"] {
        let rail = group(&initial_nodes, &format!("{mode} expanded reference rail"));
        let destination = initial_nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Tab && contains(rail, node))
            .map(|(_, node)| node.bounds());
        let sample = rgba(
            &initial_pixels,
            1024,
            (rail.x0 + rail.width() * 0.5) as usize,
            (rail.y1 - 20.0) as usize,
        );
        eprintln!(
            "initial frame={} {mode} expanded native destination={destination:?}, container sample={sample:?}",
            renderer.frame_index()
        );
    }
    let nodes = settled_reference_nodes(&mut renderer);
    let pixels = renderer.read_pixels().unwrap();
    let mut targets = Vec::new();
    for (mode, background, container) in [
        ("Light", [253, 247, 254, 255], [242, 236, 245, 255]),
        ("Dark", [20, 18, 24, 255], [33, 31, 38, 255]),
    ] {
        let half = group(&nodes, &format!("{mode} navigation rail colors"));
        assert_eq!(
            half.width(),
            1024.0,
            "{mode} reference spans the full width"
        );
        let actual = rgba(&pixels, 1024, half.x0 as usize + 12, half.y0 as usize + 100);
        eprintln!("{mode} reference background RGBA={actual:?}, expected={background:?}");
        assert_eq!(actual, background, "{mode} reference uses its own surface");
        for (variant, width) in [("collapsed", 96.0), ("expanded", 220.0)] {
            let rail = group(&nodes, &format!("{mode} {variant} reference rail"));
            assert_eq!(rail.width(), width);
            assert_eq!(rail.height(), 560.0);
            let actual = rgba(
                &pixels,
                1024,
                (rail.x0 + rail.width() * 0.5) as usize,
                (rail.y1 - 20.0) as usize,
            );
            eprintln!(
                "{mode} {variant} reference container RGBA={actual:?}, expected={container:?}"
            );
            let mut destinations: Vec<_> = nodes
                .iter()
                .filter(|(_, node)| node.role() == Role::Tab && contains(rail, node))
                .collect();
            destinations.sort_by(|a, b| {
                let (a, b) = (a.1.bounds().unwrap(), b.1.bounds().unwrap());
                a.y0.total_cmp(&b.y0)
            });
            assert_eq!(destinations.len(), 4);
            eprintln!(
                "settled frame={} {mode} {variant} group={rail:?}, native destinations={:?}",
                renderer.frame_index(),
                destinations
                    .iter()
                    .map(|(_, node)| node.bounds())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                actual, container,
                "{mode} {variant} optional surface container"
            );
            for (_, destination) in &destinations {
                assert_eq!(
                    destination.bounds().unwrap().width(),
                    width,
                    "{mode} {variant}: actual native destination geometry must match the settled variant"
                );
            }
            // The second destination starts unselected, so it can prove focus
            // and activation stay independent.
            targets.push(destinations[1].0);
        }
    }
    for saved in targets {
        for action in [Action::Focus, Action::Click] {
            assert!(renderer.dispatch_accessibility_action(ActionRequest {
                action,
                target_tree: TreeId::ROOT,
                target_node: saved,
                data: None,
            }));
            let nodes = settled_reference_nodes(&mut renderer);
            let saved = nodes.iter().find(|(id, _)| *id == saved).unwrap();
            assert_eq!(
                saved.1.is_selected(),
                Some(action == Action::Click),
                "focus must remain independent of destination activation"
            );
        }
    }
}
