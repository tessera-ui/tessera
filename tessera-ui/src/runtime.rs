//! This module provides the global runtime state management for tessera.

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

use parking_lot::RwLock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::{
    NodeId,
    accessibility::{AccessibilityActionHandler, AccessibilityNode},
    component_tree::ComponentTree,
    execution_context::{OrderFrame, with_execution_context, with_execution_context_mut},
    focus::{
        FocusDirection, FocusGroupNode, FocusHandleId, FocusNode, FocusProperties,
        FocusRegistration, FocusRegistrationKind, FocusRequester, FocusRequesterId,
        FocusRevealRequest, FocusScopeNode, FocusState, FocusTraversalPolicy,
    },
    layout::LayoutModifierDyn,
    modifier::Modifier,
    prop::{CallbackWith, ComponentReplayData, ErasedComponentRunner, Prop},
    time::Instant,
};

#[derive(Clone, Copy)]
enum OrderCounterKind {
    Remember,
    Functor,
    Context,
    FrameReceiver,
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
    with_execution_context_mut(|context| {
        debug_assert!(!context.order_frame_stack.is_empty(), "{empty_message}");
        let frame = context.order_frame_stack.last_mut().expect(empty_message);
        match kind {
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
            OrderCounterKind::FrameReceiver => {
                let counter = frame.frame_receiver;
                frame.frame_receiver = frame.frame_receiver.wrapping_add(1);
                counter
            }
        }
    })
}

fn next_child_instance_call_index() -> u64 {
    with_execution_context_mut(|context| {
        let Some(frame) = context.order_frame_stack.last_mut() else {
            return 0;
        };
        let index = frame.instance;
        frame.instance = frame.instance.wrapping_add(1);
        index
    })
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
    instance_logic_id: u64,
    slot_hash: u64,
    type_id: TypeId,
}

impl Default for SlotKey {
    fn default() -> Self {
        Self {
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
    value: Option<Box<dyn Any + Send + Sync>>,
    last_alive_epoch: u64,
    retained: bool,
}

#[derive(Default)]
struct InstanceSlotCursor {
    previous_order: SmallVec<[SlotHandle; 4]>,
    current_order: SmallVec<[SlotHandle; 4]>,
    cursor: usize,
    epoch: u64,
}

impl InstanceSlotCursor {
    fn begin_epoch(&mut self, epoch: u64) {
        if self.epoch == epoch {
            return;
        }
        self.previous_order = std::mem::take(&mut self.current_order);
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
    entries: SlotMap<SlotHandle, SlotEntry>,
    key_to_slot: HashMap<SlotKey, SlotHandle>,
    cursors_by_instance_logic_id: HashMap<u64, InstanceSlotCursor>,
    epoch: u64,
}
impl SlotTable {
    fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }

    fn reset(&mut self) -> Vec<SlotEntry> {
        let removed = self.entries.drain().map(|(_, entry)| entry).collect();
        self.key_to_slot.clear();
        self.cursors_by_instance_logic_id.clear();
        self.epoch = 0;
        removed
    }
    fn try_fast_slot_lookup(&mut self, key: SlotKey) -> Option<SlotHandle> {
        let epoch = self.epoch;
        let candidate = {
            let cursor = self
                .cursors_by_instance_logic_id
                .entry(key.instance_logic_id)
                .or_default();
            cursor.begin_epoch(epoch);
            cursor.fast_candidate()
        }?;

        let is_match = self
            .entries
            .get(candidate)
            .is_some_and(|entry| entry.key == key);

        if !is_match {
            return None;
        }

        let cursor = self
            .cursors_by_instance_logic_id
            .get_mut(&key.instance_logic_id)
            .expect("cursor entry should exist");
        cursor.record_fast_match(candidate);
        Some(candidate)
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
    fn retain_instance_keys(
        &mut self,
        live_instance_keys: &HashSet<u64>,
    ) -> RemovedPersistentFocusHandles {
        let mut removed = RemovedPersistentFocusHandles::default();
        self.targets.retain(|key, handle| {
            if !live_instance_keys.contains(&key.instance_key) {
                if handle.retain_for_frame() {
                    true
                } else {
                    removed.handle_ids.insert(handle.value.handle_id());
                    false
                }
            } else {
                handle.mark_live();
                true
            }
        });
        self.scopes.retain(|key, scope| {
            if !live_instance_keys.contains(&key.instance_key) {
                if scope.retain_for_frame() {
                    true
                } else {
                    removed.handle_ids.insert(scope.value.handle_id());
                    false
                }
            } else {
                scope.mark_live();
                true
            }
        });
        self.groups.retain(|key, group| {
            if !live_instance_keys.contains(&key.instance_key) {
                if group.retain_for_frame() {
                    true
                } else {
                    removed.handle_ids.insert(group.value.handle_id());
                    false
                }
            } else {
                group.mark_live();
                true
            }
        });
        self.requesters.retain(|key, requester| {
            if !live_instance_keys.contains(&key.instance_key) {
                if requester.retain_for_frame() {
                    true
                } else {
                    removed.requester_ids.insert(requester.value.requester_id());
                    false
                }
            } else {
                requester.mark_live();
                true
            }
        });
        removed
    }

    fn contains_handle(&self, handle_id: FocusHandleId) -> bool {
        self.targets
            .values()
            .any(|entry| entry.value.handle_id() == handle_id)
            || self
                .scopes
                .values()
                .any(|entry| entry.value.handle_id() == handle_id)
            || self
                .groups
                .values()
                .any(|entry| entry.value.handle_id() == handle_id)
    }

    fn clear(&mut self) {
        self.targets.clear();
        self.scopes.clear();
        self.groups.clear();
        self.requesters.clear();
    }
}

fn with_slot_table<R>(f: impl FnOnce(&SlotTable) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.slot_table.borrow()))
}

fn with_slot_table_mut<R>(f: impl FnOnce(&mut SlotTable) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.slot_table.borrow_mut()))
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctorHandle {
    slot: SlotHandle,
    generation: u64,
}

impl FunctorHandle {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self { slot, generation }
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
        *self.current.write() = next;
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
        *self.current.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn(T) -> R + Send + Sync> {
        Arc::clone(&self.current.read())
    }
}

struct RenderSlotCell {
    current: RwLock<Arc<dyn Fn() + Send + Sync>>,
}

impl RenderSlotCell {
    fn new(current: Arc<dyn Fn() + Send + Sync>) -> Self {
        Self {
            current: RwLock::new(current),
        }
    }

    fn update(&self, next: Arc<dyn Fn() + Send + Sync>) {
        *self.current.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn() + Send + Sync> {
        Arc::clone(&self.current.read())
    }
}

struct RenderSlotWithCell<T> {
    current: RwLock<Arc<dyn Fn(T) + Send + Sync>>,
}

impl<T> RenderSlotWithCell<T> {
    fn new(current: Arc<dyn Fn(T) + Send + Sync>) -> Self {
        Self {
            current: RwLock::new(current),
        }
    }

    fn update(&self, next: Arc<dyn Fn(T) + Send + Sync>) {
        *self.current.write() = next;
    }

