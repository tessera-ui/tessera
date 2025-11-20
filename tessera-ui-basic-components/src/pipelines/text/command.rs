use tessera_ui::DrawCommand;

use super::pipeline::TextData;

/// Draw command carrying formatted text layout data.
#[derive(Debug, Clone, PartialEq)]
pub struct TextCommand {
    /// Glyph layout and styling for the text draw.
    pub data: TextData,
}

impl DrawCommand for TextCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // No specific barrier requirements for text commands
        None
    }
}

/// Describes size constraints for a text draw
#[derive(Debug, PartialEq, Clone)]
pub struct TextConstraint {
    /// Maximum width of the text
    /// If None, it will be calculated by the text renderer
    pub max_width: Option<f32>,
    /// Maximum height of the text
    /// If None, it will be calculated by the text renderer
    pub max_height: Option<f32>,
}

impl std::hash::Hash for TextConstraint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
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

impl TextConstraint {
    /// Creates a new `TextConstraint` with no limits.
    pub const NONE: Self = Self {
        max_width: None,
        max_height: None,
    };
}
