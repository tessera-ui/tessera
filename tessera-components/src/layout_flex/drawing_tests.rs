use std::{
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, Px, PxPosition, State,
    modifier::ModifierCapabilityExt as _, remember, tessera,
};

use crate::{
    boxed::boxed,
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs, with_pointer_input},
    row::row,
};

#[derive(Clone)]
struct Inputs {
    color: Color,
    offset: Dp,
    width: Dp,
}

thread_local! {
    static INPUTS: Cell<Option<State<Inputs>>> = const { Cell::new(None) };
    static WAKE: Cell<Option<State<u32>>> = const { Cell::new(None) };
    static CLIPPED_OFFSET: Cell<Option<State<PxPosition>>> = const { Cell::new(None) };
}

#[derive(PartialEq)]
struct ClippedOffset(State<PxPosition>);

static CLIPPED_HIT: AtomicUsize = AtomicUsize::new(usize::MAX);

impl tessera_ui::PlacementModifierNode for ClippedOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + self.0.get()
    }
}

#[tessera]
fn clipped_scene_fixture() {
    let offset = remember(|| PxPosition::ZERO);
    CLIPPED_OFFSET.set(Some(offset));
    boxed()
        .modifier(Modifier::new().size(Dp(64.0), Dp(64.0)).clip_to_bounds())
        .children(move || {
            column()
                .modifier(Modifier::new().push_placement(ClippedOffset(offset)))
                .children(|| {
                    for index in 0..128 {
                        let modifier = Modifier::new().size(Dp(32.0), Dp(16.0));
                        let modifier = if index % 2 == 0 {
                            modifier.clip_to_bounds()
                        } else {
                            modifier
                        };
                        let content = Modifier::new()
                            .fill_max_size()
                            .background(if index % 2 == 0 {
                                Color::RED
                            } else {
                                Color::BLUE
                            })
                            .semantics(SemanticsArgs {
                                label: Some(format!("Clipped row {index}")),
                                ..Default::default()
                            });
                        boxed().modifier(modifier).children(move || {
                            boxed().modifier(with_pointer_input(content.clone(), move |input| {
                                if input.cursor_position_rel.is_some_and(|position| {
                                    position.x >= 0.0
                                        && position.x < 32.0
                                        && position.y >= 0.0
                                        && position.y < 16.0
                                }) && input.pointer_changes.iter().any(|change| {
                                    matches!(
                                        change.content,
                                        tessera_ui::CursorEventContent::Pressed(_)
                                    )
                                }) {
                                    CLIPPED_HIT.store(index, Ordering::SeqCst);
                                }
                            }));
                        });
                    }
                });
        });
}

#[test]
#[ignore = "Requires an offscreen GPU adapter"]
fn clipped_translation_updates_visible_scene_branches_and_reveals_retained_pixels() {
    crate::test_utils::run_isolated_test(
        "layout_flex::drawing_tests::clipped_translation_updates_visible_scene_branches_and_reveals_retained_pixels",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(clipped_scene_fixture),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 80,
                    height: 80,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.render_frames(3);
            assert_eq!(pixel(&renderer, 4, 4), [255, 0, 0, 255]);
            CLIPPED_OFFSET
                .get()
                .unwrap()
                .set(PxPosition::from_f32_xy(0.0, -4.0));
            renderer.step_frame();
            assert!(
                renderer.diagnostics().scene_nodes_updated < 100,
                "A small scroll must not rebuild the scene for every fully clipped retained row: {:?}",
                renderer.diagnostics()
            );
            assert_eq!(pixel(&renderer, 4, 4), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 4, 14), [0, 0, 255, 255]);
            assert!(renderer.diagnostics().pointer_nodes_dispatched < 32);
            CLIPPED_OFFSET
                .get()
                .unwrap()
                .set(PxPosition::from_f32_xy(0.0, -2016.0));
            renderer.step_frame();
            assert_eq!(pixel(&renderer, 4, 4), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 4, 20), [0, 0, 255, 255]);
            renderer.pointer_press(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(4.0, 4.0),
                tessera_ui::PressKeyEventType::Left,
            );
            renderer.step_frame();
            assert_eq!(CLIPPED_HIT.load(Ordering::SeqCst), 126);
            renderer.pointer_release(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(4.0, 4.0),
                tessera_ui::PressKeyEventType::Left,
            );
            renderer.step_frame();
            let semantics = renderer.accessibility_snapshot().unwrap();
            let last = semantics
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Clipped row 127"))
                .unwrap();
            assert_eq!(last.1.bounds().unwrap().y0, 16.0);
            CLIPPED_OFFSET.get().unwrap().set(PxPosition::ZERO);
            renderer.step_frame();
            assert_eq!(pixel(&renderer, 4, 4), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 4, 20), [0, 0, 255, 255]);
            renderer.pointer_press(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(4.0, 4.0),
                tessera_ui::PressKeyEventType::Left,
            );
            renderer.step_frame();
            assert_eq!(CLIPPED_HIT.load(Ordering::SeqCst), 0);
        },
    );
}

static POINTER_CALLS: AtomicUsize = AtomicUsize::new(0);
static FIRST_HOVERED: AtomicBool = AtomicBool::new(false);

