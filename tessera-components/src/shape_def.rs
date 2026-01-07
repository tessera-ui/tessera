//! Defines the [`Shape`] enum and its variants, used for describing the
//! geometric form of UI components.
//!
//! This module provides a flexible way to define very basic components' shape,
//! including [`crate::surface::surface`] and
//! [`crate::fluid_glass::fluid_glass`].

use tessera_ui::{PxSize, dp::Dp};

/// Capsule shapes use a constant `g2_k_value` to maintain circular ends.
pub const CAPSULE_G2_K_VALUE: f32 = 2.0;

/// Corner definition: capsule or manual radius with per-corner G2.
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
}

impl RoundedCorner {
    /// A corner with zero radius.
    pub const ZERO: Self = RoundedCorner::Manual {
        radius: Dp(0.0),
        g2_k_value: 3.0,
    };

    /// Helper to create a manual corner.
    pub const fn manual(radius: Dp, g2_k_value: f32) -> Self {
        Self::Manual { radius, g2_k_value }
    }

    /// Resolves into `(radius_px, g2)` using the provided size.
    pub fn resolve(self, size: PxSize) -> (f32, f32) {
        match self {
            RoundedCorner::Capsule => (
                size.width.to_f32().min(size.height.to_f32()) / 2.0,
                CAPSULE_G2_K_VALUE,
            ),
            RoundedCorner::Manual { radius, g2_k_value } => (radius.to_pixels_f32(), g2_k_value),
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
///
/// `Shape` is used by multiple components (`surface`, `fluid_glass`, sliders,
/// progress, buttons) to define visual outline, hit-testing, and pipeline
/// behavior.
///
/// # Variants
/// * [`Shape::RoundedRectangle`] – Per-corner capsule or manual radius +
///   per-corner G2
/// * [`Shape::Ellipse`] – Ellipse filling the component bounds
///
/// # Example
///
/// ```
/// use tessera_components::shape_def::{RoundedCorner, Shape};
/// use tessera_ui::dp::Dp;
///
/// // Explicit rounded rectangle
/// let rr = Shape::RoundedRectangle {
///     top_left: RoundedCorner::manual(Dp(8.0), 3.0),
///     top_right: RoundedCorner::manual(Dp(8.0), 3.0),
///     bottom_right: RoundedCorner::manual(Dp(8.0), 3.0),
///     bottom_left: RoundedCorner::manual(Dp(8.0), 3.0),
/// };
///
/// // Ellipse
/// let ellipse = Shape::Ellipse;
///
/// // Mixed capsule/fixed corners (left side capsule, right side explicit)
/// let mixed = Shape::RoundedRectangle {
///     top_left: RoundedCorner::Capsule, // auto radius = min(width, height) / 2
///     top_right: RoundedCorner::manual(Dp(8.0), 3.0),
///     bottom_right: RoundedCorner::manual(Dp(8.0), 3.0),
///     bottom_left: RoundedCorner::Capsule, // also capsule
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Rounded rectangle with per-corner capsule or manual radius + G2.
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
    /// use tessera_components::shape_def::{RoundedCorner, Shape};
    /// use tessera_ui::dp::Dp;
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

    /// A helper to create a uniform capsule on all corners.
    pub const fn capsule() -> Self {
        Shape::RoundedRectangle {
            top_left: RoundedCorner::Capsule,
            top_right: RoundedCorner::Capsule,
            bottom_right: RoundedCorner::Capsule,
            bottom_left: RoundedCorner::Capsule,
        }
    }

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
