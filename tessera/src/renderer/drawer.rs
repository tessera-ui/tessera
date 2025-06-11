mod command;
mod pos_misc;
mod shape;
mod text;

pub use crate::renderer::drawer::shape::{
    MAX_CONCURRENT_SHAPES, ShapeUniforms, ShapeVertexData, Vertex as ShapeVertex,
};
pub use command::{DrawCommand, TextConstraint};
use pos_misc::pixel_to_ndc;
pub use text::{GlyphonTextRender, TextData, read_font_system, write_font_system};

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
        gpu: &wgpu::Device,
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

                // Check if shadow needs to be drawn
                // shadow_color[3] is alpha, render_params[2] is shadow_smoothness
                let has_shadow = uniforms.shadow_color[3] > 0.0 && uniforms.render_params[2] > 0.0;

                if has_shadow {
                    let dynamic_offset = self.current_shape_uniform_offset;
                    if dynamic_offset > self.max_shape_uniform_buffer_offset {
                        panic!(
                            "Shape uniform buffer overflow for shadow: offset {} > max {}",
                            dynamic_offset, self.max_shape_uniform_buffer_offset
                        );
                    }

                    let mut uniforms_for_shadow = uniforms;
                    // Set render_mode to 2.0 for shadow
                    uniforms_for_shadow.render_params[3] = 2.0;

                    let vertex_data_for_shadow = ShapeVertexData {
                        polygon_vertices: &positions,
                        vertex_colors: &colors,
                        vertex_local_pos: &local_positions,
                    };

                    self.shape_pipeline.draw(
                        gpu,
                        queue,
                        render_pass,
                        &vertex_data_for_shadow,
                        &uniforms_for_shadow,
                        dynamic_offset,
                    );
                    self.current_shape_uniform_offset += self.shape_uniform_alignment;
                }

                // Draw Object (Fill or Outline)
                // The original 'uniforms' should have render_params[3] set to 0.0 (fill) or 1.0 (outline)
                // and size_cr_border_width[3] to the border_width by the caller.
                let dynamic_offset = self.current_shape_uniform_offset;
                if dynamic_offset > self.max_shape_uniform_buffer_offset {
                    panic!(
                        "Shape uniform buffer overflow for object: offset {} > max {}",
                        dynamic_offset, self.max_shape_uniform_buffer_offset
                    );
                }

                let vertex_data_for_object = ShapeVertexData {
                    polygon_vertices: &positions,
                    vertex_colors: &colors,
                    vertex_local_pos: &local_positions,
                };

                self.shape_pipeline.draw(
                    gpu,
                    queue,
                    render_pass,
                    &vertex_data_for_object,
                    &uniforms, // Use original uniforms for the object
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
