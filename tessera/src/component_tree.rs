mod basic_drawable;
mod node;

use std::num::NonZero;

use crate::renderer::{DrawCommand, write_font_system};
pub use basic_drawable::BasicDrawable;
pub use node::{
    ComponentNode, ComputedData, Constraint, DEFAULT_LAYOUT_DESC, LayoutDescription,
    PositionRelation,
};

/// Contains node and its computed data
struct Node {
    /// The node
    node: ComponentNode,
    /// The computed data
    /// None if the node is not computed yet
    computed_data: Option<ComputedData>,
    /// The node's start position
    position: Option<[u32; 2]>,
}

/// Respents a component tree
pub struct ComponentTree {
    /// We use indextree as the tree structure
    tree: indextree::Arena<Node>,
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
        Self { tree, node_queue }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        self.tree.clear();
        self.node_queue.clear();
    }

    /// Add a new node to the tree
    pub fn add_node(&mut self, node: ComponentNode) {
        // Add new node to index tree
        let node = Node {
            node,
            computed_data: None,
            position: None,
        };
        let new_node = self.tree.new_node(node);
        // If there is a current node, append the new node to it
        // And push the new node to the queue to record
        if let Some(current_node) = self.node_queue.last_mut() {
            current_node.append(new_node, &mut self.tree);
        }
        self.node_queue.push(new_node);
    }

    /// Pop the last node from the queue
    /// This should be called in the end of a component
    /// after all its children nodes are added
    /// to indicate that the component is finished
    pub fn pop_node(&mut self) {
        self.node_queue.pop();
    }

    /// Compute the ComponentTree into a list of DrawCommand
    pub fn compute(&mut self) -> Vec<DrawCommand> {
        // Mesure Stage:
        // Traverse the tree and measure the size of each node
        // From the root node to the leaf node, then compute the size of each node
        let Some(root_node) = self
            .tree
            // indextree use 1 based indexing, so the first element is at 1 and not 0.
            .get_node_id_at(NonZero::new(1).unwrap())
        else {
            return vec![];
        };
        measure_node(root_node, &mut self.tree, None);
        // Placement Stage:
        // Traverse the tree and compute the position of each node
        place_node(root_node, &mut self.tree);
        // In the end, we need to traverse the tree again and get the draw commands.
        compute_draw_commands(root_node, &mut self.tree)
    }
}

/// Measure the size of a node
fn measure_node(
    node: indextree::NodeId,
    tree: &mut indextree::Arena<Node>,
    parent_constraint: Option<Constraint>,
) -> ComputedData {
    // compute final constraint
    let node_constraint = tree.get(node).unwrap().get().node.constraint;
    let final_constraint = match parent_constraint {
        Some(parent_constraint) => node_constraint.merge(&parent_constraint),
        None => node_constraint,
    };
    // then we need to compute size
    let mut computed_data = match &mut tree.get_mut(node).unwrap().get_mut().node.drawable {
        Some(BasicDrawable::Text { data }) => {
            // if the node is a text, we need to apply constraints
            // to the text
            data.resize(
                &mut write_font_system(),
                final_constraint.max_width.map(|width| width as f32),
                final_constraint.max_height.map(|height| height as f32),
            );
            ComputedData {
                width: data.size[0],
                height: data.size[1],
            }
        }
        _ => ComputedData::ZERO,
    };
    let children: Vec<_> = node.children(tree).collect();
    for node in children {
        computed_data += measure_node(node, tree, Some(final_constraint));
    }
    // compute size cannot be smaller than the constraint's min size
    computed_data = computed_data.max(ComputedData::smallest(&final_constraint));
    // compute size cannot be larger than the constraint's max size
    computed_data = computed_data.min(ComputedData::largest(&final_constraint));
    // update measure result
    tree.get_mut(node).unwrap().get_mut().computed_data = Some(computed_data);
    // return to parent
    computed_data
}

/// Compute where to place the node
fn place_node(node: indextree::NodeId, tree: &mut indextree::Arena<Node>) {
    // get current node's position, if it is None, set it to [0, 0](root node)
    let current_pos = tree.get(node).unwrap().get().position.unwrap_or([0, 0]);
    // get all children nodes
    let children: Vec<_> = node.children(tree).collect();
    // get self measured data
    let self_measured_data = tree.get(node).unwrap().get().computed_data.unwrap();
    // get all measured data of children
    let measured_datas: Vec<_> = children
        .iter()
        .map(|node| tree.get(*node).unwrap().get().computed_data.unwrap())
        .collect();
    // get the layout descriptor
    let layout_desc = &tree.get(node).unwrap().get().node.layout_desc;
    // compute the relative position of each child node
    let rel_positions = layout_desc(&self_measured_data, &measured_datas);
    // trans into absolute position
    let abs_positions: Vec<_> = rel_positions
        .into_iter()
        .map(|pos| {
            [
                current_pos[0] + pos.relative_position.offset_x,
                current_pos[1] + pos.relative_position.offset_y,
            ]
        })
        .collect();
    // update the position of each child node
    for (index, child) in children.iter().enumerate() {
        tree.get_mut(*child).unwrap().get_mut().position = Some(abs_positions[index]);
    }
    // ask each child to place its children
    for child in children {
        place_node(child, tree);
    }
}

/// Compute the whole tree to generate draw commands
fn compute_draw_commands(
    node: indextree::NodeId,
    tree: &mut indextree::Arena<Node>,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    // `traverse` is a method that returns an iterator over the tree, depth-first
    let nodes: Vec<_> = node
        .traverse(tree)
        .filter_map(|edge| match edge {
            indextree::NodeEdge::Start(node) => Some(node),
            indextree::NodeEdge::End(_) => None,
        })
        .collect();
    for node in nodes {
        let node = tree.get_mut(node).unwrap().get_mut();
        if let Some(drawable) = node.node.drawable.take() {
            let start_pos = node.position.unwrap_or([0, 0]);
            let size = node.computed_data.unwrap_or(ComputedData::ZERO);
            let command = drawable.into_draw_command([size.width, size.height], start_pos);
            commands.push(command);
        }
    }

    commands
}
