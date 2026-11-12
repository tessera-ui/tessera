//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::hash_map::Entry,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use hashbrown::HashTable;
use rustc_hash::{FxBuildHasher, FxHashMap as HashMap, FxHashSet as HashSet};
use slotmap::{SlotMap, new_key_type};

use crate::{
    NodeId,
    accessibility::{AccessibilityActionHandler, AccessibilityNode},
    component_call::ComponentCalls,
    component_tree::{
        ComponentNode, ComponentTree, EmittedNode, NodeInput, NodeRole, SnapshotUpdate,
    },
    context::{ContextGlobals, ContextMap},
    execution_context::{
        ExecutionContext, OrderFrame, with_execution_context, with_execution_context_mut,
    },
    focus::{
        FocusDirection, FocusGroupNode, FocusHandleId, FocusNode, FocusProperties,
        FocusRegistration, FocusRegistrationKind, FocusRequester, FocusRequesterId,
        FocusRevealRequest, FocusScopeNode, FocusState, FocusTraversalPolicy,
    },
    inline_set::InlineSet,
    layout::{DrawModifierHandle, LayoutPolicyHandle},
    modifier::{Modifier, ModifierChanges},
    prop::{CallbackWith, ErasedComponent, Prop},
    read_dependencies::ReadDependencies,
    session::{try_with_runtime_globals, with_runtime_globals},
    time::Instant,
};

mod model;

pub use model::{Model, ModelInit, ModelStore, ModelStoreScope};
pub(crate) use model::{ModelScope, inject_model};

#[derive(Clone, Copy)]
enum OrderCounterKind {
    Remember,
    Functor,
    Context,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum SlotKind {
    State,
    Functor,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct SlotKey {
    kind: SlotKind,
    owned_owner: Option<u128>,
    instance_logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

impl Default for SlotKey {
    fn default() -> Self {
        Self {
            kind: SlotKind::State,
            owned_owner: None,
            instance_logic_id: 0,
            slot_hash: 0,
            type_id: TypeId::of::<()>(),
        }
    }
}

new_key_type! {
    pub(crate) struct SlotHandle;
}

#[derive(Default)]
struct SlotEntry {
    key: SlotKey,
    generation: u64,
    // Functor cells are independently shared while callbacks execute. State
    // values stay in reclaimable boxes and use the access counters below.
    value: Option<Rc<dyn Any>>,
    state_value: Option<Box<dyn Any>>,
    active_reads: usize,
    active_write: bool,
    alive: bool,
    last_alive_epoch: u64,
    retained: bool,
}

impl SlotEntry {
    fn reuse_functor<T: 'static>(
        &mut self,
        slot: SlotHandle,
        epoch: u64,
        init: &mut Option<impl FnOnce() -> T>,
    ) -> (Rc<T>, u64) {
        debug_assert_eq!(self.key.type_id, TypeId::of::<T>());
        debug_assert!(self.alive);
        self.last_alive_epoch = epoch;
        let value = self.value.get_or_insert_with(|| {
            let value = Rc::new(init.take().expect("functor slot initialized once")());
            self.generation = self.generation.wrapping_add(1);
            value
        });
        (clone_slot_value(value, slot), self.generation)
    }
}

/// Slot order of one component instance, compared positionally on replay.
#[derive(Default)]
pub(crate) struct InstanceSlotCursor {
    order: crate::gap_buffer::GapBuffer<SlotHandle>,
    cursor: usize,
    epoch: u64,
}

impl InstanceSlotCursor {
    fn begin_epoch(&mut self, epoch: u64) {
        if self.epoch == epoch {
            return;
        }
        self.order.truncate_right();
        self.order.move_gap(0);
        self.cursor = 0;
        self.epoch = epoch;
    }

    fn fast_candidate(&self) -> Option<SlotHandle> {
        self.order.get(self.cursor).copied()
    }

    fn record_fast_match(&mut self, slot: SlotHandle) {
        self.cursor = self.cursor.saturating_add(1);
        self.order.move_gap(self.cursor);
        debug_assert_eq!(self.order.get(self.cursor - 1), Some(&slot));
    }

    fn record_slow_match(&mut self, slot: SlotHandle) {
        if let Some(index) = self
            .order
            .position_from(self.cursor, |candidate| *candidate == slot)
        {
            self.order.remove(index);
        }
        self.order.move_gap(self.cursor);
        self.order.insert_at_gap(slot);
        self.cursor += 1;
    }

    pub(crate) fn finish_epoch(&mut self) {
        self.order.truncate_right();
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
        let entry = self.entries.get_mut(slot)?;
        if !entry.alive {
            return None;
        }
        entry.alive = false;
        let key = entry.key;
        if entry.active_reads != 0 || entry.active_write {
            // Keep the slot and its value in place until the access guard exits.
            // It is already unreachable through keys and State::is_alive.
            return self.remove_slot_indexes(slot, key, false);
        }
        self.remove_slot_indexes(slot, key, true)
    }

    fn remove_slot_indexes(
        &mut self,
        slot: SlotHandle,
        key: SlotKey,
        remove_entry: bool,
    ) -> Option<SlotEntry> {
        if let Ok(indexed) = self
            .slots_by_key
            .find_entry(FxBuildHasher.hash_one(&key), |&candidate| candidate == slot)
        {
            indexed.remove();
        }
        if let Some(owner) = key.owned_owner {
            if let Some(slots) = self.owned_slots.get_mut(&owner) {
                slots.remove(&slot);
                if slots.is_empty() {
                    self.owned_slots.remove(&owner);
                }
            }
            return remove_entry.then(|| self.entries.remove(slot).expect("live slot entry"));
        }
        if let Entry::Occupied(mut slots) = self.slots_by_owner.entry(key.instance_logic_id)
            && slots.get_mut().remove_and_is_empty(&slot)
        {
            slots.remove();
        }
        remove_entry.then(|| self.entries.remove(slot).expect("live slot entry"))
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
        let handles = self.entries.keys().collect::<Vec<_>>();
        let removed = handles
            .into_iter()
            .filter_map(|slot| self.remove_slot(slot))
            .collect();
        self.slots_by_key.clear();
        self.slots_by_owner.clear();
        self.owned_slots.clear();
        self.cursors_by_instance_logic_id.clear();
        self.epoch = 0;
        removed
    }
    fn cursor_for<'a>(
        cursors: &'a mut HashMap<u64, InstanceSlotCursor>,
        key: &SlotKey,
        cursor: Option<&'a mut InstanceSlotCursor>,
    ) -> &'a mut InstanceSlotCursor {
        cursor.unwrap_or_else(|| cursors.entry(key.instance_logic_id).or_default())
    }

    /// Finds a slot at its previous position in the instance's slot order,
    /// falling back to the key index when the order changed.
    fn locate_slot(
        &mut self,
        key: &SlotKey,
        cursor: Option<&mut InstanceSlotCursor>,
    ) -> Option<SlotHandle> {
        let cursor = Self::cursor_for(&mut self.cursors_by_instance_logic_id, key, cursor);
        cursor.begin_epoch(self.epoch);
        if let Some(candidate) = cursor.fast_candidate()
            && self
                .entries
                .get(candidate)
                .is_some_and(|entry| entry.key == *key)
        {
            cursor.record_fast_match(candidate);
            return Some(candidate);
        }
        let entries = &self.entries;
        let slot = self
            .slots_by_key
            .find(FxBuildHasher.hash_one(key), |&slot| {
                entries[slot].key == *key
            })
            .copied()?;
        cursor.record_slow_match(slot);
        Some(slot)
    }

    fn record_new_slot(
        &mut self,
        key: &SlotKey,
        slot: SlotHandle,
        cursor: Option<&mut InstanceSlotCursor>,
    ) {
        let cursor = Self::cursor_for(&mut self.cursors_by_instance_logic_id, key, cursor);
        cursor.begin_epoch(self.epoch);
        cursor.record_slow_match(slot);
    }
}

/// Runs `access` with the slot table and the slot order of the executing call
/// that owns `key`, when that call is available.
fn with_slot_order<R>(
    key: &SlotKey,
    access: impl FnOnce(&mut SlotTable, Option<&mut InstanceSlotCursor>) -> R,
) -> R {
    with_runtime_globals(|globals| {
        let mut table = globals.slot_table.borrow_mut();
        // Build modifiers may run while the runtime is borrowed; they use the
        // indexed order instead.
        let mut runtime = globals.runtime.try_borrow_mut().ok();
        let cursor = runtime.as_mut().and_then(|runtime| {
            runtime
                .component_tree
                .executing_call_slots(key.instance_logic_id)
        });
        access(&mut table, cursor)
    })
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
    current: RefCell<Rc<dyn Fn()>>,
}

impl CallbackCell {
    fn new(current: Rc<dyn Fn()>) -> Self {
        Self {
            current: RefCell::new(current),
        }
    }

    fn update(&self, next: Rc<dyn Fn()>) {
        let previous = std::mem::replace(&mut *self.current.borrow_mut(), next);
        drop(previous);
    }

    fn shared(&self) -> Rc<dyn Fn()> {
        Rc::clone(&self.current.borrow())
    }
}

struct CallbackWithCell<T, R> {
    current: RefCell<Rc<dyn Fn(T) -> R>>,
}

impl<T, R> CallbackWithCell<T, R> {
    fn new(current: Rc<dyn Fn(T) -> R>) -> Self {
        Self {
            current: RefCell::new(current),
        }
    }

    fn update(&self, next: Rc<dyn Fn(T) -> R>) {
        let previous = std::mem::replace(&mut *self.current.borrow_mut(), next);
        drop(previous);
    }

    fn shared(&self) -> Rc<dyn Fn(T) -> R> {
        Rc::clone(&self.current.borrow())
    }
}

struct RenderSlotCell {
    current: RefCell<(Rc<dyn Fn()>, Option<TypeId>)>,
}

impl RenderSlotCell {
    fn new(current: Rc<dyn Fn()>, static_type: Option<TypeId>) -> Self {
        Self {
            current: RefCell::new((current, static_type)),
        }
    }

    fn update(&self, next: Rc<dyn Fn()>, static_type: Option<TypeId>) -> bool {
        let previous = {
            let mut current = self.current.borrow_mut();
            // A new allocation alone must not invalidate unchanged capture-free content.
            if static_type.is_some() && current.1 == static_type {
                return false;
            }
            std::mem::replace(&mut *current, (next, static_type))
        };
        drop(previous);
        true
    }

