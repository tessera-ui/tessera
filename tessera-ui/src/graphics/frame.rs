use std::num::NonZeroU64;

use rustc_hash::FxHashMap;

#[derive(Debug)]
struct Page {
    buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    capacity: u64,
    bytes: Vec<u8>,
    last_used: u64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum Binding {
    Buffer(wgpu::Buffer, u64, Option<NonZeroU64>),
    Texture(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct GroupKey {
    layout: wgpu::BindGroupLayout,
    entries: Vec<(u32, Binding)>,
}

#[derive(Debug, Default)]
pub(crate) struct FrameResources {
    frame: u64,
    pages: Vec<Page>,
    groups: FxHashMap<GroupKey, (wgpu::BindGroup, u64)>,
}

impl FrameResources {
    pub(crate) fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.pages
            .retain(|page| self.frame.wrapping_sub(page.last_used) <= 2);
        for page in &mut self.pages {
            page.bytes.clear();
        }
        self.groups
            .retain(|_, (_, frame)| self.frame.wrapping_sub(*frame) <= 2);
    }
}

/// A distinct buffer range for commands recorded in the current frame.
/// Its contents must not be used by a later frame.
pub struct FrameBuffer {
    buffer: wgpu::Buffer,
    offset: u64,
    size: NonZeroU64,
}

impl FrameBuffer {
    /// Binds this range without exposing other commands' data.
    pub fn binding(&self) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.buffer,
            offset: self.offset,
            size: Some(self.size),
        })
    }
}

impl super::GraphicsDevice {
    /// Stages uniform or storage bytes in a reusable frame buffer. Each call
    /// owns a separate aligned range; pages are uploaded once before submission.
    pub fn frame_buffer(&self, usage: wgpu::BufferUsages, bytes: &[u8]) -> FrameBuffer {
        assert!(
            usage == wgpu::BufferUsages::UNIFORM || usage == wgpu::BufferUsages::STORAGE,
            "frame buffers must be uniform or storage buffers"
        );
        let limits = self.device.limits();
        let (alignment, maximum) = if usage == wgpu::BufferUsages::UNIFORM {
            (
                limits.min_uniform_buffer_offset_alignment,
                limits.max_uniform_buffer_binding_size,
            )
        } else {
            (
                limits.min_storage_buffer_offset_alignment,
                limits.max_storage_buffer_binding_size,
            )
        };
        let size = bytes.len() as u64;
        assert!(
            size > 0 && size <= u64::from(maximum),
            "invalid frame buffer binding size"
        );
        let alignment = u64::from(alignment).max(4);
        let mut resources = self.frame_resources.lock();
        let index = resources.pages.iter().position(|page| {
            page.usage == usage
                && (page.bytes.len() as u64).next_multiple_of(alignment) + size <= page.capacity
        });
        let index = index.unwrap_or_else(|| {
            let capacity = size
                .max(64 << 10)
                .next_power_of_two()
                .min(limits.max_buffer_size);
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Frame Buffer Page"),
                size: capacity,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.work
                .lock()
                .groups
                .entry("frame_buffers")
                .or_default()
                .buffer_allocations += 1;
            let frame = resources.frame;
            resources.pages.push(Page {
                buffer,
                usage,
                capacity,
                bytes: Vec::new(),
                last_used: frame,
            });
            resources.pages.len() - 1
        });
        let frame = resources.frame;
        let page = &mut resources.pages[index];
        let offset = (page.bytes.len() as u64).next_multiple_of(alignment);
        page.bytes.resize(offset as usize, 0);
        page.bytes.extend_from_slice(bytes);
        page.last_used = frame;
        FrameBuffer {
            buffer: page.buffer.clone(),
            offset,
            size: NonZeroU64::new(size).expect("nonempty frame buffer"),
        }
    }

    pub(crate) fn upload_frame_buffers(&self) {
        let mut resources = self.frame_resources.lock();
        for page in resources
            .pages
            .iter_mut()
            .filter(|page| !page.bytes.is_empty())
        {
            // Binding sizes need not be aligned, but queue writes do.
            let padded = page.bytes.len().next_multiple_of(4);
            page.bytes.resize(padded, 0);
            self.queue.write_buffer(&page.buffer, 0, &page.bytes);
            let mut work = self.work.lock();
            let work = work.groups.entry("frame_buffers").or_default();
            work.buffer_uploads += 1;
            work.buffer_uploaded_bytes += padded as u64;
        }
    }

