//! Switch thumb geometry transitions.
//!
//! ## Usage
//!
//! Preserve thumb motion through selection, pressing, and content changes.

use crate::animation::material_motion::{Scalar, Spec};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Geometry {
    pub offset: Scalar,
    pub diameter: Scalar,
    revision: u64,
}

impl Geometry {
    pub fn new(offset: f64, diameter: f64, revision: u64) -> Self {
        Self {
            offset: Scalar::new(offset),
            diameter: Scalar::new(diameter),
            revision,
        }
    }

    pub fn retarget(
        &mut self,
        offset: f64,
        diameter: f64,
        pressed: bool,
        revision: u64,
        frame: u64,
    ) {
        if pressed || self.revision != revision {
            *self = Self::new(offset, diameter, revision);
        } else {
            self.offset.retarget(offset, Spec::FastSpatial, frame);
            self.diameter.retarget(diameter, Spec::FastSpatial, frame);
        }
    }

    pub fn advance(&mut self, frame: u64) {
        self.offset.advance(frame);
        self.diameter.advance(frame);
    }

    pub fn is_animating(&self) -> bool {
        self.offset.is_animating() || self.diameter.is_animating()
    }
}