    fn shared(&self) -> Arc<dyn Fn(T) + Send + Sync> {
        Arc::clone(&self.current.read())
    }
}

#[derive(Default)]
struct LayoutDirtyTracker {
    previous_layout_inputs_by_node: HashMap<u64, LayoutInputSnapshot>,
    frame_layout_inputs_by_node: HashMap<u64, LayoutInputSnapshot>,
    pending_measure_self_dirty_nodes: HashSet<u64>,
    ready_measure_self_dirty_nodes: HashSet<u64>,
    pending_placement_self_dirty_nodes: HashSet<u64>,
    ready_placement_self_dirty_nodes: HashSet<u64>,
    previous_children_by_node: HashMap<u64, Vec<u64>>,
    structural_changes: StructureReconcileResult,
    pending_parent_data_dirty_nodes: HashSet<u64>,
    pending_transform_dirty_nodes: HashSet<u64>,
    pending_paint_dirty_nodes: HashSet<u64>,
    layout_readers_by_state: HashMap<StateReadDependencyKey, HashSet<PhaseReader>>,
    layout_states_by_reader: HashMap<PhaseReader, HashSet<StateReadDependencyKey>>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum ReadPhase {
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
    policy: Box<dyn LayoutModifierDyn>,
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
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct ReplayNodeSnapshot {
    pub instance_key: u64,
    pub parent_instance_key: Option<u64>,
    pub instance_logic_id: u64,
    pub group_path: Vec<u64>,
    pub instance_key_override: Option<u64>,
    pub fn_name: String,
    pub replay: ComponentReplayData,
}

#[derive(Default)]
struct ComponentReplayTracker {
    previous_nodes: HashMap<u64, ReplayNodeSnapshot>,
    current_nodes: HashMap<u64, ReplayNodeSnapshot>,
}

fn with_component_replay_tracker<R>(f: impl FnOnce(&ComponentReplayTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.component_replay_tracker.borrow()))
}

fn with_component_replay_tracker_mut<R>(f: impl FnOnce(&mut ComponentReplayTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.component_replay_tracker.borrow_mut()))
}

pub(crate) fn begin_frame_component_replay_tracking() {
    with_component_replay_tracker_mut(|tracker| tracker.current_nodes.clear());
}

pub(crate) fn finalize_frame_component_replay_tracking() {
    with_component_replay_tracker_mut(|tracker| {
        tracker.previous_nodes = std::mem::take(&mut tracker.current_nodes);
    });
}

pub(crate) fn finalize_frame_component_replay_tracking_partial() {
    with_component_replay_tracker_mut(|tracker| {
        let current = std::mem::take(&mut tracker.current_nodes);
        tracker.previous_nodes.extend(current);
    });
}

pub(crate) fn reset_component_replay_tracking() {
    with_component_replay_tracker_mut(|tracker| {
        *tracker = ComponentReplayTracker::default();
    });
}

pub(crate) fn previous_component_replay_nodes() -> HashMap<u64, ReplayNodeSnapshot> {
    with_component_replay_tracker(|tracker| tracker.previous_nodes.clone())
}

pub(crate) fn remove_previous_component_replay_nodes(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_component_replay_tracker_mut(|tracker| {
        tracker
            .previous_nodes
            .retain(|instance_key, _| !instance_keys.contains(instance_key));
        tracker
            .current_nodes
            .retain(|instance_key, _| !instance_keys.contains(instance_key));
    });
}

#[derive(Default)]
struct BuildInvalidationTracker {
    dirty_instance_keys: HashSet<u64>,
    cross_instance_dirty_instance_keys: HashSet<u64>,
}

#[derive(Default)]
pub(crate) struct BuildInvalidationSet {
    pub dirty_instance_keys: HashSet<u64>,
    pub cross_instance_dirty_instance_keys: HashSet<u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateReadDependencyKey {
    // Keep generation in the dependency key to avoid ABA when a slot is recycled
    // and later reused for another state value.
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

#[derive(Default)]
struct StateReadDependencyTracker {
    readers_by_state: HashMap<StateReadDependencyKey, HashSet<u64>>,
    states_by_reader: HashMap<u64, HashSet<StateReadDependencyKey>>,
}

#[derive(Default)]
struct FocusReadDependencyTracker {
    readers_by_focus: HashMap<FocusReadDependencyKey, HashSet<u64>>,
    focus_by_reader: HashMap<u64, HashSet<FocusReadDependencyKey>>,
}

#[derive(Default)]
struct RenderSlotReadDependencyTracker {
    readers_by_slot: HashMap<FunctorHandle, HashSet<u64>>,
    slots_by_reader: HashMap<u64, HashSet<FunctorHandle>>,
}

type RedrawWaker = Arc<dyn Fn() + Send + Sync + 'static>;

fn with_build_invalidation_tracker<R>(f: impl FnOnce(&BuildInvalidationTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.build_invalidation_tracker.borrow()))
}

fn with_build_invalidation_tracker_mut<R>(f: impl FnOnce(&mut BuildInvalidationTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.build_invalidation_tracker.borrow_mut()))
}

fn with_state_read_dependency_tracker<R>(f: impl FnOnce(&StateReadDependencyTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.state_read_dependency_tracker.borrow()))
}

fn with_state_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut StateReadDependencyTracker) -> R,
) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.state_read_dependency_tracker.borrow_mut()))
}

fn with_focus_read_dependency_tracker<R>(f: impl FnOnce(&FocusReadDependencyTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.focus_read_dependency_tracker.borrow()))
}

fn with_focus_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut FocusReadDependencyTracker) -> R,
) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.focus_read_dependency_tracker.borrow_mut()))
}

fn with_render_slot_read_dependency_tracker<R>(
    f: impl FnOnce(&RenderSlotReadDependencyTracker) -> R,
) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.render_slot_read_dependency_tracker.borrow()))
}

fn with_render_slot_read_dependency_tracker_mut<R>(
    f: impl FnOnce(&mut RenderSlotReadDependencyTracker) -> R,
) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.render_slot_read_dependency_tracker.borrow_mut()))
}

fn with_redraw_waker<R>(f: impl FnOnce(&Option<RedrawWaker>) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.redraw_waker.borrow()))
}

fn with_redraw_waker_mut<R>(f: impl FnOnce(&mut Option<RedrawWaker>) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.redraw_waker.borrow_mut()))
}

pub(crate) fn schedule_runtime_redraw() {
    let callback = with_redraw_waker(Clone::clone);
    if let Some(callback) = callback {
        callback();
    }
}

pub(crate) fn install_redraw_waker(callback: RedrawWaker) {
    with_redraw_waker_mut(|waker| *waker = Some(callback));
}

pub(crate) fn clear_redraw_waker() {
    with_redraw_waker_mut(|waker| *waker = None);
}

pub(crate) fn current_replay_boundary_instance_key_from_scope() -> Option<u64> {
    with_execution_context(|context| context.current_component_instance_stack.last().copied())
}

fn with_persistent_focus_handle_store<R>(f: impl FnOnce(&PersistentFocusHandleStore) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&globals.persistent_focus_handle_store.borrow()))
}

fn with_persistent_focus_handle_store_mut<R>(
    f: impl FnOnce(&mut PersistentFocusHandleStore) -> R,
) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.persistent_focus_handle_store.borrow_mut()))
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
    with_persistent_focus_handle_store_mut(|store| match store.targets.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let value = FocusNode::new();
            entry.insert(PersistentFocusHandleEntry::new(value));
            value
        }
    })
}

pub(crate) fn persistent_focus_scope_for_current_instance<K: Hash>(slot_key: K) -> FocusScopeNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_persistent_focus_handle_store_mut(|store| match store.scopes.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let value = FocusScopeNode::new();
            entry.insert(PersistentFocusHandleEntry::new(value));
            value
        }
    })
}

pub(crate) fn persistent_focus_group_for_current_instance<K: Hash>(slot_key: K) -> FocusGroupNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_persistent_focus_handle_store_mut(|store| match store.groups.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let value = FocusGroupNode::new();
            entry.insert(PersistentFocusHandleEntry::new(value));
            value
        }
    })
}

pub(crate) fn has_persistent_focus_handle(handle_id: FocusHandleId) -> bool {
    with_persistent_focus_handle_store(|store| store.contains_handle(handle_id))
}

pub(crate) fn retain_persistent_focus_handles(
    live_instance_keys: &HashSet<u64>,
) -> RemovedPersistentFocusHandles {
    with_persistent_focus_handle_store_mut(|store| store.retain_instance_keys(live_instance_keys))
}

pub(crate) fn prune_persistent_focus_handles() {
    let live_instance_keys =
        TesseraRuntime::with(|runtime| runtime.component_tree.live_instance_keys());
    let removed = retain_persistent_focus_handles(&live_instance_keys);
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
    group_path: &[u64],
    instance_key_override: Option<u64>,
    f: impl FnOnce() -> R,
) -> R {
    struct ReplayScopeGuard {
        previous_group_path: Option<Vec<u64>>,
        previous_instance_key_stack: Option<Vec<u64>>,
        previous_instance_logic_id_override: Option<Option<u64>>,
    }

    impl Drop for ReplayScopeGuard {
        fn drop(&mut self) {
            if let Some(previous_group_path) = self.previous_group_path.take() {
                with_execution_context_mut(|context| {
                    context.group_path_stack = previous_group_path;
                });
            }
            if let Some(previous_instance_key_stack) = self.previous_instance_key_stack.take() {
                with_execution_context_mut(|context| {
                    context.instance_key_stack = previous_instance_key_stack;
                });
            }
            if let Some(previous_instance_logic_id_override) =
                self.previous_instance_logic_id_override.take()
            {
                with_execution_context_mut(|context| {
                    context.next_node_instance_logic_id_override =
                        previous_instance_logic_id_override;
                });
            }
        }
    }

    let previous_group_path = with_execution_context_mut(|context| {
        std::mem::replace(&mut context.group_path_stack, group_path.to_vec())
    });
    let previous_instance_key_stack = with_execution_context_mut(|context| {
        let next_stack = instance_key_override.into_iter().collect::<Vec<_>>();
        std::mem::replace(&mut context.instance_key_stack, next_stack)
    });
    let previous_instance_logic_id_override = with_execution_context_mut(|context| {
        context
            .next_node_instance_logic_id_override
            .replace(instance_logic_id)
    });
    let _guard = ReplayScopeGuard {
        previous_group_path: Some(previous_group_path),
        previous_instance_key_stack: Some(previous_instance_key_stack),
        previous_instance_logic_id_override: Some(previous_instance_logic_id_override),
    };

    f()
}

