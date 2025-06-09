use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::time::Instant;

use dashmap::DashMap;
use indextree::NodeId;
use log::debug;

use crate::cursor::CursorEvent;

use super::{
    basic_drawable::BasicDrawable,
    constraint::{Constraint, DimensionValue},
};
/// A ComponentNode is a node in the component tree.
/// It represents all information about a component:
pub struct ComponentNode {
    /// Component function's name
    /// for debugging purposes
    pub fn_name: String,
    /// Describes the component in layout
    /// None means using default measure policy
    /// which does nothing but places children at the top-left corner
    /// of the parent node, with no offset
    pub measure_fn: Option<Box<MeasureFn>>,
    /// Describes the state handler for the component
    /// This is used to handle state changes
    pub state_handler_fn: Option<Box<StateHandlerFn>>,
}

/// Contains metadata of the component node
pub struct ComponentNodeMetaData {
    /// The computed data
    /// None if the node is not computed yet
    pub computed_data: Option<ComputedData>,
    /// Cached computed data for each constraint
    pub cached_computed_data: HashMap<Constraint, ComputedData>,
    /// The node's start position, relative to its parent
    /// None if the node is not placed yet
    pub rel_position: Option<[u32; 2]>,
    /// The node's start position, relative to root window
    /// This will be computed during drawing command's generation
    /// None if the node is not drawn yet
    pub abs_position: Option<[u32; 2]>,
    /// Optional basic drawable
    pub basic_drawable: Option<BasicDrawable>,
    /// The constraint that this node has intrinsically (e.g. from its arguments)
    /// This is merged with parent's constraint during layout.
    /// Default is Constraint::NONE (Wrap/Wrap)
    pub constraint: Constraint,
}

impl ComponentNodeMetaData {
    pub fn none() -> Self {
        // This function can be kept or removed if Default is preferred
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

/// A tree of component nodes
pub type ComponentNodeTree = indextree::Arena<ComponentNode>;
/// Contains all component nodes's metadatas
pub type ComponentNodeMetaDatas = DashMap<NodeId, ComponentNodeMetaData>;
/// A MeasureFn is a function that takes input `Constraint` and its children nodes
/// finish placementing inside and return its size(`ComputedData`)
pub type MeasureFn = dyn Fn(
        indextree::NodeId,
        &ComponentNodeTree,
        &Constraint, // This is the constraint from the parent
        &[indextree::NodeId],
        &mut ComponentNodeMetaDatas,
    ) -> ComputedData
    + Send
    + Sync;
/// A StateHandlerFn is a function that takes input `NodeId` as the node's id
/// and `ComputedData` as the computed data(size) of the node.
/// It is used to handle state changes
pub type StateHandlerFn = dyn Fn(&StateHandlerInput) + Send + Sync;

/// The input for state handler function
pub struct StateHandlerInput {
    /// The node's id
    pub node_id: indextree::NodeId,
    /// The computed data of the node
    pub computed_data: ComputedData,
    /// The cursor events
    pub cursor_events: Vec<CursorEvent>,
}

/// Measure a node recursively, return its size
pub fn measure_node(
    node: indextree::NodeId,
    parent_constraint: &Constraint,
    tree: &ComponentNodeTree,
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) -> ComputedData {
    let node_intrinsic_constraint = component_node_metadatas
        .get(&node)
        .map_or(Constraint::NONE, |m| m.constraint);
    let effective_constraint = node_intrinsic_constraint.merge(parent_constraint);

    if let Some(metadata) = component_node_metadatas.get(&node)
        && let Some(cached_size) = metadata.cached_computed_data.get(&effective_constraint)
    {
        debug!("Cache hit for node {node:?} with effective_constraint {effective_constraint:?}");
        return *cached_size;
    }

    let children: Vec<_> = node.children(tree).collect();
    // Timer for this measure function
    let timer = Instant::now();
    debug!("Measuring node {}", tree.get(node).unwrap().get().fn_name);
    let size = if let Some(measure_fn) = &tree.get(node).unwrap().get().measure_fn {
        // Pass the original parent_constraint; the measure_fn is responsible for
        // using its own args to determine its intrinsic behavior and merge.
        measure_fn(
            node,
            tree,
            parent_constraint,
            &children,
            component_node_metadatas,
        )
    } else {
        // DEFAULT_LAYOUT_DESC uses the fully effective_constraint
        DEFAULT_LAYOUT_DESC(
            node,
            tree,
            &effective_constraint,
            &children,
            component_node_metadatas,
        )
    };
    debug!(
        "Measured node {} in {:?}",
        tree.get(node).unwrap().get().fn_name,
        timer.elapsed(),
    );

    // Ensure metadata exists before trying to update cache or computed_data
    let mut metadata = component_node_metadatas.entry(node).or_default();
    metadata.computed_data = Some(size);
    metadata
        .cached_computed_data
        .insert(effective_constraint, size);

    size
}

/// Place node at spec relative position
pub fn place_node(
    node: indextree::NodeId,
    rel_position: [u32; 2],
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) {
    component_node_metadatas
        .entry(node) // Use entry to ensure metadata exists
        .or_default()
        .rel_position = Some(rel_position);
}

/// A default layout descriptor that does nothing but places children at the top-left corner
/// of the parent node, with no offset.
/// Children are measured with the parent's effective constraint.
const DEFAULT_LAYOUT_DESC: &MeasureFn =
    &|_current_node_id, tree, effective_constraint, children, metadatas| {
        let mut size = ComputedData::ZERO;
        for child_node_id in children {
            let child_size = measure_node(*child_node_id, effective_constraint, tree, metadatas);
            size = size.max(child_size);
            place_node(*child_node_id, [0, 0], metadatas);
        }
        size
    };

/// Layout info computed at measure Stage
#[derive(Debug, Clone, Copy)]
pub struct ComputedData {
    /// The width of the node
    pub width: u32,
    /// The height of the node
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
    /// Zero size - Represent a node that has no size
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

    /// Generate a smallest size for spec constraint
    /// This typically means Fixed values are respected, Wrap/Fill are 0 if no content.
    pub fn smallest(constraint: &Constraint) -> Self {
        let width = match constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap => 0,
            DimensionValue::Fill { .. } => 0,
        };
        let height = match constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap => 0,
            DimensionValue::Fill { .. } => 0,
        };
        Self { width, height }
    }

    /// Generate a largest size for spec constraint
    /// This typically means Fixed values are respected, Wrap/Fill take u32::MAX or their own max.
    pub fn largest(constraint: &Constraint) -> Self {
        let width = match constraint.width {
            DimensionValue::Fixed(w) => w,
            DimensionValue::Wrap => u32::MAX,
            DimensionValue::Fill { max } => max.unwrap_or(u32::MAX),
        };
        let height = match constraint.height {
            DimensionValue::Fixed(h) => h,
            DimensionValue::Wrap => u32::MAX,
            DimensionValue::Fill { max } => max.unwrap_or(u32::MAX),
        };
        Self { width, height }
    }

    /// Returns the minimum of two computed data
    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: self.width.min(rhs.width),
            height: self.height.min(rhs.height),
        }
    }

    /// Returns the maximum of two computed data
    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: self.width.max(rhs.width),
            height: self.height.max(rhs.height),
        }
    }
}
