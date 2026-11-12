//! Drag displacement and settling for sheets.
//!
//! ## Usage
//!
//! Return released sheets smoothly to their anchors at any display density.

use tessera_ui::{Dp, time::Instant};

use super::material_motion::{Scalar, Spec};

#[derive(Clone, PartialEq)]
pub(crate) struct SheetDrag {
    offset: Scalar,
    extent: f64,
}

impl SheetDrag {
    pub fn new() -> Self {
        Self {
            offset: Scalar::new(0.0),
            extent: 0.0,
        }
    }

    pub fn extent(&self) -> f64 {
        self.extent
    }

    pub fn set_extent(&mut self, extent: f64) {
        self.extent = extent.max(0.0);
        if self.offset.value > self.extent {
            self.offset.snap(self.extent);
        }
    }

    pub fn offset(&self) -> f32 {
        Dp(self.offset.value.max(0.0)).to_pixels_f32()
    }

    pub fn is_animating(&self) -> bool {
        self.offset.is_animating()
    }

    /// Moves to `now` and returns whether the offset is still settling.
    pub fn advance(&mut self, now: Instant) -> bool {
        self.offset.advance(now);
        self.offset.is_animating()
    }

    pub fn grab(&mut self) {
        self.offset.snap(self.offset.value.max(0.0));
    }

    pub fn apply_delta(&mut self, pixels: f32) -> f32 {
        let before = self.offset.value.max(0.0);
        let limit = if self.extent > 0.0 {
            self.extent
        } else {
            f64::INFINITY
        };
        let after = (before + Dp::from_pixels_f32(pixels).0).clamp(0.0, limit);
        self.offset.snap(after);
        Dp(after - before).to_pixels_f32()
    }

    pub fn settle(&mut self) {
        self.offset.request_retarget(0.0, Spec::Spatial);
    }

    /// Returns whether a release at `velocity` should dismiss the sheet and
    /// settles the offset from that velocity.
    pub fn release(&mut self, velocity: f32, distance: f64, speed: f64) -> bool {
        let velocity = Dp::from_pixels_f32(velocity).0;
        let close = if velocity.abs() >= speed {
            velocity > 0.0
        } else {
            self.offset.value >= distance
        };
        self.offset.velocity = velocity;
        self.offset.request_start(0.0, Spec::Spatial);
        close
    }
}
