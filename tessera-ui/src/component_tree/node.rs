use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Add, AddAssign},
    sync::Arc,
    time::Instant,
};

use dashmap::DashMap;
use indextree::NodeId;
use parking_lot::RwLock;
use rayon::prelude::*;
use smallvec::SmallVec;
use tracing::debug;
use winit::window::CursorIcon;

use crate::{
    Clipboard, ComputeCommand, ComputeResourceManager, DrawCommand, Px,
    accessibility::{AccessibilityActionHandler, AccessibilityNode, AccessibilityPadding},
    cursor::CursorEvent,
    dp::Dp,
    layout::{LayoutInput, LayoutOutput, LayoutResult, LayoutSpecDyn},
    px::{PxPosition, PxSize},
    renderer::Command,
    runtime::{LayoutCacheEntry, RuntimePhase, push_current_node, push_phase},
};

use super::{
    LayoutContext,
    constraint::{Constraint, DimensionValue, ParentConstraint},
};

#[cfg(feature = "profiling")]
use crate::profiler::{Phase as ProfilerPhase, ScopeGuard as ProfilerScopeGuard};

/// A guard that manages accessibility node building and automatically
/// commits the result to the metadata when dropped.
pub struct AccessibilityBuilderGuard<'a> {
    node_id: NodeId,
    metadatas: &'a ComponentNodeMetaDatas,
    node: AccessibilityNode,
}

impl<'a> AccessibilityBuilderGuard<'a> {
    /// Sets the role of this node.
    pub fn role(mut self, role: accesskit::Role) -> Self {
        self.node.role = Some(role);
        self
    }

