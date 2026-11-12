//! Native shape coverage and shared-upload regressions.
//!
//! ## Usage
//!
//! Verify clipped and scaled surfaces across repeated draw batches.

use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, State,
    modifier::ModifierCapabilityExt as _,
    remember, scaled_box, tessera,
    testing::{TestProbe, UiTest},
};

use crate::{boxed::boxed, modifier::ModifierExt as _, shape_def::Shape};

const BATCH_COUNT: usize = 300;
const VIEWPORT_WIDTH: u32 = 8 + BATCH_COUNT as u32 * 16;

#[derive(Clone, Copy, PartialEq)]
struct PaintPatch(State<Color>);

impl DrawModifierNode for PaintPatch {
    fn draw(&self, ctx: &mut DrawModifierContext<'_, '_>, content: &mut dyn DrawModifierContent) {
        ctx.render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(super::command::ShapeCommand::Rect {
                color: self.0.get(),
                corner_radii: [8.0; 4],
                corner_g2: [2.0; 4],
            });
        content.draw(ctx.render_input);
    }
}

#[tessera]
fn damage_fixture(probe: TestProbe<Option<DamageHandles>>) {
    let color = remember(|| Color::BLACK);
    let position = remember(|| [Dp(8.25), Dp(9.5)]);
    let visible = remember(|| true);
    let backdrop = remember(|| true);
    probe.set(Some(DamageHandles {
        color,
        position,
        visible,
        backdrop,
    }));
    boxed()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            if backdrop.get() {
                boxed().modifier(Modifier::new().fill_max_size().background(Color::WHITE));
            }
            if visible.get() {
                boxed().modifier(
                    Modifier::new()
                        .offset(position.get()[0], position.get()[1])
                        .size(Dp(32.0), Dp(24.0))
                        .clip_shape(Shape::rounded_rectangle(Dp(6.0)))
                        .push_draw(PaintPatch(color)),
                );
            }
            if backdrop.get() {
                for index in 0..20 {
                    boxed().modifier(
                        Modifier::new()
                            .offset(
                                Dp(64.0 + (index % 5) as f64 * 24.0),
                                Dp(8.0 + (index / 5) as f64 * 24.0),
                            )
                            .size(Dp(20.0), Dp(20.0))
                            .background(Color::BLACK.with_alpha(0.4)),
                    );
                }
            }
        });
}

