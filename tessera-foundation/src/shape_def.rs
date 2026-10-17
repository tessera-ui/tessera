//! Shape definitions shared by foundational visual APIs.
//!
//! ## Usage
//!
//! Describe rounded rectangles and ellipses for surfaces, outlines, and
//! effects.

use tessera_ui::{PxSize, dp::Dp};

/// Capsule shapes use a constant `g2_k_value` to maintain circular ends.
pub const CAPSULE_G2_K_VALUE: f32 = 2.0;

/// A fixed or size-relative corner radius with per-corner curvature.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RoundedCorner {
    /// Capsule radius derived from `min(width, height) / 2.0`, with
    /// `CAPSULE_G2_K_VALUE`.
    Capsule,
    /// Manual radius (in `Dp`) with per-corner G2.
    Manual {
        /// Corner radius in device-independent pixels.
        radius: Dp,
        /// Corner G2 value (2.0 yields circular curvature).
        g2_k_value: f32,
    },
    /// A fraction of the shortest edge plus a fixed offset, clamped to fit.
    Relative {
        /// Fraction of the shortest measured edge.
        fraction: f32,
        /// Additional radius in device-independent pixels.
        offset: Dp,
        /// Corner G2 value (2.0 yields circular curvature).
        g2_k_value: f32,
    },
}

impl RoundedCorner {
    /// A corner with zero radius.
    pub const ZERO: Self = RoundedCorner::Manual {
        radius: Dp(0.0),
        g2_k_value: 3.0,
    };

    /// Creates a rounded corner with the specified radius and a default G2
    /// k value.
    pub const fn new(radius: Dp) -> Self {
        RoundedCorner::Manual {
            radius,
            g2_k_value: 3.0,
        }
    }

    /// Helper to create a manual corner.
    pub const fn manual(radius: Dp, g2_k_value: f32) -> Self {
        Self::Manual { radius, g2_k_value }
    }

    /// Creates a corner that responds to the actual measured size.
    ///
    /// The final radius is limited to half the shortest edge. Non-finite
    /// fractions and offsets contribute zero to the radius.
    pub const fn relative(fraction: f32, offset: Dp, g2_k_value: f32) -> Self {
        Self::Relative {
            fraction,
            offset,
            g2_k_value,
        }
    }

    /// Resolves into `(radius_px, g2)` using the provided size.
    pub fn resolve(self, size: PxSize) -> (f32, f32) {
        match self {
            RoundedCorner::Capsule => (
                size.width.to_f32().min(size.height.to_f32()) / 2.0,
                CAPSULE_G2_K_VALUE,
            ),
            RoundedCorner::Manual { radius, g2_k_value } => (radius.to_pixels_f32(), g2_k_value),
            RoundedCorner::Relative {
                fraction,
                offset,
                g2_k_value,
            } => {
                let edge = size.width.to_f32().min(size.height.to_f32()).max(0.0);
                let fraction = if fraction.is_finite() { fraction } else { 0.0 };
                let offset = offset.to_pixels_f32();
                let offset = if offset.is_finite() { offset } else { 0.0 };
                (
                    (edge * fraction + offset).clamp(0.0, edge / 2.0),
                    g2_k_value,
                )
            }
        }
    }
}

/// Resolved representation of a shape for rendering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedShape {
    /// Rounded rect with resolved radii and G2 values.
    Rounded {
        /// Pixel radii for each corner.
        corner_radii: [f32; 4],
        /// G2 parameters per corner.
        corner_g2: [f32; 4],
    },
    /// Ellipse occupies the full bounds.
    Ellipse,
}

/// Shape definitions for UI components.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Rounded rectangle with independent fixed or size-relative corners.
    RoundedRectangle {
        /// Top-left corner definition.
        top_left: RoundedCorner,
        /// Top-right corner definition.
        top_right: RoundedCorner,
        /// Bottom-right corner definition.
        bottom_right: RoundedCorner,
        /// Bottom-left corner definition.
        bottom_left: RoundedCorner,
    },
    /// Ellipse fitting the component bounds.
    Ellipse,
}

