//! Shadow atlas composite pipeline for batched mask and blur passes.
//!
//! ## Usage
//!
//! Expand shadow composite commands into atlas-backed mask, blur, and draw ops.

use std::collections::{HashMap, hash_map::Entry};

use smallvec::SmallVec;
use tessera_ui::{
    Color, Command, CompositeBatchItem, CompositeCommand, CompositeContext, CompositeOutput,
    CompositePipeline, DrawCommand, Px, PxPosition, PxSize, RenderGraphOp, RenderResource,
    RenderResourceId, RenderTextureDesc, composite::CompositeReplacement,
    renderer::ExternalTextureHandle, wgpu,
};

use crate::{
    pipelines::{
        blur::command::{DualBlurCommand, downscale_factor_for_radius},
        shadow::command::{ShadowCompositeCommand, ShadowMaskCommand},
    },
    shadow::{ShadowLayer, ShadowLayers},
    shape_def::ResolvedShape,
};

/// Composite command describing a shadow atlas expansion.
#[derive(Debug, Clone)]
pub struct ShadowAtlasCommand {
    /// Resolved shape geometry for the mask.
    pub shape: ResolvedShape,
    /// Shadow layers to render.
    pub layers: ShadowLayers,
}

impl ShadowAtlasCommand {
    /// Creates a shadow atlas command for the resolved shape and layers.
    pub fn new(shape: ResolvedShape, layers: ShadowLayers) -> Self {
        Self { shape, layers }
    }
}

impl CompositeCommand for ShadowAtlasCommand {}

/// Composite pipeline that expands shadow atlas commands.
pub struct ShadowAtlasPipeline {
    stamp_cache: HashMap<ShadowStampKey, ShadowStampEntry>,
}

impl ShadowAtlasPipeline {
    /// Creates a new shadow atlas pipeline instance.
    pub fn new() -> Self {
        Self {
            stamp_cache: HashMap::new(),
        }
    }
}

