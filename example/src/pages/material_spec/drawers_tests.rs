//! Navigation drawer example geometry and interaction regressions.
//!
//! ## Usage
//!
//! Verify reference figures and real modal destinations.

use std::cell::{Cell, RefCell};

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

thread_local! {
    static HOST: Cell<(i32, i32)> = const { Cell::new((600, 240)) };
    static SELECTED: Cell<Option<State<usize>>> = const { Cell::new(None) };
    static OPEN: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static SCROLL: Cell<Option<State<ScrollableController>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let selected = remember(|| 0usize);
    let open = remember(|| true);
    let scroll = remember(ScrollableController::new);
    SELECTED.set(Some(selected));
    OPEN.set(Some(open));
    SCROLL.set(Some(scroll));
    material_theme().child(move || {
        column()
            .modifier(Modifier::new().fill_max_size())
            .children(move || {
                let (width, height) = HOST.get();
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
        HOST.set((width, height));
        let mut session = start_layout_test_session(layout_test(fixture).viewport_px(800, 800));
        pump_layout_test_session(&mut session, 0);
        let before = pump_layout_test_session(&mut session, 1_000_000_000);
        before.node("drawer-host").size(width, height);
        before.node("live-drawer").size(width.min(360), height);
        before.node("drawer-personal").width(width.min(360) - 24);
        let initial = before.node("drawer-personal").bounds();
        if height == 240 {
            assert!(initial.y.0 >= height);
            SCROLL.get().unwrap().with_mut(|scroll| {
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
                    tessera_ui::time::Instant::now(),
                    1,
                    content,
                    GestureState::TapCandidate,
                )],
                cursor,
            );
        }
        pump_layout_test_session(&mut session, 1_100_000_000);
        assert_eq!(SELECTED.get().unwrap().get(), 4);
        assert!(!OPEN.get().unwrap().get());
    }
}

thread_local! {
    static FIGURE_MODE: Cell<u8> = const { Cell::new(0) };
    static FIGURE_WIDTH: Cell<f64> = const { Cell::new(512.0) };
    static FIGURE_GEOMETRY: Cell<Option<State<Geometry>>> = const { Cell::new(None) };
    static PAGE_MODE: Cell<u8> = const { Cell::new(0) };
}

struct DrawerPipelines;
impl tessera_ui::RenderModule for DrawerPipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn native_figure_fixture() {
    let geometry = remember(Geometry::new);
    FIGURE_GEOMETRY.set(Some(geometry));
    material_theme().child(move || {
        boxed().children(move || {
            column()
                .modifier(Modifier::new().width(Dp(FIGURE_WIDTH.get())))
                .children(move || {
                    drawer_figure(true, FIGURE_MODE.get()).geometry(geometry);
                });
        });
    });
}

#[tessera]
fn page_figure_fixture() {
    material_theme().child(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| match PAGE_MODE.get() {
                0 => {
                    overview_figure();
                }
                1 => {
                    colors();
                }
                2 => {
                    states();
                }
                3 => {
                    differences_figure();
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

fn gpu_renderer<Output: 'static>(
    root: fn() -> Output,
    width: u32,
    height: u32,
) -> tessera_ui::HeadlessRenderer {
    let mut renderer = tessera_ui::HeadlessRenderer::new(
        Box::new(root),
        vec![Box::new(DrawerPipelines)],
        tessera_ui::HeadlessConfig {
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

fn settle_native_figure(renderer: &mut tessera_ui::HeadlessRenderer) {
    let geometry = FIGURE_GEOMETRY.get().expect("production geometry");
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
            FIGURE_WIDTH.set(width);
            FIGURE_MODE.set(mode);
            let mut renderer = gpu_renderer(native_figure_fixture, 600, 900);
            settle_native_figure(&mut renderer);
            let values = FIGURE_GEOMETRY.get().unwrap().with(space_geometry);
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
                    node.role() == Role::Label && node.label() == Some("Inbox") && !node.is_hidden()
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
                            .is_some_and(|label| label.starts_with("Modal navigation drawer"))
                })
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                (canvas.width() - FIGURE_WIDTH.get()).abs() <= 1.0,
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
        PAGE_MODE.set(page);
        let renderer = gpu_renderer(page_figure_fixture, 1024, 1600);
        let tree = renderer.accessibility_snapshot().unwrap();
        let canvas = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Group && node.label() == Some(group))
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
                            && node.label() == Some(caption)
                            && !figure_contains(canvas, node)
                    }),
                    "the overview legend below the shared canvas must name {caption}"
                );
            }
        }
        if page == 1 {
            for label in [
                "Surface container low",
                "On surface variant",
                "On secondary container",
                "Secondary container",
                "Scrim",
            ] {
                assert!(
                    tree.nodes
                        .iter()
                        .any(|(_, node)| node.role() == Role::Label && node.label() == Some(label)),
                    "missing color role {label}"
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

fn visibility_group(renderer: &tessera_ui::HeadlessRenderer, label: &str) -> AccessibleRect {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Group && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing actual visibility group {label}"))
        .1
        .bounds()
        .unwrap()
}

fn assert_visibility_panel(
    renderer: &tessera_ui::HeadlessRenderer,
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
            && node.label() == Some(format!("Standard drawer surface {number}").as_str())
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
            node.label() == Some(destination) && node.is_selected() == Some(true)
        }));
    } else {
        assert!(surface.is_none());
        assert!((content.x0 - viewport.x0).abs() <= 1.0);
    }
    assert!(tree.nodes.iter().any(|(_, node)| {
        node.role() == Role::Label
            && node.label() == Some(destination)
            && figure_contains(content, node)
    }));
}

