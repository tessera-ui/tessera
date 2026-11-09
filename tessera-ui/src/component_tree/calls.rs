//! Component call records that carry identity outside the layout tree.
//!
//! ## Usage
//!
//! Replay and retain components that emit any number of layout nodes.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::SmallVec;

use crate::NodeId;

use super::{ComponentTree, PreviousBuildNode, ReplayRange, ReplayReplaceContext};

/// Call key of the synthetic record that hosts top-level calls in the root node.
pub(super) const ROOT_CALL: u64 = 0;

type CallKeys = SmallVec<[u64; 2]>;
pub(super) type CallOutput = SmallVec<[NodeId; 4]>;

pub(super) struct CallRecord {
    parent: u64,
    logic_id: u64,
    name: &'static str,
    node: Option<NodeId>,
    children: CallKeys,
}

/// A node emitted by the executing call together with its replaced value.
pub(crate) struct EmittedNode {
    pub(crate) id: NodeId,
    pub(crate) previous: Option<PreviousBuildNode>,
}

struct ActiveCall {
    key: u64,
    previous_children: CallKeys,
    emitted: Option<EmittedNode>,
}

#[derive(Default)]
pub(super) struct CallTable {
    records: HashMap<u64, CallRecord>,
    active: Vec<ActiveCall>,
    replay_root: Option<u64>,
    pub(super) executed: Vec<(u64, u64)>,
    pub(super) removed_keys: HashSet<u64>,
    pub(super) removed_logic_ids: HashSet<u64>,
}

impl CallTable {
    pub(super) fn clear(&mut self) {
        self.records.clear();
        self.active.clear();
        self.replay_root = None;
        self.clear_replay_results();
    }

    fn clear_replay_results(&mut self) {
        self.executed.clear();
        self.removed_keys.clear();
        self.removed_logic_ids.clear();
    }

    pub(super) fn set_root(&mut self, root: NodeId) {
        self.records.insert(
            ROOT_CALL,
            CallRecord {
                parent: ROOT_CALL,
                logic_id: 0,
                name: "layout",
                node: Some(root),
                children: CallKeys::new(),
            },
        );
    }

    fn parent_of_new_call(&self, key: u64) -> u64 {
        if self.replay_root == Some(key) {
            self.records[&key].parent
        } else {
            self.active.last().map_or(ROOT_CALL, |call| call.key)
        }
    }

    fn output(&self, key: u64, output: &mut CallOutput) {
        let record = &self.records[&key];
        match record.node {
            Some(node) => output.push(node),
            None => {
                for &child in &record.children {
                    self.output(child, output);
                }
            }
        }
    }

    pub(super) fn host(&self, key: u64) -> Option<NodeId> {
        let mut parent = self.records.get(&key)?.parent;
        loop {
            let record = self.records.get(&parent)?;
            if let Some(node) = record.node {
                return Some(node);
            }
            parent = record.parent;
        }
    }

    /// First node emitted after `key` in composition order within its host.
    fn anchor_after(&self, mut key: u64) -> Option<NodeId> {
        while key != ROOT_CALL {
            let parent = self.records[&key].parent;
            let siblings = &self.records[&parent].children;
            let index = siblings.iter().position(|&sibling| sibling == key)?;
            let mut output = CallOutput::new();
            for &sibling in &siblings[index + 1..] {
                self.output(sibling, &mut output);
                if let Some(&first) = output.first() {
                    return Some(first);
                }
            }
            if self.records[&parent].node.is_some() {
                return None;
            }
            key = parent;
        }
        None
    }

