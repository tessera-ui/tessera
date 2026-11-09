//! Typed snapshots for inspecting component and accessibility output.
//!
//! ## Usage
//!
//! Locate test targets by tags, labels, roles, and their published ancestry.

use std::{collections::HashMap, num::NonZero};

use accesskit::{Node, NodeId as SemanticId, Role, TreeUpdate};

use crate::{
    AccessibilityNode, FloatRect, LayoutCoordinates, LocalToWindow, NodeId, PxSize,
    RuntimeSessionId, runtime::TesseraRuntime,
};

fn visible_character(character: &char) -> bool {
    !matches!(*character, '\u{2066}'..='\u{2069}')
}

/// Compares displayed text, ignoring Unicode bidirectional isolates.
pub fn text_matches(actual: Option<&str>, expected: &str) -> bool {
    actual.is_some_and(|actual| {
        actual
            .chars()
            .filter(visible_character)
            .eq(expected.chars().filter(visible_character))
    })
}

/// Returns displayed text without Unicode bidirectional isolates.
pub fn visible_text(text: &str) -> String {
    text.chars().filter(visible_character).collect()
}

/// Component geometry and semantics captured from a completed frame.
pub struct ComponentSnapshot {
    session: RuntimeSessionId,
    nodes: Vec<ComponentSnapshotNode>,
}

/// One component's observed output in a snapshot.
pub struct ComponentSnapshotNode {
    id: NodeId,
    parent: Option<NodeId>,
    name: &'static str,
    instance: u64,
    coordinates: Option<LayoutCoordinates>,
    semantics: Option<AccessibilityNode>,
}

impl ComponentSnapshotNode {
    /// Returns the arena node identity observed in this session's tree.
    pub fn node_id(&self) -> NodeId {
        self.id
    }

    /// Returns the stable component identity within its UI session.
    pub fn instance_id(&self) -> u64 {
        self.instance
    }

    /// Returns the component function name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns the observed window bounds in physical pixels.
    pub fn bounds(&self) -> Option<FloatRect> {
        self.coordinates.map(LayoutCoordinates::window_bounds)
    }

    /// Returns the component layout node's local size and window transform.
    pub fn layout_coordinates(&self) -> Option<LayoutCoordinates> {
        self.coordinates
    }

    /// Returns the component's semantic metadata.
    pub fn semantics(&self) -> Option<&AccessibilityNode> {
        self.semantics.as_ref()
    }
}

impl ComponentSnapshot {
    pub(crate) fn capture() -> Self {
        TesseraRuntime::with(|runtime| {
            let tree = runtime.component_tree.tree();
            let metadata = runtime.component_tree.metadatas();
            let nodes = tree
                .get_node_id_at(NonZero::<usize>::MIN)
                .into_iter()
                .flat_map(|root| root.descendants(tree))
                .map(|id| {
                    let node = tree.get(id).expect("snapshot component").get();
                    let data = metadata.get(&id);
                    let coordinates = data.and_then(|data| {
                        let size = data.computed_data?;
                        let origin = data.abs_position?;
                        let transform = data.local_to_window.unwrap_or_else(|| {
                            LocalToWindow::new(1.0, origin).expect("finite placed origin")
                        });
                        Some(LayoutCoordinates {
                            local_size: PxSize::new(size.width, size.height),
                            local_to_window: transform,
                        })
                    });
                    ComponentSnapshotNode {
                        id,
                        parent: id.ancestors(tree).nth(1),
                        instance: node.instance_logic_id,
                        name: node.fn_name,
                        coordinates,
                        semantics: data.and_then(|data| {
                            data.accessibility
                                .as_ref()
                                .map(|semantics| semantics.snapshot())
                        }),
                    }
                })
                .collect();
            Self {
                nodes,
                session: crate::RuntimeSession::current()
                    .expect("snapshot session")
                    .identity(),
            }
        })
    }

    /// Returns nodes in declaration tree order.
    pub fn nodes(&self) -> &[ComponentSnapshotNode] {
        &self.nodes
    }

    /// Returns the UI session that owns the observed component identities.
    pub fn session_id(&self) -> RuntimeSessionId {
        self.session
    }

