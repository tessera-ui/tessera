use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use tessera_ui::render::DrawCommand;

/// Image pixel data for rendering.
///
/// Prefer [`ImageData::from_rgba`] to construct image content. Cloning shares
/// pixel storage; independently allocated images compare by dimensions and pixels.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Premultiplied sRGB RGBA pixels.
    pub data: Arc<Vec<u8>>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    pub(super) content_hash: u64,
}

impl ImageData {
    /// Builds image data from straight-alpha sRGB RGBA pixels.
    pub fn from_rgba(data: impl Into<Arc<Vec<u8>>>, width: u32, height: u32) -> Self {
        let mut data: Arc<Vec<u8>> = data.into();
        if data.chunks_exact(4).any(|pixel| pixel[3] != 255) {
            premultiply(Arc::make_mut(&mut data).as_mut_slice());
        }
        let content_hash = content_hash(data.as_ref());
        Self {
            data,
            width,
            height,
            content_hash,
        }
    }
}

fn premultiply(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]) / 255.0;
        if alpha == 1.0 {
            continue;
        }
        for channel in &mut pixel[..3] {
            let srgb = f32::from(*channel) / 255.0;
            let linear = if srgb <= 0.04045 {
                srgb / 12.92
            } else {
                ((srgb + 0.055) / 1.055).powf(2.4)
            } * alpha;
            let encoded = if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            *channel = (encoded * 255.0).round() as u8;
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
    fn construction_premultiplies_in_linear_light_and_keeps_opaque_buffers() {
        let opaque = Arc::new(vec![128, 64, 192, 255]);
        assert!(Arc::ptr_eq(
            &ImageData::from_rgba(Arc::clone(&opaque), 1, 1).data,
            &opaque
        ));
        let translucent = ImageData::from_rgba(vec![255, 0, 0, 128, 255, 128, 64, 0], 2, 1);
        assert_eq!(*translucent.data, [188, 0, 0, 128, 0, 0, 0, 0]);
    }

    #[test]
    fn shared_pixels_with_different_dimensions_are_distinct_images() {
        let pixels = Arc::new(vec![255, 0, 0, 255, 0, 0, 255, 255]);
        let horizontal = ImageData::from_rgba(Arc::clone(&pixels), 2, 1);
        let vertical = ImageData::from_rgba(pixels, 1, 2);
        assert_ne!(horizontal, vertical);
    }
}
