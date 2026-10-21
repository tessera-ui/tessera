//! Accessibility trees and action dispatch for assistive technology.
//!
//! ## Usage
//!
//! Expose controls, text, and semantic groups to assistive technology.

use std::collections::BTreeMap;

use accesskit::{Node, NodeId as AccessKitNodeId, Rect, Tree, TreeId, TreeUpdate};
use indextree::NodeId as ComponentNodeId;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    accessibility::AccessibilityId,
    component_tree::{ComponentNodeMetaDatas, ComponentNodeTree, ComputedData, NodeRole},
    focus::{FocusOwner, bind_focus_owner},
    px::PxPosition,
};

#[derive(Clone, Default, PartialEq)]
struct DescendantContext {
    semantic: bool,
    tooltip: Option<String>,
    traverses: bool,
}

struct SemanticEntry {
    parent: Option<AccessKitNodeId>,
    order: Vec<usize>,
    children: BTreeMap<Vec<usize>, AccessKitNodeId>,
    node: Node,
}

#[derive(Default)]
pub(crate) struct AccessibilityIndex {
    components: HashMap<ComponentNodeId, (AccessKitNodeId, DescendantContext)>,
    nodes: HashMap<AccessKitNodeId, SemanticEntry>,
    focused: BTreeMap<Vec<usize>, AccessKitNodeId>,
    pending: HashSet<AccessKitNodeId>,
    root: Option<AccessKitNodeId>,
    root_label: Option<String>,
    last_focus: Option<AccessKitNodeId>,
    changed: HashSet<ComponentNodeId>,
    removed: HashSet<ComponentNodeId>,
}

impl AccessibilityIndex {
    pub(crate) fn invalidate(
        &mut self,
        changed: &HashSet<ComponentNodeId>,
        removed: &HashSet<ComponentNodeId>,
    ) {
        self.changed.extend(changed);
        self.removed.extend(removed);
    }

