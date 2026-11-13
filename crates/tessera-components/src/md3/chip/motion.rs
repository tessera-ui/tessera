//! Chip shape and elevation transitions between interaction states.
//!
//! ## Usage
//!
//! Keep chip silhouettes and shadows continuous during state changes.

use std::time::Duration;

use tessera_foundation::modifier::InteractionState;
use tessera_ui::{Dp, time::Instant};

use crate::{animation::Motion, md3::progress::cubic_bezier_easing};

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Interaction {
    #[default]
    Idle,
    Hover,
    Focus,
    Press,
    Drag,
}

impl Interaction {
    pub fn from_state(state: InteractionState) -> Self {
        if state.is_dragged() {
            Self::Drag
        } else if state.is_pressed() {
            Self::Press
        } else if state.is_focused() {
            Self::Focus
        } else if state.is_hovered() {
            Self::Hover
        } else {
            Self::Idle
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Elevation {
    value: Dp,
    origin: Dp,
    target: Dp,
    start: Option<Instant>,
    duration: Duration,
    incoming: bool,
    interaction: Interaction,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ElevationTarget {
    pub elevation: Dp,
    pub interaction: Interaction,
    pub enabled: bool,
}

impl Elevation {
    pub fn new(value: Dp) -> Self {
        Self {
            value,
            origin: value,
            target: value,
            start: None,
            duration: Duration::ZERO,
            incoming: false,
            interaction: Interaction::Idle,
        }
    }

    pub fn value(self) -> Dp {
        self.value
    }

    pub fn is_animating(self) -> bool {
        self.value != self.target
    }
}

impl Motion for Elevation {
    type Target = ElevationTarget;

    fn is_settled_at(&self, target: &ElevationTarget) -> bool {
        !self.is_animating()
            && self.target == target.elevation
            && (self.interaction == target.interaction || !target.enabled)
    }

    fn retarget(&mut self, target: ElevationTarget, now: Instant) {
        self.advance(now);
        let ElevationTarget {
            elevation,
            interaction,
            enabled,
        } = target;
        if !enabled {
            *self = Self::new(elevation);
            return;
        }
        if elevation != self.target {
            self.origin = self.value;
            self.target = elevation;
            self.start = Some(now);
            self.incoming = interaction != Interaction::Idle;
            self.duration =
                Duration::from_millis(if self.incoming || self.interaction == Interaction::Hover {
                    120
                } else {
                    150
                });
            if interaction == Interaction::Idle && self.interaction == Interaction::Idle {
                self.value = elevation;
            }
        }
        self.interaction = interaction;
    }

    fn advance(&mut self, now: Instant) -> bool {
        let Some(start) = self.start.filter(|_| self.is_animating()) else {
            return false;
        };
        let elapsed = now.saturating_duration_since(start);
        if elapsed >= self.duration {
            self.value = self.target;
            return false;
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let eased = cubic_bezier_easing(t, 0.4, 0.0, if self.incoming { 0.2 } else { 0.6 }, 1.0);
        self.value = Dp(self.origin.0 + (self.target.0 - self.origin.0) * f64::from(eased));
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::animation::test_instant as at;

    use super::*;

    fn target(elevation: Dp, interaction: Interaction, enabled: bool) -> ElevationTarget {
        ElevationTarget {
            elevation,
            interaction,
            enabled,
        }
    }

    #[test]
    fn elevation_uses_reference_durations_and_preserves_interrupted_values() {
        let mut motion = Elevation::new(Dp(1.0));
        motion.retarget(target(Dp(3.0), Interaction::Hover, true), at(0));
        motion.advance(at(60_000_000));
        assert!((motion.value().0 - 2.55112).abs() < 0.001);
        let interrupted = motion.value();
        motion.retarget(target(Dp(1.0), Interaction::Idle, true), at(60_000_000));
        assert_eq!(motion.value(), interrupted);
        motion.advance(at(180_000_000));
        assert_eq!(motion.value(), Dp(1.0));
        assert!(!motion.is_animating());
        motion.retarget(target(Dp(8.0), Interaction::Drag, true), at(200_000_000));
        motion.advance(at(320_000_000));
        assert_eq!(motion.value(), Dp(8.0));
        motion.retarget(target(Dp(1.0), Interaction::Idle, true), at(320_000_000));
        motion.advance(at(440_000_000));
        assert!(motion.is_animating());
        motion.advance(at(470_000_000));
        assert_eq!(motion.value(), Dp(1.0));
        motion.retarget(target(Dp(3.0), Interaction::Hover, true), at(500_000_000));
        motion.advance(at(550_000_000));
        motion.retarget(target(Dp::ZERO, Interaction::Idle, false), at(550_000_000));
        assert_eq!(motion.value(), Dp::ZERO);
        assert!(!motion.is_animating());
    }
}
