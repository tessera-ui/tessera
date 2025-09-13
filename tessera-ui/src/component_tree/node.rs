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
use tracing::debug;
use winit::window::CursorIcon;

use crate::{
    Clipboard, ComputeCommand, ComputeResourceManager, DrawCommand, Px,
    cursor::CursorEvent,
    px::{PxPosition, PxSize},
    renderer::Command,
};

use super::constraint::{Constraint, DimensionValue};

/// A ComponentNode is a node in the component tree.
/// It represents all information about a component.
pub struct ComponentNode {
    /// Component function's name, for debugging purposes.
    pub fn_name: String,
    /// Describes the component in layout.
    /// None means using default measure policy which places children at the top-left corner
    /// of the parent node, with no offset.
    pub measure_fn: Option<Box<MeasureFn>>,
    /// Describes the input handler for the component.
    /// This is used to handle state changes.
    pub input_handler_fn: Option<Box<InputHandlerFn>>,
}

/// Contains metadata of the component node.
#[derive(Default)]
pub struct ComponentNodeMetaData {
    /// The computed data (size) of the node.
    /// None if the node is not computed yet.
    pub computed_data: Option<ComputedData>,
    /// The node's start position, relative to its parent.
    /// None if the node is not placed yet.
    pub rel_position: Option<PxPosition>,
    /// The node's start position, relative to the root window.
    /// This will be computed during drawing command's generation.
    /// None if the node is not drawn yet.
    pub abs_position: Option<PxPosition>,
    /// Commands associated with this node.
    ///
    /// This stores both draw and compute commands in a unified vector using the
    /// new `Command` enum. Commands are collected during the measure phase and
    /// executed during rendering. The order of commands in this vector determines
    /// their execution order.
    pub(crate) commands: Vec<(Command, TypeId)>,
    /// Whether this node clips its children.
    pub clips_children: bool,
}

impl ComponentNodeMetaData {
    /// Creates a new `ComponentNodeMetaData` with default values.
    pub fn none() -> Self {
        Self {
            computed_data: None,
            rel_position: None,
            abs_position: None,
            commands: Vec::new(),
            clips_children: false,
        }
    }

    /// Pushes a draw command to the node's metadata.
    ///
    /// Draw commands are responsible for rendering visual content (shapes, text, images).
    /// This method wraps the command in the unified `Command::Draw` variant and adds it
    /// to the command queue. Commands are executed in the order they are added.
    ///
    /// # Example
    /// ```rust,ignore
    /// metadata.push_draw_command(ShapeCommand::Rect {
    ///     color: [1.0, 0.0, 0.0, 1.0],
    ///     corner_radius: 8.0,
    ///     shadow: None,
    /// });
    /// ```
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
    ///
    /// # Example
    /// ```rust,ignore
    /// metadata.push_compute_command(BlurCommand {
    ///     radius: 5.0,
    ///     sigma: 2.0,
    /// });
    /// ```
    pub fn push_compute_command<C: ComputeCommand + 'static>(&mut self, command: C) {
        let command = Box::new(command);
        let command = command as Box<dyn ComputeCommand>;
        let command = Command::Compute(command);
        self.commands.push((command, TypeId::of::<C>()));
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
    /// Indicates that metadata for the specified node was not found (currently not a primary error source in measure_node).
    NodeNotFoundInMeta,
    /// Indicates that the custom measure function (`MeasureFn`) for a node failed.
    /// Contains a string detailing the failure.
    MeasureFnFailed(String),
    /// Indicates that the measurement of a child node failed during a parent's layout calculation (e.g., in `DEFAULT_LAYOUT_DESC`).
    /// Contains the `NodeId` of the child that failed.
    ChildMeasurementFailed(NodeId),
}

/// A `MeasureFn` is a function that takes an input `Constraint` and its children nodes,
/// finishes placementing inside, and returns its size (`ComputedData`) or an error.
pub type MeasureFn =
    dyn Fn(&MeasureInput<'_>) -> Result<ComputedData, MeasurementError> + Send + Sync;

/// Input for the measure function (`MeasureFn`).
pub struct MeasureInput<'a> {
    /// The `NodeId` of the current node being measured.
    pub current_node_id: indextree::NodeId,
    /// The component tree containing all nodes.
    pub tree: &'a ComponentNodeTree,
    /// The effective constraint for this node, merged with its parent's constraint.
    pub parent_constraint: &'a Constraint,
    /// The children nodes of the current node.
    pub children_ids: &'a [indextree::NodeId],
    /// Metadata for all component nodes, used to access cached data and constraints.
    pub metadatas: &'a ComponentNodeMetaDatas,
    /// Compute resources manager
    pub compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    /// Gpu device
    pub gpu: &'a wgpu::Device,
}

