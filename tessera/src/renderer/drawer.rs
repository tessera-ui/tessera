mod command;
mod pos_misc;
mod shape;
mod text;

pub use crate::renderer::drawer::shape::{
    MAX_CONCURRENT_SHAPES, ShapeUniforms, Vertex as ShapeVertex,
};
pub use command::{DrawCommand, TextConstraint};
use pos_misc::pixel_to_ndc;
pub use text::{GlyphonTextRender, TextData};

/// Drawer is a struct that handles pipelines and draw commands.
pub struct Drawer {
    /// Shape pipeline
    shape_pipeline: shape::ShapePipeline,
    /// Text renderer
    pub text_renderer: GlyphonTextRender,
    /// Aligned size of ShapeUniforms
    shape_uniform_alignment: u32,
    /// Current offset in the dynamic uniform buffer for shapes
    current_shape_uniform_offset: u32,
    /// Max offset for the shape uniform buffer
    max_shape_uniform_buffer_offset: u32,
}

impl Drawer {
    /// Create a new drawer
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let shape_pipeline = shape::ShapePipeline::new(gpu, config);
        let text_renderer = GlyphonTextRender::new(gpu, queue, config);

        let size_of_shape_uniforms = std::mem::size_of::<ShapeUniforms>() as u32;
        let alignment = gpu.limits().min_uniform_buffer_offset_alignment;
        let shape_uniform_alignment =
            wgpu::util::align_to(size_of_shape_uniforms, alignment) as u32;
        // let shape_uniform_alignment = (size_of_shape_uniforms + alignment - 1) / alignment * alignment;

        let max_shape_uniform_buffer_offset =
            (MAX_CONCURRENT_SHAPES as u32 - 1) * shape_uniform_alignment;

        Self {
            shape_pipeline,
            text_renderer,
            shape_uniform_alignment,
            current_shape_uniform_offset: 0,
            max_shape_uniform_buffer_offset,
        }
    }

    /// Call this at the beginning of each frame to reset frame-specific states.
    pub fn begin_frame(&mut self) {
        self.current_shape_uniform_offset = 0;
    }

    /// Draw/Prepare Draw the command
    /// Some commands must be prepared before drawing
    /// and some commands can be drawn directly
    /// so we need to call both [Self::prepare_or_draw] and [Self::final_draw]
    pub fn prepare_or_draw(
        &mut self,
        gpu: &wgpu::Device, // Keep gpu for shape_pipeline.draw if it needs it for vertex buffers
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: DrawCommand,
    ) {
        match command {
            DrawCommand::Shape { vertices, uniforms } => {
                let positions: Vec<[f32; 2]> = vertices
                    .iter()
                    .map(|v| {
                        let pos = [v.position[0], v.position[1]];
                        pixel_to_ndc(pos, [config.width, config.height])
                    })
                    .collect();
                let colors: Vec<[f32; 3]> = vertices.iter().map(|v| v.color).collect();
                let local_positions: Vec<[f32; 2]> = vertices.iter().map(|v| v.local_pos).collect();

                // Two-pass drawing:
                // Pass 1: Draw Shadow (if shadow is enabled in uniforms)
                let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.shadow_params[2] > 0.0;

                if has_shadow {
                    let dynamic_offset = self.current_shape_uniform_offset;
                    if dynamic_offset > self.max_shape_uniform_buffer_offset {
                        // Handle buffer overflow, e.g., by logging or panicking
                        // For now, let's panic as this indicates too many shapes for the buffer.
                        // A more robust solution might involve resizing the buffer or limiting shapes.
                        panic!(
                            "Shape uniform buffer overflow: offset {} > max {}",
                            dynamic_offset, self.max_shape_uniform_buffer_offset
                        );
                    }

                    let mut uniforms_for_shadow = uniforms;
                    uniforms_for_shadow.size_cr_is_shadow[3] = 1.0; // Set is_shadow = true

                    self.shape_pipeline.draw(
                        gpu, // Pass gpu if shape_pipeline.draw needs it
                        queue,
                        render_pass,
                        &positions,
                        &colors,
                        &local_positions,
                        &uniforms_for_shadow,
                        dynamic_offset,
                    );
                    self.current_shape_uniform_offset += self.shape_uniform_alignment;
                }

                // Pass 2: Draw Object
                let dynamic_offset = self.current_shape_uniform_offset;
                if dynamic_offset > self.max_shape_uniform_buffer_offset {
                    panic!(
                        "Shape uniform buffer overflow: offset {} > max {}",
                        dynamic_offset, self.max_shape_uniform_buffer_offset
                    );
                }
                let mut uniforms_for_object = uniforms;
                uniforms_for_object.size_cr_is_shadow[3] = 0.0; // Set is_shadow = false

                self.shape_pipeline.draw(
                    gpu, // Pass gpu if shape_pipeline.draw needs it
                    queue,
                    render_pass,
                    &positions,
                    &colors,
                    &local_positions,
                    &uniforms_for_object,
                    dynamic_offset,
                );
                self.current_shape_uniform_offset += self.shape_uniform_alignment;
            }
            DrawCommand::Text { data } => {
                self.text_renderer.push(data);
            }
        }
    }

    /// Do the actual drawing for drawers that need to be prepared before drawing
    /// This should be called after all [Self::prepare_or_draw] calls
    /// this should only called once per render
    pub fn final_draw(
        &mut self,
        gpu: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        self.text_renderer.draw(gpu, config, queue, render_pass);
    }
}
