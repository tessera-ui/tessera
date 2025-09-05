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
/// use tessera_ui_basic_components::shape_def::Shape;
///
/// // Explicit rounded rectangle
/// let rr = Shape::RoundedRectangle {
///     top_left: 8.0,
///     top_right: 8.0,
///     bottom_right: 8.0,
///     bottom_left: 8.0,
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
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
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
    /// ```
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let default_shape = Shape::default();
    /// assert_eq!(default_shape, Shape::RoundedRectangle { top_left: 0.0, top_right: 0.0, bottom_right: 0.0, bottom_left: 0.0, g2_k_value: 3.0 });
    /// ```
    fn default() -> Self {
        Shape::RoundedRectangle {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
            g2_k_value: 3.0,
        }
    }
}
