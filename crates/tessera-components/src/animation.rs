//! Animation mapping for UI components.
//!
//! ## Usage
//!
//! Animate state changes and preserve interrupted transitions.

pub(crate) mod color_motion;
pub(crate) mod material_motion;
pub(crate) mod shape_motion;
pub(crate) mod sheet_drag;
pub(crate) mod visibility_motion;

use std::time::Duration;

use tessera_ui::{State, time::Instant, with_frame_time};

/// State advanced from frame times toward a target.
pub(crate) trait Motion: 'static {
    type Target;

    /// Returns whether the motion rests at `target`.
    fn is_settled_at(&self, target: &Self::Target) -> bool;

    /// Starts moving toward `target` from the state at `now`.
    fn retarget(&mut self, target: Self::Target, now: Instant);

    /// Moves to `now` and returns whether the motion is still running.
    fn advance(&mut self, now: Instant) -> bool;
}

/// Measures the time between frames for integrated motion.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct FrameDelta(Option<Instant>);

impl FrameDelta {
    /// Returns the seconds since the previous step, or zero for the first one.
    pub(crate) fn step(&mut self, now: Instant) -> f32 {
        let previous = self.0.unwrap_or(now);
        let now = now.max(previous);
        self.0 = Some(now);
        now.saturating_duration_since(previous).as_secs_f32()
    }
}

/// A spring integrated over frame time deltas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpringMotion {
    spring: Animatable,
    delta: FrameDelta,
}

impl SpringMotion {
    pub(crate) fn new(spring: Animatable) -> Self {
        Self {
            spring,
            delta: FrameDelta::default(),
        }
    }

    pub(crate) fn value(&self) -> f32 {
        self.spring.value()
    }

    #[cfg(test)]
    pub(crate) fn spring(&self) -> &Animatable {
        &self.spring
    }
}

impl Motion for SpringMotion {
    type Target = f32;

    fn is_settled_at(&self, target: &f32) -> bool {
        self.spring.target() == *target && self.spring.is_idle()
    }

    fn retarget(&mut self, target: f32, now: Instant) {
        self.advance(now);
        self.spring.set_target(target);
    }

    fn advance(&mut self, now: Instant) -> bool {
        let dt = self.delta.step(now);
        self.spring.tick(dt);
        !self.spring.is_idle()
    }
}

/// Calls `step` with each following frame's time until it returns `false`.
pub(crate) async fn run_frames<T: 'static>(
    state: State<T>,
    mut step: impl FnMut(&mut T, Instant) -> bool,
) {
    loop {
        let running = with_frame_time(|now| state.with_mut(|value| step(value, now))).await;
        if !running {
            break;
        }
    }
}

/// Retargets `motion` on the next frame and advances it until it settles.
///
/// Launch it keyed by `target` so that a new target restarts the motion from
/// its current value and velocity.
pub(crate) async fn animate_to<M: Motion>(motion: State<M>, target: M::Target) {
    if motion.with(|motion| motion.is_settled_at(&target)) {
        return;
    }
    let mut target = Some(target);
    run_frames(motion, move |motion, now| {
        if let Some(target) = target.take() {
            motion.retarget(target, now);
        }
        motion.advance(now)
    })
    .await;
}

/// A delay measured from an input timestamp or, without one, from the first
/// frame that observes it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Delay {
    start: Option<Instant>,
    duration: Duration,
}

impl Delay {
    pub(crate) fn new(start: Option<Instant>, duration: Duration) -> Self {
        Self { start, duration }
    }

    pub(crate) fn elapsed(&mut self, now: Instant) -> bool {
        let start = *self.start.get_or_insert(now);
        now.saturating_duration_since(start) >= self.duration
    }
}

#[cfg(test)]
pub(crate) fn test_instant(nanos: u64) -> Instant {
    static ORIGIN: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);
    *ORIGIN + Duration::from_nanos(nanos)
}

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

/// Interruptible scalar animation.
///
/// Changing the target preserves spring velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Animatable {
    value: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
}

impl Animatable {
    /// Spring starting at rest on `value`.
    pub(crate) fn spring(value: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            stiffness,
            damping,
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
    /// Spring velocity is left untouched.
    pub(crate) fn set_target(&mut self, target: f32) {
        self.target = target;
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
        self.integrate_spring(dt, self.stiffness, self.damping);
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

#[derive(Clone, PartialEq)]
pub(crate) struct SelectionAnimation {
    selected: bool,
    progress: f32,
    start_progress: f32,
    started: Option<Instant>,
}

impl SelectionAnimation {
    pub(crate) fn new(selected: bool) -> Self {
        let progress = if selected { 1.0 } else { 0.0 };
        Self {
            selected,
            progress,
            start_progress: progress,
            started: None,
        }
    }

    fn target(&self) -> f32 {
        if self.selected { 1.0 } else { 0.0 }
    }

    pub(crate) fn progress(&self) -> f32 {
        self.progress
    }
}

impl Motion for SelectionAnimation {
    type Target = bool;

    fn is_settled_at(&self, selected: &bool) -> bool {
        self.selected == *selected && self.started.is_none()
    }

    fn retarget(&mut self, selected: bool, now: Instant) {
        if self.selected != selected {
            self.selected = selected;
            self.start_progress = self.progress;
            self.started = Some(now);
        }
    }

    fn advance(&mut self, now: Instant) -> bool {
        let Some(started) = self.started else {
            return false;
        };
        let fraction = (now.saturating_duration_since(started).as_secs_f32()
            / SELECTION_DURATION.as_secs_f32())
        .min(1.0);
        let target = self.target();
        self.progress = self.start_progress + (target - self.start_progress) * easing(fraction);
        if fraction >= 1.0 {
            self.started = None;
            self.progress = target;
            self.start_progress = target;
        }
        self.started.is_some()
    }
}

const SELECTION_DURATION: Duration = Duration::from_millis(300);
