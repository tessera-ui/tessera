//! Render pass planning

use smallvec::SmallVec;

use crate::{
    Command, ComputeCommand, DrawCommand, DrawRegion, Px, PxPosition, PxRect, PxSize, SampleRegion,
    render_graph::{ExternalTextureDesc, RenderGraphOp, RenderResource, RenderResourceId},
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
        external_resources: &[ExternalTextureDesc],
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
                    let target_extent = resource_extent(
                        write_resource,
                        resources,
                        external_resources,
                        texture_size,
                    );
                    let read_extent = read_resource
                        .map(|resource| {
                            resource_extent(resource, resources, external_resources, texture_size)
                        })
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
                    let read_extent =
                        resource_extent(read_resource, resources, external_resources, texture_size);
                    let write_extent = resource_extent(
                        write_resource,
                        resources,
                        external_resources,
                        texture_size,
                    );
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
                Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop => {
                    flush_compute_pass(&mut passes, &mut compute_builder);
                    if draw_builder
                        .write_resource
                        .is_some_and(|resource| resource != RenderResourceId::SceneColor)
                    {
                        flush_draw_pass(&mut passes, &mut draw_builder);
                    }
                    draw_builder.write_resource = Some(RenderResourceId::SceneColor);
                    let clip = match command {
                        Command::ClipPush(rect) => ClipOps::Push(rect),
                        Command::ShapeClipPush(region) => ClipOps::ShapePush { region, index: 0 },
                        Command::ClipPop => ClipOps::Pop,
                        _ => unreachable!(),
                    };
                    draw_builder.push_clip(clip);
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

        Some(RenderPassPlan {
            kind: RenderPassKind::Draw,
            draws: std::mem::take(&mut self.draws),
            compute: Vec::new(),
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
    ShapePush {
        region: crate::ClipRegion,
        index: u32,
    },
    Pop,
}

struct ComputePassBuilder {
    items: Vec<ComputePlanItem>,
    read_resource: Option<RenderResourceId>,
    write_resource: Option<RenderResourceId>,
}

impl ComputePassBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
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
    let x = (start_pos.x - left.to_f32())
        .floor()
        .clamp(0.0, texture_size.width as f32);
    let y = (start_pos.y - top.to_f32())
        .floor()
        .clamp(0.0, texture_size.height as f32);
    let end_x = (start_pos.x + size.width.to_f32() + right.to_f32())
        .ceil()
        .clamp(0.0, texture_size.width as f32);
    let end_y = (start_pos.y + size.height.to_f32() + bottom.to_f32())
        .ceil()
        .clamp(0.0, texture_size.height as f32);
    PxRect::new(
        Px(x as i32),
        Px(y as i32),
        Px((end_x - x).max(0.0) as i32),
        Px((end_y - y).max(0.0) as i32),
    )
}

fn clamp_rect_to_texture(rect: PxRect, texture_size: wgpu::Extent3d) -> PxRect {
    rect.intersection(&PxRect::new(
        Px::ZERO,
        Px::ZERO,
        Px(texture_size.width as i32),
        Px(texture_size.height as i32),
    ))
    .unwrap_or(PxRect::ZERO)
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
    compute_padded_rect(
        size,
        start_pos,
        Px::ZERO,
        Px::ZERO,
        Px::ZERO,
        Px::ZERO,
        texture_size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct ProbeDraw(bool);

    impl DrawCommand for ProbeDraw {
        fn sample_region(&self) -> Option<SampleRegion> {
            self.0
                .then_some(SampleRegion::PaddedLocal(crate::PaddingRect::ZERO))
        }
        fn apply_opacity(&mut self, _: f32) {}
    }

    fn op(command: Command, sequence_index: usize, x: i32) -> RenderGraphOp {
        RenderGraphOp {
            command,
            type_id: std::any::TypeId::of::<ProbeDraw>(),
            read: None,
            write: Some(RenderResourceId::SceneColor),
            deps: SmallVec::new(),
            size: PxSize::new(Px(10), Px(10)),
            position: PxPosition::new(Px(x), Px(0)),
            opacity: 1.0,
            sequence_index,
        }
    }

    #[test]
    fn clip_state_preserves_sampled_draw_batching() {
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        let ops = vec![
            op(Command::Draw(Box::new(ProbeDraw(true))), 0, 0),
            op(
                Command::ClipPush(PxRect::new(Px(0), Px(0), Px(100), Px(100))),
                1,
                0,
            ),
            op(Command::ClipPop, 2, 0),
            op(Command::Draw(Box::new(ProbeDraw(true))), 3, 50),
        ];
        let passes = RenderPassGraph::build(ops, &[], &[], target).into_passes();
        assert_eq!(
            passes.len(),
            1,
            "clip state must not split nonoverlapping scene samples"
        );
        assert_eq!(passes[0].draws.len(), 4);
    }

    #[test]
    fn clip_state_after_local_work_targets_the_scene() {
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        let mut local = op(Command::Draw(Box::new(ProbeDraw(false))), 0, 0);
        local.write = Some(RenderResourceId::Local(0));
        let passes = RenderPassGraph::build(
            vec![
                local,
                op(
                    Command::ClipPush(PxRect::new(Px(0), Px(0), Px(10), Px(10))),
                    1,
                    0,
                ),
                op(Command::ClipPop, 2, 0),
            ],
            &[RenderResource::Texture(
                crate::render_graph::RenderTextureDesc {
                    size: PxSize::new(Px(10), Px(10)),
                    format: wgpu::TextureFormat::Rgba8Unorm,
                },
            )],
            &[],
            target,
        )
        .into_passes();
        assert_eq!(passes.len(), 2);
        assert_eq!(passes[0].write_resource, RenderResourceId::Local(0));
        assert_eq!(passes[1].write_resource, RenderResourceId::SceneColor);
        assert_eq!(passes[1].draws.len(), 2);
    }

    #[test]
    fn fractional_draw_and_sample_bounds_round_outwards() {
        let size = PxSize::new(Px(20), Px(10));
        let position = PxPosition::from_f32_xy(10.25, 10.75);
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        let expected = PxRect::new(Px(10), Px(10), Px(21), Px(11));
        assert_eq!(
            extract_draw_rect(
                DrawRegion::PaddedLocal(crate::PaddingRect::ZERO),
                size,
                position,
                target
            ),
            expected
        );
        assert_eq!(
            extract_sampling_rect(
                Some(SampleRegion::PaddedLocal(crate::PaddingRect::ZERO)),
                size,
                position,
                target
            ),
            expected
        );
        assert_eq!(extract_target_rect(size, position, target), expected);
    }

    #[test]
    fn negative_bounds_do_not_extend_the_opposite_edge() {
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        assert_eq!(
            extract_target_rect(
                PxSize::new(Px(20), Px(10)),
                PxPosition::from_f32_xy(-3.25, -2.5),
                target
            ),
            PxRect::new(Px(0), Px(0), Px(17), Px(8))
        );
        assert_eq!(
            clamp_rect_to_texture(PxRect::new(Px(-5), Px(-2), Px(20), Px(10)), target),
            PxRect::new(Px(0), Px(0), Px(15), Px(8))
        );
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
    passes.push(RenderPassPlan {
        kind: RenderPassKind::Compute,
        draws: SmallVec::new(),
        compute: std::mem::take(&mut builder.items),
        read_resource: builder.read_resource,
        write_resource: builder
            .write_resource
            .unwrap_or(RenderResourceId::SceneColor),
    });
    builder.read_resource = None;
    builder.write_resource = None;
}

fn resource_extent(
    id: RenderResourceId,
    resources: &[RenderResource],
    external_resources: &[ExternalTextureDesc],
    surface_size: wgpu::Extent3d,
) -> wgpu::Extent3d {
    match id {
        RenderResourceId::SceneColor | RenderResourceId::SceneDepth => surface_size,
        RenderResourceId::Local(index) => match resources.get(index as usize) {
            Some(RenderResource::Texture(desc)) => extent_from_px(desc.size),
            None => surface_size,
        },
        RenderResourceId::External(index) => match external_resources.get(index as usize) {
            Some(desc) => extent_from_px(desc.size),
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