    fn shared(&self) -> Rc<dyn Fn()> {
        Rc::clone(&self.current.borrow().0)
    }
}

struct RenderSlotWithCell<T> {
    current: RefCell<(Rc<dyn Fn(T)>, Option<TypeId>)>,
}

impl<T> RenderSlotWithCell<T> {
    fn new(current: Rc<dyn Fn(T)>, static_type: Option<TypeId>) -> Self {
        Self {
            current: RefCell::new((current, static_type)),
        }
    }

    fn update(&self, next: Rc<dyn Fn(T)>, static_type: Option<TypeId>) -> bool {
        let previous = {
            let mut current = self.current.borrow_mut();
            // Closure type still distinguishes different capture-free bodies at one slot.
            if static_type.is_some() && current.1 == static_type {
                return false;
            }
            std::mem::replace(&mut *current, (next, static_type))
        };
        drop(previous);
        true
    }

    fn shared(&self) -> Rc<dyn Fn(T)> {
        Rc::clone(&self.current.borrow().0)
    }
}

#[derive(Default)]
struct LayoutDirtyTracker {
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
    pub replay: Rc<dyn ErasedComponent>,
}

/// Starts recording replay snapshots for a build pass.
pub(crate) fn begin_frame_component_replay_tracking() {
    TesseraRuntime::with_mut(|runtime| runtime.component_tree.begin_call_epoch());
}

/// Drops replay snapshots of removed calls once no runtime state is borrowed.
pub(crate) fn release_retired_replay_snapshots() {
    let retired =
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.take_retired_snapshots());
    drop(retired);
}

pub(crate) fn previous_component_context(key: u64) -> Option<ContextMap> {
    TesseraRuntime::with(|runtime| {
        runtime
            .component_tree
            .call_snapshot(key)
            .map(|snapshot| snapshot.context.clone())
    })
}
#[derive(Default)]
struct BuildInvalidationTracker {
    dirty_instance_keys: HashSet<u64>,
    cross_instance_dirty_instance_keys: HashSet<u64>,
    deferred_instance_keys: HashSet<u64>,
}

impl BuildInvalidationTracker {
    fn invalidate(&mut self, readers: impl IntoIterator<Item = u64>, writer: Option<u64>) -> bool {
        let mut wake = false;
        for reader in readers {
            if self.deferred_instance_keys.contains(&reader) {
                continue;
            }
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
            TesseraRuntime::with(|runtime| runtime.component_tree.has_call(key))
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
/// - the inherited context bindings captured at the original call site
pub(crate) fn with_replay_scope<R>(
    instance_logic_id: u64,
    group_path: GroupPath,
    instance_key_override: Option<u64>,
    context_map: ContextMap,
    f: impl FnOnce() -> R,
) -> R {
    struct ReplayScopeGuard {
        group_path: GroupPath,
        key_override: Option<u64>,
        logic_id_override: Option<u64>,
        context_map: ContextMap,
    }

    impl Drop for ReplayScopeGuard {
        fn drop(&mut self) {
            with_execution_context_mut(|context| {
                context.group_path = self.group_path;
                context.instance_key_override = self.key_override;
                context.next_node_instance_logic_id_override = self.logic_id_override;
                std::mem::swap(&mut context.context_map, &mut self.context_map);
            });
        }
    }

    let _guard = with_execution_context_mut(|context| ReplayScopeGuard {
        group_path: std::mem::replace(&mut context.group_path, group_path),
        key_override: std::mem::replace(&mut context.instance_key_override, instance_key_override),
        logic_id_override: context
            .next_node_instance_logic_id_override
            .replace(instance_logic_id),
        context_map: std::mem::replace(&mut context.context_map, context_map),
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
    if inserted && current_phase() != Some(RuntimePhase::Build) {
        schedule_runtime_redraw();
    }
}

/// Re-queues invalidations that could not be processed in the current frame.
pub(crate) fn defer_build_invalidations(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_build_invalidation_tracker_mut(|tracker| {
        for key in instance_keys {
            tracker.dirty_instance_keys.remove(key);
            tracker.cross_instance_dirty_instance_keys.remove(key);
            tracker.deferred_instance_keys.insert(*key);
        }
    });
    schedule_runtime_redraw();
}

pub(crate) fn with_pointer_state_reader<R>(key: u64, run: impl FnOnce() -> R) -> R {
    struct ReaderGuard(Option<u64>);
    impl Drop for ReaderGuard {
        fn drop(&mut self) {
            with_execution_context_mut(|context| context.pointer_state_reader = self.0);
        }
    }
    let _guard = ReaderGuard(with_execution_context_mut(|context| {
        context.pointer_state_reader.replace(key)
    }));
    run()
}

pub(crate) fn take_pointer_dirty_nodes() -> HashSet<u64> {
    with_layout_dirty_tracker_mut(|tracker| {
        std::mem::take(&mut tracker.pending_pointer_dirty_nodes)
    })
}

fn track_state_read_dependency(slot: SlotHandle, generation: u64) {
    let reader = with_execution_context(|context| {
        let pointer = context.pointer_state_reader;
        let measured = context.measured_node_instance_key_stack.last().copied();
        let component = context.current_component_instance_stack.last().copied();
        let phase = match context.phase_stack.last() {
            Some(RuntimePhase::Build) => return component.map(|key| (key, None)),
            Some(RuntimePhase::Input) if pointer.is_some() => ReadPhase::Pointer,
            Some(RuntimePhase::Measure) => ReadPhase::Measure,
            Some(RuntimePhase::Place) if measured.is_some() => ReadPhase::Place,
            Some(RuntimePhase::Place) => ReadPhase::Transform,
            Some(RuntimePhase::Paint) => ReadPhase::Paint,
            _ => return None,
        };
        pointer
            .or(measured)
            .or(component)
            .map(|key| (key, Some(phase)))
    });
    let key = StateReadDependencyKey { slot, generation };
    match reader {
        Some((instance_key, None)) => {
            with_state_read_dependency_tracker_mut(|tracker| tracker.record(key, instance_key));
        }
        Some((instance_key, Some(phase))) => {
            let reader = PhaseReader {
                instance_key,
                phase,
            };
            with_layout_dirty_tracker_mut(|tracker| tracker.state_reads.record(key, reader));
        }
        None => {}
    }
}

fn mark_layout_state_readers_dirty(keys: &[StateReadDependencyKey]) -> bool {
    with_layout_dirty_tracker_mut(|tracker| {
        let mut changed = false;
        for key in keys {
            if let Some(readers) = tracker.state_reads.readers(key) {
                for reader in readers {
                    let target = match reader.phase {
                        ReadPhase::Measure => &mut tracker.pending_measure_self_dirty_nodes,
                        ReadPhase::Place => &mut tracker.pending_placement_self_dirty_nodes,
                        ReadPhase::Transform => &mut tracker.pending_transform_dirty_nodes,
                        ReadPhase::Paint => &mut tracker.pending_paint_dirty_nodes,
                        ReadPhase::Pointer => &mut tracker.pending_pointer_dirty_nodes,
                    };
                    target.insert(reader.instance_key);
                    changed = true;
                }
            }
        }
        changed
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

fn invalidate_render_slot_readers(handle: FunctorHandle, creator_instance_key: u64) {
    with_execution_context(|context| {
        let dirty = context.build_dirty_instance_keys_stack.last();
        with_runtime_globals(|globals| {
            let readers = globals.render_slot_read_dependency_tracker.borrow();
            let Some(readers) = readers.readers(&handle) else {
                return;
            };
            let mut invalidations = globals.build_invalidation_tracker.borrow_mut();
            for instance_key in readers {
                if instance_key != creator_instance_key
                    && !dirty.is_some_and(|dirty| dirty.contains(&instance_key))
                {
                    invalidations.invalidate([instance_key], None);
                }
            }
        });
    });
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

pub(crate) fn begin_frame_build_invalidations() {
    with_build_invalidation_tracker_mut(|tracker| {
        tracker
            .dirty_instance_keys
            .extend(tracker.deferred_instance_keys.drain());
    });
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
            tracker.deferred_instance_keys.remove(key);
        }
    });
}

pub(crate) fn has_pending_build_invalidations() -> bool {
    with_build_invalidation_tracker(|tracker| {
        !tracker.dirty_instance_keys.is_empty() || !tracker.deferred_instance_keys.is_empty()
    })
}

pub(crate) fn has_ready_build_invalidations() -> bool {
    with_build_invalidation_tracker(|tracker| !tracker.dirty_instance_keys.is_empty())
}

#[derive(Default)]
struct FrameClockTracker {
    frame_origin: Option<Instant>,
    current_frame_time: Option<Instant>,
    previous_frame_time: Option<Instant>,
    frame_delta: Duration,
}

fn with_frame_clock_tracker<R>(f: impl FnOnce(&FrameClockTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&globals.frame_clock_tracker.borrow()))
}

fn with_frame_clock_tracker_mut<R>(f: impl FnOnce(&mut FrameClockTracker) -> R) -> R {
    with_runtime_globals(|globals| f(&mut globals.frame_clock_tracker.borrow_mut()))
}

pub(crate) fn begin_frame_clock(now: Instant) {
    with_frame_clock_tracker_mut(|tracker| {
        tracker.frame_origin.get_or_insert(now);
        tracker.previous_frame_time = tracker.current_frame_time;
        tracker.current_frame_time = Some(now);
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

pub(crate) fn has_pending_frame_waiters() -> bool {
    crate::async_dispatch::has_frame_waiters()
}

/// Frame-clock work performed by a completed frame.
#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
#[derive(Clone, Debug, Default)]
pub struct FrameClockDiagnostics {
    /// Runtime session that owns these tasks.
    pub session: Option<crate::RuntimeSessionId>,
    /// Number of tasks resumed by the frame.
    pub resumed_tasks: usize,
    /// Number of tasks waiting for the next frame.
    pub waiting_tasks: usize,
}

#[cfg(any(
    test,
    feature = "testing",
    all(feature = "headless", not(target_family = "wasm"))
))]
pub(crate) fn frame_clock_diagnostics() -> FrameClockDiagnostics {
    FrameClockDiagnostics {
        session: crate::RuntimeSession::current().map(|session| session.identity()),
        resumed_tasks: crate::async_dispatch::frame_waiters_resumed_last_frame(),
        waiting_tasks: crate::async_dispatch::frame_waiter_count(),
    }
}

pub(crate) fn tick_frame_clock() {
    with_batched_state_writes(|| {
        crate::dispatcher::flush();
        crate::async_dispatch::resume_frame_waiters();
    });
}

/// Returns the time of the current frame, or the current time before the
/// first frame.
pub(crate) fn frame_time() -> Instant {
    with_frame_clock_tracker(|tracker| tracker.current_frame_time).unwrap_or_else(Instant::now)
}

/// Nanoseconds since the first frame, for diagnostics.
pub(crate) fn frame_nanos_since_origin() -> u64 {
    with_frame_clock_tracker(|tracker| {
        tracker
            .current_frame_time
            .zip(tracker.frame_origin)
            .map_or(0, |(time, origin)| {
                time.saturating_duration_since(origin).as_nanos() as u64
            })
    })
}

/// Returns the elapsed time since the previous frame.
#[cfg(feature = "profiling")]
pub(crate) fn frame_delta() -> Duration {
    with_frame_clock_tracker(|tracker| tracker.frame_delta)
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

fn record_layout_node_dirty(
    previous: Option<(&LayoutPolicyHandle, ModifierChanges)>,
    node: &ComponentNode,
) {
    let instance_key = node.instance_key;
    let (measure_changed, placement_changed, transform_changed, parent_data_changed) =
        if let Some((policy, changes)) = previous {
            let policy = policy.as_policy();
            let current_policy = node.layout_policy_input.as_policy();
            (
                changes.measure || !policy.dyn_measure_eq(current_policy),
                changes.placement || !policy.dyn_placement_eq(current_policy),
                changes.transform,
                changes.parent_data,
            )
        } else {
            (true, true, true, true)
        };
    with_layout_dirty_tracker_mut(|tracker| {
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
            tracker.structural_changes.removed_nodes.insert(*key);
            tracker.structural_changes.changed_nodes.remove(key);
        }
    });
}

pub(crate) fn take_layout_structure_changes() -> StructureReconcileResult {
    with_layout_dirty_tracker_mut(|tracker| std::mem::take(&mut tracker.structural_changes))
}

/// Reactive state handle created by [`remember`], [`remember_with_key`], or
/// [`State::new`] during model initialization.
///
/// `State<T>` is a `Copy` handle with `with`, `with_mut`, `get`, `set`, and
/// `set_if_changed`; use [`State::set_many`] to update several states of the
/// same type together.
///
/// Component factories such as [`remember`] run only during a Tessera build.
/// Remembered state survives recomposition while its slot is used; [`retain`]
/// also keeps unused slots until the component unmounts. [`Model`] provides
/// an owner scope for states created while initializing its value.
///
/// The same methods work on every thread. When `T: Send` the handle is `Send`
/// and `Sync`; another thread's access waits for the owning UI session's next
/// frame boundary, so a frame never observes a write midway. Access rejects
/// stale handles and handles of a closed session.
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
    _marker: PhantomData<*const T>,
}

// SAFETY: a handle reaches the value only through its owning session, either
// on the owning thread or on another thread while that session's gate excludes
// the owner, so a `Send` value is never accessed concurrently.
unsafe impl<T: Send> Send for State<T> {}
// SAFETY: as above; shared handles only copy the identity.
unsafe impl<T: Send> Sync for State<T> {}

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
    fn from_slot(slot: SlotHandle, generation: u64) -> Self {
        Self {
            session: crate::dispatcher::register_current(),
            slot,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> State<T>
where
    T: 'static,
{
    /// Creates state owned by the model currently being initialized.
    ///
    /// Use `remember` for composition-slot state. This constructor is only
    /// available while a `Model<T>` initializer is running.
    pub fn new(value: T) -> Self {
        crate::runtime::model::create_model_state(value)
    }

    /// Runs `access` with this state's session active, waiting for the
    /// session's next frame boundary when called from another thread.
    fn in_session<R>(&self, access: impl FnOnce() -> R) -> Option<R> {
        if crate::session::current_session_identity() == Some(self.session) {
            Some(access())
        } else {
            crate::session::with_inactive_session(self.session, access)
        }
    }

    #[track_caller]
    fn in_live_session<R>(&self, access: impl FnOnce() -> R) -> R {
        self.in_session(access)
            .unwrap_or_else(|| panic!("state belongs to a closed runtime session"))
    }

    /// Returns whether this handle still refers to a live state slot.
    ///
    /// Remembered state expires when its component leaves the tree; model
    /// state expires when its model closes.
    pub fn is_alive(&self) -> bool {
        self.in_session(|| {
            try_with_runtime_globals(|globals| {
                let table = globals.slot_table.borrow();
                let Some(entry) = table.entries.get(self.slot) else {
                    return false;
                };

                entry.alive
                    && entry.generation == self.generation
                    && entry.key.type_id == TypeId::of::<T>()
            })
            .unwrap_or(false)
        })
        .unwrap_or(false)
    }

    #[track_caller]
    pub(crate) fn read_untracked<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        self.in_live_session(|| self.with_untracked(read))
    }

    #[track_caller]
    pub(crate) fn write_untracked<R>(&self, update: impl FnOnce(&mut T) -> R) -> R {
        self.in_live_session(|| {
            let (mut value, guard) = borrow_state::<T>(self, true);
            // SAFETY: as in `with_mut`; this path publishes no invalidation.
            let result = unsafe { update(value.as_mut()) };
            guard.release();
            result
        })
    }

    fn with_untracked<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        let (value, guard) = borrow_state::<T>(self, false);
        // SAFETY: the guard reserves a shared access to this stable value and
        // defers destruction until the callback exits. This path deliberately
        // does not register a dependency.
        let result = unsafe { read(value.as_ref()) };
        drop(guard);
        result
    }

    /// Execute a closure with a shared reference to the stored value.
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.in_live_session(|| {
            track_state_read_dependency(self.slot, self.generation);
            let (value, guard) = borrow_state::<T>(self, false);
            // SAFETY: the guard holds a shared borrow count on this stable boxed
            // value. Removal marks the slot dead but defers its destruction until
            // this guard exits; mutable access is rejected while readers exist.
            let result = unsafe { f(value.as_ref()) };
            drop(guard);
            result
        })
    }

    /// Execute a closure with a mutable reference to the stored value.
    #[track_caller]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.in_live_session(|| {
            let (mut value, guard) = borrow_state::<T>(self, true);
            // SAFETY: the guard exclusively reserves this stable value, releases
            // the table borrow before user code, and defers removal until it exits.
            let result = unsafe { f(value.as_mut()) };
            let alive = guard.release();
            if alive {
                publish_state_write(self.slot, self.generation);
            }
            result
        })
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
        self.in_live_session(|| self.set_if_changed_in_session(value))
    }