impl CompositePipeline<ShadowAtlasCommand> for ShadowAtlasPipeline {
    fn compile(
        &mut self,
        context: &CompositeContext<'_>,
        items: &[CompositeBatchItem<'_, ShadowAtlasCommand>],
    ) -> CompositeOutput {
        if items.is_empty() {
            return CompositeOutput::empty();
        }

        let mut shadows: Vec<ShadowItem> = Vec::new();
        for item in items {
            let layers = filter_active_layers(item.command.layers);
            if layers.is_empty() {
                continue;
            }

            let (pad_x, pad_y) = shadow_padding_xy(&layers);
            let pad_pos = PxPosition::new(pad_x, pad_y);
            let mask_size = PxSize::new(
                item.size.width + pad_x + pad_x,
                item.size.height + pad_y + pad_y,
            );
            if mask_size.width.0 <= 0 || mask_size.height.0 <= 0 {
                continue;
            }

            let layer_infos = layers
                .into_iter()
                .map(|layer| ShadowLayerInfo {
                    layer,
                    radii: blur_pass_radii(layer.smoothness),
                })
                .collect();

            shadows.push(ShadowItem {
                op_index: item.op_index,
                position: item.position,
                size: item.size,
                opacity: item.opacity,
                shape: item.command.shape,
                pad_pos,
                mask_size,
                layers: layer_infos,
            });
        }

        if shadows.is_empty() {
            return CompositeOutput::empty();
        }

        let layer_count = shadows
            .iter()
            .map(|item| item.layers.len())
            .max()
            .unwrap_or(0);
        if layer_count == 0 || layer_count > 2 {
            return build_fallback_output(&shadows);
        }
        let mut output = CompositeOutput::empty();
        let device = context.resources.device;
        let registry = &context.external_textures;
        let sample_count = context.sample_count;
        let frame_index = context.frame_index;

        for item in &shadows {
            let key = build_stamp_key(item, sample_count);
            let desc = RenderTextureDesc {
                size: item.mask_size,
                format: wgpu::TextureFormat::Rgba8Unorm,
            };
            let (entry, mut needs_rebuild) = match self.stamp_cache.entry(key) {
                Entry::Vacant(slot) => {
                    let stamp = ShadowStampEntry::new(
                        registry,
                        device,
                        desc.clone(),
                        sample_count,
                        item.layers.len(),
                        frame_index,
                    );
                    (slot.insert(stamp), true)
                }
                Entry::Occupied(entry) => (entry.into_mut(), false),
            };

            entry.last_used_frame = frame_index;
            if entry.ensure(registry, device, &desc, sample_count) {
                needs_rebuild = true;
            }
            if entry.last_built_frame == frame_index {
                needs_rebuild = false;
            }

            let clear_on_first_use = needs_rebuild;
            let mut blur_resources: Vec<[RenderResourceId; 2]> =
                Vec::with_capacity(entry.blur.len());
            for handles in &mut entry.blur {
                let a = output.add_external_texture(handles[0].desc(clear_on_first_use));
                let b = output.add_external_texture(handles[1].desc(clear_on_first_use));
                blur_resources.push([a, b]);
            }

            if needs_rebuild {
                entry.last_built_frame = frame_index;
                let mask_id = output.add_external_texture(entry.mask.desc(true));
                let mut mask_command = ShadowMaskCommand::new(item.shape);
                mask_command.apply_opacity(item.opacity);
                output.prelude_ops.push(build_op(
                    Command::Draw(Box::new(mask_command)),
                    std::any::TypeId::of::<ShadowMaskCommand>(),
                    None,
                    Some(mask_id),
                    item.size,
                    item.pad_pos,
                    item.opacity,
                ));

                for (layer_index, layer_info) in item.layers.iter().enumerate() {
                    for (stage, radius) in layer_info.radii.iter().copied().enumerate() {
                        let blur_command = DualBlurCommand::horizontal_then_vertical(radius);
                        let read_resource = if stage == 0 {
                            mask_id
                        } else {
                            blur_resources[layer_index][(stage - 1) % 2]
                        };
                        let write_resource = blur_resources[layer_index][stage % 2];
                        output.prelude_ops.push(build_op(
                            Command::Compute(Box::new(blur_command)),
                            std::any::TypeId::of::<DualBlurCommand>(),
                            Some(read_resource),
                            Some(write_resource),
                            item.mask_size,
                            PxPosition::ZERO,
                            item.opacity,
                        ));
                    }
                }
            }

            let mut ops: Vec<RenderGraphOp> = Vec::new();
            for (layer_index, layer_info) in item.layers.iter().enumerate() {
                if layer_info.radii.is_empty() {
                    continue;
                }
                let final_resource =
                    blur_resources[layer_index][(layer_info.radii.len().saturating_sub(1)) % 2];
                let offset = PxPosition::new(
                    Px::from_f32(layer_info.layer.offset[0]),
                    Px::from_f32(layer_info.layer.offset[1]),
                );
                let composite_offset = offset - item.pad_pos;

                let mut composite = ShadowCompositeCommand::new(layer_info.layer.color)
                    .with_ordering(PxPosition::ZERO, item.mask_size);
                composite.uv_origin = [0.0, 0.0];
                composite.uv_size = [1.0, 1.0];
                composite.apply_opacity(item.opacity);

                ops.push(build_op(
                    Command::Draw(Box::new(composite)),
                    std::any::TypeId::of::<ShadowCompositeCommand>(),
                    Some(final_resource),
                    Some(RenderResourceId::SceneColor),
                    item.mask_size,
                    item.position + composite_offset,
                    item.opacity,
                ));
            }
            if !ops.is_empty() {
                output.replacements.push(CompositeReplacement {
                    target_op: item.op_index,
                    ops,
                });
            }
        }

        self.stamp_cache.retain(|_, entry| {
            frame_index
                <= entry
                    .last_used_frame
                    .saturating_add(SHADOW_STAMP_EVICT_FRAMES)
        });

        output
    }
}

#[derive(Clone)]
struct ShadowLayerInfo {
    layer: ShadowLayer,
    radii: SmallVec<[f32; 4]>,
}

struct ShadowItem {
    op_index: usize,
    position: PxPosition,
    size: PxSize,
    opacity: f32,
    shape: ResolvedShape,
    pad_pos: PxPosition,
    mask_size: PxSize,
    layers: Vec<ShadowLayerInfo>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ColorKey {
    r: u32,
    g: u32,
    b: u32,
    a: u32,
}

impl ColorKey {
    fn from_color(color: Color) -> Self {
        Self {
            r: color.r.to_bits(),
            g: color.g.to_bits(),
            b: color.b.to_bits(),
            a: color.a.to_bits(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum ResolvedShapeKey {
    Rounded {
        corner_radii: [u32; 4],
        corner_g2: [u32; 4],
    },
    Ellipse,
}

impl ResolvedShapeKey {
    fn from_shape(shape: ResolvedShape) -> Self {
        match shape {
            ResolvedShape::Rounded {
                corner_radii,
                corner_g2,
            } => ResolvedShapeKey::Rounded {
                corner_radii: corner_radii.map(f32::to_bits),
                corner_g2: corner_g2.map(f32::to_bits),
            },
            ResolvedShape::Ellipse => ResolvedShapeKey::Ellipse,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ShadowLayerKey {
    color: ColorKey,
    offset: [u32; 2],
    smoothness: u32,
}

impl ShadowLayerKey {
    fn new(layer: ShadowLayer) -> Self {
        Self {
            color: ColorKey::from_color(layer.color),
            offset: [layer.offset[0].to_bits(), layer.offset[1].to_bits()],
            smoothness: layer.smoothness.to_bits(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ShadowStampKey {
    shape: ResolvedShapeKey,
    size: PxSize,
    mask_size: PxSize,
    pad_pos: PxPosition,
    opacity: u32,
    layers: Vec<ShadowLayerKey>,
    sample_count: u32,
}

fn build_stamp_key(item: &ShadowItem, sample_count: u32) -> ShadowStampKey {
    ShadowStampKey {
        shape: ResolvedShapeKey::from_shape(item.shape),
        size: item.size,
        mask_size: item.mask_size,
        pad_pos: item.pad_pos,
        opacity: item.opacity.to_bits(),
        layers: item
            .layers
            .iter()
            .map(|layer| ShadowLayerKey::new(layer.layer))
            .collect(),
        sample_count,
    }
}

struct ShadowStampEntry {
    mask: ExternalTextureHandle,
    blur: Vec<[ExternalTextureHandle; 2]>,
    last_used_frame: u64,
    last_built_frame: u64,
}

impl ShadowStampEntry {
    fn new(
        registry: &tessera_ui::renderer::ExternalTextureRegistry,
        device: &wgpu::Device,
        desc: RenderTextureDesc,
        sample_count: u32,
        layer_count: usize,
        frame_index: u64,
    ) -> Self {
        let mask = registry.allocate(device, desc.clone(), sample_count);
        let mut blur = Vec::with_capacity(layer_count);
        for _ in 0..layer_count {
            let a = registry.allocate(device, desc.clone(), sample_count);
            let b = registry.allocate(device, desc.clone(), sample_count);
            blur.push([a, b]);
        }
        Self {
            mask,
            blur,
            last_used_frame: frame_index,
            last_built_frame: frame_index.wrapping_sub(1),
        }
    }

    fn ensure(
        &mut self,
        registry: &tessera_ui::renderer::ExternalTextureRegistry,
        device: &wgpu::Device,
        desc: &RenderTextureDesc,
        sample_count: u32,
    ) -> bool {
        let mut changed = false;
        if external_desc_changed(&self.mask, desc, sample_count) {
            changed = true;
        }
        self.mask
            .ensure(registry, device, desc.clone(), sample_count);
        for handles in &mut self.blur {
            if external_desc_changed(&handles[0], desc, sample_count)
                || external_desc_changed(&handles[1], desc, sample_count)
            {
                changed = true;
            }
            handles[0].ensure(registry, device, desc.clone(), sample_count);
            handles[1].ensure(registry, device, desc.clone(), sample_count);
        }
        changed
    }
}

fn filter_active_layers(shadow: ShadowLayers) -> Vec<ShadowLayer> {
    let mut layers = Vec::new();
    if let Some(layer) = shadow.ambient
        && layer.color.a > 0.0
        && layer.smoothness > 0.0
    {
        layers.push(layer);
    }
    if let Some(layer) = shadow.spot
        && layer.color.a > 0.0
        && layer.smoothness > 0.0
    {
        layers.push(layer);
    }
    layers
}

const SHADOW_AA_MARGIN_PX: f32 = 1.0;
const SHADOW_MAX_SINGLE_BLUR_RADIUS: f32 = 30.0;
const SHADOW_STAMP_EVICT_FRAMES: u64 = 120;

fn blur_pass_radii(radius: f32) -> SmallVec<[f32; 4]> {
    if radius <= 0.0 {
        return SmallVec::new();
    }

    let max_radius = SHADOW_MAX_SINGLE_BLUR_RADIUS.max(1.0);
    let ratio = radius / max_radius;
    let pass_count = (ratio * ratio).ceil().max(1.0) as u32;
    let step_radius = radius / (pass_count as f32).sqrt();

    let mut radii = SmallVec::with_capacity(pass_count as usize);
    for _ in 0..pass_count {
        radii.push(step_radius);
    }
    radii
}

fn shadow_padding_xy(layers: &[ShadowLayer]) -> (Px, Px) {
    let mut pad_x = 0.0f32;
    let mut pad_y = 0.0f32;

    let update = |pad_x: &mut f32, pad_y: &mut f32, layer: &ShadowLayer| {
        if layer.color.a <= 0.0 {
            return;
        }
        let scale = downscale_factor_for_radius(layer.smoothness) as f32;
        let blur_pad = (layer.smoothness * scale).max(0.0);
        let layer_pad_x = (blur_pad + layer.offset[0].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        let layer_pad_y = (blur_pad + layer.offset[1].abs() + SHADOW_AA_MARGIN_PX).max(0.0);
        *pad_x = (*pad_x).max(layer_pad_x);
        *pad_y = (*pad_y).max(layer_pad_y);
    };

    for layer in layers {
        update(&mut pad_x, &mut pad_y, layer);
    }

    (
        Px::new(pad_x.ceil() as i32).max(Px::ZERO),
        Px::new(pad_y.ceil() as i32).max(Px::ZERO),
    )
}

fn add_atlas_resource(resources: &mut Vec<RenderResource>, size: PxSize) -> RenderResourceId {
    let id = RenderResourceId::Local(resources.len() as u32);
    resources.push(RenderResource::Texture(RenderTextureDesc {
        size,
        format: wgpu::TextureFormat::Rgba8Unorm,
    }));
    id
}

fn build_op(
    command: Command,
    type_id: std::any::TypeId,
    read: Option<RenderResourceId>,
    write: Option<RenderResourceId>,
    size: PxSize,
    position: PxPosition,
    opacity: f32,
) -> RenderGraphOp {
    RenderGraphOp {
        command,
        type_id,
        read,
        write,
        deps: SmallVec::new(),
        size,
        position,
        opacity,
        sequence_index: 0,
    }
}

fn build_fallback_output(items: &[ShadowItem]) -> CompositeOutput {
    let mut output = CompositeOutput::empty();

    for item in items {
        let mask_id = add_atlas_resource(&mut output.resources, item.mask_size);
        let blur_id = add_atlas_resource(&mut output.resources, item.mask_size);

        let mut mask_command = ShadowMaskCommand::new(item.shape);
        mask_command.apply_opacity(item.opacity);
        output.prelude_ops.push(build_op(
            Command::Draw(Box::new(mask_command)),
            std::any::TypeId::of::<ShadowMaskCommand>(),
            None,
            Some(mask_id),
            item.size,
            item.pad_pos,
            item.opacity,
        ));

        let mut replacement_ops: Vec<RenderGraphOp> = Vec::new();

        for layer_info in &item.layers {
            let mut last_resource = None;
            for (index, radius) in layer_info.radii.iter().copied().enumerate() {
                let blur_command = DualBlurCommand::horizontal_then_vertical(radius);
                let read_resource = if index == 0 { mask_id } else { blur_id };
                let write_resource = blur_id;
                output.prelude_ops.push(build_op(
                    Command::Compute(Box::new(blur_command)),
                    std::any::TypeId::of::<DualBlurCommand>(),
                    Some(read_resource),
                    Some(write_resource),
                    item.mask_size,
                    PxPosition::ZERO,
                    item.opacity,
                ));
                last_resource = Some(write_resource);
            }

            let Some(final_resource) = last_resource else {
                continue;
            };

            let offset = PxPosition::new(
                Px::from_f32(layer_info.layer.offset[0]),
                Px::from_f32(layer_info.layer.offset[1]),
            );
            let composite_offset = offset - item.pad_pos;

            let mut composite = ShadowCompositeCommand::new(layer_info.layer.color)
                .with_ordering(PxPosition::ZERO, item.mask_size);
            composite.apply_opacity(item.opacity);

            replacement_ops.push(build_op(
                Command::Draw(Box::new(composite)),
                std::any::TypeId::of::<ShadowCompositeCommand>(),
                Some(final_resource),
                Some(RenderResourceId::SceneColor),
                item.mask_size,
                item.position + composite_offset,
                item.opacity,
            ));
        }

        if !replacement_ops.is_empty() {
            output.replacements.push(CompositeReplacement {
                target_op: item.op_index,
                ops: replacement_ops,
            });
        }
    }

    output
}

fn external_desc_changed(
    handle: &ExternalTextureHandle,
    desc: &RenderTextureDesc,
    sample_count: u32,
) -> bool {
    let current = handle.desc(false);
    current.size != desc.size
        || current.format != desc.format
        || current.sample_count != sample_count
}
