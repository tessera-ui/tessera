//! Render graph fragments and nodes for Tessera.
//!
//! ## Usage
//!
//! Build per-component render fragments and merge them into a frame graph.

#![allow(dead_code)]

use std::{
    any::TypeId,
    collections::{BinaryHeap, HashMap},
};

use smallvec::SmallVec;

use crate::{
    Command, ComputeCommand, DrawCommand, DrawRegion, SampleRegion,
    px::{Px, PxPosition, PxRect, PxSize},
};

/// Resource identifier used by render graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderResourceId {
    /// The main scene color buffer.
    SceneColor,
    /// The main scene depth buffer.
    SceneDepth,
    /// A local texture allocated by a fragment.
    Local(u32),
}

/// Descriptor for a local render texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTextureDesc {
    /// Pixel size of the texture.
    pub size: PxSize,
    /// Texture format.
    pub format: wgpu::TextureFormat,
}

/// Render graph resource description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderResource {
    /// A texture resource allocated for a fragment.
    Texture(RenderTextureDesc),
}

/// A single render op inside a fragment graph.
#[derive(Clone)]
pub struct RenderFragmentOp {
    /// The command to execute.
    pub command: Command,
    /// Type identifier used for batching.
    pub type_id: TypeId,
    /// Resource read by this op.
    pub read: Option<RenderResourceId>,
    /// Resource written by this op.
    pub write: Option<RenderResourceId>,
    /// Local dependencies inside the fragment.
    pub deps: SmallVec<[u32; 2]>,
    /// Optional size override for this op.
    pub size_override: Option<PxSize>,
    /// Optional position override for this op (SceneColor ops treat it as
    /// offset).
    pub position_override: Option<PxPosition>,
}

/// A per-component render fragment.
#[derive(Default, Clone)]
pub struct RenderFragment {
    ops: Vec<RenderFragmentOp>,
    resources: Vec<RenderResource>,
}

impl RenderFragment {
    /// Returns fragment ops in insertion order.
    #[must_use]
    pub fn ops(&self) -> &[RenderFragmentOp] {
        &self.ops
    }

    /// Returns local resources declared by this fragment.
    #[must_use]
    pub fn resources(&self) -> &[RenderResource] {
        &self.resources
    }

    /// Adds a local texture resource to the fragment.
    pub fn add_local_texture(&mut self, desc: RenderTextureDesc) -> RenderResourceId {
        let index = self.resources.len() as u32;
        self.resources.push(RenderResource::Texture(desc));
        RenderResourceId::Local(index)
    }

    /// Adds a draw command with default scene resource bindings.
    pub fn push_draw_command<C: DrawCommand + 'static>(&mut self, command: C) -> u32 {
        let type_id = TypeId::of::<C>();
        let read = command
            .sample_region()
            .is_some()
            .then_some(RenderResourceId::SceneColor);
        let write = Some(RenderResourceId::SceneColor);

        let op = RenderFragmentOp {
            command: Command::Draw(Box::new(command)),
            type_id,
            read,
            write,
            deps: SmallVec::new(),
            size_override: None,
            position_override: None,
        };
        self.push_op(op)
    }

    /// Adds a compute command with default scene resource bindings.
    pub fn push_compute_command<C: ComputeCommand + 'static>(&mut self, command: C) -> u32 {
        let type_id = TypeId::of::<C>();
        let read = Some(RenderResourceId::SceneColor);
        let write = Some(RenderResourceId::SceneColor);

        let op = RenderFragmentOp {
            command: Command::Compute(Box::new(command)),
            type_id,
            read,
            write,
            deps: SmallVec::new(),
            size_override: None,
            position_override: None,
        };
        self.push_op(op)
    }

    /// Adds a render op to the fragment.
    pub fn push_op(&mut self, op: RenderFragmentOp) -> u32 {
        let index = self.ops.len() as u32;
        self.ops.push(op);
        index
    }

    /// Clears all ops and resources in the fragment.
    pub fn clear(&mut self) {
        self.ops.clear();
        self.resources.clear();
    }
}

/// A render op after merging into the frame graph.
#[derive(Clone)]
pub struct RenderGraphOp {
    /// The command to execute.
    pub command: Command,
    /// Type identifier used for batching and ordering.
    pub type_id: TypeId,
    /// Resource read for the op.
    pub read: Option<RenderResourceId>,
    /// Resource write for the op.
    pub write: Option<RenderResourceId>,
    /// Dependencies on other ops by index.
    pub deps: SmallVec<[usize; 2]>,
    /// Measured size of the op.
    pub size: PxSize,
    /// Absolute position of the op.
    pub position: PxPosition,
    /// Opacity multiplier applied during record.
    pub opacity: f32,
    /// Sequence index used to preserve authoring order.
    pub sequence_index: usize,
}

