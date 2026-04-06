mod constraint;
mod node;

use std::{
    num::NonZero,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use dashmap::DashMap;
use parking_lot::RwLock;
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet};
use tracing::{debug, warn};

use crate::{
    ComputeResourceManager, Px, PxRect,
    cursor::{CursorEventContent, PointerChange},
    focus::{
        FocusDirection, FocusHandleId, FocusOwner, PendingFocusCallbackInvocation, bind_focus_owner,
    },
    layout::{LayoutResult, RenderInput},
    modifier::{
        DrawModifierContent, DrawModifierContext, ImeInputModifierNode, KeyboardInputModifierNode,
        OrderedModifierAction, PointerInputModifierNode,
    },
    px::{PxPosition, PxSize},
    render_graph::{RenderGraph, RenderGraphBuilder},
    runtime::{
        LayoutDirtyNodes, RuntimePhase, StructureReconcileResult,
        push_current_component_instance_key, push_current_node_with_instance_logic_id, push_phase,
    },
    time::Instant,
};

pub use constraint::{AxisConstraint, Constraint, ParentConstraint};
pub use node::{
    ComputedData, ImeInput, ImeInputHandlerFn, ImeRequest, ImeSession, KeyboardInput,
    KeyboardInputHandlerFn, MeasurementError, PointerEventPass, PointerInput,
    PointerInputHandlerFn, WindowAction,
};

pub(crate) use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, NodeRole,
    WindowRequests, direct_layout_children, measure_node, measure_nodes,
};

#[cfg(feature = "profiling")]
use crate::profiler::{NodeMeta, Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard};

pub(crate) struct LayoutSnapshotEntry {
    pub constraint_key: Constraint,
    pub layout_result: LayoutResult,
    pub child_constraints: Vec<Constraint>,
    pub child_sizes: Vec<ComputedData>,
}

pub(crate) type LayoutSnapshotMap = DashMap<u64, LayoutSnapshotEntry, FxBuildHasher>;

#[derive(Default)]
struct LayoutSnapshotStore {
    entries: LayoutSnapshotMap,
}

thread_local! {
    static LAYOUT_SNAPSHOT_STORE: LayoutSnapshotStore = LayoutSnapshotStore::default();
}

fn with_layout_snapshot_entries<R>(f: impl FnOnce(&LayoutSnapshotMap) -> R) -> R {
    LAYOUT_SNAPSHOT_STORE.with(|store| f(&store.entries))
}

pub(crate) fn clear_layout_snapshots() {
    with_layout_snapshot_entries(LayoutSnapshotMap::clear);
}

fn remove_layout_snapshots(keys: &HashSet<u64>) {
    if keys.is_empty() {
        return;
    }
    with_layout_snapshot_entries(|snapshots| {
        for key in keys {
            snapshots.remove(key);
        }
    });
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutContext<'a> {
    pub snapshots: &'a LayoutSnapshotMap,
    pub measure_self_nodes: &'a HashSet<u64>,
    pub placement_self_nodes: &'a HashSet<u64>,
    pub dirty_effective_nodes: &'a HashSet<u64>,
    pub diagnostics: &'a LayoutDiagnosticsCollector,
}

#[cfg_attr(not(feature = "profiling"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutFrameDiagnostics {
    pub dirty_nodes_param: u64,
    pub dirty_nodes_structural: u64,
    pub dirty_nodes_with_ancestors: u64,
    pub dirty_expand_ns: u64,
    pub measure_node_calls: u64,
    pub cache_hits_direct: u64,
    pub cache_hits_boundary: u64,
    pub cache_miss_no_entry: u64,
    pub cache_miss_constraint: u64,
    pub cache_miss_dirty_self: u64,
    pub cache_miss_child_size: u64,
    pub cache_store_count: u64,
    pub cache_drop_non_cacheable_count: u64,
}

#[derive(Default)]
pub(crate) struct LayoutDiagnosticsCollector {
    measure_node_calls: AtomicU64,
    cache_hits_direct: AtomicU64,
    cache_hits_boundary: AtomicU64,
    cache_miss_no_entry: AtomicU64,
    cache_miss_constraint: AtomicU64,
    cache_miss_dirty_self: AtomicU64,
    cache_miss_child_size: AtomicU64,
    cache_store_count: AtomicU64,
    cache_drop_non_cacheable_count: AtomicU64,
}

