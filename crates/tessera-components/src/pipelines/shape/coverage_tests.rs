//! Native shape coverage and shared-upload regressions.
//!
//! ## Usage
//!
//! Verify clipped and scaled surfaces across repeated draw batches.

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State, remember, scaled_box, tessera,
    testing::TestProbe,
};

use crate::{boxed::boxed, modifier::ModifierExt as _, shape_def::Shape};

const BATCH_COUNT: usize = 300;
const VIEWPORT_WIDTH: u32 = 8 + BATCH_COUNT as u32 * 16;

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
                let mut renderer = HeadlessRenderer::new(
                    Box::new(move || {
                        batch_fixture().probe(probe.clone());
                    }),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width: VIEWPORT_WIDTH,
                        height: 24,
                        sample_count,
                        ..Default::default()
                    },
                )
                .unwrap();
                renderer.render_frames(1);
                let uploads = renderer
                    .frame_graphics_work()
                    .group("shape_batches")
                    .buffer_uploads;
                assert!(
                    uploads <= 2,
                    "batches must share frame uploads, got {uploads}"
                );
                let uniform = renderer.read_pixels().unwrap();
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
                renderer.render_frames(2);
                let distinct = renderer.read_pixels().unwrap();
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
                renderer.render_frames(2);
                assert_eq!(
                    renderer
                        .graphics_work()
                        .group("shape_batches")
                        .buffer_allocations,
                    allocations,
                    "stable frame batches must reuse GPU buffers and bindings"
                );
                assert_eq!(renderer.read_pixels().unwrap(), distinct);
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
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 112,
                height: 88,
                sample_count,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(8);
        let pixels = renderer.read_pixels().unwrap();
        let allocations = renderer
            .graphics_work()
            .group("shape_batches")
            .buffer_allocations;
        renderer.render_frames(8);
        assert_eq!(
            renderer
                .graphics_work()
                .group("shape_batches")
                .buffer_allocations,
            allocations,
            "warmed shape batches must reuse GPU buffers and bindings"
        );
        assert_eq!(
            renderer.read_pixels().unwrap(),
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
