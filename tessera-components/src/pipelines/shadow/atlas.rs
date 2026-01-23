//! Shadow atlas composite pipeline for batched mask and blur passes.
//!
//! ## Usage
//!
//! Expand shadow composite commands into atlas-backed mask, blur, and draw ops.

use std::collections::HashMap;

use smallvec::SmallVec;
use tessera_ui::{
    Command, CompositeBatchItem, CompositeCommand, CompositeContext, CompositeOutput,
    CompositePipeline, DrawCommand, Px, PxPosition, PxSize, RenderGraphOp, RenderResource,
    RenderResourceId, RenderTextureDesc, composite::CompositeReplacement, wgpu,
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
pub struct ShadowAtlasPipeline;

impl ShadowAtlasPipeline {
    /// Creates a new shadow atlas pipeline instance.
    pub fn new() -> Self {
        Self
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

            let id = shadows.len();
            shadows.push(ShadowItem {
                id,
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

        let mask_items = build_mask_items(&shadows);
        let atlas_layout = match layout_mask_atlas(&mask_items, context.frame_size.width) {
            Some(layout) => layout,
            None => return build_fallback_output(&shadows),
        };

        let mut output = CompositeOutput::empty();
        let mask_atlas_id = add_atlas_resource(&mut output.resources, atlas_layout.size);
        let mut blur_atlases: Vec<[RenderResourceId; 2]> = Vec::with_capacity(layer_count);
        for _ in 0..layer_count {
            let a = add_atlas_resource(&mut output.resources, atlas_layout.size);
            let b = add_atlas_resource(&mut output.resources, atlas_layout.size);
            blur_atlases.push([a, b]);
        }

        let mut blur_ops_by_layer: Vec<Vec<Vec<RenderGraphOp>>> = vec![Vec::new(); layer_count];

        for mask_item in &mask_items {
            let item = &shadows[mask_item.index];
            let atlas_pos = atlas_layout
                .positions
                .get(&item.id)
                .copied()
                .unwrap_or(PxPosition::ZERO);

            let mut mask_command = ShadowMaskCommand::new(item.shape);
            mask_command.apply_opacity(item.opacity);
            output.prelude_ops.push(build_op(
                Command::Draw(Box::new(mask_command)),
                std::any::TypeId::of::<ShadowMaskCommand>(),
                None,
                Some(mask_atlas_id),
                item.size,
                atlas_pos + item.pad_pos,
                item.opacity,
            ));

            for (layer_index, layer_info) in item.layers.iter().enumerate() {
                for (stage, radius) in layer_info.radii.iter().copied().enumerate() {
                    if blur_ops_by_layer[layer_index].len() <= stage {
                        blur_ops_by_layer[layer_index].push(Vec::new());
                    }
                    let blur_command = DualBlurCommand::horizontal_then_vertical(radius);
                    let read_resource = if stage == 0 {
                        mask_atlas_id
                    } else {
                        blur_atlases[layer_index][(stage - 1) % 2]
                    };
                    let write_resource = blur_atlases[layer_index][stage % 2];
                    blur_ops_by_layer[layer_index][stage].push(build_op(
                        Command::Compute(Box::new(blur_command)),
                        std::any::TypeId::of::<DualBlurCommand>(),
                        Some(read_resource),
                        Some(write_resource),
                        item.mask_size,
                        atlas_pos,
                        item.opacity,
                    ));
                }
            }
        }

        for layer_ops in blur_ops_by_layer {
            for stage_ops in layer_ops {
                output.prelude_ops.extend(stage_ops);
            }
        }

        for item in &shadows {
            let mut ops: Vec<RenderGraphOp> = Vec::new();
            let atlas_pos = atlas_layout
                .positions
                .get(&item.id)
                .copied()
                .unwrap_or(PxPosition::ZERO);
            for (layer_index, layer_info) in item.layers.iter().enumerate() {
                if layer_info.radii.is_empty() {
                    continue;
                }
                let final_resource =
                    blur_atlases[layer_index][(layer_info.radii.len().saturating_sub(1)) % 2];
                let offset = PxPosition::new(
                    Px::from_f32(layer_info.layer.offset[0]),
                    Px::from_f32(layer_info.layer.offset[1]),
                );
                let composite_offset = offset - item.pad_pos;
                let ordering_offset = item.pad_pos - offset;

                let uv_origin = [
                    atlas_pos.x.to_f32() / atlas_layout.size.width.to_f32(),
                    atlas_pos.y.to_f32() / atlas_layout.size.height.to_f32(),
                ];
                let uv_size = [
                    item.mask_size.width.to_f32() / atlas_layout.size.width.to_f32(),
                    item.mask_size.height.to_f32() / atlas_layout.size.height.to_f32(),
                ];

                let mut composite = ShadowCompositeCommand::new(layer_info.layer.color)
                    .with_ordering(ordering_offset, item.size);
                composite.uv_origin = uv_origin;
                composite.uv_size = uv_size;
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

        output
    }
}

#[derive(Clone)]
struct ShadowLayerInfo {
    layer: ShadowLayer,
    radii: SmallVec<[f32; 4]>,
}

struct ShadowItem {
    id: usize,
    op_index: usize,
    position: PxPosition,
    size: PxSize,
    opacity: f32,
    shape: ResolvedShape,
    pad_pos: PxPosition,
    mask_size: PxSize,
    layers: Vec<ShadowLayerInfo>,
}

impl ShadowItem {
    fn index_key(&self) -> usize {
        self.id
    }
}

#[derive(Clone, Copy)]
struct MaskAtlasItem {
    index: usize,
    size: PxSize,
    guard: Px,
}

struct MaskAtlasLayout {
    size: PxSize,
    positions: HashMap<usize, PxPosition>,
}

fn build_mask_items(items: &[ShadowItem]) -> Vec<MaskAtlasItem> {
    let mut mask_items: Vec<MaskAtlasItem> = items
        .iter()
        .map(|item| {
            let draw_size = item.size;
            let region_size = item.mask_size;
            let guard_x = ((region_size.width - draw_size.width).max(Px::ZERO)).0 / 2;
            let guard_y = ((region_size.height - draw_size.height).max(Px::ZERO)).0 / 2;
            let guard = Px::new(guard_x.max(guard_y));
            MaskAtlasItem {
                index: item.index_key(),
                size: region_size,
                guard,
            }
        })
        .collect();
    mask_items.sort_by_key(|item| (-(item.size.height.0), -(item.size.width.0)));
    mask_items
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

fn layout_mask_atlas(items: &[MaskAtlasItem], max_width: Px) -> Option<MaskAtlasLayout> {
    if max_width.0 <= 0 {
        return None;
    }
    let mut positions: HashMap<usize, PxPosition> = HashMap::new();
    let mut x = Px::ZERO;
    let mut y = Px::ZERO;
    let mut row_height = Px::ZERO;
    let mut atlas_width = Px::ZERO;
    let mut atlas_height = Px::ZERO;

    for item in items {
        let guard = item.guard.max(Px::ZERO);
        let slot_width = item.size.width + guard + guard;
        let slot_height = item.size.height + guard + guard;
        if x > Px::ZERO && x + slot_width > max_width {
            y += row_height;
            x = Px::ZERO;
            row_height = Px::ZERO;
        }

        let inner_pos = PxPosition::new(x + guard, y + guard);
        positions.insert(item.index, inner_pos);

        x += slot_width;
        row_height = row_height.max(slot_height);
        atlas_width = atlas_width.max(x);
        atlas_height = atlas_height.max(y + row_height);
    }

    if atlas_width.0 <= 0 || atlas_height.0 <= 0 {
        return None;
    }

    Some(MaskAtlasLayout {
        size: PxSize::new(atlas_width, atlas_height),
        positions,
    })
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
            let ordering_offset = item.pad_pos - offset;

            let mut composite = ShadowCompositeCommand::new(layer_info.layer.color)
                .with_ordering(ordering_offset, item.size);
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
