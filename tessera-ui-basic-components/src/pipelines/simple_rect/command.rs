use tessera_ui::{Color, DrawCommand};

/// Draw command for the simple rectangle pipeline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleRectCommand {
    /// Fill color for the rectangle.
    pub color: Color,
}

impl DrawCommand for SimpleRectCommand {
    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
