//! Chip shape and elevation transitions between interaction states.
//!
//! ## Usage
//!
//! Keep chip silhouettes and shadows continuous during state changes.

use tessera_ui::Dp;

use crate::{modifier::InteractionState, progress::cubic_bezier_easing};

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
    start: u64,
    duration: u64,
    incoming: bool,
    interaction: Interaction,
}

impl Elevation {
    pub fn new(value: Dp) -> Self {
        Self {
            value,
            origin: value,
            target: value,
            start: 0,
            duration: 0,
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

    pub fn update(&mut self, target: Dp, interaction: Interaction, enabled: bool, now: u64) {
        self.advance(now);
        if !enabled {
            *self = Self::new(target);
            return;
        }
        if target != self.target {
            self.origin = self.value;
            self.target = target;
            self.start = now;
            self.incoming = interaction != Interaction::Idle;
            self.duration = if self.incoming || self.interaction == Interaction::Hover {
                120_000_000
            } else {
                150_000_000
            };
            if interaction == Interaction::Idle && self.interaction == Interaction::Idle {
                self.value = target;
            }
        }
        self.interaction = interaction;
    }

    pub fn advance(&mut self, now: u64) {
        if !self.is_animating() {
            return;
        }
        let elapsed = now.saturating_sub(self.start);
        if elapsed >= self.duration {
            self.value = self.target;
            return;
        }
        let t = elapsed as f32 / self.duration as f32;
        let eased = cubic_bezier_easing(t, 0.4, 0.0, if self.incoming { 0.2 } else { 0.6 }, 1.0);
        self.value = Dp(self.origin.0 + (self.target.0 - self.origin.0) * f64::from(eased));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_uses_reference_durations_and_preserves_interrupted_values() {
        let mut motion = Elevation::new(Dp(1.0));
        motion.update(Dp(3.0), Interaction::Hover, true, 0);
        motion.advance(60_000_000);
        assert!((motion.value().0 - 2.55112).abs() < 0.001);
        let interrupted = motion.value();
        motion.update(Dp(1.0), Interaction::Idle, true, 60_000_000);
        assert_eq!(motion.value(), interrupted);
        motion.advance(180_000_000);
        assert_eq!(motion.value(), Dp(1.0));
        assert!(!motion.is_animating());
        motion.update(Dp(8.0), Interaction::Drag, true, 200_000_000);
        motion.advance(320_000_000);
        assert_eq!(motion.value(), Dp(8.0));
        motion.update(Dp(1.0), Interaction::Idle, true, 320_000_000);
        motion.advance(440_000_000);
        assert!(motion.is_animating());
        motion.advance(470_000_000);
        assert_eq!(motion.value(), Dp(1.0));
        motion.update(Dp(3.0), Interaction::Hover, true, 500_000_000);
        motion.advance(550_000_000);
        motion.update(Dp::ZERO, Interaction::Idle, false, 550_000_000);
        assert_eq!(motion.value(), Dp::ZERO);
        assert!(!motion.is_animating());
    }
}
