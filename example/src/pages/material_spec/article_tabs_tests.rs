//! Native article tab interaction and focus visuals.
//!
//! ## Usage
//!
//! Verify section navigation states without changing target geometry.

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use tessera_components::theme::material_theme;
use tessera_ui::{
    HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType,
    accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
};

use super::*;

thread_local! {
    static CONTENT_BOUNDS: RefCell<[Option<tessera_ui::PxRect>; 4]> = const { RefCell::new([None; 4]) };
    static SELECTED: Cell<Option<State<&'static str>>> = const { Cell::new(None) };
}

pub(super) fn record_content(index: usize, bounds: tessera_ui::PxRect) {
    CONTENT_BOUNDS.with(|value| value.borrow_mut()[index] = Some(bounds));
}

fn ink(renderer: &HeadlessRenderer, index: usize, label: bool) -> Vec<(usize, usize)> {
    let bounds = CONTENT_BOUNDS.with(|value| value.borrow()[index].unwrap());
    let rgba = renderer.read_pixels().unwrap();
    let left = bounds.x.to_f32() as usize + if label { 32 } else { 0 };
    let right = if label {
        (bounds.x + bounds.width).to_f32() as usize
    } else {
        left + 24
    };
    let mut ink = Vec::new();
    for y in bounds.y.to_f32() as usize..(bounds.y + bounds.height).to_f32() as usize {
        for x in left..right {
            let offset = (y * renderer.physical_size().width as usize + x) * 4;
            if rgba[offset..offset + 3]
                .iter()
                .all(|channel| *channel < 120)
            {
                ink.push((x, y));
            }
        }
    }
    assert!(!ink.is_empty(), "native glyph must have actual ink");
    ink
}

struct Pipelines;

impl tessera_ui::RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        tessera_components::pipelines::register_pipelines(context);
    }
}

#[tessera]
fn fixture() {
    let selected = remember(|| "Specs");
    let scroll = remember(ScrollableController::new);
    SELECTED.set(Some(selected));
    material_theme().child(move || {
        boxed()
            .alignment(Alignment::TopStart)
            .modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(tessera_ui::Color::WHITE),
            )
            .children(move || {
                article_tabs(selected, scroll).pages(
                    ARTICLE_TAB_TITLES
                        .map(|title| ArticlePage {
                            title,
                            intro: None,
                            sections: Vec::new(),
                        })
                        .to_vec(),
                );
            });
    });
}

fn tab(renderer: &HeadlessRenderer, title: &str) -> (NodeId, Node) {
    renderer
        .accessibility_snapshot()
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == Role::Tab && node.label() == Some(title))
        .expect("native article tab")
}

fn geometry(renderer: &HeadlessRenderer) -> Vec<(NodeId, tessera_ui::accesskit::Rect)> {
    ARTICLE_TAB_TITLES
        .into_iter()
        .map(|title| {
            let (id, node) = tab(renderer, title);
            (id, node.bounds().unwrap())
        })
        .collect()
}

fn center(bounds: tessera_ui::accesskit::Rect) -> PxPosition {
    PxPosition::from_f32_xy(
        ((bounds.x0 + bounds.x1) * 0.5) as f32,
        ((bounds.y0 + bounds.y1) * 0.5) as f32,
    )
}

fn pixel(renderer: &HeadlessRenderer, title: &str, y: f64) -> [u8; 3] {
    let bounds = tab(renderer, title).1.bounds().unwrap();
    let x = ((bounds.x0 + bounds.x1) * 0.5).floor() as usize;
    let y = (bounds.y0 + y).floor() as usize;
    let rgba = renderer.read_pixels().unwrap();
    let offset = (y * renderer.physical_size().width as usize + x) * 4;
    rgba[offset..offset + 3].try_into().unwrap()
}

fn assert_pixel(renderer: &HeadlessRenderer, title: &str, y: f64, expected: [u8; 3]) {
    let actual = pixel(renderer, title, y);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(b) <= 1),
        "{title} pixel at {y}: {actual:?}, expected {expected:?}"
    );
}

