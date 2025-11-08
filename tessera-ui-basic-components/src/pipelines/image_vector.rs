use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use encase::{ShaderType, UniformBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{
    Color, DrawCommand, PxPosition, PxSize,
    px::PxRect,
    renderer::drawer::DrawablePipeline,
    wgpu::{self, util::DeviceExt},
};

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

struct ImageVectorResources {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    index_count: u32,
}

#[derive(ShaderType, Clone, Copy)]
struct ImageVectorUniforms {
    origin: Vec2,
    scale: Vec2,
    tint: Vec4,
}

pub struct ImageVectorPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    resources: HashMap<ImageVectorData, ImageVectorResources>,
}

impl ImageVectorPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Vector Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image_vector/image_vector.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(ImageVectorUniforms::min_size()),
                },
                count: None,
            }],
            label: Some("image_vector_bind_group_layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("image_vector_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ImageVectorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Vector Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            resources: HashMap::new(),
        }
    }

    fn get_or_create_resources(
        &mut self,
        device: &wgpu::Device,
        data: &Arc<ImageVectorData>,
    ) -> &mut ImageVectorResources {
        self.resources
            .entry((**data).clone())
            .or_insert_with(|| Self::create_resources(device, data, &self.bind_group_layout))
    }

    fn create_resources(
        device: &wgpu::Device,
        data: &Arc<ImageVectorData>,
        layout: &wgpu::BindGroupLayout,
    ) -> ImageVectorResources {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_vector_vertex_buffer"),
            contents: bytemuck::cast_slice(data.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_vector_index_buffer"),
            contents: bytemuck::cast_slice(data.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("image_vector_uniform_buffer"),
            size: ImageVectorUniforms::min_size().get(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("image_vector_bind_group"),
        });

        ImageVectorResources {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            index_count: data.indices.len() as u32,
        }
    }

    fn compute_uniforms(
        start_pos: PxPosition,
        size: PxSize,
        tint: Color,
        config: &wgpu::SurfaceConfiguration,
    ) -> ImageVectorUniforms {
        let left = (start_pos.x.0 as f32 / config.width as f32) * 2.0 - 1.0;
        let right = ((start_pos.x.0 + size.width.0) as f32 / config.width as f32) * 2.0 - 1.0;
        let top = 1.0 - (start_pos.y.0 as f32 / config.height as f32) * 2.0;
        let bottom = 1.0 - ((start_pos.y.0 + size.height.0) as f32 / config.height as f32) * 2.0;

        ImageVectorUniforms {
            origin: Vec2::new(left, top),
            scale: Vec2::new(right - left, bottom - top),
            tint: Vec4::new(tint.r, tint.g, tint.b, tint.a),
        }
    }
}

impl DrawablePipeline<ImageVectorCommand> for ImageVectorPipeline {
    fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&ImageVectorCommand, PxSize, PxPosition)],
        _scene_texture_view: &wgpu::TextureView,
        _clip_rect: Option<PxRect>,
    ) {
        if commands.is_empty() {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);

        for (command, size, start_pos) in commands {
            let resources = self.get_or_create_resources(device, &command.data);

            let uniforms = Self::compute_uniforms(*start_pos, *size, command.tint, config);
            let mut buffer = UniformBuffer::new(Vec::new());
            buffer
                .write(&uniforms)
                .expect("uniform serialization failed");
            queue.write_buffer(&resources.uniform_buffer, 0, &buffer.into_inner());

            render_pass.set_bind_group(0, &resources.bind_group, &[]);
            render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(resources.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..resources.index_count, 0, 0..1);
        }
    }
}
