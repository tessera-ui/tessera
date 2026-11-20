//! Render graph fragments and nodes for Tessera.
//!
//! ## Usage
//!
//! Build per-component render fragments and merge them into a frame graph.

use std::{
    any::TypeId,
    sync::{Arc, OnceLock, Weak},
};

use smallvec::SmallVec;

use crate::{
    FloatRect, LocalToWindow,
    component_tree::SceneRope,
    px::{Px, PxPosition, PxRect, PxSize},
    render::{Command, ComputeCommand, DrawCommand, DrawRegion, SampleRegion},
};

/// Local drawing dimensions and their precise placement in the render target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawGeometry {
    /// Unscaled dimensions used by command coordinates, radii and strokes.
    pub local_size: PxSize,
    /// Mapping from command-local pixels to render-target pixels.
    pub local_to_window: LocalToWindow,
}

impl DrawGeometry {
    /// Creates drawing geometry with an explicit local-to-target mapping.
    #[must_use]
    pub const fn new(local_size: PxSize, local_to_window: LocalToWindow) -> Self {
        Self {
            local_size,
            local_to_window,
        }
    }

    /// Creates unscaled geometry at a target position.
    #[must_use]
    pub fn from_position_size(position: PxPosition, local_size: PxSize) -> Self {
        Self::new(
            local_size,
            LocalToWindow::new(1.0, position).expect("finite draw position"),
        )
    }

    /// Returns the precise target origin.
    #[must_use]
    pub fn window_origin(self) -> PxPosition {
        self.local_to_window.map_point(PxPosition::ZERO)
    }

    /// Returns target dimensions without integer rounding.
    #[must_use]
    pub fn window_size(self) -> [f32; 2] {
        self.local_to_window.map_size(self.local_size)
    }

    /// Returns precise target bounds.
    #[must_use]
    pub fn window_bounds(self) -> FloatRect {
        self.padded_window_bounds(crate::render::PaddingRect::ZERO)
    }

    /// Maps local padding to target bounds without rounding individual edges.
    #[must_use]
    pub fn padded_window_bounds(self, padding: crate::render::PaddingRect) -> FloatRect {
        self.local_to_window.map_rect(FloatRect {
            x: -padding.left.to_f32(),
            y: -padding.top.to_f32(),
            width: self.local_size.width.to_f32() + padding.left.to_f32() + padding.right.to_f32(),
            height: self.local_size.height.to_f32()
                + padding.top.to_f32()
                + padding.bottom.to_f32(),
        })
    }
}

/// A single render op inside a fragment.
#[derive(Clone)]
pub struct RenderFragmentOp {
    /// The command to execute.
    pub command: Command,
    /// Type identifier used for batching.
    pub type_id: TypeId,
    /// Optional size override for this op.
    pub size_override: Option<PxSize>,
    /// Optional offset of this op from its node's origin.
    pub position_override: Option<PxPosition>,
}

/// A per-component render fragment.
#[derive(Default, Clone)]
pub struct RenderFragment {
    ops: Option<Arc<SmallVec<[RenderFragmentOp; 1]>>>,
}

impl RenderFragment {
    /// Returns fragment ops in insertion order.
    #[must_use]
    pub fn ops(&self) -> &[RenderFragmentOp] {
        self.ops.as_deref().map_or(&[], SmallVec::as_slice)
    }

    /// Returns fragment ops in insertion order for local position overrides.
    #[must_use]
    pub fn ops_mut(&mut self) -> &mut [RenderFragmentOp] {
        self.ops
            .as_mut()
            .map_or(&mut [], |ops| Arc::make_mut(ops).as_mut_slice())
    }

    /// Adds a draw command.
    pub fn push_draw_command<C: DrawCommand + 'static>(&mut self, command: C) -> u32 {
        self.push_op(Command::Draw(Box::new(command)), TypeId::of::<C>())
    }

    /// Adds a compute command.
    pub fn push_compute_command<C: ComputeCommand + 'static>(&mut self, command: C) -> u32 {
        self.push_op(Command::Compute(Box::new(command)), TypeId::of::<C>())
    }

    #[cfg(any(target_os = "android", target_os = "ios", test))]
    pub(crate) fn push_platform_view(&mut self, id: u64) -> u32 {
        self.push_op(Command::PlatformView(id), TypeId::of::<Command>())
    }

    fn push_op(&mut self, command: Command, type_id: TypeId) -> u32 {
        let ops = Arc::make_mut(self.ops.get_or_insert_with(Arc::default));
        let index = ops.len() as u32;
        ops.push(RenderFragmentOp {
            command,
            type_id,
            size_override: None,
            position_override: None,
        });
        index
    }

    /// Clears all ops in the fragment.
    pub fn clear(&mut self) {
        if let Some(ops) = self.ops.as_mut().and_then(Arc::get_mut) {
            ops.clear();
        } else {
            self.ops = None;
        }
    }

    pub(crate) fn shares_ops(&self, other: &Self) -> bool {
        match (&self.ops, &other.ops) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn supports_partial_redraw(&self) -> bool {
        self.ops().iter().all(|op| partial_redraw_safe(&op.command))
    }
}

/// A render op after merging into the frame graph.
#[derive(Clone)]
pub struct RenderGraphOp {
    /// The command to execute.
    pub command: Command,
    /// Type identifier used for batching and ordering.
    pub type_id: TypeId,
    /// Local drawing dimensions and their precise target mapping.
    pub geometry: DrawGeometry,
    /// Opacity multiplier applied during record.
    pub opacity: f32,
    /// Sequence index used to preserve authoring order.
    pub sequence_index: usize,
}

