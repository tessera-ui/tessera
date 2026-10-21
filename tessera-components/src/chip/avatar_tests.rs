//! Input avatar geometry, clipping, and source-color regressions.
//!
//! ## Usage
//!
//! Preserve contact artwork when input chips change state or density.

use std::cell::Cell;

use tessera_ui::{Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State, remember, tessera};

use crate::{
    column::column,
    image::ImageData,
    image_vector::TryIntoImageVectorData as _,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    painter::Painter,
    theme::{MaterialTheme, material_theme},
};

use super::{ChipBorder, ChipDefaults, ChipStyle, ChipVariant, chip};

#[derive(Clone, Copy, Default, PartialEq)]
struct Settings {
    source: usize,
    disabled: bool,
    selected: bool,
    trailing: bool,
    no_avatar: bool,
    assist: bool,
}

thread_local! {
    static SETTINGS: Cell<Option<State<Settings>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let settings = remember(Settings::default);
    let painters = remember(|| {
        let red = [255, 0, 0, 255];
        let green = [0, 255, 0, 255];
        let blue = [0, 0, 255, 255];
        let landscape = [green, red, red, blue, green, red, red, blue].concat();
        let portrait = [green, green, red, red, red, red, blue, blue].concat();
        [
            Painter::from(ImageData::from_rgba(landscape, 4, 2)),
            Painter::from(ImageData::from_rgba(portrait, 2, 4)),
            Painter::from(r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><path fill="#0000ff" d="M0 0H24V24H0Z"/></svg>"##
                .as_bytes().try_into_image_vector_data().expect("avatar vector")),
        ]
    });
    SETTINGS.set(Some(settings));
    material_theme().child(move || {
        column()
            .modifier(
                Modifier::new()
                    .fill_max_size()
                    .background(Color::WHITE)
                    .padding_all(Dp(16.0)),
            )
            .children(move || {
                let settings = settings.get();
                let mut colors = ChipDefaults::colors(
                    &MaterialTheme::default().color_scheme,
                    ChipVariant::Input,
                    ChipStyle::Flat,
                );
                colors.container_color = Color::WHITE;
                colors.selected_container_color = Color::WHITE;
                colors.disabled_container_color = Color::WHITE;
                colors.disabled_selected_container_color = Color::WHITE;
                colors.leading_icon_color = Color::GREEN;
                colors.selected_leading_icon_color = Color::GREEN;
                let mut chip = chip()
                    .variant(if settings.assist {
                        ChipVariant::Assist
                    } else {
                        ChipVariant::Input
                    })
                    .label("Avatar label")
                    .accessibility_label("Avatar control")
                    .modifier(Modifier::new().semantics(SemanticsArgs {
                        test_tag: Some("avatar_chip".into()),
                        ..Default::default()
                    }))
                    .enabled(!settings.disabled)
                    .selected(settings.selected)
                    .colors(colors)
                    .border(ChipBorder {
                        width: Dp::ZERO,
                        selected_width: Dp::ZERO,
                        color: Color::TRANSPARENT,
                        selected_color: Color::TRANSPARENT,
                        disabled_color: Color::TRANSPARENT,
                        disabled_selected_color: Color::TRANSPARENT,
                    })
                    .leading_icon(filled::HOME_SVG)
                    .avatar_optional(
                        (!settings.no_avatar)
                            .then(|| painters.with(|p| p[settings.source].clone())),
                    )
                    .on_click(|| {});
                if settings.trailing {
                    chip = chip.trailing_icon(filled::CLOSE_SVG);
                }
                drop(chip);
            });
    });
}

fn rect(renderer: &HeadlessRenderer, key: &str) -> [i64; 4] {
    let snapshot = renderer.snapshot();
    let mut pending = vec![&snapshot["root"]];
    while let Some(node) = pending.pop() {
        if node["accessibility"]["key"] == key || node["accessibility"]["label"] == key {
            return ["x", "y", "width", "height"]
                .map(|field| node["bounds"][field].as_i64().unwrap());
        }
        if let Some(children) = node["children"].as_array() {
            pending.extend(children);
        }
    }
    panic!("missing {key}");
}