    /// Sets the label of this node.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.node.label = Some(label.into());
        self
    }

    /// Sets the description of this node.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.node.description = Some(description.into());
        self
    }

    /// Sets the value of this node.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.node.value = Some(value.into());
        self
    }

    /// Sets the numeric value of this node.
    pub fn numeric_value(mut self, value: f64) -> Self {
        self.node.numeric_value = Some(value);
        self
    }

    /// Sets the numeric range of this node.
    pub fn numeric_range(mut self, min: f64, max: f64) -> Self {
        self.node.min_numeric_value = Some(min);
        self.node.max_numeric_value = Some(max);
        self
    }

    /// Marks this node as focusable.
    pub fn focusable(mut self) -> Self {
        self.node.focusable = true;
        self
    }

    /// Marks this node as focused.
    pub fn focused(mut self) -> Self {
        self.node.focused = true;
        self
    }

    /// Sets the toggled state of this node.
    pub fn toggled(mut self, toggled: accesskit::Toggled) -> Self {
        self.node.toggled = Some(toggled);
        self
    }

    /// Marks this node as disabled.
    pub fn disabled(mut self) -> Self {
        self.node.disabled = true;
        self
    }

    /// Marks this node as hidden from accessibility.
    pub fn hidden(mut self) -> Self {
        self.node.hidden = true;
        self
    }

    /// Adds an action that this node supports.
    pub fn action(mut self, action: accesskit::Action) -> Self {
        self.node.actions.push(action);
        self
    }

    /// Adds multiple actions that this node supports.
    pub fn actions(mut self, actions: impl IntoIterator<Item = accesskit::Action>) -> Self {
        self.node.actions.extend(actions);
        self
    }

    /// Sets a custom accessibility key for stable ID generation.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.node.key = Some(key.into());
        self
    }

    /// Sets a state description announced with the control.
    pub fn state_description(mut self, description: impl Into<String>) -> Self {
        self.node.state_description = Some(description.into());
        self
    }

    /// Sets a custom role description for custom controls.
    pub fn role_description(mut self, description: impl Into<String>) -> Self {
        self.node.role_description = Some(description.into());
        self
    }

    /// Sets tooltip text.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.node.tooltip = Some(tooltip.into());
        self
    }

    /// Sets the live region politeness.
    pub fn live(mut self, live: accesskit::Live) -> Self {
        self.node.live = Some(live);
        self
    }

    /// Marks this node as a heading with an optional level (1-based).
    pub fn heading_level(mut self, level: u32) -> Self {
        self.node.heading_level = Some(level);
        self
    }

    /// Sets scroll X value and range.
    pub fn scroll_x(mut self, value: f64, min: f64, max: f64) -> Self {
        self.node.scroll_x = Some((value, min, max));
        self
    }

    /// Sets scroll Y value and range.
    pub fn scroll_y(mut self, value: f64, min: f64, max: f64) -> Self {
        self.node.scroll_y = Some((value, min, max));
        self
    }

    /// Sets the numeric step value for range-based controls.
    pub fn numeric_value_step(mut self, step: f64) -> Self {
        self.node.numeric_value_step = Some(step);
        self
    }

    /// Sets the numeric jump value for range-based controls.
    pub fn numeric_value_jump(mut self, jump: f64) -> Self {
        self.node.numeric_value_jump = Some(jump);
        self
    }

    /// Sets collection info: (row_count, column_count, hierarchical).
    pub fn collection_info(mut self, rows: usize, cols: usize, hierarchical: bool) -> Self {
        self.node.collection_info = Some((rows, cols, hierarchical));
        self
    }

    /// Sets collection item info: (row_index, row_span, col_index, col_span,
    /// heading).
    pub fn collection_item_info(
        mut self,
        row_index: usize,
        row_span: usize,
        col_index: usize,
        col_span: usize,
        heading: bool,
    ) -> Self {
        self.node.collection_item_info = Some((row_index, row_span, col_index, col_span, heading));
        self
    }

    /// Marks this node as editable text.
    pub fn editable_text(mut self, editable: bool) -> Self {
        self.node.is_editable_text = editable;
        self
    }

    /// Prevents child semantics from being merged, similar to Compose's
    /// `clearAndSetSemantics`.
    pub fn clear_and_set(mut self) -> Self {
        self.node.merge_descendants = false;
        self
    }

    /// Expands the semantic bounds by a fixed pixel padding.
    pub fn bounds_padding_px(mut self, left: Px, top: Px, right: Px, bottom: Px) -> Self {
        self.node.bounds_padding = Some(AccessibilityPadding {
            left,
            top,
            right,
            bottom,
        });
        self
    }

    /// Expands the semantic bounds by a density-independent padding.
    pub fn bounds_padding_dp(mut self, left: Dp, top: Dp, right: Dp, bottom: Dp) -> Self {
        self.node.bounds_padding = Some(AccessibilityPadding {
            left: left.into(),
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
        });
        self
    }

    /// Sets a testing tag by assigning a stable accessibility key.
    pub fn test_tag(mut self, tag: impl Into<String>) -> Self {
        self.node.key = Some(tag.into());
        self
    }

    /// Explicitly commits the accessibility information.
    pub fn commit(self) {
        // The Drop impl will handle the actual commit
        drop(self);
    }
}

impl Drop for AccessibilityBuilderGuard<'_> {
    fn drop(&mut self) {
        // Copy the accessibility data to metadata
        if let Some(mut metadata) = self.metadatas.get_mut(&self.node_id) {
            metadata.accessibility = Some(self.node.clone());
        }
    }
}

/// A ComponentNode is a node in the component tree.
/// It represents all information about a component.
pub struct ComponentNode {
    /// Component function's name, for debugging purposes.
    pub fn_name: String,
    /// Stable logic identifier for the component function.
    pub logic_id: u64,
    /// Stable instance identifier for this node in the current frame.
    pub instance_key: u64,
    /// Describes the input handler for the component.
    /// This is used to handle state changes.
    pub input_handler_fn: Option<Box<InputHandlerFn>>,
    /// Pure layout spec for skipping and record passes.
    pub layout_spec: Box<dyn LayoutSpecDyn>,
}

