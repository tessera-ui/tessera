//! Render graph fragments and nodes for Tessera.
//!
//! ## Usage
//!
//! Build per-component render fragments and merge them into a frame graph.

use std::{
    any::TypeId,
    collections::{BinaryHeap, HashMap},
    sync::{Arc, OnceLock},
};

use smallvec::SmallVec;

use crate::{
    Command, ComputeCommand, DrawCommand, DrawRegion, FloatRect, LocalToWindow, SampleRegion,
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
    ops: Vec<RenderFragmentOp>,
}

impl RenderFragment {
    /// Returns fragment ops in insertion order.
    #[must_use]
    pub fn ops(&self) -> &[RenderFragmentOp] {
        &self.ops
    }

    /// Returns fragment ops in insertion order for local position overrides.
    #[must_use]
    pub fn ops_mut(&mut self) -> &mut [RenderFragmentOp] {
        &mut self.ops
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
        let index = self.ops.len() as u32;
        self.ops.push(RenderFragmentOp {
            command,
            type_id,
            size_override: None,
            position_override: None,
        });
        index
    }

    /// Clears all ops in the fragment.
    pub fn clear(&mut self) {
        self.ops.clear();
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
}

#[derive(Default)]
struct RenderGraphData {
    ordered_ops: OnceLock<Vec<RenderGraphOp>>,
    ops: Vec<RenderGraphOp>,
}

impl RenderGraph {
    /// Returns ops in the graph.
    #[must_use]
    pub fn ops(&self) -> &[RenderGraphOp] {
        &self.data.ops
    }

    /// Returns the ops in execution order.
    pub(crate) fn ordered_ops(&self) -> Vec<RenderGraphOp> {
        self.data
            .ordered_ops
            .get_or_init(|| order_ops(self.data.ops.clone()))
            .clone()
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
        for op in &fragment.ops {
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
                ordered_ops: OnceLock::new(),
                ops: self.ops,
            }),
        }
    }
}

fn order_ops(ops: Vec<RenderGraphOp>) -> Vec<RenderGraphOp> {
    if ops.is_empty() {
        return ops;
    }

    let infos: Vec<OpInfo> = ops.iter().map(OpInfo::new).collect();
    let mut potentials: HashMap<(OpCategory, TypeId), usize> = HashMap::new();
    for info in &infos {
        *potentials.entry((info.category, info.type_id)).or_insert(0) += 1;
    }

    let mut outgoing: Vec<SmallVec<[usize; 4]>> = vec![SmallVec::new(); ops.len()];
    let mut in_degree = vec![0usize; ops.len()];

    let by_sequence = sequence_order(&ops);

    // State changes (clip push/pop) act as sequence-ordered
    // barriers. Without this, ops in different clip scopes have no ordering
    // edge and can be hoisted ahead of the background they must paint on top
    // of, which reorders overlapping content incorrectly.
    for (index, op) in ops.iter().enumerate() {
        if !is_state_change(op) {
            continue;
        }
        for other in 0..ops.len() {
            if other == index {
                continue;
            }
            if index < other {
                add_edge(&mut outgoing, &mut in_degree, index, other);
            } else {
                add_edge(&mut outgoing, &mut in_degree, other, index);
            }
        }
    }

    let Some(intervals) = clip_intervals(&ops, &by_sequence) else {
        return fallback_by_sequence(ops);
    };

    for interval in &intervals {
        let members = &by_sequence[interval.start..interval.end];
        let member_count = members.len();
        let mut left_offset = 0;
        while left_offset < member_count {
            let left = members[left_offset];
            let mut right_offset = left_offset + 1;
            while right_offset < member_count {
                let right = members[right_offset];
                if needs_ordering(&infos[left], &infos[right], &ops[left], &ops[right]) {
                    add_edge(&mut outgoing, &mut in_degree, left, right);
                }
                right_offset += 1;
            }
            left_offset += 1;
        }
        if let Some(pred) = interval.predecessor {
            for &member in members {
                add_edge(&mut outgoing, &mut in_degree, pred, member);
            }
        }
    }

    let mut ready = BinaryHeap::new();
    for (idx, degree) in in_degree.iter().enumerate() {
        if *degree == 0 {
            ready.push(PriorityNode::new(idx, &infos, &potentials));
        }
    }

    let mut ordered_indices = Vec::with_capacity(ops.len());
    let mut last_type_id: Option<TypeId> = None;

    while !ready.is_empty() {
        let highest_category = ready.peek().map(|node| node.category);
        let mut selected: Option<PriorityNode> = None;
        if let (Some(last_type), Some(high_cat)) = (last_type_id, highest_category) {
            let mut deferred = Vec::new();
            while let Some(node) = ready.pop() {
                if node.category == high_cat && node.type_id == last_type {
                    selected = Some(node);
                    break;
                }
                deferred.push(node);
            }
            for node in deferred {
                ready.push(node);
            }
        }

        let priority_node = selected.unwrap_or_else(|| {
            ready
                .pop()
                .expect("ready queue should not be empty while sorting ops")
        });
        let u = priority_node.node_index;
        ordered_indices.push(u);
        match priority_node.category {
            OpCategory::StateChange => last_type_id = None,
            _ => last_type_id = Some(priority_node.type_id),
        }

        for &next in &outgoing[u] {
            in_degree[next] -= 1;
            if in_degree[next] == 0 {
                ready.push(PriorityNode::new(next, &infos, &potentials));
            }
        }
    }

    if ordered_indices.len() != ops.len() {
        return fallback_by_sequence(ops);
    }

    let mut ops = ops;
    let mut ops_by_index: Vec<Option<RenderGraphOp>> = ops.drain(..).map(Some).collect();
    let mut ordered = Vec::with_capacity(ordered_indices.len());
    for index in ordered_indices {
        if let Some(op) = ops_by_index[index].take() {
            ordered.push(op);
        }
    }

    ordered
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum OpCategory {
    ContinuationDraw,
    BarrierDraw,
    Compute,
    StateChange,
}

struct OpInfo {
    category: OpCategory,
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
            type_id: op.type_id,
            sequence_index: op.sequence_index,
            read_rect: scene_read_rect(op),
            write_rect: scene_write_rect(op),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PriorityNode {
    category: OpCategory,
    type_id: TypeId,
    original_index: usize,
    node_index: usize,
    batch_potential: usize,
}

impl PriorityNode {
    fn new(
        node_index: usize,
        infos: &[OpInfo],
        potentials: &HashMap<(OpCategory, TypeId), usize>,
    ) -> Self {
        let info = &infos[node_index];
        let batch_potential = potentials
            .get(&(info.category, info.type_id))
            .copied()
            .unwrap_or(1);
        Self {
            category: info.category,
            type_id: info.type_id,
            original_index: info.sequence_index,
            node_index,
            batch_potential,
        }
    }
}

impl Ord for PriorityNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.category
            .cmp(&other.category)
            .then_with(|| other.batch_potential.cmp(&self.batch_potential))
            .then_with(|| other.original_index.cmp(&self.original_index))
    }
}

