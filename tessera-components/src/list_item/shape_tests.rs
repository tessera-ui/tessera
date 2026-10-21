//! Segmented list geometry and interaction motion regressions.
//!
//! ## Usage
//!
//! Keep grouped rows responsive and restore their resting appearance.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, FocusManager, HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, Modifier,
    PressKeyEventType, PxPosition, State,
    accesskit::{Action, ActionRequest, TreeId},
    remember, tessera, use_context,
};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::{RoundedCorner, Shape},
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    theme::{MaterialTheme, material_theme},
};

use super::{ListItemPosition, ListItemShapes, list_item};

#[derive(Clone, Copy, PartialEq)]
struct Settings {
    selected: bool,
    enabled: bool,
    dragged: bool,
    fixed: bool,
    custom: bool,
    position: ListItemPosition,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
    static CLICKS: Cell<usize> = const { Cell::new(0) };
}

#[tessera]
fn fixture() {
    let settings = remember(|| Settings {
        selected: false,
        enabled: true,
        dragged: false,
        fixed: false,
        custom: false,
        position: ListItemPosition::First,
    });
    SETTINGS.set(Some(settings));
    material_theme()
        .theme(|| {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.surface = Color::RED;
            theme.color_scheme.on_surface = Color::RED;
            theme.color_scheme.on_surface_variant = Color::RED;
            theme.color_scheme.secondary_container = Color::RED;
            theme.color_scheme.on_secondary_container = Color::RED;
            theme.color_scheme.tertiary_container = Color::RED;
            theme.color_scheme.on_tertiary_container = Color::RED;
            theme
        })
        .child(move || {
            let theme = use_context::<MaterialTheme>().unwrap().get();
            column()
                .modifier(
                    Modifier::new()
                        .background(Color::WHITE)
                        .padding_all(Dp(16.0)),
                )
                .children(move || {
                    for (index, position) in [
                        ListItemPosition::First,
                        ListItemPosition::Middle,
                        ListItemPosition::Last,
                        ListItemPosition::Single,
                        ListItemPosition::Standalone,
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        if index > 0 {
                            spacer().modifier(
                                Modifier::new().height(super::ListItemDefaults::SEGMENTED_GAP),
                            );
                        }
                        let value = settings.get();
                        let shapes = if index == 0 && value.custom {
                            let mut shapes = ListItemShapes::from_theme(&theme);
                            shapes.pressed_shape = Shape::rounded_rectangle(Dp(2.0));
                            Some(shapes)
                        } else {
                            None
                        };
                        list_item()
                            .modifier(Modifier::new().width(Dp(96.0)).semantics(SemanticsArgs {
                                test_tag: Some(format!("item-{index}")),
                                ..Default::default()
                            }))
                            .accessibility_label(format!("Item {index}"))
                            .position(if index == 0 { value.position } else { position })
                            .selected(index == 0 && value.selected)
                            .enabled(value.enabled)
                            .dragged(index == 0 && value.dragged)
                            .shape_optional((index == 0 && value.fixed).then_some(Shape::RECTANGLE))
                            .shapes_optional(shapes)
                            .on_click(|| CLICKS.set(CLICKS.get() + 1));
                    }
                    for radius in [4, 12, 16, 0] {
                        spacer().modifier(
                            Modifier::new().height(super::ListItemDefaults::SEGMENTED_GAP),
                        );
                        surface()
                            .modifier(Modifier::new().size(Dp(96.0), Dp(56.0)))
                            .style(SurfaceStyle::Filled { color: Color::RED })
                            .shape(reference_shape(radius))
                            .accessibility_label(format!("Reference {radius}"));
                    }
                    spacer()
                        .modifier(Modifier::new().height(super::ListItemDefaults::SEGMENTED_GAP));
                    surface()
                        .modifier(Modifier::new().size(Dp(96.0), Dp(56.0)))
                        .style(SurfaceStyle::Filled { color: Color::RED })
                        .shape(super::ListItemDefaults::shape())
                        .accessibility_label("Default shape");
                });
        });
}

fn reference_shape(radius: i32) -> Shape {
    let corner = RoundedCorner::manual(Dp(f64::from(radius)), 2.0);
    Shape::RoundedRectangle {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    }
}

