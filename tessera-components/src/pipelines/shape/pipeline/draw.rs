use std::ops::Range;

use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{PxPosition, PxSize, wgpu};

use super::{
    super::command::{ShapeCommand, rect_to_uniforms},
    CachedInstanceBatch, ShapeInstances, ShapePipeline, ShapeUniforms,
};

const INSTANCES_PER_BUFFER: usize = 256;

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct CachedRectUniform {
    position: Vec4,
    screen_size: Vec2,
    padding: Vec2,
}

#[derive(PartialEq, ShaderType)]
struct CachedRectInstances {
    #[shader(size(runtime))]
    rects: Vec<CachedRectUniform>,
}

#[derive(Default)]
pub(super) struct BatchResources {
    buffers: Vec<BatchBuffer>,
    current: usize,
}

struct BatchBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: usize,
    bytes: Vec<u8>,
}

impl BatchResources {
    pub(super) fn begin_frame(&mut self) {
        self.current = 0;
        for buffer in &mut self.buffers {
            buffer.bytes.clear();
        }
    }

    fn append(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        instance_count: usize,
    ) -> (&wgpu::BindGroup, Range<u32>) {
        let stride = bytes.len() / instance_count;
        debug_assert!(stride > 0 && bytes.len().is_multiple_of(instance_count));
        if self.buffers.get(self.current).is_some_and(|buffer| {
            !buffer.bytes.is_empty()
                && buffer.bytes.len().saturating_add(bytes.len()) > buffer.capacity
        }) {
            self.current += 1;
        }
        let index = self.current;
        if self
            .buffers
            .get(index)
            .is_none_or(|buffer| buffer.capacity < bytes.len())
        {
            #[cfg(test)]
            super::BATCH_ALLOCATIONS.with(|count| count.set(count.get() + 1));
            let maximum_instances =
                device.limits().max_storage_buffer_binding_size as usize / stride;
            let capacity_instances = instance_count
                .max(INSTANCES_PER_BUFFER)
                .checked_next_power_of_two()
                .unwrap_or(maximum_instances)
                .min(maximum_instances);
            assert!(
                capacity_instances >= instance_count,
                "Shape instance batch exceeds the storage binding limit"
            );
            let capacity = capacity_instances * stride;
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shape Batch Storage"),
                size: capacity as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("Shape Batch Bind Group"),
            });
            let buffer = BatchBuffer {
                buffer,
                bind_group,
                capacity,
                bytes: Vec::with_capacity(capacity),
            };
            if index == self.buffers.len() {
                self.buffers.push(buffer);
            } else {
                self.buffers[index] = buffer;
            }
        }
        let buffer = &mut self.buffers[index];
        debug_assert!(buffer.bytes.len().is_multiple_of(stride));
        let start =
            u32::try_from(buffer.bytes.len() / stride).expect("Shape instance offset overflow");
        let end = start + u32::try_from(instance_count).expect("Shape instance count overflow");
        buffer.bytes.extend_from_slice(bytes);
        (&buffer.bind_group, start..end)
    }

    pub(super) fn finish_frame(&self, queue: &wgpu::Queue) -> (usize, usize) {
        let mut uploads = 0;
        let mut bytes = 0;
        // Uploads must precede submission while retaining each recorded draw's
        // range.
        for buffer in self
            .buffers
            .iter()
            .take(self.current + 1)
            .filter(|buffer| !buffer.bytes.is_empty())
        {
            queue.write_buffer(&buffer.buffer, 0, &buffer.bytes);
            #[cfg(test)]
            super::BATCH_UPLOADS.with(|count| count.set(count.get() + 1));
            uploads += 1;
            bytes += buffer.bytes.len();
        }
        (uploads, bytes)
    }
}

fn build_instances(
    commands: &[(&ShapeCommand, tessera_ui::DrawGeometry)],
    target_size: PxSize,
) -> Vec<ShapeUniforms> {
    commands
        .iter()
        .map(|(command, geometry)| {
            let mut uniforms =
                rect_to_uniforms(command, geometry.local_size, geometry.window_origin());
            let [width, height] = geometry.window_size();
            let origin = geometry.window_origin();
            let scale = geometry.local_to_window.scale();
            uniforms.position = Vec4::new(origin.x, origin.y, width, height);
            uniforms.corner_radii *= scale;
            uniforms.border_width *= scale;
            uniforms.screen_size = [target_size.width.to_f32(), target_size.height.to_f32()].into();
            uniforms
        })
        .collect()
}

impl ShapePipeline {
    pub(super) fn draw_uncached_batch(
        &mut self,
        gpu: &wgpu::Device,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&ShapeCommand, tessera_ui::DrawGeometry)],
        indices: &[usize],
    ) {
        if indices.is_empty() {
            return;
        }

        let subset: Vec<_> = indices.iter().map(|&i| commands[i]).collect();
        let instances = build_instances(&subset, target_size);
        if instances.is_empty() {
            return;
        }

        let uniforms = ShapeInstances { instances };

        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        let (bind_group, instances) = self.uncached_batches.append(
            gpu,
            &self.bind_group_layout,
            buffer_content.as_ref(),
            uniforms.instances.len(),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, instances);
    }

    pub(super) fn draw_cached_run(
        &mut self,
        gpu: &wgpu::Device,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        texture_bind_group: &wgpu::BindGroup,
        padding: Vec2,
        instances: &[(PxPosition, PxSize)],
    ) {
        if instances.is_empty() {
            return;
        }

        let rects: Vec<CachedRectUniform> = instances
            .iter()
            .map(|(position, size)| CachedRectUniform {
                position: Vec4::new(
                    position.x - padding.x,
                    position.y - padding.y,
                    size.width.raw() as f32 + padding.x * 2.0,
                    size.height.raw() as f32 + padding.y * 2.0,
                ),
                screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
                padding,
            })
            .collect();

        let rect_instances = CachedRectInstances { rects };

        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&rect_instances)
            .expect("buffer write failed");

        let (transform_bind_group, instance_range) = self.cached_batches.append(
            gpu,
            &self.cache_transform_bind_group_layout,
            buffer_content.as_ref(),
            instances.len(),
        );

        render_pass.set_pipeline(&self.cached_pipeline);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, texture_bind_group, &[]);
        render_pass.set_bind_group(1, transform_bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, instance_range);
    }

    pub(super) fn flush_cached_run(
        &mut self,
        gpu: &wgpu::Device,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        pending: &mut CachedInstanceBatch,
    ) {
        let Some((entry, instances)) = pending.take() else {
            return;
        };
        let texture_bind_group = entry.texture_bind_group.clone();
        let padding = entry.padding;
        self.draw_cached_run(
            gpu,
            target_size,
            render_pass,
            &texture_bind_group,
            padding,
            &instances,
        );
    }
}