    pub(crate) fn update(
        &mut self,
        tree: &ComponentNodeTree,
        metadatas: &ComponentNodeMetaDatas,
        root: ComponentNodeId,
        order: &HashMap<ComponentNodeId, usize>,
    ) -> usize {
        let mut changed = std::mem::take(&mut self.changed);
        let mut removed_keys = HashSet::default();
        for id in std::mem::take(&mut self.removed) {
            if let Some((key, _)) = self.components.remove(&id) {
                removed_keys.insert(key);
            }
        }
        let mut contexts = HashMap::default();
        let mut descendants = Vec::new();
        for &id in &changed {
            if tree.get(id).is_none_or(|node| node.is_removed()) {
                continue;
            }
            let semantics = metadatas
                .get(&id)
                .and_then(|metadata| metadata.accessibility.as_ref());
            let context = DescendantContext {
                semantic: semantics.is_some(),
                tooltip: semantics.and_then(|semantics| semantics.tooltip.clone()),
                traverses: semantics.is_none_or(|semantics| semantics.merge_descendants),
            };
            if self
                .components
                .get(&id)
                .is_some_and(|(_, previous)| *previous != context)
            {
                descendants.extend(id.descendants(tree).skip(1));
            }
            contexts.insert(id, context);
        }
        changed.extend(descendants);
        let mut child_changes = HashSet::default();
        for &id in &changed {
            let Some(component) = tree.get(id).filter(|node| !node.is_removed()) else {
                continue;
            };
            let key = AccessKitNodeId(component.get().instance_key);
            removed_keys.remove(&key);
            let context = contexts.remove(&id).unwrap_or_else(|| {
                let semantics = metadatas
                    .get(&id)
                    .and_then(|metadata| metadata.accessibility.as_ref());
                DescendantContext {
                    semantic: semantics.is_some(),
                    tooltip: semantics.and_then(|node| node.tooltip.clone()),
                    traverses: semantics.is_none_or(|node| node.merge_descendants),
                }
            });
            self.components.insert(id, (key, context));
            let mut parent = None;
            let mut tooltip = None;
            let mut visible = true;
            let mut path = Vec::new();
            for ancestor in id.ancestors(tree) {
                path.push(order.get(&ancestor).copied().unwrap_or_default());
                let Some(metadata) = metadatas.get(&ancestor) else {
                    visible = false;
                    break;
                };
                if ancestor != root
                    && tree[ancestor].get().role == NodeRole::Layout
                    && metadata.abs_position.is_none()
                {
                    visible = false;
                }
                if let Some(semantics) = &metadata.accessibility {
                    if tooltip.is_none() {
                        tooltip = semantics.tooltip.as_deref();
                    }
                    if ancestor != id {
                        if !semantics.merge_descendants {
                            visible = false;
                        }
                        if parent.is_none() {
                            parent = Some(AccessKitNodeId(tree[ancestor].get().instance_key));
                        }
                    }
                }
            }
            path.reverse();
            if id != root && parent.is_none() {
                parent = Some(AccessKitNodeId(tree[root].get().instance_key));
            }
            let built = visible
                .then(|| {
                    build_node(
                        tree,
                        metadatas,
                        id,
                        id == root,
                        self.root_label.as_deref(),
                        tooltip,
                    )
                })
                .flatten();
            let Some((mut node, focused)) = built else {
                self.remove(key, &mut child_changes);
                continue;
            };
            let mut focus_order = path.clone();
            focus_order.push(usize::MAX);
            let previous = self.nodes.get(&key);
            let relink = previous.is_none_or(|entry| entry.parent != parent || entry.order != path);
            if relink {
                if let Some(previous) = previous {
                    let previous_parent = previous.parent;
                    let previous_order = previous.order.clone();
                    if let Some(parent) = previous_parent {
                        if let Some(entry) = self.nodes.get_mut(&parent) {
                            entry.children.remove(&previous_order);
                        }
                        child_changes.insert(parent);
                    }
                    let mut old_focus_order = previous_order;
                    old_focus_order.push(usize::MAX);
                    self.focused.remove(&old_focus_order);
                }
                if let Some(parent) = parent {
                    child_changes.insert(parent);
                }
            }
            if focused {
                self.focused.insert(focus_order, key);
            } else {
                self.focused.remove(&focus_order);
            }
            let entry = self.nodes.entry(key).or_insert_with(|| SemanticEntry {
                parent,
                order: path.clone(),
                children: BTreeMap::new(),
                node: Node::default(),
            });
            entry.parent = parent;
            entry.order = path;
            node.set_children(entry.children.values().copied().collect::<Vec<_>>());
            if entry.node != node {
                entry.node = node;
                self.pending.insert(key);
            }
        }
        for key in removed_keys {
            self.remove(key, &mut child_changes);
        }
        // Parents can be inserted after their children in the change journal.
        for &id in &changed {
            let Some((key, _)) = self.components.get(&id) else {
                continue;
            };
            let Some(entry) = self.nodes.get(key) else {
                continue;
            };
            let (parent, path) = (entry.parent, entry.order.clone());
            if let Some(parent) = parent.filter(|parent| child_changes.contains(parent)) {
                if let Some(entry) = self.nodes.get_mut(&parent) {
                    entry.children.insert(path, *key);
                }
            }
        }
        for parent in child_changes {
            if let Some(entry) = self.nodes.get_mut(&parent) {
                let children: Vec<_> = entry.children.values().copied().collect();
                if entry.node.children() != children.as_slice() {
                    entry.node.set_children(children);
                    self.pending.insert(parent);
                }
            }
        }
        self.root = Some(AccessKitNodeId(tree[root].get().instance_key));
        changed.len()
    }

    fn remove(&mut self, key: AccessKitNodeId, changed: &mut HashSet<AccessKitNodeId>) {
        if let Some(entry) = self.nodes.remove(&key) {
            if let Some(parent) = entry.parent {
                if let Some(parent_entry) = self.nodes.get_mut(&parent) {
                    parent_entry.children.remove(&entry.order);
                }
                changed.insert(parent);
            }
            let mut order = entry.order;
            order.push(usize::MAX);
            self.focused.remove(&order);
        }
        self.pending.remove(&key);
    }

    pub(crate) fn set_label(&mut self, label: Option<&str>, root: ComponentNodeId) {
        if self.root_label.as_deref() != label {
            self.root_label = label.map(str::to_owned);
            self.changed.insert(root);
        }
    }

