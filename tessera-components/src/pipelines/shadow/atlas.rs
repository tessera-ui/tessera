//! Shadow atlas middleware for batched mask and blur passes.
//!
//! ## Usage
//!
//! Batch shadow masks and blurs into shared atlases before compositing.

use std::collections::{HashMap, HashSet};

use tessera_ui::{
    Command, Px, PxPosition, PxSize, RenderGraph, RenderGraphOp, RenderGraphParts,
    RenderMiddleware, RenderMiddlewareContext, RenderResource, RenderResourceId, RenderTextureDesc,
    dyn_eq_compute::DynPartialEqCompute, wgpu,
};

use crate::pipelines::{
    blur::command::DualBlurCommand,
    shadow::command::{ShadowCompositeCommand, ShadowMaskCommand},
};

/// Render middleware that packs shadow masks into atlases for fewer passes.
pub struct ShadowAtlasMiddleware;

impl ShadowAtlasMiddleware {
    /// Creates a shadow atlas middleware instance.
    pub fn new() -> Self {
        Self
    }
}

impl RenderMiddleware for ShadowAtlasMiddleware {
    fn name(&self) -> &'static str {
        "ShadowAtlas"
    }

    fn process(&mut self, scene: RenderGraph, context: &RenderMiddlewareContext) -> RenderGraph {
        let RenderGraphParts {
            mut ops,
            mut resources,
        } = scene.into_parts();
        if !apply_shadow_atlas(&mut ops, &mut resources, context.frame_size) {
            return RenderGraph::from_parts(RenderGraphParts { ops, resources });
        }
        RenderGraph::from_parts(RenderGraphParts { ops, resources })
    }
}

#[derive(Clone, Copy)]
struct MaskAtlasItem {
    mask_op: usize,
    size: PxSize,
    guard: Px,
}

struct MaskAtlasLayout {
    size: PxSize,
    positions: HashMap<usize, PxPosition>,
}

#[derive(Clone)]
struct ShadowChain {
    mask_op: usize,
    blur_ops: Vec<usize>,
    composite_op: usize,
    layer_index: usize,
}