    /// Returns components whose labels match, ignoring bidirectional isolates.
    pub fn matching_label<'a>(
        &'a self,
        label: &str,
    ) -> impl Iterator<Item = &'a ComponentSnapshotNode> {
        self.nodes.iter().filter(move |node| {
            text_matches(
                node.semantics
                    .as_ref()
                    .and_then(|value| value.label.as_deref()),
                label,
            )
        })
    }

    /// Returns the uniquely tagged component, panicking if missing or ambiguous.
    pub fn tagged(&self, tag: &str) -> &ComponentSnapshotNode {
        self.find_tag(tag)
            .unwrap_or_else(|| panic!("missing component tag `{tag}`"))
    }

    /// Finds a uniquely tagged component, panicking if the tag is ambiguous.
    pub fn find_tag(&self, tag: &str) -> Option<&ComponentSnapshotNode> {
        let mut found = self.nodes.iter().filter(|node| {
            node.semantics
                .as_ref()
                .and_then(|value| value.key.as_deref())
                == Some(tag)
        });
        let node = found.next()?;
        assert!(found.next().is_none(), "ambiguous component tag `{tag}`");
        Some(node)
    }

    /// Returns the uniquely labeled component, panicking if missing or ambiguous.
    pub fn labeled(&self, label: &str) -> &ComponentSnapshotNode {
        let mut found = self.matching_label(label);
        let node = found
            .next()
            .unwrap_or_else(|| panic!("missing component label `{label}`"));
        assert!(
            found.next().is_none(),
            "ambiguous component label `{label}`"
        );
        node
    }

    /// Returns the parent within this snapshot.
    pub fn parent(&self, node: &ComponentSnapshotNode) -> Option<&ComponentSnapshotNode> {
        assert!(
            self.nodes.iter().any(|entry| std::ptr::eq(entry, node)),
            "component belongs to another snapshot"
        );
        let parent = node.parent?;
        self.nodes.iter().find(|node| node.id == parent)
    }
}

/// The complete accessibility publication captured for one frame.
pub struct AccessibilitySnapshot {
    session: RuntimeSessionId,
    tree: TreeUpdate,
    indexes: HashMap<SemanticId, usize>,
    parents: HashMap<SemanticId, SemanticId>,
}

