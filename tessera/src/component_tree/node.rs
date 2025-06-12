use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
    time::Instant,
};

use dashmap::DashMap;
use indextree::NodeId;
use log::debug;
use rayon::prelude::*;

use super::{
    basic_drawable::BasicDrawable,
    constraint::{Constraint, DimensionValue},
};
use crate::cursor::CursorEvent;

/// A ComponentNode is a node in the component tree.
/// It represents all information about a component.
pub struct ComponentNode {
    /// Component function's name, for debugging purposes.
    pub fn_name: String,
    /// Describes the component in layout.
    /// None means using default measure policy which places children at the top-left corner
    /// of the parent node, with no offset.
    pub measure_fn: Option<Box<MeasureFn>>,
    /// Describes the state handler for the component.
    /// This is used to handle state changes.
    pub state_handler_fn: Option<Box<StateHandlerFn>>,
}

/// Contains metadata of the component node.
pub struct ComponentNodeMetaData {
    /// The computed data (size) of the node.
    /// None if the node is not computed yet.
    pub computed_data: Option<ComputedData>,
    /// Cached computed data for each constraint applied to this node.
    pub cached_computed_data: HashMap<Constraint, ComputedData>,
    /// The node's start position, relative to its parent.
    /// None if the node is not placed yet.
    pub rel_position: Option<[u32; 2]>,
    /// The node's start position, relative to the root window.
    /// This will be computed during drawing command's generation.
    /// None if the node is not drawn yet.
    pub abs_position: Option<[u32; 2]>,
    /// Optional basic drawable associated with this node.
    pub basic_drawable: Option<BasicDrawable>,
    /// The constraint that this node has intrinsically (e.g., from its arguments).
    /// This is merged with parent's constraint during layout.
    /// Default is Constraint::NONE (Wrap/Wrap).
    pub constraint: Constraint,
}

impl ComponentNodeMetaData {
    /// Creates a new `ComponentNodeMetaData` with default values.
    pub fn none() -> Self {
        Self {
            cached_computed_data: HashMap::new(),
            computed_data: None,
            rel_position: None,
            abs_position: None,
            basic_drawable: None,
            constraint: Constraint::NONE,
        }
    }
}