#[derive(Clone, Copy, PartialEq)]
struct DamageHandles {
    color: State<Color>,
    position: State<[Dp; 2]>,
    visible: State<bool>,
    backdrop: State<bool>,
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn damage_redraw_matches_a_fresh_target_and_skips_unaffected_shapes() {
    crate::test_utils::run_isolated_test(
        "pipelines::shape::coverage_tests::damage_redraw_matches_a_fresh_target_and_skips_unaffected_shapes",
        || {
            for sample_count in [1, 4] {
                let probe = TestProbe::default();
                let observed = probe.clone();
                let mut renderer = UiTest::new()
                    .viewport(200, 128)
                    .samples(sample_count)
                    .rendered(crate::TesseraComponents)
                    .content(move || damage_fixture(probe.clone()));
                renderer.frames(3);
                assert_eq!(
                    renderer
                        .frame_graphics_work()
                        .group("shape_batches")
                        .buffer_uploads,
                    0,
                    "a static scene keeps its rendered pixels without uploading shape batches"
                );
                let handles = observed.get().unwrap();
                for ink in [Color::BLACK.with_alpha(0.25), Color::WHITE, Color::BLACK] {
                    handles.color.set(ink);
                    renderer.frames(1);
                    let partial = renderer.read_pixels();
                    let partial_bytes = renderer
                        .frame_graphics_work()
                        .group("shape_batches")
                        .buffer_uploaded_bytes;
                    assert!(partial_bytes > 0, "the changed patch must actually redraw");
                    // Recreating the target discards retained pixels and forces a full draw.
                    renderer.resize(200, 128);
                    renderer.frames(1);
                    assert_eq!(
                        partial,
                        renderer.read_pixels(),
                        "fractional clips and translucent pixels match full rendering at MSAA{sample_count}"
                    );
                    assert!(
                        partial_bytes
                            < renderer
                                .frame_graphics_work()
                                .group("shape_batches")
                                .buffer_uploaded_bytes,
                        "unaffected sibling shapes must not be re-encoded"
                    );
                }
                for position in [[Dp(35.5), Dp(40.25)], [Dp(-16.75), Dp(12.5)]] {
                    handles.position.set(position);
                    renderer.frames(1);
                    let moved = renderer.read_pixels();
                    renderer.resize(200, 128);
                    renderer.frames(1);
                    assert_eq!(
                        moved,
                        renderer.read_pixels(),
                        "moving a clipped patch clears its old pixels at MSAA{sample_count}"
                    );
                }
                handles.visible.set(false);
                renderer.frames(1);
                let removed = renderer.read_pixels();
                renderer.resize(200, 128);
                renderer.frames(1);
                assert_eq!(
                    removed,
                    renderer.read_pixels(),
                    "removing a patch restores the underlying scene at MSAA{sample_count}"
                );
                handles.backdrop.set(false);
                renderer.frames(1);
                assert!(
                    renderer.read_pixels().iter().all(|&channel| channel == 0),
                    "removing the last draws clears the retained target at MSAA{sample_count}"
                );
                renderer.resize(200, 128);
                renderer.frames(1);
                assert!(
                    renderer.read_pixels().iter().all(|&channel| channel == 0),
                    "a fresh empty target is transparent at MSAA{sample_count}"
                );
            }
        },
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn resized_scene_uses_logical_viewport_for_fractional_clips() {
    crate::test_utils::run_isolated_test(
        "pipelines::shape::coverage_tests::resized_scene_uses_logical_viewport_for_fractional_clips",
        || {
            let create = |width, height, sample_count| {
                UiTest::new()
                    .viewport(width, height)
                    .samples(sample_count)
                    .rendered(crate::TesseraComponents)
                    .content(|| damage_fixture(TestProbe::default()))
            };
            for sample_count in [1, 4] {
                let mut resized = create(200, 128, sample_count);
                resized.frames(2);
                for (width, height) in [(201, 133), (251, 253), (129, 135), (385, 259), (83, 61)] {
                    resized.resize(width, height);
                    resized.frames(2);
                    let mut exact = create(width, height, sample_count);
                    exact.frames(2);
                    let expected = exact.read_pixels();
                    assert!(
                        expected
                            .chunks_exact(4)
                            .any(|pixel| pixel == [0, 0, 0, 255])
                    );
                    assert_eq!(
                        resized.read_pixels(),
                        expected,
                        "resizing to {width}x{height} must preserve exact pixels at MSAA{sample_count}"
                    );
                }
            }
        },
    );
}

#[tessera]
fn batch_fixture(probe: Option<TestProbe<Option<State<bool>>>>) {
    let distinct = remember(|| false);
    if let Some(probe) = probe {
        probe.set(Some(distinct));
    }
    boxed()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            for index in 0..BATCH_COUNT {
                let color = if distinct.get() {
                    Color::from_rgb((20 + index % 15 * 10) as f32 / 255.0, 0.4, 0.6)
                } else {
                    Color::from_rgb(0.8, 0.4, 0.6)
                };
                boxed()
                    .modifier(
                        Modifier::new()
                            .offset(Dp(4.0 + index as f64 * 16.0), Dp(4.0))
                            .size(Dp(12.0), Dp(12.0))
                            .clip_to_bounds(),
                    )
                    .children(move || {
                        boxed().modifier(
                            Modifier::new()
                                .fill_max_size()
                                .background_with_shape(color, Shape::rounded_rectangle(Dp(2.0))),
                        );
                    });
            }
        });
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn shape_batches_share_uploads_and_preserve_pixels_across_buffers() {
    crate::test_utils::run_isolated_test(
        "pipelines::shape::coverage_tests::shape_batches_share_uploads_and_preserve_pixels_across_buffers",
        || {
            for sample_count in [1, 4] {
                let probe = TestProbe::<Option<State<bool>>>::default();
                let observed = probe.clone();
                let mut renderer = UiTest::new()
                    .viewport(VIEWPORT_WIDTH, 24)
                    .samples(sample_count)
                    .rendered(crate::TesseraComponents)
                    .content(move || batch_fixture().probe(probe.clone()));
                renderer.frames(1);
                let uploads = renderer
                    .frame_graphics_work()
                    .group("shape_batches")
                    .buffer_uploads;
                assert!(
                    uploads <= 2,
                    "batches must share frame uploads, got {uploads}"
                );
                let uniform = renderer.read_pixels();
                let first = pixel(&uniform, VIEWPORT_WIDTH, 10, 10);
                assert_eq!(first[3], 255);
                for index in 0..BATCH_COUNT {
                    assert_eq!(
                        pixel(&uniform, VIEWPORT_WIDTH, 10 + index * 16, 10),
                        first,
                        "batch {index} must retain its own position across buffer boundaries"
                    );
                }
                observed.get().unwrap().set(true);
                renderer.frames(2);
                let distinct = renderer.read_pixels();
                let mut previous_red = None;
                for index in 0..BATCH_COUNT {
                    let actual = pixel(&distinct, VIEWPORT_WIDTH, 10 + index * 16, 10);
                    assert_eq!(actual[3], 255);
                    if index % 15 != 0
                        && let Some(previous) = previous_red
                    {
                        assert!(
                            actual[0] > previous,
                            "batch colors must remain distinct and ordered"
                        );
                    }
                    assert_eq!(
                        actual,
                        pixel(&distinct, VIEWPORT_WIDTH, 10 + index % 15 * 16, 10),
                        "batch {index} must keep its own instance data across buffers"
                    );
                    previous_red = Some(actual[0]);
                }
                let allocations = renderer
                    .graphics_work()
                    .group("shape_batches")
                    .buffer_allocations;
                renderer.frames(2);
                assert_eq!(
                    renderer
                        .graphics_work()
                        .group("shape_batches")
                        .buffer_allocations,
                    allocations,
                    "stable frame batches must reuse GPU buffers and bindings"
                );
                assert_eq!(renderer.read_pixels(), distinct);
            }
        },
    );
}

#[tessera]
fn fixture() {
    boxed()
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(|| {
            boxed().modifier(
                Modifier::new()
                    .offset(Dp(16.0), Dp(16.0))
                    .size(Dp(32.0), Dp(24.0))
                    .border_with_shape(Dp(2.0), Color::BLACK, Shape::RECTANGLE),
            );
            boxed()
                .modifier(Modifier::new().offset(Dp(48.0), Dp::ZERO))
                .children(|| {
                    scaled_box(0.625).child(|| {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(26.0), Dp(26.0))
                                    .size(Dp(32.0), Dp(32.0))
                                    .clip_shape(Shape::RECTANGLE),
                            )
                            .children(|| {
                                boxed().modifier(
                                    Modifier::new()
                                        .required_size(Dp(48.0), Dp(32.0))
                                        .border_with_shape(Dp(3.2), Color::BLACK, Shape::RECTANGLE),
                                );
                            });
                    });
                });
            boxed()
                .modifier(Modifier::new().offset(Dp(64.0), Dp(48.0)))
                .children(|| {
                    scaled_box(0.625).child(|| {
                        boxed().modifier(
                            Modifier::new().size(Dp(48.0), Dp(32.0)).border_with_shape(
                                Dp(2.0),
                                Color::BLACK,
                                Shape::RECTANGLE,
                            ),
                        );
                    });
                });
        });
}