    fn set_if_changed_in_session(&self, value: T) -> bool
    where
        T: PartialEq,
    {
        let (mut stored, guard) = borrow_state::<T>(self, true);
        // SAFETY: this guard exclusively reserves the stable value. PartialEq
        // may re-enter the runtime, so it runs without the table borrow held.
        let changed = unsafe { stored.as_ref() != &value };
        let discarded = if changed {
            Some(unsafe { std::mem::replace(stored.as_mut(), value) })
        } else {
            Some(value)
        };
        let alive = guard.release();
        if changed && alive {
            publish_state_write(self.slot, self.generation);
        }
        drop(discarded);
        changed
    }

    /// Replace the stored value.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.in_live_session(|| self.set_in_session(value));
    }

    #[track_caller]
    fn set_in_session(&self, value: T) {
        let (discarded, alive) = with_slot_table_mut(|table| {
            let entry = table
                .entries
                .get_mut(self.slot)
                .unwrap_or_else(|| panic!("State points to freed slot: {:?}", self.slot));
            assert!(entry.alive, "State handle is stale");
            assert_eq!(entry.generation, self.generation, "State handle is stale");
            assert_eq!(entry.key.type_id, TypeId::of::<T>(), "State type mismatch");
            assert_eq!(
                entry.active_reads, 0,
                "State slot {:?} is being read",
                self.slot
            );
            assert!(
                !entry.active_write,
                "State slot {:?} is being written",
                self.slot
            );
            let stored = entry
                .state_value
                .as_deref_mut()
                .unwrap_or_else(|| panic!("State slot {:?} has been cleared", self.slot))
                .downcast_mut::<T>()
                .unwrap_or_else(|| panic!("State slot {:?} downcast failed", self.slot));
            (std::mem::replace(stored, value), true)
        });
        if alive {
            publish_state_write(self.slot, self.generation);
        }
        drop(discarded);
    }

    /// Atomically replaces several values of the same type and publishes their
    /// invalidations together.
    ///
    /// Every handle is validated before any value is changed. If validation
    /// fails, the batch leaves all states untouched.
    #[track_caller]
    pub fn set_many(updates: impl IntoIterator<Item = (Self, T)>) {
        let updates = updates.into_iter().collect::<Vec<_>>();
        let Some((first, _)) = updates.first() else {
            return;
        };
        let first = *first;
        first.in_live_session(|| Self::set_many_in_session(updates));
    }

    #[track_caller]
    fn set_many_in_session(updates: Vec<(Self, T)>) {
        let session = crate::session::current_session_identity();
        let mut keys = Vec::with_capacity(updates.len());
        let mut discarded = Vec::with_capacity(updates.len());
        with_slot_table_mut(|table| {
            for (state, _) in &updates {
                assert_eq!(
                    session,
                    Some(state.session),
                    "state belongs to a different runtime session"
                );
                let entry = table
                    .entries
                    .get(state.slot)
                    .unwrap_or_else(|| panic!("State points to freed slot: {:?}", state.slot));
                assert!(entry.alive, "State handle is stale");
                assert_eq!(entry.generation, state.generation, "State handle is stale");
                assert_eq!(entry.key.type_id, TypeId::of::<T>(), "State type mismatch");
                assert_eq!(
                    entry.active_reads, 0,
                    "State slot {:?} is being read",
                    state.slot
                );
                assert!(
                    !entry.active_write,
                    "State slot {:?} is being written",
                    state.slot
                );
                entry
                    .state_value
                    .as_ref()
                    .unwrap_or_else(|| panic!("State slot {:?} has been cleared", state.slot))
                    .downcast_ref::<T>()
                    .unwrap_or_else(|| panic!("State slot {:?} downcast failed", state.slot));
                keys.push(StateReadDependencyKey {
                    slot: state.slot,
                    generation: state.generation,
                });
            }

            for ((state, value), _) in updates.into_iter().zip(&keys) {
                let entry = table
                    .entries
                    .get_mut(state.slot)
                    .expect("state slot was validated");
                let stored = entry
                    .state_value
                    .as_deref_mut()
                    .expect("state slot was validated")
                    .downcast_mut::<T>()
                    .expect("state value type was validated");
                discarded.push(std::mem::replace(stored, value));
            }
        });

        publish_state_writes(&keys);
        drop(discarded);
    }
}

