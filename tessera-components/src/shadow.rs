//! Shadow layer definitions for Material surfaces.
//!
//! ## Usage
//!
//! Define ambient and spot shadow layers for surfaces.

use tessera_ui::{Color, Dp};

use crate::theme::MaterialColorScheme;

/// A single shadow layer (ambient or spot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowLayer {
    /// Color of the shadow (RGBA).
    pub color: Color,
    /// Offset of the shadow in the format [x, y].
    pub offset: [f32; 2],
    /// Smoothness / blur radius in pixels.
    pub smoothness: f32,
}

/// Collection of shadow layers (ambient + spot).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShadowLayers {
    /// Ambient (diffused) shadow layer.
    pub ambient: Option<ShadowLayer>,
    /// Spot (directional / offset) shadow layer.
    pub spot: Option<ShadowLayer>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ElevationGeometry {
    key_y: f64,
    key_blur: f64,
    ambient_y: f64,
    ambient_blur: f64,
    spread: f64,
}

fn elevation_geometry(elevation: Dp) -> ElevationGeometry {
    const LEVELS: [(f64, [f64; 5]); 6] = [
        (0.0, [0.0, 0.0, 0.0, 0.0, 0.0]),
        (1.0, [1.0, 2.0, 1.0, 3.0, 1.0]),
        (3.0, [1.0, 2.0, 2.0, 6.0, 2.0]),
        (6.0, [1.0, 3.0, 4.0, 8.0, 3.0]),
        (8.0, [2.0, 3.0, 6.0, 10.0, 4.0]),
        (12.0, [4.0, 4.0, 8.0, 12.0, 6.0]),
    ];
    let elevation = if elevation.0.is_nan() {
        0.0
    } else {
        elevation.0.clamp(0.0, 12.0)
    };
    let mut values = LEVELS[5].1;
    for pair in LEVELS.windows(2) {
        let [(start, from), (end, to)] = pair else {
            unreachable!()
        };
        if elevation <= *end {
            let fraction = (elevation - start) / (end - start);
            values =
                std::array::from_fn(|index| from[index] + (to[index] - from[index]) * fraction);
            break;
        }
    }
    ElevationGeometry {
        key_y: values[0],
        key_blur: values[1],
        ambient_y: values[2],
        ambient_blur: values[3],
        spread: values[4],
    }
}

pub(crate) fn material_shadow(elevation: Dp, scheme: &MaterialColorScheme) -> (ShadowLayers, f32) {
    let geometry = elevation_geometry(elevation);
    if geometry.key_blur == 0.0 {
        return (ShadowLayers::default(), 0.0);
    }
    // CSS blur uses sigma = blur / 2; the GPU kernel radius is three sigma.
    let layer = |y: f64, blur: f64, alpha| ShadowLayer {
        color: scheme.shadow.with_alpha(alpha),
        offset: [0.0, Dp(y).to_pixels_f32()],
        smoothness: Dp(blur * 1.5).to_pixels_f32(),
    };
    (
        ShadowLayers {
            ambient: Some(layer(geometry.ambient_y, geometry.ambient_blur, 0.15)),
            spot: Some(layer(geometry.key_y, geometry.key_blur, 0.3)),
        },
        Dp(geometry.spread).to_pixels_f32(),
    )
}

#[cfg(test)]
mod tests {
    use tessera_ui::Dp;

    use crate::theme::MaterialColorScheme;

    use super::{ElevationGeometry, elevation_geometry, material_shadow};

    #[test]
    fn menu_elevation_converts_blur_and_spread_without_losing_density() {
        let (layers, spread) = material_shadow(Dp(3.0), &MaterialColorScheme::default());
        let pixel = Dp(1.0).to_pixels_f32();
        let key = layers.spot.expect("key shadow");
        let ambient = layers.ambient.expect("ambient shadow");
        assert_eq!(key.offset, [0.0, pixel]);
        assert_eq!(key.smoothness, 3.0 * pixel);
        assert_eq!(key.color.a, 0.3);
        assert_eq!(ambient.offset, [0.0, 2.0 * pixel]);
        assert_eq!(ambient.smoothness, 9.0 * pixel);
        assert_eq!(ambient.color.a, 0.15);
        assert_eq!(spread, 2.0 * pixel);
        let (zero, spread) = material_shadow(Dp::ZERO, &MaterialColorScheme::default());
        assert!(zero.ambient.is_none() && zero.spot.is_none());
        assert_eq!(spread, 0.0);
    }

    #[test]
    fn material_elevation_levels_use_two_distinct_shadow_profiles() {
        for (elevation, expected) in [
            (1.0, [1.0, 2.0, 1.0, 3.0, 1.0]),
            (3.0, [1.0, 2.0, 2.0, 6.0, 2.0]),
            (6.0, [1.0, 3.0, 4.0, 8.0, 3.0]),
            (8.0, [2.0, 3.0, 6.0, 10.0, 4.0]),
            (12.0, [4.0, 4.0, 8.0, 12.0, 6.0]),
        ] {
            assert_eq!(
                elevation_geometry(Dp(elevation)),
                ElevationGeometry {
                    key_y: expected[0],
                    key_blur: expected[1],
                    ambient_y: expected[2],
                    ambient_blur: expected[3],
                    spread: expected[4]
                }
            );
        }
        let midpoint = elevation_geometry(Dp(2.0));
        assert_eq!(midpoint.ambient_y, 1.5);
        assert_eq!(midpoint.ambient_blur, 4.5);
        assert_eq!(midpoint.spread, 1.5);
        assert_eq!(elevation_geometry(Dp(-1.0)), elevation_geometry(Dp::ZERO));
        assert_eq!(
            elevation_geometry(Dp(f64::NAN)),
            elevation_geometry(Dp::ZERO)
        );
        assert_eq!(
            elevation_geometry(Dp(f64::INFINITY)),
            elevation_geometry(Dp(12.0))
        );
    }
}
