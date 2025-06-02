mod basic_drawable;
mod constraint;
mod node;

use std::num::NonZero;

use crate::renderer::DrawCommand;
pub use basic_drawable::BasicDrawable;
pub use constraint::Constraint;
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
    pub fn add_node(&mut self, node: ComponentNode) {
        // Add new node to index tree
        let new_node = self.tree.new_node(node);
        // If there is a current node, append the new node to it
        // And push the new node to the queue to record
        if let Some(current_node) = self.node_queue.last_mut() {
            current_node.append(new_node, &mut self.tree);
        }
        // we also need to add/reset a metadata for the new node
        self.metadatas
            .insert(new_node, ComponentNodeMetaData::none());
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
        // Let components measure and place themselves
        let screen_constraint = Constraint {
            max_width: Some(screen_size[0]),
            max_height: Some(screen_size[1]),
            min_width: None,
            min_height: None,
        };
        measure_node(
            root_node,
            &screen_constraint,
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
    let rel_pos = metadatas
        .get(&node_id)
        .unwrap()
        .rel_position
        .unwrap_or([0, 0]);
    let self_pos = [start_pos[0] + rel_pos[0], start_pos[1] + rel_pos[1]];
    if let Some(drawable) = metadatas.get_mut(&node_id).unwrap().basic_drawable.take() {
        let size = metadatas.get(&node_id).unwrap().computed_data.unwrap();
        let command = drawable.into_draw_command([size.width, size.height], self_pos);
        commands.push(command);
    }

    let children: Vec<_> = node_id.children(tree).collect();
    for child in children {
        compute_draw_commands_inner(self_pos, child, tree, metadatas, commands);
    }
}
