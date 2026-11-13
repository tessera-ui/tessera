//! Component call records that carry identity outside the layout tree.
//!
//! ## Usage
//!
//! Replay and retain components that emit any number of layout nodes.

use rustc_hash::FxHashMap as HashMap;
use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::{
    NodeId,
    runtime::{InstanceSlotCursor, ReplayNodeSnapshot},
};

use super::{ComponentTree, PreviousBuildNode, ReplayRange, ReplayReplaceContext};

new_key_type! {
    /// Dense handle of a call record.
    pub(crate) struct CallId;
}

type CallIds = SmallVec<[CallId; 2]>;
pub(super) type CallOutput = SmallVec<[NodeId; 4]>;

struct CallRecord {
    key: u64,
    parent: Option<CallId>,
    logic_id: u64,
    name: &'static str,
    node: Option<NodeId>,
    /// Children declared before `node` was emitted; their output precedes it
    /// in the host.
    leading: usize,
    children: CallIds,
    snapshot: Option<ReplayNodeSnapshot>,
    /// Build epoch in which `snapshot` was last replaced.
    snapshot_epoch: u64,
    slots: InstanceSlotCursor,
    mark: u64,
}

impl CallRecord {
    fn new(key: u64, parent: Option<CallId>, logic_id: u64, name: &'static str) -> Self {
        Self {
            key,
            parent,
            logic_id,
            name,
            node: None,
            leading: 0,
            children: CallIds::new(),
            snapshot: None,
            snapshot_epoch: 0,
            slots: InstanceSlotCursor::default(),
            mark: 0,
        }
    }
}

/// A node emitted by the executing call together with its replaced value.
pub(crate) struct EmittedNode {
    pub(crate) id: NodeId,
    pub(crate) previous: Option<PreviousBuildNode>,
}

struct ActiveCall {
    id: CallId,
    previous_children: CallIds,
    /// Position in `previous_children` expected to match the next child call.
    cursor: usize,
    /// The previous output of this call was already marked dirty by an ancestor.
    output_paint_marked: bool,
    emitted: Option<EmittedNode>,
}

/// How a component call should update its stored replay snapshot.
pub(crate) enum SnapshotUpdate {
    Keep,
    Refresh,
    Replace,
}

#[derive(Default)]
pub(super) struct CallTable {
    records: SlotMap<CallId, CallRecord>,
    index: HashMap<u64, CallId>,
    root: Option<CallId>,
    active: Vec<ActiveCall>,
    replay_root: Option<CallId>,
    epoch: u64,
    mark: u64,
    pub(super) executed: Vec<(u64, u64)>,
    pub(super) removed_keys: rustc_hash::FxHashSet<u64>,
    pub(super) removed_logic_ids: rustc_hash::FxHashSet<u64>,
    retired: Vec<ReplayNodeSnapshot>,
}

impl CallTable {
    pub(super) fn clear(&mut self) {
        self.retired.extend(
            self.records
                .drain()
                .filter_map(|(_, record)| record.snapshot),
        );
        self.index.clear();
        self.root = None;
        self.active.clear();
        self.replay_root = None;
        self.clear_replay_results();
    }

    fn clear_replay_results(&mut self) {
        self.executed.clear();
        self.removed_keys.clear();
        self.removed_logic_ids.clear();
    }

    fn set_root(&mut self, root: NodeId) {
        let mut record = CallRecord::new(0, None, 0, "layout");
        record.node = Some(root);
        self.root = Some(self.records.insert(record));
    }

    pub(super) fn id(&self, key: u64) -> Option<CallId> {
        self.index.get(&key).copied()
    }

    pub(super) fn live_logic_id(&self, key: u64) -> Option<u64> {
        self.id(key).map(|id| self.records[id].logic_id)
    }

    fn parent_of_new_call(&self, id: Option<CallId>) -> Option<CallId> {
        match id {
            Some(id) if self.replay_root == Some(id) => self.records[id].parent,
            _ => self.active.last().map(|call| call.id).or(self.root),
        }
    }