/// Contains metadata of the component node.
pub struct ComponentNodeMetaData {
    /// The computed data (size) of the node.
    /// None if the node is not computed yet.
    pub computed_data: Option<ComputedData>,
    /// Whether the layout cache was hit for this node in the current frame.
    pub layout_cache_hit: bool,
    /// The node's start position, relative to its parent.
    /// None if the node is not placed yet.
    pub rel_position: Option<PxPosition>,
    /// The node's start position, relative to the root window.
    /// This will be computed during drawing command's generation.
    /// None if the node is not drawn yet.
    pub abs_position: Option<PxPosition>,
    /// The effective clipping rectangle for this node, considering all its
    /// ancestors. This is calculated once per frame before event handling.
    pub event_clip_rect: Option<crate::PxRect>,
    /// Commands associated with this node.
    ///
    /// This stores both draw and compute commands in a unified vector using the
    /// new `Command` enum. Commands are collected during the measure phase and
    /// executed during rendering. The order of commands in this vector
    /// determines their execution order.
    pub(crate) commands: SmallVec<[(Command, TypeId); 4]>,
    /// Whether this node clips its children.
    pub clips_children: bool,
    /// Opacity multiplier applied to this node and its descendants.
    pub opacity: f32,
    /// Accessibility information for this node.
    pub accessibility: Option<AccessibilityNode>,
    /// Handler for accessibility actions on this node.
    pub accessibility_action_handler: Option<AccessibilityActionHandler>,
}

impl ComponentNodeMetaData {
    /// Creates a new `ComponentNodeMetaData` with default values.
    pub fn none() -> Self {
        Self {
            computed_data: None,
            layout_cache_hit: false,
            rel_position: None,
            abs_position: None,
            event_clip_rect: None,
            commands: SmallVec::new(),
            clips_children: false,
            opacity: 1.0,
            accessibility: None,
            accessibility_action_handler: None,
        }
    }

    /// Pushes a draw command to the node's metadata.
    ///
    /// Draw commands are responsible for rendering visual content (shapes,
    /// text, images). This method wraps the command in the unified
    /// `Command::Draw` variant and adds it to the command queue. Commands
    /// are executed in the order they are added.
    pub fn push_draw_command<C: DrawCommand + 'static>(&mut self, command: C) {
        let command = Box::new(command);
        let command = command as Box<dyn DrawCommand>;
        let command = Command::Draw(command);
        self.commands.push((command, TypeId::of::<C>()));
    }

    /// Pushes a compute command to the node's metadata.
    ///
    /// Compute commands perform GPU computation tasks (post-processing effects,
    /// complex calculations). This method wraps the command in the unified
    /// `Command::Compute` variant and adds it to the command queue.
    pub fn push_compute_command<C: ComputeCommand + 'static>(&mut self, command: C) {
        let command = Box::new(command);
        let command = command as Box<dyn ComputeCommand>;
        let command = Command::Compute(command);
        self.commands.push((command, TypeId::of::<C>()));
    }
}

impl Default for ComponentNodeMetaData {
    fn default() -> Self {
        Self::none()
    }
}

/// A tree of component nodes, using `indextree::Arena` for storage.
pub type ComponentNodeTree = indextree::Arena<ComponentNode>;
/// Contains all component nodes' metadatas, using a thread-safe `DashMap`.
pub type ComponentNodeMetaDatas = DashMap<NodeId, ComponentNodeMetaData>;

/// Represents errors that can occur during node measurement.
#[derive(Debug, Clone, PartialEq)]
pub enum MeasurementError {
    /// Indicates that the specified node was not found in the component tree.
    NodeNotFoundInTree,
    /// Indicates that metadata for the specified node was not found (currently
    /// not a primary error source in measure_node).
    NodeNotFoundInMeta,
    /// Indicates that the layout spec for a node failed. Contains a string
    /// detailing the failure.
    MeasureFnFailed(String),
    /// Indicates that the measurement of a child node failed during a
    /// parent's layout calculation. Contains the `NodeId` of the child
    /// that failed.
    ChildMeasurementFailed(NodeId),
}

