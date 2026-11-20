use std::collections::HashMap;

use encase::{ShaderType, StorageBuffer, UniformBuffer};
use glam::{IVec4, Vec4};
use tessera_ui::{
    render::{
        DrawContext, DrawGeometry, DrawPreparationContext, DrawablePipeline, FrameContext,
        ScratchFit, ScratchTexture, ScratchTextureDescriptor,
    },
    wgpu,
};

use super::{
    LiquidGlassCommand,
    params::{self, GlassParams, MAX_LEVELS},
};

#[derive(ShaderType)]
struct BackdropParams {
    region: IVec4,
    capture: IVec4,
    scene: IVec4,
    levels: IVec4,
    sizes: IVec4,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct PreparedKey {
    command: usize,
    origin: [u32; 2],
}

impl PreparedKey {
    fn new(command: &LiquidGlassCommand, geometry: DrawGeometry) -> Self {
        let origin = geometry.window_origin();
        Self {
            // Equal groups in different draws must not share a backdrop chain.
            command: std::ptr::from_ref(command) as usize,
            origin: [origin.x.to_bits(), origin.y.to_bits()],
        }
    }
}

struct Prepared {
    atlas: ScratchTexture,
    params: GlassParams,
}

/// Backdrop region and blur chain placement, in pixels from the bottom-left target corner.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ChainLayout {
    region: [i32; 4],
    texels: [i32; 2],
    surface_origin: [i32; 2],
    surface_size: [i32; 2],
    levels: usize,
}

impl ChainLayout {
    fn new(bounds: [f32; 4], margin_px: f32, target: [i32; 2], levels: usize) -> Self {
        let [x0, y0, x1, y1] = bounds;
        let expand = margin_px;
        let rx0 = ((x0 - expand).floor() as i32).max(0);
        let ry0 = ((y0 - expand).floor() as i32).max(0);
        let rx1 = ((x1 + expand).ceil() as i32).min(target[0]);
        let ry1 = ((y1 + expand).ceil() as i32).min(target[1]);
        let texels = [((rx1 - rx0) / 2).max(1), ((ry1 - ry0) / 2).max(1)];
        let align = 1 << levels;
        let reach = (1 << (levels + 1)) - 2;
        let sx0 = (rx0 - reach).div_euclid(align) * align;
        let sy0 = (ry0 - reach).div_euclid(align) * align;
        let offset = [(rx0 - sx0) / 2, (ry0 - sy0) / 2];
        let round = |value: i32| (value + 63) / 64 * 64;
        Self {
            region: [rx0, ry0, rx1, ry1],
            texels,
            surface_origin: [sx0, sy0],
            surface_size: [
                round(texels[0] + 2 * offset[0]),
                round(texels[1] + 2 * offset[1]),
            ],
            levels,
        }
    }

    fn capture_offset(&self) -> [i32; 2] {
        [
            (self.region[0] - self.surface_origin[0]) / 2,
            (self.region[1] - self.surface_origin[1]) / 2,
        ]
    }

    /// Atlas placement of each level: level 0 on the left, later levels stacked to its right.
    fn level_rects(&self) -> Vec<[i32; 4]> {
        let [w, h] = self.surface_size;
        let mut y = 0;
        (0..self.levels)
            .map(|level| {
                if level == 0 {
                    return [0, 0, w, h];
                }
                let size = [(w >> level).max(1), (h >> level).max(1)];
                let rect = [w, y, size[0], size[1]];
                y += size[1];
                rect
            })
            .collect()
    }

    fn atlas_size(&self) -> [u32; 2] {
        let [w, h] = self.surface_size;
        let extra = if self.levels > 1 { w / 2 } else { 0 };
        [(w + extra) as u32, h as u32]
    }
}

pub(crate) struct LiquidGlassPipeline {
    glass: wgpu::RenderPipeline,
    glass_layout: wgpu::BindGroupLayout,
    base: wgpu::ComputePipeline,
    downsample: wgpu::ComputePipeline,
    backdrop_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    prepared: HashMap<PreparedKey, Prepared>,
}

