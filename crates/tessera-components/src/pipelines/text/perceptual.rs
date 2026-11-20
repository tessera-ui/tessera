use std::{
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

use lru::LruCache;
use tessera_ui::{
    Color, Px, PxPosition, PxSize,
    px::PxRect,
    render::{
        DrawCommand, DrawContext, DrawRegion, DrawablePipeline, FrameContext, PaddingRect,
        SampleRegion,
    },
    wgpu,
};

use super::{ShapedText, TextData, line_baseline, shaping_fonts};

#[cfg(test)]
#[path = "perceptual_tests.rs"]
mod drawing_tests;

const MASK_BYTES: u64 = 32 * 1024 * 1024;
const GLYPH_BYTES: usize = 16 * 1024 * 1024;
const IDLE_FRAMES: u64 = 120;

struct CachedGlyph {
    image: Option<glyphon::SwashImage>,
    frame: u64,
}

struct GlyphCache {
    rasterizer: glyphon::SwashCache,
    images: LruCache<(glyphon::CacheKey, u32), CachedGlyph>,
    bytes: usize,
}

impl GlyphCache {
    fn new() -> Self {
        Self {
            rasterizer: glyphon::SwashCache::new(),
            images: LruCache::new(NonZeroUsize::new(4096).unwrap()),
            bytes: 0,
        }
    }

    fn evict(&mut self) {
        if let Some((_, glyph)) = self.images.pop_lru() {
            self.bytes -= glyph.image.as_ref().map_or(0, |image| image.data.len());
        }
    }

    fn begin_frame(&mut self, frame: u64) {
        while self
            .images
            .peek_lru()
            .is_some_and(|(_, glyph)| frame.saturating_sub(glyph.frame) > IDLE_FRAMES)
        {
            self.evict();
        }
    }

    fn image(
        &mut self,
        fonts: &mut glyphon::FontSystem,
        key: glyphon::CacheKey,
        revision: u32,
        frame: u64,
    ) -> Option<&glyphon::SwashImage> {
        let key = (key, revision);
        if !self.images.contains(&key) {
            let image = self.rasterizer.get_image_uncached(fonts, key.0);
            let bytes = image.as_ref().map_or(0, |image| image.data.len());
            if bytes > GLYPH_BYTES {
                return None;
            }
            while self.bytes + bytes > GLYPH_BYTES || self.images.len() == self.images.cap().get() {
                self.evict();
            }
            self.bytes += bytes;
            self.images.put(key, CachedGlyph { image, frame });
        }
        let glyph = self.images.get_mut(&key).unwrap();
        glyph.frame = frame;
        glyph.image.as_ref()
    }
}

struct InkPixels {
    coverage: Vec<u8>,
    intrinsic: Option<Vec<[u8; 4]>>,
}

impl InkPixels {
    fn new(len: usize) -> Self {
        Self {
            coverage: vec![0; len],
            intrinsic: None,
        }
    }

    fn blend(&mut self, index: usize, mask: u8, color: [u8; 4]) {
        let alpha = u32::from(mask.max(color[3]));
        let remaining = 255 - alpha;
        self.coverage[index] =
            (u32::from(mask) + u32::from(self.coverage[index]) * remaining / 255) as u8;
        if color[3] != 0 && self.intrinsic.is_none() {
            self.intrinsic = Some(vec![[0; 4]; self.coverage.len()]);
        }
        if let Some(intrinsic) = &mut self.intrinsic {
            let pixel = &mut intrinsic[index];
            for channel in 0..3 {
                pixel[channel] = (u32::from(color[channel]) * u32::from(color[3]) / 255
                    + u32::from(pixel[channel]) * remaining / 255)
                    as u8;
            }
            pixel[3] = (u32::from(color[3]) + u32::from(pixel[3]) * remaining / 255) as u8;
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct PerceptualTextCommand {
    pub data: TextData,
    pub tint: Option<(Color, Vec<PxRect>)>,
}

impl DrawCommand for PerceptualTextCommand {
    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::uniform_padding_local(Px(1)))
    }

    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::uniform(Px(1)))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.data.apply_opacity(opacity);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MaskKey {
    layout: usize,
    scale: u32,
    position: [u32; 2],
    bounds: [u32; 2],
    light_ink: bool,
}

struct Mask {
    layout: Weak<ShapedText>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    intrinsic: Option<(wgpu::Texture, wgpu::TextureView)>,
    frame: u64,
}

impl Mask {
    fn bytes(&self) -> u64 {
        u64::from(self.texture.width())
            * u64::from(self.texture.height())
            * if self.intrinsic.is_some() { 5 } else { 1 }
    }
}

pub(crate) struct PerceptualTextPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    masks: LruCache<MaskKey, Mask>,
    glyphs: GlyphCache,
    transparent: wgpu::TextureView,
    bytes: u64,
    frame: u64,
}

impl PerceptualTextPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Perceptual text"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Perceptual text"),
            source: wgpu::ShaderSource::Wgsl(include_str!("perceptual.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Perceptual text"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Perceptual text"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: tessera_ui::render::ui_stencil_state(),
            multisample: wgpu::MultisampleState {
                count: samples,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        });
        let transparent = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Empty text color"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        Self {
            pipeline,
            layout,
            masks: LruCache::new(NonZeroUsize::new(256).expect("nonzero capacity")),
            glyphs: GlyphCache::new(),
            transparent,
            bytes: 0,
            frame: 0,
        }
    }

    fn evict(&mut self) {
        if let Some((_, mask)) = self.masks.pop_lru() {
            self.bytes -= mask.bytes();
        }
    }

    fn mask(
        &mut self,
        data: &TextData,
        origin: PxPosition,
        scale: f32,
        size: PxSize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<MaskKey> {
        let fraction = [origin.x - origin.x.floor(), origin.y - origin.y.floor()];
        let key = MaskKey {
            layout: Arc::as_ptr(&data.layout) as usize,
            scale: scale.to_bits(),
            position: fraction.map(f32::to_bits),
            bounds: [size.width.positive(), size.height.positive()],
            light_ink: data.current_color.r * 0.2126
                + data.current_color.g * 0.7152
                + data.current_color.b * 0.0722
                > 0.5,
        };
        if let Some(mask) = self.masks.get_mut(&key) {
            if mask.layout.upgrade().is_some() {
                mask.frame = self.frame;
                return Some(key);
            }
        }
        let width = (data.layout.measure.size[0].min(key.bounds[0]) as f32 * scale + fraction[0])
            .ceil() as u32;
        let height = (data.layout.measure.size[1].min(key.bounds[1]) as f32 * scale + fraction[1])
            .ceil() as u32;
        let area = u64::from(width) * u64::from(height);
        if width == 0
            || height == 0
            || area * 5 > MASK_BYTES
            || width > device.limits().max_texture_dimension_2d
            || height > device.limits().max_texture_dimension_2d
        {
            return None;
        }
        let mut pixels = InkPixels::new(area as usize);
        let mut fonts = shaping_fonts();
        let x = fraction[0] - data.layout.buffer.scroll().horizontal * scale;
        for run in data.layout.buffer.layout_runs() {
            let baseline = line_baseline(&run, data.layout.native_grid);
            for glyph in run.glyphs {
                let mut physical = glyph.physical((x, baseline * scale + fraction[1]), scale);
                if key.light_ink {
                    physical.cache_key.flags |=
                        glyphon::cosmic_text::CacheKeyFlags::NATIVE_LIGHT_INK;
                }
                let Some(image) = self.glyphs.image(
                    &mut fonts,
                    physical.cache_key,
                    data.layout.font_revision,
                    self.frame,
                ) else {
                    continue;
                };
                for y in 0..image.placement.height {
                    let target_y = physical.y - image.placement.top + y as i32;
                    if !(0..height as i32).contains(&target_y) {
                        continue;
                    }
                    for x in 0..image.placement.width {
                        let target_x = physical.x + image.placement.left + x as i32;
                        if !(0..width as i32).contains(&target_x) {
                            continue;
                        }
                        let source = (y * image.placement.width + x) as usize;
                        let target = (target_y as u32 * width + target_x as u32) as usize;
                        let (mask, color) = match image.content {
                            glyphon::SwashContent::Mask => (image.data[source], [0; 4]),
                            glyphon::SwashContent::SubpixelMask => {
                                let source = source * 4;
                                let alpha = (u32::from(image.data[source])
                                    + u32::from(image.data[source + 1])
                                    + u32::from(image.data[source + 2]))
                                    / 3;
                                (alpha as u8, [0; 4])
                            }
                            glyphon::SwashContent::Color => (
                                0,
                                image.data[source * 4..source * 4 + 4].try_into().unwrap(),
                            ),
                        };
                        pixels.blend(target, mask, color);
                    }
                }
            }
        }
        drop(fonts);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Perceptual text coverage"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &pixels.coverage,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            texture.size(),
        );
        let view = texture.create_view(&Default::default());
        let intrinsic = pixels.intrinsic.map(|pixels| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Intrinsic text color"),
                size: texture.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                texture.as_image_copy(),
                bytemuck::cast_slice(&pixels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                texture.size(),
            );
            let view = texture.create_view(&Default::default());
            (texture, view)
        });
        let bytes = area * if intrinsic.is_some() { 5 } else { 1 };
        while self.bytes + bytes > MASK_BYTES || self.masks.len() == self.masks.cap().get() {
            self.evict();
        }
        if let Some(old) = self.masks.put(
            key,
            Mask {
                layout: Arc::downgrade(&data.layout),
                texture,
                view,
                intrinsic,
                frame: self.frame,
            },
        ) {
            self.bytes -= old.bytes();
        }
        self.bytes += bytes;
        Some(key)
    }
}