fn bounds(renderer: &HeadlessRenderer, index: usize) -> [i64; 4] {
    bounds_label(renderer, &format!("Item {index}"))
}

fn bounds_label(renderer: &HeadlessRenderer, label: &str) -> [i64; 4] {
    let snapshot = renderer.accessibility_snapshot().unwrap();
    let node = &snapshot
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap()
        .1;
    let bounds = node.bounds().unwrap();
    [
        bounds.x0 as i64,
        bounds.y0 as i64,
        bounds.width() as i64,
        bounds.height() as i64,
    ]
}

fn patch(renderer: &HeadlessRenderer, index: usize, bottom: bool, scale: f64) -> Vec<u8> {
    patch_label(renderer, &format!("Item {index}"), bottom, scale)
}

fn patch_label(renderer: &HeadlessRenderer, label: &str, bottom: bool, scale: f64) -> Vec<u8> {
    let pixels = renderer.read_pixels().unwrap();
    let [x, y, _, height] = bounds_label(renderer, label);
    let mut result = Vec::new();
    for dy in 0..(16.0 * scale) as i64 {
        for dx in 0..(16.0 * scale) as i64 {
            let py = if bottom { y + height - 1 - dy } else { y + dy };
            let offset =
                ((py as u32 * renderer.physical_size().width + (x + dx) as u32) * 4) as usize;
            result.extend_from_slice(&pixels[offset..offset + 4]);
        }
    }
    result
}

fn equal_patch(actual: &[u8], expected: &[u8]) {
    assert_eq!(actual.len(), expected.len());
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(*b) <= 2)
    );
}

fn equal_feedback_patch(actual: &[u8], expected: &[u8]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.chunks_exact(4).zip(expected.chunks_exact(4)) {
        // Ripple and state-layer passes add coverage at partially covered
        // edges.
        let tolerance = if expected[1] > 0 && expected[1] < 255 {
            16
        } else {
            2
        };
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(*b) <= tolerance),
            "{actual:?} != {expected:?}"
        );
    }
}

fn ink(patch: &[u8]) -> u64 {
    patch
        .chunks_exact(4)
        .map(|pixel| u64::from(255 - pixel[1]))
        .sum()
}