#[tessera]
fn tile(index: u32) {
    let inputs = remember(|| Inputs {
        color: Color::RED,
        offset: Dp::ZERO,
        width: Dp(8.0),
    });
    let wake = remember(|| 0_u32);
    if index == 0 {
        INPUTS.set(Some(inputs));
        WAKE.set(Some(wake));
    }
    let current = inputs.get();
    boxed().modifier(with_pointer_input(
        Modifier::new()
            .size(current.width, Dp(8.0))
            .offset(current.offset, Dp::ZERO)
            .background(current.color)
            .semantics(SemanticsArgs {
                label: Some(format!("Tile {index}")),
                ..Default::default()
            }),
        move |input| {
            let _ = wake.get();
            POINTER_CALLS.fetch_add(1, Ordering::SeqCst);
            if index == 0 {
                FIRST_HOVERED.store(
                    input.cursor_position_rel.is_some_and(|position| {
                        position.x >= 0.0
                            && position.x < input.computed_data.width.0 as f32
                            && position.y >= 0.0
                            && position.y < input.computed_data.height.0 as f32
                    }),
                    Ordering::SeqCst,
                );
            }
        },
    ));
}

#[tessera]
fn fixture() {
    row().children(|| {
        for index in 0..128 {
            tile(index);
        }
    });
}

fn pixel(renderer: &HeadlessRenderer, x: usize, y: usize) -> [u8; 4] {
    let pixels = renderer.read_pixels().expect("GPU readback");
    let offset = (y * renderer.physical_size().width as usize + x) * 4;
    pixels[offset..offset + 4].try_into().unwrap()
}

#[test]
#[ignore = "Requires an offscreen GPU adapter"]
fn full_frames_retain_clean_branches_and_update_pixels_and_pointer_dependencies() {
    crate::test_utils::run_isolated_test(
        "layout_flex::drawing_tests::full_frames_retain_clean_branches_and_update_pixels_and_pointer_dependencies",
        || {
            let mut renderer = HeadlessRenderer::new(
                Box::new(|| {
                    fixture();
                }),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 1040,
                    height: 32,
                    sample_count: 1,
                    ..Default::default()
                },
            )
            .expect("offscreen renderer");
            renderer.step_frame();
            assert_eq!(pixel(&renderer, 2, 2), [255, 0, 0, 255]);
            assert!(
                renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Tile 127"))
            );
            let inputs = INPUTS.get().unwrap();
            renderer.step_frame();
            let idle = renderer.diagnostics();
            assert_eq!(idle.measure_node_calls, 1);
            assert_eq!(idle.geometry_nodes_updated, 0);
            assert!(idle.geometry_nodes_visited <= 2);
            assert_eq!(idle.measure_dirty_children_checked, 0);
            assert_eq!(idle.paint_nodes_recorded, 0);
            assert_eq!(idle.scene_nodes_updated, 0);
            assert_eq!(idle.pointer_nodes_updated, 0);
            assert_eq!(idle.pointer_nodes_dispatched, 0);
            assert_eq!(idle.accessibility_nodes_updated, 0);

            POINTER_CALLS.store(0, Ordering::SeqCst);
            WAKE.get().unwrap().set(1);
            renderer.step_frame();
            assert_eq!(
                POINTER_CALLS.load(Ordering::SeqCst),
                1,
                "an off-hover state reader must receive input without waking unrelated handlers"
            );
            assert_eq!(renderer.diagnostics().paint_nodes_recorded, 0);

            inputs.with_mut(|inputs| inputs.color = Color::BLUE);
            renderer.step_frame();
            let paint = renderer.diagnostics();
            assert_eq!(paint.measure_node_calls, 1);
            assert_eq!(paint.geometry_nodes_updated, 0);
            assert!(paint.paint_nodes_recorded > 0 && paint.paint_nodes_recorded < 12);
            assert!(paint.scene_nodes_updated < 16);
            assert!(paint.pointer_nodes_updated < 12);
            assert!(paint.accessibility_nodes_updated < 12);
            assert_eq!(pixel(&renderer, 2, 2), [0, 0, 255, 255]);
            assert_eq!(pixel(&renderer, 10, 2), [255, 0, 0, 255]);

            inputs.with_mut(|inputs| inputs.offset = Dp(4.0));
            renderer.step_frame();
            let moved = renderer.diagnostics();
            assert_eq!(moved.measure_node_calls, 1);
            assert!(moved.geometry_nodes_updated > 0 && moved.geometry_nodes_updated < 16);
            assert!(moved.geometry_nodes_visited < 20);
            assert_eq!(pixel(&renderer, 6, 2), [0, 0, 255, 255]);
            assert_ne!(pixel(&renderer, 2, 2), [0, 0, 255, 255]);

            renderer.pointer_move(tessera_ui::MOUSE_POINTER_ID, PxPosition::new(Px(6), Px(2)));
            renderer.step_frame();
            assert!(FIRST_HOVERED.load(Ordering::SeqCst));
            assert!(renderer.diagnostics().pointer_nodes_dispatched < 4);
            assert!(renderer.diagnostics().pointer_hit_test_candidates < 64);
            renderer.pointer_move(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::new(Px(700), Px(2)),
            );
            renderer.step_frame();
            assert!(
                !FIRST_HOVERED.load(Ordering::SeqCst),
                "the previous hover target must receive its exit"
            );
            assert!(renderer.diagnostics().pointer_nodes_dispatched < 4);

            inputs.with_mut(|inputs| {
                inputs.offset = Dp::ZERO;
                inputs.width = Dp(12.0);
            });
            renderer.step_frame();
            assert!(
                renderer.diagnostics().cache_store_count > 0
                    && renderer.diagnostics().cache_store_count < 16
            );
            assert!(renderer.diagnostics().measure_dirty_children_checked < 16);
            assert_eq!(pixel(&renderer, 10, 2), [0, 0, 255, 255]);
            assert_eq!(pixel(&renderer, 14, 2), [255, 0, 0, 255]);
            let semantics = renderer.accessibility_snapshot().unwrap();
            let first = semantics
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Tile 0"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let second = semantics
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some("Tile 1"))
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert_eq!(first.x1, 12.0);
            assert_eq!(second.x0, 12.0);
        },
    );
}
