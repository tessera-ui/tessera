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
    Command, ComputeCommand, DrawCommand, DrawRegion, FloatRect, LocalToWindow, SampleRegion,
    component_tree::SceneRope,
    px::{Px, PxPosition, PxRect, PxSize},
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
        self.padded_window_bounds(crate::PaddingRect::ZERO)
    }

    /// Maps local padding to target bounds without rounding individual edges.
    #[must_use]
    pub fn padded_window_bounds(self, padding: crate::PaddingRect) -> FloatRect {
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

/// Frame-level render graph.
#[derive(Clone, Default)]
pub struct RenderGraph {
    data: Arc<RenderGraphData>,
    damage: Option<PxRect>,
    previous: Option<Weak<RenderGraphData>>,
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
    pub(crate) fn from_partial_scene(scene: Arc<SceneRope>) -> Self {
        Self {
            data: Arc::new(RenderGraphData {
                scene: Some(scene),
                ..RenderGraphData::default()
            }),
            damage: None,
            previous: None,
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
    pub(crate) fn push_shape_clip_push(&mut self, region: crate::ClipRegion) {
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
            if let Command::Draw(ref mut command) = command {
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
            Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop => {
                OpCategory::StateChange
            }
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
        Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop => None,
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
        Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop => None,
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
        Command::ClipPush(_) | Command::ShapeClipPush(_) | Command::ClipPop
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PaddingRect, Px, PxPosition, PxSize, SampleRegion, renderer::drawer::command::DrawCommand,
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
                LocalToWindow::new(0.625, PxPosition::from_f32_xy(10.25, 20.5)).unwrap(),
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
