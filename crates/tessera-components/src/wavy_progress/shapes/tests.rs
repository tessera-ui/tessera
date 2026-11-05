use super::*;

fn read_u32(bytes: &mut &[u8]) -> u32 {
    let (value, tail) = bytes.split_at(4);
    *bytes = tail;
    u32::from_le_bytes(value.try_into().unwrap())
}

fn read_curves(bytes: &mut &[u8]) -> Vec<Cubic> {
    let count = read_u32(bytes);
    (0..count)
        .map(|_| {
            Cubic(std::array::from_fn(|_| {
                Vec2::new(
                    f32::from_bits(read_u32(bytes)),
                    f32::from_bits(read_u32(bytes)),
                )
            }))
        })
        .collect()
}

fn samples(curves: &[Cubic]) -> Vec<Vec2> {
    curves
        .iter()
        .flat_map(|curve| (0..=16).map(|i| curve.point(i as f32 / 16.0)))
        .collect()
}

fn assert_outline(actual: &[Cubic], expected: &[Cubic], n: usize, amplitude: f32) {
    let error = |points: Vec<Vec2>, curves: &[Cubic]| {
        points
            .into_iter()
            .map(|point| {
                curves
                    .iter()
                    .flat_map(|curve| {
                        (0..64).map(move |i| {
                            (
                                curve.point(i as f32 / 64.0),
                                curve.point((i + 1) as f32 / 64.0),
                            )
                        })
                    })
                    .map(|(a, b)| {
                        let d = b - a;
                        point.distance(
                            a + d
                                * ((point - a).dot(d) / d.length_squared().max(1e-15))
                                    .clamp(0.0, 1.0),
                        )
                    })
                    .fold(f32::INFINITY, f32::min)
            })
            .fold(0.0, f32::max)
    };
    let distance = error(samples(actual), expected).max(error(samples(expected), actual));
    assert!(
        distance < 0.00015,
        "n={n}, amplitude={amplitude}: outline error {distance}"
    );
    assert!(
        actual[0].0[0].distance(expected[0].0[0]) < 0.00015,
        "n={n}, amplitude={amplitude}: path phase differs"
    );
    assert!(actual.last().unwrap().0[3].distance(actual[0].0[0]) < 0.00001);
}

#[test]
fn circular_morph_matches_independent_graphics_shapes_reference() {
    let mut bytes = &include_bytes!("../../../assets/circular_wave_reference.bin")[..];
    assert_eq!(&bytes[..4], b"MCW1");
    bytes = &bytes[4..];
    let count = read_u32(&mut bytes);
    for _ in 0..count {
        let n = read_u32(&mut bytes) as usize;
        let morph = circular_morph(n);
        assert_outline(&morph.track, &read_curves(&mut bytes), n, -1.0);
        for amplitude in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let mut curves: Vec<_> = morph
                .pairs
                .iter()
                .map(|pair| {
                    Cubic(std::array::from_fn(|i| {
                        pair[0].0[i].lerp(pair[1].0[i], amplitude)
                    }))
                })
                .collect();
            let first = curves[0].0[0];
            curves.last_mut().unwrap().0[3] = first;
            assert_outline(&curves, &read_curves(&mut bytes), n, amplitude);
        }
    }
    assert!(bytes.is_empty());
}
