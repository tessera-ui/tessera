use bytemuck::{Pod, Zeroable};
use earcutr::earcut;
use wgpu::{include_wgsl, util::DeviceExt};

/// Vertex for any shapes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    /// Position of the vertex(x, y, z)
    position: [f32; 3],
    /// Color of the vertex
    color: [f32; 3],
}

impl Vertex {
    /// Describe the vertex attributes
    /// 0: position (x, y, z)
    /// 1: color (r, g, b)
    /// The vertex attribute array is used to describe the vertex buffer layout
    const ATTR: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    /// Create a new vertex
    fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        let x = pos[0];
        let y = pos[1];
        Self {
            position: [x, y, 0.0],
            color,
        }
    }

    /// Describe the vertex buffer layout
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTR,
        }
    }
}

pub struct ShapePipeline {
    /// Shape pipeline
    pipeline: wgpu::RenderPipeline,
    /// Shape pipeline layout
    pipeline_layout: wgpu::PipelineLayout,
}

impl ShapePipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = gpu.create_shader_module(include_wgsl!("shaders/shape.wgsl"));
        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            pipeline_layout,
        }
    }

    /// Draw the shape
    pub fn draw(
        &self,
        gpu: &wgpu::Device,
        render_pass: &mut wgpu::RenderPass<'_>,
        vertices: Vec<[f32; 2]>,
        colors: Vec<[f32; 3]>,
    ) {
        // Flatten 2D vertex array for earcutr
        let flat: Vec<f64> = vertices
            .iter()
            .flat_map(|[x, y]| vec![*x as f64, *y as f64])
            .collect();

        // Triangulate using earcutr
        let indices = earcut(&flat, &[], 2).unwrap(); // no holes, 2D
        let vertex_data: Vec<Vertex> = indices
            .iter()
            .map(|&i| Vertex::new(vertices[i], colors[i]))
            .collect();

        let vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangulated Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertex_data.len() as u32, 0..1);
    }
}
