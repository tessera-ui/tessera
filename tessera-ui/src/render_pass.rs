//! Render pass planning

use smallvec::SmallVec;

use crate::{
    Command, ComputeCommand, DrawCommand, DrawRegion, Px, PxRect, SampleRegion,
    render_graph::{DrawGeometry, RenderGraphOp},
};

/// A pass planning result for the current frame.
pub(crate) struct RenderPassGraph {
    passes: Vec<RenderPassPlan>,
}

impl RenderPassGraph {
    /// Builds a render pass graph from ordered ops.
    pub(crate) fn build(ops: Vec<RenderGraphOp>, texture_size: wgpu::Extent3d) -> Self {
        let mut passes = Vec::new();
        let mut draw_builder = DrawPassBuilder::default();
        let mut compute_items = Vec::new();

        for op in ops {
            let RenderGraphOp {
                command,
                geometry,
                type_id,
                ..
            } = op;

            match command {
                Command::Draw(cmd) => {
                    flush_compute_pass(&mut passes, &mut compute_items);

                    let sampling_rect = cmd
                        .sample_region()
                        .map(|region| extract_sampling_rect(Some(region), geometry, texture_size));
                    if (draw_builder.reads_scene != sampling_rect.is_some()
                        && !draw_builder.draws.is_empty())
                        || should_start_new_pass(
                            draw_builder.last_draw(),
                            sampling_rect,
                            &draw_builder.sampling_rects,
                        )
                    {
                        flush_draw_pass(&mut passes, &mut draw_builder);
                    }
                    draw_builder.reads_scene |= sampling_rect.is_some();
                    if let Some(rect) = sampling_rect {
                        draw_builder.sampling_rects.push(rect);
                    }
                    let draw_rect = extract_draw_rect(cmd.draw_region(), geometry, texture_size);
                    draw_builder
                        .draws
                        .push(DrawOrClip::Draw(DrawCommandWithMetadata {
                            command: cmd,
                            type_id,
                            geometry,
                            draw_rect,
                        }));
                }
                Command::Compute(cmd) => {
                    flush_draw_pass(&mut passes, &mut draw_builder);
                    let sampling_rect =
                        extract_sampling_rect(Some(cmd.barrier()), geometry, texture_size);
                    let target_rect = extract_target_rect(geometry, texture_size);
                    compute_items.push(ComputePlanItem {
                        command: cmd,
                        geometry,
                        target_rect,
                        sampling_rect,
                    });
                }
                Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop => {
                    flush_compute_pass(&mut passes, &mut compute_items);
                    let clip = match command {
                        Command::ClipPush(rect) => ClipOps::Push(rect),
                        Command::ShapeClipPush(region) => ClipOps::ShapePush { region, index: 0 },
                        Command::ClipPop => ClipOps::Pop,
                        _ => unreachable!(),
                    };
                    draw_builder.draws.push(DrawOrClip::Clip(clip));
                }
            }
        }

        flush_draw_pass(&mut passes, &mut draw_builder);
        flush_compute_pass(&mut passes, &mut compute_items);

        Self { passes }
    }

    /// Returns the render passes in submission order.
    pub(crate) fn into_passes(self) -> Vec<RenderPassPlan> {
        self.passes
    }
}

/// A render pass plan with draw commands or compute work.
pub(crate) enum RenderPassPlan {
    /// Draws into the scene; `reads_scene` passes sample the scene first.
    Draw {
        draws: SmallVec<[DrawOrClip; 32]>,
        reads_scene: bool,
    },
    /// Filters the scene for the draws that sample it next.
    Compute(Vec<ComputePlanItem>),
}

#[derive(Default)]
struct DrawPassBuilder {
    draws: SmallVec<[DrawOrClip; 32]>,
    sampling_rects: SmallVec<[PxRect; 16]>,
    reads_scene: bool,
}

