//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::hash_map::Entry,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

use hashbrown::{HashTable, hash_table::Entry as TableEntry};
use parking_lot::RwLock;
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet};
use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::{
    NodeId,
    accessibility::{AccessibilityActionHandler, AccessibilityNode},
    component_call::ComponentCalls,
    component_tree::{ComponentNode, ComponentTree, NodeRole, PreviousLayoutNode},
    context::ContextMap,
    execution_context::{
        ExecutionContext, OrderFrame, with_execution_context, with_execution_context_mut,
    },
    focus::{
        FocusDirection, FocusGroupNode, FocusHandleId, FocusNode, FocusProperties,
        FocusRegistration, FocusRegistrationKind, FocusRequester, FocusRequesterId,
        FocusRevealRequest, FocusScopeNode, FocusState, FocusTraversalPolicy,
    },
    inline_set::InlineSet,
    layout::LayoutModifierHandle,
    modifier::{DrawModifierNode, Modifier},
    prop::{CallbackWith, ErasedComponent, Prop},
    read_dependencies::ReadDependencies,
    session::{try_with_runtime_globals, with_runtime_globals},
    time::Instant,
};

mod state_owner;

pub use state_owner::StateOwner;

#[derive(Clone, Copy)]
enum OrderCounterKind {
    Remember,
    Functor,
    Context,
    FrameReceiver,
    ModifierAttachment,
}

fn push_order_frame() {
    with_execution_context_mut(|context| {
        context.order_frame_stack.push(OrderFrame::default());
    });
}

fn pop_order_frame(underflow_message: &str) {
    with_execution_context_mut(|context| {
        let popped = context.order_frame_stack.pop();
        debug_assert!(popped.is_some(), "{underflow_message}");
    });
}

fn next_order_counter(kind: OrderCounterKind, empty_message: &str) -> u64 {
    with_execution_context_mut(|context| next_order_counter_in(context, kind, empty_message))
}

fn next_order_counter_in(
    context: &mut ExecutionContext,
    kind: OrderCounterKind,
    empty_message: &str,
) -> u64 {
    debug_assert!(!context.order_frame_stack.is_empty(), "{empty_message}");
    let frame = context.order_frame_stack.last_mut().expect(empty_message);
    let counter = match kind {
        OrderCounterKind::Remember => {
            let counter = frame.remember;
            frame.remember = frame.remember.wrapping_add(1);
            counter
        }
        OrderCounterKind::Functor => {
            let counter = frame.functor;
            frame.functor = frame.functor.wrapping_add(1);
            counter
        }
        OrderCounterKind::Context => {
            let counter = frame.context;
            frame.context = frame.context.wrapping_add(1);
            counter
        }
        OrderCounterKind::ModifierAttachment => {
            let counter = frame.modifier_attachment;
            frame.modifier_attachment = frame
                .modifier_attachment
                .checked_add(1)
                .expect("modifier attachment counter exhausted");
            counter
        }
        OrderCounterKind::FrameReceiver => {
            let counter = frame.frame_receiver;
            frame.frame_receiver = frame.frame_receiver.wrapping_add(1);
            counter
        }
    };
    salt_order_counter(frame.occurrence_salt, counter)
}

fn salt_order_counter(salt: u64, counter: u64) -> u64 {
    if salt == 0 {
        counter
    } else {
        hash_components(&[&salt, &counter])
    }
}

fn next_child_instance_call_index(context: &mut ExecutionContext) -> u64 {
    let Some(frame) = context.order_frame_stack.last_mut() else {
        return 0;
    };
    let index = frame.instance;
    frame.instance = frame.instance.wrapping_add(1);
    salt_order_counter(frame.occurrence_salt, index)
}

pub(crate) fn compute_context_slot_key() -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();

    let call_counter = next_order_counter(
        OrderCounterKind::Context,
        "ORDER_FRAME_STACK is empty; provide_context must be called inside a component",
    );

    let slot_hash = hash_components(&[&group_path_hash, &call_counter]);
    (instance_logic_id, slot_hash)
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SlotKey {
    owned_owner: Option<u128>,
    instance_logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

impl Default for SlotKey {
    fn default() -> Self {
        Self {
            owned_owner: None,
            instance_logic_id: 0,
            slot_hash: 0,
            type_id: TypeId::of::<()>(),
        }
    }
}

new_key_type! {
    struct SlotHandle;
}

#[derive(Default)]
struct SlotEntry {
    key: SlotKey,
    generation: u64,
    value: Option<Arc<dyn Any + Send + Sync>>,
    last_alive_epoch: u64,
    retained: bool,
}

impl SlotEntry {
    fn reuse_functor<T: Send + Sync + 'static>(
        &mut self,
        slot: SlotHandle,
        epoch: u64,
        init: &mut Option<impl FnOnce() -> T>,
    ) -> (Arc<T>, u64) {
        debug_assert_eq!(self.key.type_id, TypeId::of::<T>());
        self.last_alive_epoch = epoch;
        let value = self.value.get_or_insert_with(|| {
            let value = Arc::new(init.take().expect("functor slot initialized once")());
            self.generation = self.generation.wrapping_add(1);
            value
        });
        (clone_slot_value(value, slot), self.generation)
    }
}

#[derive(Default)]
struct InstanceSlotCursor {
    previous_order: SmallVec<[SlotHandle; 1]>,
    current_order: SmallVec<[SlotHandle; 1]>,
    cursor: usize,
    epoch: u64,
}

impl InstanceSlotCursor {
    fn begin_epoch(&mut self, epoch: u64) {
        if self.epoch == epoch {
            return;
        }
        std::mem::swap(&mut self.previous_order, &mut self.current_order);
        if self.current_order.capacity() > self.previous_order.len().saturating_mul(2).max(4) {
            self.current_order = SmallVec::new();
        } else {
            self.current_order.clear();
        }
        self.cursor = 0;
        self.epoch = epoch;
    }

    fn fast_candidate(&self) -> Option<SlotHandle> {
        self.previous_order.get(self.cursor).copied()
    }

    fn record_fast_match(&mut self, slot: SlotHandle) {
        self.cursor = self.cursor.saturating_add(1);
        self.current_order.push(slot);
    }

    fn record_slow_match(&mut self, slot: SlotHandle) {
        if self.cursor < self.previous_order.len()
            && let Some(offset) = self.previous_order[self.cursor..]
                .iter()
                .position(|candidate| *candidate == slot)
        {
            self.cursor += offset + 1;
        }
        self.current_order.push(slot);
    }
}

#[derive(Default)]
struct SlotTable {
    owned_slots: HashMap<u128, HashSet<SlotHandle>>,
    entries: SlotMap<SlotHandle, SlotEntry>,
    slots_by_key: HashTable<SlotHandle>,
    slots_by_owner: HashMap<u64, InlineSet<SlotHandle>>,
    cursors_by_instance_logic_id: HashMap<u64, InstanceSlotCursor>,
    epoch: u64,
}
impl SlotTable {
    fn find_slot(&self, key: &SlotKey) -> Option<SlotHandle> {
        self.slots_by_key
            .find(FxBuildHasher.hash_one(key), |&slot| {
                self.entries[slot].key == *key
            })
            .copied()
    }

    fn index_slot(&mut self, slot: SlotHandle) {
        let key = &self.entries[slot].key;
        self.slots_by_key
            .insert_unique(FxBuildHasher.hash_one(key), slot, |&slot| {
                FxBuildHasher.hash_one(&self.entries[slot].key)
            });
        if let Some(owner) = key.owned_owner {
            self.owned_slots.entry(owner).or_default().insert(slot);
            return;
        }
        InlineSet::insert_at(self.slots_by_owner.entry(key.instance_logic_id), slot);
    }

    fn remove_slot(&mut self, slot: SlotHandle) -> Option<SlotEntry> {
        let entry = self.entries.remove(slot)?;
        if let Ok(indexed) = self
            .slots_by_key
            .find_entry(FxBuildHasher.hash_one(&entry.key), |&candidate| {
                candidate == slot
            })
        {
            indexed.remove();
        }
        if let Some(owner) = entry.key.owned_owner {
            if let Some(slots) = self.owned_slots.get_mut(&owner) {
                slots.remove(&slot);
                if slots.is_empty() {
                    self.owned_slots.remove(&owner);
                }
            }
            return Some(entry);
        }
        if let Entry::Occupied(mut slots) = self.slots_by_owner.entry(entry.key.instance_logic_id)
            && slots.get_mut().remove_and_is_empty(&slot)
        {
            slots.remove();
        }
        Some(entry)
    }

    fn slots_for_instances<'a>(
        &'a self,
        instance_logic_ids: &'a HashSet<u64>,
    ) -> impl Iterator<Item = SlotHandle> + 'a {
        instance_logic_ids
            .iter()
            .filter_map(|id| self.slots_by_owner.get(id))
            .flat_map(InlineSet::iter)
    }

    fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }

    fn reset(&mut self) -> Vec<SlotEntry> {
        let removed = self.entries.drain().map(|(_, entry)| entry).collect();
        self.slots_by_key.clear();
        self.slots_by_owner.clear();
        self.owned_slots.clear();
        self.cursors_by_instance_logic_id.clear();
        self.epoch = 0;
        removed
    }
    fn try_fast_slot_lookup(&mut self, key: &SlotKey) -> Option<(SlotHandle, &mut SlotEntry)> {
        let cursor = self
            .cursors_by_instance_logic_id
            .entry(key.instance_logic_id)
            .or_default();
        cursor.begin_epoch(self.epoch);
        let candidate = cursor.fast_candidate()?;

        let entry = self
            .entries
            .get_mut(candidate)
            .filter(|entry| entry.key == *key)?;

        cursor.record_fast_match(candidate);
        Some((candidate, entry))
    }

    fn record_slot_usage_slow(&mut self, instance_logic_id: u64, slot: SlotHandle) {
        let epoch = self.epoch;
        let cursor = self
            .cursors_by_instance_logic_id
            .entry(instance_logic_id)
            .or_default();
        cursor.begin_epoch(epoch);
        cursor.record_slow_match(slot);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PersistentFocusHandleKey {
    instance_key: u64,
    slot_hash: u64,
}

#[derive(Clone, Copy)]
struct PersistentFocusHandleEntry<T> {
    value: T,
    missing_frames: u8,
}

impl<T: Copy> PersistentFocusHandleEntry<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            missing_frames: 0,
        }
    }

    fn mark_live(&mut self) -> T {
        self.missing_frames = 0;
        self.value
    }

    fn retain_for_frame(&mut self) -> bool {
        if self.missing_frames == 0 {
            self.missing_frames = 1;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
struct PersistentFocusHandleStore {
    keys_by_owner: HashMap<u64, HashSet<PersistentFocusHandleKey>>,
    prune_candidates: HashSet<u64>,
    live_handles: HashSet<FocusHandleId>,
    targets: HashMap<PersistentFocusHandleKey, PersistentFocusHandleEntry<FocusNode>>,
    scopes: HashMap<PersistentFocusHandleKey, PersistentFocusHandleEntry<FocusScopeNode>>,
    groups: HashMap<PersistentFocusHandleKey, PersistentFocusHandleEntry<FocusGroupNode>>,
    requesters: HashMap<PersistentFocusHandleKey, PersistentFocusHandleEntry<FocusRequester>>,
}

#[derive(Default)]
pub(crate) struct RemovedPersistentFocusHandles {
    pub handle_ids: HashSet<FocusHandleId>,
    pub requester_ids: HashSet<FocusRequesterId>,
}

impl PersistentFocusHandleStore {
    fn prune_removed(
        &mut self,
        removed_owners: HashSet<u64>,
        is_live: impl Fn(u64) -> bool,
    ) -> RemovedPersistentFocusHandles {
        self.prune_candidates.extend(removed_owners);
        let mut removed = RemovedPersistentFocusHandles::default();
        for owner in std::mem::take(&mut self.prune_candidates) {
            let live = is_live(owner);
            let Some(keys) = self.keys_by_owner.get(&owner) else {
                continue;
            };
            let mut retained = false;
            for &key in keys {
                if let Some(value) = prune_focus_entry(&mut self.targets, key, live) {
                    removed.handle_ids.insert(value.handle_id());
                }
                if let Some(value) = prune_focus_entry(&mut self.scopes, key, live) {
                    removed.handle_ids.insert(value.handle_id());
                }
                if let Some(value) = prune_focus_entry(&mut self.groups, key, live) {
                    removed.handle_ids.insert(value.handle_id());
                }
                if let Some(value) = prune_focus_entry(&mut self.requesters, key, live) {
                    removed.requester_ids.insert(value.requester_id());
                }
                retained |= self.targets.contains_key(&key)
                    || self.scopes.contains_key(&key)
                    || self.groups.contains_key(&key)
                    || self.requesters.contains_key(&key);
            }
            if !live && retained {
                self.prune_candidates.insert(owner);
            } else if !retained {
                self.keys_by_owner.remove(&owner);
            }
        }
        for handle in &removed.handle_ids {
            self.live_handles.remove(handle);
        }
        removed
    }

    fn contains_handle(&self, handle_id: FocusHandleId) -> bool {
        self.live_handles.contains(&handle_id)
    }

    fn clear(&mut self) {
        self.keys_by_owner.clear();
        self.prune_candidates.clear();
        self.live_handles.clear();
        self.targets.clear();
        self.scopes.clear();
        self.groups.clear();
        self.requesters.clear();
    }
}

fn prune_focus_entry<T: Copy>(
    entries: &mut HashMap<PersistentFocusHandleKey, PersistentFocusHandleEntry<T>>,
    key: PersistentFocusHandleKey,
    live: bool,
) -> Option<T> {
    let entry = entries.get_mut(&key)?;
    if live {
        entry.mark_live();
        return None;
    }
    if entry.retain_for_frame() {
        return None;
    }
    entries.remove(&key).map(|entry| entry.value)
}

fn with_slot_table<R>(f: impl FnOnce(&SlotTable) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.slot_table.borrow()))
}

fn with_slot_table_mut<R>(f: impl FnOnce(&mut SlotTable) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.slot_table.borrow_mut()))
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctorHandle {
    session: u128,
    slot: SlotHandle,
    generation: u64,
}

impl FunctorHandle {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self {
            session: crate::session::current_session_identity().expect("active functor session"),
            slot,
            generation,
        }
    }
}

struct CallbackCell {
    current: RwLock<Arc<dyn Fn() + Send + Sync>>,
}

impl CallbackCell {
    fn new(current: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            current: RwLock::new(current),
        }
    }

    fn update(&self, next: Arc<dyn Fn() + Send + Sync>) {
        let previous = std::mem::replace(&mut *self.current.write(), next);
        drop(previous);
    }

    fn shared(&self) -> Arc<dyn Fn() + Send + Sync> {
        Arc::clone(&self.current.read())
    }
}

struct CallbackWithCell<T, R> {
    current: RwLock<Arc<dyn Fn(T) -> R + Send + Sync>>,
}

impl<T, R> CallbackWithCell<T, R> {
    fn new(current: Arc<dyn Fn(T) -> R + Send + Sync>) -> Self {
        Self {
            current: RwLock::new(current),
        }
    }

    fn update(&self, next: Arc<dyn Fn(T) -> R + Send + Sync>) {
        let previous = std::mem::replace(&mut *self.current.write(), next);
        drop(previous);
    }

    fn shared(&self) -> Arc<dyn Fn(T) -> R + Send + Sync> {
        Arc::clone(&self.current.read())
    }
}

struct RenderSlotCell {
    current: RwLock<(Arc<dyn Fn() + Send + Sync>, Option<TypeId>)>,
}

impl RenderSlotCell {
    fn new(current: Arc<dyn Fn() + Send + Sync>, static_type: Option<TypeId>) -> Self {
        Self {
            current: RwLock::new((current, static_type)),
        }
    }

    fn update(&self, next: Arc<dyn Fn() + Send + Sync>, static_type: Option<TypeId>) -> bool {
        let previous = {
            let mut current = self.current.write();
            // A new allocation alone must not invalidate unchanged capture-free content.
            if static_type.is_some() && current.1 == static_type {
                return false;
            }
            std::mem::replace(&mut *current, (next, static_type))
        };
        drop(previous);
        true
    }

    fn shared(&self) -> Arc<dyn Fn() + Send + Sync> {
        Arc::clone(&self.current.read().0)
    }
}

struct RenderSlotWithCell<T> {
    current: RwLock<(Arc<dyn Fn(T) + Send + Sync>, Option<TypeId>)>,
}

impl<T> RenderSlotWithCell<T> {
    fn new(current: Arc<dyn Fn(T) + Send + Sync>, static_type: Option<TypeId>) -> Self {
        Self {
            current: RwLock::new((current, static_type)),
        }
    }

