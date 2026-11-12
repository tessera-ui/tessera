use tessera_components::theme::material_theme;
use tessera_ui::{
    CursorEventContent, GestureState, PointerChange, PressKeyEventType, PxPosition,
    accesskit::{Node, Rect as AccessibleRect},
    testing::{
        layout_test, pump_layout_test_session, pump_layout_test_session_with_input,
        start_layout_test_session,
    },
};

use super::*;

#[tessera]
fn fixture(
    host: (i32, i32),
    output: tessera_ui::testing::TestProbe<
        Option<(State<usize>, State<bool>, State<ScrollableController>)>,
    >,
) {
    let selected = remember(|| 0usize);
    let open = remember(|| true);
    let scroll = remember(ScrollableController::new);
    output.set(Some((selected, open, scroll)));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                let (width, height) = host;
                boxed()
                    .modifier(
                        Modifier::new()
                            .size(Dp(f64::from(width)), Dp(f64::from(height)))
                            .semantics(SemanticsArgs {
                                test_tag: Some("drawer-host".into()),
                                merge_descendants: true,
                                ..Default::default()
                            }),
                    )
                    .children(move || {
                        modal_side_sheet_provider()
                            .position(SideSheetPosition::Start)
                            .max_width(Dp(360.0))
                            .content_padding(Dp::ZERO)
                            .is_open(open.get())
                            .on_close_request(move || open.set(false))
                            .main_content(|| {
                                boxed().modifier(Modifier::new().fill_max_size());
                            })
                            .side_sheet_content(move || {
                                live_drawer(selected, open, scroll);
                            });
                    });
            });
    });
}

