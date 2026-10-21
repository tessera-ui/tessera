//! Interruptible motion for transient surfaces.
//!
//! ## Usage
//!
//! Preserve position and velocity while surfaces open, close and reverse.

use super::material_motion::{Scalar, Spec};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct VisibilityMotion {
    position: Scalar,
    opacity: Scalar,
    spatial: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VisibilitySample {
    pub position: f32,
    pub opacity: f32,
    pub active: bool,
}

impl VisibilityMotion {
    pub fn new(open: bool, spatial: bool) -> Self {
        let value = f64::from(open);
        Self {
            position: Scalar::new(value),
            opacity: Scalar::new(value),
            spatial,
        }
    }

    pub fn retarget(&mut self, open: bool, frame: u64) {
        let target = f64::from(open);
        self.position.retarget(
            target,
            if self.spatial {
                Spec::Spatial
            } else {
                Spec::Effects
            },
            frame,
        );
        self.opacity.retarget(target, Spec::Effects, frame);
    }

    pub fn sample(self, frame: u64) -> VisibilitySample {
        let mut position = self.position;
        let mut opacity = self.opacity;
        position.advance(frame);
        opacity.advance(frame);
        VisibilitySample {
            position: position.value.clamp(0.0, 1.0) as f32,
            opacity: opacity.value.clamp(0.0, 1.0) as f32,
            active: position.is_animating() || opacity.is_animating(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VisibilityMotion;

    #[test]
    fn reversal_samples_current_values_and_preserves_incoming_velocities() {
        let mut motion = VisibilityMotion::new(false, true);
        motion.retarget(true, 0);
        let before = motion.sample(100_000_000);
        assert!((before.position - 0.65983106).abs() < 0.000001);
        assert!((before.opacity - 0.9084218).abs() < 0.000001);
        motion.retarget(false, 100_000_000);
        let after = motion.sample(100_000_000);
        assert_eq!(after.position, before.position);
        assert_eq!(after.opacity, before.opacity);
        assert!((motion.position.velocity - 6.2883265905).abs() < 0.000001);
        assert!((motion.opacity.velocity - 2.9305022222).abs() < 0.000001);
        let closing = motion.sample(130_000_000);
        motion.retarget(true, 130_000_000);
        assert_eq!(motion.sample(130_000_000).position, closing.position);
        assert_eq!(motion.sample(130_000_000).opacity, closing.opacity);
        let settled = motion.sample(2_000_000_000);
        assert_eq!(
            (settled.position, settled.opacity, settled.active),
            (1.0, 1.0, false)
        );
    }

    #[test]
    fn effects_initial_states_redundant_targets_and_same_frame_reversal_settle() {
        let mut initial = VisibilityMotion::new(true, false);
        assert!(!initial.sample(0).active);
        initial.retarget(false, 0);
        initial.retarget(true, 0);
        assert!(!initial.sample(0).active);
        let mut motion = VisibilityMotion::new(false, false);
        motion.retarget(true, 0);
        motion.retarget(true, 50_000_000);
        assert!((motion.sample(100_000_000).opacity - 0.9084218).abs() < 0.000001);
        motion.retarget(false, 100_000_000);
        let settled = motion.sample(2_000_000_000);
        assert_eq!(
            (settled.position, settled.opacity, settled.active),
            (0.0, 0.0, false)
        );
    }
}
