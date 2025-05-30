use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use indextree::NodeId;

use super::{basic_drawable::BasicDrawable, constraint::Constraint};
/// A ComponentNode is a node in the component tree.
/// It represents all information about a component:
pub struct ComponentNode {
    /// Describes the component in layout
    /// None means using default measure policy
    /// which does nothing but places children at the top-left corner
    /// of the parent node, with no offset
    pub measure_fn: Option<Box<MeasureFn>>,
}

/// Contains metadata of the component node
pub struct ComponentNodeMetaData {
    /// The computed data
    /// None if the node is not computed yet
    pub computed_data: Option<ComputedData>,
    /// The node's start position, relative to its parent
    /// None if the node is not placed yet
    pub rel_position: Option<[u32; 2]>,
    /// Optional basic drawable
    pub basic_drawable: Option<BasicDrawable>,
}

impl ComponentNodeMetaData {
    pub const NONE: Self = Self {
        computed_data: None,
        rel_position: None,
        basic_drawable: None,
    };
}

/// A tree of component nodes
pub type ComponentNodeTree = indextree::Arena<ComponentNode>;
/// Contains all component nodes's metadatas
pub type ComponentNodeMetaDatas = HashMap<NodeId, ComponentNodeMetaData>;
/// A MeasureFn is a function that takes input `Constraint` and its children nodes
/// finish placementing inside and return its size(`ComputedData`)
pub type MeasureFn = dyn Fn(
        indextree::NodeId,
        &ComponentNodeTree,
        &Constraint,
        &[indextree::NodeId],
        &mut ComponentNodeMetaDatas,
    ) -> ComputedData
    + Send
    + Sync;

/// Measure a node recursively, return its size
pub fn measure_node(
    node: indextree::NodeId,
    constraint: &Constraint,
    tree: &ComponentNodeTree,
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) -> ComputedData {
    let children: Vec<_> = node.children(tree).collect();
    let size = if let Some(measure_fn) = &tree.get(node).unwrap().get().measure_fn {
        // Use the specific measure function if it exists
        measure_fn(node, tree, constraint, &children, component_node_metadatas)
    } else {
        // Use default measure function if no specific measure function is provided
        DEFAULT_LAYOUT_DESC(node, tree, constraint, &children, component_node_metadatas)
    };

    if let Some(metadata) = component_node_metadatas.get_mut(&node) {
        // Update the existing metadata with the computed size
        metadata.computed_data = Some(size);
    } else {
        // If metadata does not exist, create a new one
        component_node_metadatas.insert(
            node,
            ComponentNodeMetaData {
                computed_data: Some(size),
                rel_position: None,
                basic_drawable: None,
            },
        );
    }

    size
}

/// Place node at spec relative position
pub fn place_node(
    node: indextree::NodeId,
    rel_position: [u32; 2],
    component_node_metadatas: &mut ComponentNodeMetaDatas,
) {
    if let Some(metadata) = component_node_metadatas.get_mut(&node) {
        metadata.rel_position = Some(rel_position);
    } else {
        component_node_metadatas.insert(
            node,
            ComponentNodeMetaData {
                computed_data: None,
                rel_position: Some(rel_position),
                basic_drawable: None,
            },
        );
    }
}

/// A default layout descriptor that does nothing but places children at the top-left corner
/// of the parent node, with no offset
const DEFAULT_LAYOUT_DESC: &MeasureFn = &|_, tree, constraint, children, metadatas| {
    let mut size = ComputedData::ZERO;
    for child in children {
        let child_size = measure_node(*child, constraint, tree, metadatas);
        size = size.max(child_size);
        place_node(*child, [0, 0], metadatas);
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
    pub fn smallest(constraint: &Constraint) -> Self {
        Self {
            width: constraint.min_width.unwrap_or(0),
            height: constraint.min_height.unwrap_or(0),
        }
    }

    /// Generate a largest size for spec constraint
    pub fn largest(constraint: &Constraint) -> Self {
        Self {
            width: constraint.max_width.unwrap_or(u32::MAX),
            height: constraint.max_height.unwrap_or(u32::MAX),
        }
    }

    /// Returns the minimum of two computed data
    /// Impl trait `Ord` or `PartialOrd` does not help
    /// since we need both width and height to be minnimum
    pub fn min(self, rhs: Self) -> Self {
        Self {
            width: self.width.min(rhs.width),
            height: self.height.min(rhs.height),
        }
    }

    /// Returns the maximum of two computed data
    /// Impl trait `Ord` or `PartialOrd` does not help
    /// since we need both width and height to be maximum
    pub fn max(self, rhs: Self) -> Self {
        Self {
            width: self.width.max(rhs.width),
            height: self.height.max(rhs.height),
        }
    }
}
