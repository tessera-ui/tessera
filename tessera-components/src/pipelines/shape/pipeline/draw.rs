use encase::{ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{PxPosition, PxSize, wgpu};

use super::{
    super::command::{ShapeCommand, rect_to_uniforms},
    CachedInstanceBatch, ShapeInstances, ShapePipeline, ShapeUniforms,
};

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
    slots: Vec<BatchSlot>,
    next: usize,
}

struct BatchSlot {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: u64,
}

impl BatchResources {
    pub(super) fn begin_frame(&mut self) {
        self.next = 0;
    }

    fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
    ) -> &wgpu::BindGroup {
        let index = self.next;
        self.next += 1;
        if self
            .slots
            .get(index)
            .is_none_or(|slot| slot.capacity < bytes.len() as u64)
        {
            #[cfg(test)]
            super::BATCH_ALLOCATIONS.with(|count| count.set(count.get() + 1));
            let capacity = (bytes.len() as u64).next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shape Batch Storage"),
                size: capacity,
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
            let slot = BatchSlot {
                buffer,
                bind_group,
                capacity,
            };
            if index == self.slots.len() {
                self.slots.push(slot);
            } else {
                self.slots[index] = slot;
            }
        }
        let slot = &self.slots[index];
        queue.write_buffer(&slot.buffer, 0, bytes);
        &slot.bind_group
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
        gpu_queue: &wgpu::Queue,
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
        let bind_group = self.uncached_batches.upload(
            gpu,
            gpu_queue,
            &self.bind_group_layout,
            buffer_content.as_ref(),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
    }

    pub(super) fn draw_cached_run(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
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

        let transform_bind_group = self.cached_batches.upload(
            gpu,
            gpu_queue,
            &self.cache_transform_bind_group_layout,
            buffer_content.as_ref(),
        );

        render_pass.set_pipeline(&self.cached_pipeline);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, texture_bind_group, &[]);
        render_pass.set_bind_group(1, transform_bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
    }

    pub(super) fn flush_cached_run(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
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
            gpu_queue,
            target_size,
            render_pass,
            &texture_bind_group,
            padding,
            &instances,
        );
    }
}