impl Default for Shape {
    /// Returns the default shape, which is a rectangle with zero corner radius.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_foundation::shape_def::{RoundedCorner, Shape};
    /// use tessera_ui::dp::Dp;
    ///
    /// let default_shape = Shape::default();
    /// assert_eq!(
    ///     default_shape,
    ///     Shape::RoundedRectangle {
    ///         top_left: RoundedCorner::manual(Dp(0.0), 3.0),
    ///         top_right: RoundedCorner::manual(Dp(0.0), 3.0),
    ///         bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
    ///         bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
    ///     }
    /// );
    /// ```
    fn default() -> Self {
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(Dp(0.0), 3.0),
            top_right: RoundedCorner::manual(Dp(0.0), 3.0),
            bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
            bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
        }
    }
}

impl Shape {
    /// A pure rectangle shape with no rounded corners.
    pub const RECTANGLE: Self = Shape::RoundedRectangle {
        top_left: RoundedCorner::manual(Dp(0.0), 3.0),
        top_right: RoundedCorner::manual(Dp(0.0), 3.0),
        bottom_right: RoundedCorner::manual(Dp(0.0), 3.0),
        bottom_left: RoundedCorner::manual(Dp(0.0), 3.0),
    };

    /// A helper to create a uniform rounded rectangle shape with manual
    /// corners.
    pub const fn rounded_rectangle(radius: Dp) -> Self {
        Shape::RoundedRectangle {
            top_left: RoundedCorner::manual(radius, 3.0),
            top_right: RoundedCorner::manual(radius, 3.0),
            bottom_right: RoundedCorner::manual(radius, 3.0),
            bottom_left: RoundedCorner::manual(radius, 3.0),
        }
    }

    /// A uniform capsule on all corners.
    pub const CAPSULE: Self = Shape::RoundedRectangle {
        top_left: RoundedCorner::Capsule,
        top_right: RoundedCorner::Capsule,
        bottom_right: RoundedCorner::Capsule,
        bottom_left: RoundedCorner::Capsule,
    };

    /// Resolves a shape into pixel radii and per-corner G2 parameters for a
    /// given size.
    pub fn resolve_for_size(self, size: PxSize) -> ResolvedShape {
        match self {
            Shape::RoundedRectangle {
                top_left,
                top_right,
                bottom_right,
                bottom_left,
            } => {
                let (tl_r, tl_g2) = top_left.resolve(size);
                let (tr_r, tr_g2) = top_right.resolve(size);
                let (br_r, br_g2) = bottom_right.resolve(size);
                let (bl_r, bl_g2) = bottom_left.resolve(size);

                ResolvedShape::Rounded {
                    corner_radii: [tl_r, tr_r, br_r, bl_r],
                    corner_g2: [tl_g2, tr_g2, br_g2, bl_g2],
                }
            }
            Shape::Ellipse => ResolvedShape::Ellipse,
        }
    }
}

#[cfg(test)]
mod tests {
    use tessera_ui::Px;

    use super::*;

    #[test]
    fn relative_corners_follow_measured_bounds_and_clamp_after_interpolation() {
        for (width, height) in [(260, 32), (260, 136), (20, 200), (0, 0)] {
            let size = PxSize::new(Px(width), Px(height));
            let half = width.min(height) as f32 / 2.0;
            assert_eq!(
                RoundedCorner::relative(0.5, Dp(0.0), 2.0).resolve(size),
                (half, 2.0)
            );
            let fixed = Dp(12.0).to_pixels_f32();
            assert_eq!(
                RoundedCorner::relative(0.0, Dp(12.0), 2.0).resolve(size).0,
                fixed.min(half)
            );
            let interpolated = RoundedCorner::relative(0.25, Dp(6.0), 2.0).resolve(size).0;
            assert_eq!(
                interpolated,
                (half / 2.0 + Dp(6.0).to_pixels_f32()).min(half)
            );
            assert_eq!(
                RoundedCorner::relative(-0.5, Dp(-12.0), 2.0)
                    .resolve(size)
                    .0,
                0.0
            );
            assert_eq!(
                RoundedCorner::relative(1.0, Dp(12.0), 2.0).resolve(size).0,
                half
            );
        }
        let size = PxSize::new(Px(100), Px(100));
        assert_eq!(
            RoundedCorner::relative(f32::NAN, Dp(10.0), 2.0)
                .resolve(size)
                .0,
            Dp(10.0).to_pixels_f32().min(50.0)
        );
        assert_eq!(
            RoundedCorner::relative(0.5, Dp(f64::INFINITY), 2.0)
                .resolve(size)
                .0,
            50.0
        );
    }
}