/// A native view's placement in the ordered scene.
#[cfg(any(target_os = "android", target_os = "ios", test))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlatformViewPlacement {
    pub(crate) id: u64,
    pub(crate) geometry: DrawGeometry,
    pub(crate) opacity: f32,
    pub(crate) clip_bounds: Option<PxRect>,
    pub(crate) unsupported_shape_clip: bool,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Clone, Copy)]
pub(crate) struct PlatformViewInputBlocker {
    pub(crate) id: u64,
    pub(crate) bounds: PxRect,
}

/// Tessera draw slices separated by native views in paint order.
#[cfg(any(target_os = "android", target_os = "ios", test))]
pub(crate) struct PlatformViewSlices {
    pub(crate) views: Vec<PlatformViewPlacement>,
    pub(crate) draws: Vec<PlatformViewDrawSlice>,
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
impl PlatformViewSlices {
    fn compact(&mut self) {
        if self.draws.iter().any(|slice| slice.samples_native_scene) {
            return;
        }
        let mut previous = 0;
        let mut previous_coverage = self.draws[0].coverage();
        for index in 1..self.draws.len() {
            let coverage = self.draws[index].coverage();
            if coverage == SliceCoverage::Empty {
                self.draws[index].ops.clear();
                continue;
            }
            if let SliceCoverage::Bounded(bounds) = coverage
                && previous_coverage.is_disjoint(bounds)
                && self.views[previous..index].iter().all(|view| {
                    view.visible_bounds()
                        .is_none_or(|view_bounds| bounds.intersection(&view_bounds).is_none())
                })
            {
                // Do not change which compositor blends overlapping Tessera pixels.
                // Each slice's clip stack is balanced, so it can be appended intact.
                let ops = std::mem::take(&mut self.draws[index].ops);
                self.draws[previous].ops.extend(ops);
                previous_coverage = match previous_coverage {
                    SliceCoverage::Bounded(earlier) => {
                        SliceCoverage::Bounded(earlier.union(&bounds))
                    }
                    _ => coverage,
                };
            } else {
                previous = index;
                previous_coverage = coverage;
            }
        }
    }

