//! Defines the [`Shape`] enum and its variants, used for describing the geometric form of UI components.
//!
//! This module provides a flexible way to define very basic components' shape, including
//! [`crate::surface::surface`], [`crate::fluid_glass::fluid_glass`].

use tessera_ui::dp::Dp;

/// Shape definitions for UI components
///
/// `Shape` is used by multiple components (`surface`, `fluid_glass`, sliders, progress, buttons)
/// to define:
///
/// * Visual outline (fill / border / highlight pipelines)
/// * Interaction & ripple hit-testing region
/// * Automatic corner radius derivation for capsule variants
///
/// # Variants
/// * [`Shape::RoundedRectangle`] – Independent corner radii + `g2_k_value` curvature control
/// * [`Shape::Ellipse`] – Ellipse filling the component bounds
/// * [`Shape::HorizontalCapsule`] – Pill where corner radius = height / 2 (resolved at render)
/// * [`Shape::VerticalCapsule`] – Pill where corner radius = width / 2 (resolved at render)
///
/// Capsule variants are convenience markers; they are internally converted into a rounded rectangle
/// whose four radii equal half of the minor axis (height for horizontal, width for vertical).
///
/// # Example
///
/// ```
/// use tessera_ui::dp::Dp;
/// use tessera_ui_basic_components::shape_def::Shape;
///
/// // Explicit rounded rectangle
/// let rr = Shape::RoundedRectangle {
///     top_left: Dp(8.0),
///     top_right: Dp(8.0),
///     bottom_right: Dp(8.0),
///     bottom_left: Dp(8.0),
///     g2_k_value: 3.0,
/// };
///
/// // Ellipse
/// let ellipse = Shape::Ellipse;
///
/// // Capsules (auto radius from minor axis)
/// let h_capsule = Shape::HorizontalCapsule;
/// let v_capsule = Shape::VerticalCapsule;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// Rounded rectangle with independent corner radii and a curvature factor:
    /// * `g2_k_value` controls the transition curve (G2 continuity parameter).
    RoundedRectangle {
        top_left: Dp,
        top_right: Dp,
        bottom_right: Dp,
        bottom_left: Dp,
        g2_k_value: f32,
    },
    /// Ellipse fitting the component bounds.
    Ellipse,
    /// Horizontal capsule (pill) – rendered as a rounded rectangle whose corner radius
    /// is computed as `height / 2.0` at draw time.
    HorizontalCapsule,
    /// Vertical capsule (pill) – rendered as a rounded rectangle whose corner radius
    /// is computed as `width / 2.0` at draw time.
    VerticalCapsule,
}

impl Default for Shape {
    /// Returns the default shape, which is a rectangle with zero corner radius.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui::dp::Dp;
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let default_shape = Shape::default();
    /// assert_eq!(default_shape, Shape::RoundedRectangle { top_left: Dp(0.0), top_right: Dp(0.0), bottom_right: Dp(0.0), bottom_left: Dp(0.0), g2_k_value: 3.0 });
    /// ```
    fn default() -> Self {
        Shape::RoundedRectangle {
            top_left: Dp(0.0),
            top_right: Dp(0.0),
            bottom_right: Dp(0.0),
            bottom_left: Dp(0.0),
            g2_k_value: 3.0,
        }
    }
}

impl Shape {
    /// A pure rectangle shape with no rounded corners.
    pub const RECTANGLE: Self = Shape::RoundedRectangle {
        top_left: Dp(0.0),
        top_right: Dp(0.0),
        bottom_right: Dp(0.0),
        bottom_left: Dp(0.0),
        g2_k_value: 3.0,
    };

    /// A Quick helper to create a uniform rounded rectangle shape.
    ///
    /// # Example
    ///
    /// ```
    /// use tessera_ui::dp::Dp;
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let shape = Shape::rounded_rectangle(Dp(8.0));
    /// assert_eq!(shape, Shape::RoundedRectangle { top_left: Dp(8.0), top_right: Dp(8.0), bottom_right: Dp(8.0), bottom_left: Dp(8.0), g2_k_value: 3.0 });
    /// ```
    pub const fn rounded_rectangle(radius: Dp) -> Self {
        Shape::RoundedRectangle {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
            g2_k_value: 3.0,
        }
    }
}
