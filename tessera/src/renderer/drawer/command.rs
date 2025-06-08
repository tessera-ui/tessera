use super::{
    shape::{ShapeUniforms, Vertex as ShapeVertex},
    text::TextData,
};

/// Every draw command is a command that can be executed by the drawer.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a shape with a spec color at vertices
    Shape {
        /// positions of the vertices(x, y)
        vertices: Vec<ShapeVertex>,
        uniforms: ShapeUniforms,
    },
    Text {
        /// Text data to draw
        data: TextData,
    },
}

/// Describes size constraints for a text draw
#[derive(Debug, PartialEq)]
pub struct TextConstraint {
    /// Maximum width of the text
    /// If None, it will be calculated by the text renderer
    pub max_width: Option<f32>,
    /// Maximum height of the text
    /// If None, it will be calculated by the text renderer
    pub max_height: Option<f32>,
}

impl TextConstraint {
    /// Creates a new `TextConstraint` with no limits.
    pub const NONE: Self = Self {
        max_width: None,
        max_height: None,
    };
}