#[test]
fn live_drawer_fills_available_height_and_scrolls_to_an_activatable_last_destination() {
    for (width, height) in [(600, 240), (600, 720), (320, 240), (320, 720)] {
        let output = tessera_ui::testing::TestProbe::default();
        let probe = output.clone();
        let mut session = start_layout_test_session(
            layout_test(move || fixture((width, height), probe.clone())).viewport_px(800, 800),
        );
        pump_layout_test_session(&mut session, 0);
        let before = pump_layout_test_session(&mut session, 1_000_000_000);
        before.node("drawer-host").size(width, height);
        before.node("live-drawer").size(width.min(360), height);
        before.node("drawer-personal").width(width.min(360) - 24);
        let initial = before.node("drawer-personal").bounds();
        if height == 240 {
            assert!(initial.y.0 >= height);
            output.get().unwrap().2.with_mut(|scroll| {
                scroll.set_scroll_position(PxPosition::from_f32_xy(0.0, -240.0));
            });
        }
        let after = pump_layout_test_session(&mut session, 1_016_666_667);
        let bounds = after.node("drawer-personal").bounds();
        assert!(bounds.y.0 >= 0);
        assert!(bounds.y.0 + bounds.height.0 <= height);
        let cursor = Some(PxPosition::from_f32_xy(
            bounds.x.to_f32() + bounds.width.to_f32() * 0.5,
            bounds.y.to_f32() + bounds.height.to_f32() * 0.5,
        ));
        for (index, content) in [
            CursorEventContent::Pressed(PressKeyEventType::Left),
            CursorEventContent::Released(PressKeyEventType::Left),
        ]
        .into_iter()
        .enumerate()
        {
            pump_layout_test_session_with_input(
                &mut session,
                1_000_000_000 + (index as u64 + 2) * 16_666_667,
                vec![PointerChange::new(
                    tessera_ui::current_frame_time().unwrap(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                cursor,
            );
        }
        pump_layout_test_session(&mut session, 1_100_000_000);
        assert_eq!(output.get().unwrap().0.get(), 4);
        assert!(!output.get().unwrap().1.get());
    }
}

#[tessera]
fn native_figure_fixture(
    width: f64,
    mode: u8,
    output: tessera_ui::testing::TestProbe<Option<State<Geometry>>>,
) {
    let geometry = remember(Geometry::new);
    output.set(Some(geometry));
    material_theme().child(move || {
        boxed().children(move || {
            column()
                .modifier(Modifier::new().width(Dp(width)))
                .children(move || {
                    drawer_figure(true, mode).geometry(geometry);
                });
        });
    });
}

#[tessera]
fn page_figure_fixture(page: u8) {
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(move || match page {
                0 => {
                    overview_figure();
                }
                1 => {
                    colors();
                }
                2 => {
                    states();
                }
                5 => {
                    visibility_examples();
                }
                _ => {
                    responsive_figure(3);
                }
            });
    });
}

#[derive(Clone)]
enum Scene {
    Native(
        f64,
        u8,
        tessera_ui::testing::TestProbe<Option<State<Geometry>>>,
    ),
    Page(u8),
    Live(
        (i32, i32),
        tessera_ui::testing::TestProbe<
            Option<(State<usize>, State<bool>, State<ScrollableController>)>,
        >,
    ),
}

fn gpu_renderer(scene: Scene, width: u32, height: u32) -> tessera_ui::headless::HeadlessRenderer {
    let mut renderer = tessera_ui::headless::HeadlessRenderer::new(
        Box::new(move || match scene.clone() {
            Scene::Native(width, mode, output) => {
                native_figure_fixture(width, mode, output);
            }
            Scene::Page(page) => {
                page_figure_fixture(page);
            }
            Scene::Live(viewport, output) => {
                fixture(viewport, output);
            }
        }),
        vec![Box::new(crate::TestPipelines)],
        tessera_ui::headless::HeadlessConfig {
            width,
            height,
            sample_count: 1,
            frame_time: std::time::Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("drawer reference renderer");
    renderer.render_frames(6);
    renderer
}

fn settle_native_figure(
    renderer: &mut tessera_ui::headless::HeadlessRenderer,
    geometry: State<Geometry>,
) {
    let mut previous = None;
    let mut stable = 0;
    let mut history = Vec::new();
    for _ in 0..16 {
        renderer.step_frame();
        let current = geometry.get();
        stable = if previous.as_ref() == Some(&current) {
            stable + 1
        } else {
            1
        };
        history.push(current.clone());
        previous = Some(current);
        if stable == 3 {
            return;
        }
    }
    panic!("drawer geometry did not stabilize: {history:#?}");
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn drawer_reference_rulers_measure_native_items_in_one_canonical_space() {
    for width in [512.0, 400.0, 320.0] {
        for mode in [1, 2] {
            let output = tessera_ui::testing::TestProbe::default();
            let probe = output.clone();
            let mut renderer = gpu_renderer(Scene::Native(width, mode, probe), 600, 900);
            let geometry = output.get().unwrap();
            settle_native_figure(&mut renderer, geometry);
            let values = geometry.with(space_geometry);
            let container = values["container"];
            let indicator = values["indicator"];
            let icon = values["icon"];
            let label = values["label"];
            let badge = values["badge"];
            let divider = values["divider"];
            assert_eq!(container.w, 360.0);
            assert_eq!(indicator.w, 336.0);
            assert_eq!(indicator.h, 56.0);
            assert_eq!((icon.w, icon.h), (24.0, 24.0));
            assert_eq!(icon.x - indicator.x, 16.0);
            assert_eq!(divider.x - container.x, 28.0);
            assert_eq!(container.x + container.w - divider.x - divider.w, 28.0);
            assert_eq!(indicator.x + indicator.w - badge.x - badge.w, 24.0);
            assert_eq!(container.x + container.w - badge.x - badge.w, 36.0);
            assert_eq!(divider.y - values["trash"].y - values["trash"].h, 16.0);
            let tree = renderer.accessibility_snapshot().unwrap();
            let actual_label = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Label
                        && crate::has_label(&node, "Inbox")
                        && !node.is_hidden()
                })
                .expect("native item label")
                .1
                .bounds()
                .unwrap();
            let canvas = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Group
                        && node
                            .label()
                            .map(crate::visible_text)
                            .is_some_and(|label| label.starts_with("Modal navigation drawer"))
                })
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                (canvas.width() - width).abs() <= 1.0,
                "the actual figure canvas must use the requested width {width} in mode {mode}: {canvas:?}"
            );
            let scale = width / 512.0;
            assert!(
                (actual_label.x0 - canvas.x0 - label.x * scale).abs() <= 1.0,
                "the equivalent probe must agree with the actual native label: {actual_label:?}, {label:?}"
            );
            assert!((actual_label.width() - label.w * scale).abs() <= 1.0);
            assert!((actual_label.height() - label.h * scale).abs() <= 1.0);
            assert!(label.x >= icon.x + icon.w);
            let (ruler_start, ruler_end, ruler_y) = if mode == 1 {
                (
                    container.x + container.w * 0.5 - 24.0,
                    container.x + container.w * 0.5 + 24.0,
                    container.y - 28.0,
                )
            } else {
                (
                    badge.x + badge.w + 6.0,
                    indicator.x + indicator.w - 6.0,
                    indicator.y - 20.0,
                )
            };
            let canvas_scale = canvas.width() / 512.0;
            let left = (canvas.x0 + ruler_start * canvas_scale).ceil() as usize;
            let right = (canvas.x0 + ruler_end * canvas_scale).floor() as usize;
            let ruler_y = canvas.y0 + ruler_y * canvas_scale;
            let top = ruler_y.floor() as usize - 1;
            let bottom = ruler_y.ceil() as usize + 2;
            let size = renderer.physical_size();
            assert!(right > left && right <= size.width as usize);
            assert!(bottom <= size.height as usize);
            let pixels = renderer.read_pixels().unwrap();
            let pink_columns = (left..right)
                .filter(|&x| {
                    (top..bottom).any(|y| {
                        let offset = (y * size.width as usize + x) * 4;
                        let [red, green, blue] = [
                            u16::from(pixels[offset]),
                            u16::from(pixels[offset + 1]),
                            u16::from(pixels[offset + 2]),
                        ];
                        // Allow #e85c79 to blend with the neutral canvas at
                        // antialiased edges.
                        (224..=255).contains(&red)
                            && (88..=232).contains(&green)
                            && (115..=242).contains(&blue)
                            && red >= green + 18
                            && blue >= green + 3
                            && red >= blue + 10
                    })
                })
                .count();
            assert!(
                pink_columns >= (right - left).div_ceil(2).max(3),
                "SVG ruler must paint a pink segment at width {width}, mode {mode}: pink_columns={pink_columns}, ROI=({left}..{right}, {top}..{bottom}), canvas={canvas:?}, geometry={values:?}"
            );
        }
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn overview_and_color_figures_publish_distinct_native_drawers_and_legends() {
    for (page, group, expected) in [
        (0, "Standard and modal navigation drawers", 2),
        (1, "Modal navigation drawer color roles", 9),
    ] {
        let renderer = gpu_renderer(Scene::Page(page), 1024, 1600);
        let tree = renderer.accessibility_snapshot().unwrap();
        let canvas = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Group && crate::has_label(&node, group))
            .expect("production figure canvas")
            .1
            .bounds()
            .unwrap();
        let markers: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.role() == Role::Note
                    && node
                        .label()
                        .map(crate::visible_text)
                        .is_some_and(|label| label.starts_with("Drawer marker "))
                    && node.bounds().is_some_and(|bounds| {
                        bounds.x0 >= canvas.x0
                            && bounds.x1 <= canvas.x1
                            && bounds.y0 >= canvas.y0
                            && bounds.y1 <= canvas.y1
                    })
            })
            .collect();
        assert_eq!(markers.len(), expected);
        let navigation_count = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Navigation)
            .count();
        assert_eq!(navigation_count, if page == 0 { 2 } else { 1 });
        if page == 0 {
            for caption in ["Standard navigation drawer", "Modal navigation drawer"] {
                assert!(
                    tree.nodes.iter().any(|(_, node)| {
                        node.role() == Role::Label
                            && crate::has_label(&node, caption)
                            && !figure_contains(canvas, node)
                    }),
                    "the overview legend below the shared canvas must name {caption}"
                );
            }
        }
    }
}