    /// Finds the record of a declared call, preferring its previous position.
    fn locate(&mut self, key: u64) -> Option<CallId> {
        if let Some(call) = self.active.last_mut()
            && let Some(&candidate) = call.previous_children.get(call.cursor)
            && self
                .records
                .get(candidate)
                .is_some_and(|record| record.key == key)
        {
            call.cursor += 1;
            return Some(candidate);
        }
        self.id(key)
    }

    fn output(&self, id: CallId, output: &mut CallOutput) {
        let record = &self.records[id];
        let hosted = match record.node {
            Some(_) => &record.children[..record.leading],
            None => &record.children[..],
        };
        for &child in hosted {
            self.output(child, output);
        }
        output.extend(record.node);
    }

    /// Whether `child` is laid out inside the node emitted by `parent`.
    fn inside_node(&self, parent: CallId, child: CallId) -> bool {
        let record = &self.records[parent];
        record.node.is_some() && !record.children[..record.leading].contains(&child)
    }

    pub(super) fn host(&self, id: CallId) -> Option<NodeId> {
        let mut id = id;
        loop {
            let parent = self.records.get(id)?.parent?;
            if self.inside_node(parent, id) {
                return self.records[parent].node;
            }
            id = parent;
        }
    }

    /// First node emitted after `id` in composition order within its host.
    fn anchor_after(&self, mut id: CallId) -> Option<NodeId> {
        while let Some(parent) = self.records[id].parent {
            let record = &self.records[parent];
            let index = record.children.iter().position(|&sibling| sibling == id)?;
            let inside = self.inside_node(parent, id);
            let end = if record.node.is_some() && !inside {
                record.leading
            } else {
                record.children.len()
            };
            let mut output = CallOutput::new();
            for &sibling in &record.children[index + 1..end] {
                self.output(sibling, &mut output);
                if let Some(&first) = output.first() {
                    return Some(first);
                }
            }
            if inside {
                return None;
            }
            if let Some(node) = record.node {
                return Some(node);
            }
            id = parent;
        }
        None
    }

    fn remove(&mut self, id: CallId) {
        let Some(record) = self.records.remove(id) else {
            return;
        };
        self.index.remove(&record.key);
        self.removed_keys.insert(record.key);
        self.removed_logic_ids.insert(record.logic_id);
        self.retired.extend(record.snapshot);
        for child in record.children {
            self.remove(child);
        }
    }
}

impl ComponentTree {
    /// Creates the root node that hosts top-level calls.
    pub(crate) fn begin_build_root(&mut self) {
        let root = self.add_node(super::ComponentNode::new(
            "__tessera_build_root",
            super::NodeRole::Group,
        ));
        self.calls.set_root(root);
    }

    /// Starts a build pass; snapshots replaced afterwards belong to it.
    pub(crate) fn begin_call_epoch(&mut self) {
        self.calls.epoch = self.calls.epoch.wrapping_add(1);
    }

    /// Releases replay snapshots removed since the last call, outside any
    /// runtime borrow.
    pub(crate) fn take_retired_snapshots(&mut self) -> Vec<ReplayNodeSnapshot> {
        std::mem::take(&mut self.calls.retired)
    }

    pub(crate) fn has_call(&self, key: u64) -> bool {
        self.calls.index.contains_key(&key)
    }