fn apply_shadow_atlas(
    ops: &mut Vec<RenderGraphOp>,
    resources: &mut Vec<RenderResource>,
    frame_size: PxSize,
) -> bool {
    let mut mask_ops: Vec<usize> = Vec::new();
    let mut blur_ops: HashSet<usize> = HashSet::new();
    let mut composite_ops: Vec<usize> = Vec::new();

    for (index, op) in ops.iter().enumerate() {
        if is_shadow_mask(op) {
            mask_ops.push(index);
        } else if is_shadow_composite(op) {
            composite_ops.push(index);
        } else if is_shadow_blur(op) {
            blur_ops.insert(index);
        }
    }

    if mask_ops.is_empty() || composite_ops.is_empty() {
        return false;
    }

    let mut chains: Vec<ShadowChain> = Vec::new();
    let mut layer_counts: HashMap<usize, usize> = HashMap::new();
    let mut mask_draw_sizes: HashMap<usize, PxSize> = HashMap::new();
    let mut mask_draw_positions: HashMap<usize, PxPosition> = HashMap::new();
    let mut mask_region_sizes: HashMap<usize, PxSize> = HashMap::new();

    for mask_op in mask_ops.iter().copied() {
        if let Some(op) = ops.get(mask_op) {
            mask_draw_sizes.insert(mask_op, op.size);
            mask_draw_positions.insert(mask_op, op.position);
        }
    }

    composite_ops.sort();
    for composite_op in composite_ops.iter().copied() {
        let Some((mask_op, blur_chain)) = trace_blur_chain(ops, &blur_ops, &mask_ops, composite_op)
        else {
            continue;
        };
        if let Some(first_blur) = blur_chain.first().copied()
            && let Some(op) = ops.get(first_blur)
        {
            let entry = mask_region_sizes.entry(mask_op).or_insert(op.size);
            entry.width = entry.width.max(op.size.width);
            entry.height = entry.height.max(op.size.height);
        }

        let layer_index = layer_counts.entry(mask_op).or_insert(0);
        let current_layer = *layer_index;
        *layer_index = (*layer_index).saturating_add(1);

        chains.push(ShadowChain {
            mask_op,
            blur_ops: blur_chain,
            composite_op,
            layer_index: current_layer,
        });
    }

    if chains.is_empty() {
        return false;
    }

    let max_layer_index = chains
        .iter()
        .map(|chain| chain.layer_index)
        .max()
        .unwrap_or(0);
    let layer_count = max_layer_index + 1;
    if layer_count > 2 {
        return false;
    }

    let mut mask_in_chains: HashSet<usize> = HashSet::new();
    for chain in &chains {
        mask_in_chains.insert(chain.mask_op);
    }

    let mut mask_items: Vec<MaskAtlasItem> = mask_ops
        .iter()
        .filter(|index| mask_in_chains.contains(index))
        .filter_map(|index| {
            let region_size = mask_region_sizes.get(index).copied()?;
            let draw_size = mask_draw_sizes.get(index).copied().unwrap_or(region_size);
            let guard_x = ((region_size.width - draw_size.width).max(Px::ZERO)).0 / 2;
            let guard_y = ((region_size.height - draw_size.height).max(Px::ZERO)).0 / 2;
            let guard = Px::new(guard_x.max(guard_y));
            Some(MaskAtlasItem {
                mask_op: *index,
                size: region_size,
                guard,
            })
        })
        .collect();
    mask_items.sort_by_key(|item| (-(item.size.height.0), -(item.size.width.0)));

    let atlas_layout = match layout_mask_atlas(&mask_items, frame_size.width) {
        Some(layout) => layout,
        None => return false,
    };

    let mask_atlas_id = add_atlas_resource(resources, atlas_layout.size);
    let mut blur_atlases: Vec<[RenderResourceId; 2]> = Vec::with_capacity(layer_count);
    for _ in 0..layer_count {
        let a = add_atlas_resource(resources, atlas_layout.size);
        let b = add_atlas_resource(resources, atlas_layout.size);
        blur_atlases.push([a, b]);
    }

    for item in &mask_items {
        let Some(pos) = atlas_layout.positions.get(&item.mask_op).copied() else {
            continue;
        };
        let op = &mut ops[item.mask_op];
        let draw_pos = mask_draw_positions
            .get(&item.mask_op)
            .copied()
            .unwrap_or(PxPosition::ZERO);
        let draw_size = mask_draw_sizes
            .get(&item.mask_op)
            .copied()
            .unwrap_or(item.size);
        op.read = None;
        op.write = Some(mask_atlas_id);
        op.position = pos + draw_pos;
        op.size = draw_size;
    }

    for chain in &chains {
        for (stage, blur_index) in chain.blur_ops.iter().enumerate() {
            let read_resource = if stage == 0 {
                mask_atlas_id
            } else {
                blur_atlases[chain.layer_index][(stage - 1) % 2]
            };
            let write_resource = blur_atlases[chain.layer_index][stage % 2];
            let op = &mut ops[*blur_index];
            op.read = Some(read_resource);
            op.write = Some(write_resource);
            if let Some(pos) = atlas_layout.positions.get(&chain.mask_op).copied() {
                op.position = pos + op.position;
            }
        }
    }

    for chain in &chains {
        let final_resource = blur_atlases[chain.layer_index][(chain.blur_ops.len() - 1) % 2];
        let op = &mut ops[chain.composite_op];
        op.read = Some(final_resource);

        let Some(pos) = atlas_layout.positions.get(&chain.mask_op).copied() else {
            continue;
        };
        let Some(size) = mask_region_sizes.get(&chain.mask_op).copied() else {
            continue;
        };
        let uv_origin = [
            pos.x.to_f32() / atlas_layout.size.width.to_f32(),
            pos.y.to_f32() / atlas_layout.size.height.to_f32(),
        ];
        let uv_size = [
            size.width.to_f32() / atlas_layout.size.width.to_f32(),
            size.height.to_f32() / atlas_layout.size.height.to_f32(),
        ];
        if let Command::Draw(command) = &op.command
            && let Some(shadow) = command.as_any().downcast_ref::<ShadowCompositeCommand>()
        {
            let mut updated = *shadow;
            updated.uv_origin = uv_origin;
            updated.uv_size = uv_size;
            op.command = Command::Draw(Box::new(updated));
        }
    }

    let mut max_stage_per_layer = vec![0usize; layer_count];
    for chain in &chains {
        max_stage_per_layer[chain.layer_index] =
            max_stage_per_layer[chain.layer_index].max(chain.blur_ops.len());
    }

    let mut moved: HashSet<usize> = HashSet::new();
    let mut new_order: Vec<usize> = Vec::with_capacity(ops.len());
    let mut mask_order: Vec<usize> = mask_items.iter().map(|item| item.mask_op).collect();
    mask_order.sort();
    for index in mask_order {
        moved.insert(index);
        new_order.push(index);
    }

    for (layer, stage_count) in max_stage_per_layer.iter().enumerate().take(layer_count) {
        for stage in 0..*stage_count {
            let mut stage_ops: Vec<usize> = Vec::new();
            for chain in chains.iter().filter(|chain| chain.layer_index == layer) {
                if stage < chain.blur_ops.len() {
                    stage_ops.push(chain.blur_ops[stage]);
                }
            }
            stage_ops.sort();
            for index in stage_ops {
                if moved.insert(index) {
                    new_order.push(index);
                }
            }
        }
    }

    for index in 0..ops.len() {
        if moved.insert(index) {
            new_order.push(index);
        }
    }

    *ops = reorder_ops(ops, &new_order);
    compact_resources(ops, resources);
    true
}