    pub(crate) fn overlays_after(&self) -> Vec<usize> {
        self.draws
            .iter()
            .skip(1)
            .enumerate()
            .filter_map(|(index, slice)| {
                slice
                    .ops
                    .iter()
                    .any(|op| matches!(op.command, Command::Draw(_) | Command::Compute(_)))
                    .then_some(index)
            })
            .collect()
    }
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
impl PlatformViewPlacement {
    fn visible_bounds(&self) -> Option<PxRect> {
        if self.opacity <= 0.0 || self.unsupported_shape_clip {
            return None;
        }
        let bounds = self.geometry.window_bounds().outward();
        let bounds = if let Some(clip) = self.clip_bounds {
            bounds.intersection(&clip)?
        } else {
            bounds
        };
        Some(bounds)
    }
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
#[derive(Clone, Copy, PartialEq)]
enum SliceCoverage {
    Empty,
    Bounded(PxRect),
    Unknown,
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
impl SliceCoverage {
    fn is_disjoint(self, bounds: PxRect) -> bool {
        match self {
            Self::Empty => true,
            Self::Bounded(other) => bounds.intersection(&other).is_none(),
            Self::Unknown => false,
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
pub(crate) struct PlatformViewDrawSlice {
    pub(crate) ops: Vec<RenderGraphOp>,
    pub(crate) samples_native_scene: bool,
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
impl PlatformViewDrawSlice {
    fn coverage(&self) -> SliceCoverage {
        let mut coverage: Option<PxRect> = None;
        let mut clips: SmallVec<[PxRect; 8]> = SmallVec::new();
        for op in &self.ops {
            let clip = match &op.command {
                Command::ClipPush(bounds) => *bounds,
                Command::ShapeClipPush(region) => region.bounds(),
                Command::ClipPop => {
                    if clips.pop().is_none() {
                        return SliceCoverage::Unknown;
                    }
                    continue;
                }
                Command::Draw(command) if partial_redraw_safe(&op.command) => {
                    let region = command.draw_region();
                    if matches!(region, DrawRegion::Global) {
                        return SliceCoverage::Unknown;
                    }
                    let bounds = draw_region_rect(region, op.geometry);
                    let bounds = clips
                        .last()
                        .map_or(Some(bounds), |c| c.intersection(&bounds));
                    if let Some(bounds) =
                        bounds.filter(|b| b.width > Px::ZERO && b.height > Px::ZERO)
                    {
                        coverage = Some(coverage.map_or(bounds, |old| old.union(&bounds)));
                    }
                    continue;
                }
                _ => return SliceCoverage::Unknown,
            };
            let clip = clips.last().map_or(clip, |active| {
                active.intersection(&clip).unwrap_or(PxRect::ZERO)
            });
            clips.push(clip);
        }
        if !clips.is_empty() {
            return SliceCoverage::Unknown;
        }
        coverage.map_or(SliceCoverage::Empty, SliceCoverage::Bounded)
    }

    pub(crate) fn render_graph(&self) -> RenderGraph {
        RenderGraphBuilder {
            ops: self.ops.clone(),
        }
        .finish()
    }
}

/// Frame-level render graph.
#[derive(Clone, Default)]
pub struct RenderGraph {
    data: Arc<RenderGraphData>,
    damage: Option<PxRect>,
    previous: Option<Weak<RenderGraphData>>,
    #[cfg(any(target_os = "android", target_os = "ios"))]
    platform_input_blockers: Vec<PlatformViewInputBlocker>,
}

#[derive(Default)]
struct RenderGraphData {
    order_plan: OnceLock<Arc<RenderOrderPlan>>,
    scene: Option<Arc<SceneRope>>,
    flat: OnceLock<FlatRenderGraph>,
}

#[derive(Default)]
struct FlatRenderGraph {
    ops: Vec<RenderGraphOp>,
    partial_redraw: bool,
}

impl RenderGraphData {
    fn flat(&self) -> &FlatRenderGraph {
        self.flat.get_or_init(|| {
            let Some(scene) = &self.scene else {
                return FlatRenderGraph::default();
            };
            let mut builder = RenderGraphBuilder::new();
            scene.emit(&mut builder, None);
            builder.into_flat()
        })
    }

    fn supports_partial_redraw(&self) -> bool {
        self.flat
            .get()
            .map_or(self.scene.is_some(), |flat| flat.partial_redraw)
    }
}

struct RenderOrderPlan {
    inputs: Box<[OpInfo]>,
    indices: Box<[usize]>,
}

impl RenderOrderPlan {
    fn new(ops: &[RenderGraphOp]) -> Self {
        let inputs: Box<[_]> = ops.iter().map(OpInfo::new).collect();
        let indices = order_indices(ops, &inputs).into_boxed_slice();
        Self { inputs, indices }
    }

    fn matches(&self, ops: &[RenderGraphOp]) -> bool {
        self.inputs.len() == ops.len()
            && self
                .inputs
                .iter()
                .zip(ops)
                .all(|(input, op)| *input == OpInfo::new(op))
    }
}

impl RenderGraph {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn with_platform_input_blockers(
        mut self,
        blockers: Vec<PlatformViewInputBlocker>,
    ) -> Self {
        self.platform_input_blockers = blockers;
        self
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn platform_input_blockers(&self) -> &[PlatformViewInputBlocker] {
        &self.platform_input_blockers
    }

    pub(crate) fn cached_op_count(&self) -> Option<usize> {
        self.data.flat.get().map(|flat| flat.ops.len())
    }

    pub(crate) fn from_partial_scene(scene: Arc<SceneRope>) -> Self {
        Self {
            data: Arc::new(RenderGraphData {
                scene: Some(scene),
                ..RenderGraphData::default()
            }),
            damage: None,
            previous: None,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            platform_input_blockers: Vec::new(),
        }
    }

    pub(crate) fn with_damage(mut self, damage: Option<PxRect>, previous: Option<&Self>) -> Self {
        self.damage = damage;
        self.previous = previous.map(|graph| Arc::downgrade(&graph.data));
        self
    }

    pub(crate) fn damage_since(&self, previous: Option<&Self>) -> Option<PxRect> {
        let previous = previous?;
        if !self.data.supports_partial_redraw() || !previous.data.supports_partial_redraw() {
            return None;
        }
        if Arc::ptr_eq(&self.data, &previous.data) {
            return Some(PxRect::ZERO);
        }
        if self
            .previous
            .as_ref()
            .is_some_and(|data| std::ptr::eq(data.as_ptr(), Arc::as_ptr(&previous.data)))
        {
            self.damage
        } else {
            // A frame was skipped after the scene journals advanced.
            None
        }
    }

    pub(crate) fn for_damage(&self, damage: PxRect) -> Option<Self> {
        let scene = self.data.scene.as_ref()?;
        let mut builder = RenderGraphBuilder::new();
        scene.emit(&mut builder, Some(damage));
        Some(builder.finish())
    }

    pub(crate) fn supports_partial_redraw(&self) -> bool {
        self.data.supports_partial_redraw()
    }

    /// Returns ops in the graph.
    #[must_use]
    pub fn ops(&self) -> &[RenderGraphOp] {
        &self.data.flat().ops
    }

    fn order_plan(&self, previous: Option<&Self>) -> &Arc<RenderOrderPlan> {
        self.data.order_plan.get_or_init(|| {
            if let Some(plan) = previous.and_then(|graph| graph.data.order_plan.get())
                && plan.matches(self.ops())
            {
                Arc::clone(plan)
            } else {
                Arc::new(RenderOrderPlan::new(self.ops()))
            }
        })
    }

    pub(crate) fn ordered_ops(
        &self,
        previous: Option<&Self>,
    ) -> impl Iterator<Item = &RenderGraphOp> {
        let ops = self.ops();
        self.order_plan(previous)
            .indices
            .iter()
            .map(|&index| &ops[index])
    }

    #[cfg(any(target_os = "android", target_os = "ios", test))]
    pub(crate) fn has_platform_views(&self) -> bool {
        self.ops()
            .iter()
            .any(|op| matches!(op.command, Command::PlatformView(_)))
    }

    #[cfg(any(target_os = "android", target_os = "ios", test))]
    pub(crate) fn platform_view_slices(&self) -> PlatformViewSlices {
        let mut views = Vec::new();
        let mut draws = vec![PlatformViewDrawSlice {
            ops: Vec::new(),
            samples_native_scene: false,
        }];
        let mut active_clips: Vec<RenderGraphOp> = Vec::new();
        for op in self.ordered_ops(None) {
            if let crate::render::Command::PlatformView(id) = &op.command {
                let current = draws.last_mut().expect("scene has an initial slice");
                for _ in 0..active_clips.len() {
                    current
                        .ops
                        .push(state_op(Command::ClipPop, op.sequence_index));
                }
                let mut clip_bounds = None;
                let mut unsupported_shape_clip = false;
                for clip in &active_clips {
                    let bounds = match &clip.command {
                        Command::ClipPush(rect) => *rect,
                        Command::ShapeClipPush(region) => {
                            unsupported_shape_clip = true;
                            region.bounds()
                        }
                        _ => continue,
                    };
                    clip_bounds = Some(clip_bounds.map_or(bounds, |current: PxRect| {
                        current.intersection(&bounds).unwrap_or(PxRect::ZERO)
                    }));
                }
                views.push(PlatformViewPlacement {
                    id: *id,
                    geometry: op.geometry,
                    opacity: op.opacity,
                    clip_bounds,
                    unsupported_shape_clip,
                });
                let mut next = PlatformViewDrawSlice {
                    ops: Vec::with_capacity(active_clips.len()),
                    samples_native_scene: false,
                };
                next.ops.extend(active_clips.iter().cloned());
                draws.push(next);
            } else if matches!(op.command, Command::ClipPush(_) | Command::ShapeClipPush(_)) {
                active_clips.push(op.clone());
                draws
                    .last_mut()
                    .expect("scene has an initial slice")
                    .ops
                    .push(op.clone());
            } else if matches!(op.command, Command::ClipPop) {
                active_clips.pop();
                draws
                    .last_mut()
                    .expect("scene has an initial slice")
                    .ops
                    .push(op.clone());
            } else {
                let draw = draws.last_mut().expect("scene has an initial slice");
                draw.samples_native_scene |= match &op.command {
                    Command::Draw(command) => command.sample_region().is_some(),
                    Command::Compute(_) => true,
                    _ => false,
                } && !views.is_empty();
                draw.ops.push(op.clone());
            }
        }
        if let Some(last) = draws.last_mut() {
            let sequence = self.ops().len();
            for _ in 0..active_clips.len() {
                last.ops.push(state_op(Command::ClipPop, sequence));
            }
        }
        debug_assert_eq!(draws.len(), views.len() + 1);
        let mut slices = PlatformViewSlices { views, draws };
        slices.compact();
        slices
    }
}

#[cfg(any(target_os = "android", target_os = "ios", test))]
fn state_op(command: Command, sequence_index: usize) -> RenderGraphOp {
    RenderGraphOp {
        command,
        type_id: TypeId::of::<Command>(),
        geometry: DrawGeometry::new(PxSize::ZERO, LocalToWindow::IDENTITY),
        opacity: 1.0,
        sequence_index,
    }
}

/// Builder for a frame-level render graph.
pub(crate) struct RenderGraphBuilder {
    ops: Vec<RenderGraphOp>,
}

impl RenderGraphBuilder {
    /// Creates a new render graph builder.
    pub(crate) fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            ops: Vec::with_capacity(capacity),
        }
    }

    fn push_state(&mut self, command: Command) {
        self.ops.push(RenderGraphOp {
            command,
            type_id: TypeId::of::<Command>(),
            geometry: DrawGeometry::new(PxSize::ZERO, LocalToWindow::IDENTITY),
            opacity: 1.0,
            sequence_index: self.ops.len(),
        });
    }

    /// Pushes a clip push op into the graph.
    pub(crate) fn push_clip_push(&mut self, rect: PxRect) {
        self.push_state(Command::ClipPush(rect));
    }

    /// Pushes a shape clip op into the graph.
    pub(crate) fn push_shape_clip_push(&mut self, region: crate::render::ClipRegion) {
        self.push_state(Command::ShapeClipPush(region));
    }

    /// Pushes a clip pop op into the graph.
    pub(crate) fn push_clip_pop(&mut self) {
        self.push_state(Command::ClipPop);
    }

    /// Appends a fragment into the frame graph.
    pub(crate) fn append_fragment(
        &mut self,
        fragment: &RenderFragment,
        geometry: DrawGeometry,
        opacity: f32,
    ) {
        for op in fragment.ops() {
            let mut command = op.command.clone();
            if opacity != 1.0
                && let Command::Draw(ref mut command) = command
            {
                command.apply_opacity(opacity);
            }
            let mapping = geometry
                .local_to_window
                .with_local_origin(op.position_override.unwrap_or(PxPosition::ZERO));
            self.ops.push(RenderGraphOp {
                command,
                type_id: op.type_id,
                geometry: DrawGeometry::new(
                    op.size_override.unwrap_or(geometry.local_size),
                    mapping,
                ),
                opacity,
                sequence_index: self.ops.len(),
            });
        }
    }

    /// Finishes graph construction.
    pub(crate) fn finish(self) -> RenderGraph {
        RenderGraph {
            data: Arc::new(RenderGraphData {
                flat: OnceLock::from(self.into_flat()),
                ..RenderGraphData::default()
            }),
            damage: None,
            previous: None,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            platform_input_blockers: Vec::new(),
        }
    }

    fn into_flat(self) -> FlatRenderGraph {
        let partial_redraw = self.ops.iter().all(|op| partial_redraw_safe(&op.command));
        FlatRenderGraph {
            ops: self.ops,
            partial_redraw,
        }
    }
}

fn partial_redraw_safe(command: &Command) -> bool {
    match command {
        Command::Draw(command) => {
            command.sample_region().is_none() && command.supports_partial_redraw()
        }
        Command::Compute(_) => false,
        Command::PlatformView(_) => false,
        _ => true,
    }
}
mod ordering;
#[cfg(test)]
mod retained_order_tests;

use ordering::order_indices;

#[cfg(test)]
fn order_ops(ops: Vec<RenderGraphOp>) -> Vec<RenderGraphOp> {
    let infos: Vec<_> = ops.iter().map(OpInfo::new).collect();
    order_indices(&ops, &infos)
        .into_iter()
        .map(|index| ops[index].clone())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum OpCategory {
    ContinuationDraw,
    BarrierDraw,
    Compute,
    StateChange,
}

#[derive(PartialEq, Eq)]
struct OpInfo {
    category: OpCategory,
    command_kind: std::mem::Discriminant<Command>,
    type_id: TypeId,
    sequence_index: usize,
    read_rect: Option<PxRect>,
    write_rect: Option<PxRect>,
}

impl OpInfo {
    fn new(op: &RenderGraphOp) -> Self {
        let category = match &op.command {
            Command::Draw(command) => {
                if command.sample_region().is_some() {
                    OpCategory::BarrierDraw
                } else {
                    OpCategory::ContinuationDraw
                }
            }
            Command::Compute(_) => OpCategory::Compute,
            Command::ClipPush(_)
            | Command::ShapeClipPush(_)
            | Command::ClipPop
            | Command::PlatformView(_) => OpCategory::StateChange,
        };

        Self {
            category,
            command_kind: std::mem::discriminant(&op.command),
            type_id: op.type_id,
            sequence_index: op.sequence_index,
            read_rect: scene_read_rect(op),
            write_rect: scene_write_rect(op),
        }
    }
}

fn scene_read_rect(op: &RenderGraphOp) -> Option<PxRect> {
    match &op.command {
        Command::Draw(command) => command
            .sample_region()
            .map(|region| sample_region_rect(region, op.geometry)),
        Command::Compute(command) => Some(sample_region_rect(command.barrier(), op.geometry)),
        Command::ClipPush(_)
        | Command::ShapeClipPush(_)
        | Command::ClipPop
        | Command::PlatformView(_) => None,
    }
}

fn scene_write_rect(op: &RenderGraphOp) -> Option<PxRect> {
    match &op.command {
        Command::Draw(command) => Some(
            command
                .ordering_rect(op.geometry)
                .unwrap_or_else(|| draw_region_rect(command.draw_region(), op.geometry)),
        ),
        Command::Compute(_) => Some(op.geometry.window_bounds().outward()),
        Command::ClipPush(_)
        | Command::ShapeClipPush(_)
        | Command::ClipPop
        | Command::PlatformView(_) => None,
    }
}

fn sample_region_rect(region: SampleRegion, geometry: DrawGeometry) -> PxRect {
    match region {
        SampleRegion::Global => PxRect::new(Px::ZERO, Px::ZERO, Px::MAX, Px::MAX),
        SampleRegion::PaddedLocal(padding) => geometry.padded_window_bounds(padding).outward(),
        SampleRegion::Absolute(rect) => rect,
    }
}

fn draw_region_rect(region: DrawRegion, geometry: DrawGeometry) -> PxRect {
    match region {
        DrawRegion::Global => PxRect::new(Px::ZERO, Px::ZERO, Px::MAX, Px::MAX),
        DrawRegion::PaddedLocal(padding) => geometry.padded_window_bounds(padding).outward(),
        DrawRegion::Absolute(rect) => rect,
    }
}

fn is_state_change(op: &RenderGraphOp) -> bool {
    matches!(
        op.command,
        Command::ClipPush(_)
            | Command::ShapeClipPush(_)
            | Command::ClipPop
            | Command::PlatformView(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Px, PxPosition, PxSize,
        render::{DrawCommand, PaddingRect, SampleRegion},
    };

    #[test]
    fn fragment_overrides_map_through_the_node_transform_once() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(ProbeDraw {
            id: 1,
            sample: None,
        });
        for op in fragment.ops_mut() {
            op.size_override = Some(PxSize::new(Px(64), Px(32)));
            op.position_override = Some(PxPosition::from_f32_xy(4.0, 8.0));
        }
        let mut builder = RenderGraphBuilder::new();
        builder.append_fragment(
            &fragment,
            DrawGeometry::new(
                PxSize::new(Px(240), Px(56)),
                LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5))
                    .expect("test fixture provides from_f32_xy"),
            ),
            1.0,
        );
        let graph = builder.finish();
        let scene = &graph.ops()[0];
        assert_eq!(scene.geometry.local_size, PxSize::new(Px(64), Px(32)));
        assert_eq!(
            scene.geometry.window_origin(),
            PxPosition::from_f32_xy(12.75, 25.5)
        );
        assert_eq!(scene.geometry.window_size(), [40.0, 20.0]);
    }

    #[derive(Clone)]
    struct ProbeDraw {
        id: u32,
        sample: Option<SampleRegion>,
    }

    impl DrawCommand for ProbeDraw {
        fn supports_partial_redraw(&self) -> bool {
            true
        }

        fn sample_region(&self) -> Option<SampleRegion> {
            self.sample
        }

        fn apply_opacity(&mut self, _opacity: f32) {}
    }

    #[test]
    fn native_view_boundaries_split_draws_without_changing_paint_order() {
        let mut builder = RenderGraphBuilder::new();
        let geometry = DrawGeometry::from_position_size(
            PxPosition::from_f32_xy(12.5, 20.25),
            PxSize::new(Px(80), Px(40)),
        );
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(ProbeDraw {
            id: 1,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_platform_view(17);
        builder.append_fragment(&fragment, geometry, 0.75);
        fragment.clear();
        fragment.push_draw_command(ProbeDraw {
            id: 2,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_platform_view(23);
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_draw_command(ProbeDraw {
            id: 3,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);

        let graph = builder.finish();
        let slices = graph.platform_view_slices();

        assert_eq!(slices.views.len(), 2);
        assert_eq!(slices.draws.len(), 3);
        assert_eq!(slices.overlays_after(), [0, 1]);
        assert_eq!(
            slices.views.iter().map(|view| view.id).collect::<Vec<_>>(),
            [17, 23]
        );
        assert_eq!(slices.views[0].geometry, geometry);
        assert_eq!(slices.views[0].opacity, 0.75);
        let draw_ids = slices
            .draws
            .iter()
            .map(|slice| {
                slice
                    .ops
                    .iter()
                    .map(|op| {
                        op.command
                            .downcast_draw_ref::<ProbeDraw>()
                            .expect("slice contains draw commands")
                            .id
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(draw_ids, [vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn platform_view_slices_merge_only_disjoint_visible_pixels() {
        let at = |x, width| {
            DrawGeometry::from_position_size(
                PxPosition::from_f32_xy(x, 0.25),
                PxSize::new(Px(width), Px(20)),
            )
        };
        let draw = |id| {
            let mut fragment = RenderFragment::default();
            fragment.push_draw_command(ProbeDraw { id, sample: None });
            fragment
        };
        let view = |id| {
            let mut fragment = RenderFragment::default();
            fragment.push_platform_view(id);
            fragment
        };
        let mut builder = RenderGraphBuilder::new();
        builder.append_fragment(&draw(1), at(0.0, 10), 1.0);
        builder.append_fragment(&view(7), at(20.5, 10), 1.0);
        // The draw nominally intersects the view, but the clip leaves it on the right.
        let clip = PxRect::new(Px(31), Px(0), Px(10), Px(21));
        builder.push_clip_push(clip);
        builder.append_fragment(&draw(2), at(20.0, 30), 1.0);
        builder.push_clip_pop();
        builder.append_fragment(&view(8), at(50.5, 10), 1.0);
        // Would cross the earlier native view as well as the empty boundary at view 8.
        builder.append_fragment(&draw(3), at(16.0, 5), 1.0);
        builder.append_fragment(&view(9), at(65.0, 10), 1.0);
        // Does not touch view 9 but must not change how it blends with draw 3.
        builder.append_fragment(&draw(4), at(16.0, 5), 1.0);

        let slices = builder.finish().platform_view_slices();
        assert_eq!(slices.overlays_after(), [1, 2]);
        let underlay = slices.draws[0].render_graph();
        assert!(matches!(underlay.ops()[1].command, Command::ClipPush(bounds) if bounds == clip));
        assert_eq!(
            underlay.ops()[2]
                .command
                .downcast_draw_ref::<ProbeDraw>()
                .unwrap()
                .id,
            2
        );
        assert!(matches!(underlay.ops()[3].command, Command::ClipPop));
    }

    #[test]
    fn platform_view_slices_reopen_clips_and_flag_native_scene_sampling() {
        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(40), Px(30)));
        let clip = PxRect::new(Px(3), Px(4), Px(20), Px(12));
        let mut builder = RenderGraphBuilder::new();
        builder.push_clip_push(clip);
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(ProbeDraw {
            id: 1,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_platform_view(9);
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_draw_command(ProbeDraw {
            id: 2,
            sample: Some(SampleRegion::Global),
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        builder.push_clip_pop();

        let slices = builder.finish().platform_view_slices();

        assert_eq!(slices.views[0].clip_bounds, Some(clip));
        assert!(!slices.views[0].unsupported_shape_clip);
        assert_eq!(slices.draws.len(), 2);
        assert!(
            matches!(slices.draws[0].ops.first().map(|op| &op.command), Some(Command::ClipPush(rect)) if *rect == clip)
        );
        assert!(matches!(
            slices.draws[0].ops.last().map(|op| &op.command),
            Some(Command::ClipPop)
        ));
        assert!(
            matches!(slices.draws[1].ops.first().map(|op| &op.command), Some(Command::ClipPush(rect)) if *rect == clip)
        );
        assert!(matches!(
            slices.draws[1].ops.last().map(|op| &op.command),
            Some(Command::ClipPop)
        ));
        assert!(slices.draws[1].samples_native_scene);
    }

    #[test]
    fn platform_view_slices_intersect_nested_clips_and_restore_the_outer_clip() {
        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(40), Px(30)));
        let outer = PxRect::new(Px(2), Px(3), Px(20), Px(15));
        let mut builder = RenderGraphBuilder::new();
        let mut fragment = RenderFragment::default();
        builder.push_clip_push(outer);
        builder.push_clip_push(PxRect::new(Px(10), Px(0), Px(20), Px(10)));
        fragment.push_platform_view(1);
        builder.append_fragment(&fragment, geometry, 1.0);
        builder.push_clip_pop();
        builder.push_clip_push(PxRect::new(Px(23), Px(3), Px(10), Px(10)));
        fragment.clear();
        fragment.push_platform_view(2);
        builder.append_fragment(&fragment, geometry, 1.0);
        builder.push_clip_pop();
        fragment.clear();
        fragment.push_draw_command(ProbeDraw {
            id: 7,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_platform_view(3);
        builder.append_fragment(&fragment, geometry, 1.0);
        builder.push_clip_pop();

        let slices = builder.finish().platform_view_slices();
        assert_eq!(
            slices.views[0].clip_bounds,
            Some(PxRect::new(Px(10), Px(3), Px(12), Px(7)))
        );
        assert_eq!(slices.views[1].clip_bounds, Some(PxRect::ZERO));
        assert_eq!(slices.views[2].clip_bounds, Some(outer));
        assert_eq!(slices.overlays_after(), [1]);
    }

    #[test]
    fn platform_view_slices_report_shaped_clip_limits() {
        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(40), Px(30)));
        let mut builder = RenderGraphBuilder::new();
        builder.push_shape_clip_push(crate::render::ClipRegion {
            position: PxPosition::ZERO,
            size: [24.0, 18.0],
            shape: crate::render::ClipShape::Ellipse,
        });
        let mut fragment = RenderFragment::default();
        fragment.push_platform_view(5);
        builder.append_fragment(&fragment, geometry, 1.0);

        let slices = builder.finish().platform_view_slices();
        assert_eq!(
            slices.views[0].clip_bounds,
            Some(PxRect::new(Px(0), Px(0), Px(24), Px(18)))
        );
        assert!(slices.views[0].unsupported_shape_clip);
    }

    #[test]
    fn platform_view_draw_slice_can_be_rendered_without_replaying_native_boundaries() {
        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(32), Px(18)));
        let mut builder = RenderGraphBuilder::new();
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(ProbeDraw {
            id: 1,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_platform_view(44);
        builder.append_fragment(&fragment, geometry, 1.0);
        fragment.clear();
        fragment.push_draw_command(ProbeDraw {
            id: 2,
            sample: None,
        });
        builder.append_fragment(&fragment, geometry, 1.0);

        let slices = builder.finish().platform_view_slices();
        let second_slice = slices.draws[1].render_graph();

        assert_eq!(second_slice.ops().len(), 1);
        assert_eq!(
            second_slice.ops()[0]
                .command
                .downcast_draw_ref::<ProbeDraw>()
                .expect("the slice contains the later Tessera draw")
                .id,
            2
        );
        assert!(!second_slice.has_platform_views());
    }

    #[test]
    fn graph_builder_keeps_opaque_commands_and_applies_partial_opacity() {
        #[derive(Clone)]
        struct AlphaDraw(f32);
        impl DrawCommand for AlphaDraw {
            fn apply_opacity(&mut self, opacity: f32) {
                self.0 *= opacity;
            }
        }
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(AlphaDraw(0.4));
        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(8), Px(8)));
        let mut builder = RenderGraphBuilder::with_capacity(2);
        builder.append_fragment(&fragment, geometry, 1.0);
        builder.append_fragment(&fragment, geometry, 0.5);
        let graph = builder.finish();
        let alphas: Vec<_> = graph
            .ops()
            .iter()
            .map(|op| match &op.command {
                Command::Draw(command) => command.downcast_ref::<AlphaDraw>().unwrap().0,
                _ => panic!("expected a draw command"),
            })
            .collect();
        assert_eq!(alphas, [0.4, 0.2]);
        assert_eq!(graph.cached_op_count(), Some(2));
    }

    #[test]
    fn damage_requires_the_rendered_predecessor_and_safe_contents_in_both_frames() {
        #[derive(Clone)]
        struct ResourceDraw {
            _resource: Arc<()>,
        }

        impl DrawCommand for ResourceDraw {
            fn supports_partial_redraw(&self) -> bool {
                true
            }

            fn apply_opacity(&mut self, _: f32) {}
        }

        #[derive(Clone)]
        struct UntrackedResource;

        impl DrawCommand for UntrackedResource {
            fn apply_opacity(&mut self, _: f32) {}
        }

        #[derive(Clone)]
        struct Filter;

        impl ComputeCommand for Filter {
            fn barrier(&self) -> SampleRegion {
                SampleRegion::Global
            }
        }

        let geometry =
            DrawGeometry::from_position_size(PxPosition::ZERO, PxSize::new(Px(8), Px(8)));
        let resource = Arc::new(());
        let resource_lifetime = Arc::downgrade(&resource);
        let first = {
            let mut fragment = RenderFragment::default();
            fragment.push_draw_command(ResourceDraw {
                _resource: resource,
            });
            let mut builder = RenderGraphBuilder::new();
            builder.append_fragment(&fragment, geometry, 1.0);
            builder.finish()
        };
        let damage = PxRect::new(Px(2), Px(3), Px(4), Px(5));
        let second = RenderGraphBuilder::new()
            .finish()
            .with_damage(Some(damage), Some(&first));
        assert_eq!(
            second.damage_since(None),
            None,
            "new targets need all pixels"
        );
        assert_eq!(second.damage_since(Some(&first)), Some(damage));
        assert_eq!(second.damage_since(Some(&second)), Some(PxRect::ZERO));
        let third = RenderGraphBuilder::new()
            .finish()
            .with_damage(Some(damage), Some(&second));
        assert_eq!(
            third.damage_since(Some(&first)),
            None,
            "skipping a scene update loses its damage"
        );

        let sampled = RenderGraphBuilder {
            ops: vec![padded_sample_op(1, 0, 0, 0, 10, 10, 8)],
        }
        .finish()
        .with_damage(Some(damage), Some(&first));
        assert_eq!(
            sampled.damage_since(Some(&first)),
            None,
            "background effects need full replay"
        );
        let removed = RenderGraphBuilder::new()
            .finish()
            .with_damage(Some(damage), Some(&sampled));
        assert_eq!(
            removed.damage_since(Some(&sampled)),
            None,
            "removing an effect must discard its output too"
        );
        let mut fragment = RenderFragment::default();
        fragment.push_draw_command(UntrackedResource);
        let mut builder = RenderGraphBuilder::new();
        builder.append_fragment(&fragment, geometry, 1.0);
        let unknown = builder.finish().with_damage(Some(damage), Some(&first));
        assert_eq!(
            unknown.damage_since(Some(&first)),
            None,
            "custom commands must explicitly opt in"
        );
        fragment.clear();
        fragment.push_compute_command(Filter);
        let mut builder = RenderGraphBuilder::new();
        builder.append_fragment(&fragment, geometry, 1.0);
        let computed = builder.finish().with_damage(Some(damage), Some(&first));
        assert_eq!(
            computed.damage_since(Some(&first)),
            None,
            "compute work must fully reconstruct its input scene"
        );
        drop(first);
        assert!(
            resource_lifetime.upgrade().is_none(),
            "damage history must not keep prior draw resources alive"
        );
    }

    fn draw_op(id: u32, sequence: usize, x: i32, y: i32, w: i32, h: i32) -> RenderGraphOp {
        RenderGraphOp {
            command: Command::Draw(Box::new(ProbeDraw { id, sample: None })),
            type_id: TypeId::of::<ProbeDraw>(),
            geometry: DrawGeometry::from_position_size(
                PxPosition::new(Px::new(x), Px::new(y)),
                PxSize::new(Px::new(w), Px::new(h)),
            ),
            opacity: 1.0,
            sequence_index: sequence,
        }
    }

    fn padded_sample_op(
        id: u32,
        sequence: usize,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        pad: i32,
    ) -> RenderGraphOp {
        let mut op = draw_op(id, sequence, x, y, w, h);
        if let Command::Draw(command) = &mut op.command {
            let probe = command.downcast_mut::<ProbeDraw>().expect("probe draw");
            probe.sample = Some(SampleRegion::PaddedLocal(PaddingRect::uniform(Px::new(
                pad,
            ))));
        }
        op
    }

    fn clip_push(sequence: usize) -> RenderGraphOp {
        RenderGraphOp {
            command: Command::ClipPush(PxRect::new(Px::ZERO, Px::ZERO, Px::new(10), Px::new(10))),
            type_id: TypeId::of::<Command>(),
            geometry: DrawGeometry::new(PxSize::ZERO, LocalToWindow::IDENTITY),
            opacity: 1.0,
            sequence_index: sequence,
        }
    }

    fn clip_pop(sequence: usize) -> RenderGraphOp {
        RenderGraphOp {
            command: Command::ClipPop,
            type_id: TypeId::of::<Command>(),
            geometry: DrawGeometry::new(PxSize::ZERO, LocalToWindow::IDENTITY),
            opacity: 1.0,
            sequence_index: sequence,
        }
    }

    fn probe_id(op: &RenderGraphOp) -> u32 {
        match &op.command {
            Command::Draw(command) => command.downcast_ref::<ProbeDraw>().expect("probe").id,
            _ => u32::MAX,
        }
    }

    #[test]
    fn disjoint_clip_siblings_keep_every_draw_in_its_original_clip_scope() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ops = Vec::new();
        let mut sequence = 0usize;
        for clip in 0..8 {
            ops.push(clip_push(sequence));
            sequence += 1;
            let x = (clip as i32) * 100;
            ops.push(draw_op(clip * 2, sequence, x, 0, 20, 20));
            sequence += 1;
            ops.push(draw_op(clip * 2 + 1, sequence, x + 5, 5, 20, 20));
            sequence += 1;
            ops.push(clip_pop(sequence));
            sequence += 1;
        }

        let ordered = order_ops(ops);
        assert_eq!(ordered.len(), 32);
        for (scope, commands) in ordered.chunks_exact(4).enumerate() {
            assert!(matches!(commands[0].command, Command::ClipPush(_)));
            assert_eq!(probe_id(&commands[1]), (scope * 2) as u32);
            assert_eq!(probe_id(&commands[2]), (scope * 2 + 1) as u32);
            assert!(matches!(commands[3].command, Command::ClipPop));
        }
    }

    #[test]
    fn padded_local_sampling_cannot_move_before_the_draw_it_samples() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ops = vec![draw_op(1, 0, 14, 0, 10, 10)];
        for i in 1..24 {
            ops.push(draw_op(i as u32 + 2, i, i as i32 * 50, 0, 10, 10));
        }
        ops.push(padded_sample_op(2, 24, 0, 0, 10, 10, 8));

        let ordered = order_ops(ops);
        assert_eq!(probe_id(&ordered[0]), 1);
        assert_eq!(probe_id(&ordered[1]), 2);
    }

    #[test]
    fn touching_a_sample_region_does_not_prevent_reordering() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let earlier = draw_op(1, 0, 18, 0, 10, 10);
        let later = padded_sample_op(2, 1, 0, 0, 10, 10, 8);
        let ordered = order_ops(vec![earlier, later]);
        assert_eq!(probe_id(&ordered[0]), 2);
        assert_eq!(probe_id(&ordered[1]), 1);
    }

    #[test]
    fn ready_draws_continue_a_batch_until_an_overlapping_draw_blocks_it() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ops = vec![
            draw_op(1, 0, 0, 0, 10, 10),
            draw_op(2, 1, 20, 0, 10, 10),
            draw_op(3, 2, 40, 0, 10, 10),
            draw_op(4, 3, 60, 0, 10, 10),
            draw_op(5, 4, 20, 0, 10, 10),
            draw_op(6, 5, 80, 0, 10, 10),
        ];
        for index in [1, 3, 5] {
            ops[index].type_id = TypeId::of::<u32>();
        }

        let ordered = order_ops(ops);
        let ids: Vec<_> = ordered.iter().map(probe_id).collect();
        assert_eq!(ids, [1, 3, 2, 4, 6, 5]);
    }

    #[test]
    fn densely_overlapping_draws_fall_back_to_author_order() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut ops = vec![draw_op(0, 0, 100, 0, 10, 10)];
        ops.push(padded_sample_op(1, 1, 200, 0, 10, 10, 5));
        for i in 2..1026 {
            ops.push(draw_op(i as u32, i, 0, 0, 10, 10));
        }

        let ordered = order_ops(ops);
        assert!(ordered.iter().map(probe_id).eq(0..1026));
    }

    #[test]
    fn unclosed_clip_falls_back_to_sequence() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let ops = vec![
            clip_push(0),
            draw_op(7, 1, 0, 0, 10, 10),
            draw_op(8, 2, 0, 0, 10, 10),
        ];
        let ordered = order_ops(ops);
        assert_eq!(probe_id(&ordered[1]), 7);
        assert_eq!(probe_id(&ordered[2]), 8);
        assert!(matches!(ordered[0].command, Command::ClipPush(_)));
    }
}
