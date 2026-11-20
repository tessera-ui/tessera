use std::collections::HashMap;

use glam::{IVec4, Vec4};
use tessera_ui::{
    Px,
    render::{
        DrawCommand, DrawContext, DrawGeometry, DrawPreparationContext, DrawRegion,
        DrawablePipeline, FrameContext, PaddingRect, SampleRegion, ScratchFit, ScratchTexture,
        ScratchTextureDescriptor,
    },
    wgpu,
};

use super::{
    LiquidGlassCommand, LiquidGlassElement, LiquidGlassMaterial, ToneMatrix,
    params::{self, GlassParams},
    shared::{BackdropParams, GlassTextures, Shared},
};

/// Extent of the lens shading and its shadow beyond the lens, in points.
const REACH: f32 = 24.0;

/// Glass shading of the lens body; it samples a full-resolution backdrop without blur.
const BODY: LiquidGlassMaterial = LiquidGlassMaterial {
    margin: 2.0,
    field_extent: 8.062_464,
    inner_refraction: [-4.65, 3.1],
    outer_refraction: [0.0, 0.0],
    refraction_distance: [-1.0, 0.0],
    refraction_opacity: 0.0,
    blur_radius: 0.0,
    blur_opacity: [0.0; 4],
    blur_distance: [-10.0, 0.0, 0.0, 0.0],
    blur_fill_radius: 0.0,
    blur_fill: [0.0; 3],
    face: ToneMatrix {
        white: 1.0,
        black: 0.0,
        saturation: 1.0,
        fill: [1.0, 1.0, 1.0, 0.0],
    },
    face_max_luma: 1.0,
    face_opacity: 1.0,
    shadow: ToneMatrix {
        white: 0.0,
        black: 0.0,
        saturation: 1.0,
        fill: [0.0, 0.0, 0.0, 1.0],
    },
    shadow_opacity: 0.001,
    shadow_radius: 8.0,
    shadow_offset: 7.0,
    ring: [8.0, 4.0, 3.0, 0.1, 0.0],
    highlight: [std::f32::consts::FRAC_PI_2, 1.0, 2.094_395, 0.5, -1.0, -0.3],
    clamp: 0.0,
    vibrant: [
        [1.1202, -0.1894, -0.019, 0.0, 0.1471],
        [-0.0563, 0.9871, -0.0191, 0.0, 0.1471],
        [-0.0563, -0.1893, 1.1574, 0.0, 0.1471],
        [0.0, 0.0, 0.0, 1.0, 0.0],
    ],
};

/// Draw command for a pressed control rendered as a refracting lens.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LiquidLensCommand {
    /// Lens rectangle in local points.
    pub element: LiquidGlassElement,
    pub pixels_per_point: f32,
    pub opacity: f32,
}

impl LiquidLensCommand {
    fn padding(&self) -> Px {
        Px((REACH * self.pixels_per_point).ceil() as i32)
    }

    fn glass(&self) -> LiquidGlassCommand {
        LiquidGlassCommand {
            material: BODY,
            elements: vec![self.element],
            smoothness: 0.0,
            pixels_per_point: self.pixels_per_point,
            opacity: self.opacity,
        }
    }
}

impl DrawCommand for LiquidLensCommand {
    fn draw_region(&self) -> DrawRegion {
        DrawRegion::PaddedLocal(PaddingRect::uniform(self.padding()))
    }

    fn sample_region(&self) -> Option<SampleRegion> {
        Some(SampleRegion::uniform_padding_local(self.padding() + Px(2)))
    }

    fn apply_opacity(&mut self, opacity: f32) {
        self.opacity *= opacity.clamp(0.0, 1.0);
    }
}

struct Prepared {
    original: ScratchTexture,
    body: ScratchTexture,
    params: GlassParams,
}

pub(crate) struct LiquidLensPipeline {
    shared: Shared,
    copy: wgpu::ComputePipeline,
    displace: wgpu::RenderPipeline,
    background: wgpu::RenderPipeline,
    lens: wgpu::RenderPipeline,
    prepared: HashMap<(usize, [u32; 2]), Prepared>,
}

fn key(command: &LiquidLensCommand, geometry: DrawGeometry) -> (usize, [u32; 2]) {
    let origin = geometry.window_origin();
    (
        std::ptr::from_ref(command) as usize,
        [origin.x.to_bits(), origin.y.to_bits()],
    )
}

impl LiquidLensPipeline {
    pub fn new(
        gpu: &wgpu::Device,
        pipeline_cache: Option<&wgpu::PipelineCache>,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let shared = Shared::new(gpu);
        Self {
            copy: shared.compute(gpu, pipeline_cache, "cs_copy"),
            displace: shared.render(gpu, pipeline_cache, ["vs_region", "fs_lens_displace"], None),
            background: shared.render(
                gpu,
                pipeline_cache,
                ["vs_region", "fs_lens_background"],
                None,
            ),
            lens: shared.render(
                gpu,
                pipeline_cache,
                ["vs_main", "fs_lens"],
                Some((config.format, sample_count)),
            ),
            shared,
            prepared: HashMap::new(),
        }
    }

