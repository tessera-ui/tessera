//! Accessibility trees and action dispatch for assistive technology.
//!
//! ## Usage
//!
//! Expose controls, text, and semantic groups to assistive technology.

use accesskit::{Node, NodeId as AccessKitNodeId, Rect, Tree, TreeId, TreeUpdate};
use indextree::NodeId as ComponentNodeId;

use crate::{
    accessibility::AccessibilityId,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree, ComputedData},
    focus::{FocusOwner, bind_focus_owner},
    px::PxPosition,
};

#[derive(Default)]
struct CollectedNodes {
    nodes: Vec<(AccessKitNodeId, Node)>,
    focus: Option<AccessKitNodeId>,
}

/// Builds an accessibility update with semantic groups, bounds, actions, and
/// focus. Nodes without semantics preserve their descendants' reading order;
/// a window root is supplied when the root has no explicit semantics.
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
/// A `TreeUpdate` ready to be sent to AccessKit, or `None` if there are no
/// accessibility nodes.
pub fn build_tree_update(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    root_node_id: ComponentNodeId,
    root_label: Option<&str>,
) -> Option<TreeUpdate> {
    let mut collected = CollectedNodes::default();
    let mut root_children = Vec::new();

    // Convert root node ID
    let root_accesskit_id = AccessibilityId::from_component_node_id(root_node_id);

    // Traverse the tree and collect accessibility nodes
    let has_nodes = traverse_and_collect(
        tree,
        metadatas,
        root_node_id,
        &mut collected,
        &mut root_children,
        true,
        root_label,
    );

    // If no nodes were collected anywhere in the tree, don't create an update
    if !has_nodes || collected.nodes.is_empty() {
        return None;
    }

    // Create the tree structure
    let tree_struct = Tree::new(root_accesskit_id.to_accesskit_id());

    Some(TreeUpdate {
        nodes: collected.nodes,
        tree: Some(tree_struct),
        tree_id: TreeId::ROOT,
        focus: collected
            .focus
            .unwrap_or_else(|| root_accesskit_id.to_accesskit_id()),
    })
}

