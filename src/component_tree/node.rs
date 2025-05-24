use std::ops::{Add, AddAssign};

use super::basic_drawable::BasicDrawable;

/// A ComponentNode is a node in the component tree.
/// It represents all information about a component:
pub struct ComponentNode {
    /// Describes the component in layout
    pub layout_desc: Box<LayoutDescriptor>,
    /// Describes the constraints of the component
    pub constraint: Constraint,
    /// what to draw by Drawer(optional)
    pub drawable: Option<BasicDrawable>,
}

/// A LayoutDescriptor is a function that takes a slice of ComputedData
/// it describes how to layout the children of a node
/// and returns a vector of LayoutDescription, which describes
/// their relative position to parent node itself
pub type LayoutDescriptor = dyn Fn(&[ComputedData]) -> Vec<LayoutDescription>;

/// A default layout descriptor that does nothing but places children at the top-left corner
/// of the parent node, with no offset
pub const DEFAULT_LAYOUT_DESC: &LayoutDescriptor = &|_| {
    vec![LayoutDescription {
        relative_position: PositionRelation {
            offset_x: 0,
            offset_y: 0,
        },
    }]
};

pub struct LayoutDescription {
    /// Describes position of the child node relative to parent node
    pub relative_position: PositionRelation,
}

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
            width: constraint.min_width,
            height: constraint.min_height,
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

/// Describes constraints
#[derive(Debug, Clone, Copy)]
pub struct Constraint {
    /// max width(pixels)
    pub max_width: u32,
    /// min width(pixels)
    pub min_width: u32,
    /// max height(pixels)
    pub max_height: u32,
    /// min height(pixels)
    pub min_height: u32,
}

impl Constraint {
    /// Merge parent constraint and self constraint
    /// Parent constraint should always override self constraint
    /// if it's stricter
    pub fn merge(&self, parent: &Self) -> Self {
        Self {
            // width cannot be bigger than parent's max width
            max_width: self.max_width.min(parent.max_width),
            min_width: self.min_width.min(parent.max_width),
            // height cannot be bigger than parent's max height
            max_height: self.max_height.min(parent.max_height),
            min_height: self.min_height.min(parent.max_height),
        }
    }
}

/// Describes the position of the node to another node's
pub struct PositionRelation {
    /// offset at x axis
    pub offset_x: u32,
    /// offset at y axis
    pub offset_y: u32,
}
