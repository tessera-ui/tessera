//! Accessibility tree building
//!
//! This module contains the logic to build AccessKit TreeUpdates from Tessera's component tree.

use accesskit::{Node, NodeId as AccessKitNodeId, Rect, Tree, TreeUpdate};
use indextree::NodeId as ComponentNodeId;

use crate::{
    accessibility::AccessibilityId,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree, ComputedData},
    px::PxPosition,
};

/// Builds an AccessKit TreeUpdate from the component tree.
///
/// This function:
/// 1. Traverses the component tree starting from the root
/// 2. Collects all nodes that have accessibility metadata
/// 3. Builds AccessKit nodes with proper parent-child relationships
/// 4. Determines the current focus
///
/// # Arguments
///
/// * `tree` - The component tree structure (indextree::Arena)
/// * `metadatas` - Component metadata including accessibility information
/// * `root_node_id` - The root node of the component tree
/// * `root_label` - Optional label used when synthesizing a root window node
///
/// # Returns
///
/// A `TreeUpdate` ready to be sent to AccessKit, or `None` if there are no accessibility nodes.
pub fn build_tree_update(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    root_node_id: ComponentNodeId,
    root_label: Option<&str>,
) -> Option<TreeUpdate> {
    let mut nodes = Vec::new();
    let mut focus = None;

    // Convert root node ID
    let root_accesskit_id = AccessibilityId::from_component_node_id(root_node_id);

    // Traverse the tree and collect accessibility nodes
    let has_nodes = traverse_and_collect(
        tree,
        metadatas,
        root_node_id,
        &mut nodes,
        &mut focus,
        true,
        root_label,
    );

    // If no nodes were collected anywhere in the tree, don't create an update
    if !has_nodes || nodes.is_empty() {
        return None;
    }

    // Create the tree structure
    let tree_struct = Tree::new(root_accesskit_id.to_accesskit_id());

    Some(TreeUpdate {
        nodes,
        tree: Some(tree_struct),
        focus: focus.unwrap_or_else(|| root_accesskit_id.to_accesskit_id()),
    })
}

/// Recursively traverses the component tree and collects accessibility nodes.
///
/// Returns `true` if this subtree produced at least one accessibility node (real or synthesized),
/// `false` otherwise.
fn traverse_and_collect(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: ComponentNodeId,
    nodes: &mut Vec<(AccessKitNodeId, Node)>,
    focus: &mut Option<AccessKitNodeId>,
    is_root: bool,
    root_label: Option<&str>,
) -> bool {
    // Get metadata for this node
    let metadata = match metadatas.get(&node_id) {
        Some(metadata) => metadata,
        None => return false,
    };

    let accessibility_node = metadata.accessibility.clone();
    let abs_position = metadata.abs_position;
    let computed_data = metadata.computed_data;
    drop(metadata);

    let mut has_accessible_descendants = false;

    // Collect children with accessibility info
    let mut accessible_children = Vec::new();
    for child_id in node_id.children(tree) {
        // Recursively process child
        let child_has_accessibility =
            traverse_and_collect(tree, metadatas, child_id, nodes, focus, false, root_label);

        has_accessible_descendants |= child_has_accessibility;

        if child_has_accessibility {
            let child_accesskit_id = AccessibilityId::from_component_node_id(child_id);
            accessible_children.push(child_accesskit_id.to_accesskit_id());
        }
    }

    // Check if this node has accessibility information
    if let Some(accessibility_node) = accessibility_node {
        let accesskit_id = AccessibilityId::from_component_node_id(node_id);

        // Build AccessKit Node
        let mut node = Node::new(accessibility_node.role.unwrap_or(accesskit::Role::Unknown));

        // Set label
        if let Some(label) = accessibility_node.label {
            node.set_label(label);
        }

        // Set description
        if let Some(description) = accessibility_node.description {
            node.set_description(description);
        }

        // Set value
        if let Some(value) = accessibility_node.value {
            node.set_value(value);
        }

        // Set numeric value
        if let Some(numeric_value) = accessibility_node.numeric_value {
            node.set_numeric_value(numeric_value);
        }

        // Set focusable
        if accessibility_node.focusable {
            node.add_action(accesskit::Action::Focus);
        }

        // Set focused (and remember for TreeUpdate)
        if accessibility_node.focused {
            *focus = Some(accesskit_id.to_accesskit_id());
        }

        // Set toggled state
        if let Some(toggled) = accessibility_node.toggled {
            node.set_toggled(toggled);
        }

        // Set disabled
        if accessibility_node.disabled {
            node.set_disabled();
        }

        // Set hidden
        if accessibility_node.hidden {
            node.set_hidden();
        }

        // Add actions
        for action in accessibility_node.actions {
            node.add_action(action);
        }

        // Set children if any
        if !accessible_children.is_empty() {
            node.set_children(accessible_children);
        }

        if let Some(bounds) = rect_from_geometry(abs_position, computed_data) {
            node.set_bounds(bounds);
        }

        // Add to collection
        nodes.push((accesskit_id.to_accesskit_id(), node));

        true
    } else if is_root || has_accessible_descendants {
        let accesskit_id = AccessibilityId::from_component_node_id(node_id);
        let mut node = if is_root {
            let mut root_node = Node::new(accesskit::Role::Window);
            if let Some(label) = root_label {
                root_node.set_label(label.to_string());
            }
            root_node
        } else {
            Node::new(accesskit::Role::GenericContainer)
        };

        if !accessible_children.is_empty() {
            node.set_children(accessible_children);
        }

        if let Some(bounds) = rect_from_geometry(abs_position, computed_data) {
            node.set_bounds(bounds);
        }

        nodes.push((accesskit_id.to_accesskit_id(), node));
        true
    } else {
        false
    }
}

fn rect_from_geometry(
    abs_position: Option<PxPosition>,
    computed_data: Option<ComputedData>,
) -> Option<Rect> {
    let position = abs_position?;
    let size = computed_data?;

    let x0 = position.x.0 as f64;
    let y0 = position.y.0 as f64;
    let x1 = x0 + size.width.0 as f64;
    let y1 = y0 + size.height.0 as f64;

    Some(Rect { x0, y0, x1, y1 })
}

/// Dispatches an accessibility action to the appropriate component handler.
///
/// This function:
/// 1. Converts the AccessKit NodeId back to a component NodeId
/// 2. Looks up the component's metadata
/// 3. Calls the component's accessibility_action_handler if present
///
/// # Arguments
///
/// * `tree` - The component tree structure (for NodeId validation)
/// * `metadatas` - Component metadata including action handlers
/// * `action_request` - The action request from AccessKit
///
/// # Returns
///
/// `true` if the action was handled, `false` if no handler was found.
pub fn dispatch_action(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    action_request: accesskit::ActionRequest,
) -> bool {
    // Convert AccessKit NodeId back to AccessibilityId
    let accessibility_id = AccessibilityId::from_accesskit_id(action_request.target);

    // Convert to component NodeId using get_node_id_at
    // The AccessibilityId stores the 1-based index from indextree
    let index = std::num::NonZero::new(accessibility_id.0 as usize);
    let component_node_id = index.and_then(|idx| tree.get_node_id_at(idx));

    // Look up the component's metadata and call handler
    if let Some(node_id) = component_node_id
        && let Some(metadata) = metadatas.get(&node_id)
        && let Some(handler) = &metadata.accessibility_action_handler
    {
        // Call the handler
        handler(action_request.action);
        return true;
    }

    false
}