fn figure_contains(outer: AccessibleRect, node: &Node) -> bool {
    node.bounds().is_some_and(|inner| {
        inner.x0 >= outer.x0 - 1.0
            && inner.y0 >= outer.y0 - 1.0
            && inner.x1 <= outer.x1 + 1.0
            && inner.y1 <= outer.y1 + 1.0
    })
}

fn visibility_group(
    renderer: &tessera_ui::headless::HeadlessRenderer,
    label: &str,
) -> AccessibleRect {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Group && crate::has_label(&node, label))
        .unwrap_or_else(|| panic!("missing actual visibility group {label}"))
        .1
        .bounds()
        .unwrap()
}

fn assert_visibility_panel(
    renderer: &tessera_ui::headless::HeadlessRenderer,
    width: u32,
    number: usize,
    shown: bool,
    destination: &str,
) {
    let canvas = visibility_group(
        renderer,
        &format!("Standard drawer visibility example {number}"),
    );
    let viewport = visibility_group(renderer, &format!("Standard drawer viewport {number}"));
    let content = visibility_group(renderer, &format!("Standard drawer content {number}"));
    let scale = canvas.width() / 1024.0;
    assert!((canvas.width() - f64::from(width)).abs() <= 1.0);
    assert!((canvas.height() - 712.0 * scale).abs() <= 1.0);
    assert!((viewport.x0 - canvas.x0 - 48.0 * scale).abs() <= 1.0);
    assert!((viewport.y0 - canvas.y0 - 64.0 * scale).abs() <= 1.0);
    assert!((viewport.width() - 928.0 * scale).abs() <= 1.0);
    assert!((viewport.height() - 600.0 * scale).abs() <= 1.0);
    assert!(viewport.x1 <= canvas.x1 && viewport.y1 <= canvas.y1);
    assert!((content.y0 - viewport.y0).abs() <= 1.0);
    assert!((content.height() - viewport.height()).abs() <= 1.0);
    assert!((content.x1 - viewport.x1).abs() <= 1.0);
    let content_width = if shown { 568.0 } else { 928.0 };
    assert!((content.width() - content_width * scale).abs() <= 1.0);
    let tree = renderer.accessibility_snapshot().unwrap();
    let native: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Navigation && figure_contains(viewport, node))
        .collect();
    assert_eq!(native.len(), usize::from(shown));
    let surface = tree.nodes.iter().find(|(_, node)| {
        node.role() == Role::Group
            && crate::has_label(&node, format!("Standard drawer surface {number}").as_str())
    });
    if shown {
        let surface = surface
            .expect("visible native drawer wrapper")
            .1
            .bounds()
            .unwrap();
        assert!((surface.x0 - viewport.x0).abs() <= 1.0);
        assert!((surface.width() - 360.0 * scale).abs() <= 1.0);
        assert!((surface.height() - viewport.height()).abs() <= 1.0);
        assert!((surface.x1 - content.x0).abs() <= 1.0);
        let destinations: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Tab && figure_contains(surface, node))
            .collect();
        assert_eq!(
            destinations.len(),
            7,
            "every destination must fit the complete drawer"
        );
        assert!(destinations.iter().any(|(_, node)| {
            crate::has_label(&node, destination) && node.is_selected() == Some(true)
        }));
    } else {
        assert!(surface.is_none());
        assert!((content.x0 - viewport.x0).abs() <= 1.0);
    }
    assert!(tree.nodes.iter().any(|(_, node)| {
        node.role() == Role::Label
            && crate::has_label(&node, destination)
            && figure_contains(content, node)
    }));
}

