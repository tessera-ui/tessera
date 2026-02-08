//! Shadow atlas composite pipeline for batched mask and blur passes.
//!
//! ## Usage
//!
//! Expand shadow composite commands into atlas-backed mask, blur, and draw ops.

use std::collections::{HashMap, HashSet};

use smallvec::SmallVec;
use tessera_ui::{
    Color, Command, CompositeBatchItem, CompositeCommand, CompositeContext, CompositeOutput,
    CompositePipeline, DrawCommand, Px, PxPosition, PxSize, RenderGraphOp, RenderResource,
    RenderResourceId, RenderTextureDesc,
    composite::CompositeReplacement,
    renderer::{ExternalTextureHandle, ExternalTextureRegistry},
    wgpu,
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
    stamp_heat: HashMap<ShadowStampKey, ShadowHeatEntry>,
    atlas: ShadowAtlas,
}

impl ShadowAtlasPipeline {
    /// Creates a new shadow atlas pipeline instance.
    pub fn new() -> Self {
        Self {
            stamp_cache: HashMap::new(),
            stamp_heat: HashMap::new(),
            atlas: ShadowAtlas::new(),
        }
    }

    fn evict_unused_entries(&mut self, frame_index: u64) {
        let mut evicted: Vec<ShadowStampKey> = Vec::new();
        for (key, entry) in &self.stamp_cache {
            if frame_index
                > entry
                    .last_used_frame
                    .saturating_add(SHADOW_STAMP_EVICT_FRAMES)
            {
                evicted.push(key.clone());
            }
        }
        for key in evicted {
            if let Some(entry) = self.stamp_cache.remove(&key) {
                self.atlas.free(entry.page_index, entry.alloc_rect);
            }
        }

        self.stamp_heat.retain(|_, entry| {
            frame_index
                <= entry
                    .last_seen_frame
                    .saturating_add(SHADOW_ATLAS_HEAT_EVICT_FRAMES)
        });

        let mut active_pages: HashSet<usize> = HashSet::new();
        for entry in self.stamp_cache.values() {
            active_pages.insert(entry.page_index);
        }
        self.atlas
            .collect_garbage(&active_pages, frame_index, SHADOW_ATLAS_PAGE_EVICT_FRAMES);
    }