#[test]
#[ignore = "requires a native GPU adapter for article tab pixels and actual input"]
fn article_tab_hover_hold_selection_and_focus_preserve_targets_and_idle_work() {
    for samples in [1, 4] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1000,
                height: 160,
                sample_count: samples,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let initial = geometry(&renderer);
        assert_pixel(&renderer, "Overview", 8.0, [248, 241, 246]);
        assert_pixel(&renderer, "Specs", 8.0, [220, 218, 245]);
        let normal_icon = ink(&renderer, 0, false);
        let overview = center(tab(&renderer, "Overview").1.bounds().unwrap());
        renderer.pointer_move(MOUSE_POINTER_ID, overview);
        renderer.render_frames(4);
        assert_pixel(&renderer, "Overview", 8.0, [234, 227, 233]);
        let hover_icon = ink(&renderer, 0, false);
        assert_ne!(
            hover_icon, normal_icon,
            "hover must use the real heavier icon outline"
        );
        assert_eq!(
            geometry(&renderer),
            initial,
            "hover font grade must preserve label advances and target IDs"
        );
        renderer.pointer_press(MOUSE_POINTER_ID, overview, PressKeyEventType::Left);
        renderer.render_frames(4);
        assert_pixel(&renderer, "Overview", 8.0, [227, 220, 227]);
        assert_eq!(
            SELECTED.get().unwrap().get(),
            "Specs",
            "press must not activate"
        );
        assert_eq!(
            geometry(&renderer),
            initial,
            "pressed font grade must preserve layout"
        );
        renderer.pointer_release(MOUSE_POINTER_ID, overview, PressKeyEventType::Left);
        renderer.render_frames(14);
        assert_eq!(SELECTED.get().unwrap().get(), "Overview");
        assert_pixel(&renderer, "Overview", 8.0, [205, 203, 228]);
        let selected_icon = ink(&renderer, 0, false);
        assert!(
            selected_icon.len() > hover_icon.len(),
            "selected info uses the filled glyph"
        );
        let selected_label = ink(&renderer, 0, true);
        assert_eq!(
            geometry(&renderer),
            initial,
            "selection must preserve targets"
        );
        renderer.pointer_press(MOUSE_POINTER_ID, overview, PressKeyEventType::Left);
        renderer.render_frames(4);
        assert_pixel(&renderer, "Overview", 8.0, [197, 195, 220]);
        assert_ne!(
            ink(&renderer, 0, true),
            selected_label,
            "pressed grade must change label ink at fixed advances"
        );
        renderer.pointer_release(MOUSE_POINTER_ID, overview, PressKeyEventType::Left);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(990.0, 150.0));
        renderer.render_frames(4);
        let guidelines = tab(&renderer, "Guidelines");
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            target_node: guidelines.0,
            target_tree: TreeId::ROOT,
            action: Action::Focus,
            data: None,
        }));
        renderer.render_frames(4);
        assert_eq!(
            renderer.accessibility_snapshot().unwrap().focus,
            guidelines.0
        );
        assert_eq!(
            SELECTED.get().unwrap().get(),
            "Overview",
            "focus alone must not activate"
        );
        assert_pixel(&renderer, "Guidelines", 1.0, [28, 27, 29]);
        assert_pixel(&renderer, "Guidelines", 3.0, [255, 255, 255]);
        let guidelines_center = center(guidelines.1.bounds().unwrap());
        renderer.pointer_move(MOUSE_POINTER_ID, guidelines_center);
        renderer.render_frames(4);
        assert_pixel(&renderer, "Guidelines", 1.0, [255, 255, 255]);
        assert_pixel(&renderer, "Guidelines", 3.0, [234, 227, 233]);
        assert_eq!(
            geometry(&renderer),
            initial,
            "focus strokes must not move content"
        );
        renderer.render_frames(4);
        let diagnostics = renderer.diagnostics();
        assert_eq!(
            diagnostics.cache_store_count, 0,
            "resting tabs must not measure"
        );
        assert_eq!(
            diagnostics.paint_nodes_recorded, 0,
            "resting tabs must not repaint"
        );
    }
}

#[test]
#[ignore = "requires a native GPU adapter for timed selection motion and frame work"]
fn article_selection_pill_animates_with_clock_without_measuring_or_moving_targets() {
    for samples in [1, 4] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1000,
                height: 160,
                sample_count: samples,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let idle = renderer.diagnostics();
        assert_eq!(idle.measure_node_calls, 1);
        assert_eq!(idle.cache_hits_direct, 1);
        assert_eq!(idle.cache_store_count, 0);
        assert_eq!(idle.dirty_nodes_param, 0);
        assert_eq!(idle.geometry_nodes_updated, 0);
        let initial = geometry(&renderer);
        let bounds = tab(&renderer, "Overview").1.bounds().unwrap();
        let point = center(bounds);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.pointer_move(MOUSE_POINTER_ID, PxPosition::from_f32_xy(990.0, 150.0));
        renderer.render_frames(1);
        assert_eq!(SELECTED.get().unwrap().get(), "Overview");
        assert_pixel(&renderer, "Overview", 8.0, [248, 241, 246]);
        let edge = |renderer: &HeadlessRenderer| {
            let rgba = renderer.read_pixels().unwrap();
            let x = (bounds.x0 + bounds.width() * 0.97).floor() as usize;
            let y = ((bounds.y0 + bounds.y1) * 0.5).floor() as usize;
            let offset = (y * 1000 + x) * 4;
            <[u8; 3]>::try_from(&rgba[offset..offset + 3]).unwrap()
        };
        assert_eq!(edge(&renderer), [248, 241, 246]);
        for frame in 1..=12 {
            renderer.render_frames(1);
            let diagnostics = renderer.diagnostics();
            assert_eq!(
                diagnostics.measure_node_calls, 1,
                "animation frame{frame} must only query the cached root"
            );
            assert_eq!(
                diagnostics.cache_hits_direct, 1,
                "animation frame{frame} must retain the direct root hit"
            );
            assert_eq!(
                diagnostics.dirty_nodes_param, 0,
                "animation frame{frame} must not invalidate layout inputs"
            );
            assert_eq!(
                diagnostics.cache_store_count, 0,
                "animation frame{frame} must preserve layout snapshots"
            );
            assert_eq!(
                diagnostics.geometry_nodes_updated, 0,
                "animation frame{frame} must not place nodes"
            );
            assert_eq!(
                geometry(&renderer),
                initial,
                "animation frame{frame} must preserve targets"
            );
            if frame == 6 {
                assert_pixel(&renderer, "Overview", 8.0, [223, 221, 245]);
                assert_eq!(
                    edge(&renderer),
                    [248, 241, 246],
                    "100ms pill has not yet reached its final horizontal extent"
                );
            }
        }
        assert_pixel(&renderer, "Overview", 8.0, [220, 218, 245]);
        assert_eq!(
            edge(&renderer),
            [220, 218, 245],
            "200ms pill reaches its complete original capsule extent"
        );
        renderer.render_frames(3);
        assert_eq!(
            renderer.diagnostics().paint_nodes_recorded,
            0,
            "completed motion must unregister its driver"
        );
        assert_eq!(renderer.diagnostics().measure_node_calls, 1);
        assert_eq!(renderer.diagnostics().cache_hits_direct, 1);
        assert_eq!(renderer.diagnostics().geometry_nodes_updated, 0);
    }
}
