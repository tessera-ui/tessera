use std::sync::Arc;

use encase::{ShaderSize, StorageBuffer};
use glam::Vec2;
use tessera_ui::{Color, PxPosition, PxSize, wgpu};

use super::{
    super::command::{RippleProps, ShapeCommand, rect_to_uniforms},
    ShapeCacheEntry, ShapeInstances, ShapePipeline, ShapeUniforms,
};

#[derive(Debug, Clone)]
pub(super) struct ShapeHeatTracker {
    hit_count: u32,
    pub(super) last_seen_frame: u32,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum ShapeCacheVariant {
    Rect,
    OutlinedRect,
    FilledOutlinedRect,
    Ellipse,
    OutlinedEllipse,
    FilledOutlinedEllipse,
    RippleRect,
    RippleOutlinedRect,
    RippleFilledOutlinedRect,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct RippleKey {
    center: [u32; 2],
    bounded: bool,
    radius: u32,
    alpha: u32,
    color: [u32; 4],
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ShapeCacheKey {
    pub(super) variant: ShapeCacheVariant,
    pub(super) primary_color: [u32; 4],
    pub(super) border_color: Option<[u32; 4]>,
    pub(super) corner_radii: [u32; 4],
    pub(super) corner_g2: [u32; 4],
    pub(super) border_width: u32,
    pub(super) ripple: Option<RippleKey>,
    pub(super) width: u32,
    pub(super) height: u32,
}

fn f32_to_bits(value: f32) -> u32 {
    value.to_bits()
}

fn color_to_bits(color: Color) -> [u32; 4] {
    let arr = color.to_array();
    [
        f32_to_bits(arr[0]),
        f32_to_bits(arr[1]),
        f32_to_bits(arr[2]),
        f32_to_bits(arr[3]),
    ]
}

fn ripple_to_key(ripple: &RippleProps) -> RippleKey {
    RippleKey {
        center: [f32_to_bits(ripple.center[0]), f32_to_bits(ripple.center[1])],
        bounded: ripple.bounded,
        radius: f32_to_bits(ripple.radius),
        alpha: f32_to_bits(ripple.alpha),
        color: color_to_bits(ripple.color),
    }
}

impl ShapeCacheKey {
    pub(super) fn from_command(command: &ShapeCommand, size: PxSize) -> Option<Self> {
        let width = size.width.positive();
        let height = size.height.positive();
        if width == 0 || height == 0 {
            return None;
        }

        match command {
            ShapeCommand::Rect {
                color,
                corner_radii,
                corner_g2,
            } => Some(Self {
                variant: ShapeCacheVariant::Rect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: 0,
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::OutlinedRect {
                color,
                corner_radii,
                corner_g2,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::OutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: f32_to_bits(*border_width),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::FilledOutlinedRect {
                color,
                border_color,
                corner_radii,
                corner_g2,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::FilledOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: f32_to_bits(*border_width),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::Ellipse { color } => Some(Self {
                variant: ShapeCacheVariant::Ellipse,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: [f32_to_bits(-1.0); 4],
                corner_g2: [0; 4],
                border_width: 0,
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::OutlinedEllipse {
                color,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::OutlinedEllipse,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: [f32_to_bits(-1.0); 4],
                corner_g2: [0; 4],
                border_width: f32_to_bits(*border_width),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::FilledOutlinedEllipse {
                color,
                border_color,
                border_width,
            } => Some(Self {
                variant: ShapeCacheVariant::FilledOutlinedEllipse,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: [f32_to_bits(-1.0); 4],
                corner_g2: [0; 4],
                border_width: f32_to_bits(*border_width),
                ripple: None,
                width,
                height,
            }),
            ShapeCommand::RippleRect {
                color,
                corner_radii,
                corner_g2,
                ripple,
            } => Some(Self {
                variant: ShapeCacheVariant::RippleRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: 0,
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
            ShapeCommand::RippleOutlinedRect {
                color,
                corner_radii,
                corner_g2,
                border_width,
                ripple,
            } => Some(Self {
                variant: ShapeCacheVariant::RippleOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: None,
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: f32_to_bits(*border_width),
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
            ShapeCommand::RippleFilledOutlinedRect {
                color,
                border_color,
                corner_radii,
                corner_g2,
                border_width,
                ripple,
            } => Some(Self {
                variant: ShapeCacheVariant::RippleFilledOutlinedRect,
                primary_color: color_to_bits(*color),
                border_color: Some(color_to_bits(*border_color)),
                corner_radii: corner_radii.map(f32_to_bits),
                corner_g2: corner_g2.map(f32_to_bits),
                border_width: f32_to_bits(*border_width),
                ripple: Some(ripple_to_key(ripple)),
                width,
                height,
            }),
        }
    }
}

impl ShapePipeline {
    pub(super) fn get_or_create_cache_entry(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        command: &ShapeCommand,
        size: PxSize,
    ) -> Option<Arc<ShapeCacheEntry>> {
        let key = ShapeCacheKey::from_command(command, size)?;

        let max_dim = gpu.limits().max_texture_dimension_2d;
        if key.width > max_dim || key.height > max_dim {
            return None;
        }

        if let Some(entry) = self.cache.get(&key) {
            return Some(entry.clone());
        }

        let tracker = self
            .heat_tracker
            .entry(key.clone())
            .or_insert(ShapeHeatTracker {
                hit_count: 0,
                last_seen_frame: self.current_frame,
            });

        if tracker.last_seen_frame != self.current_frame {
            tracker.hit_count += 1;
            tracker.last_seen_frame = self.current_frame;
        }

        if tracker.hit_count >= super::CACHE_HEAT_THRESHOLD {
            let entry = Arc::new(self.build_cache_entry(gpu, gpu_queue, command, size));
            self.cache.put(key, entry.clone());
            Some(entry)
        } else {
            None
        }
    }

    fn build_cache_entry(
        &self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        command: &ShapeCommand,
        size: PxSize,
    ) -> ShapeCacheEntry {
        let object_width = size.width.positive().max(1);
        let object_height = size.height.positive().max(1);
        let width = object_width;
        let height = object_height;

        let cache_texture = gpu.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shape Cache Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.render_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let cache_view = cache_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut uniforms = rect_to_uniforms(command, size, PxPosition::ZERO);
        uniforms.screen_size = [width as f32, height as f32].into();
        let instances = vec![uniforms];

        let storage_buffer = gpu.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shape Cache Storage Buffer"),
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
            label: Some("shape_cache_bind_group"),
        });

        let mut encoder = gpu.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Shape Cache Encoder"),
        });

        let run_pass = |pass: &mut wgpu::RenderPass<'_>| {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..uniforms.instances.len() as u32);
        };

        if self.sample_count > 1 {
            let msaa_texture = gpu.create_texture(&wgpu::TextureDescriptor {
                label: Some("Shape Cache MSAA Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.render_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shape Cache Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_view,
                        resolve_target: Some(&cache_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                });
                run_pass(&mut pass);
            }
        } else {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shape Cache Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cache_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            run_pass(&mut pass);
        }

        gpu_queue.submit(Some(encoder.finish()));

        let texture_bind_group = gpu.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.cache_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.cache_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cache_view),
                },
            ],
            label: Some("shape_cache_texture_bind_group"),
        });

        ShapeCacheEntry {
            _texture: cache_texture,
            _view: cache_view,
            texture_bind_group,
            padding: Vec2::ZERO,
        }
    }
}
