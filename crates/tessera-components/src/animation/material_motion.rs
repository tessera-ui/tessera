//! Analytic scalar motion for Material components.
//!
//! ## Usage
//!
//! Preserve motion timing and velocity across interrupted state changes.

use std::time::Duration;

use tessera_ui::time::Instant;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Spec {
    Spatial,
    FastSpatial,
    ButtonGroupPress,
    Effects,
    FastEffects,
    Removal,
}

impl Spec {
    fn spring(self) -> (f64, f64) {
        match self {
            Self::Spatial => (380.0, 0.8),
            Self::FastSpatial => (800.0, 0.6),
            Self::ButtonGroupPress => (1400.0, 0.9),
            Self::Effects => (1600.0, 1.0),
            Self::FastEffects => (3800.0, 1.0),
            Self::Removal => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Scalar {
    pub value: f64,
    pub velocity: f64,
    target: f64,
    origin: f64,
    origin_velocity: f64,
    start: Option<Instant>,
    sampled: Option<Instant>,
    pub(crate) duration: Duration,
    spec: Spec,
    active: bool,
    request: Option<Request>,
}

#[derive(Clone, Copy, PartialEq)]
struct Request {
    target: f64,
    spec: Spec,
    restart: bool,
}

impl Scalar {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            origin: value,
            origin_velocity: 0.0,
            start: None,
            sampled: None,
            duration: Duration::ZERO,
            spec: Spec::Spatial,
            active: false,
            request: None,
        }
    }

    pub fn snap(&mut self, value: f64) {
        *self = Self::new(value);
    }

    /// Retargets the motion at the next [`Self::advance`].
    ///
    /// Use it where no frame time is available, such as in input handlers.
    pub fn request_retarget(&mut self, target: f64, spec: Spec) {
        self.request = Some(Request {
            target,
            spec,
            restart: false,
        });
    }

    /// Starts toward `target` from the current value and velocity at the next
    /// [`Self::advance`].
    pub fn request_start(&mut self, target: f64, spec: Spec) {
        self.request = Some(Request {
            target,
            spec,
            restart: true,
        });
    }

    pub fn retarget(&mut self, target: f64, spec: Spec, now: Instant) {
        self.request = None;
        if self.target == target && (self.spec == spec || !self.active) {
            return;
        }
        self.start_to(target, spec, now);
    }

    pub(crate) fn start_to(&mut self, target: f64, spec: Spec, now: Instant) {
        self.request = None;
        self.advance(now);
        self.origin = self.value;
        self.origin_velocity = if spec == Spec::Removal {
            0.0
        } else {
            self.velocity
        };
        self.velocity = self.origin_velocity;
        self.target = target;
        self.spec = spec;
        let start = self.sampled.map_or(now, |sampled| now.max(sampled));
        self.start = Some(start);
        self.sampled = Some(start);
        self.duration = self.settling_duration();
        self.active = self.value != target || self.velocity != 0.0;
    }

    fn settling_duration(&self) -> Duration {
        if self.spec == Spec::Removal {
            return Duration::from_millis(100);
        }
        let displacement = self.origin - self.target;
        let (stiffness, damping) = self.spec.spring();
        let omega = stiffness.sqrt();
        let seconds = if damping != 1.0 {
            let decay = omega * damping;
            let frequency = omega * (1.0 - damping * damping).sqrt();
            let coefficient = (self.origin_velocity + decay * displacement) / frequency;
            ((displacement.hypot(coefficient) / 0.01).ln() / decay).max(0.0)
        } else {
            let coefficient = self.origin_velocity + omega * displacement;
            let value = |time: f64| (displacement + coefficient * time) * (-omega * time).exp();
            let peak = if coefficient != 0.0 {
                ((coefficient - omega * displacement) / (omega * coefficient)).max(0.0)
            } else {
                0.0
            };
            let mut left = if value(peak).abs() > 0.01 { peak } else { 0.0 };
            if value(left).abs() <= 0.01 {
                return Duration::ZERO;
            }
            let zero = -displacement / coefficient;
            let mut right = if left == 0.0 && zero > 0.0 {
                zero
            } else {
                left + 1.0 / omega
            };
            for _ in 0..64 {
                if value(right).abs() <= 0.01 {
                    break;
                }
                right = right * 2.0 + 1.0 / omega;
            }
            for _ in 0..48 {
                let middle = (left + right) * 0.5;
                if value(middle).abs() > 0.01 {
                    left = middle;
                } else {
                    right = middle;
                }
            }
            right
        };
        Duration::from_millis((seconds * 1000.0) as u64)
    }

    pub fn advance(&mut self, now: Instant) {
        if let Some(request) = self.request.take() {
            if request.restart {
                self.start_to(request.target, request.spec, now);
            } else {
                self.retarget(request.target, request.spec, now);
            }
        }
        if !self.active {
            return;
        }
        let (Some(start), Some(sampled)) = (self.start, self.sampled) else {
            return;
        };
        let sampled = now.max(sampled);
        self.sampled = Some(sampled);
        let elapsed = sampled.saturating_duration_since(start);
        if elapsed >= self.duration {
            self.value = self.target;
            self.velocity = 0.0;
            self.active = false;
            return;
        }
        if self.spec == Spec::Removal {
            return;
        }
        let (stiffness, damping) = self.spec.spring();
        let seconds = elapsed.as_secs_f64();
        let omega = stiffness.sqrt();
        let displacement = self.origin - self.target;
        let decay = omega * damping;
        let envelope = (-decay * seconds).exp();
        let (position, velocity) = if damping == 1.0 {
            let coefficient = self.origin_velocity + omega * displacement;
            let position = displacement + coefficient * seconds;
            (position, coefficient - omega * position)
        } else {
            let frequency = omega * (1.0 - damping * damping).sqrt();
            let coefficient = (self.origin_velocity + decay * displacement) / frequency;
            let (sin, cos) = (frequency * seconds).sin_cos();
            let position = displacement * cos + coefficient * sin;
            let velocity = frequency * (coefficient * cos - displacement * sin) - decay * position;
            (position, velocity)
        };
        self.value = self.target + envelope * position;
        self.velocity = envelope * velocity;
    }

    /// Returns whether the motion runs or has a pending request.
    pub fn is_animating(&self) -> bool {
        self.active || self.request.is_some()
    }
}