fn tap_visibility_node(
    renderer: &mut tessera_ui::headless::HeadlessRenderer,
    label: &str,
    role: Role,
    within: Option<AccessibleRect>,
) {
    let bounds = renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| {
            node.role() == role
                && crate::has_label(&node, label)
                && within.is_none_or(|within| figure_contains(within, node))
        })
        .unwrap_or_else(|| panic!("missing actual pointer target {label}"))
        .1
        .bounds()
        .unwrap();
    let point = PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    );
    renderer.pointer_move(tessera_ui::MOUSE_POINTER_ID, point);
    renderer.pointer_press(tessera_ui::MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.pointer_release(tessera_ui::MOUSE_POINTER_ID, point, PressKeyEventType::Left);
    renderer.render_frames(6);
}

#[test]
#[ignore = "requires a native GPU adapter for real scaled destination and menu input"]
fn visibility_examples_fit_narrow_canvases_and_reflow_content_without_losing_selection() {
    for width in [600, 320] {
        let mut renderer = gpu_renderer(Scene::Page(5), width, 1600);
        assert_visibility_panel(&renderer, width, 1, true, "Inbox");
        assert_visibility_panel(&renderer, width, 2, false, "Inbox");
        let first = visibility_group(&renderer, "Standard drawer visibility example 1");
        let second = visibility_group(&renderer, "Standard drawer visibility example 2");
        assert!(second.y0 >= first.y1 + 24.0);
        let first = visibility_group(&renderer, "Standard drawer viewport 1");
        tap_visibility_node(&mut renderer, "Outbox", Role::Tab, Some(first));
        assert_visibility_panel(&renderer, width, 1, true, "Outbox");
        tap_visibility_node(
            &mut renderer,
            "Toggle standard drawer example 1",
            Role::Button,
            None,
        );
        assert_visibility_panel(&renderer, width, 1, false, "Outbox");
        tap_visibility_node(
            &mut renderer,
            "Toggle standard drawer example 1",
            Role::Button,
            None,
        );
        assert_visibility_panel(&renderer, width, 1, true, "Outbox");
        tap_visibility_node(
            &mut renderer,
            "Toggle standard drawer example 2",
            Role::Button,
            None,
        );
        assert_visibility_panel(&renderer, width, 2, true, "Inbox");
        let second = visibility_group(&renderer, "Standard drawer viewport 2");
        tap_visibility_node(&mut renderer, "Favorites", Role::Tab, Some(second));
        assert_visibility_panel(&renderer, width, 2, true, "Favorites");
        tap_visibility_node(
            &mut renderer,
            "Toggle standard drawer example 2",
            Role::Button,
            None,
        );
        assert_visibility_panel(&renderer, width, 2, false, "Favorites");
        assert_visibility_panel(&renderer, width, 1, true, "Outbox");
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn static_state_presentations_render_hover_focus_and_press_without_changing_selection() {
    let renderer = gpu_renderer(Scene::Page(2), 1024, 900);
    let tree = renderer.accessibility_snapshot().unwrap();
    for label in ["Enabled", "Hovered", "Focused", "Pressed"] {
        assert!(tree.nodes.iter().any(|(_, node)| node.role() == Role::Group
            && crate::has_label(&node, format!("Drawer {label} state presentation").as_str())));
    }
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, "Inbox"))
            .count(),
        4
    );
    let pixels = renderer.read_pixels().unwrap();
    let panel = |label: &str| {
        let bounds = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Group
                    && crate::has_label(
                        &node,
                        format!("Drawer {label} state presentation").as_str(),
                    )
            })
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("missing {label} state presentation bounds"));
        (bounds.x0, bounds.y0)
    };
    // Samples sit on the Favorites row, which carries the presented state.
    let sample = |label: &str| {
        let (x0, y0) = panel(label);
        let (x, y) = ((x0 + 360.0) as usize, (y0 + 304.0) as usize);
        let offset = (y * 1024 + x) * 4;
        pixels[offset..offset + 3].to_vec()
    };
    let enabled = sample("Enabled");
    assert_ne!(enabled, sample("Hovered"), "hover layer must render");
    assert_ne!(enabled, sample("Focused"), "focus presentation must render");
    assert_ne!(
        sample("Hovered"),
        sample("Pressed"),
        "pressed ripple must differ from hover"
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn modal_drawer_initial_focus_targets_the_first_actual_destination() {
    let output = tessera_ui::testing::TestProbe::default();
    let probe = output.clone();
    let renderer = gpu_renderer(Scene::Live((600, 720), probe), 800, 800);
    let tree = renderer.accessibility_snapshot().unwrap();
    let inbox = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Tab && crate::has_label(&node, "Inbox"))
        .expect("first native destination")
        .0;
    assert_eq!(
        tree.focus, inbox,
        "opening modal navigation must focus Inbox, not a wrapper or later item"
    );
    assert_eq!(output.get().unwrap().0.get(), 0);
    assert!(output.get().unwrap().1.get());
}

