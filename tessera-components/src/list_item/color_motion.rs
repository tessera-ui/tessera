//! Synchronized color transitions for list row content.
//!
//! ## Usage
//!
//! Animate selection, enabled state, theme, and controlled drag colors.

use tessera_ui::Color;

use crate::animation::{color_motion::ColorMotion, material_motion::Spec};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct ListColorsMotion([ColorMotion; 6]);

impl ListColorsMotion {
    pub fn new(colors: [Color; 6]) -> Self {
        Self(colors.map(ColorMotion::new))
    }

    pub fn retarget(&mut self, colors: [Color; 6], frame: u64) {
        self.advance(frame);
        for (motion, color) in self.0.iter_mut().zip(colors) {
            motion.retarget(color, Spec::Effects, frame, false);
        }
    }

    pub fn advance(&mut self, frame: u64) {
        for motion in &mut self.0 {
            motion.advance(frame);
        }
    }

    pub fn is_animating(self) -> bool {
        self.0.iter().any(ColorMotion::is_animating)
    }

    pub fn value(self) -> [Color; 6] {
        self.0.map(|motion| motion.value())
    }
}
