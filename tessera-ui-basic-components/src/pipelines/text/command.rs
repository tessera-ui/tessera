use tessera_ui::DrawCommand;

use super::TextData;

pub struct TextCommand {
    pub data: TextData,
}

impl DrawCommand for TextCommand {
    fn barrier(&self) -> Option<tessera_ui::BarrierRequirement> {
        // No specific barrier requirements for text commands
        None
    }
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