fn publish_state_write(slot: SlotHandle, generation: u64) {
    let key = StateReadDependencyKey { slot, generation };
    if with_state_write_batch(|batch| {
        if batch.depth == 0 {
            false
        } else {
            batch.pending.push(key);
            true
        }
    }) {
        return;
    }
    publish_state_writes(std::slice::from_ref(&key));
}

#[derive(Default)]
struct StateWriteBatch {
    depth: usize,
    pending: Vec<StateReadDependencyKey>,
}

fn with_state_write_batch<R>(update: impl FnOnce(&mut StateWriteBatch) -> R) -> R {
    with_runtime_globals(|globals| update(&mut globals.state_write_batch.borrow_mut()))
}

fn with_batched_state_writes<R>(run: impl FnOnce() -> R) -> R {
    with_state_write_batch(|batch| batch.depth += 1);
    struct BatchGuard;
    impl Drop for BatchGuard {
        fn drop(&mut self) {
            let pending = with_state_write_batch(|batch| {
                batch.depth -= 1;
                if batch.depth == 0 {
                    Some(std::mem::take(&mut batch.pending))
                } else {
                    None
                }
            });
            if let Some(mut pending) = pending {
                publish_state_writes(&pending);
                pending.clear();
                with_state_write_batch(|batch| {
                    pending.append(&mut batch.pending);
                    batch.pending = pending;
                });
            }
        }
    }
    let _guard = BatchGuard;
    run()
}

fn publish_state_writes(keys: &[StateReadDependencyKey]) {
    if keys.is_empty() {
        return;
    }
    // Writes during a build may invalidate readers that have already run, so
    // retain the writer identity for the same backwards-write handling as set.
    let writer_instance_key = with_execution_context(|context| {
        if context.phase_stack.last() == Some(&RuntimePhase::Build) {
            context.current_component_instance_stack.last().copied()
        } else {
            None
        }
    });
    let build_dirty = with_runtime_globals(|globals| {
        let dependencies = globals.state_read_dependency_tracker.borrow();
        let mut tracker = globals.build_invalidation_tracker.borrow_mut();
        keys.iter().fold(false, |wake, key| {
            wake | dependencies
                .readers(key)
                .is_some_and(|readers| tracker.invalidate(readers, writer_instance_key))
        })
    });
    let layout_dirty = mark_layout_state_readers_dirty(&keys);
    if build_dirty || layout_dirty {
        schedule_runtime_redraw();
    }
}

pub(crate) struct RuntimeGlobals {
    pub(crate) component_calls: ComponentCalls,
    pub(crate) context: ContextGlobals,
    slot_table: RefCell<SlotTable>,
    build_invalidation_tracker: RefCell<BuildInvalidationTracker>,
    state_read_dependency_tracker: RefCell<StateReadDependencyTracker>,
    focus_read_dependency_tracker: RefCell<FocusReadDependencyTracker>,
    render_slot_read_dependency_tracker: RefCell<RenderSlotReadDependencyTracker>,
    redraw_waker: RefCell<Option<RedrawWaker>>,
    persistent_focus_handle_store: RefCell<PersistentFocusHandleStore>,
    frame_clock_tracker: RefCell<FrameClockTracker>,
    layout_dirty_tracker: RefCell<LayoutDirtyTracker>,
    state_write_batch: RefCell<StateWriteBatch>,
    runtime: RefCell<TesseraRuntime>,
}

impl RuntimeGlobals {
    #[inline]
    pub(crate) fn with_runtime<R>(&self, read: impl FnOnce(&TesseraRuntime) -> R) -> R {
        read(&self.runtime.borrow())
    }

    #[inline]
    pub(crate) fn with_runtime_mut<R>(&self, update: impl FnOnce(&mut TesseraRuntime) -> R) -> R {
        update(&mut self.runtime.borrow_mut())
    }

    fn new() -> Self {
        Self {
            component_calls: ComponentCalls::default(),
            context: ContextGlobals::default(),
            slot_table: RefCell::new(SlotTable::default()),
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
            state_write_batch: RefCell::new(StateWriteBatch::default()),
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
    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        with_runtime_globals(|globals| globals.with_runtime(f))
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    #[inline]
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let result = with_runtime_globals(|globals| globals.with_runtime_mut(f));
        crate::modifier::flush_lifecycle();
        result
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
            node.modifier = std::mem::take(&mut node.modifier).then(modifier);
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
            metadatas
                .entry_or_default(node_id)
                .set_accessibility(accessibility, handler);
        } else if let Some(metadata) = metadatas.get_mut(&node_id) {
            metadata.set_accessibility(None, None);
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

pub(crate) fn reserve_component_identity_in(
    context: &mut ExecutionContext,
    component_type_id: u64,
) -> u64 {
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
        let identity = resolve_identity(context);
        let parent = push_node_context(context, node_id, identity);
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

fn push_node_context(
    context: &mut ExecutionContext,
    node_id: NodeId,
    identity: u64,
) -> Option<NodeId> {
    let parent = context.node_context_stack.last().copied();
    context.node_context_stack.push(node_id);
    context.instance_logic_id_stack.push(identity);
    context.order_frame_stack.push(OrderFrame::default());
    parent
}

struct ComponentBuildContext {
    entered_node: bool,
    #[cfg(feature = "profiling")]
    profiling_guard: Option<crate::profiler::ScopeGuard>,
}

impl ComponentBuildContext {
    fn begin(component_type_id: u64) -> (Self, u64, u64) {
        let (logic_id, instance_key) = with_execution_context_mut(|context| {
            let logic_id = reserve_component_identity_in(context, component_type_id);
            let instance_key = instance_key_in(context, logic_id);
            context.phase_stack.push(RuntimePhase::Build);
            context.current_component_instance_stack.push(instance_key);
            (logic_id, instance_key)
        });
        (
            Self {
                entered_node: false,
                #[cfg(feature = "profiling")]
                profiling_guard: None,
            },
            logic_id,
            instance_key,
        )
    }

    fn enter_node(&mut self, node_id: NodeId, logic_id: u64, name: &str) {
        let parent =
            with_execution_context_mut(|context| push_node_context(context, node_id, logic_id));
        self.entered_node = true;
        #[cfg(feature = "profiling")]
        {
            self.profiling_guard = crate::profiler::make_build_scope_guard(node_id, parent, name);
        }
        #[cfg(not(feature = "profiling"))]
        let _ = (parent, name);
    }
}

impl Drop for ComponentBuildContext {
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        drop(self.profiling_guard.take());
        with_execution_context_mut(|context| {
            let instance = context.current_component_instance_stack.pop();
            debug_assert!(instance.is_some(), "component instance must be active");
            if self.entered_node {
                pop_node_context(context);
            }
            let phase = context.phase_stack.pop();
            debug_assert_eq!(phase, Some(RuntimePhase::Build));
        });
    }
}

/// A component build scope that restores the previous node and build context on exit.
pub struct ComponentNodeScope {
    instance_logic_id: u64,
    retained: bool,
    _context: ComponentBuildContext,
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
        if self.retained {
            return;
        }
        TesseraRuntime::with_mut(|runtime| {
            let tree = &mut runtime.component_tree;
            let Some(EmittedNode { id, previous }) = tree.finish_call() else {
                return;
            };
            let node = tree.get_mut(id).expect("emitted component node");
            node.attached_modifiers.update(
                node.instance_logic_id,
                node.modifier.clone(),
                node.draw_node.clone(),
            );
            let changes = previous.as_ref().map(|previous| {
                let node = tree.get(id).expect("emitted component node");
                let changes = node.modifier.changes_from(&previous.modifier);
                tree.reconcile_build_node(id, previous, changes);
                (&previous.policy, changes)
            });
            let node = tree.get(id).expect("emitted component node");
            if node.role == NodeRole::Layout {
                record_layout_node_dirty(changes, node);
            }
            tree.pop_node();
        });
    }
}

/// Enter the build scope for one registered component call.
pub fn begin_component_node<P: Prop>(
    name: &'static str,
    component_type_id: u64,
    props: &Rc<P>,
) -> ComponentNodeScope {
    let replay = props;
    let props: &dyn ErasedComponent = props.as_ref();
    let (mut context, instance_logic_id, instance_key) =
        ComponentBuildContext::begin(component_type_id);
    let pending_dirty = consume_pending_build_invalidation(instance_key);
    let dirty = pending_dirty || is_instance_key_build_dirty(instance_key);
    let (host, retained) = TesseraRuntime::with_mut(|runtime| {
        let tree = &mut runtime.component_tree;
        let (call, update) = tree.locate_call(instance_key, |previous, executed| {
            if previous.instance_logic_id != instance_logic_id || !previous.replay.equals(props) {
                return SnapshotUpdate::Replace;
            }
            let same_scope = with_execution_context(|context| {
                previous.group_path == context.group_path
                    && previous.instance_key_override == context.instance_key_override
                    && previous.context.ptr_eq(&context.context_map)
            });
            if same_scope && !executed {
                SnapshotUpdate::Keep
            } else {
                SnapshotUpdate::Refresh
            }
        });
        let retained = !dirty
            && !matches!(update, SnapshotUpdate::Replace)
            && call.is_some_and(|id| tree.attach_retained_call(id, instance_logic_id));
        let id = match call {
            Some(id) if retained => id,
            call => tree.begin_call(call, instance_key, instance_logic_id, name),
        };
        match update {
            SnapshotUpdate::Keep => {}
            SnapshotUpdate::Refresh => tree.refresh_call_snapshot(id, |snapshot| {
                with_execution_context(|context| {
                    snapshot.group_path = context.group_path;
                    snapshot.instance_key_override = context.instance_key_override;
                    snapshot.context = context.context_map.clone();
                });
            }),
            SnapshotUpdate::Replace => {
                let snapshot = with_execution_context(|context| ReplayNodeSnapshot {
                    instance_logic_id,
                    group_path: context.group_path,
                    instance_key_override: context.instance_key_override,
                    context: context.context_map.clone(),
                    replay: replay.clone(),
                });
                tree.store_call_snapshot(id, snapshot);
            }
        }
        let host = tree
            .current_build_node_id()
            .expect("component calls execute inside a host node");
        (host, retained)
    });
    context.enter_node(host, instance_logic_id, name);
    ComponentNodeScope {
        instance_logic_id,
        retained,
        _context: context,
    }
}
/// Emits the tree node owned by the executing call and attaches its modifiers.
pub(crate) fn emit_node<P>(role: NodeRole, policy: &P, draw: DrawModifierHandle, modifier: Modifier)
where
    P: crate::modifier::LayoutPolicy + Clone + PartialEq,
{
    let (instance_key, instance_logic_id) = with_execution_context(|context| {
        (
            context
                .current_component_instance_stack
                .last()
                .copied()
                .expect("nodes are emitted by a component call"),
            context
                .instance_logic_id_stack
                .last()
                .copied()
                .expect("nodes are emitted by a component call"),
        )
    });
    let id = TesseraRuntime::with_mut(|runtime| {
        let tree = &mut runtime.component_tree;
        let (id, previous) = tree.rebuild_node(NodeInput {
            fn_name: tree.executing_call_name(),
            role,
            instance_logic_id,
            instance_key,
            policy,
            draw,
        });
        tree.emit_call_node(id, previous);
        id
    });
    with_execution_context_mut(|context| {
        *context
            .node_context_stack
            .last_mut()
            .expect("component calls enter a node context") = id;
    });
    modifier.attach();
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
    with_execution_context_mut(pop_node_context);
}

fn pop_node_context(context: &mut ExecutionContext) {
    let popped = context.node_context_stack.pop();
    debug_assert!(
        popped.is_some(),
        "Attempted to pop current node from an empty stack"
    );
    let order_frame = context.order_frame_stack.pop();
    debug_assert!(order_frame.is_some(), "component order frame must exist");
    context.instance_logic_id_stack.pop();
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

struct StateBorrowGuard {
    session: u128,
    slot: SlotHandle,
    generation: u64,
    mutable: bool,
    active: bool,
}

impl StateBorrowGuard {
    fn release(mut self) -> bool {
        self.active = false;
        release_state_borrow(self.session, self.slot, self.generation, self.mutable)
    }
}

impl Drop for StateBorrowGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = release_state_borrow(self.session, self.slot, self.generation, self.mutable);
        }
    }
}