impl Default for ComponentNodeMetaData {
    fn default() -> Self {
        Self {
            cached_computed_data: HashMap::new(),
            computed_data: None,
            rel_position: None,
            abs_position: None,
            basic_drawable: None,
            constraint: Constraint::NONE, // Default intrinsic constraint
        }
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
pub type MeasureFn = dyn Fn(
        indextree::NodeId,
        &ComponentNodeTree,
        &Constraint,
        &[indextree::NodeId],
        &ComponentNodeMetaDatas,
    ) -> Result<ComputedData, MeasurementError>
    + Send
    + Sync;

/// A `StateHandlerFn` is a function that handles state changes for a component.
pub type StateHandlerFn = dyn Fn(&StateHandlerInput) + Send + Sync;

/// Input for the state handler function (`StateHandlerFn`).
pub struct StateHandlerInput {
    /// The `NodeId` of the component node that this state handler is for.
    /// Usually used to access the component's metadata.
    pub node_id: indextree::NodeId,
    /// The size of the component node, computed during the measure stage.
    pub computed_data: ComputedData,
    /// The position of the cursor, if available.
    /// Relative to the root position of the component.
    pub cursor_position: Option<[i32; 2]>,
    /// Cursor events from the event loop, if any.
    pub cursor_events: Vec<CursorEvent>,
    /// Keyboard events from the event loop, if any.
    pub keyboard_events: Vec<winit::event::KeyEvent>,
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
) -> Result<ComputedData, MeasurementError> {
    let node_data_ref = tree
        .get(node_id)
        .ok_or(MeasurementError::NodeNotFoundInTree)?;
    let node_data = node_data_ref.get();

    let node_intrinsic_constraint = component_node_metadatas
        .get(&node_id)
        .map_or(Constraint::NONE, |m| m.constraint);
    let effective_constraint = node_intrinsic_constraint.merge(parent_constraint);

    if let Some(metadata_entry) = component_node_metadatas.get(&node_id)
        && let Some(cached_size) = metadata_entry
            .cached_computed_data
            .get(&effective_constraint)
    {
        debug!("Cache hit for node {node_id:?} with effective_constraint {effective_constraint:?}");
        return Ok(*cached_size);
    }

    let children: Vec<_> = node_id.children(tree).collect(); // No .as_ref() needed for &Arena
    let timer = Instant::now();
    debug!("Measuring node {}", node_data.fn_name);

    let size = if let Some(measure_fn) = &node_data.measure_fn {
        measure_fn(
            node_id,
            tree,
            parent_constraint, // Should this be effective_constraint? Original code used parent_constraint.
            // Custom measure functions might want the original parent_constraint to decide how to merge.
            // Or they might expect the already merged effective_constraint.
            // For now, keeping parent_constraint as per original logic.
            &children,
            component_node_metadatas,
        )
    } else {
        DEFAULT_LAYOUT_DESC(
            node_id,
            tree,
            &effective_constraint, // Default layout uses the merged constraint.
            &children,
            component_node_metadatas,
        )
    }?;

    debug!(
        "Measured node {} in {:?}",
        node_data.fn_name,
        timer.elapsed()
    );

    let mut metadata = component_node_metadatas.entry(node_id).or_default();
    metadata.computed_data = Some(size);
    metadata
        .cached_computed_data
        .insert(effective_constraint, size);

    Ok(size)
}

/// Places a node at the specified relative position within its parent.
pub fn place_node(
    node: indextree::NodeId,
    rel_position: [u32; 2],
    component_node_metadatas: &ComponentNodeMetaDatas,
) {
    component_node_metadatas
        .entry(node)
        .or_default()
        .rel_position = Some(rel_position);
}

/// A default layout descriptor (`MeasureFn`) that places children at the top-left corner ([0,0])
/// of the parent node with no offset. Children are measured concurrently using `measure_nodes`.
pub const DEFAULT_LAYOUT_DESC: &MeasureFn =
    &|current_node_id, tree, effective_constraint, children_ids, metadatas| {
        if children_ids.is_empty() {
            // If there are no children, the size depends on the effective_constraint
            // For Fixed, it's the fixed size. For Wrap/Fill, it's typically 0 if no content.
            // This part might need refinement based on how min constraints in Wrap/Fill should behave for empty nodes.
            // For now, returning ZERO, assuming intrinsic size of an empty node is zero before min constraints are applied.
            // The actual min size enforcement happens when the parent (or this node itself if it has intrinsic min)
            // considers its own DimensionValue.
            return Ok(ComputedData::min_from_constraint(effective_constraint));
        }

        let nodes_to_measure: Vec<(NodeId, Constraint)> = children_ids
            .iter()
            .map(|&child_id| (child_id, *effective_constraint)) // Children inherit parent's effective constraint
            .collect();

        let children_results_map = measure_nodes(nodes_to_measure, tree, metadatas);

        let mut aggregate_size = ComputedData::ZERO;
        let mut first_error: Option<MeasurementError> = None;
        let mut successful_children_data = Vec::new();

        for &child_id in children_ids {
            match children_results_map.get(&child_id) {
                Some(Ok(child_size)) => {
                    successful_children_data.push((child_id, *child_size));
                }
                Some(Err(e)) => {
                    debug!(
                        "Child node {child_id:?} measurement failed for parent {current_node_id:?}: {e:?}"
                    );
                    if first_error.is_none() {
                        first_error = Some(MeasurementError::ChildMeasurementFailed(child_id));
                    }
                }
                None => {
                    debug!(
                        "Child node {child_id:?} was not found in measure_nodes results for parent {current_node_id:?}"
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
        if successful_children_data.is_empty() && !children_ids.is_empty() {
            // This case should ideally be caught by first_error if all children failed.
            // If it's reached, it implies some logic issue.
            return Err(MeasurementError::MeasureFnFailed(
                "All children failed to measure or results missing in DEFAULT_LAYOUT_DESC"
                    .to_string(),
            ));
        }

        // For default layout (stacking), the aggregate size is the max of children's sizes.
        for (child_id, child_size) in successful_children_data {
            aggregate_size = aggregate_size.max(child_size);
            place_node(child_id, [0, 0], metadatas); // All children at [0,0] for simple stacking
        }

        // The aggregate_size is based on children. Now apply current node's own constraints.
        // If current node is Fixed, its size is fixed.
        // If current node is Wrap, its size is aggregate_size (clamped by its own min/max).
        // If current node is Fill, its size is aggregate_size (clamped by its own min/max, and parent's available space if parent was Fill).
        // This final clamping/adjustment based on `effective_constraint` should ideally happen
        // when `ComputedData` is returned from `measure_node` itself, or by the caller of `measure_node`.
        // For DEFAULT_LAYOUT_DESC, it should return the size required by its children,
        // and then `measure_node` will finalize it based on `effective_constraint`.

        // Let's refine: DEFAULT_LAYOUT_DESC should calculate the "natural" size based on children.
        // Then, `measure_node` (or its caller) would apply the `effective_constraint` to this natural size.
        // However, `measure_node` currently directly returns the result of `DEFAULT_LAYOUT_DESC` or custom `measure_fn`.
        // So, `DEFAULT_LAYOUT_DESC` itself needs to consider `effective_constraint` for its final size.

        let mut final_width = aggregate_size.width;
        let mut final_height = aggregate_size.height;

        match effective_constraint.width {
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
        match effective_constraint.height {
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
) -> HashMap<NodeId, Result<ComputedData, MeasurementError>> {
    if nodes_to_measure.is_empty() {
        return HashMap::new();
    }
    nodes_to_measure
        .into_par_iter()
        .map(|(node_id, parent_constraint)| {
            let result = measure_node(node_id, &parent_constraint, tree, component_node_metadatas);
            (node_id, result)
        })
        .collect::<HashMap<NodeId, Result<ComputedData, MeasurementError>>>()
}

/// Layout information computed at the measure stage, representing the size of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputedData {
    pub width: u32,
    pub height: u32,
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
        width: 0,
        height: 0,
    };

    /// Calculates a "minimum" size based on a constraint.
    /// For Fixed, it's the fixed value. For Wrap/Fill, it's their 'min' if Some, else 0.
    pub fn min_from_constraint(constraint: &Constraint) -> Self {
        let width = match constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(0),
            DimensionValue::Fill { min, .. } => min.unwrap_or(0),
        };
        let height = match constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap { min, .. } => min.unwrap_or(0),
            DimensionValue::Fill { min, .. } => min.unwrap_or(0),
        };
        Self { width, height }
    }

    // Old smallest and largest might not be as relevant with min/max in Wrap/Fill
    // pub fn smallest(constraint: &Constraint) -> Self {
    //     let width = match constraint.width {
    //         DimensionValue::Fixed(w) => w,
    //         DimensionValue::Wrap { min, .. } => min.unwrap_or(0), // Use min from Wrap, or 0
    //         DimensionValue::Fill { min, .. } => min.unwrap_or(0), // Use min from Fill, or 0
    //     };
    //     let height = match constraint.height {
    //         DimensionValue::Fixed(h) => h,
    //         DimensionValue::Wrap { min, .. } => min.unwrap_or(0), // Use min from Wrap, or 0
    //         DimensionValue::Fill { min, .. } => min.unwrap_or(0), // Use min from Fill, or 0
    //     };
    //     Self { width, height }
    // }

    // pub fn largest(constraint: &Constraint) -> Self {
    //     let width = match constraint.width {
    //         DimensionValue::Fixed(w) => w,
    //         DimensionValue::Wrap { max, .. } => max.unwrap_or(u32::MAX), // Use max from Wrap, or u32::MAX
    //         DimensionValue::Fill { max, .. } => max.unwrap_or(u32::MAX),
    //     };
    //     let height = match constraint.height {
    //         DimensionValue::Fixed(h) => h,
    //         DimensionValue::Wrap { max, .. } => max.unwrap_or(u32::MAX), // Use max from Wrap, or u32::MAX
    //         DimensionValue::Fill { max, .. } => max.unwrap_or(u32::MAX),
    //     };
    //     Self { width, height }
    // }

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