    fn should_promote_to_atlas(&mut self, key: &ShadowStampKey, frame_index: u64) -> bool {
        let entry = self
            .stamp_heat
            .entry(key.clone())
            .or_insert_with(|| ShadowHeatEntry {
                count: 0,
                last_seen_frame: frame_index.wrapping_sub(1),
            });
        if entry.last_seen_frame != frame_index {
            // Avoid promoting mid-frame when the same key appears multiple times.
            entry.count = entry.count.saturating_add(1);
            entry.last_seen_frame = frame_index;
        }
        entry.count >= SHADOW_ATLAS_HEAT_THRESHOLD
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

        if self.atlas.ensure_sample_count(sample_count) {
            self.stamp_cache.clear();
            self.stamp_heat.clear();
        }
        self.atlas.ensure_limits(device);

        let mut key_counts: HashMap<ShadowStampKey, usize> = HashMap::new();
        for item in &shadows {
            let key = build_stamp_key(item, sample_count);
            *key_counts.entry(key).or_insert(0) += 1;
        }

        let mut use_atlas_by_key: HashMap<ShadowStampKey, bool> =
            HashMap::with_capacity(key_counts.len());
        for (key, count) in &key_counts {
            let use_atlas = self.stamp_cache.contains_key(key)
                || *count > 1
                || self.should_promote_to_atlas(key, frame_index);
            use_atlas_by_key.insert(key.clone(), use_atlas);
        }

        for item in &shadows {
            let key = build_stamp_key(item, sample_count);
            if !*use_atlas_by_key.get(&key).unwrap_or(&false) {
                emit_fallback_item(&mut output, item);
                continue;
            }
            let mut needs_rebuild = false;
            if !self.stamp_cache.contains_key(&key) {
                let guard = guard_padding_for_layers(&item.layers);
                let mask_width = item.mask_size.width.positive();
                let mask_height = item.mask_size.height.positive();
                let alloc_width = mask_width.saturating_add(guard.saturating_mul(2));
                let alloc_height = mask_height.saturating_add(guard.saturating_mul(2));
                let allocation = match self.atlas.allocate(
                    registry,
                    device,
                    alloc_width,
                    alloc_height,
                    sample_count,
                    frame_index,
                ) {
                    Some(allocation) => allocation,
                    None => {
                        self.stamp_cache.clear();
                        self.stamp_heat.clear();
                        self.atlas.reset();
                        return build_fallback_output(&shadows);
                    }
                };
                let inner_rect = allocation.rect.inset(guard);
                let stamp = ShadowStampEntry::new(
                    allocation.page_index,
                    allocation.rect,
                    inner_rect,
                    frame_index,
                );
                self.stamp_cache.insert(key.clone(), stamp);
                needs_rebuild = true;
            }
            let entry = self
                .stamp_cache
                .get_mut(&key)
                .expect("shadow stamp cache entry should exist");
            self.stamp_heat.remove(&key);

            entry.last_used_frame = frame_index;
            self.atlas.mark_page_used(entry.page_index, frame_index);
            if entry.last_built_frame == frame_index {
                needs_rebuild = false;
            }

            let page = self.atlas.page(entry.page_index);
            let mut blur_resources: Vec<[RenderResourceId; 2]> =
                Vec::with_capacity(item.layers.len());
            for layer_index in 0..item.layers.len() {
                let a = output.add_external_texture(page.blur[layer_index][0].desc(false));
                let b = output.add_external_texture(page.blur[layer_index][1].desc(false));
                blur_resources.push([a, b]);
            }

            let alloc_origin = entry.alloc_rect.to_px_position();
            let alloc_size = entry.alloc_rect.to_px_size();
            let inner_origin = entry.inner_rect.to_px_position();
            let inner_size = entry.inner_rect.to_px_size();
            let uv_origin = [
                entry.inner_rect.x as f32 / page.size.width.to_f32(),
                entry.inner_rect.y as f32 / page.size.height.to_f32(),
            ];
            let uv_size = [
                entry.inner_rect.width as f32 / page.size.width.to_f32(),
                entry.inner_rect.height as f32 / page.size.height.to_f32(),
            ];

            if needs_rebuild {
                entry.last_built_frame = frame_index;
                let mask_id = output.add_external_texture(page.mask.desc(false));

                let mut clear_command = ShadowMaskCommand::new(clear_rect_shape());
                clear_command.color = Color::TRANSPARENT;
                output.prelude_ops.push(build_op(
                    Command::Draw(Box::new(clear_command)),
                    std::any::TypeId::of::<ShadowMaskCommand>(),
                    None,
                    Some(mask_id),
                    alloc_size,
                    alloc_origin,
                    1.0,
                ));

                let mask_command = ShadowMaskCommand::new(item.shape);
                output.prelude_ops.push(build_op(
                    Command::Draw(Box::new(mask_command)),
                    std::any::TypeId::of::<ShadowMaskCommand>(),
                    None,
                    Some(mask_id),
                    item.size,
                    inner_origin + item.pad_pos,
                    1.0,
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
                            inner_size,
                            inner_origin,
                            1.0,
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

        self.evict_unused_entries(frame_index);

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
    layers: Vec<ShadowLayerKey>,
    sample_count: u32,
}

fn build_stamp_key(item: &ShadowItem, sample_count: u32) -> ShadowStampKey {
    ShadowStampKey {
        shape: ResolvedShapeKey::from_shape(item.shape),
        size: item.size,
        mask_size: item.mask_size,
        pad_pos: item.pad_pos,
        layers: item
            .layers
            .iter()
            .map(|layer| ShadowLayerKey::new(layer.layer))
            .collect(),
        sample_count,
    }
}

struct ShadowStampEntry {
    page_index: usize,
    alloc_rect: AtlasRect,
    inner_rect: AtlasRect,
    last_used_frame: u64,
    last_built_frame: u64,
}

impl ShadowStampEntry {
    fn new(
        page_index: usize,
        alloc_rect: AtlasRect,
        inner_rect: AtlasRect,
        frame_index: u64,
    ) -> Self {
        Self {
            page_index,
            alloc_rect,
            inner_rect,
            last_used_frame: frame_index,
            last_built_frame: frame_index.wrapping_sub(1),
        }
    }
}

struct ShadowHeatEntry {
    count: u8,
    last_seen_frame: u64,
}

#[derive(Clone, Copy)]
struct AtlasRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl AtlasRect {
    fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    fn fits(self, width: u32, height: u32) -> bool {
        width <= self.width && height <= self.height
    }

    fn right(self) -> u32 {
        self.x + self.width
    }

    fn bottom(self) -> u32 {
        self.y + self.height
    }

    fn inset(self, padding: u32) -> Self {
        let pad = padding.saturating_mul(2);
        Self {
            x: self.x + padding,
            y: self.y + padding,
            width: self.width.saturating_sub(pad),
            height: self.height.saturating_sub(pad),
        }
    }

    fn to_px_position(self) -> PxPosition {
        PxPosition::new(Px::new(self.x as i32), Px::new(self.y as i32))
    }

    fn to_px_size(self) -> PxSize {
        PxSize::new(Px::new(self.width as i32), Px::new(self.height as i32))
    }

    fn merge(self, other: Self) -> Option<Self> {
        if self.x == other.x && self.width == other.width {
            if self.bottom() == other.y {
                return Some(Self {
                    x: self.x,
                    y: self.y,
                    width: self.width,
                    height: self.height + other.height,
                });
            }
            if other.bottom() == self.y {
                return Some(Self {
                    x: self.x,
                    y: other.y,
                    width: self.width,
                    height: self.height + other.height,
                });
            }
        }
        if self.y == other.y && self.height == other.height {
            if self.right() == other.x {
                return Some(Self {
                    x: self.x,
                    y: self.y,
                    width: self.width + other.width,
                    height: self.height,
                });
            }
            if other.right() == self.x {
                return Some(Self {
                    x: other.x,
                    y: self.y,
                    width: self.width + other.width,
                    height: self.height,
                });
            }
        }
        None
    }
}

struct AtlasAllocation {
    page_index: usize,
    rect: AtlasRect,
}

struct ShadowAtlasPage {
    size: PxSize,
    mask: ExternalTextureHandle,
    blur: [[ExternalTextureHandle; 2]; SHADOW_ATLAS_LAYER_CAPACITY],
    free_rects: Vec<AtlasRect>,
    last_used_frame: u64,
}

impl ShadowAtlasPage {
    fn new(
        registry: &ExternalTextureRegistry,
        device: &wgpu::Device,
        size: u32,
        sample_count: u32,
        frame_index: u64,
    ) -> Self {
        let size_px = PxSize::new(Px::new(size as i32), Px::new(size as i32));
        let desc = RenderTextureDesc {
            size: size_px,
            format: wgpu::TextureFormat::Rgba8Unorm,
        };
        let mask = registry.allocate(device, desc.clone(), sample_count);
        let blur = std::array::from_fn(|_| {
            let a = registry.allocate(device, desc.clone(), sample_count);
            let b = registry.allocate(device, desc.clone(), sample_count);
            [a, b]
        });
        Self {
            size: size_px,
            mask,
            blur,
            free_rects: vec![AtlasRect {
                x: 0,
                y: 0,
                width: size,
                height: size,
            }],
            last_used_frame: frame_index,
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<AtlasRect> {
        if width == 0 || height == 0 {
            return None;
        }
        let mut best_index = None;
        let mut best_area = u64::MAX;
        for (index, rect) in self.free_rects.iter().enumerate() {
            if rect.fits(width, height) {
                let area = rect.area();
                if area < best_area {
                    best_area = area;
                    best_index = Some(index);
                }
            }
        }
        let index = best_index?;
        let rect = self.free_rects.swap_remove(index);
        let allocated = AtlasRect {
            x: rect.x,
            y: rect.y,
            width,
            height,
        };

        let remaining_width = rect.width.saturating_sub(width);
        let remaining_height = rect.height.saturating_sub(height);
        if remaining_width > 0 {
            self.free_rects.push(AtlasRect {
                x: rect.x + width,
                y: rect.y,
                width: remaining_width,
                height,
            });
        }
        if remaining_height > 0 {
            self.free_rects.push(AtlasRect {
                x: rect.x,
                y: rect.y + height,
                width: rect.width,
                height: remaining_height,
            });
        }

        Some(allocated)
    }

    fn free(&mut self, rect: AtlasRect) {
        self.free_rects.push(rect);
        self.merge_free_rects();
    }

    fn merge_free_rects(&mut self) {
        let mut merged = true;
        while merged {
            merged = false;
            let mut i = 0;
            while i < self.free_rects.len() {
                let mut j = i + 1;
                while j < self.free_rects.len() {
                    if let Some(rect) = self.free_rects[i].merge(self.free_rects[j]) {
                        self.free_rects[i] = rect;
                        self.free_rects.swap_remove(j);
                        merged = true;
                        break;
                    }
                    j += 1;
                }
                if merged {
                    break;
                }
                i += 1;
            }
        }
    }
}

struct ShadowAtlas {
    pages: Vec<Option<ShadowAtlasPage>>,
    default_size: u32,
    max_dimension: u32,
    sample_count: Option<u32>,
}

impl ShadowAtlas {
    fn new() -> Self {
        Self {
            pages: Vec::new(),
            default_size: SHADOW_ATLAS_DEFAULT_SIZE,
            max_dimension: 0,
            sample_count: None,
        }
    }

    fn ensure_limits(&mut self, device: &wgpu::Device) {
        if self.max_dimension == 0 {
            let max_dimension = device.limits().max_texture_dimension_2d;
            self.max_dimension = max_dimension;
            self.default_size = self
                .default_size
                .min(max_dimension)
                .max(SHADOW_ATLAS_MIN_SIZE);
        }
    }

    fn ensure_sample_count(&mut self, sample_count: u32) -> bool {
        if self.sample_count != Some(sample_count) {
            self.sample_count = Some(sample_count);
            self.pages.clear();
            return true;
        }
        false
    }

    fn reset(&mut self) {
        self.pages.clear();
    }

    fn allocate(
        &mut self,
        registry: &ExternalTextureRegistry,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        frame_index: u64,
    ) -> Option<AtlasAllocation> {
        self.ensure_limits(device);

        if width == 0 || height == 0 {
            return None;
        }

        if width > self.max_dimension || height > self.max_dimension {
            return None;
        }

        for (index, page) in self.pages.iter_mut().enumerate() {
            let Some(page) = page.as_mut() else {
                continue;
            };
            if let Some(rect) = page.allocate(width, height) {
                return Some(AtlasAllocation {
                    page_index: index,
                    rect,
                });
            }
        }

        let needed = width.max(height).max(SHADOW_ATLAS_MIN_SIZE);
        let mut page_size = self.default_size.max(needed);
        page_size = page_size.min(self.max_dimension);
        let page_index = self.add_page(registry, device, page_size, sample_count, frame_index);
        let rect = self
            .pages
            .get_mut(page_index)
            .and_then(|page| page.as_mut())
            .and_then(|page| page.allocate(width, height))
            .expect("allocation should fit in a new shadow atlas page");
        Some(AtlasAllocation { page_index, rect })
    }

    fn add_page(
        &mut self,
        registry: &ExternalTextureRegistry,
        device: &wgpu::Device,
        size: u32,
        sample_count: u32,
        frame_index: u64,
    ) -> usize {
        let page = ShadowAtlasPage::new(registry, device, size, sample_count, frame_index);
        if let Some((index, slot)) = self
            .pages
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(page);
            return index;
        }
        self.pages.push(Some(page));
        self.pages.len() - 1
    }

    fn page(&self, index: usize) -> &ShadowAtlasPage {
        self.pages[index]
            .as_ref()
            .expect("shadow atlas page missing")
    }

    fn free(&mut self, page_index: usize, rect: AtlasRect) {
        if let Some(Some(page)) = self.pages.get_mut(page_index) {
            page.free(rect);
        }
    }

    fn mark_page_used(&mut self, page_index: usize, frame_index: u64) {
        if let Some(Some(page)) = self.pages.get_mut(page_index) {
            page.last_used_frame = frame_index;
        }
    }

    fn collect_garbage(
        &mut self,
        active_pages: &HashSet<usize>,
        frame_index: u64,
        delay_frames: u64,
    ) {
        for (index, slot) in self.pages.iter_mut().enumerate() {
            let evict = match slot.as_ref() {
                None => false,
                Some(page) => {
                    if active_pages.contains(&index) {
                        false
                    } else {
                        frame_index > page.last_used_frame.saturating_add(delay_frames)
                    }
                }
            };
            if evict {
                *slot = None;
            }
        }
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

const SHADOW_ATLAS_DEFAULT_SIZE: u32 = 2048;
const SHADOW_ATLAS_MIN_SIZE: u32 = 256;
const SHADOW_ATLAS_GUARD_PX: u32 = 1;
const SHADOW_ATLAS_LAYER_CAPACITY: usize = 2;
const SHADOW_ATLAS_PAGE_EVICT_FRAMES: u64 = 2;
const SHADOW_ATLAS_HEAT_THRESHOLD: u8 = 2;
const SHADOW_ATLAS_HEAT_EVICT_FRAMES: u64 = 120;
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

fn guard_padding_for_layers(layers: &[ShadowLayerInfo]) -> u32 {
    let max_radius = layers
        .iter()
        .map(|layer| layer.layer.smoothness)
        .fold(0.0f32, f32::max);
    let scale = downscale_factor_for_radius(max_radius).max(1) as u32;
    SHADOW_ATLAS_GUARD_PX.saturating_mul(scale).max(1)
}

fn clear_rect_shape() -> ResolvedShape {
    ResolvedShape::Rounded {
        corner_radii: [0.0; 4],
        corner_g2: [3.0; 4],
    }
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

fn emit_fallback_item(output: &mut CompositeOutput, item: &ShadowItem) {
    let mask_id = add_atlas_resource(&mut output.resources, item.mask_size);
    let mut blur_ids: Vec<RenderResourceId> = Vec::with_capacity(item.layers.len());
    for _ in &item.layers {
        blur_ids.push(add_atlas_resource(&mut output.resources, item.mask_size));
    }

    let mask_command = ShadowMaskCommand::new(item.shape);
    output.prelude_ops.push(build_op(
        Command::Draw(Box::new(mask_command)),
        std::any::TypeId::of::<ShadowMaskCommand>(),
        None,
        Some(mask_id),
        item.size,
        item.pad_pos,
        1.0,
    ));

    let mut replacement_ops: Vec<RenderGraphOp> = Vec::new();

    for (layer_index, layer_info) in item.layers.iter().enumerate() {
        let blur_id = blur_ids[layer_index];
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
                1.0,
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

fn build_fallback_output(items: &[ShadowItem]) -> CompositeOutput {
    let mut output = CompositeOutput::empty();
    for item in items {
        emit_fallback_item(&mut output, item);
    }
    output
}
