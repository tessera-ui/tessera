mod constraint;
mod node;

use std::{num::NonZero, time::Instant};

use log::{debug, error};
use rayon::prelude::*;

use crate::{
    cursor::CursorEvent,
    px::{PxPosition, PxSize},
    renderer::DrawCommand,
};
pub use constraint::{Constraint, DimensionValue};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    MeasureFn, MeasurementError, StateHandlerFn, StateHandlerInput, WindowRequests, measure_node,
    measure_nodes, place_node,
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
    pub fn add_node(&mut self, node_component: ComponentNode) {
        let new_node_id = self.tree.new_node(node_component);
        if let Some(current_node_id) = self.node_queue.last_mut() {
            current_node_id.append(new_node_id, &mut self.tree);
        }
        let metadata = ComponentNodeMetaData::none();
        self.metadatas.insert(new_node_id, metadata);
        self.node_queue.push(new_node_id);
    }

    /// Pop the last node from the queue
    pub fn pop_node(&mut self) {
        self.node_queue.pop();
    }

    /// Compute the ComponentTree into a list of DrawCommand
    pub fn compute(
        &mut self,
        screen_size: PxSize,
        cursor_position: Option<PxPosition>,
        mut cursor_events: Vec<CursorEvent>,
        mut keyboard_events: Vec<winit::event::KeyEvent>,
    ) -> (
        Vec<(PxPosition, PxSize, Box<dyn DrawCommand>)>,
        WindowRequests,
    ) {
        let Some(root_node) = self.tree.get_node_id_at(NonZero::new(1).unwrap()) else {
            return (vec![], WindowRequests::default());
        };
        let screen_constraint = Constraint::new(
            DimensionValue::Fixed(screen_size.width),
            DimensionValue::Fixed(screen_size.height),
        );

        let measure_timer = Instant::now();
        debug!("Start measuring the component tree...");

        // Call measure_node with &self.tree and &self.metadatas
        // Handle the Result from measure_node
        match measure_node(root_node, &screen_constraint, &self.tree, &self.metadatas) {
            Ok(_root_computed_data) => {
                debug!("Component tree measured in {:?}", measure_timer.elapsed());
            }
            Err(e) => {
                error!(
                    "Root node ({root_node:?}) measurement failed: {e:?}. Aborting draw command computation."
                );
                return (vec![], WindowRequests::default()); // Early return if root measurement fails
            }
        }

        let compute_draw_timer = Instant::now();
        debug!("Start computing draw commands...");
        // compute_draw_commands_parallel expects &ComponentNodeTree and &ComponentNodeMetaDatas
        // It also uses get_mut on metadatas internally, which is fine for DashMap with &self.
        let commands = compute_draw_commands_parallel(root_node, &self.tree, &self.metadatas);
        debug!(
            "Draw commands computed in {:?}, total commands: {}",
            compute_draw_timer.elapsed(),
            commands.len()
        );

        let state_handler_timer = Instant::now();
        let mut window_requests = WindowRequests::default();
        debug!("Start executing state handlers...");
        for node_id in root_node
            .reverse_traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
        {
            let Some(state_handler) = self
                .tree
                .get(node_id)
                .and_then(|n| n.get().state_handler_fn.as_ref())
            else {
                continue;
            };

            // Compute the relative cursor position for the current node, if available
            let current_cursor_position = cursor_position.map(|pos| {
                // Get the absolute position of the current node
                let abs_pos = self
                    .metadatas
                    .get(&node_id)
                    .and_then(|m| m.abs_position)
                    .unwrap_or(PxPosition::ZERO);
                // Calculate the relative position
                pos - abs_pos
            });
            // Get the computed_data for the current node
            let computed_data_option = self.metadatas.get(&node_id).and_then(|m| m.computed_data);

            if let Some(node_computed_data) = computed_data_option {
                // Check if computed_data exists
                let input = StateHandlerInput {
                    node_id,
                    computed_data: node_computed_data,
                    cursor_position: current_cursor_position,
                    cursor_events: &mut cursor_events,
                    keyboard_events: &mut keyboard_events,
                    requests: &mut window_requests,
                };
                state_handler(input);
                // if state_handler set ime request, it's position must be None, and we set it here
                if let Some(ref mut ime_request) = window_requests.ime_request {
                    ime_request.position = Some(
                        self.metadatas
                            .get(&node_id)
                            .and_then(|m| m.abs_position)
                            .unwrap(),
                    )
                }
            } else {
                log::warn!(
                    "Computed data not found for node {node_id:?} during state handler execution."
                );
            }
        }
        debug!(
            "State handlers executed in {:?}",
            state_handler_timer.elapsed()
        );
        (commands, window_requests)
    }
}

// This function seems to take &ComponentNodeTree and &ComponentNodeMetaDatas, which is consistent.
// Internally, it uses metadatas.get_mut(&node_id). DashMap allows this with &self.
fn compute_draw_commands_parallel(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) -> Vec<(PxPosition, PxSize, Box<dyn DrawCommand>)> {
    compute_draw_commands_inner_parallel(PxPosition::ZERO, true, node_id, tree, metadatas)
}

fn compute_draw_commands_inner_parallel(
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) -> Vec<(PxPosition, PxSize, Box<dyn DrawCommand>)> {
    let mut local_commands = Vec::new();

    // Accessing metadatas with get_mut. DashMap's get_mut returns a RefMut,
    // which is fine with an immutable reference to the DashMap itself (&DashMap).
    if let Some(mut entry) = metadatas.get_mut(&node_id) {
        let rel_pos = match entry.rel_position {
            Some(pos) => pos,
            None => {
                if is_root {
                    // If it's the root node and rel_position is None, we assume it starts at [0, 0]
                    PxPosition::ZERO
                } else {
                    // If not root and rel_position is None, we skip this node
                    return local_commands;
                }
            }
        };
        let self_pos = start_pos + rel_pos;
        entry.abs_position = Some(self_pos); // Modifying through RefMut

        if let Some(cmd) = entry.basic_drawable.take() {
            let size = entry.computed_data.unwrap();
            local_commands.push((
                self_pos,
                PxSize {
                    width: size.width,
                    height: size.height,
                },
                cmd,
            ));
        }
    } // RefMut is dropped here, lock released if any

    // Recursive call, passing references
    let children: Vec<_> = node_id.children(tree).collect();
    let child_results: Vec<Vec<_>> = children
        .into_par_iter()
        .map(|child| {
            compute_draw_commands_inner_parallel(
                metadatas
                    .get(&node_id)
                    .and_then(|m| m.abs_position)
                    .unwrap_or(start_pos), // Get self_pos again for children
                false,
                child,
                tree,
                metadatas,
            )
        })
        .collect();

    for child_cmds in child_results {
        local_commands.extend(child_cmds);
    }

    local_commands
}
