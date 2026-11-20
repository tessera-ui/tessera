use std::collections::HashMap;

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
    shared::{BackdropParams, GlassTextures, Shared},
};

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
    shared: Shared,
    glass: wgpu::RenderPipeline,
    base: wgpu::ComputePipeline,
    downsample: wgpu::ComputePipeline,
    prepared: HashMap<PreparedKey, Prepared>,
}

impl LiquidGlassPipeline {
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let shared = Shared::new(gpu);
        Self {
            glass: shared.render(
                gpu,
                pipeline_cache,
                ["vs_main", "fs_main"],
                Some((config.format, sample_count)),
            ),
            base: shared.compute(gpu, pipeline_cache, "cs_base"),
            downsample: shared.compute(gpu, pipeline_cache, "cs_downsample"),
            shared,
            prepared: HashMap::new(),
        }
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
            self.shared.dispatch(
                context.graphics,
                context.encoder,
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
            let bindings = self.shared.bindings(
                context.graphics,
                &prepared.params,
                &params::element_data(command),
                GlassTextures {
                    surface: &prepared.atlas,
                    scene: context.scene_texture_view,
                    original: context.scene_texture_view,
                },
            );
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
