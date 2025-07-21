mod constraint;
mod node;

use std::{num::NonZero, sync::Arc, time::Instant};

use log::debug;
use parking_lot::RwLock;
use rayon::prelude::*;

use crate::{
    ComputeResourceManager,
    cursor::CursorEvent,
    px::{PxPosition, PxSize},
    renderer::Command,
};

pub use constraint::{Constraint, DimensionValue};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    ImeRequest, MeasureFn, MeasurementError, StateHandlerFn, StateHandlerInput, WindowRequests,
    measure_node, measure_nodes, place_node,
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

    /// Compute the ComponentTree into a list of rendering commands
    ///
    /// This method processes the component tree through three main phases:
    /// 1. **Measure Phase**: Calculate sizes and positions for all components
    /// 2. **Command Generation**: Extract draw commands from component metadata
    /// 3. **State Handling**: Process user interactions and events
    ///
    /// Returns a tuple of (commands, window_requests) where commands contain
    /// the rendering instructions with their associated sizes and positions.
    pub fn compute(
        &mut self,
        screen_size: PxSize,
        cursor_position: Option<PxPosition>,
        mut cursor_events: Vec<CursorEvent>,
        mut keyboard_events: Vec<winit::event::KeyEvent>,
        mut ime_events: Vec<winit::event::Ime>,
        modifiers: winit::keyboard::ModifiersState,
        compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
        gpu: &wgpu::Device,
    ) -> (Vec<(Command, PxSize, PxPosition)>, WindowRequests) {
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
        match measure_node(
            root_node,
            &screen_constraint,
            &self.tree,
            &self.metadatas,
            compute_resource_manager,
            gpu,
        ) {
            Ok(_root_computed_data) => {
                debug!("Component tree measured in {:?}", measure_timer.elapsed());
            }
            Err(e) => {
                panic!(
                    "Root node ({root_node:?}) measurement failed: {e:?}. Aborting draw command computation."
                );
            }
        }

        let compute_draw_timer = Instant::now();
        debug!("Start computing draw commands...");
        // compute_draw_commands_parallel expects &ComponentNodeTree and &ComponentNodeMetaDatas
        // It also uses get_mut on metadatas internally, which is fine for DashMap with &self.
        let commands = compute_draw_commands_parallel(
            root_node,
            &self.tree,
            &self.metadatas,
            screen_size.width.0,
            screen_size.height.0,
        );
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
                    ime_events: &mut ime_events,
                    key_modifiers: modifiers,
                    requests: &mut window_requests,
                };
                state_handler(input);
                // if state_handler set ime request, it's position must be None, and we set it here
                if let Some(ref mut ime_request) = window_requests.ime_request
                    && ime_request.position.is_none()
                {
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

// Helper struct for rectangle and intersection check
#[derive(Debug, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// Parallel computation of draw commands from the component tree
///
/// This function traverses the component tree and extracts rendering commands
/// from each node's metadata. It uses parallel processing for better performance
/// when dealing with large component trees.
///
/// The function maintains thread-safety by using DashMap's concurrent access
/// capabilities, allowing multiple threads to safely read and modify metadata.
fn compute_draw_commands_parallel(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    // New params: screen width and height
    screen_width: i32,
    screen_height: i32,
) -> Vec<(Command, PxSize, PxPosition)> {
    compute_draw_commands_inner_parallel(
        PxPosition::ZERO,
        true,
        node_id,
        tree,
        metadatas,
        screen_width,
        screen_height,
    )
}

fn compute_draw_commands_inner_parallel(
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    screen_width: i32,
    screen_height: i32,
) -> Vec<(Command, PxSize, PxPosition)> {
    let mut local_commands = Vec::new();

    // Process current node's metadata and extract its rendering commands
    if let Some(mut entry) = metadatas.get_mut(&node_id) {
        // Calculate absolute position: root nodes start at origin, others use relative positioning
        let rel_pos = match entry.rel_position {
            Some(pos) => pos,
            None if is_root => PxPosition::ZERO,
            _ => return local_commands, // Skip nodes without position data
        };
        let self_pos = start_pos + rel_pos;
        entry.abs_position = Some(self_pos);

        let size = entry
            .computed_data
            .map(|d| PxSize {
                width: d.width,
                height: d.height,
            })
            .unwrap_or_default();

        // Viewport culling: skip if the node is completely outside the screen
        let screen_rect = Rect {
            x: 0,
            y: 0,
            width: screen_width,
            height: screen_height,
        };
        let node_rect = Rect {
            x: self_pos.x.0,
            y: self_pos.y.0,
            width: size.width.0,
            height: size.height.0,
        };
        // If the node is completely invisible, skip it
        if size.width.0 > 0 && size.height.0 > 0 && !node_rect.intersects(&screen_rect) {
            return local_commands;
        }

        // Drain all commands from this node and add them to the output
        for cmd in entry.commands.drain(..) {
            local_commands.push((cmd, size, self_pos));
        }
    }

    // Process all child nodes in parallel for better performance
    let children: Vec<_> = node_id.children(tree).collect();
    let child_results: Vec<Vec<_>> = children
        .into_par_iter()
        .map(|child| {
            let self_pos = metadatas
                .get(&node_id)
                .and_then(|m| m.abs_position)
                .unwrap_or(start_pos);
            compute_draw_commands_inner_parallel(
                self_pos,
                false,
                child,
                tree,
                metadatas,
                screen_width,
                screen_height,
            )
        })
        .collect();

    for child_cmds in child_results {
        local_commands.extend(child_cmds);
    }

    local_commands
}
