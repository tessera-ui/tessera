//! Render pass planning for Tessera frames.
//!
//! ## Usage
//!
//! Build render pass plans from ordered render graph ops.

use smallvec::SmallVec;

use crate::{
    Command, ComputeCommand, DrawCommand, DrawRegion, Px, PxPosition, PxRect, PxSize, SampleRegion,
    render_graph::{RenderGraphOp, RenderResource, RenderResourceId},
};

/// A pass planning result for the current frame.
pub(crate) struct RenderPassGraph {
    passes: Vec<RenderPassPlan>,
}

impl RenderPassGraph {
    /// Builds a render pass graph from ordered ops.
    pub(crate) fn build(
        ops: Vec<RenderGraphOp>,
        resources: &[RenderResource],
        texture_size: wgpu::Extent3d,
    ) -> Self {
        let mut passes = Vec::new();
        let mut draw_builder = DrawPassBuilder::new();
        let mut compute_builder = ComputePassBuilder::new();

        for op in ops {
            let RenderGraphOp {
                command,
                size,
                position,
                type_id,
                read,
                write,
                ..
            } = op;
            let write_resource = write.unwrap_or(RenderResourceId::SceneColor);
            let read_resource = read;

            match command {
                Command::Draw(cmd) => {
                    flush_compute_pass(&mut passes, &mut compute_builder);

                    let read_resource = read_resource.or_else(|| {
                        cmd.sample_region()
                            .is_some()
                            .then_some(RenderResourceId::SceneColor)
                    });
                    let target_extent = resource_extent(write_resource, resources, texture_size);
                    let read_extent = read_resource
                        .map(|resource| resource_extent(resource, resources, texture_size))
                        .unwrap_or(target_extent);
                    let needs_flush = draw_builder.ensure_resources(read_resource, write_resource);
                    if needs_flush {
                        flush_draw_pass(&mut passes, &mut draw_builder);
                        draw_builder.ensure_resources(read_resource, write_resource);
                    }

                    let reads_scene = read_resource == Some(RenderResourceId::SceneColor);
                    let sampling_rect = if reads_scene {
                        cmd.sample_region().map(|region| {
                            extract_sampling_rect(Some(region), op.size, op.position, read_extent)
                        })
                    } else {
                        None
                    };
                    let requires_barrier =
                        read_resource == Some(write_resource) && sampling_rect.is_some();
                    let need_new_pass = should_start_new_pass(
                        draw_builder.last_draw(),
                        requires_barrier,
                        sampling_rect,
                        &draw_builder.sampling_rects,
                    );
                    if need_new_pass {
                        flush_draw_pass(&mut passes, &mut draw_builder);
                        draw_builder.ensure_resources(read_resource, write_resource);
                    }

                    draw_builder.push_draw(DrawCommandInput {
                        command: cmd,
                        type_id,
                        size,
                        start_pos: position,
                        texture_size: target_extent,
                        sampling_rect,
                        read_resource,
                    });
                }
                Command::Compute(cmd) => {
                    flush_draw_pass(&mut passes, &mut draw_builder);

                    let read_resource = read_resource.unwrap_or(RenderResourceId::SceneColor);
                    let read_extent = resource_extent(read_resource, resources, texture_size);
                    let write_extent = resource_extent(write_resource, resources, texture_size);
                    let needs_flush =
                        compute_builder.ensure_resources(read_resource, write_resource);
                    if needs_flush {
                        flush_compute_pass(&mut passes, &mut compute_builder);
                        compute_builder.ensure_resources(read_resource, write_resource);
                    }

                    let sampling_rect =
                        extract_sampling_rect(Some(cmd.barrier()), size, position, read_extent);
                    let target_rect = extract_target_rect(size, position, write_extent);
                    compute_builder.push_compute(ComputePlanItem {
                        command: cmd,
                        size,
                        start_pos: position,
                        target_rect,
                        sampling_rect,
                    });
                }
                Command::Composite(_) => {
                    panic!("Composite commands must be expanded before render pass planning");
                }
                Command::ClipPush(rect) => {
                    flush_compute_pass(&mut passes, &mut compute_builder);
                    draw_builder.push_clip(ClipOps::Push(rect));
                }
                Command::ClipPop => {
                    flush_compute_pass(&mut passes, &mut compute_builder);
                    draw_builder.push_clip(ClipOps::Pop);
                }
            }
        }

        flush_draw_pass(&mut passes, &mut draw_builder);
        flush_compute_pass(&mut passes, &mut compute_builder);

        Self { passes }
    }

