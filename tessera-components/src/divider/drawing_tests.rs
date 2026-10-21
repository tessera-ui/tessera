use std::{cell::Cell, sync::atomic::Ordering};

use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, LayoutModifierNode, LayoutResult,
    MeasurementError, Modifier, PxPosition, State,
    layout::{MeasureScope, layout},
    remember, tessera,
};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};

use super::{DividerDefaults, horizontal_divider, vertical_divider};

thread_local! {
    static DARK: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static FAINT: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static CLIPPED: Cell<Option<State<bool>>> = const { Cell::new(None) };
}

#[derive(Clone, PartialEq)]
struct HalfPosition;

impl LayoutModifierNode for HalfPosition {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let child = scope.children()[0];
        let constraint = scope.parent_constraint().without_min();
        let size = child.measure(scope, &constraint)?.size();
        let mut result = LayoutResult::new(size);
        result.place_child(child, PxPosition::from_f32_xy(0.5, 0.5));
        Ok(result)
    }
}

#[tessera]
fn fractional_fixture() {
    let faint = remember(|| false);
    let clipped = remember(|| false);
    FAINT.set(Some(faint));
    CLIPPED.set(Some(clipped));
    column()
        .modifier(
            Modifier::new()
                .background(Color::WHITE)
                .padding_all(Dp(4.0)),
        )
        .children(move || {
            let modifier = Modifier::new().size(Dp(8.0), Dp(1.0));
            let modifier = if clipped.get() {
                modifier.clip_shape(Shape::rounded_rectangle(Dp::ZERO))
            } else {
                modifier
            };
            layout()
                .modifier(modifier)
                .policy(HalfPosition)
                .child(move || {
                    horizontal_divider()
                        .modifier(Modifier::new().width(Dp(8.0)).alpha(if faint.get() {
                            0.5
                        } else {
                            1.0
                        }))
                        .color(Color::BLACK);
                });
        });
}

fn tagged(name: &str) -> Modifier {
    Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(name.into()),
        ..Default::default()
    })
}

#[tessera]
fn fixture() {
    let dark = remember(|| false);
    DARK.set(Some(dark));
    material_theme()
        .theme(move || {
            let mut theme = MaterialTheme::default();
            theme.color_scheme.outline_variant = if dark.get() { Color::BLUE } else { Color::RED };
            theme
        })
        .child(|| {
            column().children(|| {
                column()
                    .modifier(Modifier::new().width(Dp(80.0)).background(Color::WHITE))
                    .children(|| {
                        horizontal_divider()
                            .modifier(tagged("horizontal").padding_symmetric(Dp(16.0), Dp(4.0)));
                        horizontal_divider()
                            .modifier(tagged("hairline").padding_symmetric(Dp(16.0), Dp(4.0)))
                            .thickness(Dp::ZERO);
                        horizontal_divider()
                            .modifier(tagged("explicit").padding_symmetric(Dp(16.0), Dp(4.0)))
                            .thickness(Dp(2.0))
                            .color(Color::GREEN);
                        horizontal_divider()
                            .modifier(tagged("defaults").padding_symmetric(Dp(16.0), Dp(4.0)))
                            .color(DividerDefaults::color());
                        vertical_divider().modifier(
                            tagged("vertical")
                                .height(Dp(32.0))
                                .padding_symmetric(Dp(4.0), Dp(8.0)),
                        );
                    });
            });
        });
}

fn rect(renderer: &HeadlessRenderer, tag: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == tag {
            return ["x", "y", "width", "height"].map(|key| node["bounds"][key].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {tag}");
}

fn pixel(renderer: &HeadlessRenderer, pixels: &[u8], x: i64, y: i64) -> [u8; 4] {
    let offset = ((y as u32 * renderer.physical_size().width + x as u32) * 4) as usize;
    pixels[offset..offset + 4].try_into().unwrap()
}

fn assert_pixel_near(actual: [u8; 4], expected: [u8; 4]) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, b)| a.abs_diff(b) <= 2),
        "{actual:?} != {expected:?}"
    );
}