    fn update(&self, next: Arc<dyn Fn(T) + Send + Sync>, static_type: Option<TypeId>) -> bool {
        let previous = {
            let mut current = self.current.write();
            // Closure type still distinguishes different capture-free bodies at one slot.
            if static_type.is_some() && current.1 == static_type {
                return false;
            }
            std::mem::replace(&mut *current, (next, static_type))
        };
        drop(previous);
        true
    }

    fn shared(&self) -> Arc<dyn Fn(T) + Send + Sync> {
        Arc::clone(&self.current.read().0)
    }
}

#[derive(Default)]
struct LayoutDirtyTracker {
    layout_inputs_by_node: HashMap<u64, LayoutInputSnapshot>,
    pending_measure_self_dirty_nodes: HashSet<u64>,
    ready_measure_self_dirty_nodes: HashSet<u64>,
    pending_placement_self_dirty_nodes: HashSet<u64>,
    ready_placement_self_dirty_nodes: HashSet<u64>,
    previous_children_by_node: HashMap<u64, Box<[u64]>>,
    structural_changes: StructureReconcileResult,
    pending_parent_data_dirty_nodes: HashSet<u64>,
    pending_transform_dirty_nodes: HashSet<u64>,
    pending_paint_dirty_nodes: HashSet<u64>,
    pending_pointer_dirty_nodes: HashSet<u64>,
    state_reads: ReadDependencies<StateReadDependencyKey, PhaseReader>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum ReadPhase {
    Pointer,
    Measure,
    Place,
    Transform,
    Paint,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct PhaseReader {
    instance_key: u64,
    phase: ReadPhase,
}

struct LayoutInputSnapshot {
    policy: LayoutModifierHandle,
    modifier: Modifier,
}

#[derive(Default)]
pub(crate) struct LayoutDirtyNodes {
    pub measure_self_nodes: HashSet<u64>,
    pub placement_self_nodes: HashSet<u64>,
    pub parent_data_nodes: HashSet<u64>,
    pub transform_nodes: HashSet<u64>,
    pub paint_nodes: HashSet<u64>,
}

#[derive(Default)]
pub(crate) struct StructureReconcileResult {
    pub changed_nodes: HashSet<u64>,
    pub removed_nodes: HashSet<u64>,
}

/// Persisted replay snapshot for one component instance.
#[derive(Clone)]
pub(crate) struct ReplayNodeSnapshot {
    pub instance_logic_id: u64,
    pub group_path: GroupPath,
    pub instance_key_override: Option<u64>,
    pub context: ContextMap,
    pub replay: Arc<dyn ErasedComponent>,
}

#[derive(Default)]
struct ReplaySnapshots {
    entries: Vec<(u64, ReplayNodeSnapshot)>,
    indices: HashTable<usize>,
}

impl ReplaySnapshots {
    fn get(&self, key: &u64) -> Option<&ReplayNodeSnapshot> {
        let index = *self.indices.find(FxBuildHasher.hash_one(key), |&index| {
            self.entries[index].0 == *key
        })?;
        Some(&self.entries[index].1)
    }

    fn insert(&mut self, key: u64, snapshot: ReplayNodeSnapshot) -> Option<ReplayNodeSnapshot> {
        let entry = self.indices.entry(
            FxBuildHasher.hash_one(key),
            |&index| self.entries[index].0 == key,
            |&index| FxBuildHasher.hash_one(self.entries[index].0),
        );
        match entry {
            TableEntry::Occupied(entry) => Some(std::mem::replace(
                &mut self.entries[*entry.get()].1,
                snapshot,
            )),
            TableEntry::Vacant(entry) => {
                let index = self.entries.len();
                self.entries.push((key, snapshot));
                entry.insert(index);
                None
            }
        }
    }

    fn remove(&mut self, key: &u64) -> Option<ReplayNodeSnapshot> {
        let entry = self
            .indices
            .find_entry(FxBuildHasher.hash_one(key), |&index| {
                self.entries[index].0 == *key
            })
            .ok()?;
        let (index, _) = entry.remove();
        let (_, snapshot) = self.entries.swap_remove(index);
        if let Some((moved_key, _)) = self.entries.get(index) {
            let previous_index = self.entries.len();
            let mut moved = self
                .indices
                .find_entry(FxBuildHasher.hash_one(moved_key), |&index| {
                    index == previous_index
                })
                .unwrap_or_else(|_| unreachable!("moved replay snapshot must be indexed"));
            *moved.get_mut() = index;
        }
        Some(snapshot)
    }

    fn drain(&mut self) -> impl Iterator<Item = (u64, ReplayNodeSnapshot)> + '_ {
        self.indices.clear();
        self.entries.drain(..)
    }
}

#[derive(Default)]
struct ComponentReplayTracker {
    previous_nodes: ReplaySnapshots,
    current_nodes: ReplaySnapshots,
}

fn with_component_replay_tracker<R>(f: impl FnOnce(&ComponentReplayTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.component_replay_tracker.borrow()))
}

fn with_component_replay_tracker_mut<R>(f: impl FnOnce(&mut ComponentReplayTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.component_replay_tracker.borrow_mut()))
}

pub(crate) fn begin_frame_component_replay_tracking() {
    let retired = with_component_replay_tracker_mut(|tracker| {
        (!tracker.current_nodes.entries.is_empty())
            .then(|| std::mem::take(&mut tracker.current_nodes))
    });
    drop(retired);
}

pub(crate) fn finalize_frame_component_replay_tracking() {
    let retired = with_component_replay_tracker_mut(|tracker| {
        std::mem::replace(
            &mut tracker.previous_nodes,
            std::mem::take(&mut tracker.current_nodes),
        )
    });
    drop(retired);
}

pub(crate) fn finalize_frame_component_replay_tracking_partial() {
    let retired = with_component_replay_tracker_mut(|tracker| {
        tracker
            .current_nodes
            .drain()
            .filter_map(|(key, snapshot)| tracker.previous_nodes.insert(key, snapshot))
            .collect::<Vec<_>>()
    });
    drop(retired);
}

pub(crate) fn reset_component_replay_tracking() {
    let removed = with_component_replay_tracker_mut(std::mem::take);
    drop(removed);
}

pub(crate) fn previous_component_replay_nodes(
    keys: &[u64],
) -> Result<Vec<ReplayNodeSnapshot>, u64> {
    with_component_replay_tracker(|tracker| {
        keys.iter()
            .map(|key| tracker.previous_nodes.get(key).cloned().ok_or(*key))
            .collect()
    })
}

pub(crate) fn previous_component_context(key: u64) -> Option<ContextMap> {
    with_component_replay_tracker(|tracker| {
        tracker
            .previous_nodes
            .get(&key)
            .map(|node| node.context.clone())
    })
}

pub(crate) fn remove_previous_component_replay_nodes(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let retired = with_component_replay_tracker_mut(|tracker| {
        let mut retired = Vec::new();
        for key in instance_keys {
            retired.extend(tracker.previous_nodes.remove(key));
            retired.extend(tracker.current_nodes.remove(key));
        }
        retired
    });
    drop(retired);
}

#[derive(Default)]
struct BuildInvalidationTracker {
    dirty_instance_keys: HashSet<u64>,
    cross_instance_dirty_instance_keys: HashSet<u64>,
}

impl BuildInvalidationTracker {
    fn invalidate(&mut self, readers: impl IntoIterator<Item = u64>, writer: Option<u64>) -> bool {
        let mut wake = false;
        for reader in readers {
            wake |= self.dirty_instance_keys.insert(reader);
            if writer.is_some_and(|writer| writer != reader) {
                self.cross_instance_dirty_instance_keys.insert(reader);
                wake = true;
            }
        }
        wake
    }
}

#[derive(Default)]
pub(crate) struct BuildInvalidationSet {
    pub dirty_instance_keys: HashSet<u64>,
    pub cross_instance_dirty_instance_keys: HashSet<u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateReadDependencyKey {
    // Keep generation in the dependency key to avoid ABA when a slot is
    // recycled and later reused for another state value.
    slot: SlotHandle,
    generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FocusReadDependencyKey {
    kind: FocusReadDependencyKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FocusReadDependencyKind {
    Handle(FocusHandleId),
    Requester(FocusRequesterId),
}

type StateReadDependencyTracker = ReadDependencies<StateReadDependencyKey, u64>;

type FocusReadDependencyTracker = ReadDependencies<FocusReadDependencyKey, u64>;

type RenderSlotReadDependencyTracker = ReadDependencies<FunctorHandle, u64>;

type RedrawWaker = Arc<dyn Fn() + Send + Sync + 'static>;

fn with_build_invalidation_tracker<R>(f: impl FnOnce(&BuildInvalidationTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.build_invalidation_tracker.borrow()))
}

fn with_build_invalidation_tracker_mut<R>(f: impl FnOnce(&mut BuildInvalidationTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.build_invalidation_tracker.borrow_mut()))
}

fn with_state_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut StateReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&mut globals.state_read_dependency_tracker.borrow_mut()))
}

fn with_focus_read_dependency_tracker<R>(f: impl FnOnce(&FocusReadDependencyTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.focus_read_dependency_tracker.borrow()))
}

fn with_focus_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut FocusReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&mut globals.focus_read_dependency_tracker.borrow_mut()))
}

fn with_render_slot_read_dependency_tracker<R>(
    f: impl FnOnce(&RenderSlotReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&globals.render_slot_read_dependency_tracker.borrow()))
}

fn with_render_slot_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut RenderSlotReadDependencyTracker) -> R,
) -> R {
    with_runtime_globals(|globals| f(&mut globals.render_slot_read_dependency_tracker.borrow_mut()))
}

fn with_redraw_waker<R>(f: impl FnOnce(&Option<RedrawWaker>) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.redraw_waker.borrow()))
}

fn with_redraw_waker_mut<R>(f: impl FnOnce(&mut Option<RedrawWaker>) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.redraw_waker.borrow_mut()))
}

pub(crate) fn schedule_runtime_redraw() {
    let callback = with_redraw_waker(Clone::clone);
    if let Some(callback) = callback {
        callback();
    }
}

pub(crate) fn install_redraw_waker(callback: RedrawWaker) {
    crate::dispatcher::set_waker(Some(callback.clone()));
    let previous = with_redraw_waker_mut(|waker| waker.replace(callback));
    drop(previous);
}

pub(crate) fn clear_redraw_waker() {
    crate::dispatcher::set_waker(None);
    let removed = with_redraw_waker_mut(Option::take);
    drop(removed);
}

pub(crate) fn current_replay_boundary_instance_key_from_scope() -> Option<u64> {
    with_execution_context(|context| context.current_component_instance_stack.last().copied())
}

fn with_persistent_focus_handle_store<R>(f: impl FnOnce(&PersistentFocusHandleStore) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.persistent_focus_handle_store.borrow()))
}

fn with_persistent_focus_handle_store_mut<R>(
    f: impl FnOnce(&mut PersistentFocusHandleStore) -> R,
) -> R {
    with_runtime_globals(|globals| f(&mut globals.persistent_focus_handle_store.borrow_mut()))
}

fn current_persistent_focus_handle_key<K: Hash>(slot_key: K) -> PersistentFocusHandleKey {
    let Some(instance_key) = current_replay_boundary_instance_key_from_scope() else {
        panic!("persistent focus handles must be requested during a component build");
    };
    let slot_hash = hash_components(&[&slot_key]);
    PersistentFocusHandleKey {
        instance_key,
        slot_hash,
    }
}

pub(crate) fn persistent_focus_target_for_current_instance<K: Hash>(slot_key: K) -> FocusNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_persistent_focus_handle_store_mut(|store| {
        store
            .keys_by_owner
            .entry(key.instance_key)
            .or_default()
            .insert(key);
        match store.targets.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusNode::new();
                store.live_handles.insert(value.handle_id());
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    })
}

pub(crate) fn persistent_focus_scope_for_current_instance<K: Hash>(slot_key: K) -> FocusScopeNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_persistent_focus_handle_store_mut(|store| {
        store
            .keys_by_owner
            .entry(key.instance_key)
            .or_default()
            .insert(key);
        match store.scopes.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusScopeNode::new();
                store.live_handles.insert(value.handle_id());
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    })
}

pub(crate) fn persistent_focus_group_for_current_instance<K: Hash>(slot_key: K) -> FocusGroupNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_persistent_focus_handle_store_mut(|store| {
        store
            .keys_by_owner
            .entry(key.instance_key)
            .or_default()
            .insert(key);
        match store.groups.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusGroupNode::new();
                store.live_handles.insert(value.handle_id());
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    })
}

pub(crate) fn has_persistent_focus_handle(handle_id: FocusHandleId) -> bool {
    with_persistent_focus_handle_store(|store| store.contains_handle(handle_id))
}

pub(crate) fn has_pending_persistent_focus_prune() -> bool {
    with_persistent_focus_handle_store(|store| !store.prune_candidates.is_empty())
}

pub(crate) fn prune_persistent_focus_handles() {
    let removed_owners =
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.take_removed_focus_owners());
    let removed = with_persistent_focus_handle_store_mut(|store| {
        store.prune_removed(removed_owners, |key| {
            TesseraRuntime::with(|runtime| {
                runtime
                    .component_tree
                    .find_node_id_by_instance_key(key)
                    .is_some()
            })
        })
    });
    if !removed.handle_ids.is_empty() || !removed.requester_ids.is_empty() {
        TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .focus_owner_mut()
                .remove_handles(&removed.handle_ids, &removed.requester_ids);
        });
    }
}

pub(crate) fn clear_persistent_focus_handles() {
    with_persistent_focus_handle_store_mut(PersistentFocusHandleStore::clear);
}

#[cfg(test)]
fn take_next_node_instance_logic_id_override() -> Option<u64> {
    with_execution_context_mut(|context| context.next_node_instance_logic_id_override.take())
}

/// Runs `f` inside a replay scope restored from a previously recorded component
/// snapshot.
///
/// The replay scope restores:
/// - the control-flow group path active at the original call site
/// - the keyed-instance override active at the original call site
/// - a one-shot instance-logic-id override for the replayed component root
pub(crate) fn with_replay_scope<R>(
    instance_logic_id: u64,
    group_path: GroupPath,
    instance_key_override: Option<u64>,
    f: impl FnOnce() -> R,
) -> R {
    struct ReplayScopeGuard {
        group_path: GroupPath,
        key_override: Option<u64>,
        logic_id_override: Option<u64>,
    }

    impl Drop for ReplayScopeGuard {
        fn drop(&mut self) {
            with_execution_context_mut(|context| {
                context.group_path = self.group_path;
                context.instance_key_override = self.key_override;
                context.next_node_instance_logic_id_override = self.logic_id_override;
            });
        }
    }

    let _guard = with_execution_context_mut(|context| ReplayScopeGuard {
        group_path: std::mem::replace(&mut context.group_path, group_path),
        key_override: std::mem::replace(&mut context.instance_key_override, instance_key_override),
        logic_id_override: context
            .next_node_instance_logic_id_override
            .replace(instance_logic_id),
    });

    f()
}

pub(crate) fn with_build_dirty_instance_keys<R>(
    dirty_instance_keys: Arc<HashSet<u64>>,
    f: impl FnOnce() -> R,
) -> R {
    struct BuildDirtyScopeGuard {
        popped: bool,
    }

    impl Drop for BuildDirtyScopeGuard {
        fn drop(&mut self) {
            if self.popped {
                return;
            }
            with_execution_context_mut(|context| {
                let popped = context.build_dirty_instance_keys_stack.pop();
                debug_assert!(
                    popped.is_some(),
                    "BUILD_DIRTY_INSTANCE_KEYS_STACK underflow: attempted to pop from empty stack"
                );
            });
            self.popped = true;
        }
    }

    with_execution_context_mut(|context| {
        context
            .build_dirty_instance_keys_stack
            .push(dirty_instance_keys);
    });
    let _guard = BuildDirtyScopeGuard { popped: false };
    f()
}

pub(crate) fn is_instance_key_build_dirty(instance_key: u64) -> bool {
    with_execution_context(|context| {
        context
            .build_dirty_instance_keys_stack
            .last()
            .is_some_and(|dirty_instance_keys| dirty_instance_keys.contains(&instance_key))
    })
}

fn consume_pending_build_invalidation(instance_key: u64) -> bool {
    with_build_invalidation_tracker_mut(|tracker| {
        tracker
            .cross_instance_dirty_instance_keys
            .remove(&instance_key);
        tracker.dirty_instance_keys.remove(&instance_key)
    })
}

pub(crate) fn record_replay_boundary_invalidation_for_instance_key(instance_key: u64) {
    let inserted =
        with_build_invalidation_tracker_mut(|tracker| tracker.invalidate([instance_key], None));
    if inserted {
        schedule_runtime_redraw();
    }
}

/// Re-queues invalidations that could not be processed in the current frame.
pub(crate) fn defer_build_invalidations(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_build_invalidation_tracker_mut(|tracker| {
        tracker
            .dirty_instance_keys
            .extend(instance_keys.iter().copied());
    });
    schedule_runtime_redraw();
}