/// A `InputHandlerFn` is a function that handles state changes for a component.
///
/// The rule of execution order is:
///
/// 1. Children's input handlers are executed earlier than parent's.
/// 2. Newer components' input handlers are executed earlier than older ones.
///
/// Acutally, rule 2 includes rule 1, because a newer component is always a
/// child of an older component :)
pub type InputHandlerFn = dyn Fn(InputHandlerInput) + Send + Sync;

/// Input for the input handler function (`InputHandlerFn`).
///
/// Note that you can modify the `cursor_events` and `keyboard_events` vectors
/// for exmaple block some keyboard events or cursor events to prevent them from
/// propagating to parent components and older brother components.
pub struct InputHandlerInput<'a> {
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// The position of the cursor, if available.
    /// Relative to the root position of the component.
    pub cursor_position_rel: Option<PxPosition>,
    /// The mut ref of absolute position of the cursor in the window.
    /// Used to block cursor fully if needed, since cursor_position_rel use
    /// this. Not a public field for now.
    pub(crate) cursor_position_abs: &'a mut Option<PxPosition>,
    /// Cursor events from the event loop, if any.
    pub cursor_events: &'a mut Vec<CursorEvent>,
    /// Keyboard events from the event loop, if any.
    pub keyboard_events: &'a mut Vec<winit::event::KeyEvent>,
    /// IME events from the event loop, if any.
    pub ime_events: &'a mut Vec<winit::event::Ime>,
    /// The current state of the keyboard modifiers at the time of the event.
    /// This allows for implementing keyboard shortcuts (e.g., Ctrl+C).
    pub key_modifiers: winit::keyboard::ModifiersState,
    /// A context for making requests to the window for the current frame.
    pub requests: &'a mut WindowRequests,
    /// Clipboard
    pub clipboard: &'a mut Clipboard,
    /// The current node ID (for accessibility setup)
    pub(crate) current_node_id: indextree::NodeId,
    /// Reference to component metadatas (for accessibility setup)
    pub(crate) metadatas: &'a ComponentNodeMetaDatas,
}

impl InputHandlerInput<'_> {
    /// Blocks the cursor to other components.
    pub fn block_cursor(&mut self) {
        // Block the cursor by setting its position to None.
        self.cursor_position_abs.take();
        // Clear all cursor events to prevent them from propagating.
        self.cursor_events.clear();
    }

    /// Blocks the keyboard events to other components.
    pub fn block_keyboard(&mut self) {
        // Clear all keyboard events to prevent them from propagating.
        self.keyboard_events.clear();
    }

    /// Blocks the IME events to other components.
    pub fn block_ime(&mut self) {
        // Clear all IME events to prevent them from propagating.
        self.ime_events.clear();
    }

    /// Block all events (cursor, keyboard, IME) to other components.
    pub fn block_all(&mut self) {
        self.block_cursor();
        self.block_keyboard();
        self.block_ime();
    }

    /// Provides a fluent API for setting accessibility information for the
    /// current component.
    ///
    /// This method returns a builder that allows you to set various
    /// accessibility properties like role, label, actions, and state. The
    /// accessibility information is automatically committed when the
    /// builder is dropped or when `.commit()` is called explicitly.
    ///
    /// # Example
    ///
    /// ```
    /// use accesskit::{Action, Role};
    /// use tessera_ui::tessera;
    ///
    /// #[tessera]
    /// fn accessible_button() {
    ///     input_handler(|input| {
    ///         input
    ///             .accessibility()
    ///             .role(Role::Button)
    ///             .label("Click me")
    ///             .focusable()
    ///             .action(Action::Click);
    ///
    ///         // Handle clicks...
    ///     });
    /// }
    /// ```
    ///
    /// Note: The builder should be committed with `.commit()` or allowed to
    /// drop, which will automatically store the accessibility information
    /// in the metadata.
    pub fn accessibility(&self) -> AccessibilityBuilderGuard<'_> {
        AccessibilityBuilderGuard {
            node_id: self.current_node_id,
            metadatas: self.metadatas,
            node: AccessibilityNode::new(),
        }
    }

    /// Sets an action handler for accessibility actions.
    ///
    /// This handler will be called when assistive technologies request actions
    /// like clicking, focusing, or changing values.
    ///
    /// # Example
    ///
    /// ```
    /// use accesskit::Action;
    /// use tessera_ui::tessera;
    ///
    /// #[tessera]
    /// fn interactive_button() {
    ///     input_handler(|input| {
    ///         input.set_accessibility_action_handler(|action| {
    ///             if action == Action::Click {
    ///                 // Handle click from assistive technology
    ///             }
    ///         });
    ///     });
    /// }
    /// ```
    pub fn set_accessibility_action_handler(
        &self,
        handler: impl Fn(accesskit::Action) + Send + Sync + 'static,
    ) {
        if let Some(mut metadata) = self.metadatas.get_mut(&self.current_node_id) {
            metadata.accessibility_action_handler = Some(Box::new(handler));
        }
    }
}