#[tessera]
fn cold_specs_fixture(count: usize, output: tessera_ui::testing::TestProbe<Vec<State<Geometry>>>) {
    let mut geometries = Vec::new();
    for _ in 0..count {
        geometries.push(remember(Geometry::new));
    }
    output.set(geometries.clone());
    material_theme().child(move || {
        let geometries = geometries.clone();
        column()
            .modifier(Modifier::new().width(Dp(1024.0)))
            .children(move || {
                for geometry in &geometries {
                    drawer_figure(true, 0).geometry(*geometry);
                    gap(Dp(24.0));
                }
            });
    });
}

fn enter_cold_child(test: &str) -> bool {
    const FLAG: &str = "TESSERA_DRAWER_COLD_CHILD";
    if std::env::var(FLAG).ok().as_deref() == Some(test) {
        return true;
    }
    let module = module_path!()
        .split_once("::")
        .map_or(module_path!(), |(_, module)| module);
    let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            &format!("{module}::{test}"),
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(FLAG, test)
        .output()
        .expect("fresh cold-start test process");
    assert!(
        output.status.success(),
        "cold child failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

fn check_cold_specs_svg(count: usize) {
    let output = tessera_ui::testing::TestProbe::default();
    let probe = output.clone();
    let mut renderer = tessera_ui::headless::HeadlessRenderer::new(
        Box::new(move || cold_specs_fixture(count, probe.clone())),
        vec![Box::new(crate::TestPipelines)],
        tessera_ui::headless::HeadlessConfig {
            width: 1024,
            height: 1100,
            sample_count: 1,
            frame_time: std::time::Duration::from_nanos(16_666_667),
            ..Default::default()
        },
    )
    .expect("cold drawer renderer");
    let mut previous = None;
    let mut stable = 0;
    let mut history = Vec::new();
    let mut settled = false;
    for _ in 0..16 {
        renderer.step_frame();
        let current: Vec<_> =
            output.with(|geometries| geometries.iter().map(|geometry| geometry.get()).collect());
        let complete = current.len() == count
            && current.iter().all(|geometry| {
                ["figure", "container", "indicator", "icon", "label", "badge"]
                    .iter()
                    .all(|key| geometry.contains_key(key))
            });
        stable = if complete && previous.as_ref() == Some(&current) {
            stable + 1
        } else if complete {
            1
        } else {
            0
        };
        history.push((renderer.frame_index(), current.clone()));
        previous = Some(current);
        if stable == 3 {
            settled = true;
            break;
        }
    }
    assert!(settled, "cold geometry never settled: {history:#?}");
    let tree = renderer.accessibility_snapshot().unwrap();
    let mut canvases: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::Group && crate::has_label(&node, "Modal navigation drawer anatomy")
        })
        .map(|(_, node)| node.bounds().unwrap())
        .collect();
    canvases.sort_by(|left, right| left.y0.total_cmp(&right.y0));
    assert_eq!(canvases.len(), count);
    let geometries = output.get();
    let pixels = renderer.read_pixels().unwrap();
    for (index, (canvas, geometry)) in canvases.into_iter().zip(geometries).enumerate() {
        let values = geometry.with(space_geometry);
        let container = values["container"];
        let marker = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Note
                    && crate::has_label(&node, "Drawer marker 4")
                    && node.bounds().is_some_and(|bounds| {
                        bounds.x0 >= canvas.x0
                            && bounds.x1 <= canvas.x1
                            && bounds.y0 >= canvas.y0
                            && bounds.y1 <= canvas.y1
                    })
            })
            .expect("active-indicator callout")
            .1
            .bounds()
            .unwrap();
        let center = (marker.x0 + marker.x1) * 0.5;
        let top = (canvas.y0 + container.y + 8.0).ceil() as usize;
        let bottom = (canvas.y0 + container.y + 28.0).floor() as usize;
        assert!(marker.y1 < top as f64);
        let mut ink = 0;
        for y in top..bottom {
            for x in (center.floor() as usize - 2)..=(center.ceil() as usize + 2) {
                let offset = (y * 1024 + x) * 4;
                if pixels[offset..offset + 3]
                    .iter()
                    .all(|channel| *channel < 185)
                {
                    ink += 1;
                }
            }
        }
        assert!(
            ink >= 6,
            "cold Specs figure {index} must paint its SVG leader on the empty header background: ink={ink}, marker={marker:?}, container={container:?}, history={history:#?}"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter; runs in a fresh child process"]
fn cold_single_specs_figure_paints_its_first_svg_leader() {
    if enter_cold_child("cold_single_specs_figure_paints_its_first_svg_leader") {
        check_cold_specs_svg(1);
    }
}

#[test]
#[ignore = "requires a native GPU adapter; runs in a fresh child process"]
fn cold_two_specs_figures_each_paint_their_svg_leaders() {
    if enter_cold_child("cold_two_specs_figures_each_paint_their_svg_leaders") {
        check_cold_specs_svg(2);
    }
}
