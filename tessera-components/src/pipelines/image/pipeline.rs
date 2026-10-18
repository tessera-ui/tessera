use std::{
    borrow::Cow,
    collections::HashMap,
    num::NonZeroU64,
    sync::{Arc, Weak},
};

use encase::{ShaderType, UniformBuffer};
use glam::Vec4;
use tessera_ui::{
    PxPosition, PxSize,
    renderer::drawer::pipeline::{DrawContext, DrawablePipeline, FrameContext},
    wgpu,
};

use crate::pipelines::SharedQuad;

use super::command::{ImageCommand, ImageData};

#[cfg(test)]
pub(crate) static INSTANCE_ALLOCATIONS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
pub(crate) static TEXTURE_UPLOADS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[derive(PartialEq, ShaderType)]
struct ImageUniforms {
    rect: Vec4,
    opacity: f32,
}

struct ImageResources {
    bind_group: wgpu::BindGroup,
    last_used: u64,
    bytes: u64,
}

struct ImageAlias {
    // Keep allocation identities unique without retaining duplicate pixel buffers.
    pixels: Weak<Vec<u8>>,
    key: ImageData,
}

/// Pipeline for rendering images in UI components.
pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    uniform_group: wgpu::BindGroup,
    uniform_capacity: u64,
    uniform_used: u64,
    sampler: wgpu::Sampler,
    frame: u64,
    cached_bytes: u64,
    quad_vertex_buffer: Arc<wgpu::Buffer>,
    quad_index_buffer: Arc<wgpu::Buffer>,
    resources: HashMap<ImageData, ImageResources>,
    aliases: HashMap<(usize, u32, u32), ImageAlias>,
}

impl ImagePipeline {
    /// Create a new ImagePipeline.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        sample_count: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Instance Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(ImageUniforms::min_size()),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        });

        let shared_quad = SharedQuad::get(device);
        let (uniform_buffer, uniform_group) =
            Self::uniform_resources(device, &uniform_layout, 4096);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_layout,
            uniform_buffer,
            uniform_group,
            uniform_capacity: 4096,
            uniform_used: 0,
            sampler,
            frame: 0,
            cached_bytes: 0,
            quad_vertex_buffer: Arc::clone(shared_quad.vertex()),
            quad_index_buffer: Arc::clone(shared_quad.index()),
            resources: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Return existing resources for `data` or create them.
    fn get_or_create_resources(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &ImageData,
    ) -> &ImageResources {
        let identity = (Arc::as_ptr(&data.data) as usize, data.width, data.height);
        let key = if let Some(alias) = self.aliases.get(&identity) {
            alias.key.clone()
        } else {
            let key = if let Some((key, _)) = self.resources.get_key_value(data) {
                key.clone()
            } else {
                let bytes = u64::from(data.width) * u64::from(data.height) * 4;
                while self.cached_bytes.saturating_add(bytes) > 64 * 1024 * 1024 {
                    let oldest = self
                        .resources
                        .iter()
                        .filter(|(_, resource)| resource.last_used != self.frame)
                        .min_by_key(|(_, resource)| resource.last_used)
                        .map(|(data, _)| data.clone());
                    let Some(oldest) = oldest else {
                        break;
                    };
                    if let Some(resource) = self.resources.remove(&oldest) {
                        self.cached_bytes = self.cached_bytes.saturating_sub(resource.bytes);
                    }
                    self.aliases.retain(|_, alias| {
                        !Arc::ptr_eq(&alias.key.data, &oldest.data)
                            || alias.key.width != oldest.width
                            || alias.key.height != oldest.height
                    });
                }
                let mut resource = Self::create_image_resources(
                    device,
                    queue,
                    &self.bind_group_layout,
                    &self.sampler,
                    data,
                );
                resource.last_used = self.frame;
                self.cached_bytes = self.cached_bytes.saturating_add(bytes);
                self.resources.insert(data.clone(), resource);
                data.clone()
            };
            self.aliases.insert(
                identity,
                ImageAlias {
                    pixels: Arc::downgrade(&data.data),
                    key: key.clone(),
                },
            );
            key
        };
        let resource = self.resources.get_mut(&key).expect("cached image");
        resource.last_used = self.frame;
        resource
    }

