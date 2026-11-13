//! Consumer ancestry across retained component replay.
//!
//! ## Usage
//!
//! Preserve interaction and semantics when retained controls move.

use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;

use crate::{
    NodeId,
    focus::{FocusHandleId, FocusRegistrationKind},
};

use super::{ComponentNode, ComponentTree, NodeRole, sibling_index};

pub(super) struct ReplayCandidate {
    pub(super) id: NodeId,
    pub(super) keeps_link: bool,
}

#[derive(Default)]
pub(super) struct ReplayAncestry {
    pending_children: Vec<(NodeId, PendingChildren)>,
    pub(super) detached_roots: HashMap<NodeId, NodeId>,
    replaced: HashMap<NodeId, Ancestor>,
    reused: HashMap<NodeId, ConsumerAncestry>,
}

struct PendingChildren {
    first: NodeId,
    end: Option<NodeId>,
}

struct Ancestor {
    parent: Option<NodeId>,
    order: usize,
    semantics: Option<(u64, bool, Option<String>)>,
    focus: Option<(FocusHandleId, FocusRegistrationKind)>,
    coordinate_space: Option<u64>,
}

#[derive(Default, PartialEq, Eq)]
struct ConsumerAncestry {
    // Retained roots are captured every frame; typical depths stay inline.
    order: SmallVec<[usize; 32]>,
    semantics: Vec<(u64, bool, Option<String>)>,
    focus: Vec<(FocusHandleId, FocusRegistrationKind)>,
    coordinate_space: Option<u64>,
}

impl ReplayAncestry {
    pub(super) fn clear_detached(&mut self) {
        fn recycle<T>(entries: &mut HashMap<NodeId, T>) {
            if entries.capacity() > 256 {
                *entries = HashMap::default();
            } else {
                entries.clear();
            }
        }
        recycle(&mut self.detached_roots);
        recycle(&mut self.replaced);
        self.pending_children.clear();
        if self.pending_children.capacity() > 256 {
            self.pending_children.shrink_to(256);
        }
    }
}

impl ComponentTree {
    pub(super) fn queue_replay_children(
        &mut self,
        parent: NodeId,
        first: NodeId,
        end: Option<NodeId>,
    ) {
        let pending = &mut self.replay_ancestry.pending_children;
        if let Some((_, children)) = pending.iter_mut().rev().find(|(id, _)| *id == parent) {
            *children = PendingChildren { first, end };
        } else {
            pending.push((parent, PendingChildren { first, end }));
        }
    }

    pub(super) fn preserve_replay_children(&mut self, id: NodeId) {
        if let Some(first) = self.tree[id].first_child() {
            // Retained descendants need the ancestor's old inherited metadata
            // even when the ancestor keeps its arena ID.
            let previous = self.ancestor(id, true);
            self.replay_ancestry.replaced.insert(id, previous);
            self.queue_replay_children(id, first, None);
        }
    }

    pub(super) fn next_replay_child(&self, parent: NodeId) -> Option<NodeId> {
        self.replay_ancestry
            .pending_children
            .iter()
            .rev()
            .find_map(|(id, children)| (*id == parent).then_some(children.first))
    }

    pub(super) fn detach_replay_suffix(&mut self, parent: NodeId) {
        let children = &mut self.replay_ancestry.pending_children;
        let Some(index) = children.iter().rposition(|(id, _)| *id == parent) else {
            return;
        };
        let (_, pending) = children.remove(index);
        self.tree[parent].get_mut().layout_children_dirty = true;
        let mut next = Some(pending.first);
        while let Some(child) = next.filter(|&child| Some(child) != pending.end) {
            next = child.next_sibling(&self.tree);
            child.detach(&mut self.tree);
            self.replay_ancestry.detached_roots.insert(child, parent);
        }
    }

    pub(super) fn claim_replay_candidate(
        &mut self,
        instance_key: u64,
        compatible: impl FnOnce(&ComponentNode) -> bool,
    ) -> Option<ReplayCandidate> {
        if self.replay_ancestry.pending_children.is_empty()
            && self.replay_ancestry.detached_roots.is_empty()
        {
            return None;
        }
        if let Some(parent) = self.current_build_node_id()
            && let Some(index) = self
                .replay_ancestry
                .pending_children
                .iter()
                .rposition(|(id, _)| *id == parent)
            && self.tree[self.replay_ancestry.pending_children[index].1.first]
                .get()
                .instance_key
                == instance_key
        {
            let pending = &mut self.replay_ancestry.pending_children[index].1;
            let id = pending.first;
            if !compatible(self.tree[id].get()) {
                return None;
            }
            match id.next_sibling(&self.tree) {
                Some(next) if Some(next) != pending.end => pending.first = next,
                _ => {
                    self.replay_ancestry.pending_children.remove(index);
                }
            }
            return Some(ReplayCandidate {
                id,
                keeps_link: true,
            });
        }
        let id = self.find_node_id_by_instance_key(instance_key)?;
        let mut branch = id;
        while !self.replay_ancestry.detached_roots.contains_key(&branch) {
            let parent = self.tree[branch].parent()?;
            // Reordered children become individually claimable. Detaching a
            // pending suffix once avoids searching it for every moved child.
            self.detach_replay_suffix(parent);
            if self.replay_ancestry.detached_roots.contains_key(&branch) {
                break;
            }
            branch = parent;
        }
        compatible(self.tree[id].get()).then_some(ReplayCandidate {
            id,
            keeps_link: false,
        })
    }

    pub(super) fn capture_reused_ancestry(&mut self, root: NodeId) {
        let ancestry = self.consumer_ancestry(root, true);
        self.replay_ancestry.reused.insert(root, ancestry);
    }

    fn ancestor(&self, id: NodeId, previous: bool) -> Ancestor {
        let entry = &self.tree[id];
        let node = entry.get();
        let parent = entry.parent().or_else(|| {
            previous
                .then(|| self.replay_ancestry.detached_roots.get(&id).copied())
                .flatten()
        });
        Ancestor {
            parent,
            order: sibling_index(id, &self.tree),
            semantics: self
                .metadatas
                .get(&id)
                .and_then(|metadata| metadata.accessibility.as_ref())
                .map(|semantics| {
                    let (traverses, tooltip) = semantics.descendant_context();
                    (node.instance_key, traverses, tooltip.map(str::to_owned))
                }),
            focus: node
                .interaction()
                .focus_registration
                .map(|focus| (focus.id, focus.kind)),
            coordinate_space: (node.layout_coordinate_space && node.role == NodeRole::Layout)
                .then_some(node.instance_key),
        }
    }

    fn consumer_ancestry(&self, root: NodeId, previous: bool) -> ConsumerAncestry {
        let mut result = ConsumerAncestry::default();
        let mut next = Some(root);
        while let Some(id) = next {
            let saved = previous
                .then(|| self.replay_ancestry.replaced.get(&id))
                .flatten();
            let live;
            let ancestor = match saved {
                Some(saved) => saved,
                None => {
                    live = self.ancestor(id, previous);
                    &live
                }
            };
            result.order.push(ancestor.order);
            result.semantics.extend(ancestor.semantics.iter().cloned());
            result.focus.extend(ancestor.focus);
            if result.coordinate_space.is_none() {
                result.coordinate_space = ancestor.coordinate_space;
            }
            next = ancestor.parent;
        }
        result
    }

    pub(super) fn finish_replay_ancestry(&mut self) {
        let mut reused = std::mem::take(&mut self.replay_ancestry.reused);
        for (root, previous) in reused.drain() {
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
        self.replay_ancestry.reused = reused;
    }
}