fn tap_visibility_node(
    renderer: &mut tessera_ui::HeadlessRenderer,
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
                && node.label() == Some(label)
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
        PAGE_MODE.set(5);
        let mut renderer = gpu_renderer(page_figure_fixture, width, 1600);
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
    PAGE_MODE.set(2);
    let renderer = gpu_renderer(page_figure_fixture, 1024, 900);
    let tree = renderer.accessibility_snapshot().unwrap();
    for label in ["Enabled", "Hovered", "Focused", "Pressed"] {
        assert!(tree.nodes.iter().any(|(_, node)| node.role() == Role::Group
            && node.label() == Some(format!("Drawer {label} state presentation").as_str())));
    }
    assert_eq!(
        tree.nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Tab && node.label() == Some("Inbox"))
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
                    && node.label() == Some(format!("Drawer {label} state presentation").as_str())
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
    HOST.set((600, 720));
    let renderer = gpu_renderer(fixture, 800, 800);
    let tree = renderer.accessibility_snapshot().unwrap();
    let inbox = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Tab && node.label() == Some("Inbox"))
        .expect("first native destination")
        .0;
    assert_eq!(
        tree.focus, inbox,
        "opening modal navigation must focus Inbox, not a wrapper or later item"
    );
    assert_eq!(SELECTED.get().unwrap().get(), 0);
    assert!(OPEN.get().unwrap().get());
}

thread_local! {
    static COLD_COUNT: Cell<usize> = const { Cell::new(1) };
    static COLD_GEOMETRIES: RefCell<Vec<State<Geometry>>> = const { RefCell::new(Vec::new()) };
    static COLD_EMPTY_ON_ATTACH: Cell<bool> = const { Cell::new(false) };
}

#[tessera]
fn cold_specs_fixture() {
    let mut geometries = Vec::new();
    for _ in 0..COLD_COUNT.get() {
        geometries.push(remember(Geometry::new));
    }
    COLD_GEOMETRIES.with(|registered| {
        if registered.borrow().is_empty() {
            COLD_EMPTY_ON_ATTACH.set(
                geometries
                    .iter()
                    .all(|geometry| geometry.with(|value| value.is_empty())),
            );
        }
        registered.replace(geometries.clone());
    });
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
    COLD_COUNT.set(count);
    let mut renderer = tessera_ui::HeadlessRenderer::new(
        Box::new(cold_specs_fixture),
        vec![Box::new(DrawerPipelines)],
        tessera_ui::HeadlessConfig {
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
        let current: Vec<_> = COLD_GEOMETRIES.with(|geometries| {
            geometries
                .borrow()
                .iter()
                .map(|geometry| geometry.get())
                .collect()
        });
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
    assert!(
        COLD_EMPTY_ON_ATTACH.get(),
        "the first real production geometry must start empty"
    );
    assert!(settled, "cold geometry never settled: {history:#?}");
    let tree = renderer.accessibility_snapshot().unwrap();
    let mut canvases: Vec<_> = tree
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::Group && node.label() == Some("Modal navigation drawer anatomy")
        })
        .map(|(_, node)| node.bounds().unwrap())
        .collect();
    canvases.sort_by(|left, right| left.y0.total_cmp(&right.y0));
    assert_eq!(canvases.len(), count);
    let geometries = COLD_GEOMETRIES.with(|geometries| geometries.borrow().clone());
    let pixels = renderer.read_pixels().unwrap();
    for (index, (canvas, geometry)) in canvases.into_iter().zip(geometries).enumerate() {
        let values = geometry.with(space_geometry);
        let container = values["container"];
        let marker = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Note
                    && node.label() == Some("Drawer marker 4")
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