/// Frame-level render graph.
#[derive(Default)]
pub struct RenderGraph {
    ops: Vec<RenderGraphOp>,
    resources: Vec<RenderResource>,
}

/// Owned render graph payload for middleware transforms.
pub struct RenderGraphParts {
    /// Render ops for the frame.
    pub ops: Vec<RenderGraphOp>,
    /// Resource declarations referenced by the ops.
    pub resources: Vec<RenderResource>,
}

impl RenderGraph {
    /// Returns ops in the graph.
    #[must_use]
    pub fn ops(&self) -> &[RenderGraphOp] {
        &self.ops
    }

    /// Returns local resources in the graph.
    #[must_use]
    pub fn resources(&self) -> &[RenderResource] {
        &self.resources
    }

    /// Decomposes the graph into owned parts for middleware processing.
    #[must_use]
    pub fn into_parts(self) -> RenderGraphParts {
        RenderGraphParts {
            ops: self.ops,
            resources: self.resources,
        }
    }

    /// Builds a render graph from owned parts.
    #[must_use]
    pub fn from_parts(parts: RenderGraphParts) -> Self {
        Self {
            ops: parts.ops,
            resources: parts.resources,
        }
    }

    /// Consumes the graph and returns an execution-ready payload.
    pub(crate) fn into_execution(self) -> RenderGraphExecution {
        RenderGraphExecution {
            ops: order_ops(self.ops),
            resources: self.resources,
        }
    }
}

/// Ordered render ops with resource declarations for the current frame.
pub(crate) struct RenderGraphExecution {
    pub(crate) ops: Vec<RenderGraphOp>,
    pub(crate) resources: Vec<RenderResource>,
}

/// Builder for a frame-level render graph.
pub(crate) struct RenderGraphBuilder {
    ops: Vec<RenderGraphOp>,
    resources: Vec<RenderResource>,
    sequence_index: usize,
}

impl RenderGraphBuilder {
    /// Creates a new render graph builder.
    pub(crate) fn new() -> Self {
        Self {
            ops: Vec::new(),
            resources: Vec::new(),
            sequence_index: 0,
        }
    }

    /// Pushes a clip push op into the graph.
    pub(crate) fn push_clip_push(&mut self, rect: PxRect) {
        self.ops.push(RenderGraphOp {
            command: Command::ClipPush(rect),
            type_id: TypeId::of::<Command>(),
            read: None,
            write: None,
            deps: SmallVec::new(),
            size: PxSize::ZERO,
            position: PxPosition::ZERO,
            opacity: 1.0,
            sequence_index: self.sequence_index,
        });
        self.sequence_index += 1;
    }

    /// Pushes a clip pop op into the graph.
    pub(crate) fn push_clip_pop(&mut self) {
        self.ops.push(RenderGraphOp {
            command: Command::ClipPop,
            type_id: TypeId::of::<Command>(),
            read: None,
            write: None,
            deps: SmallVec::new(),
            size: PxSize::ZERO,
            position: PxPosition::ZERO,
            opacity: 1.0,
            sequence_index: self.sequence_index,
        });
        self.sequence_index += 1;
    }

    /// Appends a fragment into the frame graph.
    pub(crate) fn append_fragment(
        &mut self,
        mut fragment: RenderFragment,
        size: PxSize,
        position: PxPosition,
        opacity: f32,
    ) {
        if fragment.ops.is_empty() {
            return;
        }

        let mut resource_map: Vec<RenderResourceId> = Vec::with_capacity(fragment.resources.len());
        for resource in fragment.resources.drain(..) {
            let index = self.resources.len() as u32;
            self.resources.push(resource);
            resource_map.push(RenderResourceId::Local(index));
        }

        let base_index = self.ops.len();

        for mut op in fragment.ops.drain(..) {
            let writes_scene = op.write == Some(RenderResourceId::SceneColor);
            let position_override = op.position_override.unwrap_or(PxPosition::ZERO);
            let size_override = op.size_override.unwrap_or(size);

            let read = op.read.map(|r| map_resource(r, &resource_map));
            let write = op.write.map(|w| map_resource(w, &resource_map));
            let deps = op
                .deps
                .iter()
                .map(|dep| base_index + *dep as usize)
                .collect::<SmallVec<[usize; 2]>>();

            if let Command::Draw(ref mut command) = op.command {
                command.apply_opacity(opacity);
            }

            let resolved_position = if writes_scene {
                position + position_override
            } else {
                position_override
            };

            self.ops.push(RenderGraphOp {
                command: op.command,
                type_id: op.type_id,
                read,
                write,
                deps,
                size: size_override,
                position: resolved_position,
                opacity,
                sequence_index: self.sequence_index,
            });
            self.sequence_index += 1;
        }
    }

