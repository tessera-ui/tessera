mod constraint;
mod node;

use std::{any::TypeId, num::NonZero, sync::Arc, time::Instant};

use parking_lot::RwLock;
use rayon::prelude::*;
use tracing::{debug, warn};

use crate::{
    Clipboard, ComputeResourceManager, Px, PxRect,
    cursor::CursorEvent,
    px::{PxPosition, PxSize},
    renderer::Command,
};

pub use constraint::{Constraint, DimensionValue};
pub use node::{
    ComponentNode, ComponentNodeMetaData, ComponentNodeMetaDatas, ComponentNodeTree, ComputedData,
    ImeRequest, InputHandlerFn, InputHandlerInput, MeasureFn, MeasureInput, MeasurementError,
    WindowRequests, measure_node, measure_nodes, place_node,
};

/// Parameters for the compute function
pub struct ComputeParams<'a> {
    pub screen_size: PxSize,
    pub cursor_position: Option<PxPosition>,
    pub cursor_events: Vec<CursorEvent>,
    pub keyboard_events: Vec<winit::event::KeyEvent>,
    pub ime_events: Vec<winit::event::Ime>,
    pub modifiers: winit::keyboard::ModifiersState,
    pub compute_resource_manager: Arc<RwLock<ComputeResourceManager>>,
    pub gpu: &'a wgpu::Device,
    pub clipboard: &'a mut Clipboard,
}

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
    #[tracing::instrument(level = "debug", skip(self, params))]
    pub fn compute(
        &mut self,
        params: ComputeParams<'_>,
    ) -> (Vec<(Command, TypeId, PxSize, PxPosition)>, WindowRequests) {
        let ComputeParams {
            screen_size,
            mut cursor_position,
            mut cursor_events,
            mut keyboard_events,
            mut ime_events,
            modifiers,
            compute_resource_manager,
            gpu,
            clipboard,
        } = params;
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

        let input_handler_timer = Instant::now();
        let mut window_requests = WindowRequests::default();
        debug!("Start executing input handlers...");
        for node_id in root_node
            .reverse_traverse(&self.tree)
            .filter_map(|edge| match edge {
                indextree::NodeEdge::Start(id) => Some(id),
                indextree::NodeEdge::End(_) => None,
            })
        {
            let Some(input_handler) = self
                .tree
                .get(node_id)
                .and_then(|n| n.get().input_handler_fn.as_ref())
            else {
                continue;
            };

            // Compute the relative cursor position for the current node
            let current_cursor_position = cursor_position.map(|pos| {
                // Get the absolute position of the current node
                let abs_pos = self
                    .metadatas
                    .get(&node_id)
                    .and_then(|m| m.abs_position)
                    .unwrap();
                // Calculate the relative position
                pos - abs_pos
            });
            // Get the computed_data for the current node
            let computed_data_option = self.metadatas.get(&node_id).and_then(|m| m.computed_data);

            if let Some(node_computed_data) = computed_data_option {
                // Check if computed_data exists
                let input = InputHandlerInput {
                    computed_data: node_computed_data,
                    cursor_position_rel: current_cursor_position,
                    cursor_position_abs: &mut cursor_position,
                    cursor_events: &mut cursor_events,
                    keyboard_events: &mut keyboard_events,
                    ime_events: &mut ime_events,
                    key_modifiers: modifiers,
                    requests: &mut window_requests,
                    clipboard,
                };
                input_handler(input);
                // if input_handler set ime request, it's position must be None, and we set it here
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
                warn!(
                    "Computed data not found for node {node_id:?} during input handler execution."
                );
            }
        }
        debug!(
            "Input Handlers executed in {:?}",
            input_handler_timer.elapsed()
        );
        (commands, window_requests)
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
#[tracing::instrument(level = "trace", skip(tree, metadatas))]
fn compute_draw_commands_parallel(
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    // New params: screen width and height
    screen_width: i32,
    screen_height: i32,
) -> Vec<(Command, TypeId, PxSize, PxPosition)> {
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

#[tracing::instrument(level = "trace", skip(tree, metadatas))]
fn compute_draw_commands_inner_parallel(
    start_pos: PxPosition,
    is_root: bool,
    node_id: indextree::NodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    screen_width: i32,
    screen_height: i32,
) -> Vec<(Command, TypeId, PxSize, PxPosition)> {
    let mut local_commands = Vec::new();

    // Get metadata and calculate absolute position. This MUST happen for all nodes.
    let mut metadata = metadatas.get_mut(&node_id).unwrap();
    let rel_pos = match metadata.rel_position {
        Some(pos) => pos,
        None if is_root => PxPosition::ZERO,
        _ => return local_commands, // Skip nodes that were not placed at all.
    };
    let self_pos = start_pos + rel_pos;
    metadata.abs_position = Some(self_pos);

    let size = metadata
        .computed_data
        .map(|d| PxSize {
            width: d.width,
            height: d.height,
        })
        .unwrap_or_default();

    let clips_children = metadata.clips_children;
    // Add Clip command if the node clips its children
    if clips_children {
        local_commands.push((
            Command::ClipPush(PxRect {
                x: self_pos.x,
                y: self_pos.y,
                width: size.width,
                height: size.height,
            }),
            TypeId::of::<Command>(),
            size,
            self_pos,
        ));
    }

    // Viewport culling check
    let screen_rect = PxRect {
        x: Px(0),
        y: Px(0),
        width: Px(screen_width),
        height: Px(screen_height),
    };
    let node_rect = PxRect {
        x: Px(self_pos.x.0),
        y: Px(self_pos.y.0),
        width: Px(size.width.0),
        height: Px(size.height.0),
    };

    // Only drain commands if the node is visible.
    if size.width.0 > 0 && size.height.0 > 0 && !node_rect.is_orthogonal(&screen_rect) {
        for (cmd, type_id) in metadata.commands.drain(..) {
            local_commands.push((cmd, type_id, size, self_pos));
        }
    }

    drop(metadata); // Release lock before recursing

    // ALWAYS recurse to children to ensure their abs_position is calculated.
    let children: Vec<_> = node_id.children(tree).collect();
    let child_results: Vec<Vec<_>> = children
        .into_par_iter()
        .map(|child| {
            // The unwrap is safe because we just set the parent's abs_position.
            let parent_abs_pos = metadatas.get(&node_id).unwrap().abs_position.unwrap();
            compute_draw_commands_inner_parallel(
                parent_abs_pos, // Pass the calculated absolute position
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

    // If the node clips its children, we need to pop the clip command
    if clips_children {
        local_commands.push((Command::ClipPop, TypeId::of::<Command>(), size, self_pos));
    }

    local_commands
}