impl<'a> MeasureInput<'a> {
    /// Returns a mutable reference to the metadata of the current node.
    ///
    /// This is a convenience method that simplifies accessing the current node's metadata
    /// from within a `measure` function. It encapsulates the `DashMap::get_mut` call and panics
    /// if the metadata is not found, as it's an invariant that it must exist.
    pub fn metadata_mut(&self) -> dashmap::mapref::one::RefMut<'_, NodeId, ComponentNodeMetaData> {
        self.metadatas
            .get_mut(&self.current_node_id)
            .expect("Metadata for current node must exist during measure")
    }

    /// Measures all specified child nodes under the given constraint.
    ///
    /// Returns a map of each child's computed layout data, or the first measurement error encountered.
    pub fn measure_children(
        &self,
        nodes_to_measure: Vec<(NodeId, Constraint)>,
    ) -> Result<HashMap<NodeId, ComputedData>, MeasurementError> {
        let results = measure_nodes(
            nodes_to_measure,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
        );

        let mut successful_results = HashMap::new();
        for (child_id, result) in results {
            match result {
                Ok(size) => successful_results.insert(child_id, size),
                Err(e) => {
                    debug!("Measurement error for child {child_id:?}: {e:?}");
                    return Err(e);
                }
            };
        }
        Ok(successful_results)
    }

    /// Measures a single child node under the given constraint.
    ///
    /// Returns the computed layout data or a measurement error.
    pub fn measure_child(
        &self,
        child_id: NodeId,
        constraint: &Constraint,
    ) -> Result<ComputedData, MeasurementError> {
        measure_node(
            child_id,
            constraint,
            self.tree,
            self.metadatas,
            self.compute_resource_manager.clone(),
            self.gpu,
        )
    }

    /// Sets the relative position of a child node.
    pub fn place_child(&self, child_id: NodeId, position: PxPosition) {
        place_node(child_id, position, self.metadatas);
    }

    /// Enables clipping for the current node.
    pub fn enable_clipping(&self) {
        // Set the clipping flag to true for this node.
        self.metadata_mut().clips_children = true;
    }

    /// Disables clipping for the current node.
    pub fn disable_clipping(&self) {
        // Set the clipping flag to false for this node.
        self.metadata_mut().clips_children = false;
    }
}

/// A `InputHandlerFn` is a function that handles state changes for a component.
///
/// The rule of execution order is:
///
/// 1. Children's input handlers are executed earlier than parent's.
/// 2. Newer components' input handlers are executed earlier than older ones.
///
/// Acutally, rule 2 includes rule 1, because a newer component is always a child of an older component :)
pub type InputHandlerFn = dyn Fn(InputHandlerInput) + Send + Sync;

/// Input for the input handler function (`InputHandlerFn`).
///
/// Note that you can modify the `cursor_events` and `keyboard_events` vectors
/// for exmaple block some keyboard events or cursor events to prevent them from propagating
/// to parent components and older brother components.
pub struct InputHandlerInput<'a> {
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// The position of the cursor, if available.
    /// Relative to the root position of the component.
    pub cursor_position_rel: Option<PxPosition>,
    /// The mut ref of absolute position of the cursor in the window.
    /// Used to block cursor fully if needed, since cursor_position_rel use this.
    /// Not a public field for now.
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
}

/// A collection of requests that components can make to the windowing system for the current frame.
/// This struct's lifecycle is confined to a single `compute` pass.
#[derive(Default, Debug)]
pub struct WindowRequests {
    /// The cursor icon requested by a component. If multiple components request a cursor,
    /// the last one to make a request in a frame "wins", since it's executed later.
    pub cursor_icon: CursorIcon,
    /// An Input Method Editor (IME) request.
    /// If multiple components request IME, the one from the "newer" component (which is
    /// processed later in the state handling pass) will overwrite previous requests.
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
    pub fn new(size: PxSize) -> Self {
        Self {
            size,
            position: None, // Position will be set during the compute phase
        }
    }
}