impl LiquidGlassPipeline {
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let glass_shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("liquid glass"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glass.wgsl").into()),
        });
        let backdrop_shader = gpu.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("liquid glass backdrop"),
            source: wgpu::ShaderSource::Wgsl(include_str!("backdrop.wgsl").into()),
        });
        let texture = |binding, visibility, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let buffer = |binding, visibility, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let fragment = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;
        let glass_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("liquid glass layout"),
            entries: &[
                buffer(0, fragment, wgpu::BufferBindingType::Uniform),
                buffer(
                    1,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BufferBindingType::Storage { read_only: true },
                ),
                texture(2, wgpu::ShaderStages::FRAGMENT, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture(4, wgpu::ShaderStages::FRAGMENT, false),
            ],
        });
        let backdrop_layout = gpu.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("liquid glass backdrop layout"),
            entries: &[
                buffer(
                    0,
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::BufferBindingType::Uniform,
                ),
                texture(1, wgpu::ShaderStages::COMPUTE, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let glass_pipeline_layout = gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("liquid glass pipeline layout"),
            bind_group_layouts: &[Some(&glass_layout)],
            immediate_size: 0,
        });
        let glass = gpu.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("liquid glass"),
            layout: Some(&glass_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &glass_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &glass_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    // The shader composites over the sampled scene itself.
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::render::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: pipeline_cache,
        });
        let backdrop_pipeline_layout =
            gpu.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("liquid glass backdrop pipeline layout"),
                bind_group_layouts: &[Some(&backdrop_layout)],
                immediate_size: 0,
            });
        let compute = |entry_point| {
            gpu.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&backdrop_pipeline_layout),
                module: &backdrop_shader,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: pipeline_cache,
            })
        };
        let sampler = gpu.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("liquid glass sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            glass,
            glass_layout,
            base: compute("cs_base"),
            downsample: compute("cs_downsample"),
            backdrop_layout,
            sampler,
            prepared: HashMap::new(),
        }
    }

    fn encode_level(
        &self,
        context: &mut DrawPreparationContext<'_, LiquidGlassCommand>,
        pipeline: &wgpu::ComputePipeline,
        uniforms: &BackdropParams,
        source: &wgpu::TextureView,
        scratch: &wgpu::TextureView,
        size: [i32; 2],
    ) {
        let mut content = UniformBuffer::new(Vec::<u8>::new());
        content
            .write(uniforms)
            .expect("liquid glass backdrop uniforms");
        let buffer = context
            .graphics
            .frame_buffer(wgpu::BufferUsages::UNIFORM, content.as_ref());
        let bindings = context
            .graphics
            .cached_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("liquid glass backdrop bindings"),
                layout: &self.backdrop_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scratch),
                    },
                ],
            });
        let mut pass = context
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("liquid glass backdrop"),
                timestamp_writes: None,
            });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups(
            (size[0] as u32).div_ceil(8),
            (size[1] as u32).div_ceil(8),
            1,
        );
    }

    fn prepare_chain(
        &self,
        context: &mut DrawPreparationContext<'_, LiquidGlassCommand>,
    ) -> Prepared {
        let command = context.command;
        let geometry = context.geometry;
        let scale = command.pixels_per_point;
        let origin = geometry.window_origin();
        let target = [context.target_size.width.0, context.target_size.height.0];
        let height = target[1] as f32;
        let (min, max) =
            command
                .elements
                .iter()
                .fold(([f32::MAX; 2], [f32::MIN; 2]), |(min, max), element| {
                    (
                        [min[0].min(element.x), min[1].min(element.y)],
                        [
                            max[0].max(element.x + element.width),
                            max[1].max(element.y + element.height),
                        ],
                    )
                });
        let left = origin.x + min[0] * scale;
        let right = origin.x + max[0] * scale;
        let top = origin.y + min[1] * scale;
        let bottom = origin.y + max[1] * scale;
        let layout = ChainLayout::new(
            [left, height - bottom, right, height - top],
            (command.material.margin + 2.0) * scale,
            target,
            command.material.levels(),
        );
        let rects = layout.level_rects();
        let atlas_size = layout.atlas_size();
        let lease = |context: &mut DrawPreparationContext<'_, LiquidGlassCommand>,
                     size: [u32; 2],
                     usage| {
            context.graphics.scratch_texture(ScratchTextureDescriptor {
                width: size[0],
                height: size[1],
                format: wgpu::TextureFormat::Rgba16Float,
                usage,
                sample_count: 1,
                fit: ScratchFit::Exact,
            })
        };
        let atlas = lease(
            context,
            atlas_size,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let scratch = lease(
            context,
            [layout.surface_size[0] as u32, layout.surface_size[1] as u32],
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        );
        let scene = [context.scene_rect.x.0, context.scene_rect.y.0];
        let offset = layout.capture_offset();
        let region = IVec4::new(
            layout.region[0],
            layout.region[1],
            layout.texels[0],
            layout.texels[1],
        );
        for level in 0..layout.levels {
            let rect = rects[level];
            let (pipeline, source, source_rect) = if level == 0 {
                (&self.base, context.scene_texture_view.clone(), [0, 0, 0, 0])
            } else {
                (&self.downsample, (*atlas).clone(), rects[level - 1])
            };
            let uniforms = BackdropParams {
                region,
                capture: IVec4::new(offset[0], offset[1], target[1], i32::from(level == 1)),
                scene: IVec4::new(scene[0], scene[1], 0, 0),
                levels: IVec4::new(source_rect[0], source_rect[1], 0, 0),
                sizes: IVec4::new(source_rect[2], source_rect[3], rect[2], rect[3]),
            };
            self.encode_level(
                context,
                pipeline,
                &uniforms,
                &source,
                &scratch,
                [rect[2], rect[3]],
            );
            context.encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: scratch.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: atlas.texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: rect[0] as u32,
                        y: rect[1] as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: rect[2] as u32,
                    height: rect[3] as u32,
                    depth_or_array_layers: 1,
                },
            );
        }

        let mut glass = params::material_params(command);
        let field_padding = command.field_padding() + command.material.shadow_offset.abs();
        let pad = field_padding * scale;
        glass.origin_scale = Vec4::new(origin.x, origin.y, scale, command.opacity);
        glass.target_scene = Vec4::new(
            target[0] as f32,
            target[1] as f32,
            scene[0] as f32,
            scene[1] as f32,
        );
        glass.draw_rect = Vec4::new(
            (left - pad).floor().max(0.0),
            (top - pad).floor().max(0.0),
            (right + pad).ceil().min(target[0] as f32),
            (bottom + pad).ceil().min(height),
        );
        glass.surface = Vec4::new(
            layout.surface_origin[0] as f32,
            layout.surface_origin[1] as f32,
            layout.levels as f32,
            0.0,
        );
        for (level, rect) in rects.iter().enumerate().take(MAX_LEVELS) {
            glass.levels[level] = Vec4::new(
                rect[0] as f32,
                rect[1] as f32,
                rect[2] as f32,
                rect[3] as f32,
            );
        }
        Prepared {
            atlas,
            params: glass,
        }
    }
}

