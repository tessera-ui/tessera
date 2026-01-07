use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use tessera_ui::DrawCommand;

/// Image pixel data for rendering.
///
/// # Fields
///
/// - `data`: Raw pixel data (RGBA).
/// - `width`: Image width in pixels.
/// - `height`: Image height in pixels.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw RGBA pixel buffer.
    pub data: Arc<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

impl Hash for ImageData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ref().hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.data.as_ref() == other.data.as_ref()
    }
}

impl Eq for ImageData {}

/// Command for rendering an image in a UI component.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCommand {
    /// Shared image buffer used by the draw pass.
    pub data: Arc<ImageData>,
    /// Opacity multiplier applied to the sampled image.
    pub opacity: f32,
}

impl DrawCommand for ImageCommand {
    fn sample_region(&self) -> Option<tessera_ui::SampleRegion> {
        // This command does not require any specific barriers.
        None
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.opacity = (self.opacity * opacity).clamp(0.0, 1.0);
    }
}
