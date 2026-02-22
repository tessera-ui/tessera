mod constraint;
mod node;

use std::{
    num::NonZero,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use dashmap::DashMap;
use parking_lot::RwLock;
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet};
use tracing::{debug, warn};

use crate::{
    ComputeResourceManager, Px, PxRect,
    cursor::CursorEvent,
    layout::{LayoutResult, RenderInput},
    px::{PxPosition, PxSize},
    render_graph::{RenderGraph, RenderGraphBuilder},
    runtime::{
        RuntimePhase, StructureReconcileResult, push_current_component_instance_key,
        push_current_node_with_instance_logic_id, push_phase,
    },
};

pub use constraint::{Constraint, DimensionValue, ParentConstraint};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    ImeRequest, InputHandlerFn, InputHandlerInput, MeasurementError, WindowAction, WindowRequests,
};

pub(crate) use node::{measure_node, measure_nodes};

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

static LAYOUT_SNAPSHOT_STORE: OnceLock<LayoutSnapshotStore> = OnceLock::new();

fn layout_snapshot_entries() -> &'static LayoutSnapshotMap {
    &LAYOUT_SNAPSHOT_STORE
        .get_or_init(LayoutSnapshotStore::default)
        .entries
}

pub(crate) fn clear_layout_snapshots() {
    layout_snapshot_entries().clear();
}

