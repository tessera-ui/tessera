//! Intrinsic translucent-vector and layer-opacity composition.
//!
//! ## Usage
//!
//! Preserve translucent vector color across atlas sampling and layer opacity.

use std::cell::Cell;

use tessera_ui::{Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State, remember, tessera};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    image::{ImageData, image},
    image_vector::TryIntoImageVectorData as _,
    modifier::ModifierExt as _,
};

thread_local! {
    static INVERTED: Cell<Option<State<bool>>> = const { Cell::new(None) };
    static COUNT: Cell<Option<State<usize>>> = const { Cell::new(None) };
}

#[tessera]
fn fixture() {
    let inverted = remember(|| false);
    INVERTED.set(Some(inverted));
    let count = remember(|| 65_usize);
    COUNT.set(Some(count));
    let translucent = remember(|| {
        b"<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'><rect width='32' height='32' fill='#ff0000' fill-opacity='.5'/></svg>".as_slice()
        .try_into_image_vector_data().expect("Inline translucent vector")
    });
    let opaque = remember(|| {
        b"<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'><rect width='32' height='32' fill='#ff0000'/></svg>".as_slice()
        .try_into_image_vector_data().expect("Inline opaque vector")
    });
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(Modifier::new().fill_max_size().background(Color::WHITE))
        .children(move || {
            image()
                .painter(ImageData::from_rgba(vec![255, 0, 0, 128], 1, 1))
                .modifier(
                    Modifier::new()
                        .offset(Dp(8.0), Dp(8.0))
                        .size(Dp(32.0), Dp(32.0)),
                );
            image().painter(translucent.get()).modifier(
                Modifier::new()
                    .offset(Dp(48.0), Dp(8.0))
                    .size(Dp(32.0), Dp(32.0)),
            );
            for index in 0..count.get() {
                let painter = if (index % 2 == 0) ^ inverted.get() {
                    opaque.get()
                } else {
                    translucent.get()
                };
                image().painter(painter).modifier(
                    Modifier::new()
                        .offset(
                            Dp(8.0 + (index % 13) as f64 * 12.0),
                            Dp(56.0 + (index / 13) as f64 * 12.0),
                        )
                        .size(Dp(8.0), Dp(8.0)),
                );
            }
            image().painter(opaque.get()).modifier(
                Modifier::new()
                    .offset(Dp(88.0), Dp(8.0))
                    .size(Dp(32.0), Dp(32.0))
                    .alpha(0.5),
            );
            image().painter(translucent.get()).modifier(
                Modifier::new()
                    .offset(Dp(128.0), Dp(8.0))
                    .size(Dp(32.0), Dp(32.0))
                    .alpha(0.5),
            );
        });
}

#[test]
#[ignore = "requires a native GPU adapter for real raster/vector alpha comparison"]
fn translucent_atlas_color_is_not_premultiplied_twice() {
    crate::test_utils::run_isolated_test(
        "pipelines::image_vector::alpha_tests::translucent_atlas_color_is_not_premultiplied_twice",
        || {
            for samples in [1, 4] {
                let mut renderer = HeadlessRenderer::new(
                    Box::new(fixture),
                    vec![Box::new(crate::TesseraComponents)],
                    HeadlessConfig {
                        width: 176,
                        height: 128,
                        sample_count: samples,
                        ..Default::default()
                    },
                )
                .expect("Native GPU adapter");
                renderer.render_frames(4);
                let initial = renderer.read_pixels().unwrap();
                assert!(
                    super::pipeline::SAMPLE_UPLOADS.get() <= 2,
                    "68 vector samples must use at most two uniform uploads"
                );
                for (x, expected) in [
                    (24, [255_u8, 187, 187]),
                    (64, [255, 187, 187]),
                    (104, [255, 188, 188]),
                    (144, [255, 225, 225]),
                ] {
                    let actual: [u8; 3] = initial[(24 * 176 + x) * 4..][..3].try_into().unwrap();
                    assert!(
                        actual
                            .into_iter()
                            .zip(expected)
                            .all(|(a, b)| a.abs_diff(b) <= 1),
                        "MSAA{samples} pixel{x}: {actual:?}, expected {expected:?}"
                    );
                }
                renderer.render_frames(3);
                assert_eq!(
                    renderer.read_pixels().unwrap(),
                    initial,
                    "Atlas cache-hit output must preserve alpha"
                );
                for inverted in [false, true, false] {
                    INVERTED.get().unwrap().set(inverted);
                    renderer.render_frames(1);
                    let pixels = renderer.read_pixels().unwrap();
                    assert!(super::pipeline::SAMPLE_UPLOADS.get() <= 2);
                    for index in 0..65 {
                        let x = 12 + (index % 13) * 12;
                        let y = 60 + (index / 13) * 12;
                        let pixel = &pixels[(y * 176 + x) * 4..][..4];
                        let expected = if (index % 2 == 0) ^ inverted {
                            [255_u8, 0, 0, 255]
                        } else {
                            [255_u8, 187, 187, 255]
                        };
                        assert!(
                            pixel
                                .iter()
                                .zip(expected)
                                .all(|(&actual, expected)| actual.abs_diff(expected) <= 1),
                            "MSAA{samples} vector {index} after inversion {inverted}: {pixel:?}"
                        );
                    }
                }
                COUNT.get().unwrap().set(1);
                renderer.render_frames(1);
                assert!(super::pipeline::SAMPLE_UPLOADS.get() <= 1);
                let shrunk = renderer.read_pixels().unwrap();
                assert_eq!(&shrunk[(108 * 176 + 156) * 4..][..4], &[255, 255, 255, 255]);
                COUNT.get().unwrap().set(65);
                renderer.render_frames(1);
                assert_eq!(renderer.read_pixels().unwrap(), initial);
            }
        },
    );
}