fn check(samples: u32) {
    for (scale, width, thickness, double, inset, gap, height) in [
        (1.0, 80, 1, 2, 16, 4, 32),
        (1.25, 100, 1, 3, 20, 5, 40),
        (1.5, 120, 2, 3, 24, 6, 48),
        (2.0, 160, 2, 4, 32, 8, 64),
    ] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 200,
                height: 200,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(3);
        let baseline = renderer.read_pixels().unwrap();
        let allocations = crate::pipelines::divider::ALLOCATIONS.load(Ordering::Relaxed);
        for dark in [true, false] {
            DARK.get().unwrap().set(dark);
            renderer.render_frames(3);
            let pixels = renderer.read_pixels().unwrap();
            let expected = if dark {
                [0, 0, 255, 255]
            } else {
                [255, 0, 0, 255]
            };
            let mut y = 0;
            for (tag, line, color, full, tail) in [
                (
                    "horizontal",
                    thickness,
                    expected,
                    1,
                    if scale == 1.25 {
                        225
                    } else if scale == 1.5 {
                        188
                    } else {
                        255
                    },
                ),
                ("hairline", 1, expected, 1, 255),
                (
                    "explicit",
                    double,
                    [0, 255, 0, 255],
                    if scale == 1.25 { 2 } else { double },
                    if scale == 1.25 { 188 } else { 255 },
                ),
            ] {
                assert_eq!(rect(&renderer, tag), [0, y, width, line + gap * 2]);
                assert_eq!(pixel(&renderer, &pixels, inset, y + gap), color);
                let stroke_pixels = if tag == "horizontal" && (scale == 1.25 || scale == 1.5) {
                    2
                } else {
                    line
                };
                let edge = if tail == 255 {
                    color
                } else {
                    color.map(|channel| if channel == 0 { tail } else { channel })
                };
                assert_pixel_near(
                    pixel(
                        &renderer,
                        &pixels,
                        width - inset - 1,
                        y + gap + stroke_pixels - 1,
                    ),
                    edge,
                );
                if full > 1 {
                    assert_eq!(pixel(&renderer, &pixels, inset, y + gap + full - 1), color);
                }
                for (x, sample_y) in [
                    (inset - 1, y + gap),
                    (width - inset, y + gap),
                    (inset, y + gap - 1),
                    (inset, y + gap + stroke_pixels),
                ] {
                    assert_eq!(pixel(&renderer, &pixels, x, sample_y), [255; 4]);
                }
                y += line + gap * 2;
            }
            assert_eq!(
                rect(&renderer, "defaults"),
                [0, y, width, thickness + gap * 2]
            );
            let color = MaterialColorScheme::default().outline_variant;
            let encode = |channel: f32| {
                ((if channel <= 0.0031308 {
                    12.92 * channel
                } else {
                    1.055 * channel.powf(1.0 / 2.4) - 0.055
                }) * 255.0)
                    .round() as u8
            };
            assert_pixel_near(
                pixel(&renderer, &pixels, inset, y + gap),
                [encode(color.r), encode(color.g), encode(color.b), 255],
            );
            y += thickness + gap * 2;
            assert_eq!(
                rect(&renderer, "vertical"),
                [0, y, thickness + gap * 2, height]
            );
            assert_eq!(pixel(&renderer, &pixels, gap, y + gap * 2), expected);
            assert_eq!(pixel(&renderer, &pixels, gap - 1, y + gap * 2), [255; 4]);
            assert_eq!(pixel(&renderer, &pixels, gap, y + gap * 2 - 1), [255; 4]);
        }
        assert_eq!(renderer.read_pixels().unwrap(), baseline);
        renderer.resize(240, 200);
        renderer.render_frames(3);
        assert_eq!(
            rect(&renderer, "horizontal"),
            [0, 0, width, thickness + gap * 2]
        );
        renderer.resize(200, 200);
        renderer.render_frames(3);
        assert_eq!(renderer.read_pixels().unwrap(), baseline);
        renderer.render_frames(8);
        assert_eq!(
            crate::pipelines::divider::ALLOCATIONS.load(Ordering::Relaxed),
            allocations
        );
    }
    check_fractional_placement(samples);
    check_constrained_strokes(samples);
}

fn interior_pixel(renderer: &HeadlessRenderer, pixels: &[u8], x: i64, y: i64) -> [u8; 4] {
    pixel(renderer, pixels, x + 4, y + 4)
}