    pub(crate) fn tree_update(
        &mut self,
        tree: &ComponentNodeTree,
        focused: Option<ComponentNodeId>,
        full: bool,
    ) -> Option<TreeUpdate> {
        let root = self.root?;
        let focus = focused
            .and_then(|focused| {
                tree.get(focused).filter(|node| !node.is_removed())?;
                focused.ancestors(tree).find_map(|id| {
                    let key = AccessKitNodeId(tree[id].get().instance_key);
                    self.nodes
                        .get(&key)
                        .filter(|entry| !entry.node.is_hidden())
                        .map(|_| key)
                })
            })
            .or_else(|| self.focused.last_key_value().map(|(_, key)| *key))
            .unwrap_or(root);
        if !full && self.pending.is_empty() && self.last_focus == Some(focus) {
            return None;
        }
        self.last_focus = Some(focus);
        let keys = if full {
            self.nodes.keys().copied().collect()
        } else {
            std::mem::take(&mut self.pending)
        };
        self.pending.clear();
        let mut nodes: Vec<_> = keys
            .into_iter()
            .filter_map(|key| self.nodes.get(&key).map(|entry| (key, entry)))
            .collect();
        nodes.sort_unstable_by(|(_, left), (_, right)| {
            left.order
                .iter()
                .copied()
                .chain(std::iter::once(usize::MAX))
                .cmp(
                    right
                        .order
                        .iter()
                        .copied()
                        .chain(std::iter::once(usize::MAX)),
                )
        });
        Some(TreeUpdate {
            nodes: nodes
                .into_iter()
                .map(|(key, entry)| (key, entry.node.clone()))
                .collect(),
            tree: Some(Tree::new(root)),
            tree_id: TreeId::ROOT,
            focus,
        })
    }
}

#[cfg(test)]
pub(crate) fn build_tree_update(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    root: ComponentNodeId,
    label: Option<&str>,
    focused: Option<ComponentNodeId>,
) -> Option<TreeUpdate> {
    let mut index = AccessibilityIndex::default();
    let changed = root.descendants(tree).collect();
    let order = root
        .descendants(tree)
        .flat_map(|id| id.children(tree).enumerate().map(|(order, id)| (id, order)))
        .collect();
    index.invalidate(&changed, &HashSet::default());
    index.set_label(label, root);
    index.update(tree, metadatas, root, &order);
    index.tree_update(tree, focused, true)
}

