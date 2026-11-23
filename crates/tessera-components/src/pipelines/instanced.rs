use std::{marker::PhantomData, ops::Range};

use bytemuck::Pod;
use tessera_ui::{
    render::{GraphicsDevice, TrackedBuffer},
    wgpu,
};

struct Page {
    buffer: TrackedBuffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
}

impl Page {
    fn new<T: Pod>(
        device: &GraphicsDevice,
        layout: &wgpu::BindGroupLayout,
        capacity: usize,
        tracked_label: &'static str,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer(
            tracked_label,
            &wgpu::BufferDescriptor {
                label: Some(label),
                size: (capacity * size_of::<T>()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Self {
            buffer,
            bind_group,
            capacity,
        }
    }
}

pub(super) struct InstancePages<T> {
    layout: wgpu::BindGroupLayout,
    pages: Vec<Page>,
    page: usize,
    offset: usize,
    pub(super) max_instances: usize,
    staging: Vec<T>,
    tracked_label: &'static str,
    label: &'static str,
}

impl<T: Pod> InstancePages<T> {
    pub(super) fn new(
        device: &GraphicsDevice,
        layout: wgpu::BindGroupLayout,
        tracked_label: &'static str,
        label: &'static str,
    ) -> Self {
        let limits = device.limits();
        let max_bytes = limits
            .max_buffer_size
            .min(limits.max_storage_buffer_binding_size);
        Self {
            layout,
            pages: Vec::new(),
            page: 0,
            offset: 0,
            max_instances: (max_bytes / size_of::<T>() as u64).min(u64::from(u32::MAX)) as usize,
            staging: Vec::new(),
            tracked_label,
            label,
        }
    }

    pub(super) fn begin_frame(&mut self) {
        self.page = 0;
        self.offset = 0;
    }

    fn reserve(&mut self, device: &GraphicsDevice, count: usize) {
        if self
            .pages
            .get(self.page)
            .is_some_and(|page| self.offset > 0 && self.offset + count > page.capacity)
        {
            self.page += 1;
            self.offset = 0;
        }
        if self
            .pages
            .get(self.page)
            .is_some_and(|page| page.capacity >= count)
        {
            return;
        }
        let capacity = count.max(1024).next_power_of_two().min(self.max_instances);
        let page = Page::new::<T>(
            device,
            &self.layout,
            capacity,
            self.tracked_label,
            self.label,
        );
        if self.page < self.pages.len() {
            self.pages[self.page] = page;
        } else {
            self.pages.push(page);
        }
    }

    pub(super) fn write(
        &mut self,
        device: &GraphicsDevice,
        instances: impl ExactSizeIterator<Item = T>,
    ) -> (&wgpu::BindGroup, Range<u32>) {
        let count = instances.len();
        self.reserve(device, count);
        self.staging.clear();
        self.staging.extend(instances);
        let page = &self.pages[self.page];
        // Each draw retains a distinct slice until the frame is submitted.
        device.write_buffer(
            &page.buffer,
            (self.offset * size_of::<T>()) as u64,
            bytemuck::cast_slice(&self.staging),
        );
        let range = self.offset as u32..(self.offset + count) as u32;
        self.offset += count;
        (&page.bind_group, range)
    }
}

pub(super) fn storage_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
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
    })
}

pub(super) fn storage_quad_pipeline(
    device: &GraphicsDevice,
    shader: wgpu::ShaderModuleDescriptor<'_>,
    format: wgpu::TextureFormat,
    sample_count: u32,
    cache: Option<&wgpu::PipelineCache>,
    label: &str,
) -> (wgpu::BindGroupLayout, wgpu::RenderPipeline) {
    let layout = storage_layout(device, label);
    let pipeline = QuadPipeline {
        label,
        layout_label: label,
        format,
        sample_count,
        cache,
    }
    .create(
        device,
        &[Some(&layout)],
        shader,
        Some(wgpu::Face::Back),
        &[],
    );
    (layout, pipeline)
}

pub(super) const QUAD_VERTICES: &[Option<wgpu::VertexBufferLayout<'_>>] =
    &[Some(wgpu::VertexBufferLayout {
        array_stride: size_of::<[f32; 2]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    })];

pub(super) struct QuadPipeline<'a> {
    pub label: &'a str,
    pub layout_label: &'a str,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub cache: Option<&'a wgpu::PipelineCache>,
}

impl QuadPipeline<'_> {
    pub(super) fn create(
        self,
        device: &wgpu::Device,
        bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
        shader: wgpu::ShaderModuleDescriptor<'_>,
        cull_mode: Option<wgpu::Face>,
        buffers: &[Option<wgpu::VertexBufferLayout<'_>>],
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(self.layout_label),
            bind_group_layouts,
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(self.label),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                ..Default::default()
            },
            depth_stencil: tessera_ui::render::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: self.sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache: self.cache,
        })
    }
}

pub(super) struct DrawBuffers<T> {
    layout: wgpu::BindGroupLayout,
    buffers: Vec<Page>,
    cursor: usize,
    tracked_label: &'static str,
    label: &'static str,
    marker: PhantomData<T>,
}

impl<T: Pod> DrawBuffers<T> {
    pub(super) fn new(
        layout: wgpu::BindGroupLayout,
        tracked_label: &'static str,
        label: &'static str,
    ) -> Self {
        Self {
            layout,
            buffers: Vec::new(),
            cursor: 0,
            tracked_label,
            label,
            marker: PhantomData,
        }
    }

    pub(super) fn begin_frame(&mut self) {
        self.cursor = 0;
    }

    pub(super) fn write(&mut self, device: &GraphicsDevice, instances: &[T]) -> &wgpu::BindGroup {
        let count = instances.len();
        if self
            .buffers
            .get(self.cursor)
            .is_none_or(|buffer| buffer.capacity < count)
        {
            let capacity = count.max(1).next_power_of_two();
            let slot = Page::new::<T>(
                device,
                &self.layout,
                capacity,
                self.tracked_label,
                self.label,
            );
            if self.cursor < self.buffers.len() {
                self.buffers[self.cursor] = slot;
            } else {
                self.buffers.push(slot);
            }
        }
        let slot = &self.buffers[self.cursor];
        // Each draw owns a slot so later uploads cannot change queued draws.
        device.write_buffer(&slot.buffer, 0, bytemuck::cast_slice(instances));
        self.cursor += 1;
        &slot.bind_group
    }
}
