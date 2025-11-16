use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use tessera_ui::{Color, DrawCommand};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageVectorVertex {
    pub position: [f32; 2],
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct ImageVectorData {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub vertices: Arc<Vec<ImageVectorVertex>>,
    pub indices: Arc<Vec<u32>>,
}

impl ImageVectorData {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ImageVectorCommand {
    pub data: Arc<ImageVectorData>,
    pub tint: Color,
}

impl DrawCommand for ImageVectorCommand {}