const POINTER_STATE_READER: crate::session::SessionLocal<std::cell::Cell<Option<u64>>> =
    crate::session::SessionLocal::new();

pub(crate) fn with_pointer_state_reader<R>(key: u64, run: impl FnOnce() -> R) -> R {
    struct ReaderGuard(Option<u64>);
    impl Drop for ReaderGuard {
        fn drop(&mut self) {
            POINTER_STATE_READER.with(|reader| reader.set(self.0));
        }
    }
    let _guard = ReaderGuard(POINTER_STATE_READER.with(|reader| reader.replace(Some(key))));
    run()
}

pub(crate) fn take_pointer_dirty_nodes() -> HashSet<u64> {
    with_layout_dirty_tracker_mut(|tracker| {
        std::mem::take(&mut tracker.pending_pointer_dirty_nodes)
    })
}

fn track_state_read_dependency(slot: SlotHandle, generation: u64) {
    let phase = match current_phase() {
        Some(RuntimePhase::Input) if POINTER_STATE_READER.with(|reader| reader.get().is_some()) => {
            Some(ReadPhase::Pointer)
        }
        Some(RuntimePhase::Measure) => Some(ReadPhase::Measure),
        Some(RuntimePhase::Place) if current_measured_node_instance_key().is_some() => {
            Some(ReadPhase::Place)
        }
        Some(RuntimePhase::Place) => Some(ReadPhase::Transform),
        Some(RuntimePhase::Paint) => Some(ReadPhase::Paint),
        _ => None,
    };
    if let Some(phase) = phase {
        track_layout_state_read_dependency(slot, generation, phase);
        return;
    }
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_replay_boundary_instance_key_from_scope() else {
        return;
    };

    let key = StateReadDependencyKey { slot, generation };
    with_state_read_dependency_tracker_mut(|tracker| tracker.record(key, reader_instance_key));
}

fn track_layout_state_read_dependency(slot: SlotHandle, generation: u64, phase: ReadPhase) {
    let Some(reader_instance_key) = POINTER_STATE_READER
        .with(|reader| reader.get())
        .or_else(current_measured_node_instance_key)
        .or_else(current_replay_boundary_instance_key_from_scope)
    else {
        return;
    };
    let reader_instance_key = PhaseReader {
        instance_key: reader_instance_key,
        phase,
    };
    let key = StateReadDependencyKey { slot, generation };
    with_layout_dirty_tracker_mut(|tracker| tracker.state_reads.record(key, reader_instance_key));
}

fn mark_layout_state_readers_dirty(key: StateReadDependencyKey) -> bool {
    with_layout_dirty_tracker_mut(|tracker| {
        let Some(readers) = tracker.state_reads.readers(&key) else {
            return false;
        };
        for reader in readers {
            let target = match reader.phase {
                ReadPhase::Measure => &mut tracker.pending_measure_self_dirty_nodes,
                ReadPhase::Place => &mut tracker.pending_placement_self_dirty_nodes,
                ReadPhase::Transform => &mut tracker.pending_transform_dirty_nodes,
                ReadPhase::Paint => &mut tracker.pending_paint_dirty_nodes,
                ReadPhase::Pointer => &mut tracker.pending_pointer_dirty_nodes,
            };
            target.insert(reader.instance_key);
        }
        true
    })
}

pub(crate) fn clear_phase_state_reads(instance_key: u64, phase: ReadPhase) {
    with_layout_dirty_tracker_mut(|tracker| {
        let reader = PhaseReader {
            instance_key,
            phase,
        };
        tracker.state_reads.remove_reader(&reader);
    });
}

pub(crate) fn remove_layout_state_read_dependencies(instance_keys: &HashSet<u64>) {
    for &instance_key in instance_keys {
        for phase in [
            ReadPhase::Pointer,
            ReadPhase::Measure,
            ReadPhase::Place,
            ReadPhase::Transform,
            ReadPhase::Paint,
        ] {
            clear_phase_state_reads(instance_key, phase);
        }
    }
}

fn track_focus_dependency(kind: FocusReadDependencyKind) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_replay_boundary_instance_key_from_scope() else {
        return;
    };

    let key = FocusReadDependencyKey { kind };
    with_focus_read_dependency_tracker_mut(|tracker| tracker.record(key, reader_instance_key));
}

fn focus_read_subscribers_by_kind(kind: FocusReadDependencyKind) -> Vec<u64> {
    let key = FocusReadDependencyKey { kind };
    with_focus_read_dependency_tracker(|tracker| {
        tracker
            .readers(&key)
            .map(Iterator::collect)
            .unwrap_or_default()
    })
}

pub(crate) fn track_focus_read_dependency(handle_id: FocusHandleId) {
    track_focus_dependency(FocusReadDependencyKind::Handle(handle_id));
}

pub(crate) fn track_focus_requester_read_dependency(requester_id: FocusRequesterId) {
    track_focus_dependency(FocusReadDependencyKind::Requester(requester_id));
}

pub(crate) fn focus_read_subscribers(handle_id: FocusHandleId) -> Vec<u64> {
    focus_read_subscribers_by_kind(FocusReadDependencyKind::Handle(handle_id))
}

pub(crate) fn focus_requester_read_subscribers(requester_id: FocusRequesterId) -> Vec<u64> {
    focus_read_subscribers_by_kind(FocusReadDependencyKind::Requester(requester_id))
}

pub(crate) fn track_render_slot_read_dependency(handle: FunctorHandle) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_replay_boundary_instance_key_from_scope() else {
        return;
    };

    with_render_slot_read_dependency_tracker_mut(|tracker| {
        tracker.record(handle, reader_instance_key)
    });
}

fn render_slot_read_subscribers(handle: FunctorHandle) -> Vec<u64> {
    with_render_slot_read_dependency_tracker(|tracker| {
        tracker
            .readers(&handle)
            .map(Iterator::collect)
            .unwrap_or_default()
    })
}

pub(crate) fn remove_state_read_dependencies(instance_keys: &HashSet<u64>) {
    crate::nested_scroll::remove_readers(instance_keys);
    if instance_keys.is_empty() {
        return;
    }
    with_state_read_dependency_tracker_mut(|tracker| tracker.remove_readers(instance_keys));
}

pub(crate) fn remove_focus_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_focus_read_dependency_tracker_mut(|tracker| tracker.remove_readers(instance_keys));
}

pub(crate) fn remove_render_slot_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_render_slot_read_dependency_tracker_mut(|tracker| tracker.remove_readers(instance_keys));
}

pub(crate) fn reset_state_read_dependencies() {
    with_state_read_dependency_tracker_mut(|tracker| {
        *tracker = StateReadDependencyTracker::default();
    });
}

pub(crate) fn reset_focus_read_dependencies() {
    with_focus_read_dependency_tracker_mut(|tracker| {
        *tracker = FocusReadDependencyTracker::default();
    });
}

pub(crate) fn reset_render_slot_read_dependencies() {
    with_render_slot_read_dependency_tracker_mut(|tracker| {
        *tracker = RenderSlotReadDependencyTracker::default();
    });
}

pub(crate) fn take_build_invalidations() -> BuildInvalidationSet {
    with_build_invalidation_tracker_mut(|tracker| BuildInvalidationSet {
        dirty_instance_keys: std::mem::take(&mut tracker.dirty_instance_keys),
        cross_instance_dirty_instance_keys: std::mem::take(
            &mut tracker.cross_instance_dirty_instance_keys,
        ),
    })
}

pub(crate) fn reset_build_invalidations() {
    with_build_invalidation_tracker_mut(|tracker| {
        *tracker = BuildInvalidationTracker::default();
    });
}

pub(crate) fn remove_build_invalidations(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_build_invalidation_tracker_mut(|tracker| {
        for key in instance_keys {
            tracker.dirty_instance_keys.remove(key);
            tracker.cross_instance_dirty_instance_keys.remove(key);
        }
    });
}

pub(crate) fn has_pending_build_invalidations() -> bool {
    with_build_invalidation_tracker(|tracker| !tracker.dirty_instance_keys.is_empty())
}

#[derive(Default)]
struct FrameClockTracker {
    frame_origin: Option<Instant>,
    current_frame_time: Option<Instant>,
    current_frame_nanos: u64,
    previous_frame_time: Option<Instant>,
    frame_delta: Duration,
    receivers: HashMap<FrameNanosReceiverKey, FrameNanosReceiver>,
    executed_receiver_owners: Vec<u64>,
    receivers_by_owner: HashMap<u64, HashSet<FrameNanosReceiverKey>>,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct FrameNanosReceiverKey {
    instance_logic_id: u64,
    receiver_hash: u64,
}

/// Control flow for [`receive_frame_nanos`] callbacks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameNanosControl {
    /// Keep this receiver registered and run it again on the next frame.
    Continue,
    /// Unregister this receiver after the current frame tick.
    Stop,
}

type FrameNanosReceiverCallback = Box<dyn FnMut(u64) -> FrameNanosControl + Send + 'static>;

struct FrameNanosReceiver {
    owner_instance_key: u64,
    callback: FrameNanosReceiverCallback,
}

fn with_frame_clock_tracker<R>(f: impl FnOnce(&FrameClockTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.frame_clock_tracker.borrow()))
}

fn with_frame_clock_tracker_mut<R>(f: impl FnOnce(&mut FrameClockTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.frame_clock_tracker.borrow_mut()))
}

pub(crate) fn begin_frame_clock(now: Instant) {
    with_frame_clock_tracker_mut(|tracker| {
        let frame_origin = *tracker.frame_origin.get_or_insert(now);
        tracker.previous_frame_time = tracker.current_frame_time;
        tracker.current_frame_time = Some(now);
        tracker.current_frame_nanos = now
            .saturating_duration_since(frame_origin)
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        tracker.frame_delta = tracker
            .previous_frame_time
            .map(|previous| now.saturating_duration_since(previous))
            .unwrap_or_default();
    });
}

pub(crate) fn reset_frame_clock() {
    let removed = with_frame_clock_tracker_mut(std::mem::take);
    drop(removed);
}

pub(crate) fn has_pending_frame_nanos_receivers() -> bool {
    with_frame_clock_tracker(|tracker| !tracker.receivers.is_empty())
}

/// Frame-clock work performed by a completed frame.
#[derive(Clone, Debug, Default)]
pub struct FrameClockDiagnostics {
    /// Runtime session that owns these subscriptions.
    pub session: Option<crate::RuntimeSessionId>,
    /// Number of actual frame callback invocations.
    pub callbacks_executed: usize,
    /// Number of subscriptions remaining after the frame.
    pub pending_subscriptions: usize,
    /// Replay boundary owners whose callbacks executed this frame.
    pub executed_owners: Vec<u64>,
}

#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
pub(crate) fn frame_clock_diagnostics() -> FrameClockDiagnostics {
    with_frame_clock_tracker(|tracker| FrameClockDiagnostics {
        session: crate::RuntimeSession::current().map(|session| session.identity()),
        callbacks_executed: tracker.executed_receiver_owners.len(),
        pending_subscriptions: tracker.receivers.len(),
        executed_owners: tracker.executed_receiver_owners.clone(),
    })
}

pub(crate) fn tick_frame_nanos_receivers() {
    crate::dispatcher::flush();
    let (frame_nanos, receivers) = with_frame_clock_tracker_mut(|tracker| {
        tracker.executed_receiver_owners.clear();
        (
            tracker.current_frame_nanos,
            std::mem::take(&mut tracker.receivers),
        )
    });
    for (key, mut receiver) in receivers {
        let registered = with_frame_clock_tracker(|tracker| {
            tracker
                .receivers_by_owner
                .get(&receiver.owner_instance_key)
                .is_some_and(|keys| keys.contains(&key))
        });
        if !registered {
            continue;
        }
        with_frame_clock_tracker_mut(|tracker| {
            tracker
                .executed_receiver_owners
                .push(receiver.owner_instance_key)
        });
        let active = matches!(
            (receiver.callback)(frame_nanos),
            FrameNanosControl::Continue
        );
        let discarded = with_frame_clock_tracker_mut(|tracker| {
            if active
                && tracker
                    .receivers_by_owner
                    .get(&receiver.owner_instance_key)
                    .is_some_and(|keys| keys.contains(&key))
            {
                match tracker.receivers.entry(key) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(receiver);
                        return None;
                    }
                    std::collections::hash_map::Entry::Occupied(_) => {}
                }
            } else if !tracker.receivers.contains_key(&key) {
                if let Some(keys) = tracker
                    .receivers_by_owner
                    .get_mut(&receiver.owner_instance_key)
                {
                    keys.remove(&key);
                    if keys.is_empty() {
                        tracker
                            .receivers_by_owner
                            .remove(&receiver.owner_instance_key);
                    }
                }
            }
            Some(receiver)
        });
        drop(discarded);
    }
}

pub(crate) fn remove_frame_nanos_receivers(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    let removed = with_frame_clock_tracker_mut(|tracker| {
        let mut removed = Vec::new();
        for owner in instance_keys {
            if let Some(keys) = tracker.receivers_by_owner.remove(owner) {
                for key in keys {
                    removed.extend(tracker.receivers.remove(&key));
                }
            }
        }
        removed
    });
    drop(removed);
}

pub(crate) fn clear_frame_nanos_receivers() {
    let removed = with_frame_clock_tracker_mut(|tracker| {
        tracker.receivers_by_owner.clear();
        std::mem::take(&mut tracker.receivers)
    });
    drop(removed);
}

/// Returns the timestamp of the current frame, if available.
///
/// The value is set by the renderer at frame begin.
pub fn current_frame_time() -> Option<Instant> {
    with_frame_clock_tracker(|tracker| tracker.current_frame_time)
}

/// Returns the current frame timestamp in nanoseconds from runtime origin.
pub fn current_frame_nanos() -> u64 {
    with_frame_clock_tracker(|tracker| tracker.current_frame_nanos)
}

/// Returns the elapsed time since the previous frame.
pub fn frame_delta() -> Duration {
    with_frame_clock_tracker(|tracker| tracker.frame_delta)
}

fn ensure_frame_receive_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("receive_frame_nanos must not be called inside measure")
        }
        Some(RuntimePhase::Place | RuntimePhase::Paint | RuntimePhase::Input) => {
            panic!("receive_frame_nanos must be called inside a tessera component build")
        }
        None => panic!("receive_frame_nanos must be called inside a tessera component build"),
    }
}

fn compute_frame_nanos_receiver_key() -> FrameNanosReceiverKey {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();

    let call_counter = next_order_counter(
        OrderCounterKind::FrameReceiver,
        "ORDER_FRAME_STACK is empty; receive_frame_nanos must be called inside a component",
    );

    let receiver_hash = hash_components(&[&group_path_hash, &call_counter]);
    FrameNanosReceiverKey {
        instance_logic_id,
        receiver_hash,
    }
}

/// Register a per-frame callback driven by the renderer's frame clock.
///
/// Registration is keyed by the current callsite identity. Repeated calls from
/// the same position keep the existing active callback until it returns
/// [`FrameNanosControl::Stop`].
pub fn receive_frame_nanos<F>(callback: F)
where
    F: FnMut(u64) -> FrameNanosControl + Send + 'static,
{
    ensure_frame_receive_phase();
    let frame_nanos_state = remember(current_frame_nanos);
    let _ = frame_nanos_state.get();

    let owner_instance_key = current_replay_boundary_instance_key_from_scope()
        .unwrap_or_else(|| panic!("receive_frame_nanos requires an active component node context"));
    let key = compute_frame_nanos_receiver_key();

    let mut unused = Some(callback);
    with_frame_clock_tracker_mut(|tracker| {
        tracker.receivers.entry(key).or_insert_with(|| {
            let mut callback = unused.take().expect("new component clock callback");
            FrameNanosReceiver {
                owner_instance_key,
                callback: Box::new(move |frame_nanos| {
                    if !frame_nanos_state.is_alive() {
                        return FrameNanosControl::Stop;
                    }
                    frame_nanos_state.set(frame_nanos);
                    callback(frame_nanos)
                }),
            }
        });
        let owner = tracker.receivers[&key].owner_instance_key;
        tracker
            .receivers_by_owner
            .entry(owner)
            .or_default()
            .insert(key);
    });
    drop(unused);
}

pub(crate) fn drop_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    crate::nested_scroll::remove_owners(instance_logic_ids);
    if instance_logic_ids.is_empty() {
        return;
    }

    let removed = with_slot_table_mut(|table| {
        let mut freed = Vec::new();
        for slot in table.slots_for_instances(instance_logic_ids) {
            let Some(entry) = table.entries.get(slot) else {
                continue;
            };
            if entry.retained {
                continue;
            }
            freed.push(slot);
        }
        let mut removed = Vec::with_capacity(freed.len());
        for slot in freed {
            removed.extend(table.remove_slot(slot));
        }
        for instance_logic_id in instance_logic_ids {
            table.cursors_by_instance_logic_id.remove(instance_logic_id);
        }
        removed
    });
    drop(removed);
}

