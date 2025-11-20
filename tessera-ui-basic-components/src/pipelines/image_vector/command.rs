use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use tessera_ui::{Color, DrawCommand};

/// Vertex attributes for tessellated vector geometry.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVectorVertex {
    /// 2D position in normalized SVG viewport coordinates.
    pub position: [f32; 2],
    /// Premultiplied color for the vertex.
    pub color: Color,
}

/// Tessellated vector data ready for rendering.
#[derive(Debug, Clone)]
pub struct ImageVectorData {
    /// Width of the original SVG viewport.
    pub viewport_width: f32,
    /// Height of the original SVG viewport.
    pub viewport_height: f32,
    /// Vertex data (positions and colors).
    pub vertices: Arc<Vec<ImageVectorVertex>>,
    /// Triangle indices referencing `vertices`.
    pub indices: Arc<Vec<u32>>,
}

impl ImageVectorData {
    /// Creates vector data from raw vertices and indices.
    pub fn new(
        viewport_width: f32,
        viewport_height: f32,
        vertices: Arc<Vec<ImageVectorVertex>>,
        indices: Arc<Vec<u32>>,
    ) -> Self {
        Self {
            viewport_width,
            viewport_height,
            vertices,
            indices,
        }
    }
}

impl PartialEq for ImageVectorData {
    fn eq(&self, other: &Self) -> bool {
        self.viewport_width.to_bits() == other.viewport_width.to_bits()
            && self.viewport_height.to_bits() == other.viewport_height.to_bits()
            && Arc::ptr_eq(&self.vertices, &other.vertices)
            && Arc::ptr_eq(&self.indices, &other.indices)
    }
}

impl Eq for ImageVectorData {}

impl Hash for ImageVectorData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.viewport_width.to_bits());
        state.write_u32(self.viewport_height.to_bits());
        state.write_usize(Arc::as_ptr(&self.vertices) as usize);
        state.write_usize(Arc::as_ptr(&self.indices) as usize);
    }
}

/// Draw command for vector images.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageVectorCommand {
    /// Shared vector mesh data.
    pub data: Arc<ImageVectorData>,
    /// Tint color multiplied with the mesh.
    pub tint: Color,
}

impl DrawCommand for ImageVectorCommand {}
