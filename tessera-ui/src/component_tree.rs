mod constraint;
mod node;
mod pointer;
mod scene;

use std::{collections::VecDeque, num::NonZero, sync::Arc};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::SmallVec;
use tracing::{debug, warn};

use crate::{
    ComputeResourceManager, NodeId, PxRect,
    clipboard::{self, ClipboardInput, ClipboardResponse, QueuedClipboardAction},
    cursor::{CursorEventContent, MOUSE_POINTER_ID, PointerChange},
    focus::{
        FocusDirection, FocusHandleId, FocusOwner, PendingFocusCallbackInvocation, bind_focus_owner,
    },
    keyboard_state::QueuedKeyEvent,
    layout::{LayoutResult, RenderInput},
    modifier::{
        DrawModifierContent, DrawModifierContext, ImeInputModifierNode, KeyboardInputModifierNode,
        OrderedModifierAction, PointerInputModifierNode,
    },
    px::{PxPosition, PxSize},
    render_graph::RenderGraph,
    runtime::{
        LayoutDirtyNodes, RuntimePhase, StructureReconcileResult, TesseraRuntime,
        push_current_component_instance_key, push_current_node_with_instance_logic_id, push_phase,
    },
    time::Instant,
};

pub use constraint::{AxisConstraint, Constraint, ParentConstraint};
pub use node::{
    ComputedData, GloballyPositionedHandler, ImeInput, ImeInputHandlerFn, ImeRequest, ImeSession,
    KeyboardInput, KeyboardInputHandlerFn, MeasurementError, PointerEventPass, PointerInput,
    PointerInputHandlerFn,
};

pub(crate) use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, NodeRole,
    WindowRequests, direct_layout_children, measure_node,
};

#[cfg(feature = "profiling")]
use crate::profiler::{NodeMeta, Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard};

#[derive(Clone)]
pub(crate) struct LayoutSnapshotEntry {
    pub constraint_key: Constraint,
    pub layout_result: LayoutResult,
    pub base_placements: Arc<[(u64, PxPosition)]>,
    pub child_constraints: Arc<[Constraint]>,
    pub child_sizes: Arc<[ComputedData]>,
    pub child_indices: Arc<HashMap<u64, usize>>,
    pub intrinsic_dependencies: bool,
    pub draw_layout: NodeDrawLayout,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawLayoutBounds {
    pub position: PxPosition,
    pub size: ComputedData,
}

#[derive(Clone, PartialEq)]
pub(crate) struct NodeDrawLayout {
    // Cached frames share immutable layer bounds instead of allocating a copy
    // for every restored node.
    pub layers: Option<Arc<[Option<DrawLayoutBounds>]>>,
    pub base_constraint: Constraint,
    pub base_size: ComputedData,
}

impl Default for NodeDrawLayout {
    fn default() -> Self {
        Self {
            layers: None,
            base_constraint: Constraint::NONE,
            base_size: ComputedData::ZERO,
        }
    }
}

#[derive(Default)]
pub(crate) struct LayoutSnapshotMap {
    entries: HashMap<u64, Arc<LayoutSnapshotEntry>>,
}

impl LayoutSnapshotMap {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn remove(&mut self, instance_key: &u64) -> Option<Arc<LayoutSnapshotEntry>> {
        self.entries.remove(instance_key)
    }

    pub(crate) fn insert(
        &mut self,
        instance_key: u64,
        entry: LayoutSnapshotEntry,
    ) -> Option<Arc<LayoutSnapshotEntry>> {
        self.entries.insert(instance_key, Arc::new(entry))
    }

    pub(crate) fn get_cloned(&self, instance_key: &u64) -> Option<Arc<LayoutSnapshotEntry>> {
        self.entries.get(instance_key).cloned()
    }
}

pub(crate) fn nearest_replay_boundary_instance_key(
    node_id: NodeId,
    tree: &ComponentNodeTree,
) -> u64 {
    let mut current_id = node_id;
    let mut fallback = 0;

    loop {
        let Some(node_ref) = tree.get(current_id) else {
            return fallback;
        };
        let node = node_ref.get();
        if fallback == 0 {
            fallback = node.instance_key;
        }
        if node.replay.is_some() {
            return node.instance_key;
        }
        let Some(parent_id) = node_ref.parent() else {
            return fallback;
        };
        current_id = parent_id;
    }
}

pub(crate) fn clear_layout_snapshots() {
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.clear_layout_snapshots();
    });
}

/// Invokes globally positioned callbacks collected by the last compute pass.
///
/// Must be called after the runtime borrow used by `compute` is released so
/// callbacks can freely update state.
pub(crate) fn flush_pending_globally_positioned_callbacks() {
    let invocations = TesseraRuntime::with_mut(|runtime| {
        runtime
            .component_tree
            .take_pending_globally_positioned_invocations()
    });
    for (handler, rect) in invocations {
        handler.call(rect);
    }
    TesseraRuntime::with_mut(|runtime| runtime.component_tree.focus_owner_mut().finish_layout());
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutContext<'a> {
    pub state_reader_instance_key: Option<u64>,
    snapshots: *mut LayoutSnapshotMap,
    pub measure_self_nodes: &'a HashSet<u64>,
    pub placement_self_nodes: &'a HashSet<u64>,
    pub dirty_effective_nodes: &'a HashSet<u64>,
    pub dirty_children: Option<&'a HashMap<u64, Vec<NodeId>>>,
    diagnostics: *mut LayoutDiagnosticsCollector,
    pub intrinsic: bool,
}

#[cfg_attr(not(feature = "profiling"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default)]
/// Layout work observed during one frame.
pub struct LayoutFrameDiagnostics {
    /// Number of nodes whose window geometry was resolved in this frame.
    pub geometry_nodes_updated: u64,
    /// Geometry traversal entries, including entries that immediately reused a
    /// cache.
    pub geometry_nodes_visited: u64,
    /// Dirty direct children checked for cached size dependencies.
    pub measure_dirty_children_checked: u64,
    /// Nodes examined while refreshing pointer geometry and dispatch plans.
    pub pointer_nodes_updated: u64,
    /// Distinct pointer nodes selected for dispatch in this frame.
    pub pointer_nodes_dispatched: u64,
    /// Spatial candidates checked for exact pointer hits.
    pub pointer_hit_test_candidates: u64,
    /// Nodes whose draw commands were recorded.
    pub paint_nodes_recorded: u64,
    /// Persistent scene branches updated, including affected ancestors.
    pub scene_nodes_updated: u64,
    /// Nodes examined while updating the semantic index.
    pub accessibility_nodes_updated: u64,
    /// Nodes with changed measurement or child-placement inputs.
    pub dirty_nodes_param: u64,
    /// Nodes whose direct layout children changed.
    pub dirty_nodes_structural: u64,
    /// Invalidated nodes including ancestors that depend on their sizes.
    pub dirty_nodes_with_ancestors: u64,
    /// Time spent identifying affected ancestors, in nanoseconds.
    pub dirty_expand_ns: u64,
    /// Measurement entrypoint calls, including cache lookups.
    pub measure_node_calls: u64,
    /// Cache hits that skipped an unchanged subtree.
    pub cache_hits_direct: u64,
    /// Cache hits after checking changed descendants or updating placements.
    pub cache_hits_boundary: u64,
    /// Cache misses without a reusable size snapshot.
    pub cache_miss_no_entry: u64,
    /// Cache misses caused by changed parent constraints.
    pub cache_miss_constraint: u64,
    /// Cache misses caused by the node's own layout inputs.
    pub cache_miss_dirty_self: u64,
    /// Cache misses caused by child size dependencies.
    pub cache_miss_child_size: u64,
    /// Measured or repositioned layouts saved for reuse.
    pub cache_store_count: u64,
    /// Layouts excluded from caching because child measurements were incomplete
    /// or inconsistent.
    pub cache_drop_non_cacheable_count: u64,
}

#[derive(Default)]
pub(crate) struct LayoutDiagnosticsCollector {
    measure_dirty_children_checked: u64,
    measure_node_calls: u64,
    cache_hits_direct: u64,
    cache_hits_boundary: u64,
    cache_miss_no_entry: u64,
    cache_miss_constraint: u64,
    cache_miss_dirty_self: u64,
    cache_miss_child_size: u64,
    cache_store_count: u64,
    cache_drop_non_cacheable_count: u64,
}

impl LayoutDiagnosticsCollector {
    fn snapshot(
        &self,
        dirty_nodes_param: u64,
        dirty_nodes_structural: u64,
        dirty_nodes_with_ancestors: u64,
        dirty_expand_ns: u64,
    ) -> LayoutFrameDiagnostics {
        let cache_hits_direct = self.cache_hits_direct;
        let cache_hits_boundary = self.cache_hits_boundary;
        LayoutFrameDiagnostics {
            geometry_nodes_updated: 0,
            geometry_nodes_visited: 0,
            measure_dirty_children_checked: self.measure_dirty_children_checked,
            pointer_nodes_updated: 0,
            pointer_nodes_dispatched: 0,
            pointer_hit_test_candidates: 0,
            paint_nodes_recorded: 0,
            scene_nodes_updated: 0,
            accessibility_nodes_updated: 0,
            dirty_nodes_param,
            dirty_nodes_structural,
            dirty_nodes_with_ancestors,
            dirty_expand_ns,
            measure_node_calls: self.measure_node_calls,
            cache_hits_direct,
            cache_hits_boundary,
            cache_miss_no_entry: self.cache_miss_no_entry,
            cache_miss_constraint: self.cache_miss_constraint,
            cache_miss_dirty_self: self.cache_miss_dirty_self,
            cache_miss_child_size: self.cache_miss_child_size,
            cache_store_count: self.cache_store_count,
            cache_drop_non_cacheable_count: self.cache_drop_non_cacheable_count,
        }
    }
}

