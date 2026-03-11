use std::{
    any::{Any, TypeId},
    hash::Hash,
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    context::ContextMap,
    execution_context::{OrderCounterKind, next_order_counter},
    focus::{
        FocusGroupNode, FocusHandleId, FocusNode, FocusRequester, FocusRequesterId, FocusScopeNode,
    },
    prop::ComponentReplayData,
};

use super::{
    build_scope::{
        RuntimePhase, current_component_instance_key_from_scope, current_group_path_hash,
        current_instance_logic_id, current_phase, hash_components,
    },
    session::with_composition_runtime,
    slot_table::{FunctorHandle, SlotTable, remember},
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub(crate) struct ContextSlotKey {
    pub instance_logic_id: u64,
    pub slot_hash: u64,
    pub type_id: TypeId,
}

impl Default for ContextSlotKey {
    fn default() -> Self {
        Self {
            instance_logic_id: 0,
            slot_hash: 0,
            type_id: TypeId::of::<()>(),
        }
    }
}

#[derive(Default)]
pub(crate) struct ContextSlotEntry {
    pub key: ContextSlotKey,
    pub generation: u64,
    pub value: Option<Arc<dyn Any + Send + Sync>>,
    pub last_alive_epoch: u64,
}

#[derive(Default)]
pub(crate) struct ContextSlotTable {
    pub entries: Vec<ContextSlotEntry>,
    pub free_list: Vec<u32>,
    pub key_to_slot: HashMap<ContextSlotKey, u32>,
    pub epoch: u64,
}

impl ContextSlotTable {
    pub(crate) fn begin_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
    }
}

#[derive(Default)]
pub(crate) struct ContextSnapshotTracker {
    pub previous_by_instance_key: HashMap<u64, ContextMap>,
    pub current_by_instance_key: HashMap<u64, ContextMap>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ContextReadDependencyKey {
    pub slot: u32,
    pub generation: u64,
}

#[derive(Default)]
pub(crate) struct ContextReadDependencyTracker {
    pub readers_by_context: HashMap<ContextReadDependencyKey, HashSet<u64>>,
    pub contexts_by_reader: HashMap<u64, HashSet<ContextReadDependencyKey>>,
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

