//! Consumer ancestry across retained component replay.
//!
//! ## Usage
//!
//! Preserve interaction and semantics when retained controls move.

use rustc_hash::FxHashMap as HashMap;

use crate::{
    NodeId,
    focus::{FocusHandleId, FocusRegistrationKind},
};

use super::ComponentTree;

#[derive(Default)]
pub(super) struct ReplayAncestry {
    detached_root: Option<NodeId>,
    external_ancestors: Vec<NodeId>,
    reused: HashMap<NodeId, ConsumerAncestry>,
}

#[derive(PartialEq, Eq)]
struct ConsumerAncestry {
    order: Vec<usize>,
    semantics: Vec<(u64, bool, Option<String>)>,
    focus: Vec<(FocusHandleId, FocusRegistrationKind)>,
    coordinate_space: Option<u64>,
}

impl ComponentTree {
    pub(super) fn begin_replay_ancestry(&mut self, root: NodeId, parent: NodeId) {
        self.replay_ancestry = ReplayAncestry {
            detached_root: Some(root),
            external_ancestors: parent.ancestors(&self.tree).collect(),
            reused: HashMap::default(),
        };
    }

    pub(super) fn capture_reused_ancestry(&mut self, root: NodeId) {
        let ancestry = self.consumer_ancestry(root, true);
        self.replay_ancestry.reused.insert(root, ancestry);
    }

    fn consumer_ancestry(&self, root: NodeId, detached: bool) -> ConsumerAncestry {
        let mut ancestors = root.ancestors(&self.tree).collect::<Vec<_>>();
        if detached && ancestors.last().copied() == self.replay_ancestry.detached_root {
            ancestors.extend(self.replay_ancestry.external_ancestors.iter().copied());
        }
        let mut result = ConsumerAncestry {
            order: Vec::new(),
            semantics: Vec::new(),
            focus: Vec::new(),
            coordinate_space: None,
        };
        for id in ancestors {
            let node = self.tree[id].get();
            if result.coordinate_space.is_none()
                && node.layout_coordinate_space
                && node.role == super::NodeRole::Layout
            {
                result.coordinate_space = Some(node.instance_key);
            }
            result
                .order
                .push(self.composition_order.get(&id).copied().unwrap_or_default());
            if let Some(semantics) = self
                .metadatas
                .get(&id)
                .and_then(|metadata| metadata.accessibility.as_ref())
            {
                result.semantics.push((
                    node.instance_key,
                    semantics.merge_descendants,
                    semantics.tooltip.clone(),
                ));
            }
            if let Some(focus) = node.focus_registration {
                result.focus.push((focus.id, focus.kind));
            }
        }
        result
    }

    pub(super) fn finish_replay_ancestry(&mut self) {
        let reused = std::mem::take(&mut self.replay_ancestry.reused);
        for (root, previous) in reused {
            if self.tree.get(root).is_none_or(|node| node.is_removed()) {
                continue;
            }
            let current = self.consumer_ancestry(root, false);
            if current.coordinate_space != previous.coordinate_space {
                self.pending_position_observers.extend(
                    root.descendants(&self.tree)
                        .filter(|id| self.position_observer_nodes.contains(id)),
                );
            }
            if current != previous {
                tracing::debug!(
                    target: "tessera_ui::replay",
                    fn_name = %self.tree[root].get().fn_name,
                    previous_order = ?previous.order,
                    current_order = ?current.order,
                    previous_semantics = ?previous.semantics,
                    current_semantics = ?current.semantics,
                    focus_changed = current.focus != previous.focus,
                    "retained_ancestry_changed"
                );
                self.changed_frame_nodes
                    .extend(root.descendants(&self.tree));
            }
        }
        self.replay_ancestry = ReplayAncestry::default();
    }
}