    fn uniform_resources(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        capacity: u64,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        #[cfg(test)]
        INSTANCE_ALLOCATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Image Instances"),
            size: capacity,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Image Instances"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: NonZeroU64::new(ImageUniforms::min_size().get()),
                }),
            }],
        });
        (buffer, group)
    }

    /// Compute the ImageUniforms for a given command size and position.
    fn compute_uniforms(
        start_pos: PxPosition,
        size: PxSize,
        target_size: PxSize,
        opacity: f32,
    ) -> ImageUniforms {
        // Convert pixel positions/sizes into normalized device coordinates and
        // size ratios.
        let rect = [
            (start_pos.x / target_size.width.to_f32()) * 2.0 - 1.0
                + (size.width.0 as f32 / target_size.width.to_f32()),
            (start_pos.y / target_size.height.to_f32()) * -2.0 + 1.0
                - (size.height.0 as f32 / target_size.height.to_f32()),
            size.width.0 as f32 / target_size.width.to_f32(),
            size.height.0 as f32 / target_size.height.to_f32(),
        ]
        .into();

        ImageUniforms { rect, opacity }
    }

    // Create GPU resources for an image. Kept as a single helper to avoid
    // duplicating GPU setup logic while keeping `draw` concise.
    fn create_image_resources(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        data: &ImageData,
    ) -> ImageResources {
        #[cfg(test)]
        TEXTURE_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let texture_size = wgpu::Extent3d {
            width: data.width,
            height: data.height,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &premultiply_rgba(&data.data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * data.width),
                rows_per_image: Some(data.height),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        ImageResources {
            bind_group: diffuse_bind_group,
            last_used: 0,
            bytes: u64::from(data.width) * u64::from(data.height) * 4,
        }
    }
}

impl DrawablePipeline<ImageCommand> for ImagePipeline {
    fn begin_frame(&mut self, _context: &FrameContext<'_>) {
        self.uniform_used = 0;
        self.frame = self.frame.wrapping_add(1);
        self.aliases.retain(|_, alias| {
            alias.pixels.strong_count() > 0 && self.resources.contains_key(&alias.key)
        });
    }

    fn draw(&mut self, context: &mut DrawContext<ImageCommand>) {
        context.render_pass.set_pipeline(&self.pipeline);
        context
            .render_pass
            .set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        context
            .render_pass
            .set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (command, size, start_pos) in context.commands.iter() {
            let uniforms =
                Self::compute_uniforms(*start_pos, *size, context.target_size, command.opacity);

            let mut buffer = UniformBuffer::new(Vec::new());
            buffer.write(&uniforms).expect("buffer write failed");
            let bytes = buffer.into_inner();
            let alignment =
                u64::from(context.device.limits().min_uniform_buffer_offset_alignment).max(1);
            let offset = self.uniform_used.next_multiple_of(alignment);
            let required = offset + bytes.len() as u64;
            if required > self.uniform_capacity {
                self.uniform_capacity = required.next_power_of_two();
                (self.uniform_buffer, self.uniform_group) = Self::uniform_resources(
                    context.device,
                    &self.uniform_layout,
                    self.uniform_capacity,
                );
            }
            context
                .queue
                .write_buffer(&self.uniform_buffer, offset, &bytes);
            self.uniform_used = required;
            context.render_pass.set_bind_group(
                0,
                &self.uniform_group,
                &[u32::try_from(offset).expect("image instance offset")],
            );
            let resource =
                self.get_or_create_resources(context.device, context.queue, &command.data);
            context
                .render_pass
                .set_bind_group(1, &resource.bind_group, &[]);
            context.render_pass.draw_indexed(0..6, 0, 0..1);
        }
    }
}

fn premultiply_rgba(bytes: &[u8]) -> Cow<'_, [u8]> {
    if bytes.chunks_exact(4).all(|pixel| pixel[3] == 255) {
        return Cow::Borrowed(bytes);
    }
    let mut result = bytes.to_vec();
    for pixel in result.chunks_exact_mut(4) {
        if pixel[3] == 255 {
            continue;
        }
        if pixel[3] == 0 {
            pixel[..3].fill(0);
            continue;
        }
        let alpha = f32::from(pixel[3]) / 255.0;
        for channel in &mut pixel[..3] {
            let srgb = f32::from(*channel) / 255.0;
            let linear = if srgb <= 0.04045 {
                srgb / 12.92
            } else {
                ((srgb + 0.055) / 1.055).powf(2.4)
            } * alpha;
            let encoded = if linear <= 0.0031308 {
                linear * 12.92
            } else {
                1.055 * linear.powf(1.0 / 2.4) - 0.055
            };
            *channel = (encoded * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    Cow::Owned(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_pixels_reuse_the_original_bytes() {
        let bytes = [128, 64, 192, 255];
        assert!(matches!(premultiply_rgba(&bytes), Cow::Borrowed(_)));
    }

    #[test]
    fn transparent_pixels_are_premultiplied_in_linear_light() {
        assert_eq!(
            premultiply_rgba(&[255, 0, 0, 128]).as_ref(),
            [188, 0, 0, 128]
        );
        assert_eq!(premultiply_rgba(&[255, 128, 64, 0]).as_ref(), [0, 0, 0, 0]);
    }
}