    fn tick_missing(&mut self) {
        self.missing_frames = self.missing_frames.saturating_add(1);
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PersistentFocusHandleKey {
    pub instance_key: u64,
    pub slot_hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct StateReadDependencyKey {
    slot: super::SlotHandle,
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

#[derive(Clone)]
pub(crate) struct ReplayNodeSnapshot {
    pub instance_key: u64,
    pub instance_logic_id: u64,
    pub group_path: Vec<u64>,
    pub instance_key_override: Option<u64>,
    pub replay: ComponentReplayData,
}

#[derive(Default)]
struct ComponentReplayTracker {
    previous_nodes: HashMap<u64, ReplayNodeSnapshot>,
    current_nodes: HashMap<u64, ReplayNodeSnapshot>,
}

#[derive(Default)]
struct BuildInvalidationTracker {
    dirty_instance_keys: HashSet<u64>,
}

#[derive(Default)]
pub(crate) struct BuildInvalidationSet {
    pub dirty_instance_keys: HashSet<u64>,
}

type RedrawWaker = Arc<dyn Fn() + Send + Sync + 'static>;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameNanosControl {
    Continue,
    Stop,
}

type FrameNanosReceiverCallback = Box<dyn FnMut(u64) -> FrameNanosControl + Send + 'static>;

struct FrameNanosReceiver {
    owner_instance_key: u64,
    callback: FrameNanosReceiverCallback,
}

pub(crate) struct CompositionRuntime {
    slot_table: Arc<RwLock<SlotTable>>,
    context_slot_table: Arc<RwLock<ContextSlotTable>>,
    context_snapshot_tracker: Arc<RwLock<ContextSnapshotTracker>>,
    context_read_dependency_tracker: Arc<RwLock<ContextReadDependencyTracker>>,
    component_replay_tracker: Arc<RwLock<ComponentReplayTracker>>,
    build_invalidation_tracker: Arc<RwLock<BuildInvalidationTracker>>,
    state_read_dependency_tracker: Arc<RwLock<StateReadDependencyTracker>>,
    focus_read_dependency_tracker: Arc<RwLock<FocusReadDependencyTracker>>,
    render_slot_read_dependency_tracker: Arc<RwLock<RenderSlotReadDependencyTracker>>,
    redraw_waker: Arc<RwLock<Option<RedrawWaker>>>,
    persistent_focus_handle_store: Arc<RwLock<PersistentFocusHandleStore>>,
    frame_clock_tracker: Arc<Mutex<FrameClockTracker>>,
}

impl Default for CompositionRuntime {
    fn default() -> Self {
        Self {
            slot_table: Arc::new(RwLock::new(SlotTable::default())),
            context_slot_table: Arc::new(RwLock::new(ContextSlotTable::default())),
            context_snapshot_tracker: Arc::new(RwLock::new(ContextSnapshotTracker::default())),
            context_read_dependency_tracker: Arc::new(RwLock::new(
                ContextReadDependencyTracker::default(),
            )),
            component_replay_tracker: Arc::new(RwLock::new(ComponentReplayTracker::default())),
            build_invalidation_tracker: Arc::new(RwLock::new(BuildInvalidationTracker::default())),
            state_read_dependency_tracker: Arc::new(RwLock::new(
                StateReadDependencyTracker::default(),
            )),
            focus_read_dependency_tracker: Arc::new(RwLock::new(
                FocusReadDependencyTracker::default(),
            )),
            render_slot_read_dependency_tracker: Arc::new(RwLock::new(
                RenderSlotReadDependencyTracker::default(),
            )),
            redraw_waker: Arc::new(RwLock::new(None)),
            persistent_focus_handle_store: Arc::new(RwLock::new(
                PersistentFocusHandleStore::default(),
            )),
            frame_clock_tracker: Arc::new(Mutex::new(FrameClockTracker::default())),
        }
    }
}

impl PersistentFocusHandleStore {
    fn contains_handle(&self, handle_id: super::FocusHandleId) -> bool {
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

    fn retain_instance_keys(
        &mut self,
        live_instance_keys: &HashSet<u64>,
    ) -> RemovedPersistentFocusHandles {
        let mut removed = RemovedPersistentFocusHandles::default();

        self.targets.retain(|key, entry| {
            if live_instance_keys.contains(&key.instance_key) {
                entry.missing_frames = 0;
                true
            } else {
                entry.tick_missing();
                let keep = entry.missing_frames <= 1;
                if !keep {
                    removed.handle_ids.insert(entry.value.handle_id());
                }
                keep
            }
        });

        self.scopes.retain(|key, entry| {
            if live_instance_keys.contains(&key.instance_key) {
                entry.missing_frames = 0;
                true
            } else {
                entry.tick_missing();
                let keep = entry.missing_frames <= 1;
                if !keep {
                    removed.handle_ids.insert(entry.value.handle_id());
                }
                keep
            }
        });

        self.groups.retain(|key, entry| {
            if live_instance_keys.contains(&key.instance_key) {
                entry.missing_frames = 0;
                true
            } else {
                entry.tick_missing();
                let keep = entry.missing_frames <= 1;
                if !keep {
                    removed.handle_ids.insert(entry.value.handle_id());
                }
                keep
            }
        });

        self.requesters.retain(|key, entry| {
            if live_instance_keys.contains(&key.instance_key) {
                entry.missing_frames = 0;
                true
            } else {
                entry.tick_missing();
                let keep = entry.missing_frames <= 1;
                if !keep {
                    removed.requester_ids.insert(entry.value.id());
                }
                keep
            }
        });

        removed
    }

    fn clear(&mut self) {
        self.targets.clear();
        self.scopes.clear();
        self.groups.clear();
        self.requesters.clear();
    }
}

impl CompositionRuntime {
    pub(super) fn slot_table(&self) -> Arc<RwLock<SlotTable>> {
        Arc::clone(&self.slot_table)
    }

    pub(crate) fn context_slot_table(&self) -> Arc<RwLock<ContextSlotTable>> {
        Arc::clone(&self.context_slot_table)
    }

    pub(crate) fn context_snapshot_tracker(&self) -> Arc<RwLock<ContextSnapshotTracker>> {
        Arc::clone(&self.context_snapshot_tracker)
    }

    pub(crate) fn context_read_dependency_tracker(
        &self,
    ) -> Arc<RwLock<ContextReadDependencyTracker>> {
        Arc::clone(&self.context_read_dependency_tracker)
    }

    fn begin_frame_component_replay_tracking(&self) {
        self.component_replay_tracker.write().current_nodes.clear();
    }

    fn finalize_frame_component_replay_tracking(&self) {
        let mut tracker = self.component_replay_tracker.write();
        tracker.previous_nodes = std::mem::take(&mut tracker.current_nodes);
    }

    fn finalize_frame_component_replay_tracking_partial(&self) {
        let mut tracker = self.component_replay_tracker.write();
        let current = std::mem::take(&mut tracker.current_nodes);
        tracker.previous_nodes.extend(current);
    }

    fn reset_component_replay_tracking(&self) {
        *self.component_replay_tracker.write() = ComponentReplayTracker::default();
    }

    fn previous_component_replay_nodes(&self) -> HashMap<u64, ReplayNodeSnapshot> {
        self.component_replay_tracker.read().previous_nodes.clone()
    }

    pub(super) fn previous_component_replay_node(
        &self,
        instance_key: u64,
    ) -> Option<ReplayNodeSnapshot> {
        self.component_replay_tracker
            .read()
            .previous_nodes
            .get(&instance_key)
            .cloned()
    }

    fn remove_previous_component_replay_nodes(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        let mut tracker = self.component_replay_tracker.write();
        tracker
            .previous_nodes
            .retain(|instance_key, _| !instance_keys.contains(instance_key));
        tracker
            .current_nodes
            .retain(|instance_key, _| !instance_keys.contains(instance_key));
    }

    pub(crate) fn record_component_replay_snapshot(&self, snapshot: ReplayNodeSnapshot) {
        self.component_replay_tracker
            .write()
            .current_nodes
            .insert(snapshot.instance_key, snapshot);
    }

    fn schedule_redraw(&self) {
        let callback = self.redraw_waker.read().clone();
        if let Some(callback) = callback {
            callback();
        }
    }

    fn install_redraw_waker(&self, callback: RedrawWaker) {
        *self.redraw_waker.write() = Some(callback);
    }

    fn clear_redraw_waker(&self) {
        *self.redraw_waker.write() = None;
    }

    pub(crate) fn record_component_invalidation_for_instance_key(&self, instance_key: u64) {
        let inserted = self
            .build_invalidation_tracker
            .write()
            .dirty_instance_keys
            .insert(instance_key);
        if inserted {
            self.schedule_redraw();
        }
    }

    pub(crate) fn consume_pending_build_invalidation(&self, instance_key: u64) -> bool {
        self.build_invalidation_tracker
            .write()
            .dirty_instance_keys
            .remove(&instance_key)
    }

    fn has_persistent_focus_handle(&self, handle_id: FocusHandleId) -> bool {
        self.persistent_focus_handle_store
            .read()
            .contains_handle(handle_id)
    }

    fn persistent_focus_target_for_key(&self, key: PersistentFocusHandleKey) -> FocusNode {
        let mut store = self.persistent_focus_handle_store.write();
        match store.targets.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusNode::new();
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    }

    fn persistent_focus_scope_for_key(&self, key: PersistentFocusHandleKey) -> FocusScopeNode {
        let mut store = self.persistent_focus_handle_store.write();
        match store.scopes.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusScopeNode::new();
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    }

    fn persistent_focus_group_for_key(&self, key: PersistentFocusHandleKey) -> FocusGroupNode {
        let mut store = self.persistent_focus_handle_store.write();
        match store.groups.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusGroupNode::new();
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    }

    fn persistent_focus_requester_for_key(&self, key: PersistentFocusHandleKey) -> FocusRequester {
        let mut store = self.persistent_focus_handle_store.write();
        match store.requesters.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => entry.get_mut().mark_live(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = FocusRequester::new();
                entry.insert(PersistentFocusHandleEntry::new(value));
                value
            }
        }
    }

    fn retain_persistent_focus_handles(
        &self,
        live_instance_keys: &HashSet<u64>,
    ) -> RemovedPersistentFocusHandles {
        self.persistent_focus_handle_store
            .write()
            .retain_instance_keys(live_instance_keys)
    }

    fn clear_persistent_focus_handles(&self) {
        self.persistent_focus_handle_store.write().clear();
    }

    fn take_build_invalidations(&self) -> BuildInvalidationSet {
        let mut tracker = self.build_invalidation_tracker.write();
        BuildInvalidationSet {
            dirty_instance_keys: std::mem::take(&mut tracker.dirty_instance_keys),
        }
    }

    fn reset_build_invalidations(&self) {
        *self.build_invalidation_tracker.write() = BuildInvalidationTracker::default();
    }

    fn remove_build_invalidations(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        self.build_invalidation_tracker
            .write()
            .dirty_instance_keys
            .retain(|instance_key| !instance_keys.contains(instance_key));
    }

    fn has_pending_build_invalidations(&self) -> bool {
        !self
            .build_invalidation_tracker
            .read()
            .dirty_instance_keys
            .is_empty()
    }

    fn track_state_read_dependency(&self, key: StateReadDependencyKey, reader_instance_key: u64) {
        let tracker = self.state_read_dependency_tracker.upgradable_read();
        if tracker
            .readers_by_state
            .get(&key)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        let mut tracker = RwLockUpgradableReadGuard::upgrade(tracker);
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
    }

    fn state_read_subscribers(&self, key: StateReadDependencyKey) -> Vec<u64> {
        self.state_read_dependency_tracker
            .read()
            .readers_by_state
            .get(&key)
            .map(|readers| readers.iter().copied().collect())
            .unwrap_or_default()
    }

    fn track_focus_dependency(&self, key: FocusReadDependencyKey, reader_instance_key: u64) {
        let tracker = self.focus_read_dependency_tracker.upgradable_read();
        if tracker
            .readers_by_focus
            .get(&key)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        let mut tracker = RwLockUpgradableReadGuard::upgrade(tracker);
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
    }

    fn focus_read_subscribers(&self, key: FocusReadDependencyKey) -> Vec<u64> {
        self.focus_read_dependency_tracker
            .read()
            .readers_by_focus
            .get(&key)
            .map(|readers| readers.iter().copied().collect())
            .unwrap_or_default()
    }

    fn track_render_slot_read_dependency(&self, handle: FunctorHandle, reader_instance_key: u64) {
        let tracker = self.render_slot_read_dependency_tracker.upgradable_read();
        if tracker
            .readers_by_slot
            .get(&handle)
            .is_some_and(|readers| readers.contains(&reader_instance_key))
        {
            return;
        }
        let mut tracker = RwLockUpgradableReadGuard::upgrade(tracker);
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
    }

    fn render_slot_read_subscribers(&self, handle: FunctorHandle) -> Vec<u64> {
        self.render_slot_read_dependency_tracker
            .read()
            .readers_by_slot
            .get(&handle)
            .map(|readers| readers.iter().copied().collect())
            .unwrap_or_default()
    }

    fn remove_state_read_dependencies(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        let mut tracker = self.state_read_dependency_tracker.write();
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
    }

    fn remove_focus_read_dependencies(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        let mut tracker = self.focus_read_dependency_tracker.write();
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
    }

    fn remove_render_slot_read_dependencies(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        let mut tracker = self.render_slot_read_dependency_tracker.write();
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
    }

    fn reset_state_read_dependencies(&self) {
        *self.state_read_dependency_tracker.write() = StateReadDependencyTracker::default();
    }

    fn reset_focus_read_dependencies(&self) {
        *self.focus_read_dependency_tracker.write() = FocusReadDependencyTracker::default();
    }

    fn reset_render_slot_read_dependencies(&self) {
        *self.render_slot_read_dependency_tracker.write() =
            RenderSlotReadDependencyTracker::default();
    }

    fn begin_frame_clock(&self, now: Instant) {
        let mut tracker = self.frame_clock_tracker.lock();
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
    }

    fn reset_frame_clock(&self) {
        *self.frame_clock_tracker.lock() = FrameClockTracker::default();
    }

    fn has_pending_frame_nanos_receivers(&self) -> bool {
        !self.frame_clock_tracker.lock().receivers.is_empty()
    }

    fn tick_frame_nanos_receivers(&self) {
        let frame_nanos = self.frame_clock_tracker.lock().current_frame_nanos;
        let mut tracker = self.frame_clock_tracker.lock();
        tracker.receivers.retain(|_, receiver| {
            matches!(
                (receiver.callback)(frame_nanos),
                FrameNanosControl::Continue
            )
        });
    }

    fn remove_frame_nanos_receivers(&self, instance_keys: &HashSet<u64>) {
        if instance_keys.is_empty() {
            return;
        }
        self.frame_clock_tracker
            .lock()
            .receivers
            .retain(|_, receiver| !instance_keys.contains(&receiver.owner_instance_key));
    }

    fn clear_frame_nanos_receivers(&self) {
        self.frame_clock_tracker.lock().receivers.clear();
    }

    fn current_frame_time(&self) -> Option<Instant> {
        self.frame_clock_tracker.lock().current_frame_time
    }

    fn current_frame_nanos(&self) -> u64 {
        self.frame_clock_tracker.lock().current_frame_nanos
    }

    fn frame_delta(&self) -> Duration {
        self.frame_clock_tracker.lock().frame_delta
    }
}

pub(crate) fn begin_frame_component_replay_tracking() {
    with_composition_runtime(CompositionRuntime::begin_frame_component_replay_tracking);
}

pub(crate) fn finalize_frame_component_replay_tracking() {
    with_composition_runtime(CompositionRuntime::finalize_frame_component_replay_tracking);
}

pub(crate) fn finalize_frame_component_replay_tracking_partial() {
    with_composition_runtime(CompositionRuntime::finalize_frame_component_replay_tracking_partial);
}

pub(crate) fn reset_component_replay_tracking() {
    with_composition_runtime(CompositionRuntime::reset_component_replay_tracking);
}

pub(crate) fn previous_component_replay_nodes() -> HashMap<u64, ReplayNodeSnapshot> {
    with_composition_runtime(CompositionRuntime::previous_component_replay_nodes)
}

pub(crate) fn remove_previous_component_replay_nodes(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| {
        runtime.remove_previous_component_replay_nodes(instance_keys)
    });
}

pub(crate) fn install_redraw_waker(callback: Arc<dyn Fn() + Send + Sync + 'static>) {
    with_composition_runtime(|runtime| runtime.install_redraw_waker(callback));
}

pub(crate) fn clear_redraw_waker() {
    with_composition_runtime(CompositionRuntime::clear_redraw_waker);
}

fn current_persistent_focus_handle_key<K: Hash>(slot_key: K) -> PersistentFocusHandleKey {
    let Some(instance_key) = current_component_instance_key_from_scope() else {
        panic!("persistent focus handles must be requested during a component build");
    };
    let slot_hash = hash_components(&[&slot_key]);
    PersistentFocusHandleKey {
        instance_key,
        slot_hash,
    }
}

#[doc(hidden)]
pub fn persistent_focus_target_for_current_instance<K: Hash>(slot_key: K) -> FocusNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_composition_runtime(|runtime| runtime.persistent_focus_target_for_key(key))
}

#[doc(hidden)]
pub fn persistent_focus_scope_for_current_instance<K: Hash>(slot_key: K) -> FocusScopeNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_composition_runtime(|runtime| runtime.persistent_focus_scope_for_key(key))
}

#[doc(hidden)]
pub fn persistent_focus_group_for_current_instance<K: Hash>(slot_key: K) -> FocusGroupNode {
    let key = current_persistent_focus_handle_key(slot_key);
    with_composition_runtime(|runtime| runtime.persistent_focus_group_for_key(key))
}

#[doc(hidden)]
pub fn persistent_focus_requester_for_current_instance<K: Hash>(slot_key: K) -> FocusRequester {
    let key = current_persistent_focus_handle_key(slot_key);
    with_composition_runtime(|runtime| runtime.persistent_focus_requester_for_key(key))
}

pub(crate) fn has_persistent_focus_handle(handle_id: FocusHandleId) -> bool {
    with_composition_runtime(|runtime| runtime.has_persistent_focus_handle(handle_id))
}

pub(crate) fn retain_persistent_focus_handles(
    live_instance_keys: &HashSet<u64>,
) -> RemovedPersistentFocusHandles {
    with_composition_runtime(|runtime| runtime.retain_persistent_focus_handles(live_instance_keys))
}

pub(crate) fn clear_persistent_focus_handles() {
    with_composition_runtime(CompositionRuntime::clear_persistent_focus_handles);
}

pub(crate) fn record_component_invalidation_for_instance_key(instance_key: u64) {
    with_composition_runtime(|runtime| {
        runtime.record_component_invalidation_for_instance_key(instance_key)
    });
}

pub(crate) fn track_state_read_dependency(slot: super::SlotHandle, generation: u64) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_from_scope() else {
        return;
    };
    let key = StateReadDependencyKey { slot, generation };
    with_composition_runtime(|runtime| {
        runtime.track_state_read_dependency(key, reader_instance_key)
    });
}

