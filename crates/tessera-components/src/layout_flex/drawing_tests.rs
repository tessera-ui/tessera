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

#[tessera]
fn layout_feedback_pixels_fixture(presses: tessera_ui::testing::TestProbe<usize>) {
    let width = remember(|| Dp(16.0));
    let measured_width = remember(|| Dp::ZERO);
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            let presses = presses.clone();
            boxed().modifier(with_pointer_input(
                Modifier::new()
                    .size(width.get(), Dp(16.0))
                    .on_layout_size_changed(move |size: tessera_ui::PxSize| {
                        let next = Dp::from(size.width);
                        if measured_width.get() != next {
                            measured_width.set(next);
                        }
                    }),
                move |input| {
                    for change in input.pointer_changes.iter() {
                        if matches!(change.content, tessera_ui::CursorEventContent::Pressed(_)) {
                            presses.with_mut(|count| *count += 1);
                            width.set(Dp(24.0));
                        }
                    }
                },
            ));
            boxed().modifier(
                Modifier::new()
                    .offset(Dp::ZERO, Dp(20.0))
                    .size(measured_width.get(), Dp(16.0))
                    .background(Color::RED),
            );
        });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn layout_build_feedback_presents_current_pixels_without_replaying_input() {
    crate::test_utils::run_isolated_test(
        "layout_flex::drawing_tests::layout_build_feedback_presents_current_pixels_without_replaying_input",
        || {
            let presses = tessera_ui::testing::TestProbe::default();
            let observed = presses.clone();
            let mut renderer = HeadlessRenderer::new(
                Box::new(move || layout_feedback_pixels_fixture(presses.clone())),
                vec![Box::new(crate::TesseraComponents)],
                HeadlessConfig {
                    width: 64,
                    height: 64,
                    ..Default::default()
                },
            )
            .unwrap();
            renderer.step_frame();
            assert_eq!(pixel(&renderer, 8, 24), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 20, 24), [255, 255, 255, 255]);
            renderer.pointer_press(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(8.0, 8.0),
                tessera_ui::PressKeyEventType::Left,
            );
            renderer.step_frame();
            assert_eq!(observed.get(), 1);
            assert_eq!(pixel(&renderer, 20, 24), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 28, 24), [255, 255, 255, 255]);
        },
    );
}

#[derive(Clone)]
struct Inputs {
    color: Color,
    offset: Dp,
    width: Dp,
}

#[derive(Clone, Copy, Default)]
struct FixtureHandles {
    inputs: Option<State<Inputs>>,
    wake: Option<State<u32>>,
    clipped_offset: Option<State<PxPosition>>,
    clipped_hit: Option<usize>,
    first_hovered: bool,
}

#[derive(PartialEq)]
struct ClippedOffset(State<PxPosition>);

impl tessera_ui::PlacementModifierNode for ClippedOffset {
    fn transform_position(&self, position: PxPosition) -> PxPosition {
        position + self.0.get()
    }
}