impl DrawablePipeline<LiquidGlassCommand> for LiquidGlassPipeline {
    fn prepare(&mut self, context: &mut DrawPreparationContext<'_, LiquidGlassCommand>) {
        if context.command.elements.is_empty() {
            return;
        }
        let prepared = self.prepare_chain(context);
        self.prepared.insert(
            PreparedKey::new(context.command, context.geometry),
            prepared,
        );
    }

    fn end_frame(&mut self, _: &FrameContext<'_>) {
        self.prepared.clear();
    }

    fn draw(&mut self, context: &mut DrawContext<LiquidGlassCommand>) {
        for &(command, geometry) in context.commands {
            let Some(prepared) = self.prepared.get(&PreparedKey::new(command, geometry)) else {
                continue;
            };
            let mut uniforms = UniformBuffer::new(Vec::<u8>::new());
            uniforms
                .write(&prepared.params)
                .expect("liquid glass uniforms");
            let uniforms = context
                .graphics
                .frame_buffer(wgpu::BufferUsages::UNIFORM, uniforms.as_ref());
            let mut elements = StorageBuffer::new(Vec::<u8>::new());
            elements
                .write(&params::element_data(command))
                .expect("liquid glass elements");
            let elements = context
                .graphics
                .frame_buffer(wgpu::BufferUsages::STORAGE, elements.as_ref());
            let bindings = context
                .graphics
                .cached_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("liquid glass bindings"),
                    layout: &self.glass_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniforms.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: elements.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&prepared.atlas),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(
                                context.scene_texture_view,
                            ),
                        },
                    ],
                });
            context.render_pass.set_pipeline(&self.glass);
            context.render_pass.set_bind_group(0, &bindings, &[]);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ChainLayout;

    #[test]
    fn chain_layout_follows_the_capture_rules() {
        // A 764x40 pt group at 2x whose bottom edge is 910 px above the bottom of a 1600x1040
        // target, with a 16 pt margin and a five-level chain.
        let regular = ChainLayout::new([32.0, 910.0, 1560.0, 990.0], 36.0, [1600, 1040], 5);
        assert_eq!(regular.region, [0, 874, 1596, 1026]);
        assert_eq!(regular.texels, [798, 76]);
        assert_eq!(regular.surface_origin, [-64, 800]);
        assert_eq!(regular.surface_size, [896, 192]);

        let clear = ChainLayout::new([32.0, 430.0, 1560.0, 510.0], 6.0, [1600, 1040], 2);
        assert_eq!(clear.region, [26, 424, 1566, 516]);
        assert_eq!(clear.surface_origin, [20, 416]);
        assert_eq!(clear.surface_size, [832, 64]);
        assert_eq!(
            clear.level_rects(),
            vec![[0, 0, 832, 64], [832, 0, 416, 32]]
        );
    }
}