/// Recursively traverses the component tree and collects accessibility nodes.
///
/// Returns `true` if this subtree produced at least one accessibility node
/// (real or synthesized root), `false` otherwise.
fn traverse_and_collect(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: ComponentNodeId,
    collected: &mut CollectedNodes,
    parent_children: &mut Vec<AccessKitNodeId>,
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

    // Layout wrappers have no semantics. Publishing each wrapper makes every
    // scroll update change many redundant bounds and multiplies platform
    // events.
    if accessibility_node.is_none() && !is_root {
        let mut has_accessible_descendants = false;
        for child_id in node_id.children(tree) {
            has_accessible_descendants |= traverse_and_collect(
                tree,
                metadatas,
                child_id,
                collected,
                parent_children,
                false,
                root_label,
            );
        }
        return has_accessible_descendants;
    }

    let merge_descendants = accessibility_node
        .as_ref()
        .map(|node| node.merge_descendants)
        .unwrap_or(true);

    // Collect children with accessibility info
    let mut accessible_children = Vec::new();
    if merge_descendants {
        for child_id in node_id.children(tree) {
            // Recursively process child
            traverse_and_collect(
                tree,
                metadatas,
                child_id,
                collected,
                &mut accessible_children,
                false,
                root_label,
            );
        }
    }

    // Check if this node has accessibility information
    if let Some(accessibility_node) = accessibility_node {
        let accesskit_id = AccessibilityId::from_component_node_id(node_id);

        // Build AccessKit Node
        let mut node = Node::new(accessibility_node.role.unwrap_or_else(|| {
            if accessibility_node.heading_level.is_some() {
                accesskit::Role::Heading
            } else {
                accesskit::Role::Unknown
            }
        }));

        // Set label
        if let Some(label) = accessibility_node.label {
            node.set_label(label);
        }

        // Set description
        if let Some(description) = accessibility_node.description {
            node.set_description(description);
        }

        if let Some(state_description) = accessibility_node.state_description {
            node.set_state_description(state_description);
        }

        if let Some(role_description) = accessibility_node.role_description {
            node.set_role_description(role_description);
        }

        if let Some(tooltip) = accessibility_node.tooltip {
            node.set_tooltip(tooltip);
        }

        if let Some(live) = accessibility_node.live {
            node.set_live(live);
        }

        if let Some(level) = accessibility_node.heading_level {
            node.set_level(level as usize);
        }

        // Set value
        if let Some(value) = accessibility_node.value {
            node.set_value(value);
        }

        // Set numeric value
        if let Some(numeric_value) = accessibility_node.numeric_value {
            node.set_numeric_value(numeric_value);
        }

        if let Some(step) = accessibility_node.numeric_value_step {
            node.set_numeric_value_step(step);
        }

        if let Some(jump) = accessibility_node.numeric_value_jump {
            node.set_numeric_value_jump(jump);
        }

        if let Some((value, min, max)) = accessibility_node.scroll_x {
            node.set_scroll_x(value);
            node.set_scroll_x_min(min);
            node.set_scroll_x_max(max);
        }

        if let Some((value, min, max)) = accessibility_node.scroll_y {
            node.set_scroll_y(value);
            node.set_scroll_y_min(min);
            node.set_scroll_y_max(max);
        }

        if let Some((rows, cols, hierarchical)) = accessibility_node.collection_info {
            node.set_row_count(rows);
            node.set_column_count(cols);
            if hierarchical {
                node.set_level(1);
            }
        }

        if let Some((row_index, row_span, col_index, col_span, heading)) =
            accessibility_node.collection_item_info
        {
            node.set_row_index(row_index);
            node.set_row_span(row_span);
            node.set_column_index(col_index);
            node.set_column_span(col_span);
            if heading {
                node.set_level(1);
            }
        }

        if accessibility_node.is_editable_text {
            node.set_live(accesskit::Live::Polite);
        }

        // Set focusable
        if accessibility_node.focusable {
            node.add_action(accesskit::Action::Focus);
        }

        // Set focused (and remember for TreeUpdate)
        if accessibility_node.focused {
            collected.focus = Some(accesskit_id.to_accesskit_id());
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

        let bounds = match accessibility_node.bounds_padding {
            Some(padding) => rect_from_geometry_with_padding(abs_position, computed_data, padding),
            None => rect_from_geometry(abs_position, computed_data),
        };

        if let Some(bounds) = bounds {
            node.set_bounds(bounds);
        }

        // Add to collection
        collected.nodes.push((accesskit_id.to_accesskit_id(), node));
        parent_children.push(accesskit_id.to_accesskit_id());

        true
    } else if is_root {
        let accesskit_id = AccessibilityId::from_component_node_id(node_id);
        let mut node = Node::new(accesskit::Role::Window);
        if let Some(label) = root_label {
            node.set_label(label.to_string());
        }

        if !accessible_children.is_empty() {
            node.set_children(accessible_children);
        }

        if let Some(bounds) = rect_from_geometry(abs_position, computed_data) {
            node.set_bounds(bounds);
        }

        collected.nodes.push((accesskit_id.to_accesskit_id(), node));
        parent_children.push(accesskit_id.to_accesskit_id());
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

    let x0 = position.x as f64;
    let y0 = position.y as f64;
    let x1 = x0 + size.width.0 as f64;
    let y1 = y0 + size.height.0 as f64;

    Some(Rect { x0, y0, x1, y1 })
}

fn rect_from_geometry_with_padding(
    abs_position: Option<PxPosition>,
    computed_data: Option<ComputedData>,
    padding: crate::accessibility::AccessibilityPadding,
) -> Option<Rect> {
    let position = abs_position?;
    let size = computed_data?;

    let x0 = position.x as f64 - padding.left.0 as f64;
    let y0 = position.y as f64 - padding.top.0 as f64;
    let x1 = x0 + size.width.0 as f64 + (padding.left + padding.right).0 as f64;
    let y1 = y0 + size.height.0 as f64 + (padding.top + padding.bottom).0 as f64;

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
    focus_owner: &mut FocusOwner,
    action_request: accesskit::ActionRequest,
) -> bool {
    // Convert AccessKit NodeId back to AccessibilityId
    if action_request.target_tree != TreeId::ROOT {
        return false;
    }

    let accessibility_id = AccessibilityId::from_accesskit_id(action_request.target_node);

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
        let _focus_owner_guard = bind_focus_owner(focus_owner);
        handler(action_request.action);
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use accesskit::{Action, Role};

    use crate::{
        Modifier, Px,
        accessibility::AccessibilityNode,
        component_tree::{ComponentNode, ComponentNodeMetaData, NodeRole},
        layout::{DefaultLayoutNode, NoopDrawModifier},
    };

    use super::*;

    fn append_node(
        tree: &mut ComponentNodeTree,
        metadatas: &mut ComponentNodeMetaDatas,
        parent: Option<ComponentNodeId>,
        accessibility: Option<AccessibilityNode>,
    ) -> ComponentNodeId {
        let id = tree.new_node(ComponentNode {
            fn_name: "test_node".to_string(),
            role: NodeRole::Layout,
            instance_logic_id: 0,
            instance_key: 0,
            pointer_preview_handlers: Vec::new(),
            pointer_handlers: Vec::new(),
            pointer_final_handlers: Vec::new(),
            keyboard_preview_handlers: Vec::new(),
            keyboard_handlers: Vec::new(),
            ime_preview_handlers: Vec::new(),
            ime_handlers: Vec::new(),
            focus_requester_binding: None,
            focus_registration: None,
            focus_restorer_fallback: None,
            focus_traversal_policy: None,
            focus_changed_handler: None,
            focus_event_handler: None,
            focus_beyond_bounds_handler: None,
            focus_reveal_handler: None,
            globally_positioned_handlers: Vec::new(),
            modifier: Modifier::default(),
            layout_node: Box::new(DefaultLayoutNode),
            draw_node: Box::new(NoopDrawModifier),
            replay: None,
            props_unchanged_from_previous: false,
        });
        if let Some(parent) = parent {
            parent.append(id, tree);
        }
        let mut metadata = ComponentNodeMetaData::none();
        metadata.accessibility = accessibility;
        metadata.abs_position = Some(PxPosition::new(Px(10), Px(20)));
        metadata.computed_data = Some(ComputedData {
            width: Px(40),
            height: Px(50),
        });
        metadatas.insert(id, metadata);
        id
    }

    fn accesskit_id(id: ComponentNodeId) -> AccessKitNodeId {
        AccessibilityId::from_component_node_id(id).to_accesskit_id()
    }

    fn node(update: &TreeUpdate, id: ComponentNodeId) -> &Node {
        &update
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == accesskit_id(id))
            .expect("semantic node must be present")
            .1
    }

    #[test]
    fn wrappers_preserve_semantic_groups_order_bounds_and_focus() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let wrapper = append_node(&mut tree, &mut metadatas, Some(root), None);
        let group = append_node(
            &mut tree,
            &mut metadatas,
            Some(wrapper),
            Some(AccessibilityNode::new().with_role(Role::Group)),
        );
        let mut parent = group;
        for _ in 0..12 {
            parent = append_node(&mut tree, &mut metadatas, Some(parent), None);
        }
        let day = append_node(
            &mut tree,
            &mut metadatas,
            Some(parent),
            Some(
                AccessibilityNode::new()
                    .with_role(Role::Button)
                    .with_label("12")
                    .with_action(Action::Click)
                    .focusable()
                    .focused(),
            ),
        );
        let label = append_node(
            &mut tree,
            &mut metadatas,
            Some(parent),
            Some(AccessibilityNode::new().with_role(Role::Label)),
        );
        let next = append_node(
            &mut tree,
            &mut metadatas,
            Some(wrapper),
            Some(AccessibilityNode::new().with_role(Role::Button)),
        );

        let update = build_tree_update(&tree, &metadatas, root, Some("Calendar"))
            .expect("valid tree must produce an accessibility update");
        assert_eq!(update.nodes.len(), 5);
        assert_eq!(node(&update, root).role(), Role::Window);
        assert_eq!(node(&update, root).label(), Some("Calendar"));
        assert_eq!(
            node(&update, root).children(),
            &[accesskit_id(group), accesskit_id(next)]
        );
        assert_eq!(node(&update, group).role(), Role::Group);
        assert_eq!(
            node(&update, group).children(),
            &[accesskit_id(day), accesskit_id(label)]
        );
        assert_eq!(node(&update, day).label(), Some("12"));
        assert_eq!(
            node(&update, day).bounds(),
            Some(Rect {
                x0: 10.0,
                y0: 20.0,
                x1: 50.0,
                y1: 70.0
            })
        );
        assert!(node(&update, day).supports_action(Action::Click));
        assert!(node(&update, day).supports_action(Action::Focus));
        assert_eq!(update.focus, accesskit_id(day));
    }

    #[test]
    fn cleared_semantics_still_suppress_descendants_and_their_focus() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let mut semantics = AccessibilityNode::new().with_role(Role::Button);
        semantics.merge_descendants = false;
        let button = append_node(&mut tree, &mut metadatas, Some(root), Some(semantics));
        let wrapper = append_node(&mut tree, &mut metadatas, Some(button), None);
        append_node(
            &mut tree,
            &mut metadatas,
            Some(wrapper),
            Some(AccessibilityNode::new().with_role(Role::Label).focused()),
        );

        let update = build_tree_update(&tree, &metadatas, root, None)
            .expect("valid tree must produce an accessibility update");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(node(&update, root).children(), &[accesskit_id(button)]);
        assert!(node(&update, button).children().is_empty());
        assert_eq!(update.focus, accesskit_id(root));
    }

    #[test]
    fn tree_without_semantics_keeps_only_the_window_root() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let wrapper = append_node(&mut tree, &mut metadatas, Some(root), None);
        append_node(&mut tree, &mut metadatas, Some(wrapper), None);

        let update = build_tree_update(&tree, &metadatas, root, Some("Empty"))
            .expect("valid tree must produce an accessibility update");
        assert_eq!(update.nodes.len(), 1);
        assert_eq!(node(&update, root).role(), Role::Window);
        assert!(node(&update, root).children().is_empty());
        assert_eq!(update.focus, accesskit_id(root));
    }
}
