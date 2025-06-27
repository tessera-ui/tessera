use tessera::{
    renderer::{DrawablePipeline, RenderRequirement},
    Px, PxPosition,
};

use crate::fluid_glass::FluidGlassCommand;

pub(crate) struct FluidGlassPipeline;

impl FluidGlassPipeline {
    pub fn new(_gpu: &wgpu::Device, _config: &wgpu::SurfaceConfiguration) -> Self {
        // TODO
        Self
    }
}

impl DrawablePipeline<FluidGlassCommand> for FluidGlassPipeline {
    fn draw(
        &mut self,
        _gpu: &wgpu::Device,
        _queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        _render_pass: &mut wgpu::RenderPass,
        _command: &FluidGlassCommand,
        _size: [Px; 2],
        _start_pos: PxPosition,
        _scene_texture_view: Option<&wgpu::TextureView>,
    ) {
        // TODO
    }
}