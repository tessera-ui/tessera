//! Animation mapping for UI components.
///
/// Cubic ease-in-out mapping (smooth start and end).
/// Input: linear progress in [0.0, 1.0].
/// Output: eased progress in [0.0, 1.0].
pub(crate) fn easing(progress: f32) -> f32 {
    // Cubic ease-in-out
    let t = progress.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Motion model used by [`Animatable`].
///
/// Spring motion is a velocity-preserving oscillator. Easing motion
/// approaches its target over a finite duration and does not inherit velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AnimationSpec {
    /// Spring. `stiffness` is the natural frequency ω (rad/s).
    /// `damping` is the damping ratio ζ in `[0.0, 1.0]`.
    Spring { stiffness: f32, damping: f32 },
    /// Finite-duration cubic ease-in-out toward the target. Velocity is not
    /// carried.
    Easing { duration_secs: f32 },
}

/// Interruptible scalar animation.
///
/// Changing the target preserves spring velocity and restarts easing from the
/// current value without inheriting velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Animatable {
    value: f32,
    velocity: f32,
    target: f32,
    spec: AnimationSpec,
    /// Seconds already spent on the current easing segment. Unused by springs.
    ease_elapsed: f32,
    /// Value captured when the current easing segment started.
    ease_origin: f32,
}

