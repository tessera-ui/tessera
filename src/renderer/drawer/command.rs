use super::text::TextData;

/// Every draw command is a command that can be executed by the drawer.
#[derive(Debug, PartialEq)]
pub enum DrawCommand {
    /// Draw a shape with a spec color at vertices
    Shape {
        /// positions of the vertices(x, y)
        vertices: Vec<ShapeVertex>,
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

/// A vertex of a shape
#[derive(Debug, PartialEq)]
pub struct ShapeVertex {
    /// Position of the vertex(pixlel; x, y)
    pub position: [u32; 2],
    /// Color of the vertex
    pub color: [f32; 3],
}