impl DrawPassBuilder {
    fn last_draw(&self) -> Option<&DrawCommandWithMetadata> {
        self.draws.iter().rev().find_map(|command| match command {
            DrawOrClip::Draw(cmd) => Some(cmd),
            DrawOrClip::Clip(_) => None,
        })
    }
}

/// Compute work required for a render pass.
pub(crate) struct ComputePlanItem {
    pub(crate) command: Box<dyn ComputeCommand>,
    pub(crate) geometry: DrawGeometry,
    pub(crate) target_rect: PxRect,
    pub(crate) sampling_rect: PxRect,
}

/// Metadata for draw commands submitted in a pass.
pub(crate) struct DrawCommandWithMetadata {
    pub(crate) command: Box<dyn DrawCommand>,
    pub(crate) type_id: std::any::TypeId,
    pub(crate) geometry: DrawGeometry,
    pub(crate) draw_rect: PxRect,
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
    geometry: DrawGeometry,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    match barrier {
        Some(SampleRegion::Global) => PxRect {
            x: Px(0),
            y: Px(0),
            width: Px(texture_size.width as i32),
            height: Px(texture_size.height as i32),
        },
        Some(SampleRegion::PaddedLocal(sampling)) => clamp_rect_to_texture(
            geometry.padded_window_bounds(sampling).outward(),
            texture_size,
        ),
        Some(SampleRegion::Absolute(rect)) => clamp_rect_to_texture(rect, texture_size),
        None => extract_target_rect(geometry, texture_size),
    }
}

fn extract_draw_rect(
    region: DrawRegion,
    geometry: DrawGeometry,
    texture_size: wgpu::Extent3d,
) -> PxRect {
    match region {
        DrawRegion::Global => PxRect {
            x: Px(0),
            y: Px(0),
            width: Px(texture_size.width as i32),
            height: Px(texture_size.height as i32),
        },
        DrawRegion::PaddedLocal(padding) => clamp_rect_to_texture(
            geometry.padded_window_bounds(padding).outward(),
            texture_size,
        ),
        DrawRegion::Absolute(rect) => clamp_rect_to_texture(rect, texture_size),
    }
}

fn extract_target_rect(geometry: DrawGeometry, texture_size: wgpu::Extent3d) -> PxRect {
    clamp_rect_to_texture(geometry.window_bounds().outward(), texture_size)
}