pub(crate) fn with_build_dirty_instance_keys<R>(
    dirty_instance_keys: &HashSet<u64>,
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
            .push(Arc::new(dirty_instance_keys.clone()));
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

/// Returns whether the instance key still has an unresolved build invalidation.
pub(crate) fn is_instance_key_pending_build_invalidation(instance_key: u64) -> bool {
    with_build_invalidation_tracker(|tracker| {
        tracker.dirty_instance_keys.contains(&instance_key)
            || tracker
                .cross_instance_dirty_instance_keys
                .contains(&instance_key)
    })
}

/// Returns whether the node with `instance_key` still needs a rebuild.
///
/// This is the single predicate behind the subtree-reuse precondition: a
/// subtree may only be reused while no node inside it still needs a rebuild.
/// Both deferred build invalidations and the current frame's dirty set count
/// as "needs a rebuild".
pub(crate) fn instance_key_needs_rebuild(instance_key: u64) -> bool {
    is_instance_key_build_dirty(instance_key)
        || is_instance_key_pending_build_invalidation(instance_key)
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
    let inserted = with_build_invalidation_tracker_mut(|tracker| {
        tracker.dirty_instance_keys.insert(instance_key)
    });
    if inserted {
        schedule_runtime_redraw();
    }
}

fn record_cross_instance_build_invalidation(instance_key: u64) {
    with_build_invalidation_tracker_mut(|tracker| {
        tracker.dirty_instance_keys.insert(instance_key);
        tracker
            .cross_instance_dirty_instance_keys
            .insert(instance_key);
    });
    schedule_runtime_redraw();
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

fn track_state_read_dependency(slot: SlotHandle, generation: u64) {
    let phase = match current_phase() {
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
    with_state_read_dependency_tracker_mut(|tracker| {
        if tracker
            .readers_by_state
            .get(&key)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        tracker
            .readers_by_state
            .entry(key)
            .or_default()
            .insert(reader_instance_key);
        tracker
            .states_by_reader
            .entry(reader_instance_key)
            .or_default()
            .insert(key);
    });
}

fn track_layout_state_read_dependency(slot: SlotHandle, generation: u64, phase: ReadPhase) {
    let Some(reader_instance_key) = current_measured_node_instance_key()
        .or_else(current_replay_boundary_instance_key_from_scope)
    else {
        return;
    };
    let reader_instance_key = PhaseReader {
        instance_key: reader_instance_key,
        phase,
    };
    let key = StateReadDependencyKey { slot, generation };
    with_layout_dirty_tracker_mut(|tracker| {
        if tracker
            .layout_readers_by_state
            .get(&key)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        tracker
            .layout_readers_by_state
            .entry(key)
            .or_default()
            .insert(reader_instance_key);
        tracker
            .layout_states_by_reader
            .entry(reader_instance_key)
            .or_default()
            .insert(key);
    });
}

fn mark_layout_state_readers_dirty(slot: SlotHandle, generation: u64) {
    let key = StateReadDependencyKey { slot, generation };
    let readers = with_layout_dirty_tracker_mut(|tracker| {
        tracker
            .layout_readers_by_state
            .get(&key)
            .map(|readers| readers.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default()
    });
    if readers.is_empty() {
        return;
    }
    with_layout_dirty_tracker_mut(|tracker| {
        for reader in readers {
            let target = match reader.phase {
                ReadPhase::Measure => &mut tracker.pending_measure_self_dirty_nodes,
                ReadPhase::Place => &mut tracker.pending_placement_self_dirty_nodes,
                ReadPhase::Transform => &mut tracker.pending_transform_dirty_nodes,
                ReadPhase::Paint => &mut tracker.pending_paint_dirty_nodes,
            };
            target.insert(reader.instance_key);
        }
    });
    schedule_runtime_redraw();
}

pub(crate) fn clear_phase_state_reads(instance_key: u64, phase: ReadPhase) {
    with_layout_dirty_tracker_mut(|tracker| {
        let reader = PhaseReader {
            instance_key,
            phase,
        };
        let Some(state_keys) = tracker.layout_states_by_reader.remove(&reader) else {
            return;
        };
        for state_key in state_keys {
            if let Some(readers) = tracker.layout_readers_by_state.get_mut(&state_key) {
                readers.remove(&reader);
                if readers.is_empty() {
                    tracker.layout_readers_by_state.remove(&state_key);
                }
            }
        }
    });
}

pub(crate) fn remove_layout_state_read_dependencies(instance_keys: &HashSet<u64>) {
    for &instance_key in instance_keys {
        for phase in [
            ReadPhase::Measure,
            ReadPhase::Place,
            ReadPhase::Transform,
            ReadPhase::Paint,
        ] {
            clear_phase_state_reads(instance_key, phase);
        }
    }
}

fn state_read_subscribers(slot: SlotHandle, generation: u64) -> Vec<u64> {
    let key = StateReadDependencyKey { slot, generation };
    with_state_read_dependency_tracker(|tracker| {
        tracker
            .readers_by_state
            .get(&key)
            .map(|readers| readers.iter().copied().collect())
            .unwrap_or_default()
    })
}

fn track_focus_dependency(kind: FocusReadDependencyKind) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_replay_boundary_instance_key_from_scope() else {
        return;
    };

    let key = FocusReadDependencyKey { kind };
    with_focus_read_dependency_tracker_mut(|tracker| {
        if tracker
            .readers_by_focus
            .get(&key)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        tracker
            .readers_by_focus
            .entry(key)
            .or_default()
            .insert(reader_instance_key);
        tracker
            .focus_by_reader
            .entry(reader_instance_key)
            .or_default()
            .insert(key);
    });
}

fn focus_read_subscribers_by_kind(kind: FocusReadDependencyKind) -> Vec<u64> {
    let key = FocusReadDependencyKey { kind };
    with_focus_read_dependency_tracker(|tracker| {
        tracker
            .readers_by_focus
            .get(&key)
            .map(|readers| readers.iter().copied().collect())
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
        if tracker
            .readers_by_slot
            .get(&handle)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        tracker
            .readers_by_slot
            .entry(handle)
            .or_default()
            .insert(reader_instance_key);
        tracker
            .slots_by_reader
            .entry(reader_instance_key)
            .or_default()
            .insert(handle);
    });
}

fn render_slot_read_subscribers(handle: FunctorHandle) -> Vec<u64> {
    with_render_slot_read_dependency_tracker(|tracker| {
        tracker
            .readers_by_slot
            .get(&handle)
            .map(|readers| readers.iter().copied().collect())
            .unwrap_or_default()
    })
}

pub(crate) fn remove_state_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_state_read_dependency_tracker_mut(|tracker| {
        for instance_key in instance_keys {
            let Some(state_keys) = tracker.states_by_reader.remove(instance_key) else {
                continue;
            };
            for state_key in state_keys {
                let mut remove_entry = false;
                if let Some(readers) = tracker.readers_by_state.get_mut(&state_key) {
                    readers.remove(instance_key);
                    remove_entry = readers.is_empty();
                }
                if remove_entry {
                    tracker.readers_by_state.remove(&state_key);
                }
            }
        }
    });
}

pub(crate) fn remove_focus_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_focus_read_dependency_tracker_mut(|tracker| {
        for instance_key in instance_keys {
            let Some(focus_keys) = tracker.focus_by_reader.remove(instance_key) else {
                continue;
            };
            for focus_key in focus_keys {
                let mut remove_entry = false;
                if let Some(readers) = tracker.readers_by_focus.get_mut(&focus_key) {
                    readers.remove(instance_key);
                    remove_entry = readers.is_empty();
                }
                if remove_entry {
                    tracker.readers_by_focus.remove(&focus_key);
                }
            }
        }
    });
}

pub(crate) fn remove_render_slot_read_dependencies(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_render_slot_read_dependency_tracker_mut(|tracker| {
        for instance_key in instance_keys {
            let Some(slot_keys) = tracker.slots_by_reader.remove(instance_key) else {
                continue;
            };
            for slot_key in slot_keys {
                let mut remove_entry = false;
                if let Some(readers) = tracker.readers_by_slot.get_mut(&slot_key) {
                    readers.remove(instance_key);
                    remove_entry = readers.is_empty();
                }
                if remove_entry {
                    tracker.readers_by_slot.remove(&slot_key);
                }
            }
        }
    });
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
        tracker
            .dirty_instance_keys
            .retain(|instance_key| !instance_keys.contains(instance_key));
        tracker
            .cross_instance_dirty_instance_keys
            .retain(|instance_key| !instance_keys.contains(instance_key));
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
    RUNTIME_GLOBALS.with(|globals| f(&globals.frame_clock_tracker.borrow()))
}

fn with_frame_clock_tracker_mut<R>(f: impl FnOnce(&mut FrameClockTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.frame_clock_tracker.borrow_mut()))
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
    with_frame_clock_tracker_mut(|tracker| *tracker = FrameClockTracker::default());
}

pub(crate) fn has_pending_frame_nanos_receivers() -> bool {
    with_frame_clock_tracker(|tracker| !tracker.receivers.is_empty())
}

pub(crate) fn tick_frame_nanos_receivers() {
    with_frame_clock_tracker_mut(|tracker| {
        let frame_nanos = tracker.current_frame_nanos;
        tracker.receivers.retain(|_, receiver| {
            matches!(
                (receiver.callback)(frame_nanos),
                FrameNanosControl::Continue
            )
        });
    });
}

pub(crate) fn remove_frame_nanos_receivers(instance_keys: &HashSet<u64>) {
    if instance_keys.is_empty() {
        return;
    }
    with_frame_clock_tracker_mut(|tracker| {
        tracker
            .receivers
            .retain(|_, receiver| !instance_keys.contains(&receiver.owner_instance_key));
    });
}