impl Animatable {
    /// Spring starting at rest on `value`.
    pub(crate) fn spring(value: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            spec: AnimationSpec::Spring { stiffness, damping },
            ease_elapsed: 0.0,
            ease_origin: value,
        }
    }

    /// Finite ease starting at rest on `value`.
    pub(crate) fn easing(value: f32, duration_secs: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            spec: AnimationSpec::Easing { duration_secs },
            ease_elapsed: 0.0,
            ease_origin: value,
        }
    }

    /// Override velocity. Used when a spring retarget must keep the carried
    /// speed.
    pub(crate) fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity;
        self
    }

    /// Current integrated value.
    pub(crate) fn value(&self) -> f32 {
        self.value
    }

    /// Current velocity in units per second.
    pub(crate) fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Value the animation is moving toward.
    pub(crate) fn target(&self) -> f32 {
        self.target
    }

    /// Point the animation at a new target.
    ///
    /// Spring velocity is left untouched. Easing drops velocity and restarts
    /// its finite approach from the current value.
    pub(crate) fn set_target(&mut self, target: f32) {
        if (self.target - target).abs() <= f32::EPSILON {
            self.target = target;
            return;
        }
        self.target = target;
        if matches!(self.spec, AnimationSpec::Easing { .. }) {
            self.velocity = 0.0;
            self.ease_elapsed = 0.0;
            self.ease_origin = self.value;
        }
    }

    /// Advance by `dt` seconds taken from the caller (a frame-time delta).
    ///
    /// Large steps are subdivided so an underdamped spring cannot diverge.
    /// At rest, `value == target` and velocity is approximately zero.
    pub(crate) fn tick(&mut self, dt: f32) -> f32 {
        if !dt.is_finite() || dt <= 0.0 {
            return self.value;
        }
        let dt = dt.min(0.25);
        match self.spec {
            AnimationSpec::Spring { stiffness, damping } => {
                self.integrate_spring(dt, stiffness, damping);
            }
            AnimationSpec::Easing { duration_secs } => {
                self.integrate_easing(dt, duration_secs);
            }
        }
        self.value
    }

    /// True when the value has settled on the target and velocity is ~0.
    pub(crate) fn is_idle(&self) -> bool {
        (self.value - self.target).abs() <= 1.0e-4 && self.velocity.abs() <= 1.0e-3
    }

    fn integrate_spring(&mut self, dt: f32, stiffness: f32, damping: f32) {
        let omega = stiffness.max(0.0);
        let zeta = damping.clamp(0.0, 1.0);
        if omega == 0.0 {
            self.value = self.target;
            self.velocity = 0.0;
            return;
        }

        let max_step = (0.25 / omega).clamp(1.0 / 480.0, 1.0 / 30.0);
        let mut remaining = dt;
        while remaining > 1.0e-8 {
            let step = remaining.min(max_step);
            remaining -= step;
            let displacement = self.value - self.target;
            let accel = -omega * omega * displacement - 2.0 * zeta * omega * self.velocity;
            self.velocity += accel * step;
            self.value += self.velocity * step;
            if !self.value.is_finite() || !self.velocity.is_finite() {
                self.value = self.target;
                self.velocity = 0.0;
                return;
            }
        }

        self.settle_if_resting();
    }

    fn integrate_easing(&mut self, dt: f32, duration_secs: f32) {
        let duration = duration_secs.max(1.0e-4);
        if self.ease_elapsed <= 0.0 && (self.value - self.target).abs() <= 1.0e-6 {
            self.value = self.target;
            self.velocity = 0.0;
            self.ease_elapsed = duration;
            self.ease_origin = self.target;
            return;
        }

        self.ease_elapsed = (self.ease_elapsed + dt).min(duration);
        let t = (self.ease_elapsed / duration).clamp(0.0, 1.0);
        let origin = self.ease_origin;
        self.value = origin + (self.target - origin) * easing(t);
        self.velocity = 0.0;
        if t >= 1.0 {
            self.value = self.target;
            self.ease_elapsed = duration;
        }
    }

    fn settle_if_resting(&mut self) {
        let displacement = (self.value - self.target).abs();
        if displacement <= 1.0e-4 && self.velocity.abs() <= 1.0e-3 {
            self.value = self.target;
            self.velocity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Animatable, easing};

    #[test]
    fn easing_clamps_progress_at_endpoints() {
        assert_eq!(easing(0.0), 0.0);
        assert_eq!(easing(1.0), 1.0);
        assert!((easing(0.5) - 0.5).abs() < 1.0e-6);
        assert_eq!(easing(-1.0), 0.0);
        assert_eq!(easing(2.0), 1.0);
    }

    #[test]
    fn spring_keeps_velocity_when_target_reverses() {
        let mut anim = Animatable::spring(0.0, 15.0, 0.35);
        anim.set_target(1.0);
        let dt = 1.0 / 60.0;
        for _ in 0..8 {
            anim.tick(dt);
        }
        let velocity_before = anim.velocity();
        let value_before = anim.value();
        assert!(
            velocity_before > 0.05,
            "expected outward velocity before reverse, got {velocity_before}"
        );

        anim.set_target(0.0);
        assert_eq!(anim.velocity(), velocity_before);
        assert_eq!(anim.value(), value_before);

        anim.tick(dt);
        let velocity_after = anim.velocity();
        assert!(velocity_after > 0.0, "got {velocity_after}");
        assert!(velocity_after < velocity_before);
        assert!((velocity_after - velocity_before).abs() < velocity_before);
    }

    #[test]
    fn easing_approaches_in_finite_time_without_inheriting_velocity() {
        let mut anim = Animatable::spring(0.0, 15.0, 0.2);
        anim.set_target(1.0);
        anim.tick(1.0 / 60.0);
        assert!(anim.velocity().abs() > 0.0);

        let mut anim = Animatable::easing(anim.value(), 0.25);
        assert_eq!(anim.velocity(), 0.0);
        anim.set_target(1.0);
        assert_eq!(anim.velocity(), 0.0);

        let start = anim.value();
        anim.tick(0.125);
        let mid = anim.value();
        assert!(mid > start && mid < 1.0, "mid={mid} start={start}");
        assert_eq!(anim.velocity(), 0.0);

        anim.tick(0.125);
        assert_eq!(anim.value(), 1.0);
        assert!(anim.velocity().abs() < 1.0e-6);
        assert!(anim.is_idle());
    }

    #[test]
    fn large_dt_does_not_diverge_and_rest_snaps() {
        let mut anim = Animatable::spring(0.0, 40.0, 0.15);
        anim.set_target(1.0);
        let value = anim.tick(2.0);
        assert!(value.is_finite(), "diverged to {value}");
        assert!(value.abs() < 8.0, "ran away: {value}");

        for _ in 0..240 {
            anim.tick(1.0 / 60.0);
        }
        assert_eq!(anim.value(), anim.target());
        assert!(anim.velocity().abs() < 1.0e-3);
        assert!(anim.is_idle());
    }
}