    fn remove(&mut self, key: u64) {
        let Some(record) = self.records.remove(&key) else {
            return;
        };
        self.removed_keys.insert(key);
        self.removed_logic_ids.insert(record.logic_id);
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

    pub(crate) fn has_call(&self, key: u64) -> bool {
        self.calls.records.contains_key(&key)
    }

    pub(crate) fn call_name(&self, key: u64) -> &'static str {
        self.calls.records[&key].name
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn call_subtree_len(&self, key: u64) -> u64 {
        let mut output = CallOutput::new();
        self.calls.output(key, &mut output);
        output
            .iter()
            .map(|&id| self.tree[id].get().subtree_len as u64)
            .sum()
    }

    /// Returns the nearest enclosing call key, excluding the root record.
    pub(crate) fn call_parent(&self, key: u64) -> Option<u64> {
        self.calls
            .records
            .get(&key)
            .map(|record| record.parent)
            .filter(|&parent| parent != ROOT_CALL)
    }

    /// Records a call before its body executes.
    pub(crate) fn begin_call(&mut self, key: u64, logic_id: u64, name: &'static str) {
        let parent = self.calls.parent_of_new_call(key);
        if self
            .calls
            .records
            .get(&key)
            .is_some_and(|record| record.node.is_none())
        {
            self.mark_call_output_paint_dirty(key);
        }
        let previous_children = match self.calls.records.get_mut(&key) {
            Some(record) => {
                record.parent = parent;
                record.logic_id = logic_id;
                std::mem::take(&mut record.children)
            }
            None => {
                self.calls.records.insert(
                    key,
                    CallRecord {
                        parent,
                        logic_id,
                        name,
                        node: None,
                        children: CallKeys::new(),
                    },
                );
                CallKeys::new()
            }
        };
        if self.calls.replay_root != Some(key) {
            self.register_child_call(parent, key);
        }
        self.calls.executed.push((key, logic_id));
        self.calls.active.push(ActiveCall {
            key,
            previous_children,
            emitted: None,
        });
    }

    /// Re-executing a call can leave retained drawing behind rebuilt content,
    /// and even an empty fragment may become drawable.
    fn mark_call_output_paint_dirty(&mut self, key: u64) {
        let mut output = CallOutput::new();
        self.calls.output(key, &mut output);
        for root in output {
            for descendant in root.descendants(&self.tree) {
                if self.tree[descendant].get().role == super::NodeRole::Layout {
                    self.metadatas.mark_paint_dirty(descendant);
                }
            }
        }
    }

    fn register_child_call(&mut self, parent: u64, key: u64) {
        if let Some(record) = self.calls.records.get_mut(&parent) {
            record.children.push(key);
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
        if let Some(record) = self.calls.records.get_mut(&call.key) {
            record.node = Some(id);
        }
        call.emitted = Some(EmittedNode { id, previous });
    }

    /// Names an emitted node after the component that requested it.
    pub(crate) fn executing_call_name(&self) -> &'static str {
        let key = self
            .calls
            .active
            .last()
            .expect("nodes are emitted by an executing call")
            .key;
        let parent = self.calls.records[&key].parent;
        self.calls.records[&parent].name
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
    pub(crate) fn finish_call(&mut self, key: u64) -> Option<EmittedNode> {
        let call = self.calls.active.pop().expect("balanced component calls");
        debug_assert_eq!(call.key, key, "component calls finish in order");
        let current: HashSet<u64> = self.calls.records[&key].children.iter().copied().collect();
        for stale in call.previous_children {
            if !current.contains(&stale) {
                self.calls.remove(stale);
            }
        }
        call.emitted
    }

    /// Reattaches every node emitted by an unchanged call.
    pub(crate) fn attach_retained_call(&mut self, key: u64, logic_id: u64) -> bool {
        if self.calls.replay_root == Some(key)
            || self
                .calls
                .records
                .get(&key)
                .is_none_or(|record| record.logic_id != logic_id)
        {
            return false;
        }
        let mut output = CallOutput::new();
        self.calls.output(key, &mut output);
        for id in output {
            let instance_key = self.tree[id].get().instance_key;
            let candidate = self
                .claim_replay_candidate(instance_key, |_| true)
                .expect("retained call output stays live");
            self.capture_reused_ancestry(candidate.id);
            self.attach_replayed_node(candidate, true);
            self.pop_node();
        }
        let parent = self.calls.parent_of_new_call(key);
        self.register_child_call(parent, key);
        true
    }

    /// Prepares the host range of a call for replay.
    pub(crate) fn begin_replace_call(&mut self, key: u64) -> ReplayReplaceContext {
        self.calls.clear_replay_results();
        let host = self.calls.host(key).expect("replayed call has a host");
        let mut output = CallOutput::new();
        self.calls.output(key, &mut output);
        let next_sibling = match output.last() {
            Some(&last) => last.next_sibling(&self.tree),
            None => self.calls.anchor_after(key),
        };
        self.replay_range = Some(ReplayRange {
            parent: host,
            next_sibling,
        });
        if let Some(&first) = output.first() {
            self.queue_replay_children(host, first, next_sibling);
        }
        self.calls.replay_root = Some(key);
        let previous_queue =
            (!self.node_queue.is_empty()).then(|| std::mem::take(&mut self.node_queue));
        self.node_queue.push(host);
        ReplayReplaceContext {
            parent_id: host,
            previous_queue,
        }
    }

    pub(super) fn finish_replay_root(&mut self) {
        self.calls.replay_root = None;
    }
}