    /// Returns the render passes in submission order.
    pub(crate) fn into_passes(self) -> Vec<RenderPassPlan> {
        self.passes
    }
}

/// A render pass plan with draw commands and optional compute work.
pub(crate) struct RenderPassPlan {
    pub(crate) kind: RenderPassKind,
    pub(crate) draws: SmallVec<[DrawOrClip; 32]>,
    pub(crate) compute: Vec<ComputePlanItem>,
    pub(crate) copy_rect: Option<PxRect>,
    pub(crate) read_resource: Option<RenderResourceId>,
    pub(crate) write_resource: RenderResourceId,
}

/// Identifies the pass type.
pub(crate) enum RenderPassKind {
    Draw,
    Compute,
}

struct DrawPassBuilder {
    draws: SmallVec<[DrawOrClip; 32]>,
    sampling_rects: SmallVec<[PxRect; 16]>,
    write_resource: Option<RenderResourceId>,
    reads_scene: bool,
}

struct DrawCommandInput {
    command: Box<dyn DrawCommand>,
    type_id: std::any::TypeId,
    size: PxSize,
    start_pos: PxPosition,
    texture_size: wgpu::Extent3d,
    sampling_rect: Option<PxRect>,
    read_resource: Option<RenderResourceId>,
}

impl DrawPassBuilder {
    fn new() -> Self {
        Self {
            draws: SmallVec::new(),
            sampling_rects: SmallVec::new(),
            write_resource: None,
            reads_scene: false,
        }
    }

    fn ensure_resources(
        &mut self,
        read_resource: Option<RenderResourceId>,
        write_resource: RenderResourceId,
    ) -> bool {
        let write_changed =
            self.write_resource.is_some() && self.write_resource != Some(write_resource);
        if write_changed {
            return true;
        }

        let reads_scene = read_resource == Some(RenderResourceId::SceneColor);
        if self.reads_scene != reads_scene && !self.draws.is_empty() {
            return true;
        }
        self.reads_scene |= reads_scene;

        if self.write_resource.is_none() {
            self.write_resource = Some(write_resource);
        }
        false
    }

    fn last_draw(&self) -> Option<&DrawCommandWithMetadata> {
        self.draws.iter().rev().find_map(|command| match command {
            DrawOrClip::Draw(cmd) => Some(cmd),
            DrawOrClip::Clip(_) => None,
        })
    }

    fn push_clip(&mut self, clip: ClipOps) {
        self.draws.push(DrawOrClip::Clip(clip));
    }

    fn push_draw(&mut self, input: DrawCommandInput) {
        if let Some(rect) = input.sampling_rect {
            self.sampling_rects.push(rect);
        }
        let draw_rect = extract_draw_rect(
            input.command.draw_region(),
            input.size,
            input.start_pos,
            input.texture_size,
        );
        self.draws.push(DrawOrClip::Draw(DrawCommandWithMetadata {
            command: input.command,
            type_id: input.type_id,
            size: input.size,
            start_pos: input.start_pos,
            draw_rect,
            read_resource: input.read_resource,
        }));
    }

    fn finish(&mut self) -> Option<RenderPassPlan> {
        if self.draws.is_empty() {
            return None;
        }

        let copy_rect = if self.reads_scene {
            union_rects(&self.sampling_rects)
        } else {
            None
        };
        Some(RenderPassPlan {
            kind: RenderPassKind::Draw,
            draws: std::mem::take(&mut self.draws),
            compute: Vec::new(),
            copy_rect,
            read_resource: self.reads_scene.then_some(RenderResourceId::SceneColor),
            write_resource: self.write_resource.unwrap_or(RenderResourceId::SceneColor),
        })
    }
}

