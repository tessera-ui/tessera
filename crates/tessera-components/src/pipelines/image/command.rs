use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use tessera_ui::DrawCommand;

/// Image pixel data for rendering.
///
/// Prefer [`ImageData::from_rgba`] to construct image content. Cloning shares
/// pixel storage; independently allocated images compare by dimensions and pixels.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw RGBA pixel buffer.
    pub data: Arc<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    pub(super) content_hash: u64,
}

impl ImageData {
    /// Builds image data from an RGBA buffer.
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
        self.width == other.width
            && self.height == other.height
            && self.content_hash == other.content_hash
            && (Arc::ptr_eq(&self.data, &other.data) || self.data == other.data)
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
    fn apply_opacity(&mut self, opacity: f32) {
        self.opacity = (self.opacity * opacity).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ImageData;

    #[test]
    fn independently_allocated_images_compare_pixels_even_when_hashes_collide() {
        let first = ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1);
        let equal = ImageData::from_rgba(vec![255, 0, 0, 255], 1, 1);
        assert_eq!(first, equal);
        let mut collision = ImageData::from_rgba(vec![0, 0, 255, 255], 1, 1);
        collision.content_hash = first.content_hash;
        assert_ne!(first, collision);
    }

    #[test]
    fn shared_pixels_with_different_dimensions_are_distinct_images() {
        let pixels = Arc::new(vec![255, 0, 0, 255, 0, 0, 255, 255]);
        let horizontal = ImageData::from_rgba(Arc::clone(&pixels), 2, 1);
        let vertical = ImageData::from_rgba(pixels, 1, 2);
        assert_ne!(horizontal, vertical);
    }
}