fn pixel(pixels: &[u8], width: u32, x: usize, y: usize) -> [u8; 4] {
    pixels[(y * width as usize + x) * 4..][..4]
        .try_into()
        .unwrap()
}

fn assert_black(pixels: &[u8], width: u32, x: usize, y: usize, samples: u32) {
    let actual = pixel(pixels, width, x, y);
    assert!(
        actual[..3].iter().all(|channel| *channel <= 1) && actual[3] == 255,
        "fully covered black border pixel ({x},{y}) must stay opaque at MSAA{samples}: {actual:?}"
    );
}

fn assert_white(pixels: &[u8], width: u32, x: usize, y: usize, samples: u32) {
    let actual = pixel(pixels, width, x, y);
    assert!(
        actual[..3].iter().all(|channel| *channel >= 254) && actual[3] == 255,
        "uncovered white pixel ({x},{y}) must stay white at MSAA{samples}: {actual:?}"
    );
}

#[test]
#[ignore = "requires a native GPU adapter supporting one and four samples"]
fn native_borders_keep_fully_covered_pixels_opaque_at_integer_and_fractional_clipped_geometry() {
    for sample_count in [1, 4] {
        let mut renderer = UiTest::new()
            .viewport(112, 88)
            .samples(sample_count)
            .rendered(crate::TesseraComponents)
            .content(fixture);
        renderer.frames(8);
        let pixels = renderer.read_pixels();
        let allocations = renderer
            .graphics_work()
            .group("shape_batches")
            .buffer_allocations;
        renderer.frames(8);
        assert_eq!(
            renderer
                .graphics_work()
                .group("shape_batches")
                .buffer_allocations,
            allocations,
            "warmed shape batches must reuse GPU buffers and bindings"
        );
        assert_eq!(
            renderer.read_pixels(),
            pixels,
            "reused batch slots must preserve fractional shape pixels"
        );
        for x in [24, 32, 40] {
            assert_black(&pixels, 112, x, 16, sample_count);
            assert_black(&pixels, 112, x, 17, sample_count);
            assert_white(&pixels, 112, x, 18, sample_count);
            assert_white(&pixels, 112, x, 15, sample_count);
        }
        for y in [24, 28, 32] {
            assert_black(&pixels, 112, 16, y, sample_count);
            assert_black(&pixels, 112, 17, y, sample_count);
            assert_white(&pixels, 112, 18, y, sample_count);
        }
        // The transformed border covers [16.25,18.25) vertically, so row 17 is
        // wholly inside.
        for x in [72, 76, 80] {
            assert_black(&pixels, 112, x, 17, sample_count);
            assert_white(&pixels, 112, x, 20, sample_count);
        }
        assert_white(&pixels, 112, 87, 17, sample_count);
        // A native 2dp border becomes 1.25 physical pixels; its first pixel
        // remains wholly covered.
        for x in [72, 76, 80] {
            assert_black(&pixels, 112, x, 48, sample_count);
            assert_white(&pixels, 112, x, 50, sample_count);
        }
        assert_white(&pixels, 112, 72, 47, sample_count);
    }
}