fn with_layout_dirty_tracker_mut<R>(f: impl FnOnce(&mut LayoutDirtyTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.layout_dirty_tracker.borrow_mut()))
}

fn record_layout_policy_dirty(
    instance_key: u64,
    layout_policy: &LayoutModifierHandle,
    modifier: &Modifier,
) {
    if current_phase() != Some(RuntimePhase::Build) {
        return;
    }
    with_layout_dirty_tracker_mut(|tracker| {
        let snapshot = || LayoutInputSnapshot {
            policy: layout_policy.clone(),
            modifier: modifier.clone(),
        };
        let (measure_changed, placement_changed, transform_changed, parent_data_changed) =
            match tracker.layout_inputs_by_node.entry(instance_key) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let previous = entry.get();
                    let previous_policy = previous.policy.as_policy();
                    let policy = layout_policy.as_policy();
                    let measure_changed = !previous_policy.dyn_measure_eq(policy)
                        || !previous.modifier.layout_measure_eq(modifier);
                    let placement_changed = !previous_policy.dyn_placement_eq(policy)
                        || !previous.modifier.layout_placement_eq(modifier);
                    let transform_changed = !previous.modifier.position_eq(modifier);
                    let parent_data_changed = !previous.modifier.parent_data_eq(modifier);
                    if measure_changed
                        || placement_changed
                        || transform_changed
                        || parent_data_changed
                    {
                        entry.insert(snapshot());
                    }
                    (
                        measure_changed,
                        placement_changed,
                        transform_changed,
                        parent_data_changed,
                    )
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(snapshot());
                    (true, true, true, true)
                }
            };
        if measure_changed {
            tracker
                .pending_measure_self_dirty_nodes
                .insert(instance_key);
        } else if placement_changed {
            tracker
                .pending_placement_self_dirty_nodes
                .insert(instance_key);
        }
        if parent_data_changed {
            tracker.pending_parent_data_dirty_nodes.insert(instance_key);
        }
        if transform_changed {
            tracker.pending_transform_dirty_nodes.insert(instance_key);
        }
    });
}

pub(crate) fn begin_frame_layout_dirty_tracking() {
    with_layout_dirty_tracker_mut(|tracker| {
        // Marks queued outside the build (input handlers, frame callbacks,
        // external state writes) live in the pending sets until the next
        // layout pass. Carry them into the ready sets instead of dropping them.
        tracker
            .ready_measure_self_dirty_nodes
            .extend(std::mem::take(
                &mut tracker.pending_measure_self_dirty_nodes,
            ));
        tracker
            .ready_placement_self_dirty_nodes
            .extend(std::mem::take(
                &mut tracker.pending_placement_self_dirty_nodes,
            ));
    });
}

pub(crate) fn finalize_frame_layout_dirty_tracking() {
    with_layout_dirty_tracker_mut(|tracker| {
        tracker
            .ready_measure_self_dirty_nodes
            .extend(std::mem::take(
                &mut tracker.pending_measure_self_dirty_nodes,
            ));
        tracker
            .ready_placement_self_dirty_nodes
            .extend(std::mem::take(
                &mut tracker.pending_placement_self_dirty_nodes,
            ));
    });
}

pub(crate) fn take_layout_dirty_nodes() -> LayoutDirtyNodes {
    with_layout_dirty_tracker_mut(|tracker| {
        // Layout-only writes must reach layout even when no component build
        // runs.
        let mut measure_self_nodes = std::mem::take(&mut tracker.ready_measure_self_dirty_nodes);
        measure_self_nodes.extend(std::mem::take(
            &mut tracker.pending_measure_self_dirty_nodes,
        ));
        let mut placement_self_nodes =
            std::mem::take(&mut tracker.ready_placement_self_dirty_nodes);
        placement_self_nodes.extend(std::mem::take(
            &mut tracker.pending_placement_self_dirty_nodes,
        ));
        LayoutDirtyNodes {
            measure_self_nodes,
            placement_self_nodes,
            parent_data_nodes: std::mem::take(&mut tracker.pending_parent_data_dirty_nodes),
            transform_nodes: std::mem::take(&mut tracker.pending_transform_dirty_nodes),
            paint_nodes: std::mem::take(&mut tracker.pending_paint_dirty_nodes),
        }
    })
}

pub(crate) fn has_pending_layout_invalidations() -> bool {
    with_layout_dirty_tracker_mut(|tracker| {
        !tracker.pending_measure_self_dirty_nodes.is_empty()
            || !tracker.ready_measure_self_dirty_nodes.is_empty()
            || !tracker.pending_placement_self_dirty_nodes.is_empty()
            || !tracker.ready_placement_self_dirty_nodes.is_empty()
            || !tracker.pending_parent_data_dirty_nodes.is_empty()
            || !tracker.pending_transform_dirty_nodes.is_empty()
            || !tracker.pending_paint_dirty_nodes.is_empty()
            || !tracker.pending_pointer_dirty_nodes.is_empty()
    })
}

pub(crate) fn reset_layout_dirty_tracking() {
    with_layout_dirty_tracker_mut(|tracker| *tracker = LayoutDirtyTracker::default());
}

pub(crate) fn record_layout_structure(
    instance_key: u64,
    children: impl ExactSizeIterator<Item = u64> + Clone,
) {
    with_layout_dirty_tracker_mut(|tracker| {
        let previous = tracker
            .previous_children_by_node
            .get(&instance_key)
            .map_or(&[][..], Box::as_ref);
        if !previous.iter().copied().eq(children.clone()) {
            tracker
                .structural_changes
                .changed_nodes
                .insert(instance_key);
            if children.len() == 0 {
                tracker.previous_children_by_node.remove(&instance_key);
            } else {
                tracker
                    .previous_children_by_node
                    .insert(instance_key, children.collect());
            }
        }
        tracker
            .structural_changes
            .removed_nodes
            .remove(&instance_key);
    });
}

pub(crate) fn remove_layout_structure(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_layout_dirty_tracker_mut(|tracker| {
        for key in instance_keys {
            tracker.previous_children_by_node.remove(key);
            tracker.layout_inputs_by_node.remove(key);
            tracker.structural_changes.removed_nodes.insert(*key);
            tracker.structural_changes.changed_nodes.remove(key);
        }
    });
}

pub(crate) fn take_layout_structure_changes() -> StructureReconcileResult {
    with_layout_dirty_tracker_mut(|tracker| std::mem::take(&mut tracker.structural_changes))
}

/// Handle to memoized state created by [`remember`] and [`remember_with_key`].
///
/// `State<T>` is `Copy + Send + Sync` and provides `with`, `with_mut`, `get`,
/// `set`, and `cloned` to read or update the stored value.
///
/// Component factories such as [`remember`] run only during a Tessera build.
/// Remembered state survives recomposition while its slot is used; [`retain`]
/// also keeps unused slots until the component unmounts. [`StateOwner`] provides
/// a controller or route lifetime independent of component mounting.
///
/// Access rejects stale handles and handles owned by another UI session.
///
/// # Examples
///
/// ```
/// use tessera_ui::{remember, tessera};
///
/// #[tessera]
/// fn counter() {
///     let count = remember(|| 0usize);
///     count.with_mut(|c| *c += 1);
///     let current = count.get();
///     assert!(current >= 1);
/// }
/// ```
pub struct State<T> {
    session: u128,
    slot: SlotHandle,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Copy for State<T> {}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for State<T> {
    fn eq(&self, other: &Self) -> bool {
        self.session == other.session
            && self.slot == other.slot
            && self.generation == other.generation
    }
}

impl<T> Eq for State<T> {}

impl<T> Hash for State<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.session.hash(state);
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl<T> State<T> {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self {
            session: crate::session::current_session_identity().expect("active state session"),
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> State<T>
where
    T: Send + Sync + 'static,
{
    /// Returns whether this handle still refers to a live state slot.
    ///
    /// Remembered state can expire when its component leaves the tree;
    /// retained state remains alive until the runtime is reset.
    pub fn is_alive(&self) -> bool {
        if crate::session::current_session_identity() != Some(self.session) {
            return false;
        }
        try_with_runtime_globals(|globals| {
            let table = globals.slot_table.borrow();
            let Some(entry) = table.entries.get(self.slot) else {
                return false;
            };

            entry.generation == self.generation && entry.key.type_id == TypeId::of::<T>()
        })
        .unwrap_or(false)
    }

    /// Execute a closure with a shared reference to the stored value.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "state belongs to a different runtime session"
        );
        track_state_read_dependency(self.slot, self.generation);
        let value = with_slot_table(|table| state_value::<T>(table, self.slot, self.generation));
        f(&value)
    }

    /// Execute a closure with a mutable reference to the stored value.
    #[track_caller]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "state belongs to a different runtime session"
        );
        // Move the value out for the duration of the closure so user code can
        // re-enter runtime APIs (`remember`, callback handles) without hitting
        // the active slot-table borrow. The guard restores it on unwind too.
        let taken =
            with_slot_table_mut(|table| take_state_value::<T>(table, self.slot, self.generation));
        let mut guard = StateValueRestore::<T> {
            slot: self.slot,
            generation: self.generation,
            value: Some(taken),
        };
        let result = f(guard.value.as_mut().expect("state value is present"));
        let value = guard.value.take().expect("state value is present");
        let slot = self.slot;
        let generation = self.generation;
        let removed =
            with_slot_table_mut(|table| restore_state_value::<T>(table, slot, generation, value));
        if removed.is_none() {
            publish_state_write(self.slot, self.generation);
        }
        drop(removed);
        result
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replaces the value and invalidates readers only when it differs.
    ///
    /// The comparison does not subscribe the caller to the state. This permits
    /// publishing derived geometry without making its publisher a reader.
    #[track_caller]
    pub fn set_if_changed(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        assert_eq!(
            crate::session::current_session_identity(),
            Some(self.session),
            "state belongs to a different runtime session"
        );
        let taken =
            with_slot_table_mut(|table| take_state_value::<T>(table, self.slot, self.generation));
        let mut guard = StateValueRestore {
            slot: self.slot,
            generation: self.generation,
            value: Some(taken),
        };
        let changed = guard.value.as_ref().expect("state value is present") != &value;
        let removed = if changed {
            guard.value.replace(value)
        } else {
            Some(value)
        };
        let current = guard.value.take().expect("state value is present");
        let discarded = with_slot_table_mut(|table| {
            restore_state_value(table, self.slot, self.generation, current)
        });
        if changed && discarded.is_none() {
            publish_state_write(self.slot, self.generation);
        }
        drop(discarded);
        drop(removed);
        changed
    }

    /// Replace the stored value.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}

fn publish_state_write(slot: SlotHandle, generation: u64) {
    // A write performed while another component is building is a backwards
    // write for every other reader: they may already have been rebuilt in
    // this pass with the old value, so their invalidation must not be
    // treated as already covered by the current replay round.
    let writer_instance_key = with_execution_context(|context| {
        if context.phase_stack.last() == Some(&RuntimePhase::Build) {
            context.current_component_instance_stack.last().copied()
        } else {
            None
        }
    });
    let key = StateReadDependencyKey { slot, generation };
    let build_dirty = with_runtime_globals(|globals| {
        let dependencies = globals.state_read_dependency_tracker.borrow();
        dependencies.readers(&key).is_some_and(|readers| {
            globals
                .build_invalidation_tracker
                .borrow_mut()
                .invalidate(readers, writer_instance_key)
        })
    });
    let layout_dirty = mark_layout_state_readers_dirty(key);
    if build_dirty || layout_dirty {
        schedule_runtime_redraw();
    }
}

pub(crate) struct RuntimeGlobals {
    pub(crate) component_calls: ComponentCalls,
    slot_table: RefCell<SlotTable>,
    component_replay_tracker: RefCell<ComponentReplayTracker>,
    build_invalidation_tracker: RefCell<BuildInvalidationTracker>,
    state_read_dependency_tracker: RefCell<StateReadDependencyTracker>,
    focus_read_dependency_tracker: RefCell<FocusReadDependencyTracker>,
    render_slot_read_dependency_tracker: RefCell<RenderSlotReadDependencyTracker>,
    redraw_waker: RefCell<Option<RedrawWaker>>,
    persistent_focus_handle_store: RefCell<PersistentFocusHandleStore>,
    frame_clock_tracker: RefCell<FrameClockTracker>,
    layout_dirty_tracker: RefCell<LayoutDirtyTracker>,
    runtime: RefCell<TesseraRuntime>,
}

impl RuntimeGlobals {
    fn new() -> Self {
        Self {
            component_calls: ComponentCalls::default(),
            slot_table: RefCell::new(SlotTable::default()),
            component_replay_tracker: RefCell::new(ComponentReplayTracker::default()),
            build_invalidation_tracker: RefCell::new(BuildInvalidationTracker::default()),
            state_read_dependency_tracker: RefCell::new(StateReadDependencyTracker::default()),
            focus_read_dependency_tracker: RefCell::new(FocusReadDependencyTracker::default()),
            render_slot_read_dependency_tracker: RefCell::new(
                RenderSlotReadDependencyTracker::default(),
            ),
            redraw_waker: RefCell::new(None),
            persistent_focus_handle_store: RefCell::new(PersistentFocusHandleStore::default()),
            frame_clock_tracker: RefCell::new(FrameClockTracker::default()),
            layout_dirty_tracker: RefCell::new(LayoutDirtyTracker::default()),
            runtime: RefCell::new(TesseraRuntime::default()),
        }
    }
}

impl Default for RuntimeGlobals {
    fn default() -> Self {
        Self::new()
    }
}
/// Runtime state container.
#[derive(Default)]
pub struct TesseraRuntime {
    /// Hierarchical structure of all UI components in the application.
    pub component_tree: ComponentTree,
    /// Current window dimensions in physical pixels.
    pub(crate) window_size: [u32; 2],
    /// Cursor icon change request from UI components.
    pub cursor_icon_request: Option<winit::window::CursorIcon>,
    /// Whether the window is currently minimized.
    pub(crate) window_minimized: bool,
}

impl TesseraRuntime {
    /// Executes a closure with a shared, read-only reference to the runtime.
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        with_runtime_globals(|globals| f(&globals.runtime.borrow()))
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        with_runtime_globals(|globals| f(&mut globals.runtime.borrow_mut()))
    }

    /// Get the current window size in physical pixels.
    pub fn window_size(&self) -> [u32; 2] {
        self.window_size
    }

    pub(crate) fn set_current_children_scale(&mut self, scale: f32) {
        self.component_tree
            .current_node_mut()
            .expect("Scale must be attached to a layout node")
            .children_scale = scale;
    }

    /// Appends a modifier chain to the current component node.
    pub(crate) fn append_current_modifier(&mut self, modifier: Modifier) {
        if let Some(node) = self.component_tree.current_node_mut() {
            node.modifier = node.modifier.clone().then(modifier);
        } else {
            debug_assert!(
                false,
                "append_current_modifier must be called inside a component build"
            );
        }
    }

    pub(crate) fn push_current_globally_positioned_handler(
        &mut self,
        handler: crate::component_tree::GloballyPositionedHandler,
    ) {
        if let Some(node) = self.component_tree.current_node_mut() {
            node.interaction_mut()
                .globally_positioned_handlers
                .push(handler);
        } else {
            debug_assert!(
                false,
                "push_current_globally_positioned_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_accessibility(
        &mut self,
        accessibility: Option<AccessibilityNode>,
        handler: Option<AccessibilityActionHandler>,
    ) {
        let Some(node_id) = current_node_id() else {
            debug_assert!(
                false,
                "accessibility must be attached inside a component build"
            );
            return;
        };
        let metadatas = self.component_tree.metadatas_mut();
        if accessibility.is_some() || handler.is_some() {
            let metadata = metadatas.entry_or_default(node_id);
            metadata.accessibility = accessibility.map(Box::new);
            metadata.accessibility_action_handler = handler;
        } else if let Some(metadata) = metadatas.get_mut(&node_id) {
            metadata.accessibility = None;
            metadata.accessibility_action_handler = None;
        }
    }

    pub(crate) fn bind_current_focus_requester(&mut self, requester: FocusRequester) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.interaction_mut().focus_requester_binding = Some(requester);
        } else {
            debug_assert!(
                false,
                "bind_current_focus_requester must be called inside a component build"
            );
        }
    }

