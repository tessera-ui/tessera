//! Shape transitions for selectable Material controls.
//!
//! ## Usage
//!
//! Preserve corner geometry and velocity while interaction or selection
//! changes.

use tessera_ui::Dp;

use crate::{
    animation::material_motion::{Scalar, Spec},
    shape_def::{RoundedCorner, Shape},
};

#[derive(Clone, Copy, PartialEq)]
struct CornerMotion {
    fraction: Scalar,
    radius: Scalar,
    curvature: Scalar,
}

impl CornerMotion {
    fn new(corner: RoundedCorner) -> Self {
        let (fraction, radius, curvature) = match corner {
            RoundedCorner::Capsule => (0.5, 0.0, 2.0),
            RoundedCorner::Manual { radius, g2_k_value } => (0.0, radius.0, g2_k_value),
            RoundedCorner::Relative {
                fraction,
                offset,
                g2_k_value,
            } => (f64::from(fraction), offset.0, g2_k_value),
        };
        Self {
            fraction: Scalar::new(fraction),
            radius: Scalar::new(radius),
            curvature: Scalar::new(f64::from(curvature)),
        }
    }

    fn retarget(&mut self, target: Self, frame: u64) {
        self.fraction
            .retarget(target.fraction.value, Spec::FastSpatial, frame);
        self.radius
            .retarget(target.radius.value, Spec::FastSpatial, frame);
        self.curvature
            .retarget(target.curvature.value, Spec::FastSpatial, frame);
    }

    fn advance(&mut self, frame: u64) {
        self.fraction.advance(frame);
        self.radius.advance(frame);
        self.curvature.advance(frame);
    }

    fn is_animating(self) -> bool {
        self.fraction.is_animating() || self.radius.is_animating() || self.curvature.is_animating()
    }

    fn corner(self) -> RoundedCorner {
        RoundedCorner::relative(
            self.fraction.value as f32,
            Dp(self.radius.value),
            self.curvature.value as f32,
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ShapeMotion {
    target: Shape,
    corners: Option<[CornerMotion; 4]>,
}

impl ShapeMotion {
    pub fn new(target: Shape) -> Self {
        let corners = match target {
            Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => Some([top_left, top_right, bottom_right, bottom_left].map(CornerMotion::new)),
            Shape::Ellipse => None,
        };
        Self { target, corners }
    }

    pub fn retarget(&mut self, target: Shape, animate: bool, frame: u64) {
        self.advance(frame);
        if !animate {
            *self = Self::new(target);
        } else if self.target != target {
            let next = Self::new(target);
            if let (Some(current), Some(next)) = (&mut self.corners, next.corners) {
                for (corner, target) in current.iter_mut().zip(next) {
                    corner.retarget(target, frame);
                }
                self.target = target;
            } else {
                // Ellipses cannot be represented as size-relative rounded
                // corners.
                *self = next;
            }
        }
    }

    pub fn advance(&mut self, frame: u64) {
        if let Some(corners) = &mut self.corners {
            for corner in corners {
                corner.advance(frame);
            }
        }
    }

    pub fn is_animating(self) -> bool {
        self.corners
            .is_some_and(|corners| corners.into_iter().any(CornerMotion::is_animating))
    }

    pub fn shape(self) -> Shape {
        if !self.is_animating() {
            return self.target;
        }
        self.corners.map_or(self.target, |corners| {
            let [top_left, top_right, bottom_right, bottom_left] =
                corners.map(CornerMotion::corner);
            Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectable_shape_motion_tracks_size_relative_corners_and_interrupted_velocity() {
        let rounded = |radius| {
            let corner = RoundedCorner::manual(Dp(radius), 2.0);
            Shape::RoundedRectangle {
                top_left: corner,
                top_right: corner,
                bottom_right: corner,
                bottom_left: corner,
            }
        };
        let mut motion = ShapeMotion::new(rounded(12.0));
        motion.retarget(Shape::CAPSULE, true, 0);
        motion.advance(75_000_000);
        let corner = match motion.shape() {
            Shape::RoundedRectangle { top_left, .. } => top_left,
            _ => unreachable!(),
        };
        let radius = corner
            .resolve(tessera_ui::PxSize::new(100.into(), 32.into()))
            .0;
        assert!((radius - 15.3076).abs() < 0.001);
        let before = motion.corners.expect("corners");
        motion.retarget(rounded(8.0), true, 75_000_000);
        assert_eq!(
            motion.corners.expect("corners")[0].fraction.velocity,
            before[0].fraction.velocity
        );
        assert_eq!(
            motion.shape(),
            Shape::RoundedRectangle {
                top_left: corner,
                top_right: corner,
                bottom_right: corner,
                bottom_left: corner,
            }
        );
        motion.advance(1_000_000_000);
        assert!(!motion.is_animating());
        motion.retarget(Shape::CAPSULE, false, 1_000_000_000);
        let corner = match motion.shape() {
            Shape::RoundedRectangle { top_left, .. } => top_left,
            _ => unreachable!(),
        };
        assert_eq!(
            corner
                .resolve(tessera_ui::PxSize::new(120.into(), 40.into()))
                .0,
            20.0
        );
        assert!(!motion.is_animating());
        motion.retarget(Shape::Ellipse, true, 1_000_000_000);
        assert_eq!(motion.shape(), Shape::Ellipse);
        let asymmetric = Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(40.0), 3.0),
            top_right: RoundedCorner::ZERO,
            bottom_right: RoundedCorner::ZERO,
            bottom_left: RoundedCorner::ZERO,
        };
        motion.retarget(asymmetric, false, 1_000_000_000);
        assert_eq!(motion.shape(), asymmetric);
    }
}
