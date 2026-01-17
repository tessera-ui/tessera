use std::sync::Arc;

use encase::{ShaderSize, ShaderType, StorageBuffer};
use glam::{Vec2, Vec4};
use tessera_ui::{PxPosition, PxSize, wgpu};

use super::{
    super::command::{ShapeCommand, rect_to_uniforms},
    CachedInstanceBatch, ShapeCacheEntry, ShapeInstances, ShapePipeline, ShapeUniforms,
};

#[repr(C)]
#[derive(ShaderType, Clone, Copy, Debug, PartialEq)]
struct CachedRectUniform {
    position: Vec4,
    screen_size: Vec2,
    padding: Vec2,
}

#[derive(ShaderType)]
struct CachedRectInstances {
    #[shader(size(runtime))]
    rects: Vec<CachedRectUniform>,
}

fn build_instances(
    commands: &[(&ShapeCommand, PxSize, PxPosition)],
    target_size: PxSize,
) -> Vec<ShapeUniforms> {
    commands
        .iter()
        .flat_map(|(command, size, start_pos)| {
            let mut uniforms = rect_to_uniforms(command, *size, *start_pos);
            uniforms.screen_size = [target_size.width.to_f32(), target_size.height.to_f32()].into();
            vec![uniforms]
        })
        .collect()
}

impl ShapePipeline {
    pub(super) fn draw_uncached_batch(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        commands: &[(&ShapeCommand, PxSize, PxPosition)],
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

        let storage_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Storage Buffer"),
            size: 16 + ShapeUniforms::SHADER_SIZE.get() * instances.len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = ShapeInstances { instances };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&uniforms)
            .expect("buffer write failed");
        gpu_queue.write_buffer(&storage_buffer, 0, buffer_content.as_ref());

        let bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            }],
            label: Some("shape_bind_group"),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
    }

    pub(super) fn draw_cached_run(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        target_size: PxSize,
        render_pass: &mut wgpu::RenderPass<'_>,
        entry: Arc<ShapeCacheEntry>,
        instances: &[(PxPosition, PxSize)],
    ) {
        if instances.is_empty() {
            return;
        }

        let rects: Vec<CachedRectUniform> = instances
            .iter()
            .map(|(position, size)| CachedRectUniform {
                position: Vec4::new(
                    position.x.raw() as f32 - entry.padding.x,
                    position.y.raw() as f32 - entry.padding.y,
                    size.width.raw() as f32 + entry.padding.x * 2.0,
                    size.height.raw() as f32 + entry.padding.y * 2.0,
                ),
                screen_size: Vec2::new(target_size.width.to_f32(), target_size.height.to_f32()),
                padding: entry.padding,
            })
            .collect();

        let rect_instances = CachedRectInstances { rects };
        let mut buffer_content = StorageBuffer::new(Vec::<u8>::new());
        buffer_content
            .write(&rect_instances)
            .expect("buffer write failed");

        let instance_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Cache Instance Buffer"),
            size: buffer_content.as_ref().len() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu_queue.write_buffer(&instance_buffer, 0, buffer_content.as_ref());

        let transform_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.cache_transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: instance_buffer.as_entire_binding(),
            }],
            label: Some("shape_cache_transform_bind_group"),
        });

        render_pass.set_pipeline(&self.cached_pipeline);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &entry.texture_bind_group, &[]);
        render_pass.set_bind_group(1, &transform_bind_group, &[]);
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
        if let Some((entry, instances)) = pending.take() {
            self.draw_cached_run(gpu, gpu_queue, target_size, render_pass, entry, &instances);
        }
    }
}