fn build_node(
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
    node_id: ComponentNodeId,
    is_root: bool,
    root_label: Option<&str>,
    inherited_tooltip: Option<&str>,
) -> Option<(Node, bool)> {
    let metadata = metadatas.get(&node_id)?;
    let accessibility_node = metadata.accessibility.clone();
    let abs_position = metadata.abs_position;
    let computed_data = metadata.computed_data;
    let tooltip_description = accessibility_node.as_ref().and_then(|node| {
        (node.focusable && node.description.is_none())
            .then(|| inherited_tooltip.map(str::to_owned))
            .flatten()
    });
    let mut focused = false;
    if let Some(accessibility_node) = accessibility_node {
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
        if let Some(description) = accessibility_node.description.or(tooltip_description) {
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
        if let Some(orientation) = accessibility_node.orientation {
            node.set_orientation(orientation);
        }

        if let Some(min) = accessibility_node.min_numeric_value {
            node.set_min_numeric_value(min);
        }

        if let Some(max) = accessibility_node.max_numeric_value {
            node.set_max_numeric_value(max);
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
            focused = true;
        }

        // Set toggled state
        if let Some(toggled) = accessibility_node.toggled {
            node.set_toggled(toggled);
        }
        if let Some(invalid) = accessibility_node.invalid {
            node.set_invalid(invalid);
        }
        if let Some(selected) = accessibility_node.selected {
            node.set_selected(selected);
        }
        if let Some(expanded) = accessibility_node.expanded {
            node.set_expanded(expanded);
        }
        if let Some(popup) = accessibility_node.has_popup {
            node.set_has_popup(popup);
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

        let bounds = match accessibility_node.bounds_padding {
            Some(padding) => rect_from_geometry_with_padding(abs_position, computed_data, padding),
            None => rect_from_geometry(abs_position, computed_data),
        };

        if let Some(bounds) = bounds {
            node.set_bounds(clipped_shape_bounds(bounds, node_id, tree, metadatas));
        }

        Some((node, focused))
    } else if is_root {
        let mut node = Node::new(accesskit::Role::Window);
        if let Some(label) = root_label {
            node.set_label(label.to_string());
        }

        if let Some(bounds) = rect_from_geometry(abs_position, computed_data) {
            node.set_bounds(bounds);
        }

        Some((node, false))
    } else {
        None
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

fn clipped_shape_bounds(
    mut bounds: Rect,
    node_id: ComponentNodeId,
    tree: &ComponentNodeTree,
    metadatas: &ComponentNodeMetaDatas,
) -> Rect {
    let Some(metadata) = metadatas
        .get(&node_id)
        .filter(|metadata| metadata.has_shape_clipping)
    else {
        return bounds;
    };
    let intersect = |bounds: Rect, clip: Rect| {
        let x0 = bounds.x0.max(clip.x0);
        let y0 = bounds.y0.max(clip.y0);
        Rect {
            x0,
            y0,
            x1: bounds.x1.min(clip.x1).max(x0),
            y1: bounds.y1.min(clip.y1).max(y0),
        }
    };
    if let Some(clip) = metadata.event_clip_rect {
        bounds = intersect(
            bounds,
            Rect {
                x0: clip.x.to_f32() as f64,
                y0: clip.y.to_f32() as f64,
                x1: (clip.x + clip.width).to_f32() as f64,
                y1: (clip.y + clip.height).to_f32() as f64,
            },
        );
    }
    for ancestor in node_id.ancestors(tree) {
        let Some(metadata) = metadatas.get(&ancestor) else {
            continue;
        };
        let (Some(clip), Some(origin)) = (metadata.clip_region, metadata.abs_position) else {
            continue;
        };
        let origin = origin + clip.position;
        bounds = intersect(
            bounds,
            Rect {
                x0: origin.x as f64,
                y0: origin.y as f64,
                x1: (origin.x + clip.size[0]) as f64,
                y1: (origin.y + clip.size[1]) as f64,
            },
        );
    }
    bounds
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
    if action_request.target_tree != TreeId::ROOT {
        return false;
    }

    let accessibility_id = AccessibilityId::from_accesskit_id(action_request.target_node);

    let component_node_id = tree
        .iter()
        .find(|node| !node.is_removed() && node.get().instance_key == accessibility_id.0)
        .and_then(|node| tree.get_node_id(node));

    if action_request.action == accesskit::Action::Focus {
        let Some(node_id) = component_node_id else {
            return false;
        };
        let eligible = metadatas.get(&node_id).is_some_and(|metadata| {
            metadata.abs_position.is_some()
                && metadata.accessibility.as_ref().is_some_and(|semantics| {
                    semantics.focusable && !semantics.disabled && !semantics.hidden
                })
        });
        let registration = tree[node_id].get().focus_registration;
        if eligible && let Some(registration) = registration {
            focus_owner.request_focus(registration.id);
            return true;
        }
        return false;
    }

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
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

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
            layout_children: None,
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
        tree[id].get_mut().instance_key = usize::from(id) as u64;
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

    #[test]
    fn semantic_deltas_preserve_siblings_and_refresh_inherited_tooltips() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let parent = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode {
                tooltip: Some("First hint".into()),
                ..AccessibilityNode::new()
            }),
        );
        let child = append_node(
            &mut tree,
            &mut metadatas,
            Some(parent),
            Some(AccessibilityNode {
                label: Some("Control".into()),
                focusable: true,
                ..AccessibilityNode::new()
            }),
        );
        let sibling = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode {
                label: Some("Sibling".into()),
                ..AccessibilityNode::new()
            }),
        );
        let order = [(parent, 0), (child, 0), (sibling, 1)]
            .into_iter()
            .collect();
        let mut index = AccessibilityIndex::default();
        index.invalidate(
            &[root, parent, child, sibling].into_iter().collect(),
            &HashSet::default(),
        );
        index.update(&tree, &metadatas, root, &order);
        let initial = index.tree_update(&tree, Some(child), true).unwrap();
        assert_eq!(
            initial
                .nodes
                .iter()
                .map(|(key, _)| *key)
                .collect::<Vec<_>>(),
            [child, parent, sibling, root].map(|id| AccessKitNodeId(tree[id].get().instance_key))
        );
        let child_key = AccessKitNodeId(tree[child].get().instance_key);
        assert_eq!(
            initial
                .nodes
                .iter()
                .find(|(id, _)| *id == child_key)
                .unwrap()
                .1
                .description(),
            Some("First hint")
        );
        assert_eq!(index.update(&tree, &metadatas, root, &order), 0);
        assert!(index.tree_update(&tree, Some(child), false).is_none());

        metadatas
            .get_mut(&parent)
            .unwrap()
            .accessibility
            .as_mut()
            .unwrap()
            .tooltip = Some("Updated hint".into());
        index.invalidate(&[parent].into_iter().collect(), &HashSet::default());
        assert_eq!(index.update(&tree, &metadatas, root, &order), 2);
        let delta = index.tree_update(&tree, Some(child), false).unwrap();
        assert_eq!(delta.nodes.len(), 2);
        assert_eq!(delta.focus, child_key);
        assert_eq!(
            delta
                .nodes
                .iter()
                .find(|(id, _)| *id == child_key)
                .unwrap()
                .1
                .description(),
            Some("Updated hint")
        );
        assert!(
            !delta
                .nodes
                .iter()
                .any(|(id, _)| *id == AccessKitNodeId(tree[sibling].get().instance_key))
        );
    }

    #[test]
    fn semantic_value_replacement_keeps_parent_children_and_stable_identity() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let old = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode {
                label: Some("Before".into()),
                ..AccessibilityNode::new()
            }),
        );
        let key = tree[old].get().instance_key;
        let mut index = AccessibilityIndex::default();
        index.invalidate(&[root, old].into_iter().collect(), &HashSet::default());
        index.update(&tree, &metadatas, root, &[(old, 0)].into_iter().collect());
        let initial = index.tree_update(&tree, None, true).unwrap();
        let children = initial
            .nodes
            .iter()
            .find(|(id, _)| *id == AccessKitNodeId(tree[root].get().instance_key))
            .unwrap()
            .1
            .children()
            .to_vec();
        old.remove_subtree(&mut tree);
        metadatas.remove(&old);
        let new = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode {
                label: Some("After".into()),
                ..AccessibilityNode::new()
            }),
        );
        tree[new].get_mut().instance_key = key;
        index.invalidate(&[new].into_iter().collect(), &[old].into_iter().collect());
        index.update(&tree, &metadatas, root, &[(new, 0)].into_iter().collect());
        let delta = index.tree_update(&tree, None, false).unwrap();
        assert_eq!(delta.nodes.len(), 1);
        assert_eq!(delta.nodes[0].0, AccessKitNodeId(key));
        assert_eq!(delta.nodes[0].1.label(), Some("After"));
        assert_eq!(
            index
                .tree_update(&tree, None, true)
                .unwrap()
                .nodes
                .iter()
                .find(|(id, _)| *id == AccessKitNodeId(tree[root].get().instance_key))
                .unwrap()
                .1
                .children(),
            children
        );
    }

    #[test]
    fn fractional_shape_masks_clip_semantic_bounds_without_removing_controls() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let wrapper = append_node(&mut tree, &mut metadatas, Some(root), None);
        let child = append_node(
            &mut tree,
            &mut metadatas,
            Some(wrapper),
            Some(
                AccessibilityNode::new()
                    .with_role(Role::Button)
                    .with_label("Masked action")
                    .with_action(Action::Click)
                    .focusable()
                    .focused(),
            ),
        );
        let metadata = metadatas.get_mut(&wrapper).expect("wrapper metadata");
        metadata.clip_region = Some(crate::ClipRegion {
            position: PxPosition::from_f32_xy(5.0, 3.0),
            size: [10.25, 12.5],
            shape: crate::ClipShape::Ellipse,
        });
        metadata.has_shape_clipping = true;
        metadatas
            .get_mut(&child)
            .expect("child metadata")
            .has_shape_clipping = true;
        let update =
            build_tree_update(&tree, &metadatas, root, Some("Fixture"), None).expect("tree update");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(
            node(&update, child).bounds(),
            Some(Rect {
                x0: 15.0,
                y0: 23.0,
                x1: 25.25,
                y1: 35.5
            })
        );
        assert!(node(&update, child).supports_action(Action::Click));
        assert!(node(&update, child).supports_action(Action::Focus));
        assert_eq!(update.focus, accesskit_id(child));
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

        let update = build_tree_update(&tree, &metadatas, root, Some("Calendar"), None)
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

        let update = build_tree_update(&tree, &metadatas, root, None, None)
            .expect("valid tree must produce an accessibility update");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(node(&update, root).children(), &[accesskit_id(button)]);
        assert!(node(&update, button).children().is_empty());
        assert_eq!(update.focus, accesskit_id(root));
    }

    #[test]
    fn rebuilt_controls_keep_identity_and_receive_actions_at_their_new_arena_index() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let slider = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode::new().with_role(Role::Slider).focused()),
        );
        tree[slider].get_mut().instance_key = 700;
        let before = build_tree_update(&tree, &metadatas, root, None, None).unwrap();
        let target = before.focus;
        assert_eq!(target, AccessKitNodeId(700));

        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let decoration = append_node(&mut tree, &mut metadatas, Some(root), None);
        let slider = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode::new().with_role(Role::Slider).focused()),
        );
        tree[slider].get_mut().instance_key = 700;
        let after = build_tree_update(&tree, &metadatas, root, None, None).unwrap();
        assert_eq!(after.focus, target);
        assert_eq!(
            after
                .nodes
                .iter()
                .find(|(id, _)| *id == target)
                .unwrap()
                .1
                .role(),
            Role::Slider
        );

        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        metadatas
            .get_mut(&slider)
            .unwrap()
            .accessibility_action_handler = Some(Box::new(move |action| {
            assert_eq!(action, Action::Increment);
            observed.fetch_add(1, Ordering::Relaxed);
        }));
        metadatas
            .get_mut(&decoration)
            .unwrap()
            .accessibility_action_handler = Some(Box::new(|_| {
            panic!("an arena index must not receive another control's action")
        }));
        let mut focus_owner = FocusOwner::new();
        let request = || accesskit::ActionRequest {
            action: Action::Increment,
            target_tree: TreeId::ROOT,
            target_node: target,
            data: None,
        };
        assert!(dispatch_action(
            &tree,
            &metadatas,
            &mut focus_owner,
            request()
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        tree[slider].get_mut().instance_key = 701;
        assert!(!dispatch_action(
            &tree,
            &metadatas,
            &mut focus_owner,
            request()
        ));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn contextual_help_describes_focusable_descendants_without_overriding_explicit_descriptions() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let mut help = AccessibilityNode::new().with_role(Role::Group);
        help.tooltip = Some("Contextual help".to_string());
        let anchor = append_node(&mut tree, &mut metadatas, Some(root), Some(help));
        let wrapper = append_node(&mut tree, &mut metadatas, Some(anchor), None);
        let button = append_node(
            &mut tree,
            &mut metadatas,
            Some(wrapper),
            Some(
                AccessibilityNode::new()
                    .with_role(Role::Button)
                    .with_label("Action")
                    .focusable(),
            ),
        );
        let mut custom = AccessibilityNode::new().with_role(Role::Button).focusable();
        custom.description = Some("Explicit description".to_string());
        let explicit = append_node(&mut tree, &mut metadatas, Some(anchor), Some(custom));
        let update = build_tree_update(&tree, &metadatas, root, None, None).expect("tree");
        assert_eq!(node(&update, button).label(), Some("Action"));
        assert_eq!(node(&update, button).description(), Some("Contextual help"));
        assert_eq!(
            node(&update, explicit).description(),
            Some("Explicit description")
        );
    }

    #[test]
    fn runtime_focus_overrides_stale_semantic_focus_and_survives_layout_wrappers() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let stale = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode::new().with_role(Role::Button).focused()),
        );
        let current = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(
                AccessibilityNode::new()
                    .with_role(Role::Button)
                    .with_label("Current")
                    .focusable(),
            ),
        );
        let wrapper = append_node(&mut tree, &mut metadatas, Some(current), None);
        let update = build_tree_update(&tree, &metadatas, root, None, Some(wrapper)).expect("tree");
        assert_eq!(update.focus, accesskit_id(current));
        assert_ne!(update.focus, accesskit_id(stale));
    }

    #[test]
    fn numeric_controls_publish_current_value_bounds_and_steps() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let mut semantics = AccessibilityNode::new().with_role(Role::Slider);
        semantics.numeric_value = Some(0.4);
        semantics.min_numeric_value = Some(0.2);
        semantics.max_numeric_value = Some(0.8);
        semantics.numeric_value_step = Some(0.01);
        semantics.numeric_value_jump = Some(0.1);
        semantics.orientation = Some(accesskit::Orientation::Vertical);
        let slider = append_node(&mut tree, &mut metadatas, Some(root), Some(semantics));

        let update = build_tree_update(&tree, &metadatas, root, None, None).unwrap();
        let slider = node(&update, slider);
        assert_eq!(slider.numeric_value(), Some(0.4));
        assert_eq!(slider.min_numeric_value(), Some(0.2));
        assert_eq!(slider.max_numeric_value(), Some(0.8));
        assert_eq!(slider.numeric_value_step(), Some(0.01));
        assert_eq!(slider.numeric_value_jump(), Some(0.1));
        assert_eq!(slider.orientation(), Some(accesskit::Orientation::Vertical));
    }

    #[test]
    fn menu_trigger_publishes_expanded_and_popup_type() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let mut semantics = AccessibilityNode::new()
            .with_role(Role::Button)
            .with_label("Create");
        semantics.expanded = Some(true);
        semantics.has_popup = Some(accesskit::HasPopup::Menu);
        let trigger = append_node(&mut tree, &mut metadatas, Some(root), Some(semantics));
        let update = build_tree_update(&tree, &metadatas, root, None, None).unwrap();
        assert_eq!(node(&update, trigger).is_expanded(), Some(true));
        assert_eq!(
            node(&update, trigger).has_popup(),
            Some(accesskit::HasPopup::Menu)
        );
    }

    #[test]
    fn button_selection_distinguishes_selected_unselected_and_not_applicable() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let button = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode::new().with_role(Role::Button)),
        );
        for selected in [Some(true), Some(false), None] {
            metadatas
                .get_mut(&button)
                .expect("semantic fixture")
                .accessibility
                .as_mut()
                .expect("semantic fixture")
                .selected = selected;
            let update =
                build_tree_update(&tree, &metadatas, root, None, None).expect("semantic fixture");
            assert_eq!(node(&update, button).is_selected(), selected);
            assert_eq!(node(&update, button).toggled(), None);
        }
    }

    #[test]
    fn focus_actions_reach_registered_targets_without_activating_their_handlers() {
        use crate::focus::{FocusNode, FocusRegistration};

        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let target = append_node(
            &mut tree,
            &mut metadatas,
            Some(root),
            Some(AccessibilityNode::new().with_role(Role::Button).focusable()),
        );
        tree[target].get_mut().focus_registration =
            Some(FocusRegistration::target(FocusNode::new()));
        metadatas
            .get_mut(&target)
            .expect("semantic fixture")
            .accessibility_action_handler = Some(Box::new(|_| {
            panic!("a focus action must not activate a control");
        }));
        let mut owner = FocusOwner::new();
        owner.sync_from_component_tree(root, &tree);
        owner.sync_layout_from_component_tree(root, &tree, &metadatas);
        let request = || accesskit::ActionRequest {
            action: Action::Focus,
            target_tree: TreeId::ROOT,
            target_node: accesskit_id(target),
            data: None,
        };
        assert!(dispatch_action(&tree, &metadatas, &mut owner, request()));
        assert_eq!(owner.active_component_node_id(), Some(target));
        owner.clear_focus_global(true);
        metadatas
            .get_mut(&target)
            .expect("semantic fixture")
            .accessibility
            .as_mut()
            .expect("semantic fixture")
            .disabled = true;
        assert!(!dispatch_action(&tree, &metadatas, &mut owner, request()));
        assert_eq!(owner.active_component_node_id(), None);
        metadatas
            .get_mut(&target)
            .expect("semantic fixture")
            .accessibility
            .as_mut()
            .expect("semantic fixture")
            .disabled = false;
        metadatas
            .get_mut(&target)
            .expect("semantic fixture")
            .abs_position = None;
        assert!(!dispatch_action(&tree, &metadatas, &mut owner, request()));
        assert_eq!(owner.active_component_node_id(), None);
    }

    #[test]
    fn tree_without_semantics_keeps_only_the_window_root() {
        let mut tree = ComponentNodeTree::new();
        let mut metadatas = ComponentNodeMetaDatas::new();
        let root = append_node(&mut tree, &mut metadatas, None, None);
        let wrapper = append_node(&mut tree, &mut metadatas, Some(root), None);
        append_node(&mut tree, &mut metadatas, Some(wrapper), None);

        let update = build_tree_update(&tree, &metadatas, root, Some("Empty"), None)
            .expect("valid tree must produce an accessibility update");
        assert_eq!(update.nodes.len(), 1);
        assert_eq!(node(&update, root).role(), Role::Window);
        assert!(node(&update, root).children().is_empty());
        assert_eq!(update.focus, accesskit_id(root));
    }
}