pub(crate) fn state_read_subscribers(slot: super::SlotHandle, generation: u64) -> Vec<u64> {
    let key = StateReadDependencyKey { slot, generation };
    with_composition_runtime(|runtime| runtime.state_read_subscribers(key))
}

pub(crate) fn track_focus_read_dependency(handle_id: FocusHandleId) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_from_scope() else {
        return;
    };
    let key = FocusReadDependencyKey {
        kind: FocusReadDependencyKind::Handle(handle_id),
    };
    with_composition_runtime(|runtime| runtime.track_focus_dependency(key, reader_instance_key));
}

pub(crate) fn track_focus_requester_read_dependency(requester_id: FocusRequesterId) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_from_scope() else {
        return;
    };
    let key = FocusReadDependencyKey {
        kind: FocusReadDependencyKind::Requester(requester_id),
    };
    with_composition_runtime(|runtime| runtime.track_focus_dependency(key, reader_instance_key));
}

pub(crate) fn focus_read_subscribers(handle_id: FocusHandleId) -> Vec<u64> {
    let key = FocusReadDependencyKey {
        kind: FocusReadDependencyKind::Handle(handle_id),
    };
    with_composition_runtime(|runtime| runtime.focus_read_subscribers(key))
}

pub(crate) fn focus_requester_read_subscribers(requester_id: FocusRequesterId) -> Vec<u64> {
    let key = FocusReadDependencyKey {
        kind: FocusReadDependencyKind::Requester(requester_id),
    };
    with_composition_runtime(|runtime| runtime.focus_read_subscribers(key))
}

