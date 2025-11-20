use tessera_ui::{Color, DrawCommand};

/// Draw command for the simple rectangle pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleRectCommand {
    /// Fill color for the rectangle.
    pub color: Color,
}

impl DrawCommand for SimpleRectCommand {}
