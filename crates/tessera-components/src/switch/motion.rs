//! Switch thumb geometry transitions.
//!
//! ## Usage
//!
//! Preserve thumb motion through selection, pressing, and content changes.

use tessera_ui::time::Instant;

use crate::animation::{
    Motion,
    material_motion::{Scalar, Spec},
};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Geometry {
    pub offset: Scalar,
    pub diameter: Scalar,
}

impl Geometry {
    pub fn new(offset: f64, diameter: f64) -> Self {
        Self {
            offset: Scalar::new(offset),
            diameter: Scalar::new(diameter),
        }
    }

    pub fn is_animating(&self) -> bool {
        self.offset.is_animating() || self.diameter.is_animating()
    }
}

impl Motion for Geometry {
    type Target = (f64, f64);

    fn is_settled_at(&self, &(offset, diameter): &(f64, f64)) -> bool {
        !self.is_animating() && self.offset.value == offset && self.diameter.value == diameter
    }

    fn retarget(&mut self, (offset, diameter): (f64, f64), now: Instant) {
        self.offset.retarget(offset, Spec::FastSpatial, now);
        self.diameter.retarget(diameter, Spec::FastSpatial, now);
    }

    fn advance(&mut self, now: Instant) -> bool {
        self.offset.advance(now);
        self.diameter.advance(now);
        self.is_animating()
    }
}