impl LayoutDiagnosticsCollector {
    #[inline]
    pub(crate) fn inc_measure_node_calls(&self) {
        self.measure_node_calls.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_hit_direct(&self) {
        self.cache_hits_direct.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_hit_boundary(&self) {
        self.cache_hits_boundary.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_miss_no_entry(&self) {
        self.cache_miss_no_entry.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_miss_constraint(&self) {
        self.cache_miss_constraint.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_miss_dirty_self(&self) {
        self.cache_miss_dirty_self.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_miss_child_size(&self) {
        self.cache_miss_child_size.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_store_count(&self) {
        self.cache_store_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn inc_cache_drop_non_cacheable_count(&self) {
        self.cache_drop_non_cacheable_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(
        &self,
        dirty_nodes_param: u64,
        dirty_nodes_structural: u64,
        dirty_nodes_with_ancestors: u64,
        dirty_expand_ns: u64,
    ) -> LayoutFrameDiagnostics {
        let cache_hits_direct = self.cache_hits_direct.load(Ordering::Relaxed);
        let cache_hits_boundary = self.cache_hits_boundary.load(Ordering::Relaxed);
        LayoutFrameDiagnostics {
            dirty_nodes_param,
            dirty_nodes_structural,
            dirty_nodes_with_ancestors,
            dirty_expand_ns,
            measure_node_calls: self.measure_node_calls.load(Ordering::Relaxed),
            cache_hits_direct,
            cache_hits_boundary,
            cache_miss_no_entry: self.cache_miss_no_entry.load(Ordering::Relaxed),
            cache_miss_constraint: self.cache_miss_constraint.load(Ordering::Relaxed),
            cache_miss_dirty_self: self.cache_miss_dirty_self.load(Ordering::Relaxed),
            cache_miss_child_size: self.cache_miss_child_size.load(Ordering::Relaxed),
            cache_store_count: self.cache_store_count.load(Ordering::Relaxed),
            cache_drop_non_cacheable_count: self
                .cache_drop_non_cacheable_count
                .load(Ordering::Relaxed),
        }
    }
}

/// Parameters for the compute function
pub(crate) struct ComputeParams<'a> {
    pub screen_size: PxSize,
    pub cursor_position: Option<PxPosition>,
    pub pointer_changes: Vec<PointerChange>,
    pub keyboard_events: Vec<winit::event::KeyEvent>,
    pub ime_events: Vec<winit::event::Ime>,
    pub retry_focus_move: Option<FocusDirection>,
    pub retry_focus_reveal: bool,
    pub modifiers: winit::keyboard::ModifiersState,
    pub layout_dirty_nodes: &'a LayoutDirtyNodes,
}

#[derive(Debug)]
pub(crate) enum ComputeMode<'a> {
    Full {
        compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
        gpu: &'a wgpu::Device,
    },
    #[cfg(feature = "testing")]
    LayoutOnly,
}

/// Respents a component tree
pub struct ComponentTree {
    /// We use indextree as the tree structure
    tree: indextree::Arena<ComponentNode>,
    /// Components' metadatas
    metadatas: ComponentNodeMetaDatas,
    /// Used to remember the current node
    node_queue: Vec<indextree::NodeId>,
    /// Detached old-subtree nodes keyed by instance key during replay replace.
    replay_reuse_candidates: HashMap<u64, indextree::NodeId>,
    /// Active pointer hit paths keyed by pointer id.
    /// Each path stores node instance keys from root to leaf.
    active_pointer_paths: HashMap<u64, Vec<u64>>,
    /// Per-tree focus owner used for keyboard and IME routing.
    focus_owner: FocusOwner,
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
        let metadatas = ComponentNodeMetaDatas::with_hasher(FxBuildHasher);
        Self {
            tree,
            node_queue,
            metadatas,
            replay_reuse_candidates: HashMap::default(),
            active_pointer_paths: HashMap::default(),
            focus_owner: FocusOwner::new(),
        }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        self.tree.clear();
        self.metadatas.clear();
        self.node_queue.clear();
        self.replay_reuse_candidates.clear();
        self.active_pointer_paths.clear();
    }

    /// Reset the entire component tree, including focus ownership state.
    pub fn reset(&mut self) {
        self.clear();
        self.focus_owner.reset();
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
        let root = self
            .tree
            .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))?;
        for edge in root.traverse(&self.tree) {
            let indextree::NodeEdge::Start(node_id) = edge else {
                continue;
            };
            let Some(node) = self.tree.get(node_id) else {
                continue;
            };
            if node.get().instance_key == instance_key {
                return Some(node_id);
            }
        }
        None
    }

    pub(crate) fn live_instance_keys(&self) -> HashSet<u64> {
        let Some(root) = self
            .tree
            .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))
        else {
            return HashSet::default();
        };

        let mut instance_keys = HashSet::default();
        for edge in root.traverse(&self.tree) {
            let indextree::NodeEdge::Start(node_id) = edge else {
                continue;
            };
            let Some(node) = self.tree.get(node_id) else {
                continue;
            };
            instance_keys.insert(node.get().instance_key);
        }
        instance_keys
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
        for inserted_root_id in &inserted_root_ids {
            for edge in inserted_root_id.traverse(&self.tree) {
                let indextree::NodeEdge::Start(id) = edge else {
                    continue;
                };
                if let Some(node) = self.tree.get(id) {
                    inserted_instance_keys.insert(node.get().instance_key);
                    inserted_instance_logic_ids.insert(node.get().instance_logic_id);
                    if detached_node_ids.contains(&id) {
                        reused_instance_logic_ids.insert(node.get().instance_logic_id);
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
        if !detached_root_reused {
            let detached_node_ids = detached_root_id
                .traverse(&self.tree)
                .filter_map(|edge| match edge {
                    indextree::NodeEdge::Start(id) => Some(id),
                    indextree::NodeEdge::End(_) => None,
                })
                .collect::<Vec<_>>();
            for node_id in &detached_node_ids {
                self.metadatas.remove(node_id);
            }
            detached_root_id.remove_subtree(&mut self.tree);
        }
        self.replay_reuse_candidates.clear();

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
        current_node_id.remove_subtree(&mut self.tree);
        if let Some(current_node) = self.node_queue.last_mut() {
            *current_node = candidate_node_id;
        }
        self.replay_reuse_candidates.remove(&instance_key);
        true
    }

    /// Add a new node to the tree
    /// Nodes now store their intrinsic constraints in their metadata.
    /// The `node_component` itself primarily holds the layout policy, render
    /// policy, and handlers.
    pub(crate) fn add_node(&mut self, node_component: ComponentNode) -> indextree::NodeId {
        let new_node_id = self.tree.new_node(node_component);
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
        self.node_queue.pop();
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
    pub(crate) fn metadatas(&self) -> &ComponentNodeMetaDatas {
        &self.metadatas
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
                ));
            }
            if let Some(handler) = &node.focus_event_handler {
                invocations.push(PendingFocusCallbackInvocation::new(
                    *handler,
                    notification.state,
                ));
            }
        }

        invocations
    }

    /// Collect per-node metadata for profiling output.
    #[cfg(feature = "profiling")]
    pub fn profiler_nodes(&self) -> Vec<NodeMeta> {
        let Some(root_node) = self
            .tree
            .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))
        else {
            return Vec::new();
        };

        let mut stack = vec![root_node];
        let mut nodes = Vec::new();
        while let Some(node_id) = stack.pop() {
            if let Some(node) = self.tree.get(node_id) {
                let parent = node.parent().map(|p| p.to_string());
                let fn_name = node.get().fn_name.clone();
                let metadata = self.metadatas.get(&node_id);
                let abs_pos = metadata
                    .as_ref()
                    .and_then(|m| m.abs_position)
                    .map(|p| (p.x.0, p.y.0));
                let size = metadata
                    .as_ref()
                    .and_then(|m| m.computed_data)
                    .map(|d| (d.width.0, d.height.0));
                let layout_cache_hit = metadata.as_ref().map(|m| m.layout_cache_hit);

                nodes.push(NodeMeta {
                    node_id: node_id.to_string(),
                    parent,
                    fn_name: Some(fn_name.clone()),
                    abs_pos,
                    size,
                    layout_cache_hit,
                });
            }
            stack.extend(node_id.children(&self.tree));
        }

        nodes
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
    ) {
        let ComputeParams {
            screen_size,
            mut cursor_position,
            mut pointer_changes,
            mut keyboard_events,
            mut ime_events,
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
            );
        };
        let screen_constraint = Constraint::exact(screen_size.width, screen_size.height);
        let current_children_by_node = collect_children_by_instance_key(root_node, &self.tree);
        let StructureReconcileResult {
            changed_nodes: structural_dirty_nodes,
            removed_nodes,
        } = crate::runtime::reconcile_layout_structure(&current_children_by_node);
        remove_layout_snapshots(&removed_nodes);

        let mut dirty_nodes_self = layout_dirty_nodes.measure_self_nodes.clone();
        dirty_nodes_self.extend(layout_dirty_nodes.placement_self_nodes.iter().copied());
        let dirty_nodes_param = dirty_nodes_self.len() as u64;
        let dirty_nodes_structural = structural_dirty_nodes.len() as u64;
        let dirty_prepare_start = Instant::now();
        dirty_nodes_self.extend(structural_dirty_nodes.iter().copied());
        let dirty_nodes_effective =
            expand_dirty_nodes_with_ancestors(root_node, &self.tree, &dirty_nodes_self);
        let dirty_expand_ns = dirty_prepare_start.elapsed().as_nanos() as u64;
        let diagnostics = LayoutDiagnosticsCollector::default();

        self.focus_owner
            .sync_from_component_tree(root_node, &self.tree);
        self.focus_owner.commit_pending();

        let diagnostics_snapshot = with_layout_snapshot_entries(|snapshots| {
            let layout_ctx = LayoutContext {
                snapshots,
                measure_self_nodes: &layout_dirty_nodes.measure_self_nodes,
                placement_self_nodes: &layout_dirty_nodes.placement_self_nodes,
                dirty_effective_nodes: &dirty_nodes_effective,
                diagnostics: &diagnostics,
            };

            let measure_timer = Instant::now();
            debug!("Start measuring the component tree...");

            match measure_node(
                root_node,
                &screen_constraint,
                &self.tree,
                &self.metadatas,
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

            diagnostics.snapshot(
                dirty_nodes_param,
                dirty_nodes_structural,
                dirty_nodes_effective.len() as u64,
                dirty_expand_ns,
            )
        });

        let (compute_resource_manager, gpu) = match mode {
            ComputeMode::Full {
                compute_resource_manager,
                gpu,
            } => (compute_resource_manager, gpu),
            #[cfg(feature = "testing")]
            ComputeMode::LayoutOnly => {
                populate_layout_metadata(root_node, &self.tree, &self.metadatas);
                return (
                    RenderGraph::default(),
                    WindowRequests::default(),
                    diagnostics_snapshot,
                    std::time::Duration::ZERO,
                    None,
                    false,
                );
            }
        };

        let record_timer = Instant::now();
        record_layout_commands(
            root_node,
            &self.tree,
            &self.metadatas,
            compute_resource_manager.clone(),
            gpu,
        );
        let record_cost = record_timer.elapsed();
        populate_layout_metadata(root_node, &self.tree, &self.metadatas);

        let compute_draw_timer = Instant::now();
        debug!("Start computing render graph...");
        let graph = build_render_graph(root_node, &self.tree, &self.metadatas, screen_size);
        self.focus_owner
            .sync_layout_from_component_tree(root_node, &self.tree, &self.metadatas);
        debug!(
            "Render graph built in {:?}, total ops: {}",
            compute_draw_timer.elapsed(),
            graph.ops().len()
        );

        let input_dispatch_timer = Instant::now();
        let mut window_requests = WindowRequests::default();
        debug!("Start executing typed input dispatch...");

        let node_ids_preorder: Vec<_> = root_node
            .traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
            .collect();
        let node_ids_postorder: Vec<_> = root_node
            .reverse_traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
            .collect();
        let pointer_change_paths = build_pointer_change_paths(
            root_node,
            &self.tree,
            &self.metadatas,
            &pointer_changes,
            cursor_position,
            &mut self.active_pointer_paths,
        );
        window_requests.cursor_icon =
            resolve_hover_cursor_icon(root_node, &self.tree, &self.metadatas, cursor_position)
                .unwrap_or_default();

        for node_id in node_ids_preorder.iter().copied() {
            let Some(node) = self.tree.get(node_id).map(|n| n.get()) else {
                continue;
            };
            let mut dispatch_ctx = PointerInputDispatchContext {
                tree: &self.tree,
                metadatas: &self.metadatas,
                cursor_position: &mut cursor_position,
                pointer_changes: pointer_changes.as_mut_slice(),
                pointer_change_paths: &pointer_change_paths,
                modifiers,
                window_requests: &mut window_requests,
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
                    cursor_position: &mut cursor_position,
                    pointer_changes: pointer_changes.as_mut_slice(),
                    pointer_change_paths: &pointer_change_paths,
                    modifiers,
                    window_requests: &mut window_requests,
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
                cursor_position: &mut cursor_position,
                pointer_changes: pointer_changes.as_mut_slice(),
                pointer_change_paths: &pointer_change_paths,
                modifiers,
                window_requests: &mut window_requests,
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
                    cursor_position: &mut cursor_position,
                    pointer_changes: pointer_changes.as_mut_slice(),
                    pointer_change_paths: &pointer_change_paths,
                    modifiers,
                    window_requests: &mut window_requests,
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
                cursor_position: &mut cursor_position,
                pointer_changes: pointer_changes.as_mut_slice(),
                pointer_change_paths: &pointer_change_paths,
                modifiers,
                window_requests: &mut window_requests,
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
                    cursor_position: &mut cursor_position,
                    pointer_changes: pointer_changes.as_mut_slice(),
                    pointer_change_paths: &pointer_change_paths,
                    modifiers,
                    window_requests: &mut window_requests,
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

        self.focus_owner.commit_pending();
        let pending_focus_move_retry = retry_focus_move.and_then(|direction| {
            match try_dispatch_focus_move_request(&self.tree, direction, &mut self.focus_owner) {
                FocusMoveRequestResult::Retry(direction) => Some(direction),
                FocusMoveRequestResult::Moved | FocusMoveRequestResult::NotHandled => None,
            }
        });
        let focus_chain_node_ids =
            collect_focus_chain_node_ids(root_node, &self.tree, &self.focus_owner);

        let pending_focus_move_retry = if pending_focus_move_retry.is_none() {
            let mut keyboard_dispatch_ctx = KeyboardInputDispatchContext {
                tree: &self.tree,
                metadatas: &self.metadatas,
                keyboard_events: &mut keyboard_events,
                modifiers,
                window_requests: &mut window_requests,
                focus_owner: &mut self.focus_owner,
            };

            for node_id in focus_chain_node_ids.iter().copied() {
                let Some(node) = keyboard_dispatch_ctx.tree.get(node_id).map(|n| n.get()) else {
                    continue;
                };
                for action in node.modifier.ordered_actions() {
                    match action {
                        OrderedModifierAction::KeyboardPreviewInput(modifier) => {
                            run_keyboard_modifier_for_node(
                                &mut keyboard_dispatch_ctx,
                                node_id,
                                modifier.as_ref(),
                            );
                        }
                        OrderedModifierAction::ImePreviewInput(modifier) => {
                            run_ime_modifier_for_node(
                                keyboard_dispatch_ctx.tree,
                                keyboard_dispatch_ctx.metadatas,
                                node_id,
                                modifier.as_ref(),
                                &mut ime_events,
                                keyboard_dispatch_ctx.window_requests,
                                keyboard_dispatch_ctx.focus_owner,
                            );
                        }
                        _ => {}
                    }
                }
                for handler in &node.keyboard_preview_handlers {
                    run_keyboard_handler_for_node(
                        &mut keyboard_dispatch_ctx,
                        node_id,
                        handler.as_ref(),
                    );
                }
                for handler in &node.ime_preview_handlers {
                    run_ime_handler_for_node(
                        keyboard_dispatch_ctx.tree,
                        keyboard_dispatch_ctx.metadatas,
                        node_id,
                        handler.as_ref(),
                        &mut ime_events,
                        keyboard_dispatch_ctx.window_requests,
                        keyboard_dispatch_ctx.focus_owner,
                    );
                }
            }

            for node_id in focus_chain_node_ids.iter().rev().copied() {
                let Some(node) = keyboard_dispatch_ctx.tree.get(node_id).map(|n| n.get()) else {
                    continue;
                };
                for action in node.modifier.ordered_actions() {
                    match action {
                        OrderedModifierAction::KeyboardInput(modifier) => {
                            run_keyboard_modifier_for_node(
                                &mut keyboard_dispatch_ctx,
                                node_id,
                                modifier.as_ref(),
                            );
                        }
                        OrderedModifierAction::ImeInput(modifier) => {
                            run_ime_modifier_for_node(
                                keyboard_dispatch_ctx.tree,
                                keyboard_dispatch_ctx.metadatas,
                                node_id,
                                modifier.as_ref(),
                                &mut ime_events,
                                keyboard_dispatch_ctx.window_requests,
                                keyboard_dispatch_ctx.focus_owner,
                            );
                        }
                        _ => {}
                    }
                }
                for handler in &node.keyboard_handlers {
                    run_keyboard_handler_for_node(
                        &mut keyboard_dispatch_ctx,
                        node_id,
                        handler.as_ref(),
                    );
                }
                for handler in &node.ime_handlers {
                    run_ime_handler_for_node(
                        keyboard_dispatch_ctx.tree,
                        keyboard_dispatch_ctx.metadatas,
                        node_id,
                        handler.as_ref(),
                        &mut ime_events,
                        keyboard_dispatch_ctx.window_requests,
                        keyboard_dispatch_ctx.focus_owner,
                    );
                }
            }

            dispatch_default_focus_keyboard_navigation(
                keyboard_dispatch_ctx.tree,
                keyboard_dispatch_ctx.keyboard_events,
                keyboard_dispatch_ctx.modifiers,
                keyboard_dispatch_ctx.focus_owner,
            )
        } else {
            pending_focus_move_retry
        };
        let pending_focus_reveal_retry = if pending_focus_move_retry.is_none() {
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
        )
    }
}

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
    drop(metadata);

    let Some(node_ref) = tree.get(node_id) else {
        warn!("Node not found for node {node_id:?}; skipping input handling");
        return None;
    };
    let node = node_ref.get();
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

fn hit_path_node_ids(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    position: Option<PxPosition>,
) -> Vec<indextree::NodeId> {
    fn collect_hit_path(
        node_id: indextree::NodeId,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        position: PxPosition,
    ) -> Option<Vec<indextree::NodeId>> {
        let children: Vec<_> = node_id.children(tree).collect();
        for child_id in children.into_iter().rev() {
            if let Some(mut child_path) = collect_hit_path(child_id, tree, metadatas, position) {
                let mut path = Vec::with_capacity(child_path.len() + 1);
                if let Some(metadata) = metadatas.get(&node_id)
                    && metadata.base_abs_position.is_some()
                    && metadata.abs_position.is_some()
                    && metadata.computed_data.is_some()
                {
                    path.push(node_id);
                }
                path.append(&mut child_path);
                return Some(path);
            }
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
        let bounds = PxRect::from_position_size(abs_pos, PxSize::new(size.width, size.height));
        let node_handles_hover = tree.get(node_id).is_some_and(|node| {
            node_handles_pointer_at_position(node.get(), base_abs_pos, size, position)
        }) || bounds.contains(position)
            && tree.get(node_id).is_some_and(|node| {
                let node = node.get();
                !node.pointer_preview_handlers.is_empty()
                    || !node.pointer_handlers.is_empty()
                    || !node.pointer_final_handlers.is_empty()
            });

        node_handles_hover.then_some(vec![node_id])
    }

    let Some(position) = position else {
        return Vec::new();
    };
    collect_hit_path(root_node, tree, metadatas, position).unwrap_or_default()
}

fn resolve_hover_cursor_icon(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    position: Option<PxPosition>,
) -> Option<winit::window::CursorIcon> {
    hit_path_node_ids(root_node, tree, metadatas, position)
        .into_iter()
        .rev()
        .find_map(|node_id| {
            let node_ref = tree.get(node_id)?;
            let metadata = metadatas.get(&node_id)?;
            let base_abs_pos = metadata.base_abs_position?;
            let size = metadata.computed_data?;
            drop(metadata);
            resolve_node_hover_cursor_icon(node_ref.get(), base_abs_pos, size, position?)
        })
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
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    pointer_changes: &[PointerChange],
    cursor_position: Option<PxPosition>,
    active_pointer_paths: &mut HashMap<u64, Vec<u64>>,
) -> Vec<Vec<u64>> {
    let mut paths = Vec::with_capacity(pointer_changes.len());
    for change in pointer_changes {
        let debug_position = match &change.content {
            CursorEventContent::Moved(position) => Some(*position),
            _ => cursor_position,
        };
        let path = match &change.content {
            CursorEventContent::Pressed(_) => {
                let computed_path =
                    hit_path_instance_keys(root_node, tree, metadatas, debug_position);
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
                .unwrap_or_else(|| {
                    hit_path_instance_keys(root_node, tree, metadatas, Some(*position))
                }),
            CursorEventContent::Released(_) => {
                let computed = active_pointer_paths
                    .get(&change.pointer_id)
                    .cloned()
                    .unwrap_or_else(|| {
                        hit_path_instance_keys(root_node, tree, metadatas, debug_position)
                    });
                active_pointer_paths.remove(&change.pointer_id);
                computed
            }
            CursorEventContent::Scroll(_) => active_pointer_paths
                .get(&change.pointer_id)
                .cloned()
                .unwrap_or_else(|| {
                    hit_path_instance_keys(root_node, tree, metadatas, debug_position)
                }),
        };
        paths.push(path);
    }
    paths
}

fn hit_path_instance_keys(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    position: Option<PxPosition>,
) -> Vec<u64> {
    hit_path_node_ids(root_node, tree, metadatas, position)
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

struct PointerInputDispatchContext<'a> {
    tree: &'a ComponentNodeTree,
    metadatas: &'a ComponentNodeMetaDatas,
    cursor_position: &'a mut Option<PxPosition>,
    pointer_changes: &'a mut [PointerChange],
    pointer_change_paths: &'a [Vec<u64>],
    modifiers: winit::keyboard::ModifiersState,
    window_requests: &'a mut WindowRequests,
    focus_owner: &'a mut FocusOwner,
}

fn dispatch_pointer_modifiers_for_node_pass(
    dispatch_ctx: &mut PointerInputDispatchContext<'_>,
    node_id: indextree::NodeId,
    pass: PointerEventPass,
) {
    let Some(NodeInputContext { base_abs_pos, .. }) =
        resolve_node_input_context(dispatch_ctx.tree, dispatch_ctx.metadatas, node_id)
    else {
        return;
    };
    let Some(node_ref) = dispatch_ctx.tree.get(node_id) else {
        return;
    };

    let mut current_abs_pos = base_abs_pos;
    for action in node_ref.get().modifier.ordered_actions() {
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
    let Some(NodeInputContext { abs_pos, .. }) =
        resolve_node_input_context(dispatch_ctx.tree, dispatch_ctx.metadatas, node_id)
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
    let Some(NodeInputContext {
        base_abs_pos: _,
        abs_pos: _,
        event_clip_rect,
        node_computed_data,
        instance_logic_id,
        instance_key,
        fn_name,
        parent_id,
    }) = resolve_node_input_context(dispatch_ctx.tree, dispatch_ctx.metadatas, node_id)
    else {
        return;
    };
    #[cfg(not(feature = "profiling"))]
    let _ = parent_id;

    let abs_pos = abs_pos_override;

    let mut cursor_position_ref = &mut *dispatch_ctx.cursor_position;
    let mut dummy_cursor_position = None;
    if let (Some(cursor_pos), Some(clip_rect)) = (*cursor_position_ref, event_clip_rect)
        && !clip_rect.contains(cursor_pos)
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
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(instance_key);
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
        window_action: &mut dispatch_ctx.window_requests.window_action,
    };
    dispatch(input);
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
        let abs_tuple = (abs_pos.x.0, abs_pos.y.0);
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
        instance_key,
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
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(instance_key);
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
        let abs_tuple = (abs_pos.x.0, abs_pos.y.0);
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
        instance_key,
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
    let _node_ctx_guard =
        push_current_node_with_instance_logic_id(node_id, instance_logic_id, fn_name.as_str());
    let _instance_ctx_guard = push_current_component_instance_key(instance_key);
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
        let abs_tuple = (abs_pos.x.0, abs_pos.y.0);
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
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    dirty_nodes_self: &HashSet<u64>,
) -> HashSet<u64> {
    if dirty_nodes_self.is_empty() {
        return HashSet::default();
    }

    let mut parent_by_key: HashMap<u64, u64> = HashMap::default();
    for edge in root_node.traverse(tree) {
        let indextree::NodeEdge::Start(node_id) = edge else {
            continue;
        };
        let Some(node) = tree.get(node_id) else {
            continue;
        };
        let instance_key = node.get().instance_key;
        let Some(parent_id) = node.parent() else {
            continue;
        };
        if let Some(parent) = tree.get(parent_id) {
            parent_by_key.insert(instance_key, parent.get().instance_key);
        }
    }

    let mut expanded = dirty_nodes_self.clone();
    for dirty_key in dirty_nodes_self {
        let mut current = *dirty_key;
        while let Some(parent_key) = parent_by_key.get(&current).copied() {
            expanded.insert(parent_key);
            current = parent_key;
        }
    }

    expanded
}

fn collect_children_by_instance_key(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
) -> HashMap<u64, Vec<u64>> {
    let mut children_by_node = HashMap::default();
    for edge in root_node.traverse(tree) {
        let indextree::NodeEdge::Start(node_id) = edge else {
            continue;
        };
        let Some(node) = tree.get(node_id) else {
            continue;
        };
        let instance_key = node.get().instance_key;
        let child_keys = direct_layout_children(node_id, tree)
            .into_iter()
            .filter_map(|child_id| tree.get(child_id).map(|child| child.get().instance_key))
            .collect();
        children_by_node.insert(instance_key, child_keys);
    }
    children_by_node
}

fn record_layout_commands(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
) {
    struct DrawModifierChainContent<'a> {
        draw_nodes: &'a [Arc<dyn crate::modifier::DrawModifierNode>],
        render_policy: &'a dyn crate::layout::RenderPolicyDyn,
    }

    impl DrawModifierContent for DrawModifierChainContent<'_> {
        fn draw(&mut self, input: &RenderInput<'_>) {
            if let Some((draw_modifier, remaining)) = self.draw_nodes.split_first() {
                let mut draw_ctx = DrawModifierContext {
                    render_input: input,
                };
                let mut next = DrawModifierChainContent {
                    draw_nodes: remaining,
                    render_policy: self.render_policy,
                };
                draw_modifier.draw(&mut draw_ctx, &mut next);
            } else {
                self.render_policy.record_dyn(input);
            }
        }
    }

    let mut stack = vec![root_node];
    while let Some(node_id) = stack.pop() {
        let Some(node) = tree.get(node_id) else {
            continue;
        };
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
        let input = RenderInput::new(node_id, metadatas, compute_resource_manager.clone(), gpu);
        let node_ref = node.get();
        let draw_nodes: Vec<_> = node_ref
            .modifier
            .ordered_actions()
            .into_iter()
            .filter_map(|action| match action {
                OrderedModifierAction::Draw(node) => Some(node),
                _ => None,
            })
            .collect();
        let mut content = DrawModifierChainContent {
            draw_nodes: &draw_nodes,
            render_policy: node_ref.render_policy.as_ref(),
        };
        content.draw(&input);
        stack.extend(node_id.children(tree));
    }
}

#[derive(Clone, Copy)]
struct PreparedLayoutMetadata {
    self_position: PxPosition,
    size: PxSize,
    node_rect: PxRect,
    clips_children: bool,
    child_clip_rect: Option<PxRect>,
    cumulative_opacity: f32,
}

fn prepare_layout_metadata_for_node(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    parent_clip_rect: Option<PxRect>,
    current_opacity: f32,
) -> Option<PreparedLayoutMetadata> {
    let mut metadata = metadatas.get_mut(&node_id)?;
    let rel_pos = match metadata.rel_position {
        Some(pos) => pos,
        None if is_root => PxPosition::ZERO,
        _ => {
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
        x: self_position.x,
        y: self_position.y,
        width: size.width,
        height: size.height,
    };
    let clips_children = metadata.clips_children;
    let child_clip_rect = if clips_children {
        Some(
            parent_clip_rect
                .and_then(|existing| existing.intersection(&node_rect))
                .unwrap_or(node_rect),
        )
    } else {
        parent_clip_rect
    };

    Some(PreparedLayoutMetadata {
        self_position,
        size,
        node_rect,
        clips_children,
        child_clip_rect,
        cumulative_opacity,
    })
}

fn populate_layout_metadata(
    root_node: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) {
    fn visit(
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        start_pos: PxPosition,
        is_root: bool,
        node_id: indextree::NodeId,
        clip_rect: Option<PxRect>,
        current_opacity: f32,
    ) {
        let Some(prepared) = prepare_layout_metadata_for_node(
            tree,
            metadatas,
            start_pos,
            is_root,
            node_id,
            clip_rect,
            current_opacity,
        ) else {
            for child in node_id.children(tree) {
                visit(
                    tree,
                    metadatas,
                    start_pos,
                    false,
                    child,
                    clip_rect,
                    current_opacity,
                );
            }
            return;
        };

        for child in node_id.children(tree) {
            visit(
                tree,
                metadatas,
                prepared.self_position,
                false,
                child,
                prepared.child_clip_rect,
                prepared.cumulative_opacity,
            );
        }
    }

    visit(
        tree,
        metadatas,
        PxPosition::ZERO,
        true,
        root_node,
        None,
        1.0,
    );
}

/// Sequential computation of render graph ops from the component tree.
#[tracing::instrument(level = "trace", skip(tree, metadatas))]
fn build_render_graph(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    screen_size: PxSize,
) -> RenderGraph {
    let screen_rect = PxRect {
        x: Px(0),
        y: Px(0),
        width: screen_size.width,
        height: screen_size.height,
    };

    let mut builder = RenderGraphBuilder::new();
    let mut context = RenderGraphBuildContext {
        tree,
        metadatas,
        builder: &mut builder,
        screen_rect,
    };
    build_render_graph_inner(&mut context, PxPosition::ZERO, true, node_id, None, 1.0);
    builder.finish()
}

struct RenderGraphBuildContext<'a> {
    tree: &'a ComponentNodeTree,
    metadatas: &'a ComponentNodeMetaDatas,
    builder: &'a mut RenderGraphBuilder,
    screen_rect: PxRect,
}

#[tracing::instrument(level = "trace", skip(context))]
fn build_render_graph_inner(
    context: &mut RenderGraphBuildContext<'_>,
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    clip_rect: Option<PxRect>,
    current_opacity: f32,
) {
    let Some(prepared) = prepare_layout_metadata_for_node(
        context.tree,
        context.metadatas,
        start_pos,
        is_root,
        node_id,
        clip_rect,
        current_opacity,
    ) else {
        for child in node_id.children(context.tree) {
            build_render_graph_inner(context, start_pos, false, child, clip_rect, current_opacity);
        }
        return;
    };

    if prepared.clips_children {
        context
            .builder
            .push_clip_push(prepared.child_clip_rect.unwrap_or(PxRect::ZERO));
    }

    let fragment = match context.metadatas.get_mut(&node_id) {
        Some(mut metadata) => metadata.take_fragment(),
        None => {
            warn!("Missing metadata for node {node_id:?}; skipping render graph build");
            return;
        }
    };

    if prepared.size.width.0 > 0
        && prepared.size.height.0 > 0
        && !prepared.node_rect.is_orthogonal(&context.screen_rect)
    {
        context.builder.append_fragment(
            fragment,
            prepared.size,
            prepared.self_position,
            prepared.cumulative_opacity,
        );
    }

    for child in node_id.children(context.tree) {
        build_render_graph_inner(
            context,
            prepared.self_position,
            false,
            child,
            prepared.child_clip_rect,
            prepared.cumulative_opacity,
        );
    }

    if prepared.clips_children {
        context.builder.push_clip_pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        component_tree::{ComponentNode, NodeRole},
        layout::{DefaultLayoutPolicy, NoopRenderPolicy},
        modifier::Modifier,
    };

    fn node(name: &str, instance_logic_id: u64, instance_key: u64) -> ComponentNode {
        ComponentNode {
            fn_name: name.to_string(),
            role: NodeRole::Layout,
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
            modifier: Modifier::default(),
            layout_policy: Box::new(DefaultLayoutPolicy),
            render_policy: Box::new(NoopRenderPolicy),
            replay: None,
            props_unchanged_from_previous: false,
        }
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
}