#[tessera]
fn clipped_scene_fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let offset = remember(|| PxPosition::ZERO);
    observed.with_mut(|handles| handles.clipped_offset = Some(offset));
    boxed()
        .modifier(Modifier::new().size(Dp(64.0), Dp(64.0)).clip_to_bounds())
        .children(move || {
            let observed = observed.clone();
            column()
                .modifier(Modifier::new().push_placement(ClippedOffset(offset)))
                .children(move || {
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
                        let observed = observed.clone();
                        boxed().modifier(modifier).children(move || {
                            let observed = observed.clone();
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
                                    observed.with_mut(|handles| handles.clipped_hit = Some(index));
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
            let observed = tessera_ui::testing::TestProbe::default();
            let mut renderer = HeadlessRenderer::new(
                Box::new({
                    let observed = observed.clone();
                    move || clipped_scene_fixture(observed.clone())
                }),
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
            observed
                .get()
                .clipped_offset
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
            observed
                .get()
                .clipped_offset
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
            assert_eq!(observed.get().clipped_hit, Some(126));
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
            observed.get().clipped_offset.unwrap().set(PxPosition::ZERO);
            renderer.step_frame();
            assert_eq!(pixel(&renderer, 4, 4), [255, 0, 0, 255]);
            assert_eq!(pixel(&renderer, 4, 20), [0, 0, 255, 255]);
            renderer.pointer_press(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::from_f32_xy(4.0, 4.0),
                tessera_ui::PressKeyEventType::Left,
            );
            renderer.step_frame();
            assert_eq!(observed.get().clipped_hit, Some(0));
        },
    );
}

#[tessera]
fn tile(index: u32, observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    let inputs = remember(|| Inputs {
        color: Color::RED,
        offset: Dp::ZERO,
        width: Dp(8.0),
    });
    let wake = remember(|| 0_u32);
    if index == 0 {
        observed.with_mut(|handles| handles.inputs = Some(inputs));
        observed.with_mut(|handles| handles.wake = Some(wake));
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
            if index == 0 {
                observed.with_mut(|handles| {
                    handles.first_hovered = input.cursor_position_rel.is_some_and(|position| {
                        position.x >= 0.0
                            && position.x < input.computed_data.width.0 as f32
                            && position.y >= 0.0
                            && position.y < input.computed_data.height.0 as f32
                    })
                });
            }
        },
    ));
}

#[tessera]
fn fixture(observed: tessera_ui::testing::TestProbe<FixtureHandles>) {
    row().children(move || {
        for index in 0..128 {
            tile(index, observed.clone());
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
            let observed = tessera_ui::testing::TestProbe::default();
            let mut renderer = HeadlessRenderer::new(
                Box::new({
                    let observed = observed.clone();
                    move || {
                        fixture(observed.clone());
                    }
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
            let inputs = observed.get().inputs.unwrap();
            renderer.step_frame();
            let idle = renderer.diagnostics();
            assert_eq!(
                renderer
                    .work()
                    .phase(tessera_ui::WorkPhase::Measure)
                    .count(),
                0
            );
            assert_eq!(idle.geometry_nodes_updated, 0);
            assert!(idle.geometry_nodes_visited <= 2);
            assert_eq!(idle.measure_dirty_children_checked, 0);
            assert_eq!(
                renderer.work().phase(tessera_ui::WorkPhase::Paint).count(),
                0
            );
            assert_eq!(idle.scene_nodes_updated, 0);
            assert_eq!(idle.pointer_nodes_updated, 0);
            assert_eq!(idle.pointer_nodes_dispatched, 0);
            assert_eq!(idle.accessibility_nodes_updated, 0);

            observed.get().wake.unwrap().set(1);
            renderer.step_frame();
            assert_eq!(
                renderer.diagnostics().pointer_nodes_dispatched,
                1,
                "an off-hover state reader must receive input without waking unrelated handlers"
            );
            assert_eq!(
                renderer.work().phase(tessera_ui::WorkPhase::Paint).count(),
                0
            );

            inputs.with_mut(|inputs| inputs.color = Color::BLUE);
            renderer.step_frame();
            let paint = renderer.diagnostics();
            assert_eq!(
                renderer
                    .work()
                    .phase(tessera_ui::WorkPhase::Measure)
                    .count(),
                0
            );
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
            assert_eq!(
                renderer
                    .work()
                    .phase(tessera_ui::WorkPhase::Measure)
                    .count(),
                0
            );
            assert!(moved.geometry_nodes_updated > 0 && moved.geometry_nodes_updated < 16);
            assert!(moved.geometry_nodes_visited < 20);
            assert_eq!(pixel(&renderer, 6, 2), [0, 0, 255, 255]);
            assert_ne!(pixel(&renderer, 2, 2), [0, 0, 255, 255]);

            renderer.pointer_move(tessera_ui::MOUSE_POINTER_ID, PxPosition::new(Px(6), Px(2)));
            renderer.step_frame();
            assert!(observed.get().first_hovered);
            assert!(renderer.diagnostics().pointer_nodes_dispatched < 4);
            assert!(renderer.diagnostics().pointer_hit_test_candidates < 64);
            renderer.pointer_move(
                tessera_ui::MOUSE_POINTER_ID,
                PxPosition::new(Px(700), Px(2)),
            );
            renderer.step_frame();
            assert!(
                !observed.get().first_hovered,
                "the previous hover target must receive its exit"
            );
            assert!(renderer.diagnostics().pointer_nodes_dispatched < 4);

            inputs.with_mut(|inputs| {
                inputs.offset = Dp::ZERO;
                inputs.width = Dp(12.0);
            });
            renderer.step_frame();
            assert!(
                renderer
                    .work()
                    .phase(tessera_ui::WorkPhase::Measure)
                    .count()
                    < 16,
                "changing one tile must not remeasure the other tiles"
            );
            assert!(renderer.diagnostics().cache_hits_boundary > 100);
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