/// Compute work required for a render pass.
pub(crate) struct ComputePlanItem {
    pub(crate) command: Box<dyn ComputeCommand>,
    pub(crate) size: PxSize,
    pub(crate) start_pos: PxPosition,
    pub(crate) target_rect: PxRect,
    pub(crate) sampling_rect: PxRect,
}

/// Metadata for draw commands submitted in a pass.
pub(crate) struct DrawCommandWithMetadata {
    pub(crate) command: Box<dyn DrawCommand>,
    pub(crate) type_id: std::any::TypeId,
    pub(crate) size: PxSize,
    pub(crate) start_pos: PxPosition,
    pub(crate) draw_rect: PxRect,
    pub(crate) read_resource: Option<RenderResourceId>,
}

/// Draw or clip operations inside a pass.
pub(crate) enum DrawOrClip {
    Draw(DrawCommandWithMetadata),
    Clip(ClipOps),
}

/// Clip stack operations for a render pass.
pub(crate) enum ClipOps {
    Push(PxRect),
    Pop,
}

struct ComputePassBuilder {
    items: Vec<ComputePlanItem>,
    sampling_rects: SmallVec<[PxRect; 16]>,
    read_resource: Option<RenderResourceId>,
    write_resource: Option<RenderResourceId>,
}

impl ComputePassBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            sampling_rects: SmallVec::new(),
            read_resource: None,
            write_resource: None,
        }
    }

    fn ensure_resources(
        &mut self,
        read_resource: RenderResourceId,
        write_resource: RenderResourceId,
    ) -> bool {
        let write_changed =
            self.write_resource.is_some() && self.write_resource != Some(write_resource);
        let read_changed =
            self.read_resource.is_some() && self.read_resource != Some(read_resource);
        if write_changed || read_changed {
            return true;
        }
        if self.read_resource.is_none() {
            self.read_resource = Some(read_resource);
        }
        if self.write_resource.is_none() {
            self.write_resource = Some(write_resource);
        }
        false
    }

    fn push_compute(&mut self, item: ComputePlanItem) {
        self.sampling_rects.push(item.sampling_rect);
        self.items.push(item);
    }
}

fn compute_padded_rect(
    size: PxSize,
    start_pos: PxPosition,
    top: Px,
    right: Px,
    bottom: Px,
    left: Px,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    let padded_x = (start_pos.x - left).max(Px(0));
    let padded_y = (start_pos.y - top).max(Px(0));
    let padded_width = (size.width + left + right).min(Px(texture_size.width as i32 - padded_x.0));
    let padded_height =
        (size.height + top + bottom).min(Px(texture_size.height as i32 - padded_y.0));
    PxRect {
        x: padded_x,
        y: padded_y,
        width: padded_width,
        height: padded_height,
    }
}

fn clamp_rect_to_texture(mut rect: PxRect, texture_size: wgpu::Extent3d) -> PxRect {
    rect.x = rect.x.positive().min(texture_size.width).into();
    rect.y = rect.y.positive().min(texture_size.height).into();
    rect.width = rect
        .width
        .positive()
        .min(texture_size.width - rect.x.positive())
        .into();
    rect.height = rect
        .height
        .positive()
        .min(texture_size.height - rect.y.positive())
        .into();
    rect
}

fn extract_sampling_rect(
    barrier: Option<SampleRegion>,
    size: PxSize,
    start_pos: PxPosition,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    match barrier {
        Some(SampleRegion::Global) => PxRect {
            x: Px(0),
            y: Px(0),
            width: Px(texture_size.width as i32),
            height: Px(texture_size.height as i32),
        },
        Some(SampleRegion::PaddedLocal(sampling)) => compute_padded_rect(
            size,
            start_pos,
            sampling.top,
            sampling.right,
            sampling.bottom,
            sampling.left,
            texture_size,
        ),
        Some(SampleRegion::Absolute(rect)) => clamp_rect_to_texture(rect, texture_size),
        None => extract_target_rect(size, start_pos, texture_size),
    }
}

