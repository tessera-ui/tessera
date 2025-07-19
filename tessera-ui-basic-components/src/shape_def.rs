//! Defines the basic shape types used by components.

/// An enum to explicitly define the shape of a component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// A rectangle with rounded corners.
    RoundedRectangle { corner_radius: f32, g2_k_value: f32 },
    /// An ellipse that fills the component's bounds.
    Ellipse,
}

impl Default for Shape {
    fn default() -> Self {
        Shape::RoundedRectangle {
            corner_radius: 0.0,
            g2_k_value: 3.0,
        }
    }
}
