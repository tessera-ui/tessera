//! Synchronized color transitions for list row content.
//!
//! ## Usage
//!
//! Animate selection, enabled state, theme, and controlled drag colors.

use tessera_ui::{Color, time::Instant};

use crate::animation::{Motion, color_motion::ColorMotion, material_motion::Spec};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ListColorsMotion([ColorMotion; 6]);

impl ListColorsMotion {
    pub fn new(colors: [Color; 6]) -> Self {
        Self(colors.map(ColorMotion::new))
    }

    pub fn is_animating(self) -> bool {
        self.0.iter().any(ColorMotion::is_animating)
    }

    pub fn value(self) -> [Color; 6] {
        self.0.map(|motion| motion.value())
    }
}

impl Motion for ListColorsMotion {
    type Target = [Color; 6];

    fn is_settled_at(&self, colors: &[Color; 6]) -> bool {
        !self.is_animating() && self.value() == *colors
    }

    fn retarget(&mut self, colors: [Color; 6], now: Instant) {
        self.advance(now);
        for (motion, color) in self.0.iter_mut().zip(colors) {
            motion.retarget(color, Spec::Effects, now, false);
        }
    }

    fn advance(&mut self, now: Instant) -> bool {
        for motion in &mut self.0 {
            motion.advance(now);
        }
        self.is_animating()
    }
}