fn borrow_state<T: 'static>(state: &State<T>, mutable: bool) -> (NonNull<T>, StateBorrowGuard) {
    let pointer = with_slot_table_mut(|table| {
        let entry = table
            .entries
            .get_mut(state.slot)
            .unwrap_or_else(|| panic!("State points to freed slot: {:?}", state.slot));
        assert!(entry.alive, "State handle is stale");
        assert_eq!(entry.generation, state.generation, "State handle is stale");
        assert_eq!(entry.key.type_id, TypeId::of::<T>(), "State type mismatch");
        if mutable {
            assert_eq!(
                entry.active_reads, 0,
                "State slot {:?} is being read",
                state.slot
            );
            assert!(
                !entry.active_write,
                "State slot {:?} is being written",
                state.slot
            );
        } else {
            assert!(
                !entry.active_write,
                "State slot {:?} is being written",
                state.slot
            );
        }
        let next_readers = (!mutable).then(|| {
            entry
                .active_reads
                .checked_add(1)
                .expect("state read borrow count exhausted")
        });
        let pointer = if mutable {
            let value = entry
                .state_value
                .as_deref_mut()
                .unwrap_or_else(|| panic!("State slot {:?} has been cleared", state.slot))
                .downcast_mut::<T>()
                .unwrap_or_else(|| panic!("State slot {:?} downcast failed", state.slot));
            entry.active_write = true;
            NonNull::from(value)
        } else {
            let value = entry
                .state_value
                .as_deref()
                .unwrap_or_else(|| panic!("State slot {:?} has been cleared", state.slot))
                .downcast_ref::<T>()
                .unwrap_or_else(|| panic!("State slot {:?} downcast failed", state.slot));
            entry.active_reads = next_readers.expect("shared borrow count was computed");
            NonNull::from(value)
        };
        pointer
    });
    (
        pointer,
        StateBorrowGuard {
            session: state.session,
            slot: state.slot,
            generation: state.generation,
            mutable,
            active: true,
        },
    )
}

fn release_state_borrow(session: u128, slot: SlotHandle, generation: u64, mutable: bool) -> bool {
    if crate::session::current_session_identity() != Some(session) {
        return false;
    }
    let (alive, removed) = with_slot_table_mut(|table| {
        let Some(entry) = table.entries.get_mut(slot) else {
            return (false, None);
        };
        assert_eq!(
            entry.generation, generation,
            "state borrow generation changed"
        );
        if mutable {
            assert!(entry.active_write, "state write borrow was not active");
            entry.active_write = false;
        } else {
            assert!(entry.active_reads > 0, "state read borrow was not active");
            entry.active_reads -= 1;
        }
        let alive = entry.alive;
        let remove = !alive && entry.active_reads == 0 && !entry.active_write;
        let removed = remove.then(|| table.entries.remove(slot).expect("retired state slot"));
        (alive, removed)
    });
    drop(removed);
    alive
}

fn clone_slot_value<T: 'static>(value: &Rc<dyn Any>, slot: SlotHandle) -> Rc<T> {
    value
        .clone()
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("Slot {slot:?} downcast failed"))
}

