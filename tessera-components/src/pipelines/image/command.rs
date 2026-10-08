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
///
/// Prefer [`ImageData::from_rgba`] so the cached content hash stays consistent
/// with the pixel buffer. `Hash` only writes that hash plus dimensions; it does
/// not walk the pixels again.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw RGBA pixel buffer.
    pub data: Arc<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Content hash of `data`, computed once at construction.
    content_hash: u64,
}

impl ImageData {
    /// Builds image data from an RGBA buffer and caches a content hash.
    pub fn from_rgba(data: impl Into<Arc<Vec<u8>>>, width: u32, height: u32) -> Self {
        let data = data.into();
        let content_hash = content_hash(data.as_ref());
        Self {
            data,
            width,
            height,
            content_hash,
        }
    }
}

fn content_hash(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

impl Hash for ImageData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        if self.width != other.width
            || self.height != other.height
            || self.content_hash != other.content_hash
        {
            return false;
        }
        self.data.as_ref() == other.data.as_ref()
    }
}

impl Eq for ImageData {}

/// Command for rendering an image in a UI component.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageCommand {
    /// Image buffer used by the draw pass.
    pub data: ImageData,
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
