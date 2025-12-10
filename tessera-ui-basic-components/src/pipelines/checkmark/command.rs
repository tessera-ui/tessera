use tessera_ui::{Color, DrawCommand};

/// Command for drawing an animated checkmark
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckmarkCommand {
    /// Color of the checkmark stroke
    pub color: Color,
    /// Width of the checkmark stroke in pixels
    pub stroke_width: f32,
    /// Animation progress from 0.0 (not drawn) to 1.0 (fully drawn)
    pub progress: f32,
    /// Padding around the checkmark within its bounds
    pub padding: [f32; 2], // [horizontal, vertical]
}

impl CheckmarkCommand {
    /// Create a new checkmark command with default values
    pub fn new() -> Self {
        Self {
            color: Color::new(0.0, 0.6, 0.0, 1.0), // Green checkmark
            stroke_width: 5.0,
            progress: 1.0, // Fully drawn by default
            padding: [2.0, 2.0],
        }
    }

    /// Set the color of the checkmark
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the stroke width of the checkmark
    pub fn with_stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set the animation progress (0.0 to 1.0)
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    /// Set the padding around the checkmark
    pub fn with_padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = [horizontal, vertical];
        self
    }
}

impl Default for CheckmarkCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawCommand for CheckmarkCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // No specific barrier requirements for checkmark commands
        None
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.color = self
            .color
            .with_alpha(self.color.a * opacity.clamp(0.0, 1.0));
    }
}
