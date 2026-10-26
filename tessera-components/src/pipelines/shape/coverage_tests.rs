//! Native shape coverage and shared-upload regressions.
//!
//! ## Usage
//!
//! Verify clipped, scaled and cached surfaces across repeated draw batches.

use std::cell::Cell;

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, RenderModule, State, remember,
    scaled_box, tessera,
};

use crate::{boxed::boxed, modifier::ModifierExt as _, shape_def::Shape};

struct Pipelines;

const BATCH_COUNT: usize = 300;
const VIEWPORT_WIDTH: u32 = 8 + BATCH_COUNT as u32 * 16;

thread_local! {
    static DISTINCT: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

#[tessera]
fn cache_clock_fixture() {
    let distinct = remember(|| false);
    DISTINCT.set(Some(distinct));
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
#[ignore = "requires a native GPU adapter for cache generation across isolated draw batches"]
fn shape_batches_share_uploads_and_preserve_cache_heat_and_pixels_across_buffers() {
    crate::test_utils::run_isolated_test(
        "pipelines::shape::coverage_tests::shape_batches_share_uploads_and_preserve_cache_heat_and_pixels_across_buffers",
        || {
            for sample_count in [1, 4] {
                let mut renderer = HeadlessRenderer::new(
                    Box::new(cache_clock_fixture),
                    vec![Box::new(Pipelines)],
                    HeadlessConfig {
                        width: VIEWPORT_WIDTH,
                        height: 24,
                        sample_count,
                        ..Default::default()
                    },
                )
                .unwrap();
                super::pipeline::CACHE_BUILDS.set(0);
                renderer.render_frames(1);
                assert_eq!(
                    super::pipeline::CACHE_BUILDS.get(),
                    0,
                    "repeated batches in one frame must not warm a transient shape"
                );
                assert!(
                    super::pipeline::BATCH_UPLOADS.get() <= 2,
                    "uncached batches must share frame uploads, got {}",
                    super::pipeline::BATCH_UPLOADS.get()
                );
                let uncached = renderer.read_pixels().unwrap();
                let first = pixel(&uncached, VIEWPORT_WIDTH, 10, 10);
                assert_eq!(first[3], 255);
                for index in 0..BATCH_COUNT {
                    assert_eq!(
                        pixel(&uncached, VIEWPORT_WIDTH, 10 + index * 16, 10),
                        first,
                        "uncached batch {index} must retain its own position across buffer boundaries"
                    );
                }
                renderer.render_frames(7);
                assert_eq!(
                    super::pipeline::CACHE_BUILDS.get(),
                    1,
                    "one stable shape must become cached across real frames"
                );
                assert!(
                    super::pipeline::BATCH_UPLOADS.get() <= 2,
                    "cached batches must share frame uploads"
                );
                DISTINCT.get().unwrap().set(true);
                let before = super::pipeline::CACHE_BUILDS.get();
                renderer.render_frames(8);
                assert_eq!(
                    super::pipeline::CACHE_BUILDS.get() - before,
                    15,
                    "more than ten batches per frame must not expire stable shape heat"
                );
                let cached = renderer.read_pixels().unwrap();
                let mut previous_red = None;
                for index in 0..BATCH_COUNT {
                    let x = 10 + index * 16;
                    let actual = pixel(&cached, VIEWPORT_WIDTH, x, 10);
                    assert_eq!(actual[3], 255);
                    if index % 15 != 0
                        && let Some(previous) = previous_red
                    {
                        assert!(
                            actual[0] > previous,
                            "cached batch colors must remain distinct and ordered"
                        );
                    }
                    assert_eq!(
                        actual,
                        pixel(&cached, VIEWPORT_WIDTH, 10 + index % 15 * 16, 10),
                        "cached batch {index} must preserve texture and transform selection across buffers"
                    );
                    previous_red = Some(actual[0]);
                }
                assert!(
                    super::pipeline::BATCH_UPLOADS.get() <= 2,
                    "distinct cached textures must share transform uploads"
                );
                let allocations = super::pipeline::BATCH_ALLOCATIONS.get();
                renderer.render_frames(2);
                assert_eq!(
                    super::pipeline::BATCH_ALLOCATIONS.get(),
                    allocations,
                    "stable frame batches must reuse GPU buffers and bindings"
                );
                assert_eq!(renderer.read_pixels().unwrap(), cached);
            }
        },
    );
}

impl RenderModule for Pipelines {
    fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
        crate::pipelines::register_pipelines(context);
    }
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
            vec![Box::new(Pipelines)],
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
        let allocations = super::pipeline::BATCH_ALLOCATIONS.get();
        renderer.render_frames(8);
        assert_eq!(
            super::pipeline::BATCH_ALLOCATIONS.get(),
            allocations,
            "warmed shape batches must reuse GPU buffers and bindings"
        );
        assert_eq!(
            renderer.read_pixels().unwrap(),
            pixels,
            "reused batch slots must preserve cached and fractional shape pixels"
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
