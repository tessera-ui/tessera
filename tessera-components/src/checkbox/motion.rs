//! Selection-stroke and color motion.
//!
//! ## Usage
//!
//! Animate checkbox state changes and preserve interrupted transitions.

use tessera_ui::Color;

pub(super) use crate::animation::material_motion::{Scalar, Spec};

#[cfg(test)]
mod tests {
    use tessera_ui::Color;

    use super::{Colors, ColorsMotion, Scalar, Spec, color_vector, vector_color};

    #[test]
    fn spring_samples_match_reference_parameters_and_ignore_frame_partitioning() {
        for (spec, expected_position, expected_velocity) in [
            (Spec::Spatial, 0.6598310577, 6.2883265905),
            (Spec::Effects, 0.9084218056, 2.9305022222),
            (Spec::FastEffects, 0.9849335932, 0.7991211240),
        ] {
            let mut direct = Scalar::new(0.0);
            direct.retarget(1.0, spec, 0);
            let mut partitioned = direct;
            direct.advance(100_000_000);
            for frame in [9_000_000, 23_000_000, 77_000_000, 100_000_000] {
                partitioned.advance(frame);
            }
            assert!((direct.value - expected_position).abs() < 0.000001);
            assert!((direct.velocity - expected_velocity).abs() < 0.000001);
            assert_eq!(direct.value, partitioned.value);
            assert_eq!(direct.velocity, partitioned.velocity);
            if spec == Spec::Spatial {
                direct.advance(250_000_000);
                assert!(
                    direct.value > 1.0,
                    "the spatial spring must retain its overshoot"
                );
                assert!(direct.is_animating());
            }
            direct.advance(2_000_000_000);
            assert_eq!(direct.value, 1.0);
            assert!(!direct.is_animating());
        }
    }

    #[test]
    fn oklab_conversions_match_independent_primary_and_neutral_values() {
        let red = color_vector(Color::RED);
        for (actual, expected) in red
            .into_iter()
            .zip([1.0, 0.62795536, 0.22486306, 0.12584630])
        {
            assert!((actual - expected).abs() < 0.000001);
        }
        let neutral = vector_color([0.25, 0.5, 0.0, 0.0]);
        for channel in [neutral.r, neutral.g, neutral.b] {
            assert!((channel - 0.125).abs() < 0.000001);
        }
        assert_eq!(neutral.a, 0.25);
    }

    #[test]
    fn colors_use_shared_effect_timing_and_snap_disabled_surfaces() {
        let off = Colors {
            fill: Color::TRANSPARENT,
            border: Color::RED,
            icon: Color::TRANSPARENT,
            active: false,
            enabled: true,
            snap_revision: 0,
        };
        let on = Colors {
            fill: Color::RED,
            border: Color::RED,
            icon: Color::WHITE,
            active: true,
            ..off
        };
        let mut motion = ColorsMotion::new(off);
        motion.retarget(on, 0);
        motion.advance(50_000_000);
        let (fill, _, icon) = motion.value();
        assert!((fill.a - 0.59399415).abs() < 0.00001);
        assert!((icon.r - 0.20957839).abs() < 0.0001);
        let disabled = Colors {
            fill: Color::BLUE,
            border: Color::BLUE,
            icon: Color::WHITE,
            enabled: false,
            ..on
        };
        motion.retarget(disabled, 50_000_000);
        assert_eq!(motion.value().0, Color::BLUE);
        assert_eq!(motion.value().1, Color::BLUE);
        motion.advance(1_000_000_000);
        motion.retarget(
            Colors {
                snap_revision: 1,
                ..off
            },
            1_000_000_000,
        );
        assert_eq!(motion.value(), (off.fill, off.border, off.icon));
        assert!(!motion.is_animating());
    }
}

fn color_vector(color: Color) -> [f64; 4] {
    let (r, g, b) = (f64::from(color.r), f64::from(color.g), f64::from(color.b));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        f64::from(color.a),
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

fn vector_color(vector: [f64; 4]) -> Color {
    let [alpha, lightness, a, b] = vector;
    let lightness = lightness.clamp(0.0, 1.0);
    let a = a.clamp(-0.5, 0.5);
    let b = b.clamp(-0.5, 0.5);
    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    Color::new(
        (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s).clamp(0.0, 1.0) as f32,
        (-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s).clamp(0.0, 1.0) as f32,
        (-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s).clamp(0.0, 1.0) as f32,
        alpha.clamp(0.0, 1.0) as f32,
    )
}

#[derive(Clone, PartialEq)]
struct ColorMotion {
    channels: [Scalar; 4],
    target: Color,
}

impl ColorMotion {
    fn new(color: Color) -> Self {
        Self {
            channels: color_vector(color).map(Scalar::new),
            target: color,
        }
    }

    fn retarget(&mut self, color: Color, spec: Spec, frame: u64, snap: bool) {
        if snap {
            *self = Self::new(color);
        } else if self.target != color {
            for (channel, target) in self.channels.iter_mut().zip(color_vector(color)) {
                channel.start_to(target, spec, frame);
            }
            let duration = self
                .channels
                .iter()
                .map(|channel| channel.duration_millis)
                .max()
                .unwrap_or(0);
            for channel in &mut self.channels {
                channel.duration_millis = duration;
            }
            self.target = color;
        }
    }

    fn advance(&mut self, frame: u64) {
        for channel in &mut self.channels {
            channel.advance(frame);
        }
    }

    fn is_animating(&self) -> bool {
        self.channels.iter().any(Scalar::is_animating)
    }

    fn value(&self) -> Color {
        if self.is_animating() {
            vector_color(self.channels.map(|channel| channel.value))
        } else {
            self.target
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Colors {
    pub fill: Color,
    pub border: Color,
    pub icon: Color,
    pub enabled: bool,
    pub active: bool,
    pub snap_revision: u64,
}

#[derive(Clone, PartialEq)]
pub(super) struct ColorsMotion {
    fill: ColorMotion,
    border: ColorMotion,
    icon: ColorMotion,
    pub target: Colors,
}

impl ColorsMotion {
    pub fn new(target: Colors) -> Self {
        Self {
            fill: ColorMotion::new(target.fill),
            border: ColorMotion::new(target.border),
            icon: ColorMotion::new(target.icon),
            target,
        }
    }

    pub fn retarget(&mut self, target: Colors, frame: u64) {
        self.advance(frame);
        let direct = self.target.snap_revision != target.snap_revision;
        let snap = direct || !target.enabled || self.target.enabled != target.enabled;
        let spec = if target.active {
            Spec::Effects
        } else {
            Spec::FastEffects
        };
        self.fill.retarget(target.fill, spec, frame, snap);
        self.border.retarget(target.border, spec, frame, snap);
        self.icon.retarget(target.icon, spec, frame, direct);
        self.target = target;
    }

    pub fn advance(&mut self, frame: u64) {
        self.fill.advance(frame);
        self.border.advance(frame);
        self.icon.advance(frame);
    }

    pub fn is_animating(&self) -> bool {
        self.fill.is_animating() || self.border.is_animating() || self.icon.is_animating()
    }

    pub fn value(&self) -> (Color, Color, Color) {
        (self.fill.value(), self.border.value(), self.icon.value())
    }
}
