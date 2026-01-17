mod constraint;
mod node;

use std::{num::NonZero, sync::Arc, time::Instant};

use parking_lot::RwLock;
use tracing::{debug, warn};

use crate::{
    ComputeResourceManager, Px, PxRect,
    cursor::CursorEvent,
    layout::RenderInput,
    px::{PxPosition, PxSize},
    render_graph::{RenderGraph, RenderGraphBuilder},
    runtime::{LayoutCache, RuntimePhase, TraceEntry, push_current_node, push_phase},
};

pub use constraint::{Constraint, DimensionValue, ParentConstraint};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    ImeRequest, InputHandlerFn, InputHandlerInput, MeasurementError, WindowRequests,
};

pub(crate) use node::{measure_node, measure_nodes};

#[cfg(feature = "profiling")]
use crate::profiler::{NodeMeta, Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard};

#[derive(Clone, Copy)]
pub(crate) struct LayoutContext<'a> {
    pub cache: &'a dashmap::DashMap<u64, crate::runtime::LayoutCacheEntry>,
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
    pub layout_cache: &'a mut LayoutCache,
    pub frame_trace: Vec<TraceEntry>,
}

/// Respents a component tree
pub struct ComponentTree {
    /// We use indextree as the tree structure
    tree: indextree::Arena<ComponentNode>,
    /// Components' metadatas
    metadatas: ComponentNodeMetaDatas,
    /// Used to remember the current node
    node_queue: Vec<indextree::NodeId>,
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
        }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        self.tree.clear();
        self.metadatas.clear();
        self.node_queue.clear();
    }

    /// Get node by NodeId
    pub fn get(&self, node_id: indextree::NodeId) -> Option<&ComponentNode> {
        self.tree.get(node_id).map(|n| n.get())
    }

    /// Get mutable node by NodeId
    pub fn get_mut(&mut self, node_id: indextree::NodeId) -> Option<&mut ComponentNode> {
        self.tree.get_mut(node_id).map(|n| n.get_mut())
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
    pub(crate) fn compute(&mut self, params: ComputeParams<'_>) -> (RenderGraph, WindowRequests) {
        let ComputeParams {
            screen_size,
            mut cursor_position,
            mut cursor_events,
            mut keyboard_events,
            mut ime_events,
            modifiers,
            compute_resource_manager,
            gpu,
            layout_cache,
            frame_trace: _frame_trace,
        } = params;
        let Some(root_node) = self
            .tree
            .get_node_id_at(NonZero::new(1).expect("root node index must be non-zero"))
        else {
            return (RenderGraph::default(), WindowRequests::default());
        };
        let screen_constraint = Constraint::new(
            DimensionValue::Fixed(screen_size.width),
            DimensionValue::Fixed(screen_size.height),
        );

        let layout_ctx = LayoutContext {
            cache: &layout_cache.entries,
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

        record_layout_commands(
            root_node,
            &self.tree,
            &self.metadatas,
            compute_resource_manager.clone(),
            gpu,
        );

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
                let logic_id = self
                    .tree
                    .get(node_id)
                    .map(|n| n.get().logic_id)
                    .unwrap_or_default();
                #[cfg(feature = "profiling")]
                let mut profiler_guard = {
                    let parent_id = self.tree.get(node_id).and_then(|n| n.parent());
                    let fn_name = self
                        .tree
                        .get(node_id)
                        .map(|n| n.get().fn_name.as_str().to_owned());
                    Some(ProfilerScopeGuard::new(
                        ProfilerPhase::Input,
                        Some(node_id),
                        parent_id,
                        fn_name.as_deref(),
                    ))
                };
                let fn_name = self
                    .tree
                    .get(node_id)
                    .map(|n| n.get().fn_name.as_str())
                    .unwrap_or("");
                let _node_ctx_guard = push_current_node(node_id, logic_id, fn_name);
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
        (graph, window_requests)
    }
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
