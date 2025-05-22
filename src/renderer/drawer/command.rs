/// Every draw command is a command that can be executed by the drawer.
pub enum DrawCommand {
    /// Draw a shape with a spec color at vertices
    Shape {
        /// positions of the vertices(x, y)
        vertices: Vec<ShapeVertex>,
    },
    Text {
        /// text to draw
        text: String,
        /// position of the text(pixel; x, y)
        position: [u32; 2],
        /// color of the text
        color: [f32; 3],
        /// font size of the text
        size: f32,
        /// line height of the text
        line_height: f32,
    },
}

pub struct ShapeVertex {
    /// Position of the vertex(pixlel; x, y)
    pub position: [u32; 2],
    /// Color of the vertex
    pub color: [f32; 3],
}
