use tessera_ui::{
    Color, Dp, HeadlessConfig, HeadlessRenderer, Modifier, State, remember, tessera,
    testing::TestProbe,
};

use crate::{
    alignment::Alignment,
    boxed::boxed,
    image::{ImageData, image},
    modifier::ModifierExt as _,
};

#[derive(Clone, Copy, Default, PartialEq)]
struct FixtureHandles {
    inverted: Option<State<bool>>,
    count: Option<State<usize>>,
    size: Option<State<f64>>,
}

#[tessera]
fn fixture(probe: TestProbe<FixtureHandles>) {
    let inverted = remember(|| false);
    let count = remember(|| 65_usize);
    let size = remember(|| 8.0);
    probe.set(FixtureHandles {
        inverted: Some(inverted),
        count: Some(count),
        size: Some(size),
    });
    let translucent = remember(|| {
        crate::image_vector::decode_vector(b"<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'><rect width='32' height='32' fill='#ff0000' fill-opacity='.5'/></svg>".as_slice()).expect("Inline translucent vector")
    });
    let opaque = remember(|| {
        crate::image_vector::decode_vector(b"<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32'><rect width='32' height='32' fill='#ff0000'/></svg>".as_slice()).expect("Inline opaque vector")
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
                        .required_size(Dp(size.get()), Dp(size.get())),
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
fn translucent_atlas_reuse_preserves_alpha_and_releases_sizes_after_retention() {
    crate::test_utils::run_isolated_test(
        "pipelines::image_vector::alpha_tests::translucent_atlas_reuse_preserves_alpha_and_releases_sizes_after_retention",
        || {
            for samples in [1, 4] {
                let probe = TestProbe::default();
                let observed = probe.clone();
                let mut renderer = HeadlessRenderer::new(
                    Box::new(move || {
                        fixture(probe.clone());
                    }),
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
                    renderer
                        .frame_graphics_work()
                        .group("vector_samples")
                        .buffer_uploads
                        <= 2,
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
                    observed.get().inverted.unwrap().set(inverted);
                    renderer.render_frames(1);
                    let pixels = renderer.read_pixels().unwrap();
                    assert!(
                        renderer
                            .frame_graphics_work()
                            .group("vector_samples")
                            .buffer_uploads
                            <= 2
                    );
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
                observed.get().count.unwrap().set(1);
                renderer.render_frames(1);
                assert!(
                    renderer
                        .frame_graphics_work()
                        .group("vector_samples")
                        .buffer_uploads
                        <= 1
                );
                let shrunk = renderer.read_pixels().unwrap();
                assert_eq!(&shrunk[(108 * 176 + 156) * 4..][..4], &[255, 255, 255, 255]);
                observed.get().count.unwrap().set(65);
                renderer.render_frames(1);
                assert_eq!(renderer.read_pixels().unwrap(), initial);
                let baseline = renderer.resource_snapshot().unwrap();
                for index in 0..24 {
                    observed
                        .get()
                        .size
                        .unwrap()
                        .set(512.0 + index as f64 * 16.0);
                    renderer.render_frames(3);
                }
                observed.get().size.unwrap().set(8.0);
                renderer.render_frames(4);
                assert_eq!(renderer.read_pixels().unwrap(), initial);
                let retained = renderer.resource_snapshot().unwrap();
                assert!(
                    retained["textures"].as_i64().unwrap() > baseline["textures"].as_i64().unwrap(),
                    "recently used vector sizes stay cached: {baseline} -> {retained}"
                );
                renderer.render_frames(super::pipeline::RETAINED_FRAMES as u32);
                let settled = renderer.resource_snapshot().unwrap();
                if let (Some(initial), Some(current)) = (
                    baseline["texture_bytes"].as_u64(),
                    settled["texture_bytes"].as_u64(),
                ) {
                    assert!(
                        current <= initial,
                        "Small vectors must release oversized atlas storage: {initial} -> {current}"
                    );
                }
                assert!(
                    settled["textures"].as_i64().unwrap() <= baseline["textures"].as_i64().unwrap(),
                    "Unused vector sizes must release their atlas pages: {baseline} -> {settled}"
                );
            }
        },
    );
}