    pub(crate) fn ensure_current_focus_target(&mut self, node: FocusNode) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current.interaction().focus_registration.is_none() {
                current.interaction_mut().focus_registration =
                    Some(FocusRegistration::target(node));
            }
        } else {
            debug_assert!(
                false,
                "ensure_current_focus_target must be called inside a component build"
            );
        }
    }

    pub(crate) fn ensure_current_focus_scope(&mut self, scope: FocusScopeNode) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current.interaction().focus_registration.is_none() {
                current.interaction_mut().focus_registration =
                    Some(FocusRegistration::scope(scope));
            }
        } else {
            debug_assert!(
                false,
                "ensure_current_focus_scope must be called inside a component build"
            );
        }
    }

    pub(crate) fn ensure_current_focus_group(&mut self, group: FocusGroupNode) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current.interaction().focus_registration.is_none() {
                current.interaction_mut().focus_registration =
                    Some(FocusRegistration::group(group));
            }
        } else {
            debug_assert!(
                false,
                "ensure_current_focus_group must be called inside a component build"
            );
        }
    }

    pub(crate) fn current_focus_target_handle(&self) -> Option<FocusNode> {
        let registration = self
            .component_tree
            .current_node()?
            .interaction()
            .focus_registration?;
        (registration.kind == FocusRegistrationKind::Target)
            .then(|| FocusNode::from_handle_id(registration.id))
    }

    pub(crate) fn current_focus_scope_handle(&self) -> Option<FocusScopeNode> {
        let registration = self
            .component_tree
            .current_node()?
            .interaction()
            .focus_registration?;
        (registration.kind == FocusRegistrationKind::Scope)
            .then(|| FocusScopeNode::from_handle_id(registration.id))
    }

    pub(crate) fn current_focus_group_handle(&self) -> Option<FocusGroupNode> {
        let registration = self
            .component_tree
            .current_node()?
            .interaction()
            .focus_registration?;
        (registration.kind == FocusRegistrationKind::Group)
            .then(|| FocusGroupNode::from_handle_id(registration.id))
    }

    pub(crate) fn set_current_focus_properties(&mut self, properties: FocusProperties) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if let Some(registration) = current.interaction_mut().focus_registration.as_mut() {
                registration.properties = properties;
            } else {
                debug_assert!(
                    false,
                    "set_current_focus_properties requires focus_target, focus_scope, or focus_group first"
                );
            }
        } else {
            debug_assert!(
                false,
                "set_current_focus_properties must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_preferred_tab_stop(&mut self, preferred: bool) {
        let registration = self
            .component_tree
            .current_node_mut()
            .and_then(|node| node.interaction_mut().focus_registration.as_mut());
        if let Some(registration) = registration {
            registration.properties.preferred_tab_stop = preferred;
        } else {
            debug_assert!(false, "preferred_tab_stop requires a focus registration");
        }
    }

    pub(crate) fn set_current_focus_traversal_policy(&mut self, policy: FocusTraversalPolicy) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current
                .interaction()
                .focus_registration
                .is_some_and(|registration| {
                    matches!(
                        registration.kind,
                        FocusRegistrationKind::Scope | FocusRegistrationKind::Group
                    )
                })
            {
                current.interaction_mut().focus_traversal_policy = Some(policy);
            } else {
                debug_assert!(
                    false,
                    "set_current_focus_traversal_policy requires focus_scope or focus_group first"
                );
            }
        } else {
            debug_assert!(
                false,
                "set_current_focus_traversal_policy must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_changed_handler(&mut self, handler: CallbackWith<FocusState>) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.interaction_mut().focus_changed_handler = Some(handler);
        } else {
            debug_assert!(
                false,
                "set_current_focus_changed_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_event_handler(&mut self, handler: CallbackWith<FocusState>) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.interaction_mut().focus_event_handler = Some(handler);
        } else {
            debug_assert!(
                false,
                "set_current_focus_event_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_beyond_bounds_handler(
        &mut self,
        handler: CallbackWith<FocusDirection, bool>,
    ) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.interaction_mut().focus_beyond_bounds_handler = Some(handler);
        } else {
            debug_assert!(
                false,
                "set_current_focus_beyond_bounds_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_reveal_handler(
        &mut self,
        handler: CallbackWith<FocusRevealRequest, bool>,
    ) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.interaction_mut().focus_reveal_handler = Some(handler);
        } else {
            debug_assert!(
                false,
                "set_current_focus_reveal_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_restorer_fallback(&mut self, fallback: FocusRequester) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current
                .interaction()
                .focus_registration
                .is_some_and(|registration| registration.kind == FocusRegistrationKind::Scope)
            {
                current.interaction_mut().focus_restorer_fallback = Some(fallback);
            } else {
                debug_assert!(
                    false,
                    "set_current_focus_restorer_fallback requires focus_scope or focus_restorer first"
                );
            }
        } else {
            debug_assert!(
                false,
                "set_current_focus_restorer_fallback must be called inside a component build"
            );
        }
    }

    fn finalize_current_layout_policy_dirty(&mut self, policy: &LayoutModifierHandle) {
        if let Some(node) = self.component_tree.current_node() {
            if node.role == crate::component_tree::NodeRole::Layout {
                record_layout_policy_dirty(node.instance_key, policy, &node.modifier);
            }
        } else {
            debug_assert!(
                false,
                "finalize_current_layout_policy_dirty must be called inside a component build"
            );
        }
    }
}

/// Guard that records the current component node id for the calling thread.
/// Nested components push their id and pop on drop, forming a stack.
pub struct NodeContextGuard {
    popped: bool,
    #[cfg(feature = "profiling")]
    profiling_guard: Option<crate::profiler::ScopeGuard>,
}

/// Guard that keeps the current component instance key on the execution stack.
pub struct CurrentComponentInstanceGuard {
    popped: bool,
}

/// Guard that keeps the instance key of the layout node currently being
/// measured on the execution stack.
pub(crate) struct MeasuredNodeInstanceGuard {
    popped: bool,
}

/// Execution phase for `remember` usage checks.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RuntimePhase {
    /// Component render/build phase (allowed for `remember`).
    Build,
    /// Measurement phase (disallowed for `remember`).
    Measure,
    /// Child placement and window-position resolution phase.
    Place,
    /// Drawing command recording phase.
    Paint,
    /// Input handling phase (disallowed for `remember`).
    Input,
}

/// Guard for execution phase stack.
pub struct PhaseGuard {
    popped: bool,
}

impl PhaseGuard {
    /// Pop the current phase immediately.
    pub fn pop(mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_phase();
            self.popped = true;
        }
    }
}

impl NodeContextGuard {
    /// Pop the current node id immediately. Usually you rely on `Drop` instead.
    pub fn pop(mut self) {
        self.leave();
    }

    fn leave(&mut self) {
        if !self.popped {
            self.popped = true;
            pop_current_node();
        }
    }
}

impl Drop for NodeContextGuard {
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        {
            let _ = self.profiling_guard.take();
        }
        self.leave();
    }
}

impl CurrentComponentInstanceGuard {
    /// Pop the current component instance key immediately. Usually you rely on
    /// `Drop` instead.
    pub fn pop(mut self) {
        if !self.popped {
            pop_current_component_instance_key();
            self.popped = true;
        }
    }
}

impl Drop for CurrentComponentInstanceGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_current_component_instance_key();
            self.popped = true;
        }
    }
}

/// Push the instance key of the layout node currently being measured.
pub(crate) fn push_measured_node_instance_key(instance_key: u64) -> MeasuredNodeInstanceGuard {
    with_execution_context_mut(|context| {
        context.measured_node_instance_key_stack.push(instance_key);
    });
    MeasuredNodeInstanceGuard { popped: false }
}

fn pop_measured_node_instance_key() {
    with_execution_context_mut(|context| {
        let popped = context.measured_node_instance_key_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop measured node instance key from an empty stack"
        );
    });
}

impl Drop for MeasuredNodeInstanceGuard {
    fn drop(&mut self) {
        if !self.popped {
            pop_measured_node_instance_key();
            self.popped = true;
        }
    }
}

/// Instance key of the layout node currently being measured, if any.
pub(crate) fn current_measured_node_instance_key() -> Option<u64> {
    with_execution_context(|context| context.measured_node_instance_key_stack.last().copied())
}

pub(crate) fn reserve_component_identity(component_type_id: u64) -> u64 {
    with_execution_context_mut(|context| reserve_component_identity_in(context, component_type_id))
}

fn reserve_component_identity_in(context: &mut ExecutionContext, component_type_id: u64) -> u64 {
    if let Some(identity) = context.next_node_instance_logic_id_override.take() {
        return identity;
    }
    let parent_call_index = next_child_instance_call_index(context);
    let parent_instance_logic_id = context.instance_logic_id_stack.last().copied().unwrap_or(0);
    let group_path_hash = context.group_path.digest();
    let has_group_path = !context.group_path.is_empty();
    let key_override = context.instance_key_override;

    // Combine component_type_id with parent_instance_logic_id, the current
    // control-flow group path, and the call index local to that group. This
    // ensures:
    // 1. foo(1) and foo(2) get different logic_ids (via parent_call_index)
    // 2. Components in different control-flow groups get different logic_ids
    //    even when each group starts its local call index from zero
    // 3. Components in different container instances get different logic_ids
    //    (via parent_instance_logic_id)
    let instance_salt = if let Some(key_hash) = key_override {
        hash_components(&[&key_hash, &group_path_hash, &parent_call_index])
    } else if has_group_path {
        hash_components(&[&group_path_hash, &parent_call_index])
    } else {
        parent_call_index
    };

    if parent_call_index == 0
        && parent_instance_logic_id == 0
        && key_override.is_none()
        && !has_group_path
    {
        component_type_id
    } else {
        hash_components(&[
            &component_type_id,
            &parent_instance_logic_id,
            &instance_salt,
        ])
    }
}

/// Push the given node id as the current executing component for this thread.
pub fn push_current_node(
    node_id: NodeId,
    component_type_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    enter_node_context(node_id, fn_name, |context| {
        reserve_component_identity_in(context, component_type_id)
    })
}

/// Push the given node id with an already resolved instance logic id.
///
/// This is used outside build (for example input/measure), where component
/// identity must be restored from the recorded tree node rather than derived
/// from call order.
pub fn push_current_node_with_instance_logic_id(
    node_id: NodeId,
    instance_logic_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    enter_node_context(node_id, fn_name, |context| {
        next_child_instance_call_index(context);
        instance_logic_id
    })
}

fn enter_node_context(
    node_id: NodeId,
    fn_name: &str,
    resolve_identity: impl FnOnce(&mut ExecutionContext) -> u64,
) -> NodeContextGuard {
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let (parent_node_id, phase) = with_execution_context_mut(|context| {
        let parent = context.node_context_stack.last().copied();
        let identity = resolve_identity(context);
        context.node_context_stack.push(node_id);
        context.instance_logic_id_stack.push(identity);
        context.order_frame_stack.push(OrderFrame::default());
        (parent, context.phase_stack.last().copied())
    });

    NodeContextGuard {
        popped: false,
        #[cfg(feature = "profiling")]
        profiling_guard: match phase {
            Some(RuntimePhase::Build) => {
                crate::profiler::make_build_scope_guard(node_id, parent_node_id, fn_name)
            }
            _ => None,
        },
    }
}

pub(crate) struct LayoutNodeScope<'a> {
    id: NodeId,
    previous: Option<PreviousLayoutNode>,
    policy: &'a LayoutModifierHandle,
    _context: NodeContextGuard,
}

impl Drop for LayoutNodeScope<'_> {
    fn drop(&mut self) {
        TesseraRuntime::with_mut(|runtime| {
            if let Some(previous) = &self.previous {
                runtime
                    .component_tree
                    .reconcile_layout_node(self.id, previous);
            }
            runtime.finalize_current_layout_policy_dirty(self.policy);
            runtime.component_tree.pop_node();
        });
    }
}

/// A component build scope that restores the previous node and build context on exit.
pub struct ComponentNodeScope {
    instance_logic_id: u64,
    retained: bool,
    _instance: CurrentComponentInstanceGuard,
    _context: NodeContextGuard,
    _phase: PhaseGuard,
}

impl ComponentNodeScope {
    /// Return the identity of the component in this scope.
    pub fn instance_logic_id(&self) -> u64 {
        self.instance_logic_id
    }

    /// Returns whether the previous subtree is still valid for this call.
    pub fn is_retained(&self) -> bool {
        self.retained
    }
}

impl Drop for ComponentNodeScope {
    fn drop(&mut self) {
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.pop_node());
    }
}

/// Enter the build scope for one registered component call.
pub fn begin_component_node<P: Prop>(
    name: &'static str,
    component_type_id: u64,
    props: &Arc<P>,
) -> ComponentNodeScope {
    begin_component_node_erased(name, component_type_id, props.as_ref(), &|| props.clone())
}

fn begin_component_node_erased(
    name: &'static str,
    component_type_id: u64,
    props: &dyn ErasedComponent,
    current_replay: &dyn Fn() -> Arc<dyn ErasedComponent>,
) -> ComponentNodeScope {
    let phase = push_phase(RuntimePhase::Build);
    let mut node = new_build_node(name, NodeRole::Composition, component_type_id);
    let instance_logic_id = node.instance_logic_id;
    let instance_key = node.instance_key;
    let instance = push_current_component_instance_key(instance_key);
    let previous_replay = with_component_replay_tracker(|tracker| {
        let previous = tracker.previous_nodes.get(&instance_key)?;
        (previous.instance_logic_id == instance_logic_id && previous.replay.equals(props))
            .then(|| previous.replay.clone())
    });
    let pending_dirty = consume_pending_build_invalidation(instance_key);
    let props_unchanged = previous_replay.is_some();
    let can_retain =
        props_unchanged && !pending_dirty && !is_instance_key_build_dirty(instance_key);
    let (id, retained) = TesseraRuntime::with_mut(|runtime| {
        let tree = &mut runtime.component_tree;
        if can_retain
            && let Some(id) = tree.attach_retained_subtree(instance_key, instance_logic_id)
        {
            let retained = tree.get_mut(id).expect("retained component node");
            retained.replay_boundary = true;
            retained.props_unchanged_from_previous = true;
            return (id, true);
        }
        node.replay_boundary = true;
        node.props_unchanged_from_previous = props_unchanged;
        (tree.add_node(node), false)
    });
    let context = enter_node_context(id, name, |_| instance_logic_id);
    let scope = ComponentNodeScope {
        instance_logic_id,
        retained,
        _instance: instance,
        _context: context,
        _phase: phase,
    };
    let replay = previous_replay.unwrap_or_else(current_replay);
    let snapshot = with_execution_context(|context| ReplayNodeSnapshot {
        instance_logic_id,
        group_path: context.group_path,
        instance_key_override: context.instance_key_override,
        context: context.context_map.clone(),
        replay,
    });
    let retired = with_component_replay_tracker_mut(|tracker| {
        tracker.current_nodes.insert(instance_key, snapshot)
    });
    drop(retired);
    scope
}

fn new_build_node(name: &'static str, role: NodeRole, component_type_id: u64) -> ComponentNode {
    let (instance_logic_id, instance_key) = with_execution_context_mut(|context| {
        let logic_id = reserve_component_identity_in(context, component_type_id);
        (logic_id, instance_key_in(context, logic_id))
    });
    let mut node = ComponentNode::new(name, role);
    node.instance_logic_id = instance_logic_id;
    node.instance_key = instance_key;
    node
}

pub(crate) fn begin_layout_node<'a>(
    name: &'static str,
    component_type_id: u64,
    policy: &'a LayoutModifierHandle,
    draw: Box<dyn DrawModifierNode>,
) -> LayoutNodeScope<'a> {
    let mut node = new_build_node(name, NodeRole::Layout, component_type_id);
    // The policy used for measurement may maintain a cache through its shared reference.
    node.layout_node = policy.as_policy().clone_box();
    node.draw_node = draw;
    let instance_logic_id = node.instance_logic_id;
    let (id, previous) =
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.add_layout_node(node));
    LayoutNodeScope {
        id,
        previous,
        policy,
        _context: enter_node_context(id, name, |_| instance_logic_id),
    }
}

/// Push the current component instance key for the active execution scope.
pub fn push_current_component_instance_key(instance_key: u64) -> CurrentComponentInstanceGuard {
    with_execution_context_mut(|context| {
        context.current_component_instance_stack.push(instance_key);
    });
    CurrentComponentInstanceGuard { popped: false }
}

fn pop_current_component_instance_key() {
    with_execution_context_mut(|context| {
        let popped = context.current_component_instance_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current component instance key from an empty stack"
        );
    });
}

fn pop_current_node() {
    with_execution_context_mut(|context| {
        let popped = context.node_context_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop current node from an empty stack"
        );
        let order_frame = context.order_frame_stack.pop();
        debug_assert!(order_frame.is_some(), "component order frame must exist");
        context.instance_logic_id_stack.pop();
    });
}

/// Get the node id at the top of the active session's component stack.
pub fn current_node_id() -> Option<NodeId> {
    with_execution_context(|context| context.node_context_stack.last().copied())
}

fn current_instance_logic_id_opt() -> Option<u64> {
    with_execution_context(|context| context.instance_logic_id_stack.last().copied())
}

