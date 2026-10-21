//! Analytic scalar motion for Material components.
//!
//! ## Usage
//!
//! Preserve motion timing and velocity across interrupted state changes.

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Spec {
    Spatial,
    FastSpatial,
    Effects,
    FastEffects,
    Removal,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Scalar {
    pub value: f64,
    pub velocity: f64,
    target: f64,
    origin: f64,
    origin_velocity: f64,
    start: u64,
    sampled: u64,
    pub(crate) duration_millis: u64,
    spec: Spec,
    active: bool,
}

impl Scalar {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            origin: value,
            origin_velocity: 0.0,
            start: 0,
            sampled: 0,
            duration_millis: 0,
            spec: Spec::Spatial,
            active: false,
        }
    }

    pub fn snap(&mut self, value: f64) {
        *self = Self::new(value);
    }

    pub fn retarget(&mut self, target: f64, spec: Spec, frame: u64) {
        if self.target == target && (self.spec == spec || !self.active) {
            return;
        }
        self.start_to(target, spec, frame);
    }

    pub(crate) fn start_to(&mut self, target: f64, spec: Spec, frame: u64) {
        self.advance(frame);
        self.origin = self.value;
        self.origin_velocity = if spec == Spec::Removal {
            0.0
        } else {
            self.velocity
        };
        self.velocity = self.origin_velocity;
        self.target = target;
        self.spec = spec;
        self.start = frame.max(self.sampled);
        self.sampled = self.start;
        self.duration_millis = self.duration();
        self.active = self.value != target || self.velocity != 0.0;
    }

    fn duration(&self) -> u64 {
        if self.spec == Spec::Removal {
            return 100;
        }
        let displacement = self.origin - self.target;
        let omega = match self.spec {
            Spec::Spatial => 380.0_f64.sqrt(),
            Spec::FastSpatial => 800.0_f64.sqrt(),
            Spec::Effects => 40.0,
            Spec::FastEffects => 3800.0_f64.sqrt(),
            Spec::Removal => unreachable!(),
        };
        let seconds = if matches!(self.spec, Spec::Spatial | Spec::FastSpatial) {
            let damping: f64 = if self.spec == Spec::FastSpatial {
                0.6
            } else {
                0.8
            };
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
                return 0;
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
        (seconds * 1000.0) as u64
    }

    pub fn advance(&mut self, frame: u64) {
        if !self.active {
            return;
        }
        self.sampled = frame.max(self.sampled);
        let elapsed = self.sampled.saturating_sub(self.start);
        if elapsed / 1_000_000 >= self.duration_millis {
            self.value = self.target;
            self.velocity = 0.0;
            self.active = false;
            return;
        }
        if self.spec == Spec::Removal {
            return;
        }
        let (stiffness, damping): (f64, f64) = match self.spec {
            Spec::Spatial => (380.0, 0.8),
            Spec::FastSpatial => (800.0, 0.6),
            Spec::Effects => (1600.0, 1.0),
            Spec::FastEffects => (3800.0, 1.0),
            Spec::Removal => unreachable!(),
        };
        let seconds = (elapsed / 1_000_000) as f64 / 1000.0;
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

    pub fn is_animating(&self) -> bool {
        self.active
    }
}
