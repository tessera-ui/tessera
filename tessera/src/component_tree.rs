mod basic_drawable;
mod constraint;
mod node;

use std::num::NonZero;

use crate::renderer::DrawCommand;
pub use basic_drawable::{BasicDrawable, ShadowProps};
pub use constraint::{Constraint, DimensionValue}; // Added DimensionValue
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    MeasureFn, measure_node, place_node,
};

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
    /// The `node_component` itself primarily holds the measure_fn.
    pub fn add_node(&mut self, node_component: ComponentNode, intrinsic_constraint: Constraint) {
        // Add new node to index tree
        let new_node_id = self.tree.new_node(node_component);
        // If there is a current node, append the new node to it
        if let Some(current_node_id) = self.node_queue.last_mut() {
            current_node_id.append(new_node_id, &mut self.tree);
        }
        // Add/reset metadata for the new node, including its intrinsic constraint
        let mut metadata = ComponentNodeMetaData::none();
        metadata.constraint = intrinsic_constraint; // Store the node's own constraint
        self.metadatas.insert(new_node_id, metadata);
        self.node_queue.push(new_node_id);
    }

    /// Pop the last node from the queue
    /// This should be called in the end of a component
    /// after all its children nodes are added
    /// to indicate that the component is finished
    pub fn pop_node(&mut self) {
        self.node_queue.pop();
    }

    /// Compute the ComponentTree into a list of DrawCommand
    pub fn compute(&mut self, screen_size: [u32; 2]) -> Vec<DrawCommand> {
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
        // The root node is constrained by the screen size.
        let screen_constraint = Constraint::new(
            DimensionValue::Fixed(screen_size[0]),
            DimensionValue::Fixed(screen_size[1]),
        );

        // The root node's intrinsic constraint (if any, e.g. from App component's args)
        // should also be considered. For now, assume root's intrinsic is Constraint::NONE
        // or it's handled by the root component's measure function if it has one.
        // If the root component (e.g. the main `app` function's surface) specifies Fill,
        // it will correctly merge with this screen_constraint.
        measure_node(
            root_node,
            &screen_constraint, // This is the parent_constraint for the root node
            &self.tree,
            &mut self.metadatas,
        );
        // In the end, we need to traverse the tree again and get the draw commands.
        compute_draw_commands(root_node, &mut self.tree, &mut self.metadatas)
    }
}

/// Compute the whole tree to generate draw commands
fn compute_draw_commands(
    node_id: indextree::NodeId,
    tree: &mut ComponentNodeTree,
    metadatas: &mut ComponentNodeMetaDatas,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    compute_draw_commands_inner(
        [0, 0], // Start position is [0, 0] for the root node
        node_id,
        tree,
        metadatas,
        &mut commands,
    );
    commands
}

/// Inner function to compute draw commands recursively
fn compute_draw_commands_inner(
    start_pos: [u32; 2],
    node_id: indextree::NodeId,
    tree: &mut ComponentNodeTree,
    metadatas: &mut ComponentNodeMetaDatas,
    commands: &mut Vec<DrawCommand>,
) {
    let metadata_entry = metadatas.get_mut(&node_id).unwrap(); // Should always exist after measure

    let rel_pos = metadata_entry.rel_position.unwrap_or([0, 0]);
    let self_pos = [start_pos[0] + rel_pos[0], start_pos[1] + rel_pos[1]];

    if let Some(drawable) = metadata_entry.basic_drawable.take() {
        let size = metadata_entry.computed_data.unwrap(); // Should exist after measure
        let command = drawable.into_draw_command([size.width, size.height], self_pos);
        commands.push(command);
    }

    let children: Vec<_> = node_id.children(tree).collect();
    for child in children {
        compute_draw_commands_inner(self_pos, child, tree, metadatas, commands);
    }
}