impl AccessibilitySnapshot {
    pub(crate) fn new(tree: TreeUpdate, session: RuntimeSessionId) -> Self {
        let indexes = tree
            .nodes
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (*id, index))
            .collect();
        let parents = tree
            .nodes
            .iter()
            .flat_map(|(id, node)| node.children().iter().map(move |child| (*child, *id)))
            .collect();
        Self {
            session,
            tree,
            indexes,
            parents,
        }
    }

    /// Returns published nodes in their snapshot order.
    pub fn nodes(&self) -> &[(SemanticId, Node)] {
        &self.tree.nodes
    }

    /// Visits published nodes in the reading order declared by their children.
    pub fn reading_order(&self) -> impl Iterator<Item = (SemanticId, &Node)> {
        let root = self.tree.tree.as_ref().map(|tree| tree.root).or_else(|| {
            self.tree
                .nodes
                .iter()
                .find(|(id, _)| !self.parents.contains_key(id))
                .map(|(id, _)| *id)
        });
        root.into_iter().flat_map(|root| self.subtree(root))
    }

    /// Visits a node and its descendants in published reading order.
    pub fn subtree(&self, root: SemanticId) -> impl Iterator<Item = (SemanticId, &Node)> {
        let mut pending = vec![root];
        std::iter::from_fn(move || {
            let id = pending.pop()?;
            let node = self.get(id).expect("complete accessibility publication");
            pending.extend(node.children().iter().rev().copied());
            Some((id, node))
        })
    }

    /// Returns the UI session that owns the published node identities.
    pub fn session_id(&self) -> RuntimeSessionId {
        self.session
    }

    /// Returns the published focus target.
    pub fn focus(&self) -> SemanticId {
        self.tree.focus
    }

    /// Looks up a published node by its stable accessibility identity.
    pub fn get(&self, id: SemanticId) -> Option<&Node> {
        self.indexes
            .get(&id)
            .map(|index| &self.tree.nodes[*index].1)
    }

    /// Finds a uniquely labeled node with a role, ignoring bidirectional isolates.
    /// Panics if the label and role match more than one node.
    pub fn find(&self, label: &str, role: Role) -> Option<(SemanticId, &Node)> {
        let mut found = self
            .tree
            .nodes
            .iter()
            .filter(|(_, node)| text_matches(node.label(), label) && node.role() == role);
        let (id, node) = found.next()?;
        assert!(
            found.next().is_none(),
            "ambiguous accessibility target `{label}` with role {role:?}"
        );
        Some((*id, node))
    }

    /// Finds the nearest matching ancestor, including the node itself.
    pub fn ancestor_with_role(&self, id: SemanticId, role: Role) -> Option<(SemanticId, &Node)> {
        let mut current = Some(id);
        while let Some(id) = current {
            let node = self.get(id)?;
            if node.role() == role {
                return Some((id, node));
            }
            current = self.parents.get(&id).copied();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AccessibilityActionHandler, ComputedData, Constraint, LayoutModifierNode, LayoutResult,
        MeasurementError, Modifier, Px, PxPosition, SemanticsModifierNode,
        layout::{MeasureScope, layout},
        tessera,
        testing::{layout_test, pump_layout_test_session, start_layout_test_session},
    };

    struct Target(&'static str, Role);

    impl SemanticsModifierNode for Target {
        fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
            node.key = Some(self.0.into());
            node.label = Some(format!("\u{2068}{}\u{2069}", self.0));
            node.role = Some(self.1);
        }
    }

    #[derive(Clone, PartialEq)]
    struct Figure(bool);

    impl LayoutModifierNode for Figure {
        fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
            let mut result = LayoutResult::new(ComputedData {
                width: Px(if self.0 { 50 } else { 30 }),
                height: Px(if self.0 { 40 } else { 20 }),
            });
            if self.0 {
                let child = scope.children()[0];
                child.measure(scope, &Constraint::exact(Px(30), Px(20)))?;
                result.place_child(child, PxPosition::from_f32_xy(1.25, 2.5));
            }
            Ok(result)
        }
    }

    #[tessera]
    fn query_figure() {
        layout()
            .policy(Figure(true))
            .modifier(Modifier::new().push_semantics(Target("parent", Role::Group)))
            .child(|| {
                layout()
                    .policy(Figure(false))
                    .modifier(Modifier::new().push_semantics(Target("target", Role::Button)));
            });
    }

    #[test]
    fn queries_preserve_fractional_layout_and_published_semantic_ancestry() {
        let mut session = start_layout_test_session(layout_test(query_figure));
        pump_layout_test_session(&mut session, 0);
        let components = ComponentSnapshot::capture();
        let target = components.tagged("target");
        let bounds = target.bounds().expect("test fixture provides bounds");
        assert_eq!(
            (bounds.x, bounds.y, bounds.width, bounds.height),
            (1.25, 2.5, 30.0, 20.0)
        );
        assert_eq!(
            components.labeled("target").instance_id(),
            target.instance_id()
        );
        assert!(components.find_tag("missing").is_none());

        let publication = TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.accessibility_update(None, true)
        })
        .expect("test fixture provides accessibility_update");
        let accessibility = AccessibilitySnapshot::new(publication, session.session_id());
        let (target, _) = accessibility
            .find("target", Role::Button)
            .expect("test fixture entry exists");
        let (parent, _) = accessibility
            .find("parent", Role::Group)
            .expect("test fixture entry exists");
        assert_eq!(
            accessibility
                .ancestor_with_role(target, Role::Group)
                .expect("test fixture provides ancestor_with_role")
                .0,
            parent
        );
        assert!(accessibility.find("target", Role::TextInput).is_none());
    }

    #[test]
    fn reading_order_follows_published_children_instead_of_node_storage_order() {
        let owner = crate::RuntimeSession::new();
        let mut root = Node::new(Role::Group);
        root.set_children(vec![SemanticId(2), SemanticId(1)]);
        let mut branch = Node::new(Role::Group);
        branch.set_children(vec![SemanticId(3)]);
        let snapshot = AccessibilitySnapshot::new(
            TreeUpdate {
                tree_id: accesskit::TreeId::ROOT,
                nodes: vec![
                    (SemanticId(1), Node::new(Role::Button)),
                    (SemanticId(3), Node::new(Role::Button)),
                    (SemanticId(0), root),
                    (SemanticId(2), branch),
                ],
                tree: Some(accesskit::TreeInfo::new(SemanticId(0))),
                focus: SemanticId(1),
            },
            owner.identity(),
        );
        assert_eq!(
            snapshot
                .reading_order()
                .map(|(id, _)| id.0)
                .collect::<Vec<_>>(),
            [0, 2, 3, 1]
        );
        assert_eq!(
            snapshot
                .subtree(SemanticId(2))
                .map(|(id, _)| id.0)
                .collect::<Vec<_>>(),
            [2, 3]
        );
    }
}