/// Measures a single node recursively, returning its size or an error.
///
/// See [`measure_nodes`] for concurrent measurement of multiple nodes.
/// Which is very recommended for most cases. You should only use this function
/// when your're very sure that you only need to measure a single node.
pub fn measure_node(
    node_id: NodeId,
    parent_constraint: &Constraint,
    tree: &ComponentNodeTree,
    component_node_metadatas: &ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
) -> Result<ComputedData, MeasurementError> {
    // Make sure metadata and default value exists for the node.
    component_node_metadatas.insert(node_id, Default::default());

    let node_data_ref = tree
        .get(node_id)
        .ok_or(MeasurementError::NodeNotFoundInTree)?;
    let node_data = node_data_ref.get();

    let children: Vec<_> = node_id.children(tree).collect(); // No .as_ref() needed for &Arena
    let timer = Instant::now();

    debug!(
        "Measuring node {} with {} children, parent constraint: {:?}",
        node_data.fn_name,
        children.len(),
        parent_constraint
    );

    let size = if let Some(measure_fn) = &node_data.measure_fn {
        measure_fn(&MeasureInput {
            current_node_id: node_id,
            tree,
            parent_constraint,
            children_ids: &children,
            metadatas: component_node_metadatas,
            compute_resource_manager,
            gpu,
        })
    } else {
        DEFAULT_LAYOUT_DESC(&MeasureInput {
            current_node_id: node_id,
            tree,
            parent_constraint,
            children_ids: &children,
            metadatas: component_node_metadatas,
            compute_resource_manager,
            gpu,
        })
    }?;

    debug!(
        "Measured node {} in {:?} with size {:?}",
        node_data.fn_name,
        timer.elapsed(),
        size
    );

    let mut metadata = component_node_metadatas.entry(node_id).or_default();
    metadata.computed_data = Some(size);

    Ok(size)
}

/// Places a node at the specified relative position within its parent.
pub fn place_node(
    node: indextree::NodeId,
    rel_position: PxPosition,
    component_node_metadatas: &ComponentNodeMetaDatas,
) {
    component_node_metadatas
        .entry(node)
        .or_default()
        .rel_position = Some(rel_position);
}

