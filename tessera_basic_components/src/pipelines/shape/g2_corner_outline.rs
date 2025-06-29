use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tessera::renderer::{ComputablePipeline};
use wgpu::util::DeviceExt;

use super::ShapeVertex;

/// Compute Command for generating vertices for a G2-smooth rounded rectangle outline.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct G2RoundedOutlineRectCommand {
    pub width: f32,
    pub height: f32,
    pub corner_radius: f32,
    pub border_width: f32,
    pub segments_per_corner: u32,
}

impl Eq for G2RoundedOutlineRectCommand {}

impl PartialEq for G2RoundedOutlineRectCommand {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.corner_radius == other.corner_radius
            && self.border_width == other.border_width
            && self.segments_per_corner == other.segments_per_corner
    }
}

impl Hash for G2RoundedOutlineRectCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
        self.corner_radius.to_bits().hash(state);
        self.border_width.to_bits().hash(state);
        self.segments_per_corner.hash(state);
    }
}

// --- Pipeline Definition ---

pub struct G2RoundedOutlineRectPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    cache: HashMap<G2RoundedOutlineRectCommand, Arc<wgpu::Buffer>>,
}

impl G2RoundedOutlineRectPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("G2 Corner Outline Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("g2_corner_outline.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("G2 Corner Outline Bind Group Layout"),
            entries: &[
                // size: vec2<f32>
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
                // corner_radius: f32
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // border_width: f32
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // segments_per_corner: u32
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // output: array<Vertex>
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
            label: Some("G2 Corner Outline Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("G2 Corner Outline Compute Pipeline"),
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

impl ComputablePipeline<G2RoundedOutlineRectCommand> for G2RoundedOutlineRectPipeline {
    fn dispatch_once(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command: &G2RoundedOutlineRectCommand,
    ) {
        if self.cache.contains_key(command) {
            return;
        }

        // The shader expects uniforms separately, not in a single struct
        let size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G2 Outline Size Uniform"),
            contents: bytemuck::bytes_of(&[command.width, command.height]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G2 Outline Radius Uniform"),
            contents: bytemuck::bytes_of(&command.corner_radius),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let border_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G2 Outline Border Uniform"),
            contents: bytemuck::bytes_of(&command.border_width),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let segments_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("G2 Outline Segments Uniform"),
            contents: bytemuck::bytes_of(&command.segments_per_corner),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Each segment produces two vertices for the strip (outer and inner)
        let total_vertices = command.segments_per_corner as u64 * 4 * 2;
        let output_buffer_size = total_vertices * std::mem::size_of::<ShapeVertex>() as u64;

        if output_buffer_size == 0 {
            return;
        }

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("G2 Corner Outline Vertex Output Buffer"),
            size: output_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("G2 Corner Outline Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: size_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: radius_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: border_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: segments_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: output_buffer.as_entire_binding() },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("G2 Corner Outline Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("G2 Corner Outline Compute Pass"),
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

    fn get_result(&self, command: &G2RoundedOutlineRectCommand) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.cache
            .get(command)
            .map(|buffer| buffer.clone() as Arc<dyn std::any::Any + Send + Sync>)
    }
}