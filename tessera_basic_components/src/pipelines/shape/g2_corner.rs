use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tessera::renderer::ComputablePipeline; // ComputeCommand is handled by blanket impl
use wgpu::util::DeviceExt;

use super::ShapeVertex;

/// Compute Command for generating vertices for a G2-smooth rounded rectangle.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct G2RoundedRectCommand {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub segments_per_corner: u32,
}

// Eq and Hash are manually implemented to allow this command to be used in a registry
// where commands might be cached or looked up.
impl Eq for G2RoundedRectCommand {}

impl PartialEq for G2RoundedRectCommand {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.corner_radius == other.corner_radius
            && self.segments_per_corner == other.segments_per_corner
    }
}

impl Hash for G2RoundedRectCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
        self.corner_radius.to_bits().hash(state);
        self.segments_per_corner.hash(state);
    }
}

// --- Pipeline Definition ---

pub struct G2RoundedRectPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    cache: HashMap<G2RoundedRectCommand, Arc<wgpu::Buffer>>,
}

impl G2RoundedRectPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("G2 Corner Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("g2_corner.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("G2 Corner Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("G2 Corner Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("G2 Corner Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            cache: HashMap::new(),
        }
    }
}

impl ComputablePipeline<G2RoundedRectCommand> for G2RoundedRectPipeline {
    fn dispatch_once(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command: &G2RoundedRectCommand,
    ) {
        if self.cache.contains_key(command) {
            return;
        }

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G2 Params Uniform Buffer"),
            contents: bytemuck::bytes_of(command),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let total_vertices = command.segments_per_corner as u64 * 4 * 3;
        let output_buffer_size = total_vertices * std::mem::size_of::<ShapeVertex>() as u64;

        if output_buffer_size == 0 {
            return; // Avoid creating a zero-sized buffer
        }

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("G2 Corner Vertex Output Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("G2 Corner Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("G2 Corner Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("G2 Corner Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            let workgroup_count = (command.segments_per_corner * 4 + 63) / 64;
            if workgroup_count > 0 {
                compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
            }
        }

        queue.submit(Some(encoder.finish()));
        self.cache.insert(*command, Arc::new(output_buffer));
    }

    fn get_result(
        &self,
        command: &G2RoundedRectCommand,
    ) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.cache
            .get(command)
            .map(|buffer| buffer.clone() as Arc<dyn std::any::Any + Send + Sync>)
    }
}