/// A collection of requests that components can make to the windowing system
/// for the current frame. This struct's lifecycle is confined to a single
/// `compute` pass.
#[derive(Default, Debug)]
pub struct WindowRequests {
    /// The cursor icon requested by a component. If multiple components request
    /// a cursor, the last one to make a request in a frame "wins", since
    /// it's executed later.
    pub cursor_icon: CursorIcon,
    /// An Input Method Editor (IME) request.
    /// If multiple components request IME, the one from the "newer" component
    /// (which is processed later in the state handling pass) will overwrite
    /// previous requests.
    pub ime_request: Option<ImeRequest>,
}

/// A request to the windowing system to open an Input Method Editor (IME).
/// This is typically used for text input components.
#[derive(Debug)]
pub struct ImeRequest {
    /// The size of the area where the IME is requested.
    pub size: PxSize,
    /// The absolute position where the IME should be placed.
    /// This is set internally by the component tree during the compute pass.
    pub(crate) position: Option<PxPosition>, // should be setted in tessera node tree compute
}

impl ImeRequest {
    /// Creates a new IME request with the target input area size.
    ///
    /// The absolute position is injected during the compute pass.
    pub fn new(size: PxSize) -> Self {
        Self {
            size,
            position: None, // Position will be set during the compute phase
        }
    }
}

fn apply_layout_placements(
    placements: &[(u64, PxPosition)],
    tree: &ComponentNodeTree,
    children: &[NodeId],
    component_node_metadatas: &ComponentNodeMetaDatas,
) {
    if placements.is_empty() || children.is_empty() {
        return;
    }
    let mut child_map = HashMap::new();
    for child_id in children {
        if let Some(child) = tree.get(*child_id) {
            child_map.insert(child.get().instance_key, *child_id);
        }
    }
    for (instance_key, position) in placements {
        if let Some(child_id) = child_map.get(instance_key) {
            place_node(*child_id, *position, component_node_metadatas);
        }
    }
}

