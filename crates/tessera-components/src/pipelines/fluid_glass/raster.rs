use encase::StorageBuffer;
use glam::Vec2;
use tessera_ui::{
    render::{
        DrawCommand, DrawGeometry, DrawPreparationContext, SampleRegion, ScratchFit,
        ScratchTexture, ScratchTextureDescriptor,
    },
    wgpu,
};

use super::{FluidGlassCommand, GlassInstances, glass_uniforms};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(super) struct PreparedLensKey {
    command: usize,
    geometry: [u32; 5],
}

impl PreparedLensKey {
    pub(super) fn new(command: &FluidGlassCommand, geometry: DrawGeometry) -> Self {
        let origin = geometry.window_origin();
        Self {
            // Equal materials in different draws must not share a backdrop raster.
            command: std::ptr::from_ref(command) as usize,
            geometry: [
                geometry.local_size.width.0 as u32,
                geometry.local_size.height.0 as u32,
                origin.x.to_bits(),
                origin.y.to_bits(),
                geometry.local_to_window.scale().to_bits(),
            ],
        }
    }
}

pub(super) struct PreparedLens {
    pub texture: ScratchTexture,
    pub origin: Vec2,
    pub size: Vec2,
}

pub(super) struct LensRasterPipeline {
    primary: wgpu::ComputePipeline,
    background: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl LensRasterPipeline {
    pub(super) fn new(
        device: &wgpu::Device,
        cache: Option<&wgpu::PipelineCache>,
        shader: &wgpu::ShaderModule,
    ) -> Self {
        let texture = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glass raster layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture(1, true),
                texture(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("glass raster pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry_point| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache,
            })
        };
        Self {
            primary: pipeline("cs_primary"),
            background: pipeline("cs_background"),
            layout,
        }
    }

    pub(super) fn prepare(
        &self,
        context: &mut DrawPreparationContext<'_, FluidGlassCommand>,
    ) -> PreparedLens {
        let Some(SampleRegion::PaddedLocal(padding)) = context.command.sample_region() else {
            unreachable!("lens declares local sampling bounds");
        };
        let region = context.geometry.padded_window_bounds(padding).outward();
        let origin = context.geometry.window_origin();
        let raster_origin = Vec2::new(origin.x - region.x.to_f32(), origin.y - region.y.to_f32());
        let size = Vec2::new(region.width.to_f32(), region.height.to_f32());
        let mut uniforms = glass_uniforms(
            context.command,
            context.geometry,
            context.target_size,
            context.scene_rect,
            context.scene_texture_view,
        );
        uniforms.raster_origin = raster_origin;
        uniforms.raster_size = size;
        let mut content = StorageBuffer::new(Vec::<u8>::new());
        content
            .write(&GlassInstances {
                instances: vec![uniforms],
            })
            .expect("glass raster uniforms");
        let buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::STORAGE, content.as_ref());
        let lease = || {
            context.graphics.scratch_texture(ScratchTextureDescriptor {
                width: region.width.positive(),
                height: region.height.positive(),
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                sample_count: 1,
                fit: ScratchFit::Approx,
            })
        };
        let primary = lease();
        let background = lease();
        for (pipeline, input, output) in [
            (&self.primary, context.scene_texture_view, &*primary),
            (&self.background, &*primary, &*background),
        ] {
            let bindings = context
                .graphics
                .cached_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("glass raster bindings"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                context.scene_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(input),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(output),
                        },
                    ],
                });
            let mut pass = context
                .encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("glass raster pass"),
                    timestamp_writes: None,
                });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.dispatch_workgroups(
                region.width.positive().div_ceil(8),
                region.height.positive().div_ceil(8),
                1,
            );
        }
        PreparedLens {
            texture: background,
            origin: raster_origin,
            size,
        }
    }
}
