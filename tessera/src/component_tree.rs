mod basic_drawable;
mod constraint;
mod node;

use std::{num::NonZero, time::Instant};

use glyphon::cosmic_text::ttf_parser::apple_layout::state;
use log::debug;
use rayon::prelude::*;

use crate::{component_tree::node::StateHandlerInput, cursor::CursorEvent, renderer::DrawCommand};
pub use basic_drawable::{BasicDrawable, ShadowProps};
pub use constraint::{Constraint, DimensionValue};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    MeasureFn, StateHandlerFn, measure_node, place_node,
};

/// Respents a component tree
pub struct ComponentTree {
    /// We use indextree as the tree structure
    tree: indextree::Arena<ComponentNode>,
    /// Components' metadatas
    metadatas: ComponentNodeMetaDatas,
    /// Used to remember the current node
    node_queue: Vec<indextree::NodeId>,
    /// The ID of the node that currently has focus
    pub focused_node_id: Option<indextree::NodeId>,
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
            focused_node_id: None,
        }
    }

    /// Clear the component tree
    pub fn clear(&mut self) {
        self.tree.clear();
        self.metadatas.clear();
        self.node_queue.clear();
        self.focused_node_id = None;
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
    pub fn compute(
        &mut self,
        screen_size: [u32; 2],
        cursor_events: Vec<CursorEvent>,
    ) -> Vec<DrawCommand> {
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

        // timer for measurement cost
        let measure_timer = Instant::now();
        debug!("Start measuring the component tree...");
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
        debug!("Component tree measured in {:?}", measure_timer.elapsed());
        // Traverse the tree again and get the draw commands.
        // Timer for draw commands computation
        let compute_draw_timer = Instant::now();
        debug!("Start computing draw commands...");
        let commands =
            compute_draw_commands_parallel(root_node, &mut self.tree, &mut self.metadatas);
        debug!(
            "Draw commands computed in {:?}, total commands: {}",
            compute_draw_timer.elapsed(),
            commands.len()
        );
        // After gen all drawing commands, we can execute state handlers for the whole tree.
        // This is beause some event such as mouse click cannot be ensured where it happens
        // until the whole tree is measured.
        // Timer for state handler execution
        let state_handler_timer = Instant::now();
        debug!("Start executing state handlers...");
        for node in root_node
            .reverse_traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(node_id) => Some(node_id),
                indextree::NodeEdge::End(_) => None,
            })
        {
            // Get the state handler function for the node, if it exists
            // we do it first to skip unnecessary computation
            // if there is no state handler function at all.
            let Some(state_handler) = self
                .tree
                .get(node)
                .and_then(|n| n.get().state_handler_fn.as_ref())
            else {
                continue;
            };
            // transform the cursor events to set their position
            // relative to the node's position
            let cursor_events = cursor_events
                .iter()
                .cloned()
                .map(|mut event| {
                    // Get the node's absolute position
                    let abs_position = self
                        .metadatas
                        .get(&node)
                        .and_then(|m| m.abs_position)
                        .unwrap_or([0, 0]); // Default to [0, 0] if not set
                    // Set the cursor event's position relative to the node
                    event.content = event.content.into_relative_position(abs_position);
                    event
                })
                .collect::<Vec<_>>();
            // Create the input for the state handler
            let input = StateHandlerInput {
                node_id: node,
                computed_data: self
                    .metadatas
                    .get(&node)
                    .and_then(|m| m.computed_data) // Get the computed data for the node
                    .unwrap(), // Should always exist after measure
                cursor_events,
            };
            // Call the state handler function with the input
            state_handler(&input);
        }
        debug!(
            "State handlers executed in {:?}",
            state_handler_timer.elapsed()
        );
        // Return the computed draw commands
        commands
    }
}

fn compute_draw_commands_parallel(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) -> Vec<DrawCommand> {
    compute_draw_commands_inner_parallel([0, 0], node_id, tree, metadatas)
}

fn compute_draw_commands_inner_parallel(
    start_pos: [u32; 2],
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) -> Vec<DrawCommand> {
    let mut local_commands = Vec::new();

    if let Some(mut entry) = metadatas.get_mut(&node_id) {
        let rel_pos = entry.rel_position.unwrap_or([0, 0]);
        let self_pos = [start_pos[0] + rel_pos[0], start_pos[1] + rel_pos[1]];
        entry.abs_position = Some(self_pos);

        if let Some(drawable) = entry.basic_drawable.take() {
            let size = entry.computed_data.unwrap();
            let command = drawable.into_draw_command([size.width, size.height], self_pos);
            local_commands.push(command);
        }

        drop(entry);

        let children: Vec<_> = node_id.children(tree).collect();
        let child_results: Vec<Vec<DrawCommand>> = children
            .into_par_iter()
            .map(|child| compute_draw_commands_inner_parallel(self_pos, child, tree, metadatas))
            .collect();

        for child_cmds in child_results {
            local_commands.extend(child_cmds);
        }
    }

    local_commands
}
