//! Selection-stroke and color motion.
//!
//! ## Usage
//!
//! Animate checkbox state changes and preserve interrupted transitions.

use tessera_ui::{Color, time::Instant};

use crate::animation::{Motion, color_motion::ColorMotion};

pub(super) use crate::animation::material_motion::{Scalar, Spec};

#[cfg(test)]
mod tests {
    use tessera_ui::Color;

    use crate::animation::{Motion as _, test_instant as at};

    use super::{Colors, ColorsMotion, Scalar, Spec};

    #[test]
    fn spring_samples_match_reference_parameters_and_ignore_frame_partitioning() {
        for (spec, expected_position, expected_velocity) in [
            (Spec::Spatial, 0.6598310577, 6.2883265905),
            (Spec::Effects, 0.9084218056, 2.9305022222),
            (Spec::FastEffects, 0.9849335932, 0.7991211240),
        ] {
            let mut direct = Scalar::new(0.0);
            direct.retarget(1.0, spec, at(0));
            let mut partitioned = direct;
            direct.advance(at(100_000_000));
            for frame in [9_000_000, 23_000_000, 77_000_000, 100_000_000] {
                partitioned.advance(at(frame));
            }
            assert!((direct.value - expected_position).abs() < 0.000001);
            assert!((direct.velocity - expected_velocity).abs() < 0.000001);
            assert_eq!(direct.value, partitioned.value);
            assert_eq!(direct.velocity, partitioned.velocity);
            if spec == Spec::Spatial {
                direct.advance(at(250_000_000));
                assert!(
                    direct.value > 1.0,
                    "the spatial spring must retain its overshoot"
                );
                assert!(direct.is_animating());
            }
            direct.advance(at(2_000_000_000));
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
        motion.retarget(on, at(0));
        motion.advance(at(50_000_000));
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
        motion.retarget(disabled, at(50_000_000));
        assert_eq!(motion.value().0, Color::BLUE);
        assert_eq!(motion.value().1, Color::BLUE);
        motion.advance(at(1_000_000_000));
        motion.retarget(
            Colors {
                snap_revision: 1,
                ..off
            },
            at(1_000_000_000),
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

    pub fn is_animating(&self) -> bool {
        self.fill.is_animating() || self.border.is_animating() || self.icon.is_animating()
    }

    pub fn value(&self) -> (Color, Color, Color) {
        (self.fill.value(), self.border.value(), self.icon.value())
    }

    // Returns whether moving to `target` jumps the icon and the surface
    // colors, respectively, instead of animating them.
    fn snaps(&self, target: &Colors) -> (bool, bool) {
        let direct = self.target.snap_revision != target.snap_revision;
        let surface = direct || !target.enabled || self.target.enabled != target.enabled;
        (direct, surface)
    }

    /// Applies the jumps of a change to `target` without a frame time, so they
    /// show in the same frame; [`Motion::retarget`] then starts the animated
    /// part.
    pub fn snap(&mut self, target: &Colors) {
        let (direct, surface) = self.snaps(target);
        if surface {
            self.fill = ColorMotion::new(target.fill);
            self.border = ColorMotion::new(target.border);
        }
        if direct {
            self.icon = ColorMotion::new(target.icon);
        }
    }
}

impl Motion for ColorsMotion {
    type Target = Colors;

    fn is_settled_at(&self, target: &Colors) -> bool {
        self.target == *target && !self.is_animating()
    }

    fn retarget(&mut self, target: Colors, now: Instant) {
        self.advance(now);
        let (direct, snap) = self.snaps(&target);

        let spec = if target.active {
            Spec::Effects
        } else {
            Spec::FastEffects
        };
        self.fill.retarget(target.fill, spec, now, snap);
        self.border.retarget(target.border, spec, now, snap);
        self.icon.retarget(target.icon, spec, now, direct);
        self.target = target;
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.fill.advance(now);
        self.border.advance(now);
        self.icon.advance(now);
        self.is_animating()
    }
}