fn is_shadow_mask(op: &RenderGraphOp) -> bool {
    matches!(
        &op.command,
        Command::Draw(command) if command.as_any().is::<ShadowMaskCommand>()
    )
}

fn is_shadow_composite(op: &RenderGraphOp) -> bool {
    matches!(
        &op.command,
        Command::Draw(command) if command.as_any().is::<ShadowCompositeCommand>()
    )
}

fn is_shadow_blur(op: &RenderGraphOp) -> bool {
    matches!(
        &op.command,
        Command::Compute(command)
            if DynPartialEqCompute::as_any(&**command).is::<DualBlurCommand>()
    )
}

fn trace_blur_chain(
    ops: &[RenderGraphOp],
    blur_ops: &HashSet<usize>,
    mask_ops: &[usize],
    composite_op: usize,
) -> Option<(usize, Vec<usize>)> {
    let mask_set: HashSet<usize> = mask_ops.iter().copied().collect();
    let mut chain: Vec<usize> = Vec::new();
    let mut current = composite_op;
    let mut guard = ops.len();

    while guard > 0 {
        guard -= 1;
        let deps = ops.get(current)?.deps.clone();
        let mut next_blur = None;
        for dep in deps.iter().copied() {
            if blur_ops.contains(&dep) {
                next_blur = Some(dep);
                break;
            }
        }
        let blur_index = next_blur?;
        chain.push(blur_index);
        current = blur_index;

        let deps = ops.get(current)?.deps.clone();
        for dep in deps.iter().copied() {
            if mask_set.contains(&dep) {
                chain.reverse();
                return Some((dep, chain));
            }
        }
    }
    None
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
        positions.insert(item.mask_op, inner_pos);

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

fn reorder_ops(ops: &mut Vec<RenderGraphOp>, order: &[usize]) -> Vec<RenderGraphOp> {
    let mut indexed: Vec<Option<RenderGraphOp>> = ops.drain(..).map(Some).collect();
    let mut remap = vec![0usize; order.len()];
    for (new_index, old_index) in order.iter().enumerate() {
        remap[*old_index] = new_index;
    }

    let mut new_ops: Vec<RenderGraphOp> = Vec::with_capacity(order.len());
    for old_index in order.iter().copied() {
        let mut op = indexed[old_index]
            .take()
            .unwrap_or_else(|| panic!("missing op for index {old_index}"));
        for dep in op.deps.iter_mut() {
            *dep = remap
                .get(*dep)
                .copied()
                .unwrap_or_else(|| panic!("missing remap for dep {dep}"));
        }
        new_ops.push(op);
    }

    for (index, op) in new_ops.iter_mut().enumerate() {
        op.sequence_index = index;
    }

    new_ops
}

fn compact_resources(ops: &mut [RenderGraphOp], resources: &mut Vec<RenderResource>) {
    let mut used = vec![false; resources.len()];
    for op in ops.iter() {
        mark_resources(op.read, &mut used);
        mark_resources(op.write, &mut used);
    }

    let mut remap: Vec<Option<u32>> = vec![None; resources.len()];
    let mut new_resources: Vec<RenderResource> = Vec::new();
    for (index, resource) in resources.iter().enumerate() {
        if used[index] {
            let new_index = new_resources.len() as u32;
            remap[index] = Some(new_index);
            new_resources.push(resource.clone());
        }
    }

    for op in ops.iter_mut() {
        remap_resource(&mut op.read, &remap);
        remap_resource(&mut op.write, &remap);
    }

    *resources = new_resources;
}

fn mark_resources(resource: Option<RenderResourceId>, used: &mut [bool]) {
    if let Some(RenderResourceId::Local(index)) = resource
        && let Some(entry) = used.get_mut(index as usize)
    {
        *entry = true;
    }
}

fn remap_resource(resource: &mut Option<RenderResourceId>, remap: &[Option<u32>]) {
    if let Some(RenderResourceId::Local(index)) = resource.as_mut() {
        let new_index = remap
            .get(*index as usize)
            .and_then(|value| *value)
            .unwrap_or(*index);
        *index = new_index;
    }
}