    /// Reuses bindings for buffer ranges, textures and samplers across frames.
    /// Array bindings are created directly rather than cached.
    pub fn cached_bind_group(&self, descriptor: &wgpu::BindGroupDescriptor<'_>) -> wgpu::BindGroup {
        let entries: Option<Vec<_>> = descriptor
            .entries
            .iter()
            .map(|entry| {
                let binding = match &entry.resource {
                    wgpu::BindingResource::Buffer(buffer) => {
                        Binding::Buffer(buffer.buffer.clone(), buffer.offset, buffer.size)
                    }
                    wgpu::BindingResource::TextureView(view) => Binding::Texture((*view).clone()),
                    wgpu::BindingResource::Sampler(sampler) => Binding::Sampler((*sampler).clone()),
                    _ => return None,
                };
                Some((entry.binding, binding))
            })
            .collect();
        let Some(mut entries) = entries else {
            return self.device.create_bind_group(descriptor);
        };
        entries.sort_unstable_by_key(|(binding, _)| *binding);
        let key = GroupKey {
            layout: descriptor.layout.clone(),
            entries,
        };
        let mut resources = self.frame_resources.lock();
        let frame = resources.frame;
        if let Some((group, used)) = resources.groups.get_mut(&key) {
            *used = frame;
            return group.clone();
        }
        const MAX_GROUPS: usize = 512;
        if resources.groups.len() >= MAX_GROUPS {
            let oldest = resources
                .groups
                .iter()
                .max_by_key(|(_, (_, used))| frame.wrapping_sub(*used))
                .map(|(key, _)| GroupKey {
                    layout: key.layout.clone(),
                    entries: key.entries.clone(),
                })
                .expect("nonempty binding cache");
            resources.groups.remove(&oldest);
        }
        let group = self.device.create_bind_group(descriptor);
        resources.groups.insert(key, (group.clone(), frame));
        group
    }
}

#[cfg(all(test, feature = "headless", not(target_family = "wasm")))]
mod tests {
    #[test]
    #[ignore = "requires a native GPU adapter"]
    fn queued_frame_ranges_preserve_distinct_values_and_reuse_bindings_next_frame() {
        let session = crate::RuntimeSession::new();
        let _scope = session.enter();
        let core = pollster::block_on(crate::renderer::core::RenderCore::new_offscreen(
            winit::dpi::PhysicalSize::new(1, 1),
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            crate::renderer::DEFAULT_MAX_NON_SAMPLER_BINDINGS,
        ));
        let graphics = core.resources().graphics;
        let device = graphics.device();
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                "\
                @group(0) @binding(0) var<uniform> a: u32;
                @group(0) @binding(1) var<uniform> b: u32;
                @group(0) @binding(2) var<uniform> c: u32;
                @group(0) @binding(3) var<storage, read_write> result: array<u32>;
                @compute @workgroup_size(1) fn main() { result[0] = a; result[1] = b; result[2] = c; }
            "
                .into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let layout = pipeline.get_bind_group_layout(0);
        let output = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 12,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 12,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut previous = None;
        for values in [[11_u32, 37, 91], [81, 5, 7]] {
            graphics.begin_frame();
            let a =
                graphics.frame_buffer(wgpu::BufferUsages::UNIFORM, bytemuck::bytes_of(&values[0]));
            let b =
                graphics.frame_buffer(wgpu::BufferUsages::UNIFORM, bytemuck::bytes_of(&values[1]));
            let mut padded = vec![0; 64 << 10];
            padded[..4].copy_from_slice(bytemuck::bytes_of(&values[2]));
            let c = graphics.frame_buffer(wgpu::BufferUsages::UNIFORM, &padded);
            let group = graphics.cached_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: a.binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: b.binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: c.binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: output.as_entire_binding(),
                    },
                ],
            });
            if let Some(previous) = previous {
                assert_eq!(group, previous, "stable ranges reuse their bindings");
            }
            previous = Some(group.clone());
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
            }
            encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, 12);
            graphics.upload_frame_buffers();
            graphics.queue().submit([encoder.finish()]);
            let (sender, receiver) = std::sync::mpsc::channel();
            readback
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    sender.send(result).unwrap();
                });
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            receiver.recv().unwrap().unwrap();
            assert_eq!(
                bytemuck::cast_slice::<u8, u32>(&readback.slice(..).get_mapped_range().unwrap()),
                values
            );
            readback.unmap();
        }
        let work = graphics.diagnostics().group("frame_buffers");
        assert_eq!(
            work.buffer_allocations, 2,
            "both frames reuse the pages on either side of a page boundary"
        );
        assert_eq!(work.buffer_uploads, 4, "each frame uploads each page once");
    }
}