fn remove_layout_snapshots(keys: &HashSet<u64>) {
    if keys.is_empty() {
        return;
    }
    let snapshots = layout_snapshot_entries();
    for key in keys {
        snapshots.remove(key);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LayoutContext<'a> {
    pub snapshots: &'a LayoutSnapshotMap,
    pub dirty_self_nodes: &'a HashSet<u64>,
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
    pub cursor_events: Vec<CursorEvent>,
    pub keyboard_events: Vec<winit::event::KeyEvent>,
    pub ime_events: Vec<winit::event::Ime>,
    pub modifiers: winit::keyboard::ModifiersState,
    pub compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    pub gpu: &'a wgpu::Device,
    pub layout_self_dirty_nodes: &'a HashSet<u64>,
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
        }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        self.tree.clear();
        self.metadatas.clear();
        self.node_queue.clear();
        self.replay_reuse_candidates.clear();
    }

    /// Get node by NodeId
    pub fn get(&self, node_id: indextree::NodeId) -> Option<&ComponentNode> {
        self.tree.get(node_id).map(|n| n.get())
    }

    /// Get mutable node by NodeId
    pub fn get_mut(&mut self, node_id: indextree::NodeId) -> Option<&mut ComponentNode> {
        self.tree.get_mut(node_id).map(|n| n.get_mut())
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
    pub fn current_node(&self) -> Option<&ComponentNode> {
        self.node_queue
            .last()
            .and_then(|node_id| self.get(*node_id))
    }

    /// Get mutable current node
    pub fn current_node_mut(&mut self) -> Option<&mut ComponentNode> {
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
    /// The `node_component` itself primarily holds the layout spec and
    /// handlers.
    pub fn add_node(&mut self, node_component: ComponentNode) -> indextree::NodeId {
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
    pub fn pop_node(&mut self) {
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
    ) -> (
        RenderGraph,
        WindowRequests,
        LayoutFrameDiagnostics,
        std::time::Duration,
    ) {
        let ComputeParams {
            screen_size,
            mut cursor_position,
            mut cursor_events,
            mut keyboard_events,
            mut ime_events,
            modifiers,
            compute_resource_manager,
            gpu,
            layout_self_dirty_nodes,
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
            );
        };
        let screen_constraint = Constraint::new(
            DimensionValue::Fixed(screen_size.width),
            DimensionValue::Fixed(screen_size.height),
        );
        let current_children_by_node = collect_children_by_instance_key(root_node, &self.tree);
        let StructureReconcileResult {
            changed_nodes: structural_dirty_nodes,
            removed_nodes,
        } = crate::runtime::reconcile_layout_structure(&current_children_by_node);
        remove_layout_snapshots(&removed_nodes);

        let dirty_nodes_param = layout_self_dirty_nodes.len() as u64;
        let dirty_nodes_structural = structural_dirty_nodes.len() as u64;
        let dirty_prepare_start = Instant::now();
        let mut dirty_nodes_self = layout_self_dirty_nodes.clone();
        dirty_nodes_self.extend(structural_dirty_nodes.iter().copied());
        let dirty_nodes_effective =
            expand_dirty_nodes_with_ancestors(root_node, &self.tree, &dirty_nodes_self);
        let dirty_expand_ns = dirty_prepare_start.elapsed().as_nanos() as u64;
        let diagnostics = LayoutDiagnosticsCollector::default();

        let layout_ctx = LayoutContext {
            snapshots: layout_snapshot_entries(),
            dirty_self_nodes: &dirty_nodes_self,
            dirty_effective_nodes: &dirty_nodes_effective,
            diagnostics: &diagnostics,
        };

        let measure_timer = Instant::now();
        debug!("Start measuring the component tree...");

        // Call measure_node with &self.tree and &self.metadatas
        // Handle the Result from measure_node
        match measure_node(
            root_node,
            &screen_constraint,
            &self.tree,
            &self.metadatas,
            compute_resource_manager.clone(),
            gpu,
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

        let record_timer = Instant::now();
        record_layout_commands(
            root_node,
            &self.tree,
            &self.metadatas,
            compute_resource_manager.clone(),
            gpu,
        );
        let record_cost = record_timer.elapsed();

        let compute_draw_timer = Instant::now();
        debug!("Start computing render graph...");
        let graph = build_render_graph(root_node, &self.tree, &self.metadatas, screen_size);
        debug!(
            "Render graph built in {:?}, total ops: {}",
            compute_draw_timer.elapsed(),
            graph.ops().len()
        );

        let input_handler_timer = Instant::now();
        let mut window_requests = WindowRequests::default();
        debug!("Start executing input handlers...");

        for node_id in root_node
            .reverse_traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
        {
            let Some(input_handler) = self
                .tree
                .get(node_id)
                .and_then(|n| n.get().input_handler_fn.as_ref())
            else {
                continue;
            };

            let Some(metadata) = self.metadatas.get(&node_id) else {
                warn!(
                    "Input handler metadata missing for node {node_id:?}; skipping input handling"
                );
                continue;
            };
            let Some(abs_pos) = metadata.abs_position else {
                warn!("Absolute position missing for node {node_id:?}; skipping input handling");
                continue;
            };
            let event_clip_rect = metadata.event_clip_rect;
            let node_computed_data = metadata.computed_data;
            drop(metadata); // release DashMap guard so handlers can mutate metadata if needed

            let mut cursor_position_ref = &mut cursor_position;
            let mut dummy_cursor_position = None;
            let mut cursor_events_ref = &mut cursor_events;
            let mut empty_dummy_cursor_events = Vec::new();
            if let (Some(cursor_pos), Some(clip_rect)) = (*cursor_position_ref, event_clip_rect) {
                // check if the cursor is inside the clip rect
                if !clip_rect.contains(cursor_pos) {
                    // If not, set cursor relative inputs to None
                    cursor_position_ref = &mut dummy_cursor_position;
                    cursor_events_ref = &mut empty_dummy_cursor_events;
                }
            }
            let current_cursor_position = cursor_position_ref.map(|pos| pos - abs_pos);

            if let Some(node_computed_data) = node_computed_data {
                let (instance_logic_id, instance_key, fn_name) = self
                    .tree
                    .get(node_id)
                    .map(|n| {
                        let node = n.get();
                        (
                            node.instance_logic_id,
                            node.instance_key,
                            node.fn_name.as_str().to_owned(),
                        )
                    })
                    .unwrap_or_default();
                #[cfg(feature = "profiling")]
                let mut profiler_guard = {
                    let parent_id = self.tree.get(node_id).and_then(|n| n.parent());
                    let fn_name = Some(fn_name.clone());
                    Some(ProfilerScopeGuard::new(
                        ProfilerPhase::Input,
                        Some(node_id),
                        parent_id,
                        fn_name.as_deref(),
                    ))
                };
                let _node_ctx_guard = push_current_node_with_instance_logic_id(
                    node_id,
                    instance_logic_id,
                    fn_name.as_str(),
                );
                let _instance_ctx_guard = push_current_component_instance_key(instance_key);
                let _phase_guard = push_phase(RuntimePhase::Input);
                let input = InputHandlerInput {
                    computed_data: node_computed_data,
                    cursor_position_rel: current_cursor_position,
                    cursor_position_abs: cursor_position_ref,
                    cursor_events: cursor_events_ref,
                    keyboard_events: &mut keyboard_events,
                    ime_events: &mut ime_events,
                    key_modifiers: modifiers,
                    requests: &mut window_requests,
                    current_node_id: node_id,
                    metadatas: &self.metadatas,
                };
                input_handler(input);
                #[cfg(feature = "profiling")]
                {
                    let abs_tuple = (abs_pos.x.0, abs_pos.y.0);
                    if let Some(g) = &mut profiler_guard {
                        g.set_positions(Some(abs_tuple));
                    }
                    let _ = profiler_guard.take();
                }
                // if input_handler set ime request, it's position must be None, and we set it
                // here
                if let Some(ref mut ime_request) = window_requests.ime_request
                    && ime_request.position.is_none()
                {
                    ime_request.position = Some(abs_pos);
                }
            } else {
                warn!(
                    "Computed data not found for node {:?} during input handler execution.",
                    node_id
                );
            }
        }

        debug!(
            "Input Handlers executed in {:?}",
            input_handler_timer.elapsed()
        );
        (
            graph,
            window_requests,
            diagnostics.snapshot(
                dirty_nodes_param,
                dirty_nodes_structural,
                dirty_nodes_effective.len() as u64,
                dirty_expand_ns,
            ),
            record_cost,
        )
    }
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
        let child_keys = node_id
            .children(tree)
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
        node.get().layout_spec.record_dyn(&input);
        stack.extend(node_id.children(tree));
    }
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
    // Get metadata and calculate absolute position. This MUST happen for all nodes.
    let Some(mut metadata) = context.metadatas.get_mut(&node_id) else {
        warn!("Missing metadata for node {node_id:?}; skipping render graph build");
        return;
    };
    let rel_pos = match metadata.rel_position {
        Some(pos) => pos,
        None if is_root => PxPosition::ZERO,
        _ => return, // Skip nodes that were not placed at all.
    };
    let self_pos = start_pos + rel_pos;
    let node_opacity = metadata.opacity;
    let cumulative_opacity = current_opacity * node_opacity;
    metadata.abs_position = Some(self_pos);

    let size = metadata
        .computed_data
        .map(|d| PxSize {
            width: d.width,
            height: d.height,
        })
        .unwrap_or_default();

    let node_rect = PxRect {
        x: self_pos.x,
        y: self_pos.y,
        width: size.width,
        height: size.height,
    };

    let mut clip_rect = clip_rect;
    if let Some(clip_rect) = clip_rect {
        metadata.event_clip_rect = Some(clip_rect);
    }

    let clips_children = metadata.clips_children;
    if clips_children {
        let new_clip_rect = if let Some(existing_clip) = clip_rect {
            existing_clip
                .intersection(&node_rect)
                .unwrap_or(PxRect::ZERO)
        } else {
            node_rect
        };

        clip_rect = Some(new_clip_rect);
        context.builder.push_clip_push(new_clip_rect);
    }

    let fragment = metadata.take_fragment();
    drop(metadata); // Release lock before recursing

    if size.width.0 > 0 && size.height.0 > 0 && !node_rect.is_orthogonal(&context.screen_rect) {
        context
            .builder
            .append_fragment(fragment, size, self_pos, cumulative_opacity);
    }

    for child in node_id.children(context.tree) {
        let parent_abs_pos = match context
            .metadatas
            .get(&node_id)
            .and_then(|meta| meta.abs_position)
        {
            Some(pos) => pos,
            None => {
                warn!(
                    "Missing parent absolute position for node {node_id:?}; skipping child {child:?}"
                );
                continue;
            }
        };

        build_render_graph_inner(
            context,
            parent_abs_pos,
            false,
            child,
            clip_rect,
            cumulative_opacity,
        );
    }

    if clips_children {
        context.builder.push_clip_pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{component_tree::ComponentNode, layout::DefaultLayoutSpec};

    fn node(name: &str, instance_logic_id: u64, instance_key: u64) -> ComponentNode {
        ComponentNode {
            fn_name: name.to_string(),
            component_type_id: instance_logic_id,
            instance_logic_id,
            instance_key,
            input_handler_fn: None,
            layout_spec: Box::new(DefaultLayoutSpec),
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