pub(crate) fn clear_frame_nanos_receivers() {
    with_frame_clock_tracker_mut(|tracker| tracker.receivers.clear());
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

    with_frame_clock_tracker_mut(|tracker| {
        tracker.receivers.entry(key).or_insert_with(|| {
            let mut callback = callback;
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
    });
}

pub(crate) fn drop_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let removed = with_slot_table_mut(|table| {
        let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();
        for (slot, entry) in table.entries.iter() {
            if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
                continue;
            }
            if entry.retained {
                continue;
            }
            freed.push((slot, entry.key));
        }
        let mut removed = Vec::with_capacity(freed.len());
        for (slot, key) in freed {
            removed.extend(table.entries.remove(slot));
            table.key_to_slot.remove(&key);
        }
        for instance_logic_id in instance_logic_ids {
            table.cursors_by_instance_logic_id.remove(instance_logic_id);
        }
        removed
    });
    drop(removed);
}

fn with_layout_dirty_tracker_mut<R>(f: impl FnOnce(&mut LayoutDirtyTracker) -> R) -> R {
    RUNTIME_GLOBALS.with(|globals| f(&mut globals.layout_dirty_tracker.borrow_mut()))
}

fn record_layout_policy_dirty(
    instance_key: u64,
    layout_policy: &dyn LayoutModifierDyn,
    modifier: &Modifier,
) {
    if current_phase() != Some(RuntimePhase::Build) {
        return;
    }
    with_layout_dirty_tracker_mut(|tracker| {
        let (
            measure_changed,
            placement_changed,
            transform_changed,
            parent_data_changed,
            next_layout_inputs,
        ) = match tracker.previous_layout_inputs_by_node.remove(&instance_key) {
            Some(previous) => {
                let measure_changed = !previous.policy.dyn_measure_eq(layout_policy)
                    || !previous.modifier.layout_measure_eq(modifier);
                let placement_changed = !previous.policy.dyn_placement_eq(layout_policy)
                    || !previous.modifier.layout_placement_eq(modifier);
                let transform_changed = !previous.modifier.position_eq(modifier);
                let parent_data_changed = !previous.modifier.parent_data_eq(modifier);
                if !measure_changed
                    && !placement_changed
                    && !transform_changed
                    && !parent_data_changed
                {
                    (false, false, false, false, previous)
                } else {
                    (
                        measure_changed,
                        placement_changed,
                        transform_changed,
                        parent_data_changed,
                        LayoutInputSnapshot {
                            policy: layout_policy.clone_box(),
                            modifier: modifier.clone(),
                        },
                    )
                }
            }
            None => (
                true,
                true,
                true,
                true,
                LayoutInputSnapshot {
                    policy: layout_policy.clone_box(),
                    modifier: modifier.clone(),
                },
            ),
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
        tracker
            .frame_layout_inputs_by_node
            .insert(instance_key, next_layout_inputs);
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
        tracker.frame_layout_inputs_by_node.clear();
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
        tracker
            .previous_layout_inputs_by_node
            .extend(std::mem::take(&mut tracker.frame_layout_inputs_by_node));
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
    })
}

pub(crate) fn reset_layout_dirty_tracking() {
    with_layout_dirty_tracker_mut(|tracker| *tracker = LayoutDirtyTracker::default());
}

fn record_component_replay_snapshot(runtime: &TesseraRuntime, node_id: NodeId) {
    let Some(node) = runtime.component_tree.get(node_id) else {
        return;
    };
    let Some(replay) = node.replay.clone() else {
        return;
    };

    let tree = runtime.component_tree.tree();
    let parent_instance_key = tree
        .get(node_id)
        .and_then(|n| n.parent())
        .and_then(|parent_id| tree.get(parent_id))
        .map(|parent| parent.get().instance_key);

    let snapshot = ReplayNodeSnapshot {
        instance_key: node.instance_key,
        parent_instance_key,
        instance_logic_id: node.instance_logic_id,
        group_path: current_group_path(),
        instance_key_override: current_instance_key_override(),
        fn_name: node.fn_name.clone(),
        replay,
    };
    with_component_replay_tracker_mut(|tracker| {
        tracker
            .current_nodes
            .insert(snapshot.instance_key, snapshot);
    });
}

pub(crate) fn record_layout_structure(instance_key: u64, children: Vec<u64>) {
    with_layout_dirty_tracker_mut(|tracker| {
        if tracker.previous_children_by_node.get(&instance_key) != Some(&children) {
            tracker
                .structural_changes
                .changed_nodes
                .insert(instance_key);
            tracker
                .previous_children_by_node
                .insert(instance_key, children);
        }
        tracker
            .structural_changes
            .removed_nodes
            .remove(&instance_key);
    });
}

pub(crate) fn remove_layout_structure(instance_keys: &HashSet<u64>) {
    with_layout_dirty_tracker_mut(|tracker| {
        for key in instance_keys {
            tracker.previous_children_by_node.remove(key);
            tracker.previous_layout_inputs_by_node.remove(key);
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
/// Handles are validated with a slot generation token so stale references fail
/// fast if their slot has been recycled.
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
        self.slot == other.slot && self.generation == other.generation
    }
}

impl<T> Eq for State<T> {}

impl<T> Hash for State<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl<T> State<T> {
    fn new(slot: SlotHandle, generation: u64) -> Self {
        Self {
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
        RUNTIME_GLOBALS
            .try_with(|globals| {
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
        track_state_read_dependency(self.slot, self.generation);
        with_slot_table(|table| {
            let value = state_value_ref::<T>(table, self.slot, self.generation);
            f(value)
        })
    }

    /// Execute a closure with a mutable reference to the stored value.
    #[track_caller]
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
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
        with_slot_table_mut(|table| restore_state_value::<T>(table, slot, generation, value));

        let subscribers = state_read_subscribers(self.slot, self.generation);
        // A write performed while another component is building is a backwards
        // write for every other reader: they may already have been rebuilt in
        // this pass with the old value, so their invalidation must not be
        // treated as already covered by the current replay round.
        let writer_instance_key = if matches!(current_phase(), Some(RuntimePhase::Build)) {
            current_replay_boundary_instance_key_from_scope()
        } else {
            None
        };
        for instance_key in subscribers {
            match writer_instance_key {
                Some(writer) if writer != instance_key => {
                    record_cross_instance_build_invalidation(instance_key);
                }
                _ => record_replay_boundary_invalidation_for_instance_key(instance_key),
            }
        }
        mark_layout_state_readers_dirty(self.slot, self.generation);
        result
    }

    /// Get a cloned value. Requires `T: Clone`.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.with(Clone::clone)
    }

    /// Replace the stored value.
    #[track_caller]
    pub fn set(&self, value: T) {
        self.with_mut(|slot| *slot = value);
    }
}

struct RuntimeGlobals {
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

thread_local! {
    static RUNTIME_GLOBALS: RuntimeGlobals = RuntimeGlobals::new();
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
        RUNTIME_GLOBALS.with(|globals| f(&globals.runtime.borrow()))
    }

    /// Executes a closure with an exclusive, mutable reference to the runtime.
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        RUNTIME_GLOBALS.with(|globals| f(&mut globals.runtime.borrow_mut()))
    }

    /// Get the current window size in physical pixels.
    pub fn window_size(&self) -> [u32; 2] {
        self.window_size
    }

    /// Sets identity fields for the current component node.
    pub(crate) fn set_current_node_identity(&mut self, instance_key: u64, instance_logic_id: u64) {
        self.component_tree
            .set_current_node_identity(instance_key, instance_logic_id);
    }

    /// Stores replay metadata for the current component node.
    pub(crate) fn set_current_component_replay<P>(
        &mut self,
        runner: Arc<dyn ErasedComponentRunner>,
        props: &P,
    ) -> bool
    where
        P: Prop,
    {
        let current_node_info = self
            .component_tree
            .current_node()
            .map(|node| (node.instance_key, node.instance_logic_id));
        let previous_replay = current_node_info.and_then(|(instance_key, instance_logic_id)| {
            with_component_replay_tracker(|tracker| {
                let previous = tracker.previous_nodes.get(&instance_key)?;
                if previous.instance_logic_id != instance_logic_id {
                    return None;
                }
                if previous.replay.props.equals(props) {
                    Some(previous.replay.clone())
                } else {
                    None
                }
            })
        });

        let pending_dirty = current_node_info
            .map(|(instance_key, _)| consume_pending_build_invalidation(instance_key))
            .unwrap_or(false);

        if let Some((instance_key, instance_logic_id)) = current_node_info
            && let Some(replay) = previous_replay.clone()
            && !is_instance_key_build_dirty(instance_key)
            && !pending_dirty
            && self
                .component_tree
                .try_reuse_current_subtree(instance_key, instance_logic_id)
        {
            if let Some(node) = self.component_tree.current_node_mut() {
                node.replay = Some(replay);
                node.props_unchanged_from_previous = true;
            }
            return true;
        }

        if let Some(node) = self.component_tree.current_node_mut() {
            if let Some(replay) = previous_replay {
                node.replay = Some(replay);
                node.props_unchanged_from_previous = true;
            } else {
                node.replay = Some(ComponentReplayData::new(runner, props));
                node.props_unchanged_from_previous = false;
            }
        } else {
            debug_assert!(
                false,
                "set_current_component_replay must be called inside a component build"
            );
            return false;
        }
        if let Some(node_id) = current_node_id() {
            record_component_replay_snapshot(self, node_id);
        }
        false
    }

    /// Sets the unified layout node for the current component node.
    pub(crate) fn set_current_layout_node_boxed(&mut self, policy: Box<dyn LayoutModifierDyn>) {
        if let Some(node) = self.component_tree.current_node_mut() {
            node.layout_node = policy;
        } else {
            debug_assert!(
                false,
                "set_current_layout_node_boxed must be called inside a component build"
            );
        }
    }

    /// Sets the draw modifier node for the current component node.
    pub(crate) fn set_current_draw_node_boxed(
        &mut self,
        node: Box<dyn crate::modifier::DrawModifierNode>,
    ) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.draw_node = node;
        } else {
            debug_assert!(
                false,
                "set_current_draw_node_boxed must be called inside a component build"
            );
        }
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
            node.globally_positioned_handlers.push(handler);
        } else {
            debug_assert!(
                false,
                "push_current_globally_positioned_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_accessibility(&mut self, accessibility: Option<AccessibilityNode>) {
        if let Some(node_id) = current_node_id()
            && let Some(metadata) = self.component_tree.metadatas_mut().get_mut(&node_id)
        {
            metadata.accessibility = accessibility;
        } else {
            debug_assert!(
                false,
                "set_current_accessibility must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_accessibility_action_handler(
        &mut self,
        handler: Option<AccessibilityActionHandler>,
    ) {
        if let Some(node_id) = current_node_id()
            && let Some(metadata) = self.component_tree.metadatas_mut().get_mut(&node_id)
        {
            metadata.accessibility_action_handler = handler;
        } else {
            debug_assert!(
                false,
                "set_current_accessibility_action_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn bind_current_focus_requester(&mut self, requester: FocusRequester) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.focus_requester_binding = Some(requester);
        } else {
            debug_assert!(
                false,
                "bind_current_focus_requester must be called inside a component build"
            );
        }
    }

    pub(crate) fn ensure_current_focus_target(&mut self, node: FocusNode) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current.focus_registration.is_none() {
                current.focus_registration = Some(FocusRegistration::target(node));
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
            if current.focus_registration.is_none() {
                current.focus_registration = Some(FocusRegistration::scope(scope));
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
            if current.focus_registration.is_none() {
                current.focus_registration = Some(FocusRegistration::group(group));
            }
        } else {
            debug_assert!(
                false,
                "ensure_current_focus_group must be called inside a component build"
            );
        }
    }

    pub(crate) fn current_focus_target_handle(&self) -> Option<FocusNode> {
        let registration = self.component_tree.current_node()?.focus_registration?;
        (registration.kind == FocusRegistrationKind::Target)
            .then(|| FocusNode::from_handle_id(registration.id))
    }

    pub(crate) fn current_focus_scope_handle(&self) -> Option<FocusScopeNode> {
        let registration = self.component_tree.current_node()?.focus_registration?;
        (registration.kind == FocusRegistrationKind::Scope)
            .then(|| FocusScopeNode::from_handle_id(registration.id))
    }

    pub(crate) fn current_focus_group_handle(&self) -> Option<FocusGroupNode> {
        let registration = self.component_tree.current_node()?.focus_registration?;
        (registration.kind == FocusRegistrationKind::Group)
            .then(|| FocusGroupNode::from_handle_id(registration.id))
    }

    pub(crate) fn set_current_focus_properties(&mut self, properties: FocusProperties) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if let Some(registration) = current.focus_registration.as_mut() {
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
            .and_then(|node| node.focus_registration.as_mut());
        if let Some(registration) = registration {
            registration.properties.preferred_tab_stop = preferred;
        } else {
            debug_assert!(false, "preferred_tab_stop requires a focus registration");
        }
    }

    pub(crate) fn set_current_focus_traversal_policy(&mut self, policy: FocusTraversalPolicy) {
        if let Some(current) = self.component_tree.current_node_mut() {
            if current.focus_registration.is_some_and(|registration| {
                matches!(
                    registration.kind,
                    FocusRegistrationKind::Scope | FocusRegistrationKind::Group
                )
            }) {
                current.focus_traversal_policy = Some(policy);
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
            current.focus_changed_handler = Some(handler);
        } else {
            debug_assert!(
                false,
                "set_current_focus_changed_handler must be called inside a component build"
            );
        }
    }

    pub(crate) fn set_current_focus_event_handler(&mut self, handler: CallbackWith<FocusState>) {
        if let Some(current) = self.component_tree.current_node_mut() {
            current.focus_event_handler = Some(handler);
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
            current.focus_beyond_bounds_handler = Some(handler);
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
            current.focus_reveal_handler = Some(handler);
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
                .focus_registration
                .is_some_and(|registration| registration.kind == FocusRegistrationKind::Scope)
            {
                current.focus_restorer_fallback = Some(fallback);
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

    pub(crate) fn finalize_current_layout_policy_dirty(&mut self) {
        if let Some(node) = self.component_tree.current_node() {
            record_layout_policy_dirty(
                node.instance_key,
                node.layout_node.as_ref(),
                &node.modifier,
            );
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
    instance_logic_id_popped: bool,
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
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.instance_logic_id_popped {
            pop_instance_logic_id();
            self.instance_logic_id_popped = true;
        }
    }
}

impl Drop for NodeContextGuard {
    fn drop(&mut self) {
        #[cfg(feature = "profiling")]
        {
            let _ = self.profiling_guard.take();
        }
        if !self.popped {
            pop_current_node();
            self.popped = true;
        }
        if !self.instance_logic_id_popped {
            pop_instance_logic_id();
            self.instance_logic_id_popped = true;
        }
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

/// Push the given node id as the current executing component for this thread.
pub fn push_current_node(
    node_id: NodeId,
    component_type_id: u64,
    fn_name: &str,
) -> NodeContextGuard {
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let parent_node_id = with_execution_context_mut(|context| {
        let parent = context.node_context_stack.last().copied();
        context.node_context_stack.push(node_id);
        parent
    });

    // Get the parent's call index and increment it
    // This distinguishes multiple calls to the same component (e.g., foo(1);
    // foo(2);)
    let parent_call_index = next_child_instance_call_index();
    let parent_instance_logic_id = with_execution_context(|context| {
        context.instance_logic_id_stack.last().copied().unwrap_or(0)
    });

    let group_path_hash = current_group_path_hash();
    let has_group_path = with_execution_context(|context| !context.group_path_stack.is_empty());

    // Combine component_type_id with parent_instance_logic_id, the current
    // control-flow group path, and the call index local to that group. This
    // ensures:
    // 1. foo(1) and foo(2) get different logic_ids (via parent_call_index)
    // 2. Components in different control-flow groups get different logic_ids
    //    even when each group starts its local call index from zero
    // 3. Components in different container instances get different logic_ids
    //    (via parent_instance_logic_id)
    let instance_salt = if let Some(key_hash) = current_instance_key_override() {
        hash_components(&[&key_hash, &group_path_hash, &parent_call_index])
    } else if has_group_path {
        hash_components(&[&group_path_hash, &parent_call_index])
    } else {
        parent_call_index
    };

    let instance_logic_id =
        if let Some(instance_logic_id_override) = take_next_node_instance_logic_id_override() {
            instance_logic_id_override
        } else if parent_call_index == 0
            && parent_instance_logic_id == 0
            && current_instance_key_override().is_none()
            && !has_group_path
        {
            component_type_id
        } else {
            hash_components(&[
                &component_type_id,
                &parent_instance_logic_id,
                &instance_salt,
            ])
        };

    with_execution_context_mut(|context| {
        context.instance_logic_id_stack.push(instance_logic_id);
    });

    push_order_frame();

    #[cfg(feature = "profiling")]
    let profiling_guard = match current_phase() {
        Some(RuntimePhase::Build) => {
            crate::profiler::make_build_scope_guard(node_id, parent_node_id, fn_name)
        }
        _ => None,
    };

    NodeContextGuard {
        popped: false,
        instance_logic_id_popped: false,
        #[cfg(feature = "profiling")]
        profiling_guard,
    }
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
    #[cfg(not(feature = "profiling"))]
    let _ = fn_name;
    #[allow(unused_variables)]
    let parent_node_id = with_execution_context_mut(|context| {
        let parent = context.node_context_stack.last().copied();
        context.node_context_stack.push(node_id);
        parent
    });

    let _ = next_child_instance_call_index();

    with_execution_context_mut(|context| {
        context.instance_logic_id_stack.push(instance_logic_id);
    });
    push_order_frame();

    #[cfg(feature = "profiling")]
    let profiling_guard = match current_phase() {
        Some(RuntimePhase::Build) => {
            crate::profiler::make_build_scope_guard(node_id, parent_node_id, fn_name)
        }
        _ => None,
    };

    NodeContextGuard {
        popped: false,
        instance_logic_id_popped: false,
        #[cfg(feature = "profiling")]
        profiling_guard,
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
    });
    pop_order_frame("ORDER_FRAME_STACK underflow: attempted to pop from empty stack");
}

/// Get the node id at the top of the thread-local component stack.
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
    let instance_logic_id = current_instance_logic_id_opt()
        .expect("current_instance_key must be called inside a component");
    let group_path_hash = current_group_path_hash();
    hash_components(&[&instance_logic_id, &group_path_hash])
}

fn pop_instance_logic_id() {
    with_execution_context_mut(|context| {
        let _ = context.instance_logic_id_stack.pop();
    });
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

/// Push a group id onto the thread-local control-flow stack.
pub(crate) fn push_group_id(group_id: u64) {
    with_execution_context_mut(|context| {
        context.group_path_stack.push(group_id);
    });
}

/// Pop a group id from the thread-local control-flow stack.
pub(crate) fn pop_group_id(expected_group_id: u64) {
    with_execution_context_mut(|context| {
        if let Some(popped) = context.group_path_stack.pop() {
            debug_assert_eq!(
                popped, expected_group_id,
                "Unbalanced GroupGuard stack: expected {}, got {}",
                expected_group_id, popped
            );
        } else {
            debug_assert!(false, "Attempted to pop GroupGuard from an empty stack");
        }
    });
}

/// Get a clone of the current control-flow path.
fn current_group_path() -> Vec<u64> {
    with_execution_context(|context| context.group_path_stack.clone())
}

fn current_group_path_hash() -> u64 {
    with_execution_context(|context| hash_components(&[&context.group_path_stack[..]]))
}

fn state_value_ref<'a, T: 'static>(
    table: &'a SlotTable,
    slot: SlotHandle,
    generation: u64,
) -> &'a T {
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
    entry
        .value
        .as_ref()
        .unwrap_or_else(|| panic!("State slot {slot:?} has been cleared"))
        .downcast_ref::<T>()
        .unwrap_or_else(|| panic!("State slot {slot:?} downcast failed"))
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
            with_slot_table_mut(|table| restore_state_value::<T>(table, slot, generation, value));
        }
    }
}

fn take_state_value<T: 'static>(table: &mut SlotTable, slot: SlotHandle, generation: u64) -> T {
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
    let boxed = entry
        .value
        .take()
        .unwrap_or_else(|| panic!("State slot {slot:?} has been cleared"));
    *boxed
        .downcast::<T>()
        .unwrap_or_else(|_| panic!("State slot {slot:?} downcast failed"))
}

fn restore_state_value<T: Send + Sync + 'static>(
    table: &mut SlotTable,
    slot: SlotHandle,
    generation: u64,
    value: T,
) {
    let Some(entry) = table.entries.get_mut(slot) else {
        return;
    };
    if entry.generation != generation || entry.key.type_id != TypeId::of::<T>() {
        return;
    }
    if entry.value.is_none() {
        entry.value = Some(Box::new(value));
    }
}

fn box_state_value<T: Send + Sync + 'static>(value: T) -> Box<dyn Any + Send + Sync> {
    Box::new(value)
}

fn box_functor_value<T: Send + Sync + 'static>(value: T) -> Box<dyn Any + Send + Sync> {
    Box::new(Arc::new(value))
}

fn downcast_functor_arc<T: Send + Sync + 'static>(value: &dyn Any, slot: SlotHandle) -> Arc<T> {
    value
        .downcast_ref::<Arc<T>>()
        .unwrap_or_else(|| panic!("callback slot {slot:?} downcast failed"))
        .clone()
}

fn touch_state_slot<T, F>(
    table: &mut SlotTable,
    slot: SlotHandle,
    init: &mut Option<F>,
    retained: bool,
    type_name: &'static str,
) -> u64
where
    T: Send + Sync + 'static,
    F: FnOnce() -> T,
{
    let epoch = table.epoch;
    let entry = table
        .entries
        .get_mut(slot)
        .expect("slot entry should exist");
    if entry.key.type_id != TypeId::of::<T>() {
        panic!(
            "{type_name} type mismatch: expected {}, found {:?}",
            std::any::type_name::<T>(),
            entry.key.type_id
        );
    }
    entry.last_alive_epoch = epoch;
    if retained {
        entry.retained = true;
    }
    if entry.value.is_none() {
        let init_fn = init
            .take()
            .unwrap_or_else(|| panic!("{type_name} init called more than once"));
        entry.value = Some(box_state_value(init_fn()));
        entry.generation = entry.generation.wrapping_add(1);
    }
    entry.generation
}

fn insert_state_slot<T, F>(
    table: &mut SlotTable,
    key: SlotKey,
    init: &mut Option<F>,
    retained: bool,
    type_name: &'static str,
) -> (SlotHandle, u64)
where
    T: Send + Sync + 'static,
    F: FnOnce() -> T,
{
    let epoch = table.epoch;
    let init_fn = init
        .take()
        .unwrap_or_else(|| panic!("{type_name} init called more than once"));
    let generation = 1u64;
    let slot = table.entries.insert(SlotEntry {
        key,
        generation,
        value: Some(box_state_value(init_fn())),
        last_alive_epoch: epoch,
        retained,
    });
    table.key_to_slot.insert(key, slot);
    (slot, generation)
}

fn resolve_state_slot<K, F, T>(key: K, init: F, retained: bool, type_name: &'static str) -> State<T>
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
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    with_slot_table_mut(|table| {
        let mut init_opt = Some(init);
        if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
            let generation =
                touch_state_slot::<T, F>(table, slot, &mut init_opt, retained, type_name);
            return State::new(slot, generation);
        }

        let (slot, generation) =
            insert_state_slot::<T, F>(table, slot_key, &mut init_opt, retained, type_name);
        State::new(slot, generation)
    })
}

fn current_instance_key_override() -> Option<u64> {
    with_execution_context(|context| context.instance_key_stack.last().copied())
}

/// RAII guard that tracks control-flow grouping for the current component node.
///
/// A guard pushes the provided group id when constructed and pops it when
/// dropped, ensuring grouping stays balanced even with early returns or panics.
pub struct GroupGuard {
    group_id: u64,
}

impl GroupGuard {
    /// Push a group id onto the current component's group stack.
    pub fn new(group_id: u64) -> Self {
        push_group_id(group_id);
        push_order_frame();
        Self { group_id }
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        pop_order_frame("ORDER_FRAME_STACK underflow: attempted to pop GroupGuard frame");
        pop_group_id(self.group_id);
    }
}

/// RAII guard for path-only control-flow groups, primarily used for loop
/// bodies.
///
/// Unlike [`GroupGuard`], this guard does not create a new local order frame.
/// Repeated iterations therefore continue consuming sibling call indices from
/// the surrounding component scope instead of restarting from zero each time.
pub struct PathGroupGuard {
    group_id: u64,
}

impl PathGroupGuard {
    /// Push a group id onto the current component's group stack without
    /// resetting local call-order counters.
    pub fn new(group_id: u64) -> Self {
        push_group_id(group_id);
        Self { group_id }
    }
}

impl Drop for PathGroupGuard {
    fn drop(&mut self) {
        pop_group_id(self.group_id);
    }
}

/// RAII guard that sets a stable instance key for the duration of a block.
pub struct InstanceKeyGuard {
    key_hash: u64,
}

impl InstanceKeyGuard {
    /// Push a key hash for instance identity.
    pub fn new(key_hash: u64) -> Self {
        with_execution_context_mut(|context| {
            context.instance_key_stack.push(key_hash);
        });
        Self { key_hash }
    }
}

impl Drop for InstanceKeyGuard {
    fn drop(&mut self) {
        with_execution_context_mut(|context| {
            let popped = context.instance_key_stack.pop();
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

fn compute_functor_slot_key<K: Hash>(key: &K) -> (u64, u64) {
    let instance_logic_id = current_instance_logic_id();
    let group_path_hash = current_group_path_hash();
    let key_hash = hash_components(&[key]);

    let call_counter = next_order_counter(
        OrderCounterKind::Functor,
        "ORDER_FRAME_STACK is empty; callback constructors must be called inside a component",
    );

    let slot_hash = hash_components(&[&group_path_hash, &key_hash, &call_counter]);
    (instance_logic_id, slot_hash)
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

fn remember_functor_cell_with_key<K, T, F>(key: K, init: F) -> (Arc<T>, FunctorHandle)
where
    K: Hash,
    T: Send + Sync + 'static,
    F: FnOnce() -> T,
{
    ensure_build_phase();
    let (instance_logic_id, slot_hash) = compute_functor_slot_key(&key);
    let slot_key = SlotKey {
        instance_logic_id,
        slot_hash,
        type_id: TypeId::of::<T>(),
    };

    with_slot_table_mut(|table| {
        let mut init_opt = Some(init);
        if let Some(slot) = table.try_fast_slot_lookup(slot_key) {
            let epoch = table.epoch;
            let (generation, value) = {
                let entry = table
                    .entries
                    .get_mut(slot)
                    .expect("functor slot entry should exist");

                if entry.key.type_id != slot_key.type_id {
                    panic!(
                        "callback slot type mismatch: expected {}, found {:?}",
                        std::any::type_name::<T>(),
                        entry.key.type_id
                    );
                }

                entry.last_alive_epoch = epoch;
                if entry.value.is_none() {
                    let init_fn = init_opt
                        .take()
                        .expect("callback slot init called more than once");
                    entry.value = Some(box_functor_value(init_fn()));
                    entry.generation = entry.generation.wrapping_add(1);
                }

                (
                    entry.generation,
                    downcast_functor_arc::<T>(
                        entry
                            .value
                            .as_ref()
                            .expect("callback slot must contain a value")
                            .as_ref(),
                        slot,
                    ),
                )
            };

            (value, FunctorHandle::new(slot, generation))
        } else if let Some(slot) = table.key_to_slot.get(&slot_key).copied() {
            table.record_slot_usage_slow(instance_logic_id, slot);
            let epoch = table.epoch;
            let (generation, value) = {
                let entry = table
                    .entries
                    .get_mut(slot)
                    .expect("functor slot entry should exist");

                if entry.key.type_id != slot_key.type_id {
                    panic!(
                        "callback slot type mismatch: expected {}, found {:?}",
                        std::any::type_name::<T>(),
                        entry.key.type_id
                    );
                }

                entry.last_alive_epoch = epoch;
                if entry.value.is_none() {
                    let init_fn = init_opt
                        .take()
                        .expect("callback slot init called more than once");
                    entry.value = Some(box_functor_value(init_fn()));
                    entry.generation = entry.generation.wrapping_add(1);
                }

                (
                    entry.generation,
                    downcast_functor_arc::<T>(
                        entry
                            .value
                            .as_ref()
                            .expect("callback slot must contain a value")
                            .as_ref(),
                        slot,
                    ),
                )
            };

            (value, FunctorHandle::new(slot, generation))
        } else {
            let epoch = table.epoch;
            let init_fn = init_opt
                .take()
                .expect("callback slot init called more than once");
            let generation = 1u64;
            let slot = table.entries.insert(SlotEntry {
                key: slot_key,
                generation,
                value: Some(box_functor_value(init_fn())),
                last_alive_epoch: epoch,
                retained: false,
            });

            table.key_to_slot.insert(slot_key, slot);
            table.record_slot_usage_slow(instance_logic_id, slot);

            let value = downcast_functor_arc::<T>(
                table
                    .entries
                    .get(slot)
                    .expect("functor slot entry should exist")
                    .value
                    .as_ref()
                    .expect("callback slot must contain a value")
                    .as_ref(),
                slot,
            );

            (value, FunctorHandle::new(slot, generation))
        }
    })
}

fn load_functor_cell<T>(handle: FunctorHandle) -> Arc<T>
where
    T: Send + Sync + 'static,
{
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

        downcast_functor_arc::<T>(
            entry
                .value
                .as_ref()
                .unwrap_or_else(|| panic!("Callback slot {:?} has been cleared", handle.slot))
                .as_ref(),
            handle.slot,
        )
    })
}

pub(crate) fn remember_callback_handle<F>(handler: F) -> FunctorHandle
where
    F: Fn() + Send + Sync + 'static,
{
    let handler = Arc::new(handler) as Arc<dyn Fn() + Send + Sync>;
    let (cell, handle) = remember_functor_cell_with_key((), {
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
    let render = Arc::new(render) as Arc<dyn Fn() + Send + Sync>;
    let creator_instance_key = current_replay_boundary_instance_key_from_scope()
        .unwrap_or_else(|| panic!("RenderSlot handles must be created during a component build"));
    let (cell, handle) = remember_functor_cell_with_key((), {
        let render = Arc::clone(&render);
        move || RenderSlotCell::new(render)
    });
    cell.update(render);
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
    let render = Arc::new(render) as Arc<dyn Fn(T) + Send + Sync>;
    let creator_instance_key =
        current_replay_boundary_instance_key_from_scope().unwrap_or_else(|| {
            panic!("RenderSlotWith handles must be created during a component build")
        });
    let (cell, handle) = remember_functor_cell_with_key((), {
        let render = Arc::clone(&render);
        move || RenderSlotWithCell::new(render)
    });
    cell.update(render);
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
    let (cell, handle) = remember_functor_cell_with_key((), {
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

/// Reset all slot buffers (used on suspension).
pub fn reset_slots() {
    let removed = with_slot_table_mut(SlotTable::reset);
    drop(removed);
}

pub(crate) fn recycle_recomposed_slots_for_instance_logic_ids(instance_logic_ids: &HashSet<u64>) {
    if instance_logic_ids.is_empty() {
        return;
    }

    let removed = with_slot_table_mut(|table| {
        let epoch = table.epoch;
        let mut freed: Vec<(SlotHandle, SlotKey)> = Vec::new();

        for (slot, entry) in table.entries.iter() {
            if !instance_logic_ids.contains(&entry.key.instance_logic_id) {
                continue;
            }
            if entry.last_alive_epoch == epoch || entry.retained {
                continue;
            }
            freed.push((slot, entry.key));
        }

        let mut removed = Vec::with_capacity(freed.len());
        for (slot, key) in freed {
            removed.extend(table.entries.remove(slot));
            table.key_to_slot.remove(&key);
        }
        removed
    });
    drop(removed);
}

pub(crate) fn live_slot_instance_logic_ids() -> HashSet<u64> {
    with_slot_table(|table| {
        table
            .entries
            .iter()
            .map(|(_, entry)| entry.key.instance_logic_id)
            .collect()
    })
}

/// Remember a value across frames with an explicit key.
///
/// This function allows a component to "remember" state across recomposition
/// (build) passes, using a user-provided key to identify the state. This is
/// particularly useful for state generated inside loops or dynamic collections
/// where the execution order might change.
///
/// The `init` closure is executed only once — when the key is first
/// encountered. On subsequent updates with the same key, the stored value is
/// returned and `init` is not called.
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle stored in the thread-local slot
/// table. Use `with`, `with_mut`, `get`, or `set` to read or update the value.
///
/// # Comparison with [`remember`]
///
/// Use this function when the state is generated inside a loop or dynamic
/// collection where the execution order might change. In other cases,
/// [`remember`] is sufficient.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
pub fn remember_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    resolve_state_slot(key, init, false, "remember_with_key")
}

/// Remember a value across recomposition (build) passes.
///
/// This function allows a component to "remember" state across recomposition
/// (build) passes.
/// The `init` closure is executed only once — when the component first runs.
/// On subsequent updates, the stored value is returned and `init` is not
/// called.
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle stored in the thread-local slot
/// table. Use `with`, `with_mut`, `get`, or `set` to read or update the value.
///
/// # Comparison with [`remember_with_key`]
///
/// `remember` identifies stored state based on the component's call order and
/// control-flow path. It associates state by position within a component, but
/// this does not work reliably for dynamically generated state inside loops.
/// For state that is allocated dynamically in loops, consider using
/// [`remember_with_key`] to explicitly provide a unique key.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
pub fn remember<F, T>(init: F) -> State<T>
where
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    remember_with_key((), init)
}

/// Retain a value across recomposition (build) passes with an explicit key,
/// even if unused.
///
/// Unlike [`remember_with_key`], state created with this function will **not**
/// be recycled when the component stops calling it. This is useful for state
/// that should persist across navigation, such as scroll positions or form
/// inputs.
///
/// The `init` closure is executed only once — when the key is first
/// encountered. On subsequent updates with the same key, the stored value is
/// returned and `init` is not called.
///
/// # Use Cases
///
/// - Preserving scroll position when navigating away and returning to a page
/// - Retaining form input values across route changes
/// - Caching expensive computation results that should survive component
///   unmounts
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle stored in the thread-local slot
/// table. Use `with`, `with_mut`, `get`, or `set` to read or update the value.
///
/// # Comparison with [`remember_with_key`]
///
/// Use [`remember_with_key`] for ephemeral component state that should be
/// cleaned up when the component is no longer rendered. Use `retain_with_key`
/// for persistent state that must survive even when a subtree is not rebuilt
/// for some time.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
pub fn retain_with_key<K, F, T>(key: K, init: F) -> State<T>
where
    K: Hash,
    F: FnOnce() -> T,
    T: Send + Sync + 'static,
{
    resolve_state_slot(key, init, true, "retain_with_key")
}

/// Retain a value across recomposition (build) passes, even if unused.
///
/// Unlike [`remember`], state created with this function will **not** be
/// recycled when the component stops calling it. This is useful for state that
/// should persist across navigation, such as scroll positions or form inputs.
///
/// The `init` closure is executed only once — when the component first runs.
/// On subsequent updates, the stored value is returned and `init` is not
/// called.
///
/// # Use Cases
///
/// - Preserving scroll position when navigating away and returning to a page
/// - Retaining form input values across route changes
/// - Caching expensive computation results that should survive component
///   unmounts
///
/// # Interior mutability
///
/// This function returns a `State<T>` handle stored in the thread-local slot
/// table. Use `with`, `with_mut`, `get`, or `set` to read or update the value.
///
/// # Comparison with [`retain_with_key`]
///
/// `retain` identifies stored state based on the component's call order and
/// control-flow path. It associates state by position within a component, but
/// this does not work reliably for dynamically generated state inside loops.
/// For state that is allocated dynamically in loops, consider using
/// [`retain_with_key`] to explicitly provide a unique key.
///
/// # Panics
///
/// This function must be called during a component's build/render phase.
/// Calling it during the measure or input handling phases will panic.
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
    let _group_guard = GroupGuard::new(key_hash);
    let _instance_guard = InstanceKeyGuard::new(key_hash);
    block()
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    };

    use super::*;
    use crate::component_tree::{ComponentNode, NodeRole};
    use crate::execution_context::{
        reset_execution_context, with_execution_context, with_execution_context_mut,
    };
    use crate::layout::{DefaultLayoutNode, MeasureScope, NoopDrawModifier};
    use crate::modifier::{LayoutModifierChild, LayoutModifierInput, LayoutModifierNode};
    use crate::prop::{CallbackWith, ComponentReplayData, RenderSlot, make_component_runner};
    use crate::tessera;

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
        let replay = ComponentReplayData::new(make_component_runner::<()>(|_| {}), &());
        with_component_replay_tracker_mut(|tracker| {
            tracker.previous_nodes.insert(
                instance_key,
                ReplayNodeSnapshot {
                    instance_key,
                    parent_instance_key: None,
                    instance_logic_id: 0,
                    group_path: Vec::new(),
                    instance_key_override: None,
                    fn_name: "test_component".to_string(),
                    replay,
                },
            );
        });
    }

    fn with_test_component_scope<R>(component_type_id: u64, f: impl FnOnce() -> R) -> R {
        reset_execution_context();
        let mut arena = crate::Arena::<()>::new();
        let node_id = arena.new_node(());
        let _phase_guard = push_phase(RuntimePhase::Build);
        let _node_guard = push_current_node(node_id, component_type_id, "test_component");
        let _instance_guard = push_current_component_instance_key(current_instance_key());
        f()
    }

    #[test]
    fn state_liveness_survives_updates_and_rejects_expired_slots() {
        reset_slots();
        let state = with_test_component_scope(815, || remember(|| 4_u32));
        assert!(state.is_alive());
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
        reset_frame_clock();
        begin_frame_clock(Instant::now());

        let result = std::panic::catch_unwind(|| {
            receive_frame_nanos(|_| FrameNanosControl::Continue);
        });
        assert!(result.is_err());
    }

    #[test]
    fn tick_frame_nanos_receivers_removes_stopped_receivers() {
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
        let mut outer = HashSet::default();
        outer.insert(7);

        assert!(!is_instance_key_build_dirty(7));
        with_build_dirty_instance_keys(&outer, || {
            assert!(is_instance_key_build_dirty(7));
            assert!(!is_instance_key_build_dirty(8));

            let mut inner = HashSet::default();
            inner.insert(8);
            with_build_dirty_instance_keys(&inner, || {
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
        let mut dirty = HashSet::default();
        dirty.insert(11);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_build_dirty_instance_keys(&dirty, || {
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
        reset_layout_dirty_tracking();

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            record_layout_policy_dirty(
                1,
                &DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                },
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
                &DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 1,
                },
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
                &DirtySplitPolicy {
                    measure_key: 1,
                    placement_key: 1,
                },
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
        reset_layout_dirty_tracking();
        begin_recompose_slot_epoch();
        let root_node = TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.add_node(ComponentNode {
                fn_name: "root".to_string(),
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
            })
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
        reset_layout_dirty_tracking();

        begin_frame_layout_dirty_tracking();
        {
            let _phase_guard = push_phase(RuntimePhase::Build);
            let modifier = Modifier::new().push_layout(DirtyMeasureModifierNode { width: 10 });
            record_layout_policy_dirty(
                1,
                &DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                },
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
                &DirtySplitPolicy {
                    measure_key: 0,
                    placement_key: 0,
                },
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
        with_execution_context_mut(|context| {
            context.group_path_stack = vec![1, 2, 3];
        });
        with_execution_context_mut(|context| {
            context.instance_key_stack = vec![5];
        });
        with_execution_context_mut(|context| {
            context.next_node_instance_logic_id_override = Some(9);
        });

        with_replay_scope(42, &[7, 8], Some(11), || {
            assert_eq!(current_group_path(), vec![7, 8]);
            assert_eq!(current_instance_key_override(), Some(11));
            assert_eq!(take_next_node_instance_logic_id_override(), Some(42));
            assert_eq!(take_next_node_instance_logic_id_override(), None);
        });

        assert_eq!(current_group_path(), vec![1, 2, 3]);
        assert_eq!(current_instance_key_override(), Some(5));
        let restored_override =
            with_execution_context(|context| context.next_node_instance_logic_id_override);
        assert_eq!(restored_override, Some(9));
    }

    #[test]
    fn with_replay_scope_restores_on_panic() {
        with_execution_context_mut(|context| {
            context.group_path_stack = vec![5];
        });
        with_execution_context_mut(|context| {
            context.instance_key_stack = vec![13];
        });
        with_execution_context_mut(|context| {
            context.next_node_instance_logic_id_override = None;
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_replay_scope(77, &[10], Some(17), || {
                assert_eq!(current_group_path(), vec![10]);
                assert_eq!(current_instance_key_override(), Some(17));
                panic!("expected panic");
            });
        }));
        assert!(result.is_err());

        assert_eq!(current_group_path(), vec![5]);
        assert_eq!(current_instance_key_override(), Some(13));
        let restored_override =
            with_execution_context(|context| context.next_node_instance_logic_id_override);
        assert_eq!(restored_override, None);
    }

    #[test]
    fn group_local_remember_does_not_shift_following_slots() {
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
        let observed = Arc::new(AtomicUsize::new(99));

        with_test_component_scope(11014, || {
            let observed = Arc::clone(&observed);
            optional_value_component().on_value(move |value: Option<usize>| {
                observed.store(
                    value.map_or(0, |next| next.saturating_add(1)),
                    Ordering::SeqCst,
                );
            });
        });

        assert_eq!(observed.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn tessera_accepts_non_default_required_props_when_provided() {
        let observed = Arc::new(AtomicUsize::new(0));

        with_test_component_scope(11016, || {
            let observed = Arc::clone(&observed);
            required_value_component()
                .value(13)
                .on_value(move |value: usize| {
                    observed.store(value, Ordering::SeqCst);
                });
        });

        assert_eq!(observed.load(Ordering::SeqCst), 13);
    }

    #[test]
    fn render_slot_update_invalidates_reader_instance() {
        reset_slots();
        reset_component_replay_tracking();
        reset_render_slot_read_dependencies();
        reset_build_invalidations();

        let reader_boundary_key = Arc::new(AtomicU64::new(0));

        begin_recompose_slot_epoch();
        let first = with_test_component_scope(11002, || {
            let reader_boundary_key = Arc::clone(&reader_boundary_key);
            RenderSlot::new(move || {
                let instance_key = current_replay_boundary_instance_key_from_scope()
                    .expect("slot render must run inside replay boundary");
                reader_boundary_key.store(instance_key, Ordering::SeqCst);
            })
        });

        with_test_component_scope(11003, || {
            first.render();
        });
        let reader_instance_key = reader_boundary_key.load(Ordering::SeqCst);
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
        reset_slots();
        reset_component_replay_tracking();
        reset_render_slot_read_dependencies();
        reset_build_invalidations();
        reset_execution_context();

        let reader_boundary_key = Arc::new(AtomicU64::new(0));

        begin_recompose_slot_epoch();
        let first = with_test_component_scope(12002, || {
            let reader_boundary_key = Arc::clone(&reader_boundary_key);
            RenderSlot::new(move || {
                let instance_key = current_replay_boundary_instance_key_from_scope()
                    .expect("slot render must run inside replay boundary");
                reader_boundary_key.store(instance_key, Ordering::SeqCst);
            })
        });

        with_test_component_scope(12003, || {
            first.render();
        });
        let reader_instance_key = reader_boundary_key.load(Ordering::SeqCst);
        assert!(reader_instance_key != 0);
        seed_previous_replay_snapshot(reader_instance_key);

        assert!(!has_pending_build_invalidations());

        begin_recompose_slot_epoch();
        let mut dirty = HashSet::default();
        dirty.insert(reader_instance_key);
        with_build_dirty_instance_keys(&dirty, || {
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
        begin_recompose_slot_epoch();
        let root_node = TesseraRuntime::with_mut(|runtime| {
            runtime.component_tree.add_node(ComponentNode {
                fn_name: "root".to_string(),
                role: NodeRole::Composition,
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
            })
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
        let mut table = SlotTable::default();
        let keep_key = SlotKey {
            instance_logic_id: 7,
            slot_hash: 11,
            type_id: TypeId::of::<i32>(),
        };
        let drop_key = SlotKey {
            instance_logic_id: 7,
            slot_hash: 12,
            type_id: TypeId::of::<i32>(),
        };

        let keep_slot = table.entries.insert(SlotEntry {
            key: keep_key,
            generation: 1,
            value: Some(Box::new(10_i32)),
            last_alive_epoch: 0,
            retained: true,
        });
        let drop_slot = table.entries.insert(SlotEntry {
            key: drop_key,
            generation: 1,
            value: Some(Box::new(20_i32)),
            last_alive_epoch: 0,
            retained: false,
        });
        table.key_to_slot.insert(keep_key, keep_slot);
        table.key_to_slot.insert(drop_key, drop_slot);
        with_slot_table_mut(|slot_table| *slot_table = table);

        let mut stale = HashSet::default();
        stale.insert(7_u64);
        drop_slots_for_instance_logic_ids(&stale);

        with_slot_table(|table| {
            assert!(table.entries.get(keep_slot).is_some());
            assert!(table.key_to_slot.contains_key(&keep_key));
            assert!(table.entries.get(drop_slot).is_none());
            assert!(!table.key_to_slot.contains_key(&drop_key));
        });
    }
}