fn check_fractional_placement(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(fractional_fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 20,
            height: 20,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(3);
    let baseline = renderer.read_pixels().unwrap();
    for (faint, edge, corner) in [(false, 188, 225), (true, 225, 240)] {
        FAINT.get().unwrap().set(faint);
        renderer.render_frames(3);
        let pixels = renderer.read_pixels().unwrap();
        for y in [0, 1] {
            for x in [0, 8] {
                assert_pixel_near(
                    interior_pixel(&renderer, &pixels, x, y),
                    [corner, corner, corner, 255],
                );
            }
            for x in 1..8 {
                assert_pixel_near(
                    interior_pixel(&renderer, &pixels, x, y),
                    [edge, edge, edge, 255],
                );
            }
        }
        assert_eq!(interior_pixel(&renderer, &pixels, 9, 0), [255; 4]);
        assert_eq!(interior_pixel(&renderer, &pixels, 1, 2), [255; 4]);
    }
    CLIPPED.get().unwrap().set(true);
    renderer.render_frames(3);
    let clipped = renderer.read_pixels().unwrap();
    assert_pixel_near(
        interior_pixel(&renderer, &clipped, 1, 0),
        [225, 225, 225, 255],
    );
    assert_eq!(interior_pixel(&renderer, &clipped, 1, 1), [255; 4]);
    assert_eq!(interior_pixel(&renderer, &clipped, 8, 0), [255; 4]);
    CLIPPED.get().unwrap().set(false);
    FAINT.get().unwrap().set(false);
    renderer.render_frames(3);
    assert_eq!(renderer.read_pixels().unwrap(), baseline);
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn divider_insets_hairlines_theme_changes_and_resize_msaa1() {
    crate::test_utils::run_isolated_test(
        "divider::drawing_tests::divider_insets_hairlines_theme_changes_and_resize_msaa1",
        || check(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn divider_insets_hairlines_theme_changes_and_resize_msaa4() {
    crate::test_utils::run_isolated_test(
        "divider::drawing_tests::divider_insets_hairlines_theme_changes_and_resize_msaa4",
        || check(4),
    );
}

#[tessera]
fn constrained_fixture() {
    column()
        .modifier(
            Modifier::new()
                .background(Color::WHITE)
                .padding_all(Dp(4.0)),
        )
        .children(|| {
            horizontal_divider()
                .modifier(Modifier::new().size(Dp(8.0), Dp(4.0)))
                .color(Color::BLACK);
            horizontal_divider()
                .modifier(Modifier::new().size(Dp(8.0), Dp(1.0)))
                .thickness(Dp(4.0))
                .color(Color::BLACK);
            horizontal_divider()
                .modifier(Modifier::new().size(Dp(8.0), Dp::ZERO))
                .color(Color::BLACK);
            horizontal_divider()
                .modifier(Modifier::new().width(Dp(8.0)))
                .thickness(Dp(-2.0))
                .color(Color::BLACK);
            vertical_divider()
                .modifier(Modifier::new().size(Dp(4.0), Dp(4.0)))
                .color(Color::BLACK);
        });
}

fn check_constrained_strokes(samples: u32) {
    let mut renderer = HeadlessRenderer::new(
        Box::new(constrained_fixture),
        vec![Box::new(crate::TesseraComponents)],
        HeadlessConfig {
            width: 20,
            height: 20,
            sample_count: samples,
            ..Default::default()
        },
    )
    .unwrap();
    renderer.render_frames(3);
    let pixels = renderer.read_pixels().unwrap();
    assert_eq!(interior_pixel(&renderer, &pixels, 1, 0), [0, 0, 0, 255]);
    assert_eq!(interior_pixel(&renderer, &pixels, 1, 1), [255; 4]);
    assert_eq!(interior_pixel(&renderer, &pixels, 1, 3), [255; 4]);
    assert_eq!(interior_pixel(&renderer, &pixels, 1, 4), [0, 0, 0, 255]);
    assert_eq!(interior_pixel(&renderer, &pixels, 0, 5), [0, 0, 0, 255]);
    assert_eq!(interior_pixel(&renderer, &pixels, 1, 5), [255; 4]);
    assert_eq!(interior_pixel(&renderer, &pixels, 0, 8), [0, 0, 0, 255]);
    assert_eq!(interior_pixel(&renderer, &pixels, 0, 9), [255; 4]);
}
