//! Time-based scroll physics shared by scrollable components.
//!
//! The primitives in this module keep their state in floating point pixels so
//! sub-pixel input is accumulated until it can be presented by the renderer.

use std::{collections::VecDeque, time::Duration};

use tessera_ui::time::Instant;

/// A two-dimensional velocity in pixels per second.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollVelocity {
    /// Horizontal velocity in pixels per second.
    pub x: f32,
    /// Vertical velocity in pixels per second.
    pub y: f32,
}

impl ScrollVelocity {
    /// Creates a velocity from horizontal and vertical components.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns true when both components are zero.
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

/// A bounded, time-weighted velocity estimator for pointer samples.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollVelocityTracker {
    samples: VecDeque<(Instant, Duration, ScrollVelocity)>,
    pending_delta: ScrollVelocity,
    last_sample_time: Instant,
    sample_window: Duration,
    idle_cutoff: Duration,
}

impl ScrollVelocityTracker {
    /// Creates a tracker with the supplied history and idle durations.
    pub fn new(now: Instant, sample_window: Duration, idle_cutoff: Duration) -> Self {
        Self {
            samples: VecDeque::new(),
            pending_delta: ScrollVelocity::default(),
            last_sample_time: now,
            sample_window,
            idle_cutoff,
        }
    }

    /// Adds a displacement sample, accumulating samples with equal timestamps.
    pub fn push_delta(&mut self, now: Instant, dx: f32, dy: f32) {
        if now < self.last_sample_time || !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.pending_delta.x += dx;
        self.pending_delta.y += dy;
        let duration = now.duration_since(self.last_sample_time);
        if duration.is_zero() {
            return;
        }
        let dt = duration.as_secs_f32();
        self.last_sample_time = now;
        let velocity = ScrollVelocity::new(self.pending_delta.x / dt, self.pending_delta.y / dt);
        self.pending_delta = ScrollVelocity::default();
        if velocity.x.is_finite() && velocity.y.is_finite() {
            self.samples.push_back((now, duration, velocity));
            self.prune(now);
        }
    }

    /// Returns the weighted velocity at `now`, if recent samples exist.
    pub fn resolve(&mut self, now: Instant) -> Option<ScrollVelocity> {
        self.prune(now);
        let window = self.sample_window.as_secs_f32().max(f32::EPSILON);
        let mut sum = ScrollVelocity::default();
        let mut weight_sum = 0.0;
        for &(timestamp, duration, velocity) in &self.samples {
            let age = now
                .duration_since(timestamp)
                .as_secs_f32()
                .clamp(0.0, window);
            let covered = duration.as_secs_f32().min((window - age).max(0.0));
            let weight = covered * (window - age - covered * 0.5);
            sum.x += velocity.x * weight;
            sum.y += velocity.y * weight;
            weight_sum += weight;
        }
        if weight_sum <= f32::EPSILON {
            return None;
        }
        let idle = now.duration_since(self.last_sample_time).as_secs_f32();
        let cutoff = self.idle_cutoff.as_secs_f32().max(f32::EPSILON);
        let damping = (1.0 - idle / cutoff).clamp(0.0, 1.0);
        Some(ScrollVelocity::new(
            sum.x / weight_sum * damping,
            sum.y / weight_sum * damping,
        ))
    }

    /// Returns whether no sample arrived within the idle cutoff.
    pub fn is_idle(&self, now: Instant) -> bool {
        now.duration_since(self.last_sample_time) >= self.idle_cutoff
    }

    fn prune(&mut self, now: Instant) {
        while let Some(&(timestamp, _, _)) = self.samples.front() {
            if now.duration_since(timestamp) > self.sample_window || self.samples.len() > 128 {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Exponentially decaying inertial motion with exact time integration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialInertia {
    /// Current offset in pixels.
    pub position_x: f32,
    /// Current offset in pixels.
    pub position_y: f32,
    /// Current velocity in pixels per second.
    pub velocity_x: f32,
    /// Current velocity in pixels per second.
    pub velocity_y: f32,
    decay_rate: f32,
}

impl ExponentialInertia {
    /// Creates inertial motion with a positive decay rate in inverse seconds.
    pub fn new(
        position_x: f32,
        position_y: f32,
        velocity: ScrollVelocity,
        decay_rate: f32,
    ) -> Self {
        Self {
            position_x,
            position_y,
            velocity_x: velocity.x,
            velocity_y: velocity.y,
            decay_rate: decay_rate.max(0.0),
        }
    }

    /// Advances motion by `dt`, returning the integrated displacement.
    pub fn advance(&mut self, dt: Duration) -> ScrollVelocity {
        let seconds = dt.as_secs_f32().max(0.0);
        if seconds == 0.0 {
            return ScrollVelocity::default();
        }
        let displacement_factor = if self.decay_rate > f32::EPSILON {
            (-self.decay_rate * seconds).exp_m1().abs() / self.decay_rate
        } else {
            seconds
        };
        let dx = self.velocity_x * displacement_factor;
        let dy = self.velocity_y * displacement_factor;
        self.position_x += dx;
        self.position_y += dy;
        let decay = (-self.decay_rate * seconds).exp();
        self.velocity_x *= decay;
        self.velocity_y *= decay;
        ScrollVelocity::new(dx, dy)
    }

    /// Returns the current velocity.
    pub fn velocity(&self) -> ScrollVelocity {
        ScrollVelocity::new(self.velocity_x, self.velocity_y)
    }
}

#[cfg(test)]
mod velocity_tests {
    use super::*;
    #[test]
    fn equal_timestamp_samples_accumulate_and_idle_release_stops() {
        let now = Instant::now();
        let mut tracker =
            ScrollVelocityTracker::new(now, Duration::from_millis(90), Duration::from_millis(65));
        tracker.push_delta(now, 2.0, 0.0);
        tracker.push_delta(now + Duration::from_millis(10), 8.0, 0.0);
        assert!(
            (tracker.resolve(now + Duration::from_millis(10)).unwrap().x - 1000.0).abs() < 0.01
        );
        assert!(
            tracker
                .resolve(now + Duration::from_millis(80))
                .unwrap()
                .is_zero()
        );
    }
    #[test]
    fn constant_velocity_is_independent_of_sample_frequency() {
        let now = Instant::now();
        for interval in [5, 10, 20] {
            let mut tracker = ScrollVelocityTracker::new(
                now,
                Duration::from_millis(90),
                Duration::from_millis(65),
            );
            for elapsed in (interval..=80).step_by(interval as usize) {
                tracker.push_delta(now + Duration::from_millis(elapsed), interval as f32, 0.0);
            }
            assert!(
                (tracker.resolve(now + Duration::from_millis(80)).unwrap().x - 1000.0).abs() < 0.01
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{ExponentialInertia, ScrollVelocity};
    use std::time::Duration;

    #[test]
    fn exponential_decay_integral_is_frame_rate_independent() {
        let velocity = ScrollVelocity::new(600.0, 0.0);
        let mut sixty = ExponentialInertia::new(0.0, 0.0, velocity, 5.0);
        let mut one_twenty = ExponentialInertia::new(0.0, 0.0, velocity, 5.0);
        for _ in 0..60 {
            sixty.advance(Duration::from_secs_f32(1.0 / 60.0));
        }
        for _ in 0..120 {
            one_twenty.advance(Duration::from_secs_f32(1.0 / 120.0));
        }
        assert!((sixty.position_x - one_twenty.position_x).abs() < 0.01);
        assert!((sixty.velocity_x - one_twenty.velocity_x).abs() < 0.01);
    }
}