fn check(samples: u32) {
    for scale in [1.0, 1.5] {
        CLICKS.set(0);
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 320,
                height: 1000,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let settings = SETTINGS.get().unwrap();
        for index in 0..5 {
            let current = bounds(&renderer, index);
            assert_eq!(
                [current[2], current[3]],
                [(96.0 * scale) as i64, (56.0 * scale) as i64]
            );
            if index > 0 {
                let previous = bounds(&renderer, index - 1);
                assert_eq!(current[1] - previous[1] - previous[3], (2.0 * scale) as i64);
            }
        }
        let upper = patch(&renderer, 3, false, scale);
        let inner = patch(&renderer, 1, false, scale);
        equal_patch(
            &upper,
            &patch_label(&renderer, "Reference 16", false, scale),
        );
        equal_patch(&inner, &patch_label(&renderer, "Reference 4", false, scale));
        equal_patch(
            &inner,
            &patch_label(&renderer, "Default shape", false, scale),
        );
        assert!(ink(&inner) > ink(&upper));
        equal_patch(&patch(&renderer, 0, false, scale), &upper);
        equal_patch(&patch(&renderer, 0, true, scale), &inner);
        equal_patch(&patch(&renderer, 2, false, scale), &inner);
        equal_patch(&patch(&renderer, 2, true, scale), &upper);
        equal_patch(&patch(&renderer, 4, false, scale), &inner);
        let idle = super::ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(super::ITEM_BUILDS.get(), idle);
        let [x, y, width, height] = bounds(&renderer, 0);
        let point = PxPosition::from_f32_xy((x + width / 2) as f32, (y + height / 2) as f32);
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.render_frames(60);
        let hover = patch(&renderer, 0, true, scale);
        equal_feedback_patch(
            &hover,
            &patch_label(&renderer, "Reference 12", false, scale),
        );
        assert!(ink(&hover) < ink(&inner) && ink(&hover) > ink(&upper));
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(3);
        let moving = patch(&renderer, 0, true, scale);
        assert_ne!(moving, hover);
        renderer.render_frames(60);
        equal_feedback_patch(&patch(&renderer, 0, true, scale), &upper);
        let held = super::ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(super::ITEM_BUILDS.get(), held);
        FocusManager::current().clear_focus(true);
        renderer.pointer_release(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        assert_eq!(CLICKS.get(), 1);
        equal_patch(&patch(&renderer, 0, true, scale), &hover);
        let outside = PxPosition::from_f32_xy(300.0, 780.0);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        FocusManager::current().clear_focus(true);
        renderer.render_frames(60);
        equal_patch(&patch(&renderer, 0, true, scale), &inner);
        let resting = super::ITEM_BUILDS.get();
        renderer.render_frames(8);
        assert_eq!(super::ITEM_BUILDS.get(), resting);
        for field in [0, 1] {
            settings.with_mut(|value| {
                value.selected = field == 0;
                value.dragged = field == 1;
            });
            renderer.render_frames(60);
            equal_patch(&patch(&renderer, 0, true, scale), &upper);
        }
        settings.with_mut(|value| {
            value.selected = false;
            value.dragged = false;
            value.position = ListItemPosition::Last;
        });
        renderer.render_frames(60);
        equal_patch(&patch(&renderer, 0, false, scale), &inner);
        equal_patch(&patch(&renderer, 0, true, scale), &upper);
        settings.with_mut(|value| {
            value.position = ListItemPosition::First;
            value.selected = false;
        });
        renderer.render_frames(60);
        settings.with_mut(|value| value.selected = true);
        renderer.render_frames(3);
        settings.with_mut(|value| value.selected = false);
        renderer.render_frames(60);
        equal_patch(&patch(&renderer, 0, true, scale), &inner);
        settings.with_mut(|value| {
            value.custom = true;
            value.selected = true;
        });
        renderer.render_frames(60);
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        assert!(ink(&patch(&renderer, 0, true, scale)) > ink(&inner));
        settings.with_mut(|value| value.enabled = false);
        renderer.render_frames(2);
        equal_patch(&patch(&renderer, 0, true, scale), &upper);
        renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
        settings.with_mut(|value| {
            value.enabled = true;
            value.fixed = true;
        });
        renderer.pointer_move(MOUSE_POINTER_ID, point);
        renderer.render_frames(60);
        let fixed = patch(&renderer, 0, true, scale);
        equal_feedback_patch(&fixed, &patch_label(&renderer, "Reference 0", false, scale));
        assert!(ink(&fixed) > ink(&inner));
        renderer.pointer_press(MOUSE_POINTER_ID, point, PressKeyEventType::Left);
        renderer.render_frames(60);
        equal_feedback_patch(&patch(&renderer, 0, true, scale), &fixed);
        renderer.pointer_release(MOUSE_POINTER_ID, outside, PressKeyEventType::Left);
        renderer.pointer_move(MOUSE_POINTER_ID, outside);
        FocusManager::current().clear_focus(true);
        settings.with_mut(|value| {
            value.fixed = false;
            value.custom = false;
            value.selected = false;
        });
        renderer.render_frames(60);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        let id = snapshot
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Item 0"))
            .unwrap()
            .0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: id,
            data: None
        }));
        renderer.render_frames(60);
        equal_feedback_patch(&patch(&renderer, 0, true, scale), &upper);
        FocusManager::current().clear_focus(true);
        renderer.render_frames(60);
        equal_patch(&patch(&renderer, 0, true, scale), &inner);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn grouped_list_shapes_follow_input_and_finish_motion_msaa1() {
    crate::test_utils::run_isolated_test(
        "list_item::shape_tests::grouped_list_shapes_follow_input_and_finish_motion_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn grouped_list_shapes_follow_input_and_finish_motion_msaa4() {
    crate::test_utils::run_isolated_test(
        "list_item::shape_tests::grouped_list_shapes_follow_input_and_finish_motion_msaa4",
        || check(4),
    );
}