/// Returns the current component instance logic id.
pub(crate) fn current_instance_logic_id() -> u64 {
    current_instance_logic_id_opt()
        .expect("current_instance_logic_id must be called inside a component")
}

/// Returns the instance key for the current component call site.
pub(crate) fn current_instance_key() -> u64 {
    with_execution_context(|context| {
        let instance_logic_id = context
            .instance_logic_id_stack
            .last()
            .expect("current_instance_key must be called inside a component");
        instance_key_in(context, *instance_logic_id)
    })
}

fn instance_key_in(context: &ExecutionContext, logic_id: u64) -> u64 {
    let group_path_hash = context.group_path.digest();
    hash_components(&[&logic_id, &group_path_hash])
}

/// Push an execution phase for the current thread.
pub fn push_phase(phase: RuntimePhase) -> PhaseGuard {
    with_execution_context_mut(|context| {
        context.phase_stack.push(phase);
    });
    PhaseGuard { popped: false }
}

fn pop_phase() {
    with_execution_context_mut(|context| {
        let popped = context.phase_stack.pop();
        debug_assert!(
            popped.is_some(),
            "Attempted to pop execution phase from an empty stack"
        );
    });
}

pub(crate) fn current_phase() -> Option<RuntimePhase> {
    with_execution_context(|context| context.phase_stack.last().copied())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GroupPath(Option<u64>);

impl GroupPath {
    pub(crate) fn extend(self, group_id: u64) -> Self {
        Self(Some(hash_components(&[&(self.0, group_id)])))
    }

    fn digest(self) -> u64 {
        self.0.unwrap_or_default()
    }

    fn is_empty(self) -> bool {
        self.0.is_none()
    }
}

#[cfg(test)]
fn current_group_path() -> GroupPath {
    with_execution_context(|context| context.group_path)
}

fn current_group_path_hash() -> u64 {
    with_execution_context(|context| context.group_path.digest())
}

fn state_value<T: Send + Sync + 'static>(
    table: &SlotTable,
    slot: SlotHandle,
    generation: u64,
) -> Arc<T> {
    let entry = table
        .entries
        .get(slot)
        .unwrap_or_else(|| panic!("State points to freed slot: {slot:?}"));
    if entry.generation != generation {
        panic!(
            "State is stale (slot {slot:?}, generation {generation}, current generation {})",
            entry.generation
        );
    }
    if entry.key.type_id != TypeId::of::<T>() {
        panic!(
            "State type mismatch for slot {slot:?}: expected {}, stored {:?}",
            std::any::type_name::<T>(),
            entry.key.type_id
        );
    }
    clone_slot_value(
        entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("State slot {slot:?} has been cleared")),
        slot,
    )
}

/// Restores a value moved out of the slot table by [`State::with_mut`].
struct StateValueRestore<T: Send + Sync + 'static> {
    slot: SlotHandle,
    generation: u64,
    value: Option<T>,
}

impl<T: Send + Sync + 'static> Drop for StateValueRestore<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            let slot = self.slot;
            let generation = self.generation;
            let removed = with_slot_table_mut(|table| {
                restore_state_value::<T>(table, slot, generation, value)
            });
            drop(removed);
        }
    }
}

fn take_state_value<T: Send + Sync + 'static>(
    table: &mut SlotTable,
    slot: SlotHandle,
    generation: u64,
) -> T {
    let entry = table
        .entries
        .get_mut(slot)
        .unwrap_or_else(|| panic!("State points to freed slot: {slot:?}"));
    if entry.generation != generation {
        panic!(
            "State is stale (slot {slot:?}, generation {generation}, current generation {})",
            entry.generation
        );
    }
    if entry.key.type_id != TypeId::of::<T>() {
        panic!(
            "State type mismatch for slot {slot:?}: expected {}, stored {:?}",
            std::any::type_name::<T>(),
            entry.key.type_id
        );
    }
    let value = entry
        .value
        .as_ref()
        .unwrap_or_else(|| panic!("State slot {slot:?} has been cleared"));
    assert_eq!(
        Arc::strong_count(value),
        1,
        "State slot {slot:?} is being read"
    );
    let value = entry.value.take().expect("state value is present");
    Arc::try_unwrap(
        value
            .downcast::<T>()
            .unwrap_or_else(|_| panic!("State slot {slot:?} downcast failed")),
    )
    .unwrap_or_else(|_| unreachable!("state value must be unique"))
}

fn restore_state_value<T: Send + Sync + 'static>(
    table: &mut SlotTable,
    slot: SlotHandle,
    generation: u64,
    value: T,
) -> Option<T> {
    let Some(entry) = table.entries.get_mut(slot) else {
        return Some(value);
    };
    if entry.generation != generation
        || entry.key.type_id != TypeId::of::<T>()
        || entry.value.is_some()
    {
        return Some(value);
    }
    entry.value = Some(Arc::new(value));
    None
}

fn clone_slot_value<T: Send + Sync + 'static>(
    value: &Arc<dyn Any + Send + Sync>,
    slot: SlotHandle,
) -> Arc<T> {
    value
        .clone()
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("Slot {slot:?} downcast failed"))
}

fn resolve_state_key<T: Send + Sync + 'static>(
    key: SlotKey,
    retained: bool,
    init: impl FnOnce() -> T,
) -> State<T> {
    if let Some(state) = with_slot_table_mut(|table| {
        let slot = table.find_slot(&key)?;
        let epoch = table.epoch;
        let entry = table.entries.get_mut(slot)?;
        entry.last_alive_epoch = epoch;
        entry.retained |= retained;
        if key.owned_owner.is_none() {
            entry.value.as_ref()?;
        }
        Some(State::new(slot, entry.generation))
    }) {
        return state;
    }
    let value = init();
    let (state, unused) = with_slot_table_mut(|table| {
        if let Some(slot) = table.find_slot(&key) {
            let epoch = table.epoch;
            let entry = &mut table.entries[slot];
            entry.last_alive_epoch = epoch;
            entry.retained |= retained;
            if entry.value.is_some() || key.owned_owner.is_some() {
                return (State::new(slot, entry.generation), Some(value));
            }
            entry.value = Some(Arc::new(value));
            entry.generation = entry
                .generation
                .checked_add(1)
                .expect("state generation exhausted");
            return (State::new(slot, entry.generation), None);
        }
        let slot = table.entries.insert(SlotEntry {
            key,
            generation: 1,
            value: Some(Arc::new(value)),
            last_alive_epoch: table.epoch,
            retained,
        });
        table.index_slot(slot);
        (State::new(slot, 1), None)
    });
    drop(unused);
    state
}

pub(crate) fn modifier_attachment_path() -> [u64; 2] {
    [
        current_group_path_hash(),
        next_order_counter(
            OrderCounterKind::ModifierAttachment,
            "modifier attachment requires a build scope",
        ),
    ]
}

pub(crate) fn modifier_state<T: Send + Sync + 'static>(
    instance: u64,
    path: &[u64],
    ordinal: usize,
    init: impl FnOnce() -> T,
) -> State<T> {
    let key = SlotKey {
        owned_owner: None,
        instance_logic_id: instance,
        slot_hash: hash_components(&[&("modifier-state", path, ordinal)]),
        type_id: TypeId::of::<T>(),
    };
    resolve_state_key(key, false, init)
}

struct ModifierClockLifetime {
    session: u128,
    key: FrameNanosReceiverKey,
    owner: u64,
}

impl Drop for ModifierClockLifetime {
    fn drop(&mut self) {
        if crate::session::current_session_identity() != Some(self.session) {
            return;
        }
        let removed = with_frame_clock_tracker_mut(|tracker| {
            if let Some(keys) = tracker.receivers_by_owner.get_mut(&self.owner) {
                keys.remove(&self.key);
                if keys.is_empty() {
                    tracker.receivers_by_owner.remove(&self.owner);
                }
            }
            tracker.receivers.remove(&self.key)
        });
        drop(removed);
    }
}

pub(crate) fn modifier_frame_receiver<F>(
    instance: u64,
    owner: u64,
    path: &[u64],
    ordinal: usize,
    callback: F,
) where
    F: FnMut(u64) -> FrameNanosControl + Send + 'static,
{
    let state = modifier_state(instance, path, ordinal, current_frame_nanos);
    let _ = state.get();
    let key = FrameNanosReceiverKey {
        instance_logic_id: instance,
        receiver_hash: hash_components(&[&("modifier-clock", path, ordinal)]),
    };
    let _lifetime = modifier_state(instance, path, ordinal, || ModifierClockLifetime {
        session: crate::session::current_session_identity().expect("modifier clock session"),
        key,
        owner,
    });
    let mut unused = Some(callback);
    with_frame_clock_tracker_mut(|tracker| {
        tracker.receivers.entry(key).or_insert_with(|| {
            let mut callback = unused.take().expect("new modifier clock callback");
            FrameNanosReceiver {
                owner_instance_key: owner,
                callback: Box::new(move |nanos| {
                    if !state.is_alive() {
                        return FrameNanosControl::Stop;
                    }
                    state.set(nanos);
                    callback(nanos)
                }),
            }
        });
        tracker
            .receivers_by_owner
            .entry(owner)
            .or_default()
            .insert(key);
    });
    drop(unused);
}

fn resolve_state_slot<K, F, T>(
    key: K,
    init: F,
    retained: bool,
    _type_name: &'static str,
) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    ensure_build_phase();
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();
    let key_hash = hash_components(&[&key]);
    let positional = key_hash == hash_components(&[&()]);
    // Positional state is addressed by the dedicated remember counter, not by
    // the number of preceding state slots. Conditional state (for example the
    // frame-clock slot inside `receive_frame_nanos`) must not shift later
    // `remember` calls onto another value.
    let call_counter = next_order_counter(
        OrderCounterKind::Remember,
        "ORDER_FRAME_STACK is empty; remember must be called inside a component",
    );
    let type_hash = hash_components(&[&TypeId::of::<T>()]);
    // The call counter distinguishes positional calls, while the value type
    // keeps calls of different types from sharing one slot key.
    let slot_hash = if positional {
        hash_components(&[&group_path_hash, &type_hash, &call_counter])
    } else {
        hash_components(&[&group_path_hash, &key_hash])
    };
    let slot_key = SlotKey {
        owned_owner: None,
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    resolve_state_key(slot_key, retained, init)
}

#[cfg(test)]
fn current_instance_key_override() -> Option<u64> {
    with_execution_context(|context| context.instance_key_override)
}

/// RAII guard that tracks control-flow grouping for the current component node.
///
/// A guard pushes the provided group id when constructed and pops it when
/// dropped, ensuring grouping stays balanced even with early returns or panics.
pub struct GroupGuard {
    _path: PathGroupGuard,
}

impl GroupGuard {
    /// Push a group id onto the current component's group stack.
    pub fn new(group_id: u64) -> Self {
        let occurrence_salt = with_execution_context_mut(|context| {
            let Some(frame) = context.order_frame_stack.last_mut() else {
                return 0;
            };
            let occurrence = if let Some((_, next)) = frame
                .group_occurrences
                .iter_mut()
                .find(|(id, _)| *id == group_id)
            {
                let occurrence = *next;
                *next = next.wrapping_add(1);
                occurrence
            } else {
                frame.group_occurrences.push((group_id, 1));
                0
            };
            if occurrence == 0 && frame.occurrence_salt == 0 {
                0
            } else {
                hash_components(&[&frame.occurrence_salt, &group_id, &occurrence])
            }
        });
        let guard = Self::keyed(group_id);
        with_execution_context_mut(|context| {
            context
                .order_frame_stack
                .last_mut()
                .expect("group order frame must exist")
                .occurrence_salt = occurrence_salt;
        });
        guard
    }

    fn keyed(group_id: u64) -> Self {
        let path = PathGroupGuard::new(group_id);
        push_order_frame();
        Self { _path: path }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        pop_order_frame("ORDER_FRAME_STACK underflow: attempted to pop GroupGuard frame");
    }
}

/// RAII guard for path-only control-flow groups, primarily used for loop
/// bodies.
///
/// Unlike [`GroupGuard`], this guard does not create a new local order frame.
/// Repeated iterations therefore continue consuming sibling call indices from
/// the surrounding component scope instead of restarting from zero each time.
pub struct PathGroupGuard {
    previous: GroupPath,
    #[cfg(debug_assertions)]
    entered: GroupPath,
}

impl PathGroupGuard {
    /// Push a group id onto the current component's group stack without
    /// resetting local call-order counters.
    pub fn new(group_id: u64) -> Self {
        with_execution_context_mut(|context| {
            let entered = context.group_path.extend(group_id);
            Self {
                previous: std::mem::replace(&mut context.group_path, entered),
                #[cfg(debug_assertions)]
                entered,
            }
        })
    }
}

impl Drop for PathGroupGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            #[cfg(debug_assertions)]
            debug_assert_eq!(context.group_path, self.entered, "Unbalanced group scope");
            context.group_path = self.previous;
        });
    }
}

/// RAII guard that sets a stable instance key for the duration of a block.
pub struct InstanceKeyGuard {
    key_hash: u64,
    previous: Option<u64>,
}

impl InstanceKeyGuard {
    /// Push a key hash for instance identity.
    pub fn new(key_hash: u64) -> Self {
        let previous =
            with_execution_context_mut(|context| context.instance_key_override.replace(key_hash));
        Self { key_hash, previous }
    }
}

impl Drop for InstanceKeyGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            let popped = std::mem::replace(&mut context.instance_key_override, self.previous);
            debug_assert_eq!(
                popped,
                Some(self.key_hash),
                "Unbalanced InstanceKeyGuard stack"
            );
        });
    }
}

fn hash_components<H: Hash + ?Sized>(parts: &[&H]) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    for part in parts {
        part.hash(&mut hasher);
    }
    hasher.finish()
}

fn compute_functor_slot_key() -> (u64, u64) {
    with_execution_context_mut(|context| {
        let instance_logic_id = *context
            .instance_logic_id_stack
            .last()
            .expect("current_instance_logic_id must be called inside a component");
        let group_path_hash = context.group_path.digest();
        let call_counter = next_order_counter_in(
            context,
            OrderCounterKind::Functor,
            "ORDER_FRAME_STACK is empty; callback constructors must be called inside a component",
        );
        let key_hash = hash_components(&[&()]);
        let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
        (instance_logic_id, slot_hash)
    })
}

pub(crate) fn ensure_build_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("remember must not be called inside measure; move state to component render")
        }
        Some(RuntimePhase::Place) => {
            panic!("remember must not be called during placement; move state to component render")
        }
        Some(RuntimePhase::Paint) => {
            panic!("remember must not be called during drawing; move state to component render")
        }
        Some(RuntimePhase::Input) => {
            panic!(
                "remember must not be called inside typed input handlers; move state to component render"
            )
        }
        None => panic!(
            "remember must be called inside a tessera component. Ensure you're calling this from within a function annotated with #[tessera]."
        ),
    }
}

fn remember_functor_cell<T, F>(init: F) -> (Arc<T>, FunctorHandle)
where
    T: Send + Sync + 'static,
    F: FnOnce() -> T,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_functor_slot_key();
    let slot_key = SlotKey {
        owned_owner: None,
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    let mut init = Some(init);
    let (value, slot, generation) = with_slot_table_mut(|table| {
        let epoch = table.epoch;
        if let Some((slot, entry)) = table.try_fast_slot_lookup(&slot_key) {
            let (value, generation) = entry.reuse_functor(slot, epoch, &mut init);
            return (value, slot, generation);
        }
        if let Some(slot) = table.find_slot(&slot_key) {
            table.record_slot_usage_slow(instance_logic_id, slot);
            let (value, generation) = table.entries[slot].reuse_functor(slot, epoch, &mut init);
            return (value, slot, generation);
        }

        let value = Arc::new(init.take().expect("functor slot initialized once")());
        let generation = 1;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(value.clone()),
            last_alive_epoch: epoch,
            retained: false,
        });
        table.index_slot(slot);
        table.record_slot_usage_slow(instance_logic_id, slot);
        (value, slot, generation)
    });
    (value, FunctorHandle::new(slot, generation))
}

fn load_functor_cell<T>(handle: FunctorHandle) -> Arc<T>
where
    T: Send + Sync + 'static,
{
    assert_eq!(
        crate::session::current_session_identity(),
        Some(handle.session),
        "functor belongs to a different runtime session"
    );
    with_slot_table(|table| {
        let entry = table
            .entries
            .get(handle.slot)
            .unwrap_or_else(|| panic!("Callback points to freed slot: {:?}", handle.slot));

        if entry.generation != handle.generation {
            panic!(
                "Callback is stale (slot {:?}, generation {}, current generation {})",
                handle.slot, handle.generation, entry.generation
            );
        }

        if entry.key.type_id != TypeId::of::<T>() {
            panic!(
                "Callback type mismatch for slot {:?}: expected {}, stored {:?}",
                handle.slot,
                std::any::type_name::<T>(),
                entry.key.type_id
            );
        }

        clone_slot_value::<T>(
            entry
                .value
                .as_ref()
                .unwrap_or_else(|| panic!("Callback slot {:?} has been cleared", handle.slot)),
            handle.slot,
        )
    })
}