/// Measures a single node recursively, returning its size or an error.
///
/// See [`measure_nodes`] for concurrent measurement of multiple nodes.
/// Which is very recommended for most cases. You should only use this function
/// when your're very sure that you only need to measure a single node.
pub(crate) fn measure_node(
    node_id: NodeId,
    parent_constraint: &Constraint,
    tree: &ComponentNodeTree,
    component_node_metadatas: &ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
    layout_ctx: Option<&LayoutContext<'_>>,
) -> Result<ComputedData, MeasurementError> {
    let node_data_ref = tree
        .get(node_id)
        .ok_or(MeasurementError::NodeNotFoundInTree)?;
    let node_data = node_data_ref.get();
    #[cfg(feature = "profiling")]
    let mut profiler_guard = Some(ProfilerScopeGuard::new(
        ProfilerPhase::Measure,
        Some(node_id),
        node_data_ref.parent(),
        Some(node_data.fn_name.as_str()),
    ));

    let children: Vec<_> = node_id.children(tree).collect(); // No .as_ref() needed for &Arena
    let timer = Instant::now();

    debug!(
        "Measuring node {} with {} children, parent constraint: {:?}",
        node_data.fn_name,
        children.len(),
        parent_constraint
    );

    // Ensure thread-local current node context for nested control-flow
    // instrumentation.
    let _node_ctx_guard =
        push_current_node(node_id, node_data.logic_id, node_data.fn_name.as_str());
    let _phase_guard = push_phase(RuntimePhase::Measure);

    let resolve_instance_key = |child_id: NodeId| {
        if let Some(child) = tree.get(child_id) {
            child.get().instance_key
        } else {
            debug_assert!(
                false,
                "Child node must exist when resolving layout placements"
            );
            0
        }
    };
    let child_keys: Vec<u64> = children
        .iter()
        .map(|&child_id| resolve_instance_key(child_id))
        .collect();

    let layout_spec = &node_data.layout_spec;
    if let Some(layout_ctx) = layout_ctx
        && let Some(entry) = layout_ctx.cache.get(&node_data.instance_key)
    {
        let frame_gap = layout_ctx.frame_index.saturating_sub(entry.last_seen_frame);
        if frame_gap > 1 {
            drop(entry);
            layout_ctx.cache.remove(&node_data.instance_key);
        } else {
            let can_try_reuse = entry.constraint_key == *parent_constraint
                && entry.layout_spec.dyn_eq(layout_spec.as_ref())
                && entry.child_keys == child_keys
                && entry.child_constraints.len() == child_keys.len()
                && entry.child_sizes.len() == child_keys.len();
            if can_try_reuse {
                let cached_constraints = entry.child_constraints.clone();
                let cached_sizes = entry.child_sizes.clone();
                let cached_result = entry.layout_result.clone();
                drop(entry);

                let nodes_to_measure = children
                    .iter()
                    .zip(cached_constraints.iter())
                    .map(|(child_id, constraint)| (*child_id, *constraint))
                    .collect();
                let measured = measure_nodes(
                    nodes_to_measure,
                    tree,
                    component_node_metadatas,
                    compute_resource_manager.clone(),
                    gpu,
                    Some(layout_ctx),
                );
                let mut sizes_match = true;
                for (index, child_id) in children.iter().enumerate() {
                    let Some(result) = measured.get(child_id) else {
                        sizes_match = false;
                        break;
                    };
                    match result {
                        Ok(size) => {
                            if *size != cached_sizes[index] {
                                sizes_match = false;
                                break;
                            }
                        }
                        Err(err) => return Err(err.clone()),
                    }
                }
                if sizes_match {
                    component_node_metadatas.insert(node_id, Default::default());
                    apply_layout_placements(
                        &cached_result.placements,
                        tree,
                        &children,
                        component_node_metadatas,
                    );
                    if let Some(mut metadata) = component_node_metadatas.get_mut(&node_id) {
                        metadata.computed_data = Some(cached_result.size);
                        metadata.layout_cache_hit = true;
                    }
                    if let Some(mut entry) = layout_ctx.cache.get_mut(&node_data.instance_key) {
                        entry.last_seen_frame = layout_ctx.frame_index;
                    }
                    return Ok(cached_result.size);
                }
            }
        }
    }

    component_node_metadatas.insert(node_id, Default::default());
    let input = LayoutInput::new(
        tree,
        ParentConstraint::new(parent_constraint),
        &children,
        component_node_metadatas,
        compute_resource_manager,
        gpu,
        layout_ctx,
    );
    let mut output = LayoutOutput::new(&resolve_instance_key);
    let size = layout_spec.measure_dyn(&input, &mut output)?;
    let measured_children = input.take_measured_children();
    let placements = output.finish();
    apply_layout_placements(&placements, tree, &children, component_node_metadatas);

    component_node_metadatas
        .entry(node_id)
        .or_default()
        .computed_data = Some(size);

    #[cfg(feature = "profiling")]
    if let Some(guard) = &mut profiler_guard {
        guard.set_computed_size(size.width.0, size.height.0);
    }

    if let Some(layout_ctx) = layout_ctx {
        let mut cacheable = true;
        let mut child_constraints = Vec::with_capacity(children.len());
        let mut child_sizes = Vec::with_capacity(children.len());
        for child_id in &children {
            let Some(measurement) = measured_children.get(child_id) else {
                cacheable = false;
                break;
            };
            if !measurement.consistent {
                cacheable = false;
                break;
            }
            child_constraints.push(measurement.constraint);
            child_sizes.push(measurement.size);
        }
        if cacheable {
            let layout_result = LayoutResult { size, placements };
            layout_ctx.cache.insert(
                node_data.instance_key,
                LayoutCacheEntry {
                    constraint_key: *parent_constraint,
                    layout_spec: layout_spec.clone(),
                    layout_result,
                    child_keys,
                    child_constraints,
                    child_sizes,
                    last_seen_frame: layout_ctx.frame_index,
                },
            );
        } else {
            layout_ctx.cache.remove(&node_data.instance_key);
        }
    }

    debug!(
        "Measured node {} in {:?} with size {:?}",
        node_data.fn_name,
        timer.elapsed(),
        size
    );

    #[cfg(feature = "profiling")]
    if let Some(guard) = &mut profiler_guard {
        guard.set_computed_size(size.width.0, size.height.0);
    }

    Ok(size)
}

