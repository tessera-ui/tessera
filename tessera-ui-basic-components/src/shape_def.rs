//! Defines the basic shape types used by components.

/// Defines the shape of a UI component for rendering and hit-testing.
///
/// This enum is used by components to specify their geometric outline,
/// which affects both their visual appearance and interaction area.
///
/// # Variants
///
/// - [`Shape::RoundedRectangle`]: A rectangle with configurable corner radius and curvature.
/// - [`Shape::Ellipse`]: An ellipse that fills the component's bounds.
///
/// # Example
/// ```
/// use tessera_ui_basic_components::shape_def::Shape;
///
/// // Create a rounded rectangle shape with a 6.0 radius
/// let shape = Shape::RoundedRectangle { corner_radius: 6.0, g2_k_value: 3.0 };
///
/// // Use an ellipse shape
/// let ellipse = Shape::Ellipse;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A rectangle with configurable rounded corners.
    ///
    /// - `corner_radius`: The radius of the corners in logical pixels (Dp).
    /// - `g2_k_value`: Controls the curvature of the corner (higher values produce squarer corners).
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let shape = Shape::RoundedRectangle { corner_radius: 8.0, g2_k_value: 3.0 };
    /// ```
    RoundedRectangle { corner_radius: f32, g2_k_value: f32 },

    /// An ellipse that fills the component's bounding rectangle.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let shape = Shape::Ellipse;
    /// ```
    Ellipse,
}

impl Default for Shape {
    /// Returns the default shape, which is a rectangle with zero corner radius.
    ///
    /// # Example
    /// ```
    /// use tessera_ui_basic_components::shape_def::Shape;
    /// let default_shape = Shape::default();
    /// assert_eq!(default_shape, Shape::RoundedRectangle { corner_radius: 0.0, g2_k_value: 3.0 });
    /// ```
    fn default() -> Self {
        Shape::RoundedRectangle {
            corner_radius: 0.0,
            g2_k_value: 3.0,
        }
    }
}