pub(crate) fn track_render_slot_read_dependency(handle: FunctorHandle) {
    if !matches!(current_phase(), Some(RuntimePhase::Build)) {
        return;
    }
    let Some(reader_instance_key) = current_component_instance_key_from_scope() else {
        return;
    };
    with_composition_runtime(|runtime| {
        runtime.track_render_slot_read_dependency(handle, reader_instance_key)
    });
}

pub(crate) fn render_slot_read_subscribers(handle: FunctorHandle) -> Vec<u64> {
    with_composition_runtime(|runtime| runtime.render_slot_read_subscribers(handle))
}

pub(crate) fn remove_state_read_dependencies(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| runtime.remove_state_read_dependencies(instance_keys));
}

pub(crate) fn remove_focus_read_dependencies(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| runtime.remove_focus_read_dependencies(instance_keys));
}

pub(crate) fn remove_render_slot_read_dependencies(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| runtime.remove_render_slot_read_dependencies(instance_keys));
}

pub(crate) fn reset_state_read_dependencies() {
    with_composition_runtime(CompositionRuntime::reset_state_read_dependencies);
}

pub(crate) fn reset_focus_read_dependencies() {
    with_composition_runtime(CompositionRuntime::reset_focus_read_dependencies);
}

pub(crate) fn reset_render_slot_read_dependencies() {
    with_composition_runtime(CompositionRuntime::reset_render_slot_read_dependencies);
}