impl PartialOrd for PriorityNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn add_edge(
    outgoing: &mut [SmallVec<[usize; 4]>],
    in_degree: &mut [usize],
    from: usize,
    to: usize,
) {
    if from == to || outgoing[from].contains(&to) {
        return;
    }
    outgoing[from].push(to);
    in_degree[to] += 1;
}

fn needs_ordering(
    left: &OpInfo,
    right: &OpInfo,
    left_op: &RenderGraphOp,
    right_op: &RenderGraphOp,
) -> bool {
    is_state_change(left_op) || is_state_change(right_op) || scene_conflict(left, right)
}

fn scene_conflict(left: &OpInfo, right: &OpInfo) -> bool {
    if let (Some(write_left), Some(write_right)) = (left.write_rect, right.write_rect)
        && !write_left.is_orthogonal(&write_right)
    {
        return true;
    }
    if let (Some(write_left), Some(read_right)) = (left.write_rect, right.read_rect)
        && !write_left.is_orthogonal(&read_right)
    {
        return true;
    }
    if let (Some(read_left), Some(write_right)) = (left.read_rect, right.write_rect)
        && !read_left.is_orthogonal(&write_right)
    {
        return true;
    }
    false
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

fn sequence_order(ops: &[RenderGraphOp]) -> Vec<usize> {
    let mut by_sequence: Vec<usize> = (0..ops.len()).collect();
    by_sequence.sort_by_key(|idx| ops[*idx].sequence_index);
    by_sequence
}

fn fallback_by_sequence(mut ops: Vec<RenderGraphOp>) -> Vec<RenderGraphOp> {
    ops.sort_by_key(|op| op.sequence_index);
    ops
}

struct ClipInterval {
    start: usize,
    end: usize,
    predecessor: Option<usize>,
}

fn clip_intervals(ops: &[RenderGraphOp], by_sequence: &[usize]) -> Option<Vec<ClipInterval>> {
    let mut intervals = Vec::new();
    let mut stack_start: Vec<usize> = vec![0];
    let mut stack_pred: Vec<Option<usize>> = vec![None];

    for (cursor, &idx) in by_sequence.iter().enumerate() {
        match &ops[idx].command {
            Command::ClipPush(_) | Command::ShapeClipPush(_) => {
                close_open_span(&mut intervals, &stack_start, &stack_pred, cursor);
                stack_start.push(cursor + 1);
                stack_pred.push(Some(idx));
            }
            Command::ClipPop => {
                if stack_start.len() <= 1 {
                    return None;
                }
                let start = stack_start.pop().expect("clip stack checked above");
                let predecessor = stack_pred.pop().expect("clip stack checked above");
                if cursor > start {
                    intervals.push(ClipInterval {
                        start,
                        end: cursor,
                        predecessor,
                    });
                }
                let pop_pred = if cursor > start {
                    Some(by_sequence[cursor - 1])
                } else {
                    predecessor
                };
                intervals.push(ClipInterval {
                    start: cursor,
                    end: cursor + 1,
                    predecessor: pop_pred,
                });
                let parent = stack_start.len() - 1;
                stack_start[parent] = cursor + 1;
                stack_pred[parent] = Some(idx);
            }
            _ => {}
        }
    }

    if stack_start.len() != 1 {
        return None;
    }
    close_open_span(&mut intervals, &stack_start, &stack_pred, by_sequence.len());
    Some(intervals)
}

fn close_open_span(
    intervals: &mut Vec<ClipInterval>,
    stack_start: &[usize],
    stack_pred: &[Option<usize>],
    cursor: usize,
) {
    let parent = stack_start.len() - 1;
    let start = stack_start[parent];
    if cursor > start {
        intervals.push(ClipInterval {
            start,
            end: cursor,
            predecessor: stack_pred[parent],
        });
    }
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
        fn sample_region(&self) -> Option<SampleRegion> {
            self.sample
        }

        fn apply_opacity(&mut self, _opacity: f32) {}
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

    fn ordering_edge_count(ops: &[RenderGraphOp]) -> usize {
        let infos: Vec<OpInfo> = ops.iter().map(OpInfo::new).collect();
        let by_sequence = sequence_order(ops);
        let intervals = clip_intervals(ops, &by_sequence).expect("closed clips");
        let mut outgoing: Vec<SmallVec<[usize; 4]>> = vec![SmallVec::new(); ops.len()];
        let mut in_degree = vec![0usize; ops.len()];
        for interval in &intervals {
            let members = &by_sequence[interval.start..interval.end];
            let member_count = members.len();
            let mut left_offset = 0;
            while left_offset < member_count {
                let left = members[left_offset];
                let mut right_offset = left_offset + 1;
                while right_offset < member_count {
                    let right = members[right_offset];
                    if needs_ordering(&infos[left], &infos[right], &ops[left], &ops[right]) {
                        add_edge(&mut outgoing, &mut in_degree, left, right);
                    }
                    right_offset += 1;
                }
                left_offset += 1;
            }
            if let Some(pred) = interval.predecessor {
                for &member in members {
                    add_edge(&mut outgoing, &mut in_degree, pred, member);
                }
            }
        }
        outgoing.iter().map(SmallVec::len).sum()
    }

    fn legacy_state_change_edges(ops: &[RenderGraphOp]) -> usize {
        let count = ops.len();
        count.saturating_mul(count.saturating_sub(1)) / 2
    }

    fn probe_id(op: &RenderGraphOp) -> u32 {
        match &op.command {
            Command::Draw(command) => command.downcast_ref::<ProbeDraw>().expect("probe").id,
            _ => u32::MAX,
        }
    }

    #[test]
    fn disjoint_clip_siblings_drop_cross_edges_but_keep_overlap_order() {
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

        let edges = ordering_edge_count(&ops);
        let legacy = legacy_state_change_edges(&ops);
        assert!(
            edges < legacy / 4,
            "clip intervals should cut edges: {edges} vs legacy {legacy}"
        );

        let ordered = order_ops(ops);
        let ids: Vec<u32> = ordered
            .iter()
            .map(probe_id)
            .filter(|id| *id != u32::MAX)
            .collect();
        assert_eq!(ids, (0..16).collect::<Vec<_>>());
    }

    #[test]
    fn padded_local_sampling_keeps_padding_overlap() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let left = padded_sample_op(1, 0, 0, 0, 10, 10, 8);
        let right = draw_op(2, 1, 14, 0, 10, 10);
        let infos = [OpInfo::new(&left), OpInfo::new(&right)];
        let read = infos[0].read_rect.expect("padded read rect");
        assert_eq!(read.x, Px::new(-8));
        assert_eq!(read.width, Px::new(26));
        assert!(
            scene_conflict(&infos[0], &infos[1]),
            "padding overlap must still order the later draw"
        );

        let ordered = order_ops(vec![left, right]);
        assert_eq!(probe_id(&ordered[0]), 1);
        assert_eq!(probe_id(&ordered[1]), 2);
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
