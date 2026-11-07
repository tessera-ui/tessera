//! Reusable stencil masks for nested UI shape clips.
//!
//! ## Usage
//!
//! Apply rounded or elliptical clipping across scene draw passes.

use bytemuck::{Pod, Zeroable};

use crate::{
    ClipRegion, ClipShape, PxSize,
    clip::{finite_exponent, finite_radius},
    render_pass::{ClipOps, DrawOrClip, RenderPassPlan},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaskInstance {
    bounds: [f32; 4],
    radii: [f32; 4],
    exponents: [f32; 4],
    viewport: [f32; 2],
    ellipse: u32,
    padding: u32,
}

pub(super) struct ShapeClipper {
    push_pipeline: wgpu::RenderPipeline,
    pop_pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    capacity: usize,
    layout: wgpu::BindGroupLayout,
    group: wgpu::BindGroup,
    instances: Vec<MaskInstance>,
    stack: Vec<Option<u32>>,
}

impl ShapeClipper {
    pub(super) fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/clip.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Clip masks"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Clip mask layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |operation| {
            let mut state = crate::ui_stencil_state().expect("UI stencil state");
            state.stencil.front.pass_op = operation;
            state.stencil.back.pass_op = operation;
            state.stencil.write_mask = 0xff;
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Clip mask"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: Some(state),
                multisample: wgpu::MultisampleState {
                    count: samples,
                    ..Default::default()
                },
                multiview_mask: None,
                cache: None,
            })
        };
        let buffer = Self::buffer(device, 1);
        let group = Self::group(device, &layout, &buffer);
        Self {
            push_pipeline: pipeline(wgpu::StencilOperation::IncrementClamp),
            pop_pipeline: pipeline(wgpu::StencilOperation::DecrementClamp),
            buffer,
            capacity: 1,
            layout,
            group,
            instances: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clip mask instances"),
            size: (capacity * size_of::<MaskInstance>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Clip mask instances"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        passes: &mut [RenderPassPlan],
        viewport: [f32; 2],
    ) {
        self.stack.clear();
        self.instances.clear();
        for pass in passes {
            let RenderPassPlan::Draw { draws, .. } = pass else {
                continue;
            };
            for op in draws {
                if let DrawOrClip::Clip(ClipOps::ShapePush { region, index }) = op {
                    *index = self.instances.len() as u32;
                    self.instances.push(Self::instance(*region, viewport));
                }
            }
        }
        if self.instances.len() > self.capacity {
            self.capacity = self.instances.len().next_power_of_two();
            self.buffer = Self::buffer(device, self.capacity);
            self.group = Self::group(device, &self.layout, &self.buffer);
        }
        if !self.instances.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.instances));
        }
    }

    fn instance(region: ClipRegion, viewport: [f32; 2]) -> MaskInstance {
        let size = region.size;
        let (radii, exponents, ellipse) = match region.shape {
            ClipShape::RoundedRectangle { radii, exponents } => {
                let max = size[0].min(size[1]) * 0.5;
                (
                    radii.map(|radius| finite_radius(radius, max)),
                    exponents.map(finite_exponent),
                    0,
                )
            }
            ClipShape::Ellipse => ([0.0; 4], [2.0; 4], 1),
        };
        MaskInstance {
            bounds: [region.position.x, region.position.y, size[0], size[1]],
            radii,
            exponents,
            viewport,
            ellipse,
            padding: 0,
        }
    }

    /// Redraws the active masks into a pass whose stencil starts cleared.
    pub(super) fn restore(&self, pass: &mut wgpu::RenderPass<'_>, size: PxSize) {
        let mut depth = 0;
        for index in self.stack.iter().flatten() {
            self.draw_mask(pass, size, *index, depth, true);
            depth += 1;
        }
        pass.set_stencil_reference(depth);
    }

    pub(super) fn push(
        &mut self,
        pass: &mut wgpu::RenderPass<'_>,
        size: PxSize,
        index: Option<u32>,
    ) {
        let depth = self.depth();
        if let Some(index) = index {
            assert!(
                depth < 255,
                "UI shape clips cannot be nested more than 255 levels"
            );
            self.draw_mask(pass, size, index, depth, true);
        }
        self.stack.push(index);
        pass.set_stencil_reference(self.depth());
    }

    pub(super) fn pop(&mut self, pass: &mut wgpu::RenderPass<'_>, size: PxSize) {
        let depth = self.depth();
        if let Some(Some(index)) = self.stack.pop() {
            self.draw_mask(pass, size, index, depth, false);
        }
        pass.set_stencil_reference(self.depth());
    }

    fn depth(&self) -> u32 {
        self.stack.iter().flatten().count() as u32
    }

    fn draw_mask(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        size: PxSize,
        index: u32,
        depth: u32,
        push: bool,
    ) {
        pass.set_scissor_rect(0, 0, size.width.positive(), size.height.positive());
        pass.set_stencil_reference(depth);
        pass.set_pipeline(if push {
            &self.push_pipeline
        } else {
            &self.pop_pipeline
        });
        pass.set_bind_group(0, &self.group, &[]);
        pass.draw(0..6, index..index + 1);
    }
}