pub(crate) fn take_build_invalidations() -> BuildInvalidationSet {
    with_composition_runtime(CompositionRuntime::take_build_invalidations)
}

pub(crate) fn reset_build_invalidations() {
    with_composition_runtime(CompositionRuntime::reset_build_invalidations);
}

pub(crate) fn remove_build_invalidations(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| runtime.remove_build_invalidations(instance_keys));
}

pub(crate) fn has_pending_build_invalidations() -> bool {
    with_composition_runtime(CompositionRuntime::has_pending_build_invalidations)
}

pub(crate) fn begin_frame_clock(now: Instant) {
    with_composition_runtime(|runtime| runtime.begin_frame_clock(now));
}

pub(crate) fn reset_frame_clock() {
    with_composition_runtime(CompositionRuntime::reset_frame_clock);
}

pub(crate) fn has_pending_frame_nanos_receivers() -> bool {
    with_composition_runtime(CompositionRuntime::has_pending_frame_nanos_receivers)
}

pub(crate) fn tick_frame_nanos_receivers() {
    with_composition_runtime(CompositionRuntime::tick_frame_nanos_receivers);
}

pub(crate) fn remove_frame_nanos_receivers(instance_keys: &HashSet<u64>) {
    with_composition_runtime(|runtime| runtime.remove_frame_nanos_receivers(instance_keys));
}