pub(crate) fn remember_callback_handle<F>(handler: F) -> FunctorHandle
where
    F: Fn() + Send + Sync + 'static,
{
    let handler = Arc::new(handler) as Arc<dyn Fn() + Send + Sync>;
    let (cell, handle) = remember_functor_cell({
        let handler = Arc::clone(&handler);
        move || CallbackCell::new(handler)
    });
    cell.update(handler);
    handle
}

pub(crate) fn invoke_callback_handle(handle: FunctorHandle) {
    let callback = load_functor_cell::<CallbackCell>(handle).shared();
    callback();
}

pub(crate) fn remember_render_slot_handle<F>(render: F) -> FunctorHandle
where
    F: Fn() + Send + Sync + 'static,
{
    let static_type = (std::mem::size_of::<F>() == 0).then(TypeId::of::<F>);
    let render = Arc::new(render) as Arc<dyn Fn() + Send + Sync>;
    let creator_instance_key = current_replay_boundary_instance_key_from_scope()
        .unwrap_or_else(|| panic!("RenderSlot handles must be created during a component build"));
    let (cell, handle) = remember_functor_cell({
        let render = Arc::clone(&render);
        move || RenderSlotCell::new(render, static_type)
    });
    if !cell.update(render, static_type) {
        return handle;
    }
    for instance_key in render_slot_read_subscribers(handle) {
        if instance_key != creator_instance_key && !is_instance_key_build_dirty(instance_key) {
            record_replay_boundary_invalidation_for_instance_key(instance_key);
        }
    }
    handle
}

pub(crate) fn invoke_render_slot_handle(handle: FunctorHandle) {
    let render = load_functor_cell::<RenderSlotCell>(handle).shared();
    render();
}

pub(crate) fn remember_render_slot_with_handle<T, F>(render: F) -> FunctorHandle
where
    T: 'static,
    F: Fn(T) + Send + Sync + 'static,
{
    let static_type = (std::mem::size_of::<F>() == 0).then(TypeId::of::<F>);
    let render = Arc::new(render) as Arc<dyn Fn(T) + Send + Sync>;
    let creator_instance_key =
        current_replay_boundary_instance_key_from_scope().unwrap_or_else(|| {
            panic!("RenderSlotWith handles must be created during a component build")
        });
    let (cell, handle) = remember_functor_cell({
        let render = Arc::clone(&render);
        move || RenderSlotWithCell::new(render, static_type)
    });
    if !cell.update(render, static_type) {
        return handle;
    }
    for instance_key in render_slot_read_subscribers(handle) {
        if instance_key != creator_instance_key && !is_instance_key_build_dirty(instance_key) {
            record_replay_boundary_invalidation_for_instance_key(instance_key);
        }
    }
    handle
}

pub(crate) fn invoke_render_slot_with_handle<T>(handle: FunctorHandle, value: T)
where
    T: 'static,
{
    let render = load_functor_cell::<RenderSlotWithCell<T>>(handle).shared();
    render(value)
}

pub(crate) fn remember_callback_with_handle<T, R, F>(handler: F) -> FunctorHandle
where
    T: 'static,
    R: 'static,
    F: Fn(T) -> R + Send + Sync + 'static,
{
    let handler = Arc::new(handler) as Arc<dyn Fn(T) -> R + Send + Sync>;
    let (cell, handle) = remember_functor_cell({
        let handler = Arc::clone(&handler);
        move || CallbackWithCell::new(handler)
    });
    cell.update(handler);
    handle
}

pub(crate) fn invoke_callback_with_handle<T, R>(handle: FunctorHandle, value: T) -> R
where
    T: 'static,
    R: 'static,
{
    let callback = load_functor_cell::<CallbackWithCell<T, R>>(handle).shared();
    callback(value)
}

/// Start a new state-slot epoch for the current recomposition pass.
pub fn begin_recompose_slot_epoch() {
    with_slot_table_mut(SlotTable::begin_epoch);
}

pub(crate) fn shutdown_runtime() {
    clear_frame_nanos_receivers();
    reset_slots();
    reset_component_replay_tracking();
    clear_redraw_waker();
    let tree = TesseraRuntime::with_mut(|runtime| std::mem::take(&mut runtime.component_tree));
    drop(tree);
}

pub(crate) fn reset_component_slots() {
    crate::component_call::reset_pending_calls();
    crate::context::reset_context_slots();
    let removed = with_slot_table_mut(|table| {
        let slots = table
            .entries
            .iter()
            .filter_map(|(slot, entry)| entry.key.owned_owner.is_none().then_some(slot))
            .collect::<Vec<_>>();
        slots
            .into_iter()
            .filter_map(|slot| table.remove_slot(slot))
            .collect::<Vec<_>>()
    });
    drop(removed);
}

/// Releases all slots when the UI session shuts down.
pub fn reset_slots() {
    crate::component_call::reset_pending_calls();
    crate::context::reset_context_slots();
    let removed = with_slot_table_mut(SlotTable::reset);
    drop(removed);
}

pub(crate) fn recycle_recomposed_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let removed = with_slot_table_mut(|table| {
        let epoch = table.epoch;
        let mut freed = Vec::new();

        for slot in table.slots_for_instances(instance_logic_ids) {
            let Some(entry) = table.entries.get(slot) else {
                continue;
            };
            if entry.last_alive_epoch == epoch || entry.retained {
                continue;
            }
            freed.push(slot);
        }

        let mut removed = Vec::with_capacity(freed.len());
        for slot in freed {
            removed.extend(table.remove_slot(slot));
        }
        removed
    });
    drop(removed);
}

pub(crate) fn live_slot_instance_logic_ids() -> HashSet<u64> {
    with_slot_table(|table| table.slots_by_owner.keys().copied().collect())
}

/// Initializes state by an explicit key within the current component build.
///
/// Use stable business keys for reordered collections. The initializer runs only
/// when the slot is first used; see [`State`] for access and lifetime rules.
pub fn remember_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    resolve_state_slot(key, init, false, "remember_with_key")
}

/// Initializes state by call order and control-flow position during a component build.
///
/// Use [`remember_with_key`] for reordered collections; see [`State`] for access
/// and lifetime rules.
pub fn remember<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    remember_with_key((), init)
}

/// Initializes keyed state during a build and keeps it until the component unmounts.
///
/// Unused retained slots survive conditional branches; see [`State`] for access
/// rules and [`StateOwner`] for independent controller or route lifetimes.
pub fn retain_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    resolve_state_slot(key, init, true, "retain_with_key")
}

/// Initializes state by call order during a build, retaining it until unmount.
///
/// Use [`retain_with_key`] for reordered collections; see [`State`] for access
/// and lifetime rules.
pub fn retain<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    retain_with_key((), init)
}