/// A default layout descriptor (`MeasureFn`) that places children at the top-left corner ([0,0])
/// of the parent node with no offset. Children are measured concurrently using `measure_nodes`.
pub const DEFAULT_LAYOUT_DESC: &MeasureFn = &|input| {
    if input.children_ids.is_empty() {
        // If there are no children, the size depends on the parent_constraint
        // For Fixed, it's the fixed size. For Wrap/Fill, it's typically 0 if no content.
        // This part might need refinement based on how min constraints in Wrap/Fill should behave for empty nodes.
        // For now, returning ZERO, assuming intrinsic size of an empty node is zero before min constraints are applied.
        // The actual min size enforcement happens when the parent (or this node itself if it has intrinsic min)
        // considers its own DimensionValue.
        return Ok(ComputedData::min_from_constraint(input.parent_constraint));
    }

    let nodes_to_measure: Vec<(NodeId, Constraint)> = input
        .children_ids
        .iter()
        .map(|&child_id| (child_id, *input.parent_constraint)) // Children inherit parent's effective constraint
        .collect();

    let children_results_map = measure_nodes(
        nodes_to_measure,
        input.tree,
        input.metadatas,
        input.compute_resource_manager.clone(),
        input.gpu,
    );

    let mut aggregate_size = ComputedData::ZERO;
    let mut first_error: Option<MeasurementError> = None;
    let mut successful_children_data = Vec::new();

    for &child_id in input.children_ids {
        match children_results_map.get(&child_id) {
            Some(Ok(child_size)) => {
                successful_children_data.push((child_id, *child_size));
            }
            Some(Err(e)) => {
                debug!(
                    "Child node {child_id:?} measurement failed for parent {:?}: {e:?}",
                    input.current_node_id
                );
                if first_error.is_none() {
                    first_error = Some(MeasurementError::ChildMeasurementFailed(child_id));
                }
            }
            None => {
                debug!(
                    "Child node {child_id:?} was not found in measure_nodes results for parent {:?}",
                    input.current_node_id
                );
                if first_error.is_none() {
                    first_error = Some(MeasurementError::MeasureFnFailed(format!(
                        "Result for child {child_id:?} missing"
                    )));
                }
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }
    if successful_children_data.is_empty() && !input.children_ids.is_empty() {
        // This case should ideally be caught by first_error if all children failed.
        // If it's reached, it implies some logic issue.
        return Err(MeasurementError::MeasureFnFailed(
            "All children failed to measure or results missing in DEFAULT_LAYOUT_DESC".to_string(),
        ));
    }

    // For default layout (stacking), the aggregate size is the max of children's sizes.
    for (child_id, child_size) in successful_children_data {
        aggregate_size = aggregate_size.max(child_size);
        place_node(child_id, PxPosition::ZERO, input.metadatas); // All children at [0,0] for simple stacking
    }

    // The aggregate_size is based on children. Now apply current node's own constraints.
    // If current node is Fixed, its size is fixed.
    // If current node is Wrap, its size is aggregate_size (clamped by its own min/max).
    // If current node is Fill, its size is aggregate_size (clamped by its own min/max, and parent's available space if parent was Fill).
    // This final clamping/adjustment based on `parent_constraint` should ideally happen
    // when `ComputedData` is returned from `measure_node` itself, or by the caller of `measure_node`.
    // For DEFAULT_LAYOUT_DESC, it should return the size required by its children,
    // and then `measure_node` will finalize it based on `parent_constraint`.

    // Let's refine: DEFAULT_LAYOUT_DESC should calculate the "natural" size based on children.
    // Then, `measure_node` (or its caller) would apply the `parent_constraint` to this natural size.
    // However, `measure_node` currently directly returns the result of `DEFAULT_LAYOUT_DESC` or custom `measure_fn`.
    // So, `DEFAULT_LAYOUT_DESC` itself needs to consider `parent_constraint` for its final size.

    let mut final_width = aggregate_size.width;
    let mut final_height = aggregate_size.height;

    match input.parent_constraint.width {
        DimensionValue::Fixed(w) => final_width = w,
        DimensionValue::Wrap { min, max } => {
            if let Some(min_w) = min {
                final_width = final_width.max(min_w);
            }
            if let Some(max_w) = max {
                final_width = final_width.min(max_w);
            }
        }
        DimensionValue::Fill { min, max } => {
            // Fill behaves like wrap for default layout unless children expand
            if let Some(min_w) = min {
                final_width = final_width.max(min_w);
            }
            if let Some(max_w) = max {
                final_width = final_width.min(max_w);
            }
            // If parent was Fill, this node would have gotten a Fill constraint too.
            // The actual "filling" happens because children might be Fill.
            // If children are not Fill, this node wraps them.
        }
    }
    match input.parent_constraint.height {
        DimensionValue::Fixed(h) => final_height = h,
        DimensionValue::Wrap { min, max } => {
            if let Some(min_h) = min {
                final_height = final_height.max(min_h);
            }
            if let Some(max_h) = max {
                final_height = final_height.min(max_h);
            }
        }
        DimensionValue::Fill { min, max } => {
            if let Some(min_h) = min {
                final_height = final_height.max(min_h);
            }
            if let Some(max_h) = max {
                final_height = final_height.min(max_h);
            }
        }
    }
    Ok(ComputedData {
        width: final_width,
        height: final_height,
    })
};

/// Concurrently measures multiple nodes using Rayon for parallelism.
pub fn measure_nodes(
    nodes_to_measure: Vec<(NodeId, Constraint)>,
    tree: &ComponentNodeTree,
    component_node_metadatas: &ComponentNodeMetaDatas,
    compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    gpu: &wgpu::Device,
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
            );
            (node_id, result)
        })
        .collect::<HashMap<NodeId, Result<ComputedData, MeasurementError>>>()
}

/// Layout information computed at the measure stage, representing the size of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputedData {
    pub width: Px,
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
    pub const ZERO: Self = Self {
        width: Px(0),
        height: Px(0),
    };

    /// Calculates a "minimum" size based on a constraint.
    /// For Fixed, it's the fixed value. For Wrap/Fill, it's their 'min' if Some, else 0.
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

    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: self.width.min(rhs.width),
            height: self.height.min(rhs.height),
        }
    }

    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: self.width.max(rhs.width),
            height: self.height.max(rhs.height),
        }
    }
}