impl LayoutContext<'_> {
    pub(crate) fn new<'a>(
        snapshots: &'a mut LayoutSnapshotMap,
        measure_self_nodes: &'a HashSet<u64>,
        placement_self_nodes: &'a HashSet<u64>,
        dirty_effective_nodes: &'a HashSet<u64>,
        dirty_children: &'a HashMap<u64, Vec<NodeId>>,
        diagnostics: &'a mut LayoutDiagnosticsCollector,
    ) -> LayoutContext<'a> {
        LayoutContext {
            state_reader_instance_key: None,
            snapshots,
            measure_self_nodes,
            placement_self_nodes,
            dirty_effective_nodes,
            dirty_children: Some(dirty_children),
            diagnostics,
            intrinsic: false,
        }
    }

    pub(crate) fn intrinsic_probe<'a>(
        snapshots: &'a mut LayoutSnapshotMap,
        empty: &'a HashSet<u64>,
        diagnostics: &'a mut LayoutDiagnosticsCollector,
        parent: Option<&'a LayoutContext<'_>>,
    ) -> LayoutContext<'a> {
        LayoutContext {
            state_reader_instance_key: crate::runtime::current_measured_node_instance_key(),
            snapshots,
            measure_self_nodes: empty,
            placement_self_nodes: empty,
            dirty_effective_nodes: empty,
            dirty_children: None,
            diagnostics: parent.map_or(diagnostics, |parent| parent.diagnostics),
            intrinsic: true,
        }
    }

    #[inline]
    pub(crate) fn snapshot(&self, instance_key: u64) -> Option<Arc<LayoutSnapshotEntry>> {
        // SAFETY: Layout snapshots are owned by the single-threaded compute
        // pass. The pointer is created from a unique mutable borrow for the
        // duration of the pass and only accessed on the same thread.
        unsafe { (*self.snapshots).get_cloned(&instance_key) }
    }

    #[inline]
    pub(crate) fn insert_snapshot(&self, instance_key: u64, entry: LayoutSnapshotEntry) {
        // SAFETY: See `snapshot`.
        unsafe {
            (*self.snapshots).insert(instance_key, entry);
        }
    }

    #[inline]
    pub(crate) fn remove_snapshot(&self, instance_key: u64) {
        // SAFETY: See `snapshot`.
        unsafe {
            (*self.snapshots).remove(&instance_key);
        }
    }

    #[inline]
    pub(crate) fn inc_measure_node_calls(&self) {
        // SAFETY: Layout diagnostics are owned by the single-threaded compute
        // pass. The pointer is created from a unique mutable borrow for
        // the duration of that pass and is only accessed on the same
        // thread during recursive layout measurement.
        unsafe {
            (*self.diagnostics).measure_node_calls += 1;
        }
    }

    pub(crate) fn inc_dirty_child_checked(&self) {
        // SAFETY: See `inc_measure_node_calls`.
        unsafe {
            (*self.diagnostics).measure_dirty_children_checked += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_hit_direct(&self) {
        unsafe {
            (*self.diagnostics).cache_hits_direct += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_hit_boundary(&self) {
        unsafe {
            (*self.diagnostics).cache_hits_boundary += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_miss_no_entry(&self) {
        unsafe {
            (*self.diagnostics).cache_miss_no_entry += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_miss_constraint(&self) {
        unsafe {
            (*self.diagnostics).cache_miss_constraint += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_miss_dirty_self(&self) {
        unsafe {
            (*self.diagnostics).cache_miss_dirty_self += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_miss_child_size(&self) {
        unsafe {
            (*self.diagnostics).cache_miss_child_size += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_store_count(&self) {
        unsafe {
            (*self.diagnostics).cache_store_count += 1;
        }
    }

    #[inline]
    pub(crate) fn inc_cache_drop_non_cacheable_count(&self) {
        unsafe {
            (*self.diagnostics).cache_drop_non_cacheable_count += 1;
        }
    }
}

/// Parameters for the compute function
pub(crate) struct ComputeParams<'a> {
    pub screen_size: PxSize,
    pub cursor_position: Option<PxPosition>,
    pub pointer_changes: Vec<PointerChange>,
    pub keyboard_events: Vec<QueuedKeyEvent>,
    pub ime_events: Vec<winit::event::Ime>,
    pub retry_focus_move: Option<FocusDirection>,
    pub retry_focus_reveal: bool,
    pub modifiers: winit::keyboard::ModifiersState,
    pub layout_dirty_nodes: &'a LayoutDirtyNodes,
}

#[derive(Debug)]
pub(crate) enum ComputeMode<'a> {
    Full {
        compute_resource_manager: &'a mut ComputeResourceManager,
        gpu: &'a wgpu::Device,
    },
    #[cfg(feature = "testing")]
    LayoutOnly,
}

struct GlobalPositionSnapshot {
    rect: PxRect,
    handlers: SmallVec<[GloballyPositionedHandler; 2]>,
}

enum QueuedFocusedInput {
    Keyboard(QueuedKeyEvent),
    Clipboard(QueuedClipboardAction),
}

impl QueuedFocusedInput {
    fn sequence(&self) -> u64 {
        match self {
            Self::Keyboard(event) => event.sequence,
            Self::Clipboard(event) => event.sequence,
        }
    }
}

/// Respents a component tree
pub struct ComponentTree {
    /// We use indextree as the tree structure
    tree: indextree::Arena<ComponentNode>,
    /// Components' metadatas
    metadatas: ComponentNodeMetaDatas,
    layout_snapshots: LayoutSnapshotMap,
    frame_scene: scene::FrameScene,
    pointer_index: pointer::PointerIndex,
    accessibility_index: crate::accessibility::AccessibilityIndex,
    #[cfg(feature = "profiling")]
    profiler_nodes_cache: HashMap<NodeId, NodeMeta>,
    #[cfg(feature = "profiling")]
    profiler_dirty_nodes: HashSet<NodeId>,
    scale_factor_bits: u64,
    /// Used to remember the current node
    node_queue: Vec<indextree::NodeId>,
    /// Detached old-subtree nodes keyed by instance key during replay replace.
    replay_reuse_candidates: HashMap<u64, indextree::NodeId>,
    node_ids_by_instance_key: HashMap<u64, indextree::NodeId>,
    composition_order: HashMap<NodeId, usize>,
    changed_frame_nodes: HashSet<NodeId>,
    removed_frame_nodes: HashSet<NodeId>,
    removed_focus_owners: HashSet<u64>,
    /// Active pointer hit paths keyed by pointer id.
    /// Each path stores node instance keys from root to leaf.
    active_pointer_paths: HashMap<u64, Vec<u64>>,
    active_pointer_positions: HashMap<u64, Option<PxPosition>>,
    pointer_work: (u64, u64),
    /// Per-tree focus owner used for keyboard and IME routing.
    focus_owner: FocusOwner,
    pending_keyboard_events: VecDeque<QueuedFocusedInput>,
    pending_ime_events: Vec<winit::event::Ime>,
    /// Last reported window-space bounds for nodes with globally positioned
    /// callbacks, keyed by instance key.
    last_global_rects: HashMap<u64, GlobalPositionSnapshot>,
    pending_position_observers: HashSet<NodeId>,
    /// Globally positioned callbacks collected during compute and invoked
    /// after the runtime borrow is released.
    pending_globally_positioned_invocations: Vec<(GloballyPositionedHandler, PxRect)>,
}

#[derive(Clone, PartialEq)]
pub(crate) enum ReplayReplaceError {
    NodeNotFound,
    RootNodeNotReplaceable,
    ReplayDidNotCreateRoot,
}

#[derive(Default)]
pub(crate) struct ReplayReplaceResult {
    pub removed_instance_keys: HashSet<u64>,
    pub removed_instance_logic_ids: HashSet<u64>,
    pub inserted_instance_keys: HashSet<u64>,
    pub inserted_instance_logic_ids: HashSet<u64>,
    pub reused_instance_logic_ids: HashSet<u64>,
}

pub(crate) struct ReplayReplaceContext {
    parent_id: indextree::NodeId,
    next_sibling: Option<indextree::NodeId>,
    existing_children: HashSet<indextree::NodeId>,
    previous_queue: Vec<indextree::NodeId>,
    detached_root_id: indextree::NodeId,
    detached_node_ids: HashSet<indextree::NodeId>,
    removed_instance_keys: HashSet<u64>,
    removed_instance_logic_ids: HashSet<u64>,
}

impl Default for ComponentTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTree {
    /// Create a new ComponentTree
    pub fn new() -> Self {
        let tree = indextree::Arena::new();
        let node_queue = Vec::new();
        let metadatas = ComponentNodeMetaDatas::new();
        Self {
            tree,
            node_queue,
            metadatas,
            layout_snapshots: LayoutSnapshotMap::default(),
            frame_scene: scene::FrameScene::default(),
            pointer_index: pointer::PointerIndex::default(),
            accessibility_index: crate::accessibility::AccessibilityIndex::default(),
            #[cfg(feature = "profiling")]
            profiler_nodes_cache: HashMap::default(),
            #[cfg(feature = "profiling")]
            profiler_dirty_nodes: HashSet::default(),
            scale_factor_bits: crate::dp::scale_factor().to_bits(),
            replay_reuse_candidates: HashMap::default(),
            node_ids_by_instance_key: HashMap::default(),
            composition_order: HashMap::default(),
            changed_frame_nodes: HashSet::default(),
            removed_frame_nodes: HashSet::default(),
            removed_focus_owners: HashSet::default(),
            active_pointer_paths: HashMap::default(),
            active_pointer_positions: HashMap::default(),
            pointer_work: (0, 0),
            focus_owner: FocusOwner::new(),
            pending_keyboard_events: VecDeque::new(),
            pending_ime_events: Vec::new(),
            last_global_rects: HashMap::default(),
            pending_position_observers: HashSet::default(),
            pending_globally_positioned_invocations: Vec::new(),
        }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        crate::runtime::remove_layout_structure(
            &self.node_ids_by_instance_key.keys().copied().collect(),
        );
        self.removed_focus_owners
            .extend(self.node_ids_by_instance_key.keys().copied());
        self.removed_frame_nodes
            .extend(self.node_ids_by_instance_key.values().copied());
        self.changed_frame_nodes.clear();
        self.composition_order.clear();
        #[cfg(feature = "profiling")]
        {
            self.profiler_nodes_cache.clear();
            self.profiler_dirty_nodes.clear();
        }
        self.tree.clear();
        self.metadatas.clear();
        self.layout_snapshots.clear();
        self.frame_scene = scene::FrameScene::default();
        self.pointer_index = pointer::PointerIndex::default();
        self.accessibility_index = crate::accessibility::AccessibilityIndex::default();
        self.node_queue.clear();
        self.replay_reuse_candidates.clear();
        self.node_ids_by_instance_key.clear();
        self.last_global_rects.clear();
        self.pending_position_observers.clear();
        self.pending_globally_positioned_invocations.clear();
    }

    /// Reset the entire component tree, including focus ownership state.
    pub fn reset(&mut self) {
        self.clear();
        self.active_pointer_paths.clear();
        self.active_pointer_positions.clear();
        self.focus_owner.reset();
        self.pending_keyboard_events.clear();
        clipboard::clear_requests();
        self.pending_ime_events.clear();
    }

    pub(crate) fn refresh_scale_factor(&mut self) -> bool {
        let current = crate::dp::scale_factor().to_bits();
        let changed = self.scale_factor_bits != current;
        self.scale_factor_bits = current;
        changed
    }

    /// Get node by NodeId
    pub(crate) fn get(&self, node_id: indextree::NodeId) -> Option<&ComponentNode> {
        self.tree
            .get(node_id)
            .filter(|node| !node.is_removed())
            .map(|node| node.get())
    }

    /// Get mutable node by NodeId
    pub(crate) fn get_mut(&mut self, node_id: indextree::NodeId) -> Option<&mut ComponentNode> {
        self.tree
            .get_mut(node_id)
            .filter(|node| !node.is_removed())
            .map(|node| node.get_mut())
    }

    /// Find a node id by stable instance key.
    pub(crate) fn find_node_id_by_instance_key(
        &self,
        instance_key: u64,
    ) -> Option<indextree::NodeId> {
        self.node_ids_by_instance_key.get(&instance_key).copied()
    }

    fn sync_accessibility(&mut self, root: NodeId) -> usize {
        let mut changed = self.metadatas.geometry_changed.clone();
        changed.extend(self.metadatas.paint_changed.iter().copied());
        self.accessibility_index
            .invalidate(&changed, &HashSet::default());
        self.accessibility_index
            .update(&self.tree, &self.metadatas, root, &self.composition_order)
    }

    pub(crate) fn accessibility_update(
        &mut self,
        label: Option<&str>,
        full: bool,
    ) -> Option<accesskit::TreeUpdate> {
        let root = self.tree.get_node_id_at(NonZero::new(1)?)?;
        self.accessibility_index.set_label(label, root);
        self.accessibility_index
            .update(&self.tree, &self.metadatas, root, &self.composition_order);
        self.accessibility_index.tree_update(
            &self.tree,
            self.focus_owner.active_component_node_id(),
            full,
        )
    }

    pub(crate) fn take_removed_focus_owners(&mut self) -> HashSet<u64> {
        std::mem::take(&mut self.removed_focus_owners)
    }

    pub(crate) fn begin_replace_subtree_by_instance_key(
        &mut self,
        instance_key: u64,
    ) -> Result<ReplayReplaceContext, ReplayReplaceError> {
        self.replay_reuse_candidates.clear();
        let Some(target_node_id) = self.find_node_id_by_instance_key(instance_key) else {
            return Err(ReplayReplaceError::NodeNotFound);
        };

        let Some(parent_id) = self.tree.get(target_node_id).and_then(|n| n.parent()) else {
            return Err(ReplayReplaceError::RootNodeNotReplaceable);
        };

        let mut next_sibling = None;
        let mut seen_target = false;
        for child in parent_id.children(&self.tree) {
            if seen_target {
                next_sibling = Some(child);
                break;
            }
            if child == target_node_id {
                seen_target = true;
            }
        }

        let removed_node_ids: Vec<_> = target_node_id
            .traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
            .collect();
        let detached_node_ids = removed_node_ids.iter().copied().collect::<HashSet<_>>();
        let mut removed_instance_keys = HashSet::default();
        let mut removed_instance_logic_ids = HashSet::default();
        for id in &removed_node_ids {
            if let Some(node) = self.tree.get(*id) {
                removed_instance_keys.insert(node.get().instance_key);
                removed_instance_logic_ids.insert(node.get().instance_logic_id);
                self.replay_reuse_candidates
                    .insert(node.get().instance_key, *id);
            }
        }
        target_node_id.detach(&mut self.tree);

        let existing_children = parent_id.children(&self.tree).collect();

        let previous_queue = std::mem::replace(&mut self.node_queue, vec![parent_id]);
        Ok(ReplayReplaceContext {
            parent_id,
            next_sibling,
            existing_children,
            previous_queue,
            detached_root_id: target_node_id,
            detached_node_ids,
            removed_instance_keys,
            removed_instance_logic_ids,
        })
    }

    pub(crate) fn finish_replace_subtree(
        &mut self,
        context: ReplayReplaceContext,
    ) -> Result<ReplayReplaceResult, ReplayReplaceError> {
        let ReplayReplaceContext {
            parent_id,
            next_sibling,
            existing_children,
            previous_queue,
            detached_root_id,
            detached_node_ids,
            removed_instance_keys,
            removed_instance_logic_ids,
        } = context;

        let mut inserted_root_ids = parent_id
            .children(&self.tree)
            .filter(|child_id| !existing_children.contains(child_id))
            .collect::<Vec<_>>();
        if inserted_root_ids.is_empty() {
            self.node_queue = previous_queue;
            return Err(ReplayReplaceError::ReplayDidNotCreateRoot);
        }

        if let Some(next_sibling) = next_sibling {
            for inserted_root_id in &inserted_root_ids {
                if *inserted_root_id != next_sibling {
                    next_sibling.insert_before(*inserted_root_id, &mut self.tree);
                }
            }
            inserted_root_ids = parent_id
                .children(&self.tree)
                .filter(|child_id| !existing_children.contains(child_id))
                .collect::<Vec<_>>();
        }

        let detached_root_reused = inserted_root_ids.contains(&detached_root_id);

        let mut inserted_instance_keys = HashSet::default();
        let mut inserted_instance_logic_ids = HashSet::default();
        let mut reused_instance_logic_ids = HashSet::default();
        let mut replaced_nodes = Vec::new();
        for inserted_root_id in &inserted_root_ids {
            for edge in inserted_root_id.traverse(&self.tree) {
                let indextree::NodeEdge::Start(id) = edge else {
                    continue;
                };
                if let Some(node) = self.tree.get(id) {
                    self.changed_frame_nodes.insert(id);
                    inserted_instance_keys.insert(node.get().instance_key);
                    inserted_instance_logic_ids.insert(node.get().instance_logic_id);
                    if detached_node_ids.contains(&id) {
                        reused_instance_logic_ids.insert(node.get().instance_logic_id);
                    } else if let Some(old_id) =
                        self.replay_reuse_candidates.get(&node.get().instance_key)
                    {
                        replaced_nodes.push((*old_id, id));
                    }
                }
            }
        }

        let removed_instance_keys = removed_instance_keys
            .difference(&inserted_instance_keys)
            .copied()
            .collect::<HashSet<_>>();
        let removed_instance_logic_ids = removed_instance_logic_ids
            .difference(&inserted_instance_logic_ids)
            .copied()
            .collect::<HashSet<_>>();
        for (old_id, new_id) in replaced_nodes {
            let old_parent = if old_id == detached_root_id {
                Some(parent_id)
            } else {
                self.tree.get(old_id).and_then(|node| node.parent())
            };
            let new_parent = self.tree.get(new_id).and_then(|node| node.parent());
            let same_parent = old_parent
                .and_then(|id| self.tree.get(id))
                .map(|node| node.get().instance_key)
                == new_parent
                    .and_then(|id| self.tree.get(id))
                    .map(|node| node.get().instance_key);
            if let Some(previous) = self.metadatas.remove(&old_id) {
                let metadata = self.metadatas.entry_or_default(new_id);
                metadata.computed_data = previous.computed_data;
                metadata.draw_layout = previous.draw_layout;
                metadata.rel_position = previous.rel_position;
                metadata.placement_order = previous.placement_order;
                metadata.base_abs_position = previous.base_abs_position;
                metadata.abs_position = previous.abs_position;
                metadata.event_clip_rect = previous.event_clip_rect;
                metadata.prepared_layout = previous.prepared_layout;
                metadata.geometry_initialized = previous.geometry_initialized;
                metadata.clips_children = previous.clips_children;
                metadata.clip_region = previous.clip_region;
                metadata.opacity = previous.opacity;
                metadata.has_shape_clipping = previous.has_shape_clipping;
                if same_parent && metadata.geometry_initialized {
                    self.metadatas.geometry_dirty.remove(&new_id);
                }
            }
        }
        for key in &removed_instance_keys {
            self.last_global_rects.remove(key);
            self.node_ids_by_instance_key.remove(key);
        }
        self.removed_focus_owners
            .extend(removed_instance_keys.iter().copied());
        crate::runtime::remove_layout_structure(&removed_instance_keys);
        if !detached_root_reused {
            let detached_node_ids = detached_root_id
                .traverse(&self.tree)
                .filter_map(|edge| match edge {
                    indextree::NodeEdge::Start(id) => Some(id),
                    indextree::NodeEdge::End(_) => None,
                })
                .collect::<Vec<_>>();
            for node_id in &detached_node_ids {
                self.removed_frame_nodes.insert(*node_id);
                self.changed_frame_nodes.remove(node_id);
                self.metadatas.remove(node_id);
                self.composition_order.remove(node_id);
            }
            detached_root_id.remove_subtree(&mut self.tree);
        }
        self.replay_reuse_candidates.clear();

        let mut ancestor = Some(parent_id);
        while let Some(node_id) = ancestor {
            self.refresh_layout_children(node_id);
            ancestor = self.tree.get(node_id).and_then(|node| node.parent());
        }

        self.node_queue = previous_queue;
        Ok(ReplayReplaceResult {
            removed_instance_keys,
            removed_instance_logic_ids,
            inserted_instance_keys,
            inserted_instance_logic_ids,
            reused_instance_logic_ids,
        })
    }

    /// Get current node
    pub(crate) fn current_node(&self) -> Option<&ComponentNode> {
        self.node_queue
            .last()
            .and_then(|node_id| self.get(*node_id))
    }

    /// Get mutable current node
    pub(crate) fn current_node_mut(&mut self) -> Option<&mut ComponentNode> {
        let node_id = self.node_queue.last()?;
        self.get_mut(*node_id)
    }

    pub(crate) fn set_current_node_identity(&mut self, instance_key: u64, instance_logic_id: u64) {
        if let Some(&node_id) = self.node_queue.last() {
            let node = self.tree[node_id].get_mut();
            if self.node_ids_by_instance_key.get(&node.instance_key) == Some(&node_id) {
                self.node_ids_by_instance_key.remove(&node.instance_key);
            }
            node.instance_key = instance_key;
            node.instance_logic_id = instance_logic_id;
            self.node_ids_by_instance_key.insert(instance_key, node_id);
        } else {
            debug_assert!(
                false,
                "set_current_node_identity requires a component build"
            );
        }
    }

    fn subtree_has_pending_rebuild(&self, root: NodeId) -> bool {
        crate::runtime::pending_build_instance_keys()
            .into_iter()
            .any(|key| {
                self.replay_reuse_candidates
                    .get(&key)
                    .or_else(|| self.node_ids_by_instance_key.get(&key))
                    .is_some_and(|id| id.ancestors(&self.tree).any(|id| id == root))
            })
    }

    pub(crate) fn try_reuse_current_subtree(
        &mut self,
        instance_key: u64,
        instance_logic_id: u64,
    ) -> bool {
        let Some(&candidate_node_id) = self.replay_reuse_candidates.get(&instance_key) else {
            return false;
        };
        let Some(candidate_instance_logic_id) = self
            .tree
            .get(candidate_node_id)
            .map(|node| node.get().instance_logic_id)
        else {
            self.replay_reuse_candidates.remove(&instance_key);
            return false;
        };
        if candidate_instance_logic_id != instance_logic_id {
            return false;
        }
        // Reusing the detached subtree would skip every descendant, including
        // replay boundaries that were invalidated this frame (for example by a
        // render slot whose closure was rebuilt with new props). Bail out and
        // let the caller rebuild the subtree so those nodes observe the update.
        if self.subtree_has_pending_rebuild(candidate_node_id) {
            return false;
        }

        let Some(current_node_id) = self.node_queue.last().copied() else {
            return false;
        };
        if current_node_id == candidate_node_id {
            self.replay_reuse_candidates.remove(&instance_key);
            return true;
        }

        candidate_node_id.detach(&mut self.tree);
        current_node_id.insert_before(candidate_node_id, &mut self.tree);
        self.metadatas.remove(&current_node_id);
        self.composition_order.remove(&current_node_id);
        self.changed_frame_nodes.remove(&current_node_id);
        self.removed_frame_nodes.insert(current_node_id);
        self.changed_frame_nodes.insert(candidate_node_id);
        current_node_id.remove_subtree(&mut self.tree);
        if let Some(current_node) = self.node_queue.last_mut() {
            *current_node = candidate_node_id;
        }
        self.node_ids_by_instance_key
            .insert(instance_key, candidate_node_id);
        self.replay_reuse_candidates.remove(&instance_key);
        true
    }

    /// Add a new node to the tree
    /// Nodes now store their intrinsic constraints in their metadata.
    /// The `node_component` itself primarily holds the layout policy, render
    /// policy, and handlers.
    pub(crate) fn add_node(&mut self, node_component: ComponentNode) -> indextree::NodeId {
        let instance_key = node_component.instance_key;
        let new_node_id = self.tree.new_node(node_component);
        self.changed_frame_nodes.insert(new_node_id);
        if instance_key != 0 || self.node_queue.is_empty() {
            self.node_ids_by_instance_key
                .insert(instance_key, new_node_id);
        }
        if let Some(current_node_id) = self.node_queue.last_mut() {
            current_node_id.append(new_node_id, &mut self.tree);
        }
        let metadata = ComponentNodeMetaData::none();
        self.metadatas.insert(new_node_id, metadata);
        self.node_queue.push(new_node_id);
        new_node_id
    }

    /// Pop the last node from the queue
    pub(crate) fn pop_node(&mut self) {
        if let Some(node_id) = self.node_queue.pop() {
            self.refresh_layout_children(node_id);
            if !self.tree[node_id]
                .get()
                .globally_positioned_handlers
                .is_empty()
            {
                self.pending_position_observers.insert(node_id);
            }
        }
    }

    fn refresh_layout_children(&mut self, node_id: indextree::NodeId) {
        let mut children = Vec::new();
        for (order, child_id) in node_id.children(&self.tree).enumerate() {
            if self
                .composition_order
                .insert(child_id, order)
                .is_some_and(|previous| previous != order)
            {
                self.metadatas.placement_order_dirty.insert(child_id);
                self.metadatas.mark_geometry_dirty(child_id);
            }
            let child = self.tree[child_id].get();
            if child.role == NodeRole::Layout {
                children.push(child_id);
            } else {
                children.extend(direct_layout_children(child_id, &self.tree).iter().copied());
            }
        }
        let keys = children
            .iter()
            .map(|id| self.tree[*id].get().instance_key)
            .collect();
        let node = self.tree[node_id].get_mut();
        if node.layout_children.as_deref() != Some(children.as_slice()) {
            node.layout_children = Some(children.into());
        }
        crate::runtime::record_layout_structure(node.instance_key, keys);
    }

    /// Get a reference to the underlying tree structure.
    ///
    /// This is used for accessibility tree building and other introspection
    /// needs.
    pub(crate) fn tree(&self) -> &indextree::Arena<ComponentNode> {
        &self.tree
    }

    /// Get a reference to the node metadatas.
    ///
    /// This is used for accessibility tree building and other introspection
    /// needs.
    #[cfg(any(
        feature = "testing",
        feature = "headless",
        feature = "debug-dirty-overlay"
    ))]
    pub(crate) fn metadatas(&self) -> &ComponentNodeMetaDatas {
        &self.metadatas
    }

    pub(crate) fn metadatas_mut(&mut self) -> &mut ComponentNodeMetaDatas {
        &mut self.metadatas
    }

    pub(crate) fn clear_layout_snapshots(&mut self) {
        self.layout_snapshots.clear();
    }

    fn remove_layout_snapshots(&mut self, keys: &HashSet<u64>) {
        if keys.is_empty() {
            return;
        }
        for key in keys {
            self.layout_snapshots.remove(key);
        }
    }

    pub(crate) fn focus_owner(&self) -> &FocusOwner {
        &self.focus_owner
    }

    pub(crate) fn focus_owner_mut(&mut self) -> &mut FocusOwner {
        &mut self.focus_owner
    }

    pub(crate) fn accessibility_dispatch_context(
        &mut self,
    ) -> (&ComponentNodeTree, &ComponentNodeMetaDatas, &mut FocusOwner) {
        (&self.tree, &self.metadatas, &mut self.focus_owner)
    }

    fn collect_globally_positioned_invocations(&mut self, _root_node: indextree::NodeId) {
        let mut candidates = std::mem::take(&mut self.pending_position_observers);
        candidates.extend(self.metadatas.geometry_changed.iter().copied());
        let mut invocations = Vec::new();
        for node_id in candidates {
            let Some(node_ref) = self.tree.get(node_id) else {
                continue;
            };
            let node = node_ref.get();
            if node.globally_positioned_handlers.is_empty() {
                continue;
            }
            let Some(metadata) = self.metadatas.get(&node_id) else {
                continue;
            };
            let Some(position) = metadata.abs_position else {
                self.last_global_rects.remove(&node.instance_key);
                continue;
            };
            let size = metadata.computed_data.unwrap_or(ComputedData::ZERO);
            let rect = PxRect {
                x: position.x_px(),
                y: position.y_px(),
                width: size.width,
                height: size.height,
            };
            let previous = self.last_global_rects.get(&node.instance_key);
            for handler in &node.globally_positioned_handlers {
                if previous.is_none_or(|previous| {
                    previous.rect != rect || !previous.handlers.contains(handler)
                }) {
                    invocations.push((*handler, rect));
                }
            }
            self.last_global_rects.insert(
                node.instance_key,
                GlobalPositionSnapshot {
                    rect,
                    handlers: node.globally_positioned_handlers.iter().copied().collect(),
                },
            );
        }
        self.pending_globally_positioned_invocations
            .extend(invocations);
    }

    pub(crate) fn take_pending_globally_positioned_invocations(
        &mut self,
    ) -> Vec<(GloballyPositionedHandler, PxRect)> {
        std::mem::take(&mut self.pending_globally_positioned_invocations)
    }

    pub(crate) fn take_pending_focus_callback_invocations(
        &mut self,
    ) -> Vec<PendingFocusCallbackInvocation> {
        let notifications = self.focus_owner.take_pending_notifications();
        let mut invocations = Vec::new();

        for notification in notifications {
            let Some(node_id) = self
                .focus_owner
                .component_node_id_of(notification.handle_id)
            else {
                continue;
            };
            let Some(node_ref) = self.tree.get(node_id) else {
                continue;
            };
            let node = node_ref.get();

            if notification.changed
                && let Some(handler) = &node.focus_changed_handler
            {
                invocations.push(PendingFocusCallbackInvocation::new(
                    *handler,
                    notification.state,
                    notification.direction,
                ));
            }
            if let Some(handler) = &node.focus_event_handler {
                invocations.push(PendingFocusCallbackInvocation::new(
                    *handler,
                    notification.state,
                    notification.direction,
                ));
            }
        }

        invocations
    }

    /// Collect per-node metadata for profiling output.
    #[cfg(feature = "profiling")]
    pub fn profiler_nodes(&mut self) -> Vec<NodeMeta> {
        self.profiler_dirty_nodes
            .extend(std::mem::take(&mut self.metadatas.profiling_dirty));
        self.profiler_dirty_nodes
            .extend(self.metadatas.geometry_changed.iter().copied());
        for node_id in std::mem::take(&mut self.profiler_dirty_nodes) {
            let Some(node) = self.tree.get(node_id).filter(|node| !node.is_removed()) else {
                self.profiler_nodes_cache.remove(&node_id);
                continue;
            };
            let metadata = self.metadatas.get(&node_id);
            self.profiler_nodes_cache.insert(
                node_id,
                NodeMeta {
                    node_id: node_id.to_string(),
                    parent: node.parent().map(|id| id.to_string()),
                    fn_name: Some(node.get().fn_name.clone()),
                    abs_pos: metadata
                        .and_then(|metadata| metadata.abs_position)
                        .map(|position| (position.x_px().0, position.y_px().0)),
                    size: metadata
                        .and_then(|metadata| metadata.computed_data)
                        .map(|size| (size.width.0, size.height.0)),
                    layout_cache_hit: metadata.map(|metadata| metadata.layout_cache_hit),
                },
            );
        }
        self.profiler_nodes_cache.values().cloned().collect()
    }

    /// Dispatches pointer input across the tree and returns the window requests
    /// raised by the handlers (currently the hover cursor icon).
    fn dispatch_pointer_input(
        &mut self,
        root_node: indextree::NodeId,
        pointer_changes: &mut [PointerChange],
        cursor_position: &mut Option<PxPosition>,
        modifiers: winit::keyboard::ModifiersState,
    ) -> WindowRequests {
        let mut window_requests = WindowRequests::default();
        let mut changed = self.metadatas.geometry_changed.clone();
        changed.extend(self.changed_frame_nodes.iter().copied());
        self.pointer_work.0 = self.pointer_index.update(
            &self.tree,
            &self.metadatas,
            &self.composition_order,
            &changed,
            &self.removed_frame_nodes,
        ) as u64;
        self.pointer_index.begin_dispatch();
        self.active_pointer_paths.retain(|_, path| {
            path.last().is_some_and(|key| {
                self.node_ids_by_instance_key.get(key).is_some_and(|id| {
                    self.metadatas
                        .get(id)
                        .is_some_and(|metadata| metadata.abs_position.is_some())
                })
            })
        });
        self.active_pointer_positions
            .retain(|pointer, _| self.active_pointer_paths.contains_key(pointer));
        let dirty = crate::runtime::take_pointer_dirty_nodes();
        let mut selected: HashSet<_> = dirty
            .into_iter()
            .filter_map(|key| self.node_ids_by_instance_key.get(&key).copied())
            .collect();
        for path in self.active_pointer_paths.values() {
            selected.extend(
                path.iter()
                    .filter_map(|key| self.node_ids_by_instance_key.get(key).copied()),
            );
        }
        for change in pointer_changes.iter() {
            selected.extend(self.pointer_index.sample_targets(
                &self.tree,
                &self.metadatas,
                change.resolved_position(*cursor_position),
            ));
            selected.extend(self.pointer_index.hit_path(
                &self.tree,
                &self.metadatas,
                change.resolved_position(*cursor_position),
            ));
        }
        let final_position = *cursor_position;
        let path = self
            .pointer_index
            .hit_path(&self.tree, &self.metadatas, final_position);
        window_requests.cursor_icon = path
            .into_iter()
            .rev()
            .find_map(|id| {
                let node = self.tree.get(id)?.get();
                let metadata = self.metadatas.get(&id)?;
                resolve_node_hover_cursor_icon(
                    node,
                    metadata.base_abs_position?,
                    metadata.computed_data?,
                    final_position?,
                )
            })
            .unwrap_or_default();
        let plan = self.pointer_index.plan(
            &self.tree,
            &self.metadatas,
            final_position,
            selected.into_iter(),
        );
        self.pointer_work.1 = plan.nodes.len() as u64;
        for node in plan.nodes.values() {
            crate::runtime::clear_phase_state_reads(
                node.context.instance_key,
                crate::runtime::ReadPhase::Pointer,
            );
        }
        let mut last_position = None;
        for change in pointer_changes {
            let mut position = change.resolved_position(final_position);
            last_position = Some(position);
            self.dispatch_pointer_sample(
                root_node,
                std::slice::from_mut(change),
                &mut position,
                modifiers,
                &plan,
                &mut window_requests,
            );
        }
        if last_position != Some(final_position) {
            self.dispatch_pointer_sample(
                root_node,
                &mut [],
                cursor_position,
                modifiers,
                &plan,
                &mut window_requests,
            );
        }
        window_requests
    }

    fn dispatch_pointer_sample(
        &mut self,
        _root_node: indextree::NodeId,
        pointer_changes: &mut [PointerChange],
        cursor_position: &mut Option<PxPosition>,
        modifiers: winit::keyboard::ModifiersState,
        plan: &PointerDispatchPlan,
        window_requests: &mut WindowRequests,
    ) {
        let node_ids_preorder = &plan.preorder;
        let node_ids_postorder = &plan.postorder;
        let pointer_change_paths = build_pointer_change_paths(
            &self.pointer_index,
            &self.tree,
            &self.metadatas,
            pointer_changes,
            *cursor_position,
            &mut self.active_pointer_paths,
        );
        for change in pointer_changes.iter() {
            if self.active_pointer_paths.contains_key(&change.pointer_id) {
                self.active_pointer_positions.insert(
                    change.pointer_id,
                    change.resolved_position(*cursor_position),
                );
            } else {
                self.active_pointer_positions.remove(&change.pointer_id);
            }
        }

        for node_id in node_ids_preorder.iter().copied() {
            let Some(node) = self.tree.get(node_id).map(|n| n.get()) else {
                continue;
            };
            let mut dispatch_ctx = PointerInputDispatchContext {
                tree: &self.tree,
                metadatas: &self.metadatas,
                cursor_position: &mut *cursor_position,
                pointer_changes: &mut *pointer_changes,
                pointer_change_paths: &pointer_change_paths,
                nodes: &plan.nodes,
                active_pointer_paths: &self.active_pointer_paths,
                active_pointer_positions: &self.active_pointer_positions,
                modifiers,
                window_requests: &mut *window_requests,
                focus_owner: &mut self.focus_owner,
            };
            dispatch_pointer_modifiers_for_node_pass(
                &mut dispatch_ctx,
                node_id,
                PointerEventPass::Initial,
            );
            let handlers = &node.pointer_preview_handlers;
            for handler in handlers {
                let mut dispatch_ctx = PointerInputDispatchContext {
                    tree: &self.tree,
                    metadatas: &self.metadatas,
                    cursor_position: &mut *cursor_position,
                    pointer_changes: &mut *pointer_changes,
                    pointer_change_paths: &pointer_change_paths,
                    nodes: &plan.nodes,
                    active_pointer_paths: &self.active_pointer_paths,
                    active_pointer_positions: &self.active_pointer_positions,
                    modifiers,
                    window_requests: &mut *window_requests,
                    focus_owner: &mut self.focus_owner,
                };
                run_pointer_handler_for_node(
                    &mut dispatch_ctx,
                    node_id,
                    PointerEventPass::Initial,
                    handler.as_ref(),
                );
            }
        }

        for node_id in node_ids_postorder.iter().copied() {
            let Some(node) = self.tree.get(node_id).map(|n| n.get()) else {
                continue;
            };
            let mut dispatch_ctx = PointerInputDispatchContext {
                tree: &self.tree,
                metadatas: &self.metadatas,
                cursor_position: &mut *cursor_position,
                pointer_changes: &mut *pointer_changes,
                pointer_change_paths: &pointer_change_paths,
                nodes: &plan.nodes,
                active_pointer_paths: &self.active_pointer_paths,
                active_pointer_positions: &self.active_pointer_positions,
                modifiers,
                window_requests: &mut *window_requests,
                focus_owner: &mut self.focus_owner,
            };
            dispatch_pointer_modifiers_for_node_pass(
                &mut dispatch_ctx,
                node_id,
                PointerEventPass::Main,
            );
            let handlers = &node.pointer_handlers;
            for handler in handlers {
                let mut dispatch_ctx = PointerInputDispatchContext {
                    tree: &self.tree,
                    metadatas: &self.metadatas,
                    cursor_position: &mut *cursor_position,
                    pointer_changes: &mut *pointer_changes,
                    pointer_change_paths: &pointer_change_paths,
                    nodes: &plan.nodes,
                    active_pointer_paths: &self.active_pointer_paths,
                    active_pointer_positions: &self.active_pointer_positions,
                    modifiers,
                    window_requests: &mut *window_requests,
                    focus_owner: &mut self.focus_owner,
                };
                run_pointer_handler_for_node(
                    &mut dispatch_ctx,
                    node_id,
                    PointerEventPass::Main,
                    handler.as_ref(),
                );
            }
        }

        for node_id in node_ids_preorder.iter().copied() {
            let Some(node) = self.tree.get(node_id).map(|n| n.get()) else {
                continue;
            };
            let mut dispatch_ctx = PointerInputDispatchContext {
                tree: &self.tree,
                metadatas: &self.metadatas,
                cursor_position: &mut *cursor_position,
                pointer_changes: &mut *pointer_changes,
                pointer_change_paths: &pointer_change_paths,
                nodes: &plan.nodes,
                active_pointer_paths: &self.active_pointer_paths,
                active_pointer_positions: &self.active_pointer_positions,
                modifiers,
                window_requests: &mut *window_requests,
                focus_owner: &mut self.focus_owner,
            };
            dispatch_pointer_modifiers_for_node_pass(
                &mut dispatch_ctx,
                node_id,
                PointerEventPass::Final,
            );
            let handlers = &node.pointer_final_handlers;
            for handler in handlers {
                let mut dispatch_ctx = PointerInputDispatchContext {
                    tree: &self.tree,
                    metadatas: &self.metadatas,
                    cursor_position: &mut *cursor_position,
                    pointer_changes: &mut *pointer_changes,
                    pointer_change_paths: &pointer_change_paths,
                    nodes: &plan.nodes,
                    active_pointer_paths: &self.active_pointer_paths,
                    active_pointer_positions: &self.active_pointer_positions,
                    modifiers,
                    window_requests: &mut *window_requests,
                    focus_owner: &mut self.focus_owner,
                };
                run_pointer_handler_for_node(
                    &mut dispatch_ctx,
                    node_id,
                    PointerEventPass::Final,
                    handler.as_ref(),
                );
            }
        }
    }

    fn dispatch_keyboard_event(
        &mut self,
        root_node: NodeId,
        queued: QueuedKeyEvent,
        window_requests: &mut WindowRequests,
    ) -> Option<FocusDirection> {
        let path = collect_focus_chain_node_ids(root_node, &self.tree, &self.focus_owner);
        let mut events = vec![queued.event];
        let mut context = KeyboardInputDispatchContext {
            tree: &self.tree,
            metadatas: &self.metadatas,
            keyboard_events: &mut events,
            modifiers: queued.modifiers,
            window_requests,
            focus_owner: &mut self.focus_owner,
        };
        for node_id in path.iter().copied() {
            let Some(node) = context.tree.get(node_id).map(|node| node.get()) else {
                continue;
            };
            for action in node.modifier.ordered_actions() {
                if let OrderedModifierAction::KeyboardPreviewInput(modifier) = action {
                    run_keyboard_modifier_for_node(&mut context, node_id, modifier.as_ref());
                }
            }
            for handler in &node.keyboard_preview_handlers {
                run_keyboard_handler_for_node(&mut context, node_id, handler.as_ref());
            }
        }
        for node_id in path.iter().rev().copied() {
            let Some(node) = context.tree.get(node_id).map(|node| node.get()) else {
                continue;
            };
            for action in node.modifier.ordered_actions() {
                if let OrderedModifierAction::KeyboardInput(modifier) = action {
                    run_keyboard_modifier_for_node(&mut context, node_id, modifier.as_ref());
                }
            }
            for handler in &node.keyboard_handlers {
                run_keyboard_handler_for_node(&mut context, node_id, handler.as_ref());
            }
        }
        dispatch_default_focus_keyboard_navigation(
            context.tree,
            context.keyboard_events,
            context.modifiers,
            context.focus_owner,
        )
    }

    fn dispatch_clipboard_action(&mut self, root_node: NodeId, request: QueuedClipboardAction) {
        let path = collect_focus_chain_node_ids(root_node, &self.tree, &self.focus_owner);
        let mut response = None;
        for node_id in path.into_iter().rev() {
            let Some(context) = resolve_node_input_context(&self.tree, &self.metadatas, node_id)
            else {
                continue;
            };
            let node = self.tree.get(node_id).expect("focused node").get();
            for action in node.modifier.ordered_actions() {
                if let OrderedModifierAction::ClipboardInput(modifier) = action {
                    let boundary = nearest_replay_boundary_instance_key(node_id, &self.tree);
                    let _node = push_current_node_with_instance_logic_id(
                        node_id,
                        context.instance_logic_id,
                        &context.fn_name,
                    );
                    let _instance = push_current_component_instance_key(boundary);
                    let _phase = push_phase(RuntimePhase::Input);
                    let _focus = bind_focus_owner(&mut self.focus_owner);
                    modifier.on_clipboard_input(ClipboardInput {
                        action: &request.action,
                        response: &mut response,
                    });
                    if response.is_some() {
                        break;
                    }
                }
            }
            if response.is_some() {
                break;
            }
        }
        (request.reply)(response.unwrap_or(ClipboardResponse::Ignored));
    }

    fn dispatch_ime_input(
        &mut self,
        root_node: NodeId,
        events: &mut Vec<winit::event::Ime>,
        window_requests: &mut WindowRequests,
    ) {
        if events.is_empty() {
            return;
        }
        let path = collect_focus_chain_node_ids(root_node, &self.tree, &self.focus_owner);
        for node_id in path.iter().copied() {
            let Some(node) = self.tree.get(node_id).map(|node| node.get()) else {
                continue;
            };
            for action in node.modifier.ordered_actions() {
                if let OrderedModifierAction::ImePreviewInput(modifier) = action {
                    run_ime_modifier_for_node(
                        &self.tree,
                        &self.metadatas,
                        node_id,
                        modifier.as_ref(),
                        events,
                        window_requests,
                        &mut self.focus_owner,
                    );
                }
            }
            for handler in &node.ime_preview_handlers {
                run_ime_handler_for_node(
                    &self.tree,
                    &self.metadatas,
                    node_id,
                    handler.as_ref(),
                    events,
                    window_requests,
                    &mut self.focus_owner,
                );
            }
        }
        for node_id in path.iter().rev().copied() {
            let Some(node) = self.tree.get(node_id).map(|node| node.get()) else {
                continue;
            };
            for action in node.modifier.ordered_actions() {
                if let OrderedModifierAction::ImeInput(modifier) = action {
                    run_ime_modifier_for_node(
                        &self.tree,
                        &self.metadatas,
                        node_id,
                        modifier.as_ref(),
                        events,
                        window_requests,
                        &mut self.focus_owner,
                    );
                }
            }
            for handler in &node.ime_handlers {
                run_ime_handler_for_node(
                    &self.tree,
                    &self.metadatas,
                    node_id,
                    handler.as_ref(),
                    events,
                    window_requests,
                    &mut self.focus_owner,
                );
            }
        }
    }

    /// Compute the ComponentTree into a render graph
    ///
    /// This method processes the component tree through three main phases:
    /// 1. **Measure Phase**: Calculate sizes and positions for all components
    /// 2. **Graph Generation**: Extract render fragments from component
    ///    metadata
    /// 3. **State Handling**: Process user interactions and events
    ///
    /// Returns a tuple of (graph, window_requests) where the graph contains
    /// the render ops for the current frame.
    #[tracing::instrument(level = "debug", skip(self, params))]
    pub(crate) fn compute(
        &mut self,
        params: ComputeParams<'_>,
        mode: ComputeMode<'_>,
    ) -> (
        RenderGraph,
        WindowRequests,
        LayoutFrameDiagnostics,
        std::time::Duration,
        Option<FocusDirection>,
        bool,
        bool,
    ) {
        self.metadatas.geometry_visits = 0;
        self.metadatas.geometry_changed.clear();
        self.metadatas.paint_changed.clear();
        self.focus_owner.begin_layout();
        let ComputeParams {
            screen_size,
            mut cursor_position,
            mut pointer_changes,
            keyboard_events,
            ime_events,
            retry_focus_move,
            retry_focus_reveal,
            modifiers,
            layout_dirty_nodes,
        } = params;
        let Some(root_node) = self
            .tree
            .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))
        else {
            return (
                RenderGraph::default(),
                WindowRequests::default(),
                LayoutFrameDiagnostics::default(),
                std::time::Duration::ZERO,
                None,
                false,
                false,
            );
        };
        let screen_constraint = Constraint::exact(screen_size.width, screen_size.height);
        for key in &layout_dirty_nodes.transform_nodes {
            if let Some(&node_id) = self.node_ids_by_instance_key.get(key) {
                self.metadatas.mark_geometry_dirty(node_id);
            }
        }
        for key in &layout_dirty_nodes.paint_nodes {
            if let Some(&node_id) = self.node_ids_by_instance_key.get(key) {
                self.metadatas.mark_paint_dirty(node_id);
            }
        }
        let StructureReconcileResult {
            changed_nodes: structural_dirty_nodes,
            removed_nodes,
        } = crate::runtime::take_layout_structure_changes();
        self.remove_layout_snapshots(&removed_nodes);
        // Layout nodes dropped from the tree must also drop the state-read
        // dependencies they registered during measurement, otherwise recycled
        // instance keys would keep stale placement dirtiness alive.
        crate::runtime::remove_layout_state_read_dependencies(&removed_nodes);

        let mut measure_self_nodes = layout_dirty_nodes.measure_self_nodes.clone();
        for key in &layout_dirty_nodes.parent_data_nodes {
            if let Some(&node_id) = self.node_ids_by_instance_key.get(key) {
                let mut parent = self.tree.get(node_id).and_then(|node| node.parent());
                while let Some(parent_id) = parent {
                    let node = &self.tree[parent_id];
                    if node.get().role == NodeRole::Layout || node.parent().is_none() {
                        measure_self_nodes.insert(node.get().instance_key);
                        break;
                    }
                    parent = node.parent();
                }
            }
        }
        let mut dirty_nodes_self = measure_self_nodes.clone();
        dirty_nodes_self.extend(layout_dirty_nodes.placement_self_nodes.iter().copied());
        let dirty_nodes_param = dirty_nodes_self.len() as u64;
        let dirty_nodes_structural = structural_dirty_nodes.len() as u64;
        let dirty_prepare_start = Instant::now();
        dirty_nodes_self.extend(structural_dirty_nodes.iter().copied());
        // A node whose direct layout children changed must recompute
        // placements: its cached placements are keyed by the previous
        // children's instance keys, so reusing them against a different
        // child set silently drops the new children and leaves them
        // without a position.
        let mut placement_dirty_nodes = layout_dirty_nodes.placement_self_nodes.clone();
        placement_dirty_nodes.extend(structural_dirty_nodes.iter().copied());
        let dirty_nodes_effective = expand_dirty_nodes_with_ancestors(
            &self.tree,
            &self.node_ids_by_instance_key,
            &dirty_nodes_self,
        );
        let mut dirty_children: HashMap<u64, Vec<NodeId>> = HashMap::default();
        for key in &dirty_nodes_effective {
            let Some(&id) = self.node_ids_by_instance_key.get(key) else {
                continue;
            };
            if self.tree[id].get().role != NodeRole::Layout {
                continue;
            }
            for ancestor in id.ancestors(&self.tree).skip(1) {
                let node = self.tree[ancestor].get();
                dirty_children
                    .entry(node.instance_key)
                    .or_default()
                    .push(id);
                if node.role == NodeRole::Layout {
                    break;
                }
            }
        }
        let dirty_expand_ns = dirty_prepare_start.elapsed().as_nanos() as u64;
        let mut diagnostics = LayoutDiagnosticsCollector::default();

        let changed_frame_nodes = std::mem::take(&mut self.changed_frame_nodes);
        let removed_frame_nodes = std::mem::take(&mut self.removed_frame_nodes);
        #[cfg(feature = "profiling")]
        {
            self.profiler_dirty_nodes
                .extend(changed_frame_nodes.iter().copied());
            self.profiler_dirty_nodes
                .extend(removed_frame_nodes.iter().copied());
            self.profiler_dirty_nodes.extend(
                dirty_nodes_effective
                    .iter()
                    .filter_map(|key| self.node_ids_by_instance_key.get(key).copied()),
            );
            self.profiler_dirty_nodes.insert(root_node);
        }
        self.pointer_index
            .invalidate(&changed_frame_nodes, &removed_frame_nodes);
        self.accessibility_index
            .invalidate(&changed_frame_nodes, &removed_frame_nodes);
        self.focus_owner.sync_changes(
            &self.tree,
            &self.composition_order,
            &changed_frame_nodes,
            &removed_frame_nodes,
        );
        self.focus_owner.commit_pending();

        measure_self_nodes.extend(structural_dirty_nodes);
        let layout_ctx = LayoutContext::new(
            &mut self.layout_snapshots,
            &measure_self_nodes,
            &placement_dirty_nodes,
            &dirty_nodes_effective,
            &dirty_children,
            &mut diagnostics,
        );

        let measure_timer = Instant::now();
        debug!("Start measuring the component tree...");

        match measure_node(
            root_node,
            &screen_constraint,
            &self.tree,
            &mut self.metadatas,
            Some(&layout_ctx),
        ) {
            Ok(_root_computed_data) => {
                debug!("Component tree measured in {:?}", measure_timer.elapsed());
            }
            Err(e) => {
                panic!(
                    "Root node ({root_node:?}) measurement failed: {e:?}. Aborting draw command computation."
                );
            }
        }

        let mut diagnostics_snapshot = diagnostics.snapshot(
            dirty_nodes_param,
            dirty_nodes_structural,
            dirty_nodes_effective.len() as u64,
            dirty_expand_ns,
        );

        let (compute_resource_manager, gpu) = match mode {
            ComputeMode::Full {
                compute_resource_manager,
                gpu,
            } => (compute_resource_manager, gpu),
            #[cfg(feature = "testing")]
            ComputeMode::LayoutOnly => {
                diagnostics_snapshot.geometry_nodes_updated =
                    populate_layout_metadata(root_node, &self.tree, &mut self.metadatas);
                self.collect_globally_positioned_invocations(root_node);
                diagnostics_snapshot.accessibility_nodes_updated =
                    self.sync_accessibility(root_node) as u64;
                let mut geometry_changes = self.metadatas.geometry_changed.clone();
                geometry_changes.extend(changed_frame_nodes.iter().copied());
                self.focus_owner.sync_geometry_changes(
                    &self.tree,
                    &self.composition_order,
                    &self.metadatas,
                    &geometry_changes,
                );
                let _ = self.dispatch_pointer_input(
                    root_node,
                    pointer_changes.as_mut_slice(),
                    &mut cursor_position,
                    modifiers,
                );
                diagnostics_snapshot.geometry_nodes_visited = self.metadatas.geometry_visits;
                diagnostics_snapshot.pointer_nodes_updated = self.pointer_work.0;
                diagnostics_snapshot.pointer_nodes_dispatched = self.pointer_work.1;
                diagnostics_snapshot.pointer_hit_test_candidates =
                    self.pointer_index.hit_candidates.get();
                return (
                    RenderGraph::default(),
                    WindowRequests::default(),
                    diagnostics_snapshot,
                    std::time::Duration::ZERO,
                    None,
                    false,
                    false,
                );
            }
        };

        diagnostics_snapshot.geometry_nodes_updated =
            populate_layout_metadata(root_node, &self.tree, &mut self.metadatas);
        let record_timer = Instant::now();
        let _focus_owner_guard = bind_focus_owner(&mut self.focus_owner);
        record_layout_commands(
            root_node,
            &self.tree,
            &mut self.metadatas,
            compute_resource_manager,
            gpu,
        );
        let record_cost = record_timer.elapsed();
        diagnostics_snapshot.geometry_nodes_updated +=
            populate_layout_metadata(root_node, &self.tree, &mut self.metadatas);
        self.collect_globally_positioned_invocations(root_node);
        diagnostics_snapshot.accessibility_nodes_updated =
            self.sync_accessibility(root_node) as u64;
        diagnostics_snapshot.paint_nodes_recorded = self.metadatas.paint_changed.len() as u64;

        let compute_draw_timer = Instant::now();
        debug!("Start computing render graph...");
        let (graph, scene_updates) = self.frame_scene.update(
            root_node,
            &self.tree,
            &self.metadatas,
            &changed_frame_nodes,
            &removed_frame_nodes,
            screen_size,
        );
        diagnostics_snapshot.scene_nodes_updated = scene_updates;
        let mut focus_geometry_changes = self.metadatas.geometry_changed.clone();
        focus_geometry_changes.extend(changed_frame_nodes.iter().copied());
        self.focus_owner.sync_geometry_changes(
            &self.tree,
            &self.composition_order,
            &self.metadatas,
            &focus_geometry_changes,
        );
        debug!(
            "Render graph built in {:?}, total ops: {}",
            compute_draw_timer.elapsed(),
            graph.ops().len()
        );

        let input_dispatch_timer = Instant::now();
        debug!("Start executing typed input dispatch...");
        let mut window_requests = self.dispatch_pointer_input(
            root_node,
            pointer_changes.as_mut_slice(),
            &mut cursor_position,
            modifiers,
        );

        diagnostics_snapshot.geometry_nodes_visited = self.metadatas.geometry_visits;
        diagnostics_snapshot.pointer_nodes_updated = self.pointer_work.0;
        diagnostics_snapshot.pointer_nodes_dispatched = self.pointer_work.1;
        diagnostics_snapshot.pointer_hit_test_candidates = self.pointer_index.hit_candidates.get();
        self.focus_owner.commit_pending();
        let requested_move = self
            .focus_owner
            .take_pending_navigation()
            .or(retry_focus_move);
        let pending_focus_move_retry = requested_move.and_then(|direction| {
            match try_dispatch_focus_move_request(&self.tree, direction, &mut self.focus_owner) {
                FocusMoveRequestResult::Retry(direction) => Some(direction),
                FocusMoveRequestResult::Moved | FocusMoveRequestResult::NotHandled => None,
            }
        });
        let mut focused_events: Vec<_> = keyboard_events
            .into_iter()
            .map(QueuedFocusedInput::Keyboard)
            .collect();
        focused_events.extend(
            clipboard::take_requests()
                .into_iter()
                .map(QueuedFocusedInput::Clipboard),
        );
        focused_events.sort_by_key(QueuedFocusedInput::sequence);
        self.pending_keyboard_events.extend(focused_events);
        self.pending_ime_events.extend(ime_events);
        let mut queued_events = std::mem::take(&mut self.pending_keyboard_events);
        let pending_focus_move_retry = pending_focus_move_retry.or_else(|| {
            if self.focus_owner.keyboard_is_deferred() {
                return None;
            }
            drain_pending_keyboard_events(&mut queued_events, |queued| {
                let retry = match queued {
                    QueuedFocusedInput::Keyboard(event) => {
                        self.dispatch_keyboard_event(root_node, event, &mut window_requests)
                    }
                    QueuedFocusedInput::Clipboard(request) => {
                        self.dispatch_clipboard_action(root_node, request);
                        None
                    }
                };
                self.focus_owner.commit_pending();
                (retry, self.focus_owner.keyboard_is_deferred())
            })
        });
        self.pending_keyboard_events = queued_events;
        self.focus_owner
            .retain_navigation_retry(pending_focus_move_retry);
        let pending_keyboard_retry = self.focus_owner.keyboard_is_deferred()
            && (!self.pending_keyboard_events.is_empty() || !self.pending_ime_events.is_empty());
        if pending_focus_move_retry.is_none() && !self.focus_owner.keyboard_is_deferred() {
            let mut ime_events = std::mem::take(&mut self.pending_ime_events);
            self.dispatch_ime_input(root_node, &mut ime_events, &mut window_requests);
        }
        let pending_focus_reveal_retry =
            if pending_focus_move_retry.is_none() && !self.focus_owner.keyboard_is_deferred() {
                dispatch_pending_focus_reveal_request(
                    &self.tree,
                    &self.metadatas,
                    &mut self.focus_owner,
                    retry_focus_reveal,
                )
            } else {
                false
            };

        debug!(
            "Typed input dispatch executed in {:?}",
            input_dispatch_timer.elapsed()
        );
        (
            graph,
            window_requests,
            diagnostics_snapshot,
            record_cost,
            pending_focus_move_retry,
            pending_focus_reveal_retry,
            pending_keyboard_retry,
        )
    }
}

fn drain_pending_keyboard_events<E>(
    queue: &mut VecDeque<E>,
    mut dispatch: impl FnMut(E) -> (Option<FocusDirection>, bool),
) -> Option<FocusDirection> {
    while let Some(event) = queue.pop_front() {
        let (retry, defer) = dispatch(event);
        if let Some(direction) = retry {
            return Some(direction);
        }
        if defer {
            return None;
        }
    }
    None
}

#[derive(Clone)]
struct NodeInputContext {
    base_abs_pos: PxPosition,
    abs_pos: PxPosition,
    event_clip_rect: Option<PxRect>,
    node_computed_data: ComputedData,
    instance_logic_id: u64,
    instance_key: u64,
    fn_name: String,
    parent_id: Option<indextree::NodeId>,
}

fn resolve_node_input_context(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
) -> Option<NodeInputContext> {
    let Some(node_ref) = tree.get(node_id) else {
        warn!("Node not found for node {node_id:?}; skipping input handling");
        return None;
    };
    let node = node_ref.get();
    if node.role != NodeRole::Layout {
        return None;
    }

    let Some(metadata) = metadatas.get(&node_id) else {
        warn!("Input metadata missing for node {node_id:?}; skipping input handling");
        return None;
    };
    let Some(base_abs_pos) = metadata.base_abs_position else {
        warn!("Base absolute position missing for node {node_id:?}; skipping input handling");
        return None;
    };
    let Some(abs_pos) = metadata.abs_position else {
        warn!("Absolute position missing for node {node_id:?}; skipping input handling");
        return None;
    };
    let event_clip_rect = metadata.event_clip_rect;
    let Some(node_computed_data) = metadata.computed_data else {
        warn!(
            "Computed data not found for node {:?} during input dispatch.",
            node_id
        );
        return None;
    };
    let instance_logic_id = node.instance_logic_id;
    let instance_key = node.instance_key;
    let fn_name = node.fn_name.as_str().to_owned();
    let parent_id = node_ref.parent();

    Some(NodeInputContext {
        base_abs_pos,
        abs_pos,
        event_clip_rect,
        node_computed_data,
        instance_logic_id,
        instance_key,
        fn_name,
        parent_id,
    })
}

fn attach_ime_position_if_needed(window_requests: &mut WindowRequests, abs_pos: PxPosition) {
    if let Some(ref mut ime_request) = window_requests.ime_request
        && ime_request.position.is_none()
    {
        ime_request.position = Some(abs_pos + ime_request.local_position);
    }
}

fn inside_shape_clips(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
    position: PxPosition,
) -> bool {
    if !metadatas
        .get(&node_id)
        .is_some_and(|metadata| metadata.has_shape_clipping)
    {
        return true;
    }
    node_id.ancestors(tree).all(|ancestor| {
        let Some(metadata) = metadatas.get(&ancestor) else {
            return true;
        };
        let (Some(mut region), Some(origin)) = (metadata.clip_region, metadata.abs_position) else {
            return true;
        };
        region.position = region.position + origin;
        region.contains(position)
    })
}

fn bounds_hit_path(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    position: PxPosition,
) -> Option<indextree::NodeId> {
    let node = tree.get(node_id)?;
    if node.get().role != NodeRole::Layout {
        return None;
    }
    let metadata = metadatas.get(&node_id)?;
    let base_abs_pos = metadata.base_abs_position?;
    let abs_pos = metadata.abs_position?;
    let size = metadata.computed_data?;
    if size.width.0 <= 0 || size.height.0 <= 0 {
        return None;
    }
    if let Some(clip_rect) = metadata.event_clip_rect
        && !clip_rect.contains(position)
    {
        return None;
    }
    if !inside_shape_clips(tree, metadatas, node_id, position) {
        return None;
    }
    let bounds = PxRect::from_position_size(abs_pos, PxSize::new(size.width, size.height));
    let node_data = node.get();
    let has_pointer_modifier = node_data.modifier.ordered_actions().iter().any(|action| {
        matches!(
            action,
            OrderedModifierAction::Cursor(_)
                | OrderedModifierAction::PointerPreviewInput(_)
                | OrderedModifierAction::PointerInput(_)
                | OrderedModifierAction::PointerFinalInput(_)
        )
    });
    let has_pointer_handler = !node_data.pointer_preview_handlers.is_empty()
        || !node_data.pointer_handlers.is_empty()
        || !node_data.pointer_final_handlers.is_empty();
    if !has_pointer_modifier && !has_pointer_handler {
        return None;
    }
    let node_handles_hover = has_pointer_modifier
        && node_handles_pointer_at_position(node_data, base_abs_pos, size, position)
        || bounds.contains(position) && has_pointer_handler;
    node_handles_hover.then_some(node_id)
}

fn node_handles_pointer_at_position(
    node: &crate::component_tree::ComponentNode,
    base_abs_pos: PxPosition,
    size: ComputedData,
    position: PxPosition,
) -> bool {
    let mut current_abs_pos = base_abs_pos;
    let size = PxSize::new(size.width, size.height);
    for action in node.modifier.ordered_actions() {
        match action {
            OrderedModifierAction::Placement(placement) => {
                current_abs_pos = placement.node().transform_position(current_abs_pos);
            }
            OrderedModifierAction::Cursor(_)
            | OrderedModifierAction::PointerPreviewInput(_)
            | OrderedModifierAction::PointerInput(_)
            | OrderedModifierAction::PointerFinalInput(_) => {
                let bounds = PxRect::from_position_size(current_abs_pos, size);
                if bounds.contains(position) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn resolve_node_hover_cursor_icon(
    node: &crate::component_tree::ComponentNode,
    base_abs_pos: PxPosition,
    size: ComputedData,
    position: PxPosition,
) -> Option<winit::window::CursorIcon> {
    let mut current_abs_pos = base_abs_pos;
    let size = PxSize::new(size.width, size.height);
    let mut resolved = None;
    for action in node.modifier.ordered_actions() {
        match action {
            OrderedModifierAction::Placement(placement) => {
                current_abs_pos = placement.node().transform_position(current_abs_pos);
            }
            OrderedModifierAction::Cursor(cursor) => {
                let bounds = PxRect::from_position_size(current_abs_pos, size);
                if bounds.contains(position) {
                    resolved = Some(cursor.cursor_icon());
                }
            }
            _ => {}
        }
    }
    resolved
}

fn build_pointer_change_paths(
    index: &pointer::PointerIndex,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    pointer_changes: &[PointerChange],
    cursor_position: Option<PxPosition>,
    active_pointer_paths: &mut HashMap<u64, Vec<u64>>,
) -> Vec<Vec<u64>> {
    let mut paths = Vec::with_capacity(pointer_changes.len());
    for change in pointer_changes {
        let debug_position = change.resolved_position(cursor_position);
        let path = match &change.content {
            CursorEventContent::Pressed(_) => {
                let computed_path = hit_path_instance_keys(index, tree, metadatas, debug_position);
                if computed_path.is_empty() {
                    active_pointer_paths.remove(&change.pointer_id);
                } else {
                    active_pointer_paths.insert(change.pointer_id, computed_path.clone());
                }
                computed_path
            }
            CursorEventContent::Moved(position) => active_pointer_paths
                .get(&change.pointer_id)
                .cloned()
                .unwrap_or_else(|| hit_path_instance_keys(index, tree, metadatas, Some(*position))),
            CursorEventContent::Released(_) | CursorEventContent::Cancelled(_) => {
                let computed = active_pointer_paths
                    .get(&change.pointer_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        hit_path_instance_keys(index, tree, metadatas, debug_position)
                    });
                active_pointer_paths.remove(&change.pointer_id);
                computed
            }
            CursorEventContent::Scroll(_) => active_pointer_paths
                .get(&change.pointer_id)
                .cloned()
                .unwrap_or_else(|| hit_path_instance_keys(index, tree, metadatas, debug_position)),
        };
        paths.push(path);
    }
    paths
}

fn hit_path_instance_keys(
    index: &pointer::PointerIndex,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    position: Option<PxPosition>,
) -> Vec<u64> {
    index
        .hit_path(tree, metadatas, position)
        .into_iter()
        .filter_map(|node_id| tree.get(node_id).map(|node| node.get().instance_key))
        .collect()
}

fn collect_focus_chain_node_ids(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    focus_owner: &FocusOwner,
) -> Vec<indextree::NodeId> {
    fn live_node(
        tree: &ComponentNodeTree,
        node_id: indextree::NodeId,
    ) -> Option<&indextree::Node<ComponentNode>> {
        tree.get(node_id).filter(|node| !node.is_removed())
    }

    let Some(focused_node_id) = focus_owner.active_component_node_id() else {
        return Vec::new();
    };
    if live_node(tree, focused_node_id).is_none() {
        return Vec::new();
    }

    let mut chain = Vec::new();
    let mut current = Some(focused_node_id);
    while let Some(node_id) = current {
        chain.push(node_id);
        if node_id == root_node {
            break;
        }
        current = live_node(tree, node_id).and_then(|node| node.parent());
    }
    if chain.last().copied() != Some(root_node) {
        return Vec::new();
    }
    chain.reverse();
    chain
}

#[derive(Clone)]
struct PointerNodePlan {
    context: NodeInputContext,
    actions: Vec<OrderedModifierAction>,
}

struct PointerDispatchPlan {
    preorder: Vec<indextree::NodeId>,
    postorder: Vec<indextree::NodeId>,
    nodes: HashMap<indextree::NodeId, PointerNodePlan>,
}

struct PointerInputDispatchContext<'a> {
    tree: &'a ComponentNodeTree,
    metadatas: &'a ComponentNodeMetaDatas,
    cursor_position: &'a mut Option<PxPosition>,
    pointer_changes: &'a mut [PointerChange],
    pointer_change_paths: &'a [Vec<u64>],
    nodes: &'a HashMap<indextree::NodeId, PointerNodePlan>,
    active_pointer_paths: &'a HashMap<u64, Vec<u64>>,
    active_pointer_positions: &'a HashMap<u64, Option<PxPosition>>,
    modifiers: winit::keyboard::ModifiersState,
    window_requests: &'a mut WindowRequests,
    focus_owner: &'a mut FocusOwner,
}

fn dispatch_pointer_modifiers_for_node_pass(
    dispatch_ctx: &mut PointerInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    pass: PointerEventPass,
) {
    let nodes = dispatch_ctx.nodes;
    let Some(plan) = nodes.get(&node_id) else {
        return;
    };
    let mut current_abs_pos = plan.context.base_abs_pos;
    for action in &plan.actions {
        match action {
            OrderedModifierAction::Placement(placement) => {
                current_abs_pos = placement.node().transform_position(current_abs_pos);
            }
            OrderedModifierAction::PointerPreviewInput(modifier)
                if pass == PointerEventPass::Initial =>
            {
                run_pointer_modifier_for_node(
                    dispatch_ctx,
                    node_id,
                    pass,
                    current_abs_pos,
                    modifier.as_ref(),
                );
            }
            OrderedModifierAction::PointerInput(modifier) if pass == PointerEventPass::Main => {
                run_pointer_modifier_for_node(
                    dispatch_ctx,
                    node_id,
                    pass,
                    current_abs_pos,
                    modifier.as_ref(),
                );
            }
            OrderedModifierAction::PointerFinalInput(modifier)
                if pass == PointerEventPass::Final =>
            {
                run_pointer_modifier_for_node(
                    dispatch_ctx,
                    node_id,
                    pass,
                    current_abs_pos,
                    modifier.as_ref(),
                );
            }
            _ => {}
        }
    }
}

fn run_pointer_handler_for_node(
    dispatch_ctx: &mut PointerInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    pass: PointerEventPass,
    pointer_handler: &PointerInputHandlerFn,
) {
    let Some(abs_pos) = dispatch_ctx
        .nodes
        .get(&node_id)
        .map(|node| node.context.abs_pos)
    else {
        return;
    };
    run_pointer_input_for_node(dispatch_ctx, node_id, pass, abs_pos, pointer_handler);
}

fn run_pointer_modifier_for_node(
    dispatch_ctx: &mut PointerInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    pass: PointerEventPass,
    abs_pos_override: PxPosition,
    pointer_modifier: &dyn PointerInputModifierNode,
) {
    run_pointer_input_for_node(dispatch_ctx, node_id, pass, abs_pos_override, |input| {
        pointer_modifier.on_pointer_input(input)
    });
}

fn run_pointer_input_for_node<F>(
    dispatch_ctx: &mut PointerInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    pass: PointerEventPass,
    abs_pos_override: PxPosition,
    dispatch: F,
) where
    F: FnOnce(PointerInput<'_>),
{
    let nodes = dispatch_ctx.nodes;
    let Some(NodeInputContext {
        base_abs_pos: _,
        abs_pos: _,
        event_clip_rect,
        node_computed_data,
        instance_logic_id,
        instance_key,
        fn_name,
        parent_id,
    }) = nodes.get(&node_id).map(|node| &node.context)
    else {
        return;
    };
    let event_clip_rect = *event_clip_rect;
    let node_computed_data = *node_computed_data;
    let instance_logic_id = *instance_logic_id;
    let instance_key = *instance_key;
    let parent_id = *parent_id;
    #[cfg(not(feature = "profiling"))]
    let _ = parent_id;

    let abs_pos = abs_pos_override;

    let owns_sample = dispatch_ctx
        .pointer_change_paths
        .iter()
        .any(|path| path.contains(&instance_key));
    let captured_position = if owns_sample {
        None
    } else {
        dispatch_ctx
            .active_pointer_paths
            .iter()
            .filter_map(|(&pointer_id, path)| {
                if pointer_id == MOUSE_POINTER_ID || !path.contains(&instance_key) {
                    return None;
                }
                dispatch_ctx
                    .active_pointer_positions
                    .get(&pointer_id)
                    .map(|position| (pointer_id, *position))
            })
            .min_by_key(|(pointer_id, _)| *pointer_id)
            .map(|(_, position)| position)
    };
    let mut captured_cursor_position = captured_position.flatten();
    let mut cursor_position_ref = if captured_position.is_some() {
        &mut captured_cursor_position
    } else {
        &mut *dispatch_ctx.cursor_position
    };
    let mut dummy_cursor_position = None;
    if let (Some(cursor_pos), Some(clip_rect)) = (*cursor_position_ref, event_clip_rect)
        && !clip_rect.contains(cursor_pos)
    {
        cursor_position_ref = &mut dummy_cursor_position;
    }
    if let Some(position) = *cursor_position_ref
        && !inside_shape_clips(dispatch_ctx.tree, dispatch_ctx.metadatas, node_id, position)
    {
        cursor_position_ref = &mut dummy_cursor_position;
    }
    let current_cursor_position = cursor_position_ref.map(|pos| pos - abs_pos);
    let mut selected_change_indices = Vec::new();
    let mut local_pointer_changes = Vec::new();
    for (index, change) in dispatch_ctx.pointer_changes.iter().enumerate() {
        if change.is_consumed() {
            continue;
        }
        let Some(path) = dispatch_ctx.pointer_change_paths.get(index) else {
            continue;
        };
        if path.contains(&instance_key) {
            selected_change_indices.push(index);
            local_pointer_changes.push(change.clone());
        }
    }
    #[cfg(feature = "profiling")]
    let mut profiler_guard = Some(ProfilerScopeGuard::new(
        ProfilerPhase::Input,
        Some(node_id),
        parent_id,
        Some(fn_name.as_str()),
    ));
    let replay_boundary_instance_key =
        nearest_replay_boundary_instance_key(node_id, dispatch_ctx.tree);
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(replay_boundary_instance_key);
    let _phase_guard = push_phase(RuntimePhase::Input);
    let _focus_owner_guard = bind_focus_owner(dispatch_ctx.focus_owner);
    let input = PointerInput {
        pass,
        computed_data: node_computed_data,
        cursor_position_rel: current_cursor_position,
        cursor_position_abs: cursor_position_ref,
        pointer_changes: &mut local_pointer_changes,
        key_modifiers: dispatch_ctx.modifiers,
        ime_request: &mut dispatch_ctx.window_requests.ime_request,
        request_window_drag: &mut dispatch_ctx.window_requests.request_window_drag,
    };
    crate::runtime::with_pointer_state_reader(instance_key, || dispatch(input));
    for (local_change, &original_index) in local_pointer_changes
        .iter()
        .zip(selected_change_indices.iter())
    {
        if let Some(original) = dispatch_ctx.pointer_changes.get_mut(original_index) {
            *original = local_change.clone();
        }
    }

    #[cfg(feature = "profiling")]
    {
        let abs_tuple = (abs_pos.x_px().0, abs_pos.y_px().0);
        if let Some(g) = &mut profiler_guard {
            g.set_positions(Some(abs_tuple));
        }
        let _ = profiler_guard.take();
    }
    attach_ime_position_if_needed(dispatch_ctx.window_requests, abs_pos);
}

fn run_keyboard_handler_for_node(
    dispatch_ctx: &mut KeyboardInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    keyboard_handler: &KeyboardInputHandlerFn,
) {
    run_keyboard_input_for_node(dispatch_ctx, node_id, keyboard_handler);
}

struct KeyboardInputDispatchContext<'a> {
    tree: &'a ComponentNodeTree,
    metadatas: &'a ComponentNodeMetaDatas,
    keyboard_events: &'a mut Vec<winit::event::KeyEvent>,
    modifiers: winit::keyboard::ModifiersState,
    window_requests: &'a mut WindowRequests,
    focus_owner: &'a mut FocusOwner,
}

fn run_keyboard_modifier_for_node(
    dispatch_ctx: &mut KeyboardInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    keyboard_modifier: &dyn KeyboardInputModifierNode,
) {
    run_keyboard_input_for_node(dispatch_ctx, node_id, |input| {
        keyboard_modifier.on_keyboard_input(input)
    });
}

fn run_keyboard_input_for_node<F>(
    dispatch_ctx: &mut KeyboardInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    dispatch: F,
) where
    F: FnOnce(KeyboardInput<'_>),
{
    let Some(NodeInputContext {
        abs_pos,
        event_clip_rect: _,
        node_computed_data,
        instance_logic_id,
        instance_key: _,
        fn_name,
        parent_id,
        ..
    }) = resolve_node_input_context(dispatch_ctx.tree, dispatch_ctx.metadatas, node_id)
    else {
        return;
    };
    #[cfg(not(feature = "profiling"))]
    let _ = parent_id;

    #[cfg(feature = "profiling")]
    let mut profiler_guard = Some(ProfilerScopeGuard::new(
        ProfilerPhase::Input,
        Some(node_id),
        parent_id,
        Some(fn_name.as_str()),
    ));
    let replay_boundary_instance_key =
        nearest_replay_boundary_instance_key(node_id, dispatch_ctx.tree);
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(replay_boundary_instance_key);
    let _phase_guard = push_phase(RuntimePhase::Input);
    let _focus_owner_guard = bind_focus_owner(dispatch_ctx.focus_owner);
    let input = KeyboardInput {
        computed_data: node_computed_data,
        keyboard_events: dispatch_ctx.keyboard_events,
        key_modifiers: dispatch_ctx.modifiers,
        ime_request: &mut dispatch_ctx.window_requests.ime_request,
    };
    dispatch(input);

    #[cfg(feature = "profiling")]
    {
        let abs_tuple = (abs_pos.x_px().0, abs_pos.y_px().0);
        if let Some(g) = &mut profiler_guard {
            g.set_positions(Some(abs_tuple));
        }
        let _ = profiler_guard.take();
    }
    attach_ime_position_if_needed(dispatch_ctx.window_requests, abs_pos);
}

fn run_ime_handler_for_node(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
    ime_handler: &ImeInputHandlerFn,
    ime_events: &mut Vec<winit::event::Ime>,
    window_requests: &mut WindowRequests,
    focus_owner: &mut FocusOwner,
) {
    run_ime_input_for_node(
        tree,
        metadatas,
        node_id,
        ime_events,
        window_requests,
        focus_owner,
        ime_handler,
    );
}

fn run_ime_modifier_for_node(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
    ime_modifier: &dyn ImeInputModifierNode,
    ime_events: &mut Vec<winit::event::Ime>,
    window_requests: &mut WindowRequests,
    focus_owner: &mut FocusOwner,
) {
    run_ime_input_for_node(
        tree,
        metadatas,
        node_id,
        ime_events,
        window_requests,
        focus_owner,
        |input| ime_modifier.on_ime_input(input),
    );
}

fn run_ime_input_for_node<F>(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
    ime_events: &mut Vec<winit::event::Ime>,
    window_requests: &mut WindowRequests,
    focus_owner: &mut FocusOwner,
    dispatch: F,
) where
    F: FnOnce(ImeInput<'_>),
{
    let Some(NodeInputContext {
        abs_pos,
        event_clip_rect: _,
        node_computed_data,
        instance_logic_id,
        instance_key: _,
        fn_name,
        parent_id,
        ..
    }) = resolve_node_input_context(tree, metadatas, node_id)
    else {
        return;
    };
    #[cfg(not(feature = "profiling"))]
    let _ = parent_id;

    #[cfg(feature = "profiling")]
    let mut profiler_guard = Some(ProfilerScopeGuard::new(
        ProfilerPhase::Input,
        Some(node_id),
        parent_id,
        Some(fn_name.as_str()),
    ));
    let replay_boundary_instance_key = nearest_replay_boundary_instance_key(node_id, tree);
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(replay_boundary_instance_key);
    let _phase_guard = push_phase(RuntimePhase::Input);
    let _focus_owner_guard = bind_focus_owner(focus_owner);
    let input = ImeInput {
        computed_data: node_computed_data,
        ime_events,
        ime_request: &mut window_requests.ime_request,
    };
    dispatch(input);

    #[cfg(feature = "profiling")]
    {
        let abs_tuple = (abs_pos.x_px().0, abs_pos.y_px().0);
        if let Some(g) = &mut profiler_guard {
            g.set_positions(Some(abs_tuple));
        }
        let _ = profiler_guard.take();
    }
    attach_ime_position_if_needed(window_requests, abs_pos);
}

fn dispatch_default_focus_keyboard_navigation(
    tree: &ComponentNodeTree,
    keyboard_events: &mut Vec<winit::event::KeyEvent>,
    modifiers: winit::keyboard::ModifiersState,
    focus_owner: &mut FocusOwner,
) -> Option<FocusDirection> {
    if keyboard_events.is_empty() {
        return None;
    }

    let mut pending_focus_move_retry = None;
    keyboard_events.retain(|event| {
        if pending_focus_move_retry.is_some() {
            return false;
        }
        let Some(direction) = default_focus_navigation_direction(event, modifiers) else {
            return true;
        };
        match try_dispatch_focus_move_request(tree, direction, focus_owner) {
            FocusMoveRequestResult::Moved => false,
            FocusMoveRequestResult::Retry(direction) => {
                pending_focus_move_retry = Some(direction);
                false
            }
            FocusMoveRequestResult::NotHandled => true,
        }
    });
    pending_focus_move_retry
}

fn default_focus_navigation_direction(
    event: &winit::event::KeyEvent,
    modifiers: winit::keyboard::ModifiersState,
) -> Option<FocusDirection> {
    if event.state != winit::event::ElementState::Pressed {
        return None;
    }

    if modifiers.control_key() || modifiers.alt_key() || modifiers.super_key() {
        return None;
    }

    match &event.logical_key {
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
            if modifiers.shift_key() {
                Some(FocusDirection::Previous)
            } else {
                Some(FocusDirection::Next)
            }
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
            (!modifiers.shift_key()).then_some(FocusDirection::Left)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
            (!modifiers.shift_key()).then_some(FocusDirection::Right)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
            (!modifiers.shift_key()).then_some(FocusDirection::Up)
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {
            (!modifiers.shift_key()).then_some(FocusDirection::Down)
        }
        _ => None,
    }
}

enum FocusMoveRequestResult {
    NotHandled,
    Moved,
    Retry(FocusDirection),
}

fn try_dispatch_focus_move_request(
    tree: &ComponentNodeTree,
    direction: FocusDirection,
    focus_owner: &mut FocusOwner,
) -> FocusMoveRequestResult {
    if !focus_owner.allows_navigation() {
        return FocusMoveRequestResult::NotHandled;
    }
    if focus_owner.move_focus(direction) {
        return FocusMoveRequestResult::Moved;
    }
    if dispatch_focus_beyond_bounds_request(tree, direction, focus_owner) {
        return FocusMoveRequestResult::Retry(direction);
    }
    FocusMoveRequestResult::NotHandled
}

fn dispatch_focus_beyond_bounds_request(
    tree: &ComponentNodeTree,
    direction: FocusDirection,
    focus_owner: &FocusOwner,
) -> bool {
    let mut current = focus_owner.active_component_node_id();
    while let Some(node_id) = current {
        let Some(node_ref) = tree.get(node_id) else {
            break;
        };
        let node = node_ref.get();
        if let Some(handler) = &node.focus_beyond_bounds_handler
            && handler.call(direction)
        {
            return true;
        }
        current = node_ref.parent();
    }
    false
}

fn dispatch_pending_focus_reveal_request(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    focus_owner: &mut FocusOwner,
    retry_active_focus: bool,
) -> bool {
    let handle_id = if retry_active_focus {
        focus_owner.active_handle_id()
    } else {
        focus_owner.take_pending_reveal()
    };
    let Some(handle_id) = handle_id else {
        return false;
    };
    dispatch_focus_reveal_request(tree, metadatas, focus_owner, handle_id)
}

fn dispatch_focus_reveal_request(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    focus_owner: &FocusOwner,
    handle_id: FocusHandleId,
) -> bool {
    let Some(target_node_id) = focus_owner.component_node_id_of(handle_id) else {
        return false;
    };
    let Some(target_rect) = component_node_bounds_rect(metadatas, target_node_id) else {
        return false;
    };

    let mut current = Some(target_node_id);
    while let Some(node_id) = current {
        let Some(node_ref) = tree.get(node_id) else {
            break;
        };
        let Some(viewport_rect) = component_node_viewport_rect(metadatas, node_id) else {
            current = node_ref.parent();
            continue;
        };
        let node = node_ref.get();
        if let Some(handler) = &node.focus_reveal_handler {
            let handled = handler.call(crate::focus::FocusRevealRequest::new(
                target_rect,
                viewport_rect,
            ));
            if handled {
                return true;
            }
        }
        current = node_ref.parent();
    }
    false
}

fn component_node_bounds_rect(
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
) -> Option<PxRect> {
    let metadata = metadatas.get(&node_id)?;
    let abs_position = metadata.abs_position?;
    let computed_data = metadata.computed_data?;
    Some(PxRect::from_position_size(
        abs_position,
        PxSize::new(computed_data.width, computed_data.height),
    ))
}

fn component_node_viewport_rect(
    metadatas: &ComponentNodeMetaDatas,
    node_id: indextree::NodeId,
) -> Option<PxRect> {
    let node_rect = component_node_bounds_rect(metadatas, node_id)?;
    let metadata = metadatas.get(&node_id)?;
    Some(
        metadata
            .event_clip_rect
            .and_then(|clip_rect| clip_rect.intersection(&node_rect))
            .unwrap_or(node_rect),
    )
}

fn expand_dirty_nodes_with_ancestors(
    tree: &ComponentNodeTree,
    node_ids: &HashMap<u64, indextree::NodeId>,
    dirty_nodes_self: &HashSet<u64>,
) -> HashSet<u64> {
    let mut expanded = HashSet::default();
    for dirty_key in dirty_nodes_self {
        let mut current = node_ids.get(dirty_key).copied();
        while let Some(node_id) = current {
            let Some(node) = tree.get(node_id) else { break };
            if !expanded.insert(node.get().instance_key) {
                break;
            }
            current = node.parent();
        }
    }
    expanded
}

fn record_layout_commands(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &mut ComponentNodeMetaDatas,
    compute_resource_manager: &mut ComputeResourceManager,
    gpu: &wgpu::Device,
) {
    struct DrawModifierChainContent<'a> {
        draw_nodes: &'a [(Arc<dyn crate::modifier::DrawModifierNode>, DrawLayoutBounds)],
        node_draw: &'a dyn crate::modifier::DrawModifierNode,
        node_bounds: Option<DrawLayoutBounds>,
        parent_bounds: DrawLayoutBounds,
        pending_start: usize,
    }

    struct TerminalDrawContent;

    impl DrawModifierContent for TerminalDrawContent {
        fn draw(&mut self, _input: &mut RenderInput<'_>) {}
    }

    impl DrawModifierContent for DrawModifierChainContent<'_> {
        fn draw(&mut self, input: &mut RenderInput<'_>) {
            input.finish_draw_range(self.pending_start, self.parent_bounds);
            if let Some(((draw_modifier, bounds), remaining)) = self.draw_nodes.split_first() {
                input.draw_bounds = *bounds;
                let start = input.recorded_op_count();
                let mut draw_ctx = DrawModifierContext {
                    render_input: input,
                };
                let mut next = DrawModifierChainContent {
                    draw_nodes: remaining,
                    node_draw: self.node_draw,
                    node_bounds: self.node_bounds,
                    parent_bounds: *bounds,
                    pending_start: start,
                };
                draw_modifier.draw(&mut draw_ctx, &mut next);
                input.finish_draw_range(next.pending_start, *bounds);
            } else if let Some(bounds) = self.node_bounds {
                input.draw_bounds = bounds;
                let start = input.recorded_op_count();
                let mut draw_ctx = DrawModifierContext {
                    render_input: input,
                };
                self.node_draw.draw(&mut draw_ctx, &mut TerminalDrawContent);
                input.finish_draw_range(start, bounds);
            }
            input.draw_bounds = self.parent_bounds;
            self.pending_start = input.recorded_op_count();
        }
    }

    let pending = std::mem::take(&mut metadatas.paint_dirty_nodes);
    for node_id in pending {
        let Some(node) = tree.get(node_id).filter(|node| !node.is_removed()) else {
            continue;
        };
        let visible = node_id.ancestors(tree).all(|ancestor| {
            tree[ancestor].get().role != NodeRole::Layout
                || metadatas.get(&ancestor).is_some_and(|metadata| {
                    metadata.computed_data.is_some()
                        && (ancestor == root_node || metadata.rel_position.is_some())
                })
        });
        if !visible {
            continue;
        }
        let _phase = push_phase(RuntimePhase::Paint);
        let _instance = push_current_component_instance_key(node.get().instance_key);
        crate::runtime::clear_phase_state_reads(
            node.get().instance_key,
            crate::runtime::ReadPhase::Paint,
        );
        #[cfg(feature = "profiling")]
        let _record_profiler_guard = {
            let parent_id = node.parent();
            Some(ProfilerScopeGuard::new(
                ProfilerPhase::Record,
                Some(node_id),
                parent_id,
                Some(node.get().fn_name.as_str()),
            ))
        };
        // Recording also runs on layout cache hits, so visual state must be
        // reset independently of measurement invalidation.
        let previous_visual = metadatas.get(&node_id).map(|metadata| {
            (
                metadata.clips_children,
                metadata.clip_region,
                metadata.opacity,
            )
        });
        if let Some(metadata) = metadatas.get_mut(&node_id) {
            metadata.fragment = Default::default();
            metadata.clips_children = false;
            metadata.clip_region = None;
            metadata.opacity = 1.0;
        }
        let mut input = RenderInput::new(node_id, metadatas, compute_resource_manager, gpu);
        let node_ref = node.get();
        let mut bounds = Some(input.draw_bounds);
        let mut layer_index = 0;
        let mut draw_nodes = Vec::new();
        let layers = input.draw_layers();
        for action in node_ref.modifier.ordered_actions() {
            match action {
                OrderedModifierAction::Layout(_) => {
                    bounds = layers.get(layer_index).copied().flatten();
                    layer_index += 1;
                }
                OrderedModifierAction::Draw(node) => {
                    if let Some(bounds) = bounds {
                        draw_nodes.push((node, bounds));
                    }
                }
                _ => {}
            }
        }
        let mut content = DrawModifierChainContent {
            draw_nodes: &draw_nodes,
            node_draw: node_ref.draw_node.as_ref(),
            node_bounds: bounds,
            parent_bounds: input.draw_bounds,
            pending_start: 0,
        };
        content.draw(&mut input);
        metadatas.paint_changed.insert(node_id);
        if let Some(metadata) = metadatas.get_mut(&node_id) {
            metadata.paint_dirty = false;
        }
        let next_visual = metadatas.get(&node_id).map(|metadata| {
            (
                metadata.clips_children,
                metadata.clip_region,
                metadata.opacity,
            )
        });
        if previous_visual != next_visual {
            metadatas.mark_geometry_dirty(node_id);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PreparedLayoutMetadata {
    self_position: PxPosition,
    size: PxSize,
    clips_children: bool,
    clip_region: Option<crate::ClipRegion>,
    child_clip_rect: Option<PxRect>,
    cumulative_opacity: f32,
    has_shape_clipping: bool,
}

fn prepare_layout_metadata_for_node(
    tree: &ComponentNodeTree,
    metadatas: &mut ComponentNodeMetaDatas,
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    parent_clip_rect: Option<PxRect>,
    current_opacity: f32,
) -> Option<PreparedLayoutMetadata> {
    let _phase = push_phase(RuntimePhase::Place);
    let _instance = push_current_component_instance_key(tree.get(node_id)?.get().instance_key);
    if let Some(node) = tree.get(node_id) {
        crate::runtime::clear_phase_state_reads(
            node.get().instance_key,
            crate::runtime::ReadPhase::Transform,
        );
    }
    let parent_has_shape_clipping = tree
        .get(node_id)
        .and_then(|node| node.parent())
        .and_then(|parent| metadatas.get(&parent))
        .is_some_and(|metadata| metadata.has_shape_clipping);
    let metadata = metadatas.get_mut(&node_id)?;
    metadata.has_shape_clipping = parent_has_shape_clipping || metadata.clip_region.is_some();
    let rel_pos = match metadata.rel_position {
        Some(pos) => pos,
        None if is_root => PxPosition::ZERO,
        _ => {
            metadata.base_abs_position = None;
            metadata.abs_position = None;
            metadata.event_clip_rect = None;
            return None;
        }
    };
    let base_self_position = start_pos + rel_pos;
    let mut self_position = base_self_position;
    if let Some(node) = tree.get(node_id) {
        for action in node.get().modifier.ordered_actions() {
            if let OrderedModifierAction::Placement(placement_node) = action {
                self_position = placement_node.node().transform_position(self_position);
            }
        }
    }
    let cumulative_opacity = current_opacity * metadata.opacity;
    metadata.base_abs_position = Some(base_self_position);
    metadata.abs_position = Some(self_position);
    metadata.event_clip_rect = parent_clip_rect;

    let size = metadata
        .computed_data
        .map(|d| PxSize {
            width: d.width,
            height: d.height,
        })
        .unwrap_or_default();
    let node_rect = PxRect {
        x: self_position.x_px(),
        y: self_position.y_px(),
        width: size.width,
        height: size.height,
    };
    let clips_children = metadata.clips_children;
    let clip_region = metadata.clip_region.map(|mut region| {
        region.position = region.position + self_position;
        region
    });
    let node_rect = clip_region
        .map(|region| region.bounds())
        .unwrap_or(node_rect);
    let child_clip_rect = if clips_children {
        Some(
            parent_clip_rect
                .map(|existing| existing.intersection(&node_rect).unwrap_or(PxRect::ZERO))
                .unwrap_or(node_rect),
        )
    } else {
        parent_clip_rect
    };

    Some(PreparedLayoutMetadata {
        self_position,
        size,
        clips_children,
        clip_region,
        child_clip_rect,
        cumulative_opacity,
        has_shape_clipping: metadata.has_shape_clipping,
    })
}

fn populate_layout_metadata(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &mut ComponentNodeMetaDatas,
) -> u64 {
    let mut dirty = std::mem::take(&mut metadatas.geometry_dirty);
    let mut dirty_children: HashMap<NodeId, HashSet<NodeId>> = HashMap::default();
    for mut id in dirty.iter().copied().collect::<Vec<_>>() {
        while let Some(parent) = tree.get(id).and_then(|node| node.parent()) {
            dirty_children.entry(parent).or_default().insert(id);
            if !dirty.insert(parent) {
                break;
            }
            id = parent;
        }
    }
    fn visit(
        tree: &ComponentNodeTree,
        metadatas: &mut ComponentNodeMetaDatas,
        dirty: &HashSet<indextree::NodeId>,
        dirty_children: &HashMap<NodeId, HashSet<NodeId>>,
        start_pos: PxPosition,
        is_root: bool,
        node_id: indextree::NodeId,
        clip_rect: Option<PxRect>,
        current_opacity: f32,
        inherited_changed: bool,
        updates: &mut u64,
    ) {
        metadatas.geometry_visits += 1;
        let previous = metadatas
            .get(&node_id)
            .and_then(|metadata| metadata.prepared_layout);
        let initialized = metadatas
            .get(&node_id)
            .is_some_and(|metadata| metadata.geometry_initialized);
        if initialized && !inherited_changed && !dirty.contains(&node_id) {
            return;
        }
        *updates += 1;
        let prepared = prepare_layout_metadata_for_node(
            tree,
            metadatas,
            start_pos,
            is_root,
            node_id,
            clip_rect,
            current_opacity,
        );
        if let Some(metadata) = metadatas.get_mut(&node_id) {
            metadata.prepared_layout = prepared;
            metadata.geometry_initialized = true;
        }
        let order_changed = metadatas.placement_order_dirty.remove(&node_id);
        let changed = inherited_changed || !initialized || prepared != previous || order_changed;
        if changed {
            metadatas.geometry_changed.insert(node_id);
        }
        if prepared.is_some()
            && metadatas
                .get(&node_id)
                .is_some_and(|metadata| metadata.paint_dirty)
        {
            metadatas.paint_dirty_nodes.insert(node_id);
        }
        let children: Vec<_> = if changed {
            node_id.children(tree).collect()
        } else {
            dirty_children
                .get(&node_id)
                .into_iter()
                .flatten()
                .copied()
                .collect()
        };
        let Some(prepared) = prepared else {
            if tree
                .get(node_id)
                .is_some_and(|node| node.get().role == NodeRole::Layout)
            {
                for descendant in node_id.descendants(tree) {
                    if let Some(metadata) = metadatas.get_mut(&descendant) {
                        metadata.base_abs_position = None;
                        metadata.abs_position = None;
                        metadata.event_clip_rect = None;
                    }
                    metadatas.geometry_changed.insert(descendant);
                }
                return;
            }
            for child in children {
                visit(
                    tree,
                    metadatas,
                    dirty,
                    dirty_children,
                    start_pos,
                    false,
                    child,
                    clip_rect,
                    current_opacity,
                    changed,
                    updates,
                );
            }
            return;
        };
        for child in children {
            visit(
                tree,
                metadatas,
                dirty,
                dirty_children,
                prepared.self_position,
                false,
                child,
                prepared.child_clip_rect,
                prepared.cumulative_opacity,
                changed,
                updates,
            );
        }
    }
    let mut updates = 0;
    visit(
        tree,
        metadatas,
        &dirty,
        &dirty_children,
        PxPosition::ZERO,
        true,
        root_node,
        None,
        1.0,
        false,
        &mut updates,
    );
    updates
}

#[cfg(test)]
fn build_render_graph(
    root: NodeId,
    tree: &ComponentNodeTree,
    metadata: &mut ComponentNodeMetaDatas,
    viewport: PxSize,
) -> RenderGraph {
    let changed = root.descendants(tree).collect();
    scene::FrameScene::default()
        .update(
            root,
            tree,
            metadata,
            &changed,
            &HashSet::default(),
            viewport,
        )
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        Px,
        component_tree::{ComponentNode, NodeRole},
        layout::{DefaultLayoutNode, NoopDrawModifier},
        modifier::Modifier,
    };

    fn node_with_role(
        name: &str,
        role: NodeRole,
        instance_logic_id: u64,
        instance_key: u64,
    ) -> ComponentNode {
        ComponentNode {
            fn_name: name.to_string(),
            role,
            layout_children: None,
            instance_logic_id,
            instance_key,
            pointer_preview_handlers: Vec::new(),
            pointer_handlers: Vec::new(),
            pointer_final_handlers: Vec::new(),
            keyboard_preview_handlers: Vec::new(),
            keyboard_handlers: Vec::new(),
            ime_preview_handlers: Vec::new(),
            ime_handlers: Vec::new(),
            focus_requester_binding: None,
            focus_registration: None,
            focus_restorer_fallback: None,
            focus_traversal_policy: None,
            focus_changed_handler: None,
            focus_event_handler: None,
            focus_beyond_bounds_handler: None,
            focus_reveal_handler: None,
            globally_positioned_handlers: Vec::new(),
            modifier: Modifier::default(),
            layout_node: Box::new(DefaultLayoutNode),
            draw_node: Box::new(NoopDrawModifier),
            replay: None,
            props_unchanged_from_previous: false,
        }
    }

    fn node(name: &str, instance_logic_id: u64, instance_key: u64) -> ComponentNode {
        node_with_role(name, NodeRole::Layout, instance_logic_id, instance_key)
    }

    #[test]
    fn unplaced_layout_subtrees_hide_geometry_input_drawing_semantics_and_focus() {
        use crate::focus::{FocusNode, FocusRegistration};

        #[derive(Clone)]
        struct Marker;

        impl crate::DrawCommand for Marker {
            fn apply_opacity(&mut self, _opacity: f32) {}
        }

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        tree.add_node(node_with_role("transparent", NodeRole::Composition, 2, 2));
        let panel = tree.add_node(node("panel", 3, 3));
        tree.add_node(node_with_role("content", NodeRole::Composition, 4, 4));
        let leaf_focus = FocusRegistration::target(FocusNode::new());
        let mut leaf_node = node("leaf", 5, 5);
        leaf_node.focus_registration = Some(leaf_focus);
        let leaf = tree.add_node(leaf_node);
        for _ in 0..4 {
            tree.pop_node();
        }
        let outside_focus = FocusRegistration::target(FocusNode::new());
        let mut outside_node = node("outside", 6, 6);
        outside_node.focus_registration = Some(outside_focus);
        let outside = tree.add_node(outside_node);
        tree.pop_node();
        tree.pop_node();
        for (id, length) in [(root, 200), (panel, 30), (leaf, 6), (outside, 6)] {
            tree.metadatas.entry_or_default(id).computed_data = Some(ComputedData {
                width: Px(length),
                height: Px(length),
            });
        }
        tree.metadatas.entry_or_default(leaf).rel_position = Some(PxPosition::new(Px(4), Px(5)));
        tree.metadatas.entry_or_default(outside).rel_position =
            Some(PxPosition::new(Px(240), Px(240)));
        tree.metadatas.entry_or_default(root).clips_children = true;
        tree.metadatas.entry_or_default(leaf).accessibility = Some(
            crate::accessibility::AccessibilityNode::new()
                .with_role(accesskit::Role::Button)
                .with_label("Panel action"),
        );
        for position in [Some(20), None, Some(40)] {
            tree.metadatas.mark_geometry_dirty(panel);
            tree.metadatas.entry_or_default(panel).rel_position =
                position.map(|position| PxPosition::new(Px(position), Px(position)));
            let metadata = tree.metadatas.entry_or_default(leaf);
            metadata.fragment = Default::default();
            metadata.fragment.push_draw_command(Marker);
            populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
            assert_eq!(
                tree.metadatas.get(&outside).unwrap().event_clip_rect,
                Some(PxRect::new(Px::ZERO, Px::ZERO, Px(200), Px(200))),
                "the placed traversal target is outside its ancestor's clip"
            );
            tree.focus_owner.sync_from_component_tree(root, &tree.tree);
            tree.focus_owner
                .sync_layout_from_component_tree(root, &tree.tree, &tree.metadatas);
            if position.is_none() {
                assert_eq!(
                    tree.focus_owner.active_component_node_id(),
                    None,
                    "hiding the active subtree releases focus, including capture"
                );
            }
            tree.focus_owner.clear_focus_global(true);
            tree.focus_owner.request_focus(leaf_focus.id);
            tree.focus_owner.commit_pending();
            assert_eq!(
                tree.focus_owner.active_component_node_id(),
                position.map(|_| leaf),
                "unplaced descendants reject direct focus requests"
            );
            tree.focus_owner.clear_focus_global(true);
            assert!(tree.focus_owner.move_focus(FocusDirection::Next));
            assert_eq!(
                tree.focus_owner.active_component_node_id(),
                Some(if position.is_some() { leaf } else { outside }),
                "forward traversal skips the unplaced subtree"
            );
            if position.is_some() {
                assert!(tree.focus_owner.move_focus(FocusDirection::Next));
            }
            assert_eq!(
                tree.focus_owner.move_focus(FocusDirection::Previous),
                position.is_some(),
                "reverse traversal also skips the unplaced subtree"
            );
            assert_eq!(
                tree.focus_owner.active_component_node_id(),
                Some(if position.is_some() { leaf } else { outside })
            );
            if position.is_some() {
                tree.focus_owner.capture_focus(leaf_focus.id);
            }
            let graph = build_render_graph(
                root,
                &tree.tree,
                &mut tree.metadatas,
                PxSize::new(Px(200), Px(200)),
            );
            let expected =
                position.map(|position| PxPosition::new(Px(position + 4), Px(position + 5)));
            assert_eq!(tree.metadatas.get(&leaf).unwrap().abs_position, expected);
            assert_eq!(
                tree.metadatas.get(&leaf).unwrap().base_abs_position,
                expected
            );
            assert_eq!(
                resolve_node_input_context(&tree.tree, &tree.metadatas, leaf).is_some(),
                expected.is_some()
            );
            let marker_ops: Vec<_> = graph
                .ops()
                .iter()
                .filter(|op| op.type_id == std::any::TypeId::of::<Marker>())
                .collect();
            assert_eq!(marker_ops.len(), usize::from(expected.is_some()));
            let accessibility = crate::accessibility::build_tree_update(
                &tree.tree,
                &tree.metadatas,
                root,
                None,
                Some(leaf),
            )
            .expect("accessibility tree");
            assert_eq!(
                accessibility
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Panel action")),
                expected.is_some()
            );
            assert_eq!(
                accessibility
                    .nodes
                    .iter()
                    .find(|(id, _)| *id == accessibility.focus)
                    .and_then(|(_, node)| node.label()),
                expected.map(|_| "Panel action")
            );
            if let Some(expected) = expected {
                assert_eq!(marker_ops[0].position, expected);
            }
        }
    }

    #[test]
    fn pointer_samples_route_at_their_own_positions_and_complete_passes_in_order() {
        use crate::cursor::{GestureState, MOUSE_POINTER_ID, PressKeyEventType};

        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let origin = Instant::now();
        for index in 0..2 {
            let mut target = node("target", index + 2, index + 2);
            for pass in [
                PointerEventPass::Initial,
                PointerEventPass::Main,
                PointerEventPass::Final,
            ] {
                let log = Arc::clone(&log);
                let handler: Box<PointerInputHandlerFn> = Box::new(move |input| {
                    for change in input.pointer_changes.iter() {
                        log.lock().unwrap().push((
                            index,
                            pass,
                            change.timestamp.duration_since(origin).as_nanos(),
                            input.cursor_position_rel,
                        ));
                    }
                });
                match pass {
                    PointerEventPass::Initial => target.pointer_preview_handlers.push(handler),
                    PointerEventPass::Main => target.pointer_handlers.push(handler),
                    PointerEventPass::Final => target.pointer_final_handlers.push(handler),
                }
            }
            let id = tree.add_node(target);
            let metadata = tree.metadatas.get_mut(&id).unwrap();
            let position = PxPosition::new(Px(index as i32 * 200), Px::ZERO);
            metadata.base_abs_position = Some(position);
            metadata.abs_position = Some(position);
            metadata.computed_data = Some(ComputedData {
                width: Px(100),
                height: Px(100),
            });
            tree.pop_node();
        }
        tree.pop_node();
        let metadata = tree.metadatas.get_mut(&root).unwrap();
        metadata.base_abs_position = Some(PxPosition::ZERO);
        metadata.abs_position = Some(PxPosition::ZERO);
        metadata.computed_data = Some(ComputedData {
            width: Px(400),
            height: Px(100),
        });
        let left = PxPosition::new(Px(20), Px(20));
        let right = PxPosition::new(Px(220), Px(20));
        let outside = PxPosition::new(Px(500), Px(20));
        let mut events: Vec<_> = [
            (
                CursorEventContent::Pressed(PressKeyEventType::Left),
                Some(left),
            ),
            (
                CursorEventContent::Released(PressKeyEventType::Left),
                Some(left),
            ),
            (CursorEventContent::Moved(outside), Some(outside)),
            (
                CursorEventContent::Pressed(PressKeyEventType::Left),
                Some(right),
            ),
            (
                CursorEventContent::Released(PressKeyEventType::Left),
                Some(right),
            ),
            (CursorEventContent::Moved(outside), Some(outside)),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (content, position))| {
            PointerChange::new(
                origin + std::time::Duration::from_nanos(index as u64),
                MOUSE_POINTER_ID,
                content,
                GestureState::TapCandidate,
            )
            .with_position(position)
        })
        .collect();
        tree.dispatch_pointer_input(
            root,
            &mut events,
            &mut Some(outside),
            winit::keyboard::ModifiersState::empty(),
        );
        let expected: Vec<_> = [(0, 0), (0, 1), (1, 3), (1, 4)]
            .into_iter()
            .flat_map(|(index, sample)| {
                [
                    PointerEventPass::Initial,
                    PointerEventPass::Main,
                    PointerEventPass::Final,
                ]
                .map(|pass| (index, pass, sample, Some(left)))
            })
            .collect();
        assert_eq!(*log.lock().unwrap(), expected);
        assert!(tree.active_pointer_paths.is_empty());
        log.lock().unwrap().clear();
        let mut captured: Vec<_> = [
            (
                CursorEventContent::Pressed(PressKeyEventType::Left),
                Some(left),
            ),
            (CursorEventContent::Moved(outside), Some(outside)),
            (CursorEventContent::Released(PressKeyEventType::Left), None),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (content, position))| {
            PointerChange::new(
                origin + std::time::Duration::from_nanos(index as u64),
                MOUSE_POINTER_ID,
                content,
                GestureState::TapCandidate,
            )
            .with_position(position)
        })
        .collect();
        tree.dispatch_pointer_input(
            root,
            &mut captured,
            &mut Some(right),
            winit::keyboard::ModifiersState::empty(),
        );
        let expected: Vec<_> = [Some(left), Some(outside), None]
            .into_iter()
            .enumerate()
            .flat_map(|(sample, position)| {
                [
                    PointerEventPass::Initial,
                    PointerEventPass::Main,
                    PointerEventPass::Final,
                ]
                .map(|pass| (0, pass, sample as u128, position))
            })
            .collect();
        assert_eq!(
            *log.lock().unwrap(),
            expected,
            "capture delivers outside movement and an unavailable-position release to the original target"
        );
        assert!(tree.active_pointer_paths.is_empty());

        let mut press = [PointerChange::new(
            origin,
            MOUSE_POINTER_ID,
            CursorEventContent::Pressed(PressKeyEventType::Left),
            GestureState::TapCandidate,
        )
        .with_position(Some(left))];
        tree.dispatch_pointer_input(
            root,
            &mut press,
            &mut Some(left),
            winit::keyboard::ModifiersState::empty(),
        );
        let target = tree.find_node_id_by_instance_key(2).unwrap();
        tree.metadatas.get_mut(&target).unwrap().event_clip_rect = Some(PxRect::ZERO);
        tree.metadatas.geometry_changed.insert(target);
        log.lock().unwrap().clear();
        let mut release = [PointerChange::new(
            origin + std::time::Duration::from_nanos(1),
            MOUSE_POINTER_ID,
            CursorEventContent::Released(PressKeyEventType::Left),
            GestureState::TapCandidate,
        )
        .with_position(Some(left))];
        tree.dispatch_pointer_input(
            root,
            &mut release,
            &mut Some(left),
            winit::keyboard::ModifiersState::empty(),
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                PointerEventPass::Initial,
                PointerEventPass::Main,
                PointerEventPass::Final
            ]
            .map(|pass| (0, pass, 1, None)),
            "a captured target scrolled outside its clip must still receive release"
        );
        assert!(tree.active_pointer_paths.is_empty());
    }

    #[test]
    fn recomposition_preserves_pointer_capture_but_runtime_reset_discards_it() {
        let mut tree = ComponentTree::new();
        tree.active_pointer_paths.insert(7, vec![1, 2]);
        tree.active_pointer_positions
            .insert(7, Some(PxPosition::ZERO));
        tree.clear();
        assert_eq!(tree.active_pointer_paths.get(&7), Some(&vec![1, 2]));
        assert_eq!(
            tree.active_pointer_positions.get(&7),
            Some(&Some(PxPosition::ZERO))
        );
        tree.reset();
        assert!(tree.active_pointer_paths.is_empty());
        assert!(tree.active_pointer_positions.is_empty());
    }

    fn focus_targets(attach: bool) -> (ComponentTree, Vec<(NodeId, FocusHandleId)>) {
        use crate::focus::{FocusNode, FocusRegistration};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let mut targets = Vec::new();
        for index in 0..2 {
            let registration = FocusRegistration::target(FocusNode::new());
            let mut target = node("target", index + 2, index + 2);
            target.focus_registration = Some(registration);
            let node_id = tree.add_node(target);
            targets.push((node_id, registration.id));
            tree.pop_node();
        }
        tree.pop_node();
        if attach {
            tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        }
        (tree, targets)
    }

    #[test]
    fn repeated_focus_request_does_not_replay_after_action_restores_another_target() {
        let (mut tree, targets) = focus_targets(true);
        let [(action, action_handle), (anchor, anchor_handle)] = targets.as_slice() else {
            unreachable!();
        };
        tree.focus_owner.request_focus(*action_handle);
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(*action));
        tree.focus_owner.request_focus(*action_handle);
        tree.focus_owner.request_focus(*anchor_handle);
        tree.focus_owner.commit_pending();
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(*anchor));
        tree.focus_owner.commit_pending();
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(*anchor));
    }

    #[test]
    fn clearing_an_unfocused_attached_target_does_not_clear_its_future_focus() {
        let (mut tree, targets) = focus_targets(true);
        tree.focus_owner.request_focus(targets[1].1);
        tree.focus_owner.clear_focus(targets[0].1);
        tree.focus_owner.request_focus(targets[0].1);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[0].0)
        );
    }

    #[test]
    fn rejected_capture_does_not_capture_a_later_focus_request() {
        let (mut tree, targets) = focus_targets(true);
        tree.focus_owner.request_focus(targets[1].1);
        tree.focus_owner.capture_focus(targets[0].1);
        tree.focus_owner.request_focus(targets[0].1);
        tree.focus_owner.commit_pending();
        tree.focus_owner.request_focus(targets[1].1);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[1].0)
        );
    }

    #[test]
    fn redundant_free_does_not_release_a_later_capture() {
        let (mut tree, targets) = focus_targets(true);
        tree.focus_owner.request_focus(targets[0].1);
        tree.focus_owner.free_focus(targets[0].1);
        tree.focus_owner.capture_focus(targets[0].1);
        tree.focus_owner.commit_pending();
        tree.focus_owner.request_focus(targets[1].1);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[0].0)
        );
    }

    #[test]
    fn requests_before_attachment_are_consumed_once_the_target_is_attached() {
        let (mut tree, targets) = focus_targets(false);
        tree.focus_owner.request_focus(targets[0].1);
        tree.focus_owner.request_focus(targets[0].1);
        let root = tree.tree.get(targets[0].0).unwrap().parent().unwrap();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[0].0)
        );
        tree.focus_owner.request_focus(targets[1].1);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[1].0)
        );
    }

    #[test]
    fn deferred_request_rejected_after_attachment_does_not_activate_on_enable() {
        let (mut tree, targets) = focus_targets(false);
        tree.focus_owner.request_focus(targets[0].1);
        let root = tree.tree.get(targets[0].0).unwrap().parent().unwrap();
        tree.tree
            .get_mut(targets[0].0)
            .unwrap()
            .get_mut()
            .focus_registration
            .as_mut()
            .unwrap()
            .properties
            .can_focus = false;
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.commit_pending();
        assert_eq!(tree.focus_owner.active_component_node_id(), None);
        tree.tree
            .get_mut(targets[0].0)
            .unwrap()
            .get_mut()
            .focus_registration
            .as_mut()
            .unwrap()
            .properties
            .can_focus = true;
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.commit_pending();
        assert_eq!(tree.focus_owner.active_component_node_id(), None);
    }

    #[test]
    fn queued_navigation_routes_activation_to_the_new_focus_chain() {
        use crate::focus::{FocusNode, FocusRegistration};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let mut trigger = node("trigger", 2, 2);
        trigger.focus_registration = Some(FocusRegistration::target(FocusNode::new()));
        let trigger = tree.add_node(trigger);
        tree.pop_node();
        let popup = tree.add_node(node("popup", 3, 3));
        let mut action = node("action", 4, 4);
        action.focus_registration = Some(FocusRegistration::target(FocusNode::new()));
        let action = tree.add_node(action);
        tree.pop_node();
        tree.pop_node();
        tree.pop_node();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        assert!(tree.focus_owner.move_focus(FocusDirection::Next));
        let mut events = VecDeque::from(["Tab", "Enter", "Shift+Tab", "Enter"]);
        let mut activations = Vec::new();
        let retry = drain_pending_keyboard_events(&mut events, |event| {
            match event {
                "Tab" => assert!(tree.focus_owner.move_focus(FocusDirection::Next)),
                "Shift+Tab" => assert!(tree.focus_owner.move_focus(FocusDirection::Previous)),
                "Enter" => activations.push(collect_focus_chain_node_ids(
                    root,
                    &tree.tree,
                    &tree.focus_owner,
                )),
                _ => unreachable!(),
            }
            tree.focus_owner.commit_pending();
            (None, false)
        });
        assert_eq!(retry, None);
        assert!(events.is_empty());
        assert_eq!(
            activations,
            [vec![root, popup, action], vec![root, trigger]]
        );
    }

    #[test]
    fn skipped_focus_targets_anchor_navigation_without_becoming_destinations() {
        use crate::focus::{FocusNode, FocusRegistration, FocusScopeNode};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let scope = FocusRegistration::scope(FocusScopeNode::new());
        let mut container = node("scope", 2, 2);
        container.focus_registration = Some(scope);
        tree.add_node(container);
        let mut targets = Vec::new();
        for index in 0..3 {
            let mut registration = FocusRegistration::target(FocusNode::new());
            registration.properties.skip_traversal = index == 1;
            let mut target = node("target", index + 3, index + 3);
            target.focus_registration = Some(registration);
            targets.push((tree.add_node(target), registration.id));
            tree.pop_node();
        }
        tree.pop_node();
        tree.pop_node();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        for scoped in [false, true] {
            for (direction, expected) in [(FocusDirection::Next, 2), (FocusDirection::Previous, 0)]
            {
                tree.focus_owner.request_focus(targets[1].1);
                assert_eq!(
                    tree.focus_owner.active_component_node_id(),
                    Some(targets[1].0)
                );
                assert!(if scoped {
                    tree.focus_owner
                        .move_focus_in_scope(scope.id, direction, true)
                } else {
                    tree.focus_owner.move_focus(direction)
                });
                assert_eq!(
                    tree.focus_owner.active_component_node_id(),
                    Some(targets[expected].0)
                );
            }
        }
        tree.focus_owner.request_focus(targets[0].1);
        assert!(tree.focus_owner.move_focus(FocusDirection::Next));
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[2].0)
        );
        for index in [0, 2] {
            tree.tree[targets[index].0]
                .get_mut()
                .focus_registration
                .as_mut()
                .unwrap()
                .properties
                .can_focus = false;
        }
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.request_focus(targets[1].1);
        for direction in [FocusDirection::Next, FocusDirection::Previous] {
            assert!(
                !tree
                    .focus_owner
                    .move_focus_in_scope(scope.id, direction, true)
            );
        }
    }

    #[test]
    fn removing_unscoped_focus_does_not_activate_an_unrelated_control() {
        let (mut tree, targets) = focus_targets(true);
        tree.focus_owner.request_focus(targets[0].1);
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(targets[0].0)
        );
        tree.focus_owner
            .remove_handles(&[targets[0].1].into_iter().collect(), &HashSet::default());
        assert_eq!(tree.focus_owner.active_component_node_id(), None);
    }

    #[test]
    fn removing_focused_handles_restores_to_a_surviving_scope_target() {
        use crate::focus::{FocusNode, FocusRegistration, FocusRequester, FocusScopeNode};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let scope = FocusRegistration::scope(FocusScopeNode::new());
        let fallback = FocusRequester::new();
        let mut container = node("scope", 2, 2);
        container.focus_registration = Some(scope);
        container.focus_restorer_fallback = Some(fallback);
        tree.add_node(container);
        let removed = FocusRegistration::target(FocusNode::new());
        let mut target = node("removed", 3, 3);
        target.focus_registration = Some(removed);
        let removed_node = tree.add_node(target);
        tree.pop_node();
        let mut target = node("survivor", 4, 4);
        target.focus_registration = Some(FocusRegistration::target(FocusNode::new()));
        target.focus_requester_binding = Some(fallback);
        let survivor = tree.add_node(target);
        tree.pop_node();
        tree.pop_node();
        tree.pop_node();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.request_focus(removed.id);
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(removed_node)
        );
        tree.focus_owner
            .remove_handles(&[removed.id].into_iter().collect(), &HashSet::default());
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(survivor));
    }

    #[test]
    fn detached_focus_scopes_retain_their_target_through_layout_until_reattached() {
        use crate::focus::{FocusNode, FocusRegistration, FocusScopeNode};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let mut container = node_with_role("scope", NodeRole::Composition, 2, 2);
        container.focus_registration = Some(FocusRegistration::scope(FocusScopeNode::new()));
        let scope = tree.add_node(container);
        let registration = FocusRegistration::target(FocusNode::new());
        let mut target = node("target", 3, 3);
        target.focus_registration = Some(registration);
        let target = tree.add_node(target);
        for _ in 0..3 {
            tree.pop_node();
        }
        for id in [root, target] {
            let metadata = tree.metadatas.entry_or_default(id);
            metadata.computed_data = Some(ComputedData {
                width: Px(20),
                height: Px(20),
            });
            metadata.rel_position = Some(PxPosition::ZERO);
        }
        populate_layout_metadata(root, &tree.tree, &mut tree.metadatas);
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner
            .sync_layout_from_component_tree(root, &tree.tree, &tree.metadatas);
        tree.focus_owner.request_focus(registration.id);
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(target));

        scope.detach(&mut tree.tree);
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner
            .sync_layout_from_component_tree(root, &tree.tree, &tree.metadatas);
        assert_eq!(tree.focus_owner.active_component_node_id(), None);
        root.append(scope, &mut tree.tree);
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner
            .sync_layout_from_component_tree(root, &tree.tree, &tree.metadatas);
        assert_eq!(tree.focus_owner.active_component_node_id(), Some(target));
    }

    #[test]
    fn redundant_scope_restore_does_not_pull_focus_back_from_another_scope() {
        use crate::focus::{FocusNode, FocusRegistration, FocusScopeNode};

        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let scope = FocusRegistration::scope(FocusScopeNode::new());
        let mut container = node("scope", 2, 2);
        container.focus_registration = Some(scope);
        tree.add_node(container);
        let inside = FocusRegistration::target(FocusNode::new());
        let mut target = node("inside", 3, 3);
        target.focus_registration = Some(inside);
        tree.add_node(target);
        tree.pop_node();
        tree.pop_node();
        let outside = FocusRegistration::target(FocusNode::new());
        let mut target = node("outside", 4, 4);
        target.focus_registration = Some(outside);
        let outside_node = tree.add_node(target);
        tree.pop_node();
        tree.pop_node();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        tree.focus_owner.request_focus(inside.id);
        tree.focus_owner.restore_focus(scope.id);
        tree.focus_owner.request_focus(outside.id);
        tree.focus_owner.commit_pending();
        assert_eq!(
            tree.focus_owner.active_component_node_id(),
            Some(outside_node)
        );
    }

    #[test]
    fn focus_retry_keeps_later_keys_before_newly_received_input() {
        let mut events = VecDeque::from(["Tab into lazy content", "Enter", "Shift+Tab"]);
        let mut dispatched = Vec::new();
        let retry = drain_pending_keyboard_events(&mut events, |event| {
            dispatched.push(event);
            (Some(FocusDirection::Next), false)
        });
        assert_eq!(retry, Some(FocusDirection::Next));
        assert_eq!(dispatched, ["Tab into lazy content"]);
        events.push_back("new input");
        assert_eq!(
            drain_pending_keyboard_events(&mut events, |event| {
                dispatched.push(event);
                (None, false)
            }),
            None
        );
        assert_eq!(
            dispatched,
            ["Tab into lazy content", "Enter", "Shift+Tab", "new input"]
        );
        assert!(events.is_empty());
    }

    #[test]
    fn keyboard_barrier_preserves_order_until_a_subsequent_layout_finishes() {
        use crate::focus::{FocusManager, bind_focus_owner};

        let mut owner = FocusOwner::new();
        let mut events = VecDeque::from(["close popup", "Enter", "Tab"]);
        let mut delivered = Vec::new();
        assert_eq!(
            drain_pending_keyboard_events(&mut events, |event| {
                delivered.push(event);
                {
                    let _binding = bind_focus_owner(&mut owner);
                    FocusManager::current().defer_keyboard_until_layout();
                }
                (None, owner.keyboard_is_deferred())
            }),
            None
        );
        assert_eq!(delivered, ["close popup"]);
        assert_eq!(events, ["Enter", "Tab"]);
        owner.finish_layout();
        assert!(
            owner.keyboard_is_deferred(),
            "the old layout cannot release the barrier"
        );
        owner.begin_layout();
        assert!(
            owner.keyboard_is_deferred(),
            "focus callbacks still have to finish"
        );
        {
            let _binding = bind_focus_owner(&mut owner);
            FocusManager::current().defer_keyboard_until_layout();
        }
        owner.finish_layout();
        assert!(
            owner.keyboard_is_deferred(),
            "a new transition survives the old layout callback"
        );
        owner.begin_layout();
        owner.finish_layout();
        assert!(!owner.keyboard_is_deferred());
        events.push_back("new input");
        drain_pending_keyboard_events(&mut events, |event| {
            delivered.push(event);
            (None, owner.keyboard_is_deferred())
        });
        assert_eq!(delivered, ["close popup", "Enter", "Tab", "new input"]);
        assert!(events.is_empty());
    }

    #[test]
    fn recomposition_retains_deferred_input_but_runtime_reset_discards_it() {
        use crate::focus::{FocusManager, bind_focus_owner};

        let mut tree = ComponentTree::new();
        {
            let _binding = bind_focus_owner(&mut tree.focus_owner);
            FocusManager::current().defer_keyboard_until_layout();
            assert!(FocusManager::current().request_move_focus(FocusDirection::Previous));
            assert!(FocusManager::current().request_move_focus(FocusDirection::Next));
        }
        assert_eq!(
            tree.focus_owner.take_pending_navigation(),
            Some(FocusDirection::Next)
        );
        tree.focus_owner
            .retain_navigation_retry(Some(FocusDirection::Next));
        tree.pending_ime_events
            .push(winit::event::Ime::Commit("pending text".into()));
        clipboard::request_clipboard_action(
            crate::clipboard::ClipboardAction::Paste("pending paste".into()),
            |_| panic!("reset clipboard request must not execute"),
        );
        tree.pending_keyboard_events.extend(
            clipboard::take_requests()
                .into_iter()
                .map(QueuedFocusedInput::Clipboard),
        );
        tree.clear();
        assert_eq!(tree.pending_keyboard_events.len(), 1);
        assert!(tree.focus_owner.has_pending_navigation());
        assert!(tree.focus_owner.keyboard_is_deferred());
        assert!(matches!(
            tree.pending_ime_events.as_slice(),
            [winit::event::Ime::Commit(text)] if text == "pending text"
        ));
        tree.focus_owner.set_owner_focused(false);
        assert!(!tree.focus_owner.has_pending_navigation());
        {
            let _binding = bind_focus_owner(&mut tree.focus_owner);
            assert!(!FocusManager::current().request_move_focus(FocusDirection::Previous));
        }
        tree.focus_owner.set_owner_focused(true);
        {
            let _binding = bind_focus_owner(&mut tree.focus_owner);
            assert!(FocusManager::current().request_move_focus(FocusDirection::Previous));
        }
        tree.reset();
        assert!(!tree.focus_owner.has_pending_navigation());
        assert!(!tree.focus_owner.keyboard_is_deferred());
        assert!(tree.pending_ime_events.is_empty());
        assert!(tree.pending_keyboard_events.is_empty());
    }

    #[test]
    fn ime_commit_is_delivered_once_to_the_final_focused_control() {
        use crate::focus::{FocusNode, FocusRegistration};

        let delivered = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        for index in 0..2 {
            let mut input = node("input", index + 2, index + 2);
            input.focus_registration = Some(FocusRegistration::target(FocusNode::new()));
            let delivered = Arc::clone(&delivered);
            input.ime_handlers.push(Box::new(move |input| {
                for event in input.ime_events.drain(..) {
                    if let winit::event::Ime::Commit(text) = event {
                        delivered.lock().unwrap().push((index, text));
                    }
                }
            }));
            let id = tree.add_node(input);
            let metadata = tree.metadatas.get_mut(&id).unwrap();
            metadata.base_abs_position = Some(PxPosition::ZERO);
            metadata.abs_position = Some(PxPosition::ZERO);
            metadata.computed_data = Some(ComputedData {
                width: Px(100),
                height: Px(40),
            });
            tree.pop_node();
        }
        tree.pop_node();
        tree.focus_owner.sync_from_component_tree(root, &tree.tree);
        assert!(tree.focus_owner.move_focus(FocusDirection::Next));
        assert!(tree.focus_owner.move_focus(FocusDirection::Next));
        let mut events = vec![winit::event::Ime::Commit("composed text".into())];
        let mut requests = WindowRequests::default();
        tree.dispatch_ime_input(root, &mut events, &mut requests);
        tree.dispatch_ime_input(root, &mut events, &mut requests);
        assert_eq!(*delivered.lock().unwrap(), [(1, "composed text".into())]);
    }

    #[test]
    fn begin_replace_subtree_rejects_root_instance_key() {
        let mut tree = ComponentTree::new();

        let root = tree.add_node(node("root", 1, 1));
        tree.pop_node();

        let result = tree.begin_replace_subtree_by_instance_key(1);
        assert!(matches!(
            result,
            Err(ReplayReplaceError::RootNodeNotReplaceable)
        ));
        assert!(tree.get(root).is_some());
    }

    #[test]
    fn finish_replace_subtree_keeps_inserted_roots_before_next_sibling() {
        let mut tree = ComponentTree::new();

        let root = tree.add_node(node("root", 1, 1));

        let _first = tree.add_node(node("first", 2, 2));
        let _first_child = tree.add_node(node("first_child", 3, 3));
        tree.pop_node();
        tree.pop_node();

        let second = tree.add_node(node("second", 4, 4));
        tree.pop_node();
        tree.pop_node();

        let context = match tree.begin_replace_subtree_by_instance_key(2) {
            Ok(context) => context,
            Err(_) => panic!("replace context should be created"),
        };

        let new_a = tree.add_node(node("new_a", 5, 5));
        let _new_a_child = tree.add_node(node("new_a_child", 6, 6));
        tree.pop_node();
        tree.pop_node();
        let new_b = tree.add_node(node("new_b", 7, 7));
        tree.pop_node();

        let before_finish = root.children(tree.tree()).collect::<Vec<_>>();
        assert_eq!(before_finish, vec![second, new_a, new_b]);

        let replace_result = match tree.finish_replace_subtree(context) {
            Ok(result) => result,
            Err(_) => panic!("finish replace should succeed"),
        };

        let root_children = root
            .children(tree.tree())
            .map(|id| tree.get(id).expect("child must exist").fn_name.clone())
            .collect::<Vec<_>>();
        assert_eq!(root_children, vec!["new_a", "new_b", "second"]);

        assert!(replace_result.removed_instance_keys.contains(&2));
        assert!(replace_result.removed_instance_keys.contains(&3));
        assert!(replace_result.inserted_instance_keys.contains(&5));
        assert!(replace_result.inserted_instance_keys.contains(&6));
        assert!(replace_result.inserted_instance_keys.contains(&7));
        assert!(replace_result.removed_instance_logic_ids.contains(&2));
        assert!(replace_result.removed_instance_logic_ids.contains(&3));
        assert!(replace_result.inserted_instance_logic_ids.contains(&5));
        assert!(replace_result.inserted_instance_logic_ids.contains(&6));
        assert!(replace_result.inserted_instance_logic_ids.contains(&7));

        assert!(tree.get(second).is_some());
    }

    #[test]
    fn finish_replace_subtree_keeps_reused_subtree_and_excludes_it_from_removed_sets() {
        let mut tree = ComponentTree::new();

        let root = tree.add_node(node("root", 1, 1));

        let _first = tree.add_node(node("first", 2, 2));
        let _first_child = tree.add_node(node("first_child", 3, 3));
        tree.pop_node();
        tree.pop_node();

        let second = tree.add_node(node("second", 4, 4));
        tree.pop_node();
        tree.pop_node();

        let context = match tree.begin_replace_subtree_by_instance_key(2) {
            Ok(context) => context,
            Err(_) => panic!("replace context should be created"),
        };

        let _replacement = tree.add_node(node("replacement", 2, 2));
        assert!(tree.try_reuse_current_subtree(2, 2));
        tree.pop_node();

        let replace_result = match tree.finish_replace_subtree(context) {
            Ok(result) => result,
            Err(_) => panic!("finish replace should succeed"),
        };

        let root_children = root
            .children(tree.tree())
            .map(|id| tree.get(id).expect("child must exist").fn_name.clone())
            .collect::<Vec<_>>();
        assert_eq!(root_children, vec!["first", "second"]);
        assert!(tree.find_node_id_by_instance_key(3).is_some());
        assert!(tree.get(second).is_some());

        assert!(replace_result.inserted_instance_keys.contains(&2));
        assert!(replace_result.inserted_instance_keys.contains(&3));
        assert!(replace_result.reused_instance_logic_ids.contains(&2));
        assert!(replace_result.reused_instance_logic_ids.contains(&3));
        assert!(!replace_result.removed_instance_keys.contains(&2));
        assert!(!replace_result.removed_instance_keys.contains(&3));
        assert!(!replace_result.removed_instance_logic_ids.contains(&2));
        assert!(!replace_result.removed_instance_logic_ids.contains(&3));
    }

    #[test]
    fn layout_adjacency_tracks_transparent_subtree_replacement() {
        let mut tree = ComponentTree::new();

        let root = tree.add_node(node_with_role("root", NodeRole::Composition, 1, 1));
        let layout_a = tree.add_node(node("layout_a", 2, 2));
        tree.pop_node();

        let composition = tree.add_node(node_with_role("wrapper", NodeRole::Composition, 3, 3));
        let layout_b = tree.add_node(node("layout_b", 4, 4));
        tree.pop_node();
        tree.pop_node();
        tree.pop_node();

        assert_eq!(composition.children(tree.tree()).count(), 1);
        assert_eq!(
            direct_layout_children(root, tree.tree()).as_ref(),
            &[layout_a, layout_b]
        );

        let context = tree
            .begin_replace_subtree_by_instance_key(3)
            .unwrap_or_else(|_| panic!("wrapper exists"));
        tree.add_node(node_with_role("replacement", NodeRole::Composition, 9, 9));
        tree.add_node(node("layout_b", 4, 4));
        assert!(tree.try_reuse_current_subtree(4, 4));
        tree.pop_node();
        let layout_c = tree.add_node(node("layout_c", 10, 10));
        tree.pop_node();
        tree.pop_node();
        assert!(tree.finish_replace_subtree(context).is_ok());
        assert_eq!(
            direct_layout_children(root, tree.tree()).as_ref(),
            &[layout_a, layout_b, layout_c]
        );

        let context = tree
            .begin_replace_subtree_by_instance_key(9)
            .unwrap_or_else(|_| panic!("replacement exists"));
        tree.add_node(node_with_role("reordered", NodeRole::Composition, 9, 9));
        for (name, key) in [("layout_c", 10), ("layout_b", 4)] {
            tree.add_node(node(name, key, key));
            assert!(tree.try_reuse_current_subtree(key, key));
            tree.pop_node();
        }
        tree.pop_node();
        assert!(tree.finish_replace_subtree(context).is_ok());
        assert_eq!(
            direct_layout_children(root, tree.tree()).as_ref(),
            &[layout_a, layout_c, layout_b]
        );
    }

    #[test]
    fn subtree_reuse_guard_tracks_pending_build_invalidations() {
        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let _child = tree.add_node(node("child", 2, 2));
        tree.pop_node();
        tree.pop_node();

        crate::runtime::reset_build_invalidations();
        assert!(!tree.subtree_has_pending_rebuild(root));

        crate::runtime::record_replay_boundary_invalidation_for_instance_key(2);
        assert!(
            tree.subtree_has_pending_rebuild(root),
            "a descendant with a pending build invalidation must block subtree reuse"
        );

        crate::runtime::reset_build_invalidations();
        assert!(!tree.subtree_has_pending_rebuild(root));
    }

    #[test]
    fn subtree_reuse_guard_tracks_current_frame_dirty_keys() {
        let mut tree = ComponentTree::new();
        let root = tree.add_node(node("root", 1, 1));
        let _child = tree.add_node(node("child", 2, 2));
        tree.pop_node();
        tree.pop_node();

        crate::runtime::reset_build_invalidations();
        let mut dirty = HashSet::default();
        dirty.insert(2);
        crate::runtime::with_build_dirty_instance_keys(&dirty, || {
            assert!(tree.subtree_has_pending_rebuild(root));
        });
        assert!(!tree.subtree_has_pending_rebuild(root));
    }
}