fn pixel(renderer: &HeadlessRenderer, x: i64, y: i64, color: [u8; 3]) {
    let pixels = renderer.read_pixels().unwrap();
    let offset = ((y as u32 * renderer.physical_size().width + x as u32) * 4) as usize;
    let actual = &pixels[offset..offset + 3];
    assert!(
        actual.iter().zip(color).all(|(a, b)| a.abs_diff(b) <= 2),
        "at {x},{y}: {actual:?} != {color:?}"
    );
}

fn check_avatars(samples: u32) {
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(crate::TesseraComponents)],
            HeadlessConfig {
                width: 440,
                height: 200,
                scale_factor: scale,
                sample_count: samples,
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(5);
        let settings = SETTINGS.get().unwrap();
        let px = |dp: i64| (dp as f64 * scale) as i64;
        for source in 0..3 {
            for disabled in [false, true] {
                settings.set(Settings {
                    source,
                    disabled,
                    ..Default::default()
                });
                renderer.render_frames(5);
                let [x, y, width, height] = rect(&renderer, "avatar_chip");
                let label = rect(&renderer, "Avatar label");
                assert_eq!(height, px(48));
                assert_eq!(label[0] - x, px(36));
                assert_eq!(x + width - label[0] - label[2], px(12));
                let color = match (source == 2, disabled) {
                    (false, false) => [255, 0, 0],
                    (false, true) => [255, 206, 206],
                    (true, false) => [0, 0, 255],
                    (true, true) => [206, 206, 255],
                };
                pixel(&renderer, x + px(16), y + px(24), color);
                pixel(&renderer, x + px(10), y + px(24), color);
                pixel(&renderer, x + px(16), y + px(18), color);
                pixel(&renderer, x + px(4), y + px(12), [255; 3]);
                pixel(&renderer, x + px(3), y + px(24), [255; 3]);
                pixel(&renderer, x + px(29), y + px(24), [255; 3]);
            }
        }
        settings.set(Settings {
            trailing: true,
            selected: true,
            ..Default::default()
        });
        renderer.render_frames(5);
        let [x, y, width, _] = rect(&renderer, "avatar_chip");
        let label = rect(&renderer, "Avatar label");
        assert_eq!(label[0] - x, px(36));
        assert_eq!(x + width - label[0] - label[2], px(34));
        pixel(&renderer, x + px(16), y + px(24), [255, 0, 0]);
        for assist in [false, true] {
            settings.set(Settings {
                no_avatar: !assist,
                assist,
                ..Default::default()
            });
            renderer.render_frames(5);
            let outer = rect(&renderer, "avatar_chip");
            let label = rect(&renderer, "Avatar label");
            assert_eq!(label[0] - outer[0], px(34));
        }
        settings.set(Settings::default());
        renderer.render_frames(5);
        let [x, y, _, _] = rect(&renderer, "avatar_chip");
        assert_eq!(rect(&renderer, "Avatar label")[0] - x, px(36));
        pixel(&renderer, x + px(16), y + px(24), [255, 0, 0]);
    }
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn input_avatars_crop_clip_preserve_colors_and_restore_icon_spacing_msaa1() {
    crate::test_utils::run_isolated_test(
        "chip::avatar_tests::input_avatars_crop_clip_preserve_colors_and_restore_icon_spacing_msaa1",
        || check_avatars(1),
    );
}

#[test]
#[ignore = "requires a native GPU adapter"]
fn input_avatars_crop_clip_preserve_colors_and_restore_icon_spacing_msaa4() {
    crate::test_utils::run_isolated_test(
        "chip::avatar_tests::input_avatars_crop_clip_preserve_colors_and_restore_icon_spacing_msaa4",
        || check_avatars(4),
    );
}
