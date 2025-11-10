//! Accessibility tree building
//!
//! This module contains the logic to build AccessKit TreeUpdates from Tessera's component tree.

use accesskit::{Node, NodeId as AccessKitNodeId, Tree, TreeUpdate};
use indextree::NodeId as ComponentNodeId;

use crate::{
    accessibility::AccessibilityId,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree},
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
///
/// # Returns
///
/// A `TreeUpdate` ready to be sent to AccessKit, or `None` if there are no accessibility nodes.
pub fn build_tree_update(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    root_node_id: ComponentNodeId,
) -> Option<TreeUpdate> {
    let mut nodes = Vec::new();
    let mut focus = None;

    // Convert root node ID
    let root_accesskit_id = AccessibilityId::from_component_node_id(root_node_id);

    // Traverse the tree and collect accessibility nodes
    traverse_and_collect(tree, metadatas, root_node_id, &mut nodes, &mut focus)?;

    // If no nodes were collected, don't create an update
    if nodes.is_empty() {
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
/// Returns `Some(())` if at least one accessibility node was found, `None` otherwise.
fn traverse_and_collect(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: ComponentNodeId,
    nodes: &mut Vec<(AccessKitNodeId, Node)>,
    focus: &mut Option<AccessKitNodeId>,
) -> Option<()> {
    // Get metadata for this node
    let metadata = metadatas.get(&node_id)?;

    // Check if this node has accessibility information
    if let Some(accessibility_node) = &metadata.accessibility {
        let accesskit_id = AccessibilityId::from_component_node_id(node_id);

        // Build AccessKit Node
        let mut node = Node::new(accessibility_node.role.unwrap_or(accesskit::Role::Unknown));

        // Set label
        if let Some(label) = &accessibility_node.label {
            node.set_label(label.clone());
        }

        // Set description
        if let Some(description) = &accessibility_node.description {
            node.set_description(description.clone());
        }

        // Set value
        if let Some(value) = &accessibility_node.value {
            node.set_value(value.clone());
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
        for action in &accessibility_node.actions {
            node.add_action(*action);
        }

        // Collect children with accessibility info
        let mut accessible_children = Vec::new();
        for child_id in node_id.children(tree) {
            // Recursively process child
            traverse_and_collect(tree, metadatas, child_id, nodes, focus);

            // Check if child has accessibility info
            if let Some(child_metadata) = metadatas.get(&child_id)
                && child_metadata.accessibility.is_some()
            {
                let child_accesskit_id = AccessibilityId::from_component_node_id(child_id);
                accessible_children.push(child_accesskit_id.to_accesskit_id());
            }
        }

        // Set children if any
        if !accessible_children.is_empty() {
            node.set_children(accessible_children);
        }

        // Add to collection
        nodes.push((accesskit_id.to_accesskit_id(), node));

        Some(())
    } else {
        // No accessibility info on this node, but traverse children anyway
        for child_id in node_id.children(tree) {
            traverse_and_collect(tree, metadatas, child_id, nodes, focus);
        }

        None
    }
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