    /// Finishes graph construction.
    pub(crate) fn finish(self) -> RenderGraph {
        RenderGraph {
            ops: self.ops,
            resources: self.resources,
        }
    }
}

fn map_resource(resource: RenderResourceId, local_map: &[RenderResourceId]) -> RenderResourceId {
    match resource {
        RenderResourceId::Local(index) => local_map
            .get(index as usize)
            .copied()
            .unwrap_or(RenderResourceId::Local(index)),
        other => other,
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

    for (idx, op) in ops.iter().enumerate() {
        for dep in &op.deps {
            add_edge(&mut outgoing, &mut in_degree, *dep, idx);
        }
    }

    let mut by_sequence: Vec<usize> = (0..ops.len()).collect();
    by_sequence.sort_by_key(|idx| ops[*idx].sequence_index);
    for (offset, &left) in by_sequence.iter().enumerate() {
        for &right in &by_sequence[offset + 1..] {
            if needs_ordering(&infos[left], &infos[right], &ops[left], &ops[right]) {
                add_edge(&mut outgoing, &mut in_degree, left, right);
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
        let mut fallback = ops;
        fallback.sort_by_key(|op| op.sequence_index);
        return fallback;
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
            Command::ClipPush(_) | Command::ClipPop => OpCategory::StateChange,
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
    if left.category == OpCategory::StateChange || right.category == OpCategory::StateChange {
        return true;
    }

    if non_scene_conflict(left_op, right_op) {
        return true;
    }

    scene_conflict(left, right)
}

fn non_scene_conflict(left: &RenderGraphOp, right: &RenderGraphOp) -> bool {
    if let Some(resource) = left.write {
        if resource == RenderResourceId::SceneColor {
            return false;
        }
        if Some(resource) == right.read || Some(resource) == right.write {
            return true;
        }
    }

    if let Some(resource) = left.read {
        if resource == RenderResourceId::SceneColor {
            return false;
        }
        if Some(resource) == right.write {
            return true;
        }
    }

    false
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
    if op.read != Some(RenderResourceId::SceneColor) {
        return None;
    }
    match &op.command {
        Command::Draw(command) => command
            .sample_region()
            .map(|region| sample_region_rect(region, op.position, op.size)),
        Command::Compute(command) => {
            Some(sample_region_rect(command.barrier(), op.position, op.size))
        }
        Command::ClipPush(_) | Command::ClipPop => None,
    }
}

fn scene_write_rect(op: &RenderGraphOp) -> Option<PxRect> {
    if op.write != Some(RenderResourceId::SceneColor) {
        return None;
    }
    match &op.command {
        Command::Draw(command) => Some(
            command
                .ordering_rect(op.position, op.size)
                .unwrap_or_else(|| draw_region_rect(command.draw_region(), op.position, op.size)),
        ),
        Command::Compute(_) => Some(PxRect::from_position_size(op.position, op.size)),
        Command::ClipPush(_) | Command::ClipPop => None,
    }
}

fn sample_region_rect(region: SampleRegion, position: PxPosition, size: PxSize) -> PxRect {
    match region {
        SampleRegion::Global => PxRect::new(Px::ZERO, Px::ZERO, Px::MAX, Px::MAX),
        SampleRegion::PaddedLocal(_) => {
            // Use component bounds to avoid padded sampling regions forcing dependencies.
            PxRect::from_position_size(position, size)
        }
        SampleRegion::Absolute(rect) => rect,
    }
}

fn draw_region_rect(region: DrawRegion, position: PxPosition, size: PxSize) -> PxRect {
    match region {
        DrawRegion::Global => PxRect::new(Px::ZERO, Px::ZERO, Px::MAX, Px::MAX),
        DrawRegion::PaddedLocal(padding) => padded_rect(position, size, padding),
        DrawRegion::Absolute(rect) => rect,
    }
}

fn padded_rect(position: PxPosition, size: PxSize, padding: crate::PaddingRect) -> PxRect {
    PxRect::new(
        position.x - padding.left,
        position.y - padding.top,
        size.width + padding.left + padding.right,
        size.height + padding.top + padding.bottom,
    )
}
