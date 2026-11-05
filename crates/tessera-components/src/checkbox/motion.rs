//! Selection-stroke and color motion.
//!
//! ## Usage
//!
//! Animate checkbox state changes and preserve interrupted transitions.

use tessera_ui::Color;

use crate::animation::color_motion::ColorMotion;

pub(super) use crate::animation::material_motion::{Scalar, Spec};

#[cfg(test)]
mod tests {
    use tessera_ui::Color;

    use super::{Colors, ColorsMotion, Scalar, Spec};

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