fn extract_draw_rect(
    region: DrawRegion,
    size: PxSize,
    start_pos: PxPosition,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    match region {
        DrawRegion::Global => PxRect {
            x: Px(0),
            y: Px(0),
            width: Px(texture_size.width as i32),
            height: Px(texture_size.height as i32),
        },
        DrawRegion::PaddedLocal(padding) => compute_padded_rect(
            size,
            start_pos,
            padding.top,
            padding.right,
            padding.bottom,
            padding.left,
            texture_size,
        ),
        DrawRegion::Absolute(rect) => clamp_rect_to_texture(rect, texture_size),
    }
}

fn extract_target_rect(
    size: PxSize,
    start_pos: PxPosition,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    let x = start_pos.x.positive().min(texture_size.width);
    let y = start_pos.y.positive().min(texture_size.height);
    let width = size.width.positive().min(texture_size.width - x);
    let height = size.height.positive().min(texture_size.height - y);
    PxRect {
        x: Px::from(x),
        y: Px::from(y),
        width: Px::from(width),
        height: Px::from(height),
    }
}

fn should_start_new_pass(
    last_draw: Option<&DrawCommandWithMetadata>,
    next_requires_barrier: bool,
    next_sampling_rect: Option<PxRect>,
    sampling_rects_in_pass: &[PxRect],
) -> bool {
    let Some(last_draw) = last_draw else {
        return false;
    };

    let last_requires_barrier = last_draw.command.sample_region().is_some();
    match (last_requires_barrier, next_requires_barrier) {
        (false, true) => true,
        (true, true) => {
            let Some(next_sampling_rect) = next_sampling_rect else {
                return false;
            };
            !sampling_rects_in_pass
                .iter()
                .all(|rect| rect.is_orthogonal(&next_sampling_rect))
        }
        _ => false,
    }
}

fn union_rects(rects: &[PxRect]) -> Option<PxRect> {
    let mut iter = rects.iter().copied();
    let mut combined = iter.next()?;
    for rect in iter {
        combined = combined.union(&rect);
    }
    Some(combined)
}

fn flush_draw_pass(passes: &mut Vec<RenderPassPlan>, builder: &mut DrawPassBuilder) {
    if let Some(pass) = builder.finish() {
        passes.push(pass);
    }
    builder.write_resource = None;
    builder.sampling_rects.clear();
    builder.reads_scene = false;
}

fn flush_compute_pass(passes: &mut Vec<RenderPassPlan>, builder: &mut ComputePassBuilder) {
    if builder.items.is_empty() {
        return;
    }
    let copy_rect = if builder.read_resource == builder.write_resource
        || builder.read_resource == Some(RenderResourceId::SceneColor)
    {
        union_rects(&builder.sampling_rects)
    } else {
        None
    };
    passes.push(RenderPassPlan {
        kind: RenderPassKind::Compute,
        draws: SmallVec::new(),
        compute: std::mem::take(&mut builder.items),
        copy_rect,
        read_resource: builder.read_resource,
        write_resource: builder
            .write_resource
            .unwrap_or(RenderResourceId::SceneColor),
    });
    builder.sampling_rects.clear();
    builder.read_resource = None;
    builder.write_resource = None;
}

fn resource_extent(
    id: RenderResourceId,
    resources: &[RenderResource],
    surface_size: wgpu::Extent3d,
) -> wgpu::Extent3d {
    match id {
        RenderResourceId::SceneColor | RenderResourceId::SceneDepth => surface_size,
        RenderResourceId::Local(index) => match resources.get(index as usize) {
            Some(RenderResource::Texture(desc)) => extent_from_px(desc.size),
            None => surface_size,
        },
    }
}

fn extent_from_px(size: PxSize) -> wgpu::Extent3d {
    let width = size.width.positive().max(1);
    let height = size.height.positive().max(1);
    wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    }
}