#[cfg(test)]
mod tests {
    use crate::{PxPosition, PxSize};

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
            geometry: DrawGeometry::from_position_size(
                PxPosition::new(Px(x), Px(0)),
                PxSize::new(Px(10), Px(10)),
            ),
            opacity: 1.0,
            sequence_index,
        }
    }

    #[test]
    fn clip_state_preserves_sampled_draw_batching() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
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
        let passes = RenderPassGraph::build(ops, target).into_passes();
        assert_eq!(
            passes.len(),
            1,
            "clip state must not split nonoverlapping scene samples"
        );
        let RenderPassPlan::Draw { draws, .. } = &passes[0] else {
            panic!("draw pass");
        };
        assert_eq!(draws.len(), 4);
    }

    #[test]
    fn fractional_draw_and_sample_bounds_round_outwards() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let size = PxSize::new(Px(20), Px(10));
        let position = PxPosition::from_f32_xy(10.25, 10.75);
        let geometry = DrawGeometry::from_position_size(position, size);
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        let expected = PxRect::new(Px(10), Px(10), Px(21), Px(11));
        assert_eq!(
            extract_draw_rect(
                DrawRegion::PaddedLocal(crate::PaddingRect::ZERO),
                geometry,
                target
            ),
            expected
        );
        assert_eq!(
            extract_sampling_rect(
                Some(SampleRegion::PaddedLocal(crate::PaddingRect::ZERO)),
                geometry,
                target
            ),
            expected
        );
        assert_eq!(
            extract_target_rect(DrawGeometry::from_position_size(position, size), target),
            expected
        );
    }

    #[test]
    fn scaled_draw_plan_preserves_local_geometry_and_window_sampling_under_clips() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let geometry = DrawGeometry::new(
            PxSize::new(Px(240), Px(56)),
            crate::LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5)).unwrap(),
        );
        let mut draw = op(Command::Draw(Box::new(ProbeDraw(true))), 1, 0);
        draw.geometry = geometry;
        let passes = RenderPassGraph::build(
            vec![
                op(
                    Command::ClipPush(PxRect::new(Px(12), Px(24), Px(4), Px(4))),
                    0,
                    0,
                ),
                draw,
            ],
            wgpu::Extent3d {
                width: 400,
                height: 200,
                depth_or_array_layers: 1,
            },
        )
        .into_passes();
        let draws = || {
            passes.iter().flat_map(|pass| match pass {
                RenderPassPlan::Draw { draws, .. } => draws.as_slice(),
                RenderPassPlan::Compute(_) => &[],
            })
        };
        assert!(draws().any(|command| {
            matches!(command, DrawOrClip::Clip(ClipOps::Push(rect)) if *rect == PxRect::new(Px(12), Px(24), Px(4), Px(4)))
        }));
        let draw = draws()
            .find_map(|command| match command {
                DrawOrClip::Draw(draw) => Some(draw),
                DrawOrClip::Clip(_) => None,
            })
            .expect("missing native draw");
        assert_eq!(draw.geometry, geometry);
        assert_eq!(draw.geometry.local_size, PxSize::new(Px(240), Px(56)));
        assert_eq!(draw.draw_rect, PxRect::new(Px(10), Px(20), Px(151), Px(36)));
        assert_eq!(
            extract_sampling_rect(
                Some(SampleRegion::uniform_padding_local(Px(4))),
                draw.geometry,
                wgpu::Extent3d {
                    width: 400,
                    height: 200,
                    depth_or_array_layers: 1
                },
            ),
            PxRect::new(Px(7), Px(18), Px(156), Px(40)),
            "sampling must scale local padding and remain independent of draw clipping"
        );
        let absolute = PxRect::new(Px(30), Px(40), Px(50), Px(60));
        assert_eq!(
            extract_sampling_rect(
                Some(SampleRegion::Absolute(absolute)),
                draw.geometry,
                wgpu::Extent3d {
                    width: 400,
                    height: 200,
                    depth_or_array_layers: 1
                },
            ),
            absolute
        );
    }

    #[test]
    fn negative_bounds_do_not_extend_the_opposite_edge() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let target = wgpu::Extent3d {
            width: 100,
            height: 100,
            depth_or_array_layers: 1,
        };
        assert_eq!(
            extract_target_rect(
                DrawGeometry::from_position_size(
                    PxPosition::from_f32_xy(-3.25, -2.5),
                    PxSize::new(Px(20), Px(10))
                ),
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
    next_sampling_rect: Option<PxRect>,
    sampling_rects_in_pass: &[PxRect],
) -> bool {
    let (Some(last_draw), Some(next_sampling_rect)) = (last_draw, next_sampling_rect) else {
        return false;
    };
    // A sampling draw must see earlier draws, and sampled regions in one pass
    // must not overlap.
    last_draw.command.sample_region().is_none()
        || !sampling_rects_in_pass
            .iter()
            .all(|rect| rect.is_orthogonal(&next_sampling_rect))
}

fn flush_draw_pass(passes: &mut Vec<RenderPassPlan>, builder: &mut DrawPassBuilder) {
    let builder = std::mem::take(builder);
    if !builder.draws.is_empty() {
        passes.push(RenderPassPlan::Draw {
            draws: builder.draws,
            reads_scene: builder.reads_scene,
        });
    }
}

fn flush_compute_pass(passes: &mut Vec<RenderPassPlan>, items: &mut Vec<ComputePlanItem>) {
    if !items.is_empty() {
        passes.push(RenderPassPlan::Compute(std::mem::take(items)));
    }
}