impl DrawablePipeline<PerceptualTextCommand> for PerceptualTextPipeline {
    fn begin_frame(&mut self, _: &FrameContext<'_>) {
        self.frame += 1;
        self.glyphs.begin_frame(self.frame);
        while self
            .masks
            .peek_lru()
            .is_some_and(|(_, mask)| self.frame.saturating_sub(mask.frame) > IDLE_FRAMES)
        {
            self.evict();
        }
    }

    fn draw(&mut self, context: &mut DrawContext<'_, '_, '_, PerceptualTextCommand>) {
        for (command, geometry) in context.commands {
            let origin = geometry.local_to_window.map_point(PxPosition::ZERO);
            let scale = geometry.local_to_window.scale();
            let Some(key) = self.mask(
                &command.data,
                origin,
                scale,
                geometry.local_size,
                context.device,
                context.queue,
            ) else {
                continue;
            };
            let mask = self.masks.get(&key).expect("prepared coverage");
            let (tint_color, tint_rects) = command
                .tint
                .as_ref()
                .map_or((Color::TRANSPARENT, &[][..]), |(color, rects)| {
                    (*color, rects.as_slice())
                });
            let tint_bounds: Vec<_> = tint_rects
                .iter()
                .map(|r| {
                    let min = geometry
                        .local_to_window
                        .map_point(PxPosition::new(r.x, r.y));
                    let max = geometry
                        .local_to_window
                        .map_point(PxPosition::new(r.x + r.width, r.y + r.height));
                    [min.x, min.y, max.x, max.y]
                })
                .collect();
            let tint_bounds = if tint_bounds.is_empty() {
                &[[0.0; 4]][..]
            } else {
                &tint_bounds
            };
            let tint_buffer = context.graphics.frame_buffer(
                wgpu::BufferUsages::STORAGE,
                bytemuck::cast_slice(tint_bounds),
            );
            let uniform = [
                [
                    origin.x.floor(),
                    origin.y.floor(),
                    mask.texture.width() as f32,
                    mask.texture.height() as f32,
                ],
                [
                    context.target_size.width.to_f32(),
                    context.target_size.height.to_f32(),
                    context.scene_rect.x.to_f32(),
                    context.scene_rect.y.to_f32(),
                ],
                command.data.current_color.to_array(),
                [
                    origin.x,
                    origin.y,
                    geometry.window_size()[0],
                    geometry.window_size()[1],
                ],
                tint_color.to_array(),
            ];
            let buffer = context
                .graphics
                .frame_buffer(wgpu::BufferUsages::UNIFORM, bytemuck::cast_slice(&uniform));
            let bind_group = context
                .graphics
                .cached_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Perceptual text"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&mask.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(
                                context.scene_texture_view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(
                                mask.intrinsic
                                    .as_ref()
                                    .map_or(&self.transparent, |(_, view)| view),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: tint_buffer.binding(),
                        },
                    ],
                });
            context.render_pass.set_pipeline(&self.pipeline);
            context.render_pass.set_bind_group(0, &bind_group, &[]);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colored_glyphs_and_label_masks_obey_painter_order() {
        let mut pixels = InkPixels::new(2);
        pixels.blend(0, 255, [0; 4]);
        pixels.blend(0, 0, [255, 0, 0, 128]);
        assert_eq!(pixels.coverage[0], 127);
        assert_eq!(pixels.intrinsic.as_ref().unwrap()[0], [128, 0, 0, 128]);
        pixels.blend(0, 128, [0; 4]);
        assert_eq!(pixels.coverage[0], 191);
        assert_eq!(pixels.intrinsic.as_ref().unwrap()[0], [63, 0, 0, 63]);
        pixels.blend(1, 0, [0, 255, 0, 255]);
        pixels.blend(1, 255, [0; 4]);
        assert_eq!(pixels.coverage[1], 255);
        assert_eq!(pixels.intrinsic.as_ref().unwrap()[1], [0; 4]);
    }

    #[test]
    fn glyph_cache_expires_idle_images_and_separates_font_revisions() {
        let mut database = glyphon::fontdb::Database::new();
        database.load_font_data(
            include_bytes!("../../../../tessera-cosmic-text/fonts/InterVariable.ttf").to_vec(),
        );
        let mut fonts = glyphon::FontSystem::new_with_locale_and_db("en-US".into(), database);
        let id = fonts.db().faces().next().unwrap().id;
        let glyph = fonts
            .get_font(id, glyphon::Weight::NORMAL)
            .unwrap()
            .as_swash()
            .charmap()
            .map('H');
        let key = glyphon::CacheKey::new(
            id,
            glyph,
            26.0,
            (0.0, 0.0),
            glyphon::Weight::NORMAL,
            glyphon::cosmic_text::CacheKeyFlags::empty(),
        )
        .0;
        let mut cache = GlyphCache::new();
        cache.images.resize(NonZeroUsize::new(2).unwrap());
        let bytes = cache.image(&mut fonts, key, 0, 1).unwrap().data.len();
        assert!(bytes > 0);
        cache.image(&mut fonts, key, 1, 2).unwrap();
        assert_eq!(cache.bytes, bytes * 2);
        cache.image(&mut fonts, key, 2, 3).unwrap();
        assert!(!cache.images.contains(&(key, 0)));
        assert_eq!(cache.bytes, bytes * 2);
        cache.begin_frame(123);
        assert_eq!(cache.images.len(), 1);
        cache.begin_frame(124);
        assert!(cache.images.is_empty());
        assert_eq!(cache.bytes, 0);
        assert!(cache.rasterizer.image_cache.is_empty());
    }
}
