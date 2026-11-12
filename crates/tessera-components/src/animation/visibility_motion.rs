//! Interruptible motion for transient surfaces.
//!
//! ## Usage
//!
//! Preserve position and velocity while surfaces open, close and reverse.

use tessera_ui::{
    Callback, CallbackWith,
    system_back::{SystemBackEvent, system_back_handler},
    tessera,
    time::Instant,
};

use super::{
    Motion,
    material_motion::{Scalar, Spec},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct VisibilityMotion {
    position: Scalar,
    opacity: Scalar,
    spatial: bool,
    back_start: Option<(f32, f32)>,
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
            back_start: None,
        }
    }

    fn spec(&self) -> Spec {
        if self.spatial {
            Spec::Spatial
        } else {
            Spec::Effects
        }
    }

    /// Retargets the motion at the next [`Motion::advance`].
    pub fn request(&mut self, open: bool) {
        self.back_start = None;
        let target = f64::from(open);
        self.position.request_retarget(target, self.spec());
        self.opacity.request_retarget(target, Spec::Effects);
    }

    /// Follows a predictive back gesture; a finished gesture requests the open
    /// state again.
    pub fn preview_back(&mut self, event: SystemBackEvent) {
        match event {
            SystemBackEvent::Started(_) => {
                let start = self.sample();
                self.position.snap(f64::from(start.position));
                self.opacity.snap(f64::from(start.opacity));
                self.back_start = Some((start.position, start.opacity));
            }
            SystemBackEvent::Progress { progress, .. } => {
                if let Some((position, opacity)) = self.back_start {
                    let visible = 1.0 - progress;
                    self.position.snap(f64::from(position * visible));
                    self.opacity.snap(f64::from(opacity * visible));
                }
            }
            SystemBackEvent::Cancelled | SystemBackEvent::Committed => {
                if self.back_start.is_some() {
                    self.request(true);
                }
            }
        }
    }

    pub fn sample(&self) -> VisibilitySample {
        VisibilitySample {
            position: self.position.value.clamp(0.0, 1.0) as f32,
            opacity: self.opacity.value.clamp(0.0, 1.0) as f32,
            active: self.position.is_animating() || self.opacity.is_animating(),
        }
    }
}

impl Motion for VisibilityMotion {
    type Target = bool;

    fn is_settled_at(&self, open: &bool) -> bool {
        let target = f64::from(*open);
        !self.sample().active && self.position.value == target && self.opacity.value == target
    }

    fn retarget(&mut self, open: bool, now: Instant) {
        self.back_start = None;
        let target = f64::from(open);
        self.position.retarget(target, self.spec(), now);
        self.opacity.retarget(target, Spec::Effects, now);
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.position.advance(now);
        self.opacity.advance(now);
        self.sample().active
    }
}

#[tessera]
pub(crate) fn modal_back_handler(
    enabled: bool,
    on_close: Callback,
    on_preview: Option<CallbackWith<SystemBackEvent>>,
) {
    system_back_handler(enabled).on_back(move |event| {
        if let Some(preview) = on_preview {
            preview.call(event);
        }
        if event == SystemBackEvent::Committed {
            on_close.call();
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::animation::{Motion, test_instant as at};

    use super::VisibilityMotion;

    #[test]
    fn reversal_samples_current_values_and_preserves_incoming_velocities() {
        let mut motion = VisibilityMotion::new(false, true);
        motion.retarget(true, at(0));
        motion.advance(at(100_000_000));
        let before = motion.sample();
        assert!((before.position - 0.65983106).abs() < 0.000001);
        assert!((before.opacity - 0.9084218).abs() < 0.000001);
        motion.retarget(false, at(100_000_000));
        let after = motion.sample();
        assert_eq!(after.position, before.position);
        assert_eq!(after.opacity, before.opacity);
        assert!((motion.position.velocity - 6.2883265905).abs() < 0.000001);
        assert!((motion.opacity.velocity - 2.9305022222).abs() < 0.000001);
        motion.advance(at(130_000_000));
        let closing = motion.sample();
        motion.retarget(true, at(130_000_000));
        assert_eq!(motion.sample().position, closing.position);
        assert_eq!(motion.sample().opacity, closing.opacity);
        assert!(!motion.advance(at(2_000_000_000)));
        let settled = motion.sample();
        assert_eq!(
            (settled.position, settled.opacity, settled.active),
            (1.0, 1.0, false)
        );
    }

    #[test]
    fn effects_initial_states_redundant_targets_and_same_frame_reversal_settle() {
        let mut initial = VisibilityMotion::new(true, false);
        assert!(initial.is_settled_at(&true));
        initial.retarget(false, at(0));
        initial.retarget(true, at(0));
        assert!(!initial.sample().active);
        assert!(initial.is_settled_at(&true));
        let mut motion = VisibilityMotion::new(false, false);
        motion.retarget(true, at(0));
        motion.retarget(true, at(50_000_000));
        motion.advance(at(100_000_000));
        assert!((motion.sample().opacity - 0.9084218).abs() < 0.000001);
        motion.retarget(false, at(100_000_000));
        motion.advance(at(2_000_000_000));
        let settled = motion.sample();
        assert_eq!(
            (settled.position, settled.opacity, settled.active),
            (0.0, 0.0, false)
        );
        assert!(motion.is_settled_at(&false));
    }
}