pub(crate) fn clear_frame_nanos_receivers() {
    with_composition_runtime(CompositionRuntime::clear_frame_nanos_receivers);
}

/// Returns the timestamp of the current frame, if available.
///
/// The value is set by the renderer at frame begin.
pub fn current_frame_time() -> Option<Instant> {
    with_composition_runtime(CompositionRuntime::current_frame_time)
}

/// Returns the current frame timestamp in nanoseconds from runtime origin.
pub fn current_frame_nanos() -> u64 {
    with_composition_runtime(CompositionRuntime::current_frame_nanos)
}

/// Returns the elapsed time since the previous frame.
pub fn frame_delta() -> Duration {
    with_composition_runtime(CompositionRuntime::frame_delta)
}

fn ensure_frame_receive_phase() {
    match current_phase() {
        Some(RuntimePhase::Build) => {}
        Some(RuntimePhase::Measure) => {
            panic!("receive_frame_nanos must not be called inside measure")
        }
        Some(RuntimePhase::Input) => {
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

    let owner_instance_key = current_component_instance_key_from_scope()
        .unwrap_or_else(|| panic!("receive_frame_nanos requires an active component node context"));
    let key = compute_frame_nanos_receiver_key();

    with_composition_runtime(|runtime| {
        let mut tracker = runtime.frame_clock_tracker.lock();
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
