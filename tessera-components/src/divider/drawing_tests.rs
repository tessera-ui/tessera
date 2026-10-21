use std::cell::Cell;

use tessera_ui::{Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State, remember, tessera};

use crate::{
    column::column,
    modifier::{ModifierExt as _, SemanticsArgs},
    theme::{MaterialTheme, material_theme},
};

use super::{horizontal_divider, vertical_divider};

thread_local! {
    static DARK: Cell<Option<State<bool>>> = const { Cell::new(None) };
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

fn check(samples: u32) {
    for (scale, width, thickness, double, inset, gap, height) in [
        (1.0, 80, 1, 2, 16, 4, 32),
        (1.25, 100, 1, 2, 20, 5, 40),
        (1.5, 120, 1, 3, 24, 6, 48),
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
            for (tag, line, color) in [
                ("horizontal", thickness, expected),
                ("hairline", 1, expected),
                ("explicit", double, [0, 255, 0, 255]),
            ] {
                assert_eq!(rect(&renderer, tag), [0, y, width, line + gap * 2]);
                assert_eq!(pixel(&renderer, &pixels, inset, y + gap), color);
                assert_eq!(
                    pixel(&renderer, &pixels, width - inset - 1, y + gap + line - 1),
                    color
                );
                for (x, sample_y) in [
                    (inset - 1, y + gap),
                    (width - inset, y + gap),
                    (inset, y + gap - 1),
                    (inset, y + gap + line),
                ] {
                    assert_eq!(pixel(&renderer, &pixels, x, sample_y), [255; 4]);
                }
                y += line + gap * 2;
            }
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
    }
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