fn resolve_state_key<T: 'static>(
    key: SlotKey,
    retained: bool,
    init: impl FnOnce() -> T,
) -> State<T> {
    let ordered = key.owned_owner.is_none();
    if let Some(state) = with_slot_order(&key, |table, cursor| {
        let slot = if ordered {
            table.locate_slot(&key, cursor)?
        } else {
            table.find_slot(&key)?
        };
        let epoch = table.epoch;
        let entry = table.entries.get_mut(slot)?;
        entry.last_alive_epoch = epoch;
        entry.retained |= retained;
        if key.owned_owner.is_none() {
            entry.state_value.as_ref()?;
        }
        Some(State::from_slot(slot, entry.generation))
    }) {
        return state;
    }
    let value = init();
    let (state, unused) = with_slot_order(&key, |table, cursor| {
        if let Some(slot) = table.find_slot(&key) {
            let epoch = table.epoch;
            let entry = &mut table.entries[slot];
            entry.last_alive_epoch = epoch;
            entry.retained |= retained;
            if entry.state_value.is_some() || key.owned_owner.is_some() {
                return (State::from_slot(slot, entry.generation), Some(value));
            }
            entry.state_value = Some(Box::new(value));
            entry.generation = entry
                .generation
                .checked_add(1)
                .expect("state generation exhausted");
            return (State::from_slot(slot, entry.generation), None);
        }
        let slot = table.entries.insert(SlotEntry {
            key,
            generation: 1,
            value: None,
            state_value: Some(Box::new(value)),
            active_reads: 0,
            active_write: false,
            alive: true,
            last_alive_epoch: table.epoch,
            retained,
        });
        table.index_slot(slot);
        if ordered {
            table.record_new_slot(&key, slot, cursor);
        }
        (State::from_slot(slot, 1), None)
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

pub(crate) fn modifier_state<T: 'static>(
    instance: u64,
    path: &[u64],
    ordinal: usize,
    init: impl FnOnce() -> T,
) -> State<T> {
    let key = SlotKey {
        kind: SlotKind::State,
        owned_owner: None,
        instance_logic_id: instance,
        slot_hash: hash_components(&[&("modifier-state", path, ordinal)]),
        type_id: TypeId::of::<T>(),
    };
    resolve_state_key(key, false, init)
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
    T: 'static,
{
    ensure_build_phase();
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();
    let key_hash = hash_components(&[&key]);
    let positional = key_hash == hash_components(&[&()]);
    // Positional state is addressed by the dedicated remember counter, not by
    // the number of preceding state slots. Conditional state (for example the
    // task slot inside `launch`) must not shift later `remember` calls onto
    // another value.
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
        kind: SlotKind::State,
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
    ensure_build_phase_at(current_phase());
}

pub(crate) fn ensure_build_phase_at(phase: Option<RuntimePhase>) {
    match phase {
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

fn remember_functor_cell<T, F>(init: F) -> (Rc<T>, FunctorHandle)
where
    T: 'static,
    F: FnOnce() -> T,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_functor_slot_key();
    let slot_key = SlotKey {
        kind: SlotKind::Functor,
        owned_owner: None,
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    let mut init = Some(init);
    let (value, slot, generation) = with_slot_order(&slot_key, |table, mut cursor| {
        let epoch = table.epoch;
        if let Some(slot) = table.locate_slot(&slot_key, cursor.as_deref_mut()) {
            let (value, generation) = table.entries[slot].reuse_functor(slot, epoch, &mut init);
            return (value, slot, generation);
        }

        let value = Rc::new(init.take().expect("functor slot initialized once")());
        let generation = 1;
        let slot = table.entries.insert(SlotEntry {
            key: slot_key,
            generation,
            value: Some(value.clone()),
            state_value: None,
            active_reads: 0,
            active_write: false,
            alive: true,
            last_alive_epoch: epoch,
            retained: false,
        });
        table.index_slot(slot);
        table.record_new_slot(&slot_key, slot, cursor);
        (value, slot, generation)
    });
    (value, FunctorHandle::new(slot, generation))
}

fn load_functor_cell<T: 'static>(handle: FunctorHandle) -> Rc<T> {
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

        let value = entry
            .value
            .as_ref()
            .unwrap_or_else(|| panic!("Callback slot {:?} has been cleared", handle.slot));
        clone_slot_value::<T>(value, handle.slot)
    })
}

pub(crate) fn remember_callback_handle<F>(handler: F) -> FunctorHandle
where
    F: Fn() + 'static,
{
    let handler = Rc::new(handler) as Rc<dyn Fn()>;
    let (cell, handle) = remember_functor_cell({
        let handler = Rc::clone(&handler);
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
    F: Fn() + 'static,
{
    let static_type = (std::mem::size_of::<F>() == 0).then(TypeId::of::<F>);
    let render = Rc::new(render) as Rc<dyn Fn()>;
    let creator_instance_key = current_replay_boundary_instance_key_from_scope()
        .unwrap_or_else(|| panic!("RenderSlot handles must be created during a component build"));
    let (cell, handle) = remember_functor_cell({
        let render = Rc::clone(&render);
        move || RenderSlotCell::new(render, static_type)
    });
    if !cell.update(render, static_type) {
        return handle;
    }
    invalidate_render_slot_readers(handle, creator_instance_key);
    handle
}

pub(crate) fn invoke_render_slot_handle(handle: FunctorHandle) {
    let render = load_functor_cell::<RenderSlotCell>(handle).shared();
    render();
}

pub(crate) fn remember_render_slot_with_handle<T, F>(render: F) -> FunctorHandle
where
    T: 'static,
    F: Fn(T) + 'static,
{
    let static_type = (std::mem::size_of::<F>() == 0).then(TypeId::of::<F>);
    let render = Rc::new(render) as Rc<dyn Fn(T)>;
    let creator_instance_key =
        current_replay_boundary_instance_key_from_scope().unwrap_or_else(|| {
            panic!("RenderSlotWith handles must be created during a component build")
        });
    let (cell, handle) = remember_functor_cell({
        let render = Rc::clone(&render);
        move || RenderSlotWithCell::new(render, static_type)
    });
    if !cell.update(render, static_type) {
        return handle;
    }
    invalidate_render_slot_readers(handle, creator_instance_key);
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
    F: Fn(T) -> R + 'static,
{
    let handler = Rc::new(handler) as Rc<dyn Fn(T) -> R>;
    let (cell, handle) = remember_functor_cell({
        let handler = Rc::clone(&handler);
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
    reset_slots();
    release_retired_replay_snapshots();
    clear_redraw_waker();
    let tree = TesseraRuntime::with_mut(|runtime| std::mem::take(&mut runtime.component_tree));
    drop(tree);
    crate::modifier::flush_lifecycle();
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
    T: 'static,
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
    T: 'static,
{
    remember_with_key((), init)
}

/// Initializes keyed state during a build and keeps it until the component unmounts.
///
/// Unused retained slots survive conditional branches; see [`State`] for access
/// rules and [`Model`] for state scoped to a component, parent, or route.
pub fn retain_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: 'static,
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
    T: 'static,
{
    retain_with_key((), init)
}

/// Runs `future` on this UI session while this component call keeps the same
/// `key`.
///
/// The future may capture UI-thread-local values. It starts after the current
/// build; a different `key` cancels it and starts the new future, and leaving
/// the composition cancels it.
pub fn launch<K, F>(key: K, future: F)
where
    K: PartialEq + 'static,
    F: std::future::Future<Output = ()> + 'static,
{
    let running = remember(|| None::<(K, crate::Task)>);
    if running
        .with_untracked(|running| running.as_ref().is_some_and(|(current, _)| *current == key))
    {
        return;
    }
    let task = crate::async_dispatch::spawn_local(future);
    let previous = running.with_mut(|running| running.replace((key, task)));
    drop(previous);
}

/// Runs `future` once for this component call, cancelling it when the
/// component leaves the composition.
pub fn launch_once<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    launch((), future);
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
    use std::{
        cell::Cell,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::component_tree::{ComponentNode, NodeRole};
    use crate::execution_context::{
        reset_execution_context, with_execution_context, with_execution_context_mut,
    };
    use crate::layout::MeasureScope;
    use crate::modifier::{LayoutModifierChild, LayoutModifierInput, LayoutPolicy};
    use crate::prop::{CallbackWith, RenderSlot, RenderSlotWith};
    use crate::tessera;

    #[test]
    fn instance_slot_gap_preserves_keyed_reorders_and_truncates_unused_tail() {
        let mut allocated = SlotMap::<SlotHandle, ()>::with_key();
        let first = allocated.insert(());
        let second = allocated.insert(());
        let third = allocated.insert(());
        let mut cursor = InstanceSlotCursor::default();

        cursor.begin_epoch(1);
        cursor.record_slow_match(first);
        cursor.record_slow_match(second);
        cursor.record_slow_match(third);
        cursor.finish_epoch();
        assert_eq!(
            cursor.order.iter().copied().collect::<Vec<_>>(),
            [first, second, third]
        );

        cursor.begin_epoch(2);
        assert_eq!(cursor.fast_candidate(), Some(first));
        cursor.record_fast_match(first);
        cursor.record_slow_match(third);
        assert_eq!(cursor.fast_candidate(), Some(second));
        cursor.record_fast_match(second);
        cursor.finish_epoch();
        assert_eq!(
            cursor.order.iter().copied().collect::<Vec<_>>(),
            [first, third, second]
        );

        cursor.begin_epoch(3);
        cursor.record_slow_match(third);
        cursor.record_slow_match(first);
        cursor.finish_epoch();
        assert_eq!(
            cursor.order.iter().copied().collect::<Vec<_>>(),
            [third, first]
        );
    }

    #[derive(Clone, Default, PartialEq)]
    struct ConditionalLoopProbe {
        values: crate::testing::TestProbe<[Option<State<usize>>; 3]>,
        order: crate::testing::TestProbe<Option<State<Vec<usize>>>>,
        tick: crate::testing::TestProbe<Option<State<usize>>>,
    }

    #[derive(Clone, PartialEq)]
    struct ConditionalLoopWidth(State<usize>);

    impl LayoutPolicy for ConditionalLoopWidth {
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

    #[derive(Clone, PartialEq)]
    struct LocalOnly(Rc<Cell<usize>>);

    impl LayoutPolicy for ConditionalLoopRow {
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

    #[tessera]
    fn conditional_loop_item(index: usize, probe: ConditionalLoopProbe) {
        let value = remember(|| index);
        let mut values = probe.values.get();
        values[index] = Some(value);
        probe.values.set(values);
        crate::layout::layout(ConditionalLoopWidth(value))
            .modifier(crate::Modifier::new().push_semantics(ConditionalLoopTag(index)));
    }

    #[tessera]
    fn conditional_loop_fixture(keyed: bool, probe: ConditionalLoopProbe) {
        let order = remember(|| vec![0, 1, 2]);
        let tick = remember(|| 0);
        probe.order.set(Some(order));
        probe.tick.set(Some(tick));
        crate::layout::layout(ConditionalLoopRow).child(move || {
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
            let mut session = crate::testing::UiTest::new()
                .content(|| conditional_loop_fixture(keyed, probe.clone()))
                .viewport(100, 20);
            let initial = session.frame();
            let values = probe.values.get();
            let first = values[1].expect("test fixture value is present");
            let second = values[2].expect("test fixture value is present");
            assert_eq!(first.get(), 1);
            assert_eq!(second.get(), 2);
            initial.node("loop-item-1").rect(0, 0, 1, 10);
            initial.node("loop-item-2").rect(1, 0, 2, 10);

            first.set(11);
            let changed = session.frames(1);
            changed.node("loop-item-1").rect(0, 0, 11, 10);
            changed.node("loop-item-2").rect(11, 0, 2, 10);
            assert_eq!(second.get(), 2);
            probe.tick.get().expect("test fixture entry exists").set(1);
            let replayed = session.frames(1);
            replayed.node("loop-item-1").rect(0, 0, 11, 10);
            replayed.node("loop-item-2").rect(11, 0, 2, 10);

            if keyed {
                probe
                    .order
                    .get()
                    .expect("test fixture entry exists")
                    .set(vec![0, 2, 1]);
                let reordered = session.frames(1);
                reordered.node("loop-item-2").rect(0, 0, 2, 10);
                reordered.node("loop-item-1").rect(2, 0, 11, 10);
                assert!(probe.values.get()[1].expect("test fixture entry exists") == first);
                assert!(probe.values.get()[2].expect("test fixture entry exists") == second);

                probe
                    .order
                    .get()
                    .expect("test fixture entry exists")
                    .set(vec![0, 2]);
                let removed = session.frames(1);
                removed.node("loop-item-2").rect(0, 0, 2, 10);
                assert!(!first.is_alive());
                assert!(second.is_alive());
            }
        }
    }

    #[test]
    fn batched_state_writes_invalidate_each_changed_component_reader() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let probe = ConditionalLoopProbe::default();
        let mut session = crate::testing::UiTest::new()
            .content(|| conditional_loop_fixture(false, probe.clone()))
            .viewport(100, 20);
        session.frame();
        let first = probe.values.get()[1].expect("test fixture entry exists");
        let second = probe.values.get()[2].expect("test fixture entry exists");

        with_batched_state_writes(|| {
            first.set(11);
            second.set(22);
        });

        let changed = session.frames(1);
        changed.node("loop-item-1").rect(0, 0, 11, 10);
        changed.node("loop-item-2").rect(11, 0, 22, 10);
    }

    #[test]
    fn set_many_validates_the_whole_batch_before_mutating_any_state() {
        let first_session = crate::RuntimeSession::new();
        let first = first_session.run(|| with_test_component_scope(11020, || remember(|| 4usize)));
        let other_session = crate::RuntimeSession::new();
        let foreign =
            other_session.run(|| with_test_component_scope(11021, || remember(|| 8usize)));

        first_session.run(|| {
            assert!(
                std::panic::catch_unwind(|| State::set_many([(first, 40), (foreign, 80)])).is_err()
            );
            assert_eq!(first.get(), 4);
        });
    }

    #[tessera]
    fn required_value_component(value: Option<usize>, on_value: Option<CallbackWith<usize>>) {
        let value = value.expect("missing required prop `value` in `required_value_component`");
        if let Some(on_value) = on_value {
            on_value.call(value);
        }
    }

    #[tessera]
    fn optional_value_component(
        value: Option<usize>,
        on_value: Option<CallbackWith<Option<usize>>>,
    ) {
        if let Some(on_value) = on_value {
            on_value.call(value);
        }
    }

    #[tessera]
    fn local_value_component(value: LocalOnly) {
        value.0.set(value.0.get() + 1);
    }

    #[tessera]
    fn local_slot_component(children: Option<RenderSlot>) {
        children.unwrap_or_else(RenderSlot::empty).render();
    }

    #[derive(Clone)]
    struct RuntimeOnDrop {
        generation: u32,
        dropped: crate::testing::TestProbe<u32>,
    }

    impl PartialEq for RuntimeOnDrop {
        fn eq(&self, other: &Self) -> bool {
            self.generation == other.generation
        }
    }

    impl Drop for RuntimeOnDrop {
        fn drop(&mut self) {
            // Props may own values whose destructors read runtime state.
            TesseraRuntime::with(|runtime| runtime.component_tree.has_call(0));
            self.dropped.with_mut(|count| *count += 1);
        }
    }

    #[tessera]
    fn runtime_on_drop_holder(props: RuntimeOnDrop) {
        let _ = props.generation;
        crate::layout::layout(crate::layout::Stack);
    }

    #[tessera]
    fn runtime_on_drop_parent(
        generation: crate::testing::TestProbe<Option<State<u32>>>,
        dropped: crate::testing::TestProbe<u32>,
    ) {
        let state = remember(|| 0_u32);
        generation.set(Some(state));
        runtime_on_drop_holder(RuntimeOnDrop {
            generation: state.get(),
            dropped: dropped.clone(),
        });
    }

    #[test]
    fn replaced_props_drop_after_runtime_borrows_end() {
        use crate::testing::UiTest;

        let generation = crate::testing::TestProbe::default();
        let dropped = crate::testing::TestProbe::new(0_u32);
        let mut session =
            UiTest::new().content(|| runtime_on_drop_parent(generation.clone(), dropped.clone()));
        session.frame();
        let state = generation.get().expect("parent published its state");
        for value in 1..=3 {
            session.run(|| state.set(value));
            session.frames(1);
        }
        let replaced = dropped.get();
        assert!(replaced > 0, "replaced props must be released");
    }
    fn with_test_component_scope<R>(component_type_id: u64, f: impl FnOnce() -> R) -> R {
        reset_execution_context();
        let node_id = TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.clear();
            runtime.component_tree.begin_build_root();
            runtime
                .component_tree
                .current_build_node_id()
                .expect("build root")
        });
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
    fn state_mutations_keep_the_value_allocation_stable() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let state = with_test_component_scope(823, || remember(|| 4_u32));
        let address = || {
            with_slot_table(|table| {
                table.entries[state.slot]
                    .state_value
                    .as_deref()
                    .and_then(|value| value.downcast_ref::<u32>())
                    .expect("remembered value") as *const u32
            })
        };
        let initial = address();

        state.with_mut(|value| *value += 1);
        assert_eq!(address(), initial);
        assert!(!state.set_if_changed(5));
        assert_eq!(address(), initial);
        assert!(state.set_if_changed(6));
        assert_eq!(address(), initial);
        state.set(7);
        assert_eq!(address(), initial);
        assert_eq!(state.get(), 7);
    }

    #[test]
    fn active_state_read_defers_destruction_until_reentrant_removal_finishes() {
        struct DropProbe(Rc<Cell<usize>>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                TesseraRuntime::with_mut(|_| {});
                self.0.set(self.0.get() + 1);
            }
        }

        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let drops = Rc::new(Cell::new(0));
        let state = with_test_component_scope(824, || {
            let drops = drops.clone();
            remember(move || DropProbe(drops))
        });

        state.with(|value| {
            reset_slots();
            assert!(!state.is_alive());
            assert_eq!(drops.get(), 0, "an active read keeps the value alive");
            let _ = &value.0;
        });
        assert_eq!(drops.get(), 1, "the last access releases the retired value");
    }

    #[test]
    fn active_state_write_defers_destruction_until_reentrant_removal_finishes() {
        struct DropProbe(Rc<Cell<usize>>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                TesseraRuntime::with_mut(|_| {});
                self.0.set(self.0.get() + 1);
            }
        }

        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let drops = Rc::new(Cell::new(0));
        let state = with_test_component_scope(830, || {
            let drops = drops.clone();
            remember(move || DropProbe(drops))
        });

        state.with_mut(|value| {
            reset_slots();
            assert!(!state.is_alive());
            assert_eq!(drops.get(), 0, "an active write keeps the value alive");
            let _ = &value.0;
        });
        assert_eq!(drops.get(), 1, "the last access releases the retired value");
    }

    #[test]
    fn state_borrow_guards_release_after_unwinding() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let state = with_test_component_scope(825, || remember(|| 1_u32));

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.with_mut(|value| {
                *value = 9;
                panic!("test write unwind");
            });
        }));
        assert!(panic.is_err());
        assert_eq!(state.get(), 9);
        state.with_mut(|value| *value += 1);
        assert_eq!(state.get(), 10);
    }

    fn flush_until(mut done: impl FnMut() -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !done() {
            assert!(
                std::time::Instant::now() < deadline,
                "work did not complete"
            );
            crate::dispatcher::flush();
            std::thread::yield_now();
        }
    }

    #[test]
    fn off_thread_writes_wait_for_a_frame_boundary() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let state = with_test_component_scope(826, || remember(|| 1_u32));
            let worker = std::thread::spawn(move || {
                state.with_mut(|value| *value += 1);
                state.get()
            });
            assert_eq!(state.get(), 1, "the owner holds the gate during its work");
            flush_until(|| worker.is_finished());
            assert_eq!(worker.join().expect("worker"), 2);
            assert_eq!(state.get(), 2);
        });
    }

    #[test]
    fn off_thread_access_reports_expired_state() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let state = with_test_component_scope(828, || remember(|| 4usize));
            reset_slots();
            let worker = std::thread::spawn(move || state.is_alive());
            flush_until(|| worker.is_finished());
            assert!(!worker.join().expect("worker"));
        });
    }

    #[test]
    fn io_task_hops_back_to_main_and_updates_state() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let state = with_test_component_scope(831, || remember(|| 0usize));
            crate::spawn(crate::Dispatcher::Ui, async move {
                let value = crate::on(crate::Dispatcher::Io, async { 6 + 1 }).await;
                state.set(value);
            })
            .detach();
            flush_until(|| state.get() == 7);
        });
    }

    #[test]
    fn ui_tasks_can_await_tokio_timers() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let state = with_test_component_scope(832, || remember(|| 0usize));
            crate::spawn(crate::Dispatcher::Ui, async move {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                state.set(1);
            })
            .detach();
            flush_until(|| state.get() == 1);
        });
    }

    #[test]
    fn dropping_a_task_cancels_it() {
        let session = crate::RuntimeSession::new();
        session.run(|| {
            let state = with_test_component_scope(829, || remember(|| 0usize));
            let task = crate::spawn(crate::Dispatcher::Ui, async move { state.set(1) });
            assert!(!task.is_finished());
            drop(task);
            crate::dispatcher::flush();
            crate::dispatcher::flush();
            assert_eq!(state.get(), 0);
        });
    }

    struct CountDrop(std::sync::Arc<AtomicUsize>);

    impl Drop for CountDrop {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tessera]
    fn launch_fixture(
        key_probe: crate::testing::TestProbe<Option<State<u32>>>,
        started: std::sync::Arc<AtomicUsize>,
        dropped: std::sync::Arc<AtomicUsize>,
    ) {
        let key = remember(|| 0u32);
        key_probe.set(Some(key));
        let local = Rc::new(());
        launch(key.get(), async move {
            let _local = local;
            let _guard = CountDrop(dropped);
            started.fetch_add(1, Ordering::SeqCst);
            std::future::pending::<()>().await;
        });
        crate::layout::layout(crate::layout::Stack);
    }

    #[test]
    fn launch_runs_local_futures_and_restarts_them_when_the_key_changes() {
        let key_probe = crate::testing::TestProbe::default();
        let started = std::sync::Arc::new(AtomicUsize::new(0));
        let dropped = std::sync::Arc::new(AtomicUsize::new(0));
        let mut session = crate::testing::UiTest::new()
            .content(|| launch_fixture(key_probe.clone(), started.clone(), dropped.clone()));
        session.frame();
        session.advance(Duration::from_millis(1));
        assert_eq!(started.load(Ordering::SeqCst), 1);
        session.advance(Duration::from_millis(1));
        assert_eq!(
            started.load(Ordering::SeqCst),
            1,
            "an unchanged key keeps the task"
        );

        key_probe.get().expect("key state").set(1);
        session.frames(3);
        assert_eq!(started.load(Ordering::SeqCst), 2);
        assert_eq!(
            dropped.load(Ordering::SeqCst),
            1,
            "the previous task was cancelled"
        );
    }

    #[tessera]
    fn async_update_dependency_fixture(
        state_probe: crate::testing::TestProbe<Option<State<usize>>>,
        builds: crate::testing::TestProbe<Vec<usize>>,
    ) {
        let state = remember(|| 0usize);
        state_probe.set(Some(state));
        builds.with_mut(|values| values.push(state.get()));
        crate::layout::layout(crate::layout::Stack);
    }

    #[test]
    fn off_thread_state_update_publishes_to_component_build_dependencies() {
        let state_probe = crate::testing::TestProbe::default();
        let builds = crate::testing::TestProbe::new(Vec::new());
        let mut session = crate::testing::UiTest::new()
            .content(|| async_update_dependency_fixture(state_probe.clone(), builds.clone()));
        session.frame();
        assert_eq!(builds.get(), [0]);
        let state = state_probe.get().expect("component remembered its state");

        let worker = std::thread::spawn(move || state.set(9));
        while !worker.is_finished() {
            session.advance(Duration::from_millis(1));
        }
        worker.join().expect("update worker");
        session.frames(1);
        assert_eq!(builds.get(), [0, 9]);
    }

    struct LifetimeModel {
        value: State<usize>,
    }

    impl crate::ModelInit for LifetimeModel {
        fn init() -> Self {
            Self {
                value: State::new(3),
            }
        }
    }

    #[tessera]
    fn injected_model_reader(
        model: Model<LifetimeModel>,
        model_probe: crate::testing::TestProbe<Option<Model<LifetimeModel>>>,
        state_probe: crate::testing::TestProbe<Option<State<usize>>>,
    ) {
        model_probe.set(Some(model));
        state_probe.set(Some(model.with(|model| model.value)));
        crate::layout::layout(crate::layout::Stack);
    }

    #[tessera]
    fn component_model_lifetime_fixture(
        visible_probe: crate::testing::TestProbe<Option<State<bool>>>,
        model_probe: crate::testing::TestProbe<Option<Model<LifetimeModel>>>,
        state_probe: crate::testing::TestProbe<Option<State<usize>>>,
    ) {
        let visible = remember(|| true);
        visible_probe.set(Some(visible));
        if visible.get() {
            injected_model_reader(model_probe, state_probe);
        }
        crate::layout::layout(crate::layout::Stack);
    }

    #[test]
    fn remembered_model_closes_and_releases_its_state_when_component_unmounts() {
        let visible_probe = crate::testing::TestProbe::default();
        let model_probe = crate::testing::TestProbe::default();
        let state_probe = crate::testing::TestProbe::default();
        let mut session = crate::testing::UiTest::new().content(|| {
            component_model_lifetime_fixture(
                visible_probe.clone(),
                model_probe.clone(),
                state_probe.clone(),
            )
        });
        session.frame();
        let visible = visible_probe.get().expect("component state");
        let model = model_probe.get().expect("component model");
        let state = state_probe.get().expect("model state");
        assert!(model.is_alive());
        assert!(state.is_alive());

        session.run(|| visible.set(false));
        session.advance(Duration::from_millis(16));
        assert!(!model.is_alive());
        assert!(!state.is_alive());
    }

    #[tessera]
    fn store_model_fixture(
        scope: crate::testing::TestProbe<Option<crate::ModelStoreScope>>,
        model_probe: crate::testing::TestProbe<Option<Model<LifetimeModel>>>,
        state_probe: crate::testing::TestProbe<Option<State<usize>>>,
    ) {
        let scope = scope.get().expect("test provides a store");
        crate::provide_context(scope, || injected_model_reader(model_probe, state_probe));
    }

    #[test]
    fn provided_store_owns_injected_models_until_it_is_dropped() {
        let scope_probe = crate::testing::TestProbe::default();
        let model_probe = crate::testing::TestProbe::default();
        let state_probe = crate::testing::TestProbe::default();
        let mut session = crate::testing::UiTest::new().content(|| {
            store_model_fixture(
                scope_probe.clone(),
                model_probe.clone(),
                state_probe.clone(),
            )
        });
        let store = session.run(crate::ModelStore::new);
        scope_probe.set(Some(store.scope()));
        session.frame();
        let state = state_probe.get().expect("injected model state");
        let stored = session.run(|| {
            store
                .model(<LifetimeModel as crate::ModelInit>::init)
                .with(|model| model.value)
        });
        assert!(state == stored);
        assert!(session.run(|| state.is_alive()));
        session.run(|| drop(store));
        assert!(!session.run(|| state.is_alive()));
        assert!(!session.run(|| model_probe.get().expect("injected model").is_alive()));
    }

    #[test]
    fn remembered_state_accepts_ui_thread_local_values() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let state = with_test_component_scope(817, || remember(|| Rc::new(Cell::new(4))));

        state.with(|value| value.set(9));
        assert_eq!(state.with(|value| value.get()), 9);
        state.set(Rc::new(Cell::new(12)));
        assert_eq!(state.with(|value| value.get()), 12);

        let store = crate::ModelStore::new();
        let owned = store
            .model(|| State::new(Rc::new(Cell::new(3))))
            .with(|state| *state);
        owned.with(|value| value.set(8));
        assert_eq!(owned.with(|value| value.get()), 8);
        drop(store);
        assert!(!owned.is_alive());
    }

    #[test]
    fn callbacks_and_render_slots_accept_ui_thread_local_captures() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        let value = Rc::new(Cell::new(0));

        let callback = with_test_component_scope(818, || {
            let value = Rc::clone(&value);
            crate::Callback::new(move || value.set(1))
        });
        callback.call();
        assert_eq!(value.get(), 1);

        let slot = with_test_component_scope(819, || {
            let value = Rc::clone(&value);
            RenderSlot::new(move || value.set(2))
        });
        with_test_component_scope(820, || slot.render());
        assert_eq!(value.get(), 2);

        let with_value = with_test_component_scope(821, || {
            let value = Rc::clone(&value);
            RenderSlotWith::new(move |next: Rc<Cell<usize>>| value.set(next.get()))
        });
        let next = Rc::new(Cell::new(7));
        with_test_component_scope(822, || with_value.render(Rc::clone(&next)));
        assert_eq!(value.get(), 7);
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

    impl crate::modifier::LayoutPolicy for DirtySplitPolicy {
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

    impl crate::modifier::LayoutModifier for DirtyMeasureModifierNode {
        fn measure(
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

    fn dirty_test_node(policy: DirtySplitPolicy, modifier: Modifier) -> ComponentNode {
        let mut node = ComponentNode::new("dirty_test", NodeRole::Layout);
        node.instance_key = 1;
        node.layout_policy_input = LayoutPolicyHandle::from(policy);
        node.modifier = modifier;
        node
    }

    fn record_rebuilt_node_dirty(previous: &ComponentNode, node: &ComponentNode) {
        let changes = node.modifier.changes_from(&previous.modifier);
        record_layout_node_dirty(Some((&previous.layout_policy_input, changes)), node);
    }

    #[test]
    fn layout_dirty_tracking_separates_measure_and_placement_changes() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_layout_dirty_tracking();

        let first = dirty_test_node(
            DirtySplitPolicy {
                measure_key: 0,
                placement_key: 0,
            },
            Modifier::new(),
        );
        begin_frame_layout_dirty_tracking();
        record_layout_node_dirty(None, &first);
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(dirty.measure_self_nodes.contains(&1));
        assert!(dirty.placement_self_nodes.is_empty());

        let second = dirty_test_node(
            DirtySplitPolicy {
                measure_key: 0,
                placement_key: 1,
            },
            Modifier::new(),
        );
        begin_frame_layout_dirty_tracking();
        record_rebuilt_node_dirty(&first, &second);
        finalize_frame_layout_dirty_tracking();
        let dirty = take_layout_dirty_nodes();
        assert!(!dirty.measure_self_nodes.contains(&1));
        assert!(dirty.placement_self_nodes.contains(&1));

        let third = dirty_test_node(
            DirtySplitPolicy {
                measure_key: 1,
                placement_key: 1,
            },
            Modifier::new(),
        );
        begin_frame_layout_dirty_tracking();
        record_rebuilt_node_dirty(&second, &third);
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

        let policy = DirtySplitPolicy {
            measure_key: 0,
            placement_key: 0,
        };
        let first = dirty_test_node(
            policy.clone(),
            Modifier::new().push_layout(DirtyMeasureModifierNode { width: 10 }),
        );
        begin_frame_layout_dirty_tracking();
        record_layout_node_dirty(None, &first);
        finalize_frame_layout_dirty_tracking();
        let _ = take_layout_dirty_nodes();

        let second = dirty_test_node(
            policy,
            Modifier::new().push_layout(DirtyMeasureModifierNode { width: 20 }),
        );
        begin_frame_layout_dirty_tracking();
        record_rebuilt_node_dirty(&first, &second);
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

        with_replay_scope(42, replay, Some(11), ContextMap::default(), || {
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
            with_replay_scope(77, replay, Some(17), ContextMap::default(), || {
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
    fn component_props_accept_ui_thread_local_values() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        let value = Rc::new(Cell::new(4));

        with_test_component_scope(11017, || local_value_component(LocalOnly(value.clone())));

        assert_eq!(value.get(), 5);

        let callback_value = Rc::new(Cell::new(0));
        with_test_component_scope(11018, || {
            let callback_value = Rc::clone(&callback_value);
            optional_value_component().on_value(move |value: Option<usize>| {
                callback_value.set(value.unwrap_or_default());
            });
        });
        assert_eq!(callback_value.get(), 0);

        let slot_value = Rc::new(Cell::new(0));
        with_test_component_scope(11019, || {
            let slot_value = Rc::clone(&slot_value);
            local_slot_component().children(move || slot_value.set(9));
        });
        assert_eq!(slot_value.get(), 9);
    }

    #[test]
    fn render_slot_update_invalidates_reader_instance() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        release_retired_replay_snapshots();
        reset_render_slot_read_dependencies();
        reset_build_invalidations();

        let wakeups = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&wakeups);
        install_redraw_waker(Arc::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
        }));

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

        assert!(!has_pending_build_invalidations());

        begin_recompose_slot_epoch();
        let second = with_test_component_scope(11002, || RenderSlot::new(|| {}));

        assert!(first == second);
        assert!(has_pending_build_invalidations());

        let invalidations = take_build_invalidations();
        let mut expected = HashSet::default();
        expected.insert(reader_instance_key);
        assert_eq!(invalidations.dirty_instance_keys, expected);
        assert_eq!(wakeups.load(Ordering::Relaxed), 0);
        defer_build_invalidations(&invalidations.dirty_instance_keys);
        assert_eq!(wakeups.load(Ordering::Relaxed), 1);
        clear_redraw_waker();
    }

    #[test]
    fn render_slot_update_skips_reader_already_dirty_in_current_build() {
        let _runtime_session = crate::RuntimeSession::new();
        let _runtime_scope = _runtime_session.enter();
        reset_slots();
        release_retired_replay_snapshots();
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

        assert!(!has_pending_build_invalidations());

        begin_recompose_slot_epoch();
        let mut dirty = HashSet::default();
        dirty.insert(reader_instance_key);
        with_build_dirty_instance_keys(Arc::new(dirty), || {
            let mut arena = indextree::Arena::<()>::new();
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
            let mut arena = indextree::Arena::<()>::new();
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
        let mut arena = indextree::Arena::<()>::new();
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
        let mut arena = indextree::Arena::<()>::new();
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
                .add_node(ComponentNode::new("root", NodeRole::Group))
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
            let _ = remember(|| 0u64);
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
            kind: SlotKind::State,
            owned_owner: None,
            instance_logic_id: 7,
            slot_hash: 11,
            type_id: TypeId::of::<i32>(),
        };
        let drop_key = SlotKey {
            kind: SlotKind::State,
            owned_owner: None,
            instance_logic_id: 7,
            slot_hash: 12,
            type_id: TypeId::of::<i32>(),
        };

        let keep_slot = table.entries.insert(SlotEntry {
            key: keep_key,
            generation: 1,
            value: None,
            state_value: Some(Box::new(10_i32)),
            active_reads: 0,
            active_write: false,
            alive: true,
            last_alive_epoch: 0,
            retained: true,
        });
        let drop_slot = table.entries.insert(SlotEntry {
            key: drop_key,
            generation: 1,
            value: None,
            state_value: Some(Box::new(20_i32)),
            active_reads: 0,
            active_write: false,
            alive: true,
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
