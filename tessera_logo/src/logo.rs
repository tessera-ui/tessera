use bytemuck::{Pod, Zeroable};
use delaunator::{Point, triangulate};
use derive_builder::Builder;
use rand::{Rng, SeedableRng};
use tessera::{
    ComputedData, DrawCommand, DrawablePipeline, Px, PxPosition, PxSize,
    wgpu::{self, util::DeviceExt},
};
use tessera_macros::tessera;

#[derive(Clone, Debug)]
pub struct CrystalCommand {
    pub vertices: Vec<GpuVertex>,
    pub base_color: [f32; 4],
    pub seed: [f32; 2],
}

impl DrawCommand for CrystalCommand {
    fn barrier(&self) -> Option<tessera::BarrierRequirement> {
        // This command does not require a barrier
        None
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuVertex {
    position: [f32; 2],
    normal: [f32; 3],
}

impl GpuVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    color: [f32; 4],
    time: f32,
    _padding1: [u32; 1],
    size: [f32; 2],
    seed: [f32; 2],
    _padding2: [u32; 2],
}

#[derive(Debug, Default, Builder, Clone)]
#[builder(pattern = "owned")]
pub struct CrystalShardArgs {
    #[builder(default = "[0.2, 0.4, 0.8, 0.7]")]
    pub base_color: [f32; 4],
}

pub struct CrystalPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl CrystalPipeline {
    pub fn new(gpu: &wgpu::Device, config: &wgpu::SurfaceConfiguration, sample_count: u32) -> Self {
        let shader = gpu.create_shader_module(wgpu::include_wgsl!("../shaders/crystal.wgsl"));

        let bind_group_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("crystal_bind_group_layout"),
        });

        let pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crystal Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crystal Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[GpuVertex::desc()],
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
        }
    }
}

impl DrawablePipeline<CrystalCommand> for CrystalPipeline {
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass,
        command: &CrystalCommand,
        size: PxSize,
        start_pos: PxPosition,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        if command.vertices.is_empty() {
            return;
        }

        let screen_size = [config.width as f32, config.height as f32];
        let component_size = [size.width.to_f32(), size.height.to_f32()];

        let to_ndc = |pos: [f32; 2]| {
            let ndc_x = (start_pos.x.to_f32() + pos[0]) / screen_size[0] * 2.0 - 1.0;
            let ndc_y = -((start_pos.y.to_f32() + pos[1]) / screen_size[1] * 2.0 - 1.0);
            [ndc_x, ndc_y]
        };

        let gpu_vertices: Vec<GpuVertex> = command
            .vertices
            .iter()
            .map(|v| GpuVertex {
                position: to_ndc(v.position),
                normal: v.normal,
            })
            .collect();

        let vertex_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crystal Vertex Buffer"),
            contents: bytemuck::cast_slice(&gpu_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let uniforms = Uniforms {
            color: command.base_color,
            time: 0.0, // Time is no longer used for the logo animation
            size: component_size,
            seed: command.seed,
            _padding1: [0],
            _padding2: [0, 0],
        };
        let uniform_buffer = gpu.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crystal Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("crystal_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..command.vertices.len() as u32, 0..1);
    }
}

#[tessera]
pub fn crystal_shard(args: CrystalShardArgs) {
    let args = args.clone();
    measure(Box::new(move |input| {
        const NUM_POINTS: usize = 200;
        const RADIUS: f32 = 200.0;
        let center = [RADIUS, RADIUS];

        // Use a deterministic seed based on the word "tessera"
        let seed = "tessera".as_bytes().iter().map(|&b| b as u64).sum();
        let mut rng = rand_pcg::Pcg64::seed_from_u64(seed);
        let mut points = Vec::with_capacity(NUM_POINTS);

        for _ in 0..NUM_POINTS {
            let r = RADIUS * rng.random::<f32>().sqrt();
            let theta = rng.random::<f32>() * 2.0 * std::f32::consts::PI;
            points.push(Point {
                x: (center[0] + r * theta.cos()) as f64,
                y: (center[1] + r * theta.sin()) as f64,
            });
        }

        // Use 12 points to create a dodecagon shape instead of a circle
        for i in 0..12 {
            let theta = (i as f32 / 12.0) * 2.0 * std::f32::consts::PI;
            points.push(Point {
                x: (center[0] + RADIUS * theta.cos()) as f64,
                y: (center[1] + RADIUS * theta.sin()) as f64,
            });
        }

        let triangulation = triangulate(&points);
        let mut gpu_vertices: Vec<GpuVertex> = Vec::with_capacity(triangulation.triangles.len());

        for tri_indices in triangulation.triangles.chunks(3) {
            let p0 = &points[tri_indices[0]];
            let p1 = &points[tri_indices[1]];
            let p2 = &points[tri_indices[2]];

            let v0 = [p0.x as f32, p0.y as f32, 0.0];
            let v1 = [p1.x as f32, p1.y as f32, 0.0];
            let v2 = [p2.x as f32, p2.y as f32, 0.0];

            // Generate a random normal for each triangle to give a faceted, 3D look.
            let mut normal: [f32; 3] = [
                rng.random_range(-1.0..1.0),
                rng.random_range(-1.0..1.0),
                rng.random_range(0.5..1.0), // Bias towards pointing "out" of the screen
            ];
            let len = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
            if len > 0.0 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }

            gpu_vertices.push(GpuVertex {
                position: [v0[0], v0[1]],
                normal,
            });
            gpu_vertices.push(GpuVertex {
                position: [v1[0], v1[1]],
                normal,
            });
            gpu_vertices.push(GpuVertex {
                position: [v2[0], v2[1]],
                normal,
            });
        }

        let command = CrystalCommand {
            vertices: gpu_vertices,
            base_color: args.base_color,
            seed: [center[0], center[1]],
        };

        if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
            metadata.push_draw_command(command);
        }

        Ok(ComputedData {
            width: Px((RADIUS * 2.0) as i32),
            height: Px((RADIUS * 2.0) as i32),
        })
    }));
}