    pub(crate) fn call_name(&self, key: u64) -> &'static str {
        self.calls
            .id(key)
            .map_or("<missing>", |id| self.calls.records[id].name)
    }

    pub(crate) fn call_snapshot(&self, key: u64) -> Option<&ReplayNodeSnapshot> {
        let id = self.calls.id(key)?;
        self.calls.records[id].snapshot.as_ref()
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn call_subtree_len(&self, key: u64) -> u64 {
        let Some(id) = self.calls.id(key) else {
            return 0;
        };
        let mut output = CallOutput::new();
        self.calls.output(id, &mut output);
        output
            .iter()
            .map(|&id| self.tree[id].get().subtree_len as u64)
            .sum()
    }

    pub(crate) fn call_id(&self, key: u64) -> Option<CallId> {
        self.calls.id(key)
    }

    /// Groups adjacent dirty outputs under the same layout host. Calls with
    /// multiple or non-contiguous outputs keep their independent replay path.
    pub(crate) fn group_replay_roots(&self, roots: Vec<(u64, CallId)>) -> Vec<Vec<(u64, CallId)>> {
        let mut by_node = HashMap::default();
        let mut hosts = Vec::new();
        let mut seen_hosts = rustc_hash::FxHashSet::default();
        let mut batches = Vec::new();
        for root @ (_, id) in roots {
            let mut output = CallOutput::new();
            self.calls.output(id, &mut output);
            let Some(host) = self.calls.host(id) else {
                batches.push(vec![root]);
                continue;
            };
            if output.len() != 1 || self.tree[output[0]].parent() != Some(host) {
                batches.push(vec![root]);
                continue;
            }
            if seen_hosts.insert(host) {
                hosts.push(host);
            }
            by_node.insert(output[0], root);
        }
        for host in hosts {
            let mut adjacent = Vec::new();
            for child in host.children(&self.tree) {
                if let Some(root) = by_node.remove(&child) {
                    adjacent.push(root);
                } else if !adjacent.is_empty() {
                    batches.push(std::mem::take(&mut adjacent));
                }
            }
            if !adjacent.is_empty() {
                batches.push(adjacent);
            }
        }
        debug_assert!(by_node.is_empty());
        batches
    }

    /// Returns the keys of the calls enclosing `key`, nearest first.
    pub(crate) fn call_ancestors(&self, key: u64) -> impl Iterator<Item = u64> + '_ {
        self.call_ancestors_of(self.calls.id(key))
    }

    /// Returns the keys of the calls enclosing `id`, nearest first.
    pub(crate) fn call_ancestors_of(&self, id: Option<CallId>) -> impl Iterator<Item = u64> + '_ {
        let start = id.and_then(|id| self.calls.records.get(id)?.parent);
        std::iter::successors(start, |&id| self.calls.records[id].parent)
            .filter(|&id| Some(id) != self.calls.root)
            .map(|id| self.calls.records[id].key)
    }

    /// Finds the record of a declared call and decides how to update its
    /// replay snapshot.
    pub(crate) fn locate_call(
        &mut self,
        key: u64,
        decide: impl FnOnce(&ReplayNodeSnapshot, bool) -> SnapshotUpdate,
    ) -> (Option<CallId>, SnapshotUpdate) {
        let id = self.calls.locate(key);
        let update = id
            .and_then(|id| {
                let record = &self.calls.records[id];
                let executed = record.snapshot_epoch == self.calls.epoch;
                record
                    .snapshot
                    .as_ref()
                    .map(|snapshot| decide(snapshot, executed))
            })
            .unwrap_or(SnapshotUpdate::Replace);
        (id, update)
    }

    /// Stores the snapshot of an executed or retained call.
    pub(crate) fn store_call_snapshot(&mut self, id: CallId, snapshot: ReplayNodeSnapshot) {
        let record = &mut self.calls.records[id];
        record.snapshot_epoch = self.calls.epoch;
        self.calls.retired.extend(record.snapshot.replace(snapshot));
    }

    /// Updates a snapshot's declaration scope while keeping its props.
    pub(crate) fn refresh_call_snapshot(
        &mut self,
        id: CallId,
        refresh: impl FnOnce(&mut ReplayNodeSnapshot),
    ) {
        let record = &mut self.calls.records[id];
        record.snapshot_epoch = self.calls.epoch;
        if let Some(snapshot) = &mut record.snapshot {
            refresh(snapshot);
        }
    }

    /// Returns the slot order of the executing call when it owns `logic_id`.
    pub(crate) fn executing_call_slots(
        &mut self,
        logic_id: u64,
    ) -> Option<&mut InstanceSlotCursor> {
        let id = self.calls.active.last()?.id;
        let record = &mut self.calls.records[id];
        (record.logic_id == logic_id).then_some(&mut record.slots)
    }

    /// Records a call before its body executes.
    pub(crate) fn begin_call(
        &mut self,
        id: Option<CallId>,
        key: u64,
        logic_id: u64,
        name: &'static str,
    ) -> CallId {
        let parent = self.calls.parent_of_new_call(id);
        let inherited_paint = self
            .calls
            .active
            .last()
            .is_some_and(|call| call.output_paint_marked);
        let (id, previous_children, output_paint_marked) = match id {
            Some(id) => {
                let record = &self.calls.records[id];
                let output_paint_marked = if inherited_paint && record.parent == parent {
                    true
                } else if record.node.is_none() {
                    self.mark_call_output_paint_dirty(id)
                } else {
                    false
                };
                let record = &mut self.calls.records[id];
                record.parent = parent;
                record.logic_id = logic_id;
                // A re-executed call owns only the node it emits this time.
                record.node = None;
                record.leading = 0;
                (
                    id,
                    std::mem::take(&mut record.children),
                    output_paint_marked,
                )
            }
            None => {
                let id = self
                    .calls
                    .records
                    .insert(CallRecord::new(key, parent, logic_id, name));
                self.calls.index.insert(key, id);
                (id, CallIds::new(), false)
            }
        };
        if self.calls.replay_root != Some(id) {
            self.register_child_call(parent, id);
        }
        self.calls.executed.push((key, logic_id));
        self.calls.active.push(ActiveCall {
            id,
            previous_children,
            cursor: 0,
            output_paint_marked,
            emitted: None,
        });
        id
    }

    /// Re-executing a call can leave retained drawing behind rebuilt content,
    /// and even an empty fragment may become drawable.
    fn mark_call_output_paint_dirty(&mut self, id: CallId) -> bool {
        let mut output = CallOutput::new();
        self.calls.output(id, &mut output);
        let marked = !output.is_empty();
        for root in output {
            for descendant in root.descendants(&self.tree) {
                if self.tree[descendant].get().role == super::NodeRole::Layout {
                    self.metadatas.mark_paint_dirty(descendant);
                }
            }
        }
        marked
    }

    fn register_child_call(&mut self, parent: Option<CallId>, id: CallId) {
        if let Some(record) = parent.and_then(|parent| self.calls.records.get_mut(parent)) {
            record.children.push(id);
        }
    }

    /// Records the node emitted by the executing call.
    pub(crate) fn emit_call_node(&mut self, id: NodeId, previous: Option<PreviousBuildNode>) {
        let call = self
            .calls
            .active
            .last_mut()
            .expect("layout nodes are emitted by an executing call");
        assert!(call.emitted.is_none(), "a call emits at most one node");
        let record = &mut self.calls.records[call.id];
        record.node = Some(id);
        record.leading = record.children.len();
        call.emitted = Some(EmittedNode { id, previous });
    }

    /// Names an emitted node after the component that requested it.
    pub(crate) fn executing_call_name(&self) -> &'static str {
        let id = self
            .calls
            .active
            .last()
            .expect("nodes are emitted by an executing call")
            .id;
        self.calls.records[id].name
    }

    /// Keeps the previous drawing when it equals `draw`.
    pub(crate) fn set_emitted_node_draw<N>(&mut self, draw: N)
    where
        N: crate::modifier::DrawModifierNode + PartialEq,
    {
        let emitted = self
            .calls
            .active
            .last_mut()
            .and_then(|call| call.emitted.as_mut())
            .expect("draw configures the node emitted by the current component");
        let previous = emitted
            .previous
            .as_mut()
            .map(|previous| std::mem::take(&mut previous.draw))
            .filter(|previous| previous.matches(&draw));
        self.tree[emitted.id].get_mut().draw_node =
            previous.unwrap_or_else(|| crate::layout::DrawModifierHandle::new(draw));
    }

    /// Returns the node emitted by the executing call, if any.
    pub(crate) fn executing_call_node(&self) -> Option<NodeId> {
        self.calls
            .active
            .last()
            .and_then(|call| call.emitted.as_ref())
            .map(|emitted| emitted.id)
    }

    /// Ends the executing call, removing child calls it no longer declares.
    pub(crate) fn finish_call(&mut self) -> Option<EmittedNode> {
        let call = self.calls.active.pop().expect("balanced component calls");
        let calls = &mut self.calls;
        calls.records[call.id].slots.finish_epoch();
        if calls.records[call.id].children != call.previous_children {
            calls.mark = calls.mark.wrapping_add(1);
            let mark = calls.mark;
            let current = std::mem::take(&mut calls.records[call.id].children);
            for &child in &current {
                calls.records[child].mark = mark;
            }
            calls.records[call.id].children = current;
            for stale in call.previous_children {
                if calls
                    .records
                    .get(stale)
                    .is_some_and(|record| record.mark != mark && record.parent == Some(call.id))
                {
                    calls.remove(stale);
                }
            }
        }
        call.emitted
    }

    /// Reattaches every node emitted by an unchanged call.
    pub(crate) fn attach_retained_call(&mut self, id: CallId, logic_id: u64) -> bool {
        if self.calls.replay_root == Some(id) || self.calls.records[id].logic_id != logic_id {
            return false;
        }
        let mut output = CallOutput::new();
        self.calls.output(id, &mut output);
        for node in output {
            let instance_key = self.tree[node].get().instance_key;
            let candidate = self
                .claim_replay_candidate(instance_key, |_| true)
                .expect("retained call output stays live");
            self.capture_reused_ancestry(candidate.id);
            self.attach_replayed_node(candidate, true);
            self.pop_node();
        }
        let parent = self.calls.parent_of_new_call(Some(id));
        self.calls.records[id].parent = parent;
        self.register_child_call(parent, id);
        true
    }

    /// Returns the snapshot of a call that is still live.
    pub(crate) fn live_call_snapshot(&self, id: CallId) -> Option<ReplayNodeSnapshot> {
        self.calls.records.get(id)?.snapshot.clone()
    }

    /// Prepares the host range of a call for replay.
    pub(crate) fn begin_replace_call(&mut self, id: CallId) -> ReplayReplaceContext {
        self.calls.clear_replay_results();
        let host = self.calls.host(id).expect("replayed call has a host");
        let mut output = CallOutput::new();
        self.calls.output(id, &mut output);
        let next_sibling = match output.last() {
            Some(&last) => last.next_sibling(&self.tree),
            None => self.calls.anchor_after(id),
        };
        self.replay_range = Some(ReplayRange {
            parent: host,
            next_sibling,
        });
        if let Some(&first) = output.first() {
            self.queue_replay_children(host, first, next_sibling);
        }
        self.calls.replay_root = Some(id);
        let previous_queue =
            (!self.node_queue.is_empty()).then(|| std::mem::take(&mut self.node_queue));
        self.node_queue.push(host);
        ReplayReplaceContext {
            parent_id: host,
            previous_queue,
        }
    }

    /// Replaces one contiguous range for several sibling calls. Their bodies
    /// still execute in their own replay scopes and retain their own snapshots.
    pub(crate) fn begin_replace_batch(&mut self, roots: &[(u64, CallId)]) -> ReplayReplaceContext {
        assert!(!roots.is_empty(), "replay batch must contain a call");
        self.calls.clear_replay_results();
        let host = self
            .calls
            .host(roots[0].1)
            .expect("batched calls have a host");
        let mut first = CallOutput::new();
        let mut last = CallOutput::new();
        self.calls.output(roots[0].1, &mut first);
        self.calls
            .output(roots.last().expect("nonempty batch").1, &mut last);
        debug_assert!(
            roots
                .iter()
                .all(|&(_, id)| self.calls.host(id) == Some(host))
        );
        let next_sibling = last[0].next_sibling(&self.tree);
        self.replay_range = Some(ReplayRange {
            parent: host,
            next_sibling,
        });
        self.queue_replay_children(host, first[0], next_sibling);
        self.calls.replay_root = Some(roots[0].1);
        let previous_queue =
            (!self.node_queue.is_empty()).then(|| std::mem::take(&mut self.node_queue));
        self.node_queue.push(host);
        ReplayReplaceContext {
            parent_id: host,
            previous_queue,
        }
    }

    pub(crate) fn select_replay_batch_call(&mut self, id: CallId) {
        debug_assert!(self.replay_range.is_some());
        self.calls.replay_root = Some(id);
    }

    pub(super) fn finish_replay_root(&mut self) {
        self.calls.replay_root = None;
    }
}