/// Groups the execution of a block of code with a stable key.
///
/// This is useful for maintaining state identity in dynamic lists or loops
/// where the order of items might change.
///
/// # Examples
///
/// ```
/// use tessera_ui::{key, remember, tessera};
///
/// #[tessera]
/// fn my_list(items: Vec<String>) {
///     for item in items.iter() {
///         key(item.clone(), || {
///             let state = remember(|| 0);
///         });
///     }
/// }
/// ```
pub fn key<K, F, R>(key: K, block: F) -> R
where
    K: Hash,
    F: FnOnce() -> R,
{
    let key_hash = hash_components(&[&key]);
    let _group_guard = GroupGuard::keyed(key_hash);
    let _instance_guard = InstanceKeyGuard::new(key_hash);
    block()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::component_tree::{ComponentNode, NodeRole};
    use crate::execution_context::{
        reset_execution_context, with_execution_context, with_execution_context_mut,
    };
    use crate::layout::MeasureScope;
    use crate::modifier::{LayoutModifierChild, LayoutModifierInput, LayoutModifierNode};
    use crate::prop::{CallbackWith, RenderSlot};
    use crate::tessera;

    #[derive(Clone, Default, PartialEq)]
    struct ConditionalLoopProbe {
        values: crate::testing::TestProbe<[Option<State<usize>>; 3]>,
        order: crate::testing::TestProbe<Option<State<Vec<usize>>>>,
        tick: crate::testing::TestProbe<Option<State<usize>>>,
    }

    #[derive(Clone, PartialEq)]
    struct ConditionalLoopWidth(State<usize>);

    impl LayoutModifierNode for ConditionalLoopWidth {
        fn measure(
            &self,
            _scope: &mut MeasureScope<'_>,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            Ok(crate::LayoutResult::new(crate::ComputedData {
                width: crate::Px(self.0.get() as i32),
                height: crate::Px(10),
            }))
        }
    }

    struct ConditionalLoopTag(usize);

    impl crate::SemanticsModifierNode for ConditionalLoopTag {
        fn apply(
            &self,
            semantics: &mut crate::AccessibilityNode,
            _handler: &mut Option<crate::AccessibilityActionHandler>,
        ) {
            *semantics = semantics.clone().with_key(format!("loop-item-{}", self.0));
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct ConditionalLoopRow;

    impl LayoutModifierNode for ConditionalLoopRow {
        fn measure(
            &self,
            scope: &mut MeasureScope<'_>,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            let mut result = crate::LayoutResult::new(crate::ComputedData::min_from_constraint(
                scope.parent_constraint().as_ref(),
            ));
            let mut x = crate::Px::ZERO;
            for child in scope.children() {
                let measured = child.measure(scope, &crate::Constraint::NONE)?;
                result.place_child(measured, crate::PxPosition::new(x, crate::Px::ZERO));
                x += measured.width;
            }
            Ok(result)
        }
    }

    #[tessera(crate)]
    fn conditional_loop_item(index: usize, probe: ConditionalLoopProbe) {
        let value = remember(|| index);
        let mut values = probe.values.get();
        values[index] = Some(value);
        probe.values.set(values);
        crate::layout::layout()
            .policy(ConditionalLoopWidth(value))
            .modifier(crate::Modifier::new().push_semantics(ConditionalLoopTag(index)));
    }

    #[tessera(crate)]
    fn conditional_loop_fixture(keyed: bool, probe: ConditionalLoopProbe) {
        let order = remember(|| vec![0, 1, 2]);
        let tick = remember(|| 0);
        probe.order.set(Some(order));
        probe.tick.set(Some(tick));
        crate::layout::layout()
            .policy(ConditionalLoopRow)
            .child(move || {
                let _ = tick.get();
                for index in order.get() {
                    if index > 0 {
                        if keyed {
                            key(index, || {
                                conditional_loop_item(index, probe.clone());
                            });
                        } else {
                            conditional_loop_item(index, probe.clone());
                        }
                    }
                }
            });
    }

    #[test]
    fn conditional_loop_components_keep_independent_state_through_replay_and_keyed_reorder() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        for keyed in [false, true] {
            let probe = ConditionalLoopProbe::default();
            let mut session = crate::testing::start_layout_test_session(
                crate::testing::layout_test({
                    let probe = probe.clone();
                    move || conditional_loop_fixture(keyed, probe.clone())
                })
                .viewport_px(100, 20),
            );
            let initial = crate::testing::pump_layout_test_session(&mut session, 0);
            let values = probe.values.get();
            let first = values[1].unwrap();
            let second = values[2].unwrap();
            assert_eq!(first.get(), 1);
            assert_eq!(second.get(), 2);
            initial.node("loop-item-1").size(1, 10).position(0, 0);
            initial.node("loop-item-2").size(2, 10).position(1, 0);

            first.set(11);
            let changed = crate::testing::pump_layout_test_session(&mut session, 16_666_667);
            changed.node("loop-item-1").size(11, 10).position(0, 0);
            changed.node("loop-item-2").size(2, 10).position(11, 0);
            assert_eq!(second.get(), 2);
            probe.tick.get().unwrap().set(1);
            let replayed = crate::testing::pump_layout_test_session(&mut session, 33_333_334);
            replayed.node("loop-item-1").size(11, 10).position(0, 0);
            replayed.node("loop-item-2").size(2, 10).position(11, 0);

            if keyed {
                probe.order.get().unwrap().set(vec![0, 2, 1]);
                let reordered = crate::testing::pump_layout_test_session(&mut session, 50_000_001);
                reordered.node("loop-item-2").size(2, 10).position(0, 0);
                reordered.node("loop-item-1").size(11, 10).position(2, 0);
                assert!(probe.values.get()[1].unwrap() == first);
                assert!(probe.values.get()[2].unwrap() == second);

                probe.order.get().unwrap().set(vec![0, 2]);
                let removed = crate::testing::pump_layout_test_session(&mut session, 66_666_668);
                removed.node("loop-item-2").size(2, 10).position(0, 0);
                assert!(!first.is_alive());
                assert!(second.is_alive());
            }
        }
    }

    #[tessera(crate)]
    fn required_value_component(value: Option<usize>, on_value: Option<CallbackWith<usize>>) {
        let value = value.expect("missing required prop `value` in `required_value_component`");
        if let Some(on_value) = on_value {
            on_value.call(value);
        }
    }

    #[tessera(crate)]
    fn optional_value_component(
        value: Option<usize>,
        on_value: Option<CallbackWith<Option<usize>>>,
    ) {
        if let Some(on_value) = on_value {
            on_value.call(value);
        }
    }

    fn seed_previous_replay_snapshot(instance_key: u64) {
        #[derive(Clone)]
        struct Noop;

        impl Prop for Noop {
            fn prop_eq(&self, _: &Self) -> bool {
                true
            }

            fn run_component(self: Arc<Self>) {}
        }

        with_component_replay_tracker_mut(|tracker| {
            tracker.previous_nodes.insert(
                instance_key,
                ReplayNodeSnapshot {
                    instance_logic_id: 0,
                    group_path: GroupPath::default(),
                    instance_key_override: None,
                    context: ContextMap::default(),
                    replay: Arc::new(Noop),
                },
            );
        });
    }

    #[test]
    fn removing_a_replay_snapshot_preserves_lookup_for_the_moved_snapshot() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        for key in [11, 22, 33] {
            seed_previous_replay_snapshot(key);
        }

        remove_previous_component_replay_nodes(&[11].into_iter().collect());
        assert_eq!(previous_component_replay_nodes(&[11]).err(), Some(11));
        assert!(previous_component_replay_nodes(&[33, 22]).is_ok());

        remove_previous_component_replay_nodes(&[33].into_iter().collect());
        assert_eq!(previous_component_replay_nodes(&[33]).err(), Some(33));
        assert!(previous_component_replay_nodes(&[22]).is_ok());
    }

    fn with_test_component_scope<R>(component_type_id: u64, f: impl FnOnce() -> R) -> R {
        reset_execution_context();
        let mut arena = crate::Arena::<()>::new();
        let node_id = arena.new_node(());
        let _phase_guard = push_phase(RuntimePhase::Build);
        let _node_guard = push_current_node(node_id, component_type_id, "test_component");
        let _instance_guard = push_current_component_instance_key(current_instance_key());
        crate::component_call::with_pending_scope(f)
    }

    #[test]
    fn state_liveness_survives_updates_and_rejects_expired_slots() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let state = with_test_component_scope(815, || remember(|| 4_u32));
        assert!(state.is_alive());
        let other = with_test_component_scope(816, || remember(|| 0_u32));
        state.with(|value| {
            other.set(*value + 1);
            assert_eq!(state.get(), 4);
            assert!(std::panic::catch_unwind(|| state.set(9)).is_err());
            assert_eq!(state.get(), 4);
        });
        assert_eq!(other.get(), 5);
        state.with_mut(|value| {
            assert!(state.is_alive());
            *value = 5;
        });
        assert_eq!(state.get(), 5);
        reset_slots();
        assert!(!state.is_alive());
        let replacement = with_test_component_scope(815, || remember(|| 7_u32));
        assert!(replacement.is_alive());
        assert!(!state.is_alive());
    }

    #[test]
    fn receive_frame_nanos_panics_without_component_scope() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_frame_clock();
        begin_frame_clock(Instant::now());

        let result = std::panic::catch_unwind(|| {
            receive_frame_nanos(|_| FrameNanosControl::Continue);
        });
        assert!(result.is_err());
    }

    #[test]
    fn tick_frame_nanos_receivers_removes_stopped_receivers() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_frame_clock();
        begin_frame_clock(Instant::now());

        with_frame_clock_tracker_mut(|tracker| {
            tracker.receivers.insert(
                FrameNanosReceiverKey {
                    instance_logic_id: 1,
                    receiver_hash: 1,
                },
                FrameNanosReceiver {
                    owner_instance_key: 123,
                    callback: Box::new(|_| FrameNanosControl::Stop),
                },
            );
        });

        tick_frame_nanos_receivers();
        assert!(with_frame_clock_tracker(|tracker| tracker
            .receivers
            .is_empty()));
    }

    #[test]
    fn with_build_dirty_instance_keys_marks_current_scope() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut outer = HashSet::default();
        outer.insert(7);

        assert!(!is_instance_key_build_dirty(7));
        with_build_dirty_instance_keys(Arc::new(outer), || {
            assert!(is_instance_key_build_dirty(7));
            assert!(!is_instance_key_build_dirty(8));

            let mut inner = HashSet::default();
            inner.insert(8);
            with_build_dirty_instance_keys(Arc::new(inner), || {
                assert!(!is_instance_key_build_dirty(7));
                assert!(is_instance_key_build_dirty(8));
            });

            assert!(is_instance_key_build_dirty(7));
            assert!(!is_instance_key_build_dirty(8));
        });
        assert!(!is_instance_key_build_dirty(7));
    }

    #[test]
    fn with_build_dirty_instance_keys_restores_on_panic() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut dirty = HashSet::default();
        dirty.insert(11);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_build_dirty_instance_keys(Arc::new(dirty), || {
                assert!(is_instance_key_build_dirty(11));
                panic!("expected panic");
            });
        }));
        assert!(result.is_err());
        assert!(!is_instance_key_build_dirty(11));
    }

    #[derive(Clone, PartialEq)]
    struct DirtySplitPolicy {
        measure_key: u32,
        placement_key: u32,
    }

    impl crate::modifier::LayoutModifierNode for DirtySplitPolicy {
        fn measure(
            &self,
            _input: &mut MeasureScope<'_>,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            Ok(crate::LayoutResult::new(crate::ComputedData::ZERO))
        }

        fn measure_eq(&self, other: &Self) -> bool {
            self.measure_key == other.measure_key
        }

        fn placement_eq(&self, other: &Self) -> bool {
            self.placement_key == other.placement_key
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    struct DirtyMeasureModifierNode {
        width: u32,
    }

    impl LayoutModifierNode for DirtyMeasureModifierNode {
        fn measure_content(
            &self,
            _input: &LayoutModifierInput<'_>,
            child: &mut dyn LayoutModifierChild,
        ) -> Result<crate::LayoutResult, crate::MeasurementError> {
            let size = child.measure(&crate::Constraint::exact(
                crate::Px::new(self.width as i32),
                crate::Px::new(10),
            ))?;
            child.place(crate::PxPosition::ZERO);
            Ok(crate::LayoutResult::new(size))
        }
    }

    #[test]
    fn layout_dirty_tracking_separates_measure_and_placement_changes() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_layout_dirty_tracking();

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            record_layout_policy_dirty(
                1,
                &LayoutModifierHandle::from(DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                }),
                &Modifier::new(),
            );
        }
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(dirty.measure_self_nodes.contains(&1));
        assert!(dirty.placement_self_nodes.is_empty());

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            record_layout_policy_dirty(
                1,
                &LayoutModifierHandle::from(DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 1,
                }),
                &Modifier::new(),
            );
        }
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(!dirty.measure_self_nodes.contains(&1));
        assert!(dirty.placement_self_nodes.contains(&1));

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            record_layout_policy_dirty(
                1,
                &LayoutModifierHandle::from(DirtySplitPolicy {
                    measure_key: 1,
                    placement_key: 1,
                }),
                &Modifier::new(),
            );
        }
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(dirty.measure_self_nodes.contains(&1));
        assert!(!dirty.placement_self_nodes.contains(&1));
    }

    #[test]
    fn measure_state_write_dirties_layout_before_next_build() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_layout_dirty_tracking();
        begin_recompose_slot_epoch();
        let root_node = TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .add_node(ComponentNode::new("root", NodeRole::Layout))
        });

        let state = {
            let _phase_guard = push_phase(RuntimePhase::Build);
            let _node_guard = push_current_node(root_node, 9201, "root");
            remember(|| 0i32)
        };
        let instance_key = {
            let _phase_guard = push_phase(RuntimePhase::Measure);
            let _node_guard = push_current_node_with_instance_logic_id(root_node, 9201, "root");
            let _instance_guard = push_current_component_instance_key(77);
            state.with(|value| *value)
        };
        assert_eq!(instance_key, 0);

        state.set(1);
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(dirty.measure_self_nodes.contains(&77));
    }

    #[test]
    fn layout_dirty_tracking_marks_measure_when_layout_modifier_changes() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_layout_dirty_tracking();

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            let modifier = Modifier::new().push_layout(DirtyMeasureModifierNode { width: 10 });
            record_layout_policy_dirty(
                1,
                &LayoutModifierHandle::from(DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                }),
                &modifier,
            );
        }
        finalize_frame_layout_dirty_tracking();
        let _ = take_layout_dirty_nodes();

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            let modifier = Modifier::new().push_layout(DirtyMeasureModifierNode { width: 20 });
            record_layout_policy_dirty(
                1,
                &LayoutModifierHandle::from(DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                }),
                &modifier,
            );
        }
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(dirty.measure_self_nodes.contains(&1));
        assert!(!dirty.placement_self_nodes.contains(&1));
    }

    #[test]
    fn with_replay_scope_restores_group_path_and_override() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let _outer = PathGroupGuard::new(1);
        let _middle = PathGroupGuard::new(2);
        let _inner = PathGroupGuard::new(3);
        let original = current_group_path();
        let replay = GroupPath::default().extend(7).extend(8);
        with_execution_context_mut(|context| {
            context.instance_key_override = Some(5);
        });
        with_execution_context_mut(|context| {
            context.next_node_instance_logic_id_override = Some(9);
        });

        with_replay_scope(42, replay, Some(11), || {
            assert_eq!(current_group_path(), replay);
            assert_eq!(current_instance_key_override(), Some(11));
            assert_eq!(take_next_node_instance_logic_id_override(), Some(42));
            assert_eq!(take_next_node_instance_logic_id_override(), None);
        });

        assert_eq!(current_group_path(), original);
        assert_eq!(current_instance_key_override(), Some(5));
        let restored_override =
            with_execution_context(|context| context.next_node_instance_logic_id_override);
        assert_eq!(restored_override, Some(9));
    }

    #[test]
    fn with_replay_scope_restores_on_panic() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let _outer = PathGroupGuard::new(5);
        let original = current_group_path();
        let replay = GroupPath::default().extend(10);
        with_execution_context_mut(|context| {
            context.instance_key_override = Some(13);
        });
        with_execution_context_mut(|context| {
            context.next_node_instance_logic_id_override = None;
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_replay_scope(77, replay, Some(17), || {
                assert_eq!(current_group_path(), replay);
                assert_eq!(current_instance_key_override(), Some(17));
                panic!("expected panic");
            });
        }));
        assert!(result.is_err());

        assert_eq!(current_group_path(), original);
        assert_eq!(current_instance_key_override(), Some(13));
        let restored_override =
            with_execution_context(|context| context.next_node_instance_logic_id_override);
        assert_eq!(restored_override, None);
    }

    #[test]
    fn group_local_remember_does_not_shift_following_slots() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();

        begin_recompose_slot_epoch();
        with_test_component_scope(1001, || {
            let stable_state = remember(|| 1usize);
            stable_state.set(41);
        });

        begin_recompose_slot_epoch();
        with_test_component_scope(1001, || {
            {
                let _group_guard = GroupGuard::new(7);
                let _branch_state = remember(|| 10usize);
            }
            let stable_state = remember(|| 1usize);
            assert_eq!(stable_state.get(), 41);
        });
    }

    #[test]
    fn conditional_frame_receiver_does_not_shift_following_remember_slots() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        reset_frame_clock();
        begin_frame_clock(Instant::now());

        begin_recompose_slot_epoch();
        with_test_component_scope(1002, || {
            let stable_state = remember(|| 1usize);
            stable_state.set(99);
        });

        begin_recompose_slot_epoch();
        with_test_component_scope(1002, || {
            {
                let _group_guard = GroupGuard::new(9);
                receive_frame_nanos(|_| FrameNanosControl::Stop);
            }
            let stable_state = remember(|| 1usize);
            assert_eq!(stable_state.get(), 99);
        });
    }

    #[test]
    fn tessera_uses_none_for_omitted_option_props() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let observed = crate::testing::TestProbe::new(99usize);

        with_test_component_scope(11014, || {
            let observed = observed.clone();
            optional_value_component().on_value(move |value: Option<usize>| {
                observed.set(value.map_or(0, |next| next.saturating_add(1)));
            });
        });

        assert_eq!(observed.get(), 0);
    }

    #[test]
    fn tessera_accepts_non_default_required_props_when_provided() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let observed = crate::testing::TestProbe::new(0usize);

        with_test_component_scope(11016, || {
            let observed = observed.clone();
            required_value_component()
                .value(13)
                .on_value(move |value: usize| {
                    observed.set(value);
                });
        });

        assert_eq!(observed.get(), 13);
    }

    #[test]
    fn render_slot_update_invalidates_reader_instance() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        reset_component_replay_tracking();
        reset_render_slot_read_dependencies();
        reset_build_invalidations();

        let reader_boundary_key = crate::testing::TestProbe::new(0u64);

        begin_recompose_slot_epoch();
        let first = with_test_component_scope(11002, || {
            let reader_boundary_key = reader_boundary_key.clone();
            RenderSlot::new(move || {
                let instance_key = current_replay_boundary_instance_key_from_scope()
                    .expect("slot render must run inside replay boundary");
                reader_boundary_key.set(instance_key);
            })
        });

        with_test_component_scope(11003, || {
            first.render();
        });
        let reader_instance_key = reader_boundary_key.get();
        assert!(reader_instance_key != 0);
        seed_previous_replay_snapshot(reader_instance_key);

        assert!(!has_pending_build_invalidations());

        begin_recompose_slot_epoch();
        let second = with_test_component_scope(11002, || RenderSlot::new(|| {}));

        assert!(first == second);
        assert!(has_pending_build_invalidations());

        let invalidations = take_build_invalidations();
        let mut expected = HashSet::default();
        expected.insert(reader_instance_key);
        assert_eq!(invalidations.dirty_instance_keys, expected);
    }

    #[test]
    fn render_slot_update_skips_reader_already_dirty_in_current_build() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        reset_component_replay_tracking();
        reset_render_slot_read_dependencies();
        reset_build_invalidations();
        reset_execution_context();

        let reader_boundary_key = crate::testing::TestProbe::new(0u64);

        begin_recompose_slot_epoch();
        let first = with_test_component_scope(12002, || {
            let reader_boundary_key = reader_boundary_key.clone();
            RenderSlot::new(move || {
                let instance_key = current_replay_boundary_instance_key_from_scope()
                    .expect("slot render must run inside replay boundary");
                reader_boundary_key.set(instance_key);
            })
        });

        with_test_component_scope(12003, || {
            first.render();
        });
        let reader_instance_key = reader_boundary_key.get();
        assert!(reader_instance_key != 0);
        seed_previous_replay_snapshot(reader_instance_key);

        assert!(!has_pending_build_invalidations());

        begin_recompose_slot_epoch();
        let mut dirty = HashSet::default();
        dirty.insert(reader_instance_key);
        with_build_dirty_instance_keys(Arc::new(dirty), || {
            let mut arena = crate::Arena::<()>::new();
            let node_id = arena.new_node(());
            let _phase_guard = push_phase(RuntimePhase::Build);
            let _node_guard = push_current_node(node_id, 12002, "test_component");
            let _instance_guard = push_current_component_instance_key(current_instance_key());
            let second = RenderSlot::new(|| {});
            assert!(first == second);
        });

        assert!(!has_pending_build_invalidations());
    }

    #[test]
    fn group_local_child_identity_does_not_shift_following_siblings() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        fn stable_child_instance_logic_id(with_group_child: bool) -> u64 {
            let mut arena = crate::Arena::<()>::new();
            let root_node = arena.new_node(());
            let stable_child_node = arena.new_node(());
            let group_child_node = arena.new_node(());

            let _phase_guard = push_phase(RuntimePhase::Build);
            let _root_guard = push_current_node(root_node, 2001, "root_component");
            let _root_instance_guard = push_current_component_instance_key(current_instance_key());

            if with_group_child {
                let _group_guard = GroupGuard::new(5);
                let _group_child_guard =
                    push_current_node(group_child_node, 2002, "group_child_component");
                let _group_child_instance_guard =
                    push_current_component_instance_key(current_instance_key());
                let _ = current_instance_logic_id();
            }

            let _stable_child_guard =
                push_current_node(stable_child_node, 2003, "stable_child_component");
            current_instance_logic_id()
        }

        assert_eq!(
            stable_child_instance_logic_id(false),
            stable_child_instance_logic_id(true)
        );
    }

    #[test]
    fn child_components_in_different_groups_get_distinct_instance_logic_ids() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut arena = crate::Arena::<()>::new();
        let root_node = arena.new_node(());
        let first_child_node = arena.new_node(());
        let second_child_node = arena.new_node(());

        let _phase_guard = push_phase(RuntimePhase::Build);
        let _root_guard = push_current_node(root_node, 3001, "root_component");
        let _root_instance_guard = push_current_component_instance_key(current_instance_key());

        let first_id = {
            let _group_guard = GroupGuard::new(11);
            let _child_guard = push_current_node(first_child_node, 3002, "grouped_child");
            current_instance_logic_id()
        };

        let second_id = {
            let _group_guard = GroupGuard::new(12);
            let _child_guard = push_current_node(second_child_node, 3002, "grouped_child");
            current_instance_logic_id()
        };

        assert_ne!(first_id, second_id);
    }

    #[test]
    fn child_components_in_repeated_path_groups_keep_distinct_instance_logic_ids() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut arena = crate::Arena::<()>::new();
        let root_node = arena.new_node(());
        let first_child_node = arena.new_node(());
        let second_child_node = arena.new_node(());

        let _phase_guard = push_phase(RuntimePhase::Build);
        let _root_guard = push_current_node(root_node, 4001, "root_component");
        let _root_instance_guard = push_current_component_instance_key(current_instance_key());

        let first_id = {
            let _group_guard = PathGroupGuard::new(21);
            let _child_guard = push_current_node(first_child_node, 4002, "loop_child");
            current_instance_logic_id()
        };

        let second_id = {
            let _group_guard = PathGroupGuard::new(21);
            let _child_guard = push_current_node(second_child_node, 4002, "loop_child");
            current_instance_logic_id()
        };

        assert_ne!(first_id, second_id);
    }

    #[test]
    fn conditional_remember_keeps_later_state_stable() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        begin_recompose_slot_epoch();
        let root_node = TesseraRuntime::with_mut(|runtime| {
            runtime
                .component_tree
                .add_node(ComponentNode::new("root", NodeRole::Composition))
        });
        let _phase_guard = push_phase(RuntimePhase::Build);

        let stable = {
            let _node_guard = push_current_node(root_node, 9101, "root");
            remember(|| 11u32)
        };
        stable.set(23);

        begin_recompose_slot_epoch();
        {
            let _node_guard = push_current_node(root_node, 9101, "root");
            let _ = remember(|| current_frame_nanos());
            let shifted = remember(|| 99u32);
            assert_eq!(shifted.get(), 99);
        }
        assert_eq!(stable.get(), 23);

        begin_recompose_slot_epoch();
        {
            let _node_guard = push_current_node(root_node, 9101, "root");
            let restored = remember(|| 77u32);
            assert_eq!(restored.get(), 23);
        }
        assert_eq!(stable.get(), 23);
    }

    #[test]
    fn drop_slots_for_instance_logic_ids_keeps_retained_entries() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let mut table = SlotTable::default();
        let keep_key = SlotKey {
            owned_owner: None,
            instance_logic_id: 7,
            slot_hash: 11,
            type_id: TypeId::of::<i32>(),
        };
        let drop_key = SlotKey {
            owned_owner: None,
            instance_logic_id: 7,
            slot_hash: 12,
            type_id: TypeId::of::<i32>(),
        };

        let keep_slot = table.entries.insert(SlotEntry {
            key: keep_key,
            generation: 1,
            value: Some(Arc::new(10_i32)),
            last_alive_epoch: 0,
            retained: true,
        });
        let drop_slot = table.entries.insert(SlotEntry {
            key: drop_key,
            generation: 1,
            value: Some(Arc::new(20_i32)),
            last_alive_epoch: 0,
            retained: false,
        });
        table.index_slot(keep_slot);
        table.index_slot(drop_slot);
        with_slot_table_mut(|slot_table| *slot_table = table);

        let mut stale = HashSet::default();
        stale.insert(7_u64);
        drop_slots_for_instance_logic_ids(&stale);

        with_slot_table(|table| {
            assert!(table.entries.get(keep_slot).is_some());
            assert_eq!(table.find_slot(&keep_key), Some(keep_slot));
            assert!(table.entries.get(drop_slot).is_none());
            assert_eq!(table.find_slot(&drop_key), None);
        });
    }
}
