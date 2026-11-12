use tessera_ui::{PxPosition, render::DrawCommand};

use super::pipeline::TextData;

/// Draw command carrying formatted text layout data.
#[derive(Debug, Clone, PartialEq)]
pub struct TextCommand {
    /// Glyph layout and styling for the text draw.
    pub data: TextData,
    /// Offset applied to the command relative to the component origin.
    pub offset: PxPosition,
}

impl DrawCommand for TextCommand {
    fn supports_partial_redraw(&self) -> bool {
        true
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.data.apply_opacity(opacity);
    }
}

/// Describes size constraints for a text draw
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TextConstraint {
    /// Maximum width of the text
    /// If None, it will be calculated by the text renderer
    pub max_width: Option<f32>,
    /// Maximum height of the text
    /// If None, it will be calculated by the text renderer
    pub max_height: Option<f32>,
    /// Keep text on one line and ellipsize horizontal overflow.
    pub single_line: bool,
}

impl std::hash::Hash for TextConstraint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.single_line.hash(state);
        if let Some(w) = self.max_width {
            w.to_bits().hash(state);
        } else {
            0u32.hash(state); // Hash a constant for None
        }
        if let Some(h) = self.max_height {
            h.to_bits().hash(state);
        } else {
            0u32.hash(state); // Hash a constant for None
        }
    }
}