/// Places a node at the specified relative position within its parent.
pub(crate) fn place_node(
    node: indextree::NodeId,
    rel_position: PxPosition,
    component_node_metadatas: &ComponentNodeMetaDatas,
) {
    component_node_metadatas
        .entry(node)
        .or_default()
        .rel_position = Some(rel_position);
}

/// Concurrently measures multiple nodes using Rayon for parallelism.
pub(crate) fn measure_nodes(
    nodes_to_measure: Vec<(NodeId, Constraint)>,
    tree: &ComponentNodeTree,
    component_node_metadatas: &ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
    layout_ctx: Option<&LayoutContext<'_>>,
) -> HashMap<NodeId, Result<ComputedData, MeasurementError>> {
    if nodes_to_measure.is_empty() {
        return HashMap::new();
    }
    // metadata must be reseted and initialized for each node to measure.
    for (node_id, _) in &nodes_to_measure {
        component_node_metadatas.insert(*node_id, Default::default());
    }
    nodes_to_measure
        .into_par_iter()
        .map(|(node_id, parent_constraint)| {
            let result = measure_node(
                node_id,
                &parent_constraint,
                tree,
                component_node_metadatas,
                compute_resource_manager.clone(),
                gpu,
                layout_ctx,
            );
            (node_id, result)
        })
        .collect::<HashMap<NodeId, Result<ComputedData, MeasurementError>>>()
}

/// Layout information computed at the measure stage, representing the size of a
/// node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputedData {
    /// The resolved width of the node in physical pixels.
    pub width: Px,
    /// The resolved height of the node in physical pixels.
    pub height: Px,
}

impl Add for ComputedData {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl AddAssign for ComputedData {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl ComputedData {
    /// Zero-sized layout data.
    pub const ZERO: Self = Self {
        width: Px(0),
        height: Px(0),
    };

    /// Calculates a "minimum" size based on a constraint.
    /// For Fixed, it's the fixed value. For Wrap/Fill, it's their 'min' if
    /// Some, else 0.
    pub fn min_from_constraint(constraint: &Constraint) -> Self {
        let width = match constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(Px(0)),
            DimensionValue::Fill { min, .. } => min.unwrap_or(Px(0)),
        };
        let height = match constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(Px(0)),
            DimensionValue::Fill { min, .. } => min.unwrap_or(Px(0)),
        };
        Self { width, height }
    }

    /// Returns the component-wise minimum of two computed sizes.
    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: self.width.min(rhs.width),
            height: self.height.min(rhs.height),
        }
    }

    /// Returns the component-wise maximum of two computed sizes.
    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: self.width.max(rhs.width),
            height: self.height.max(rhs.height),
        }
    }
}