    fn region_pass(
        &self,
        context: &mut DrawPreparationContext<'_, LiquidLensCommand>,
        pipeline: &wgpu::RenderPipeline,
        params: &GlassParams,
        elements: &params::Elements,
        source: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let bindings = self.shared.bindings(
            context.graphics,
            params,
            elements,
            GlassTextures {
                surface: source,
                scene: source,
                original: source,
            },
        );
        let mut pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("liquid lens region"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..6, 0..1);
    }

    fn prepare_lens(
        &self,
        context: &mut DrawPreparationContext<'_, LiquidLensCommand>,
    ) -> Prepared {
        let command = context.command.clone();
        let scale = command.pixels_per_point;
        let origin = context.geometry.window_origin();
        let target = [context.target_size.width.0, context.target_size.height.0];
        let reach = REACH * scale;
        let element = command.element;
        let x0 = ((origin.x + element.x * scale - reach).floor() as i32).max(0);
        let y0 = ((origin.y + element.y * scale - reach).floor() as i32).max(0);
        let x1 =
            ((origin.x + (element.x + element.width) * scale + reach).ceil() as i32).min(target[0]);
        let y1 = ((origin.y + (element.y + element.height) * scale + reach).ceil() as i32)
            .min(target[1]);
        let size = [(x1 - x0).max(1), (y1 - y0).max(1)];
        let lease = |context: &mut DrawPreparationContext<'_, LiquidLensCommand>, usage| {
            context.graphics.scratch_texture(ScratchTextureDescriptor {
                width: size[0] as u32,
                height: size[1] as u32,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | usage,
                sample_count: 1,
                fit: ScratchFit::Exact,
            })
        };
        let original = lease(context, wgpu::TextureUsages::STORAGE_BINDING);
        let displaced = lease(context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let body = lease(context, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let scene = [context.scene_rect.x.0, context.scene_rect.y.0];
        self.shared.dispatch(
            context.graphics,
            context.encoder,
            &self.copy,
            &BackdropParams {
                region: IVec4::new(x0, y0, 0, 0),
                capture: IVec4::ZERO,
                scene: IVec4::new(scene[0], scene[1], 0, 0),
                levels: IVec4::ZERO,
                sizes: IVec4::new(0, 0, size[0], size[1]),
            },
            context.scene_texture_view,
            &original,
            size,
        );

        let glass = command.glass();
        let mut params = params::material_params(&glass);
        params.origin_scale = Vec4::new(origin.x, origin.y, scale, command.opacity);
        params.target_scene = Vec4::new(target[0] as f32, target[1] as f32, x0 as f32, y0 as f32);
        params.draw_rect = Vec4::new(x0 as f32, y0 as f32, x1 as f32, y1 as f32);
        params.surface = Vec4::new(0.0, 0.0, 1.0, 0.0);
        params.levels[0] = Vec4::new(0.0, 0.0, size[0] as f32, size[1] as f32);
        params.sampling = Vec4::new(scale, 0.0, 0.0, 0.0);
        params.lens_displacement = Vec4::new(31.0, 1.0, 7.75 * scale, 0.0);
        params.lens_aberration = Vec4::new(1.631_579, f32::MAX, 17.222_22, 0.0);
        let angle = -std::f32::consts::PI / 12.0;
        params.lens_directions = Vec4::new(angle.cos(), angle.sin(), 1.0, 0.0);
        params.lens_edge = Vec4::new(-6.2, 0.0, 1.0, 0.0);
        let elements = params::element_data(&glass);
        self.region_pass(
            context,
            &self.displace,
            &params,
            &elements,
            &original,
            &displaced,
        );
        self.region_pass(
            context,
            &self.background,
            &params,
            &elements,
            &displaced,
            &body,
        );
        Prepared {
            original,
            body,
            params,
        }
    }
}

impl DrawablePipeline<LiquidLensCommand> for LiquidLensPipeline {
    fn prepare(&mut self, context: &mut DrawPreparationContext<'_, LiquidLensCommand>) {
        let prepared = self.prepare_lens(context);
        self.prepared
            .insert(key(context.command, context.geometry), prepared);
    }

    fn end_frame(&mut self, _: &FrameContext<'_>) {
        self.prepared.clear();
    }

    fn draw(&mut self, context: &mut DrawContext<LiquidLensCommand>) {
        for &(command, geometry) in context.commands {
            let Some(prepared) = self.prepared.get(&key(command, geometry)) else {
                continue;
            };
            let bindings = self.shared.bindings(
                context.graphics,
                &prepared.params,
                &params::element_data(&command.glass()),
                GlassTextures {
                    surface: &prepared.body,
                    scene: &prepared.body,
                    original: &prepared.original,
                },
            );
            context.render_pass.set_pipeline(&self.lens);
            context.render_pass.set_bind_group(0, &bindings, &[]);
            context.render_pass.draw(0..6, 0..1);
        }
    }
}
